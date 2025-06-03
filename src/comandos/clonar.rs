use anyhow::{Context, Result};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use reqwest::blocking::Client;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

pub(crate) fn ejecutar(url: &str, directorio_destino: &Path) -> Result<()> {
    println!("Clonando {} en {}", url, directorio_destino.display());
    
    // Crear el directorio destino y la estructura .git
    fs::create_dir_all(directorio_destino)?;
    let directorio_git = directorio_destino.join(".git");
    fs::create_dir_all(&directorio_git)?;
    fs::create_dir_all(directorio_git.join("objects"))?;
    fs::create_dir_all(directorio_git.join("refs/heads"))?;
    fs::create_dir_all(directorio_git.join("refs/tags"))?;
    
    // Extraer el nombre del repositorio y el propietario de la URL
    let partes_repo: Vec<&str> = url.trim_end_matches('/').split('/').collect();
    let _nombre_repo = partes_repo.last().unwrap_or(&"");
    
    // Inicializar el cliente HTTP
    let cliente = Client::new();
    
    // Obtener información del repositorio (refs)
    println!("Obteniendo información del repositorio...");
    let url_info_refs = format!("{}/info/refs?service=git-upload-pack", url);
    let respuesta_info_refs = cliente.get(&url_info_refs)
        .header("User-Agent", "git/2.0.0")
        .send()?
        .text()?;
    
    // Parsear la respuesta para obtener las referencias
    let mut referencias = HashMap::new();
    let mut rama_predeterminada = String::new();
    let mut commit_head = String::new();
    
    for linea in respuesta_info_refs.lines().skip(1) {  // Saltamos la primera línea (encabezado)
        if linea.is_empty() || linea.starts_with('#') || linea.starts_with("0000") {
            continue;
        }
        
        // Formato: <longitud en hex><datos>
        let linea = &linea[4..];  // Quitamos los 4 primeros caracteres (longitud)
        
        if linea.contains("refs/heads/") {
            let partes: Vec<&str> = linea.split('\0').collect();
            if partes.len() >= 2 {
                let hash = &partes[0][0..40];
                let nombre_ref = &partes[0][41..];
                referencias.insert(nombre_ref.to_string(), hash.to_string());
                
                if nombre_ref == "refs/heads/main" || nombre_ref == "refs/heads/master" {
                    rama_predeterminada = nombre_ref.to_string();
                    commit_head = hash.to_string();
                }
            }
        }
    }
    
    // Si no encontramos main o master, usar la primera rama
    if rama_predeterminada.is_empty() && !referencias.is_empty() {
        let primera_ref = referencias.keys().next().unwrap();
        rama_predeterminada = primera_ref.to_string();
        commit_head = referencias.get(primera_ref).unwrap().to_string();
    }
    
    if commit_head.is_empty() {
        anyhow::bail!("No se pudo determinar el commit HEAD");
    }
    
    println!("Rama por defecto: {} (commit: {})", rama_predeterminada, commit_head);
    
    // Solicitar el packfile
    println!("Descargando objetos...");
    let url_upload_pack = format!("{}/git-upload-pack", url);
    
    // Construir el cuerpo de la solicitud
    let cuerpo = format!(
        "0032want {}\n00000009done\n",
        commit_head
    );
    
    let respuesta_pack = cliente.post(&url_upload_pack)
        .header("Content-Type", "application/x-git-upload-pack-request")
        .header("User-Agent", "git/2.0.0")
        .body(cuerpo)
        .send()?
        .bytes()?;
    
    // Procesar el packfile
    procesar_packfile(&respuesta_pack, &directorio_git)?;
    
    // Escribir HEAD
    fs::write(
        directorio_git.join("HEAD"),
        format!("ref: {}\n", rama_predeterminada),
    )?;
    
    // Escribir la referencia de la rama por defecto
    let ruta_ref = directorio_git.join(&rama_predeterminada);
    fs::create_dir_all(ruta_ref.parent().unwrap())?;
    fs::write(ruta_ref, format!("{}\n", commit_head))?;
    
    // Checkout del trabajo
    checkout_arbol_trabajo(&directorio_git, &commit_head, directorio_destino)?;
    
    println!("Clonación completada con éxito");
    Ok(())
}

fn procesar_packfile(datos_pack: &[u8], directorio_git: &Path) -> Result<()> {
    let mut cursor = Cursor::new(datos_pack);
    
    // Buscar el inicio del packfile (PACK signature)
    let mut buffer = [0u8; 4];
    let mut inicio_pack = 0;
    
    while cursor.read_exact(&mut buffer).is_ok() {
        if &buffer == b"PACK" {
            inicio_pack = cursor.position() - 4;
            break;
        }
        cursor.seek(SeekFrom::Current(-3))?;  // Retroceder 3 bytes para la siguiente búsqueda
    }
    
    // Si no encontramos la firma PACK, es un error
    if inicio_pack == 0 {
        anyhow::bail!("No se encontró la firma PACK en la respuesta");
    }
    
    // Posicionarnos al inicio del packfile
    cursor.seek(SeekFrom::Start(inicio_pack))?;
    
    // Leer la cabecera del packfile
    cursor.read_exact(&mut buffer)?;  // "PACK"
    if &buffer != b"PACK" {
        anyhow::bail!("Formato de packfile inválido");
    }
    
    let mut buffer_version = [0u8; 4];
    cursor.read_exact(&mut buffer_version)?;
    let version = u32::from_be_bytes(buffer_version);
    if version != 2 {
        anyhow::bail!("Versión de packfile no soportada: {}", version);
    }
    
    let mut buffer_contador = [0u8; 4];
    cursor.read_exact(&mut buffer_contador)?;
    let cantidad_objetos = u32::from_be_bytes(buffer_contador);
    
    println!("Procesando packfile: {} objetos", cantidad_objetos);
    
    // Procesar cada objeto en el packfile
    let mut objetos = HashMap::new();
    
    for _ in 0..cantidad_objetos {
        let (tipo_obj, datos_obj, hash_obj) = leer_objeto_empacado(&mut cursor, &objetos)?;
        
        // Guardar el objeto en el mapa para referencias futuras
        objetos.insert(hash_obj.clone(), (tipo_obj.clone(), datos_obj.clone()));
        
        // Guardar el objeto en el sistema de archivos
        escribir_objeto_git(directorio_git, &tipo_obj, &datos_obj, &hash_obj)?;
    }
    
    println!("Objetos procesados: {}", objetos.len());
    Ok(())
}

fn leer_objeto_empacado(cursor: &mut Cursor<&[u8]>, objetos: &HashMap<String, (String, Vec<u8>)>) -> Result<(String, Vec<u8>, String)> {
    // Leer el byte de tipo y tamaño
    let mut byte_tipo = [0u8; 1];
    cursor.read_exact(&mut byte_tipo)?;
    
    // Los primeros 3 bits son el tipo de objeto
    let num_tipo_obj = (byte_tipo[0] >> 4) & 0x7;
    
    // Los 4 bits menos significativos son parte del tamaño
    let mut _tamaño: u64 = (byte_tipo[0] & 0xF) as u64;
    
    // Si el bit más significativo está activado, hay más bytes para el tamaño
    let mut desplazamiento = 4;
    while (byte_tipo[0] & 0x80) != 0 {
        cursor.read_exact(&mut byte_tipo)?;
        _tamaño |= ((byte_tipo[0] & 0x7F) as u64) << desplazamiento;
        desplazamiento += 7;
    }
    
    // Determinar el tipo de objeto
    let mut tipo_obj = match num_tipo_obj {
        1 => "commit".to_string(),
        2 => "tree".to_string(),
        3 => "blob".to_string(),
        4 => "tag".to_string(),
        6 => "ofs-delta".to_string(),
        7 => "ref-delta".to_string(),
        _ => anyhow::bail!("Tipo de objeto desconocido: {}", num_tipo_obj),
    };
    
    let mut datos_obj = Vec::new();
    
    if tipo_obj == "ofs-delta" {
        // Implementación básica para delta basado en offset
        let mut offset: u64 = 0;
        let mut desplazamiento = 0;
        
        loop {
            cursor.read_exact(&mut byte_tipo)?;
            offset |= ((byte_tipo[0] & 0x7F) as u64) << desplazamiento;
            desplazamiento += 7;
            if (byte_tipo[0] & 0x80) == 0 {
                break;
            }
        }
        
        // Calcular el offset real
        let pos_base = cursor.position() - offset;
        let pos_actual = cursor.position();
        
        // Leer el objeto base
        cursor.seek(SeekFrom::Start(pos_base))?;
        let (tipo_base, datos_base, _) = leer_objeto_empacado(cursor, objetos)?;
        
        // Volver a la posición actual
        cursor.seek(SeekFrom::Start(pos_actual))?;
        
        // Leer los datos delta comprimidos
        let mut z = ZlibDecoder::new(cursor);
        let mut datos_delta = Vec::new();
        z.read_to_end(&mut datos_delta)?;
        
        // Aplicar el delta
        datos_obj = aplicar_delta(&datos_delta, &datos_base)?;
        tipo_obj = tipo_base;
    } else if tipo_obj == "ref-delta" {
        // Implementación básica para delta basado en referencia
        let mut hash_base = [0u8; 20];
        cursor.read_exact(&mut hash_base)?;
        
        let hash_base_hex = hash_base.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        
        // Buscar el objeto base en nuestro mapa
        if let Some((tipo_base, datos_base)) = objetos.get(&hash_base_hex) {
            // Leer los datos delta comprimidos
            let mut z = ZlibDecoder::new(cursor);
            let mut datos_delta = Vec::new();
            z.read_to_end(&mut datos_delta)?;
            
            // Aplicar el delta
            datos_obj = aplicar_delta(&datos_delta, datos_base)?;
            tipo_obj = tipo_base.clone();
        } else {
            anyhow::bail!("Objeto base no encontrado: {}", hash_base_hex);
        }
    } else {
        // Para objetos normales, simplemente descomprimir
        let mut z = ZlibDecoder::new(cursor);
        z.read_to_end(&mut datos_obj)?;
    }
    
    // Calcular el hash del objeto
    let mut hasher = Sha1::new();
    hasher.update(format!("{} {}\0", tipo_obj, datos_obj.len()));
    hasher.update(&datos_obj);
    let hash = hasher.finalize();
    let hash_hex = hash.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    
    Ok((tipo_obj, datos_obj, hash_hex))
}

fn aplicar_delta(delta: &[u8], base: &[u8]) -> Result<Vec<u8>> {
    let mut resultado = Vec::new();
    let mut i = 0;
    
    // Leer el tamaño del objeto base (formato variable)
    let mut tamaño_base = 0;
    let mut desplazamiento = 0;
    while i < delta.len() {
        let byte = delta[i];
        i += 1;
        tamaño_base |= ((byte & 0x7F) as usize) << desplazamiento;
        desplazamiento += 7;
        if (byte & 0x80) == 0 {
            break;
        }
    }
    
    // Verificar que el tamaño base coincide
    if tamaño_base != base.len() {
        anyhow::bail!("Tamaño base incorrecto en delta");
    }
    
    // Leer el tamaño del objeto resultante
    let mut tamaño_resultado = 0;
    desplazamiento = 0;
    while i < delta.len() {
        let byte = delta[i];
        i += 1;
        tamaño_resultado |= ((byte & 0x7F) as usize) << desplazamiento;
        desplazamiento += 7;
        if (byte & 0x80) == 0 {
            break;
        }
    }
    
    // Reservar espacio para el resultado
    resultado.reserve(tamaño_resultado);
    
    // Aplicar las instrucciones del delta
    while i < delta.len() {
        let instruccion = delta[i];
        i += 1;
        
        if (instruccion & 0x80) != 0 {
            // Instrucción de copia desde el objeto base
            let mut offset = 0;
            let mut tamaño = 0;
            
            if (instruccion & 0x01) != 0 {
                offset = delta[i] as usize;
                i += 1;
            }
            if (instruccion & 0x02) != 0 {
                offset |= (delta[i] as usize) << 8;
                i += 1;
            }
            if (instruccion & 0x04) != 0 {
                offset |= (delta[i] as usize) << 16;
                i += 1;
            }
            if (instruccion & 0x08) != 0 {
                offset |= (delta[i] as usize) << 24;
                i += 1;
            }
            
            if (instruccion & 0x10) != 0 {
                tamaño = delta[i] as usize;
                i += 1;
            }
            if (instruccion & 0x20) != 0 {
                tamaño |= (delta[i] as usize) << 8;
                i += 1;
            }
            if (instruccion & 0x40) != 0 {
                tamaño |= (delta[i] as usize) << 16;
                i += 1;
            }
            
            // Si el tamaño es 0, usar 0x10000
            if tamaño == 0 {
                tamaño = 0x10000;
            }
            
            // Copiar datos desde el objeto base
            if offset + tamaño > base.len() {
                anyhow::bail!("Delta fuera de límites: offset={}, tamaño={}, base.len()={}", offset, tamaño, base.len());
            }
            resultado.extend_from_slice(&base[offset..offset + tamaño]);
        } else if instruccion != 0 {
            // Instrucción de insertar datos literales
            let tamaño = instruccion as usize;
            if i + tamaño > delta.len() {
                anyhow::bail!("Delta fuera de límites en datos literales");
            }
            resultado.extend_from_slice(&delta[i..i + tamaño]);
            i += tamaño;
        } else {
            anyhow::bail!("Instrucción delta inválida");
        }
    }
    
    if resultado.len() != tamaño_resultado {
        anyhow::bail!("Tamaño resultante incorrecto: esperado={}, actual={}", tamaño_resultado, resultado.len());
    }
    
    Ok(resultado)
}

fn escribir_objeto_git(directorio_git: &Path, tipo_obj: &str, datos: &[u8], hash: &str) -> Result<()> {
    let directorio_objeto = directorio_git.join("objects").join(&hash[0..2]);
    fs::create_dir_all(&directorio_objeto)?;
    
    let ruta_objeto = directorio_objeto.join(&hash[2..]);
    if ruta_objeto.exists() {
        return Ok(());  // El objeto ya existe, no hay que escribirlo
    }
    
    let mut archivo_objeto = File::create(&ruta_objeto)?;
    let mut encoder = ZlibEncoder::new(&mut archivo_objeto, Compression::default());
    
    // Escribir el encabezado
    write!(encoder, "{} {}\0", tipo_obj, datos.len())?;
    
    // Escribir los datos
    encoder.write_all(datos)?;
    encoder.finish()?;
    
    Ok(())
}

fn checkout_arbol_trabajo(directorio_git: &Path, hash_commit: &str, directorio_destino: &Path) -> Result<()> {
    println!("Realizando checkout del commit {}", hash_commit);
    
    // Leer el objeto commit
    let ruta_commit = directorio_git.join("objects").join(&hash_commit[0..2]).join(&hash_commit[2..]);
    let archivo_commit = File::open(ruta_commit)?;
    let mut z = ZlibDecoder::new(archivo_commit);
    let mut datos_commit = String::new();
    z.read_to_string(&mut datos_commit)?;
    
    // Extraer el hash del tree
    let linea_tree = datos_commit.lines()
        .find(|linea| linea.starts_with("tree "))
        .context("No se encontró la línea 'tree' en el commit")?;
    
    let hash_tree = linea_tree.split_whitespace().nth(1).unwrap();
    
    // Checkout del tree
    checkout_arbol(directorio_git, hash_tree, directorio_destino, "")?;
    
    Ok(())
}

fn checkout_arbol(directorio_git: &Path, hash_tree: &str, directorio_destino: &Path, prefijo: &str) -> Result<()> {
    // Leer el objeto tree
    let ruta_tree = directorio_git.join("objects").join(&hash_tree[0..2]).join(&hash_tree[2..]);
    let archivo_tree = File::open(ruta_tree)?;
    let mut z = ZlibDecoder::new(BufReader::new(archivo_tree));
    
    // Leer y descartar el encabezado (tree <size>\0)
    let mut cabecera = Vec::new();
    let mut lector_buf = BufReader::new(&mut z);
    lector_buf.read_until(0, &mut cabecera)?;
    
    // Leer las entradas del tree
    loop {
        // Leer el modo y nombre
        let mut modo_nombre = Vec::new();
        let n = lector_buf.read_until(0, &mut modo_nombre)?;
        if n == 0 {
            break;  // Fin del tree
        }
        
        // Separar modo y nombre
        let modo_nombre = String::from_utf8_lossy(&modo_nombre[0..modo_nombre.len()-1]);
        let pos_espacio = modo_nombre.find(' ').unwrap();
        let modo = &modo_nombre[0..pos_espacio];
        let nombre = &modo_nombre[pos_espacio+1..];
        
        // Leer el hash del objeto
        let mut bytes_hash = [0u8; 20];
        lector_buf.read_exact(&mut bytes_hash)?;
        let hash = bytes_hash.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        
        // Construir la ruta completa
        let ruta_str = if prefijo.is_empty() {
            nombre.to_string()
        } else {
            format!("{}/{}", prefijo, nombre)
        };
        let ruta = directorio_destino.join(&ruta_str);
        
        if modo.starts_with("10") {
            // Es un archivo
            let ruta_blob = directorio_git.join("objects").join(&hash[0..2]).join(&hash[2..]);
            let archivo_blob = File::open(ruta_blob)?;
            let mut blob_z = ZlibDecoder::new(archivo_blob);
            
            // Leer y descartar el encabezado
            let mut cabecera_blob = Vec::new();
            let mut lector_buf_blob = BufReader::new(&mut blob_z);
            lector_buf_blob.read_until(0, &mut cabecera_blob)?;
            
            // Leer el contenido del blob
            let mut contenido = Vec::new();
            lector_buf_blob.read_to_end(&mut contenido)?;
            
            // Crear directorios padre si es necesario
            if let Some(padre) = ruta.parent() {
                fs::create_dir_all(padre)?;
            }
            
            // Escribir el archivo
            fs::write(&ruta, contenido)?;
            
            // Establecer permisos si es ejecutable
            if modo == "100755" {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut permisos = fs::metadata(&ruta)?.permissions();
                    permisos.set_mode(0o755);
                    fs::set_permissions(&ruta, permisos)?;
                }
            }
        } else if modo == "40000" {
            // Es un directorio
            fs::create_dir_all(&ruta)?;
            checkout_arbol(directorio_git, &hash, directorio_destino, &ruta_str)?;
        } else if modo == "120000" {
            // Es un symlink
            let ruta_blob = directorio_git.join("objects").join(&hash[0..2]).join(&hash[2..]);
            let archivo_blob = File::open(ruta_blob)?;
            let mut blob_z = ZlibDecoder::new(archivo_blob);
            
            // Leer y descartar el encabezado
            let mut cabecera_blob = Vec::new();
            let mut lector_buf_blob = BufReader::new(&mut blob_z);
            lector_buf_blob.read_until(0, &mut cabecera_blob)?;
            
            // Leer el contenido del blob (destino del symlink)
            let mut contenido = Vec::new();
            lector_buf_blob.read_to_end(&mut contenido)?;
            let destino = String::from_utf8_lossy(&contenido);
            
            // Crear directorios padre si es necesario
            if let Some(padre) = ruta.parent() {
                fs::create_dir_all(padre)?;
            }
            
            // Crear el symlink
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(destino.as_ref(), &ruta)?;
            }
            #[cfg(not(unix))]
            {
                // En sistemas no Unix, simplemente escribir el contenido
                fs::write(&ruta, contenido)?;
            }
        }
    }
    
    Ok(())
} 