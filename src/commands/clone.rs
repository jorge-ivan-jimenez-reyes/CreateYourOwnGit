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

pub(crate) fn invoke(url: &str, target_dir: &Path) -> Result<()> {
    println!("Clonando {} en {}", url, target_dir.display());
    
    // Crear el directorio destino y la estructura .git
    fs::create_dir_all(target_dir)?;
    let git_dir = target_dir.join(".git");
    fs::create_dir_all(&git_dir)?;
    fs::create_dir_all(git_dir.join("objects"))?;
    fs::create_dir_all(git_dir.join("refs/heads"))?;
    fs::create_dir_all(git_dir.join("refs/tags"))?;
    
    // Extraer el nombre del repositorio y el propietario de la URL
    let repo_parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();
    let _repo_name = repo_parts.last().unwrap_or(&"");
    
    // Inicializar el cliente HTTP
    let client = Client::new();
    
    // Obtener información del repositorio (refs)
    println!("Obteniendo información del repositorio...");
    let info_refs_url = format!("{}/info/refs?service=git-upload-pack", url);
    let info_refs_response = client.get(&info_refs_url)
        .header("User-Agent", "git/2.0.0")
        .send()?
        .text()?;
    
    // Parsear la respuesta para obtener las referencias
    let mut refs = HashMap::new();
    let mut default_branch = String::new();
    let mut head_commit = String::new();
    
    for line in info_refs_response.lines().skip(1) {  // Saltamos la primera línea (encabezado)
        if line.is_empty() || line.starts_with('#') || line.starts_with("0000") {
            continue;
        }
        
        // Formato: <longitud en hex><datos>
        let line = &line[4..];  // Quitamos los 4 primeros caracteres (longitud)
        
        if line.contains("refs/heads/") {
            let parts: Vec<&str> = line.split('\0').collect();
            if parts.len() >= 2 {
                let hash = &parts[0][0..40];
                let ref_name = &parts[0][41..];
                refs.insert(ref_name.to_string(), hash.to_string());
                
                if ref_name == "refs/heads/main" || ref_name == "refs/heads/master" {
                    default_branch = ref_name.to_string();
                    head_commit = hash.to_string();
                }
            }
        }
    }
    
    // Si no encontramos main o master, usar la primera rama
    if default_branch.is_empty() && !refs.is_empty() {
        let first_ref = refs.keys().next().unwrap();
        default_branch = first_ref.to_string();
        head_commit = refs.get(first_ref).unwrap().to_string();
    }
    
    if head_commit.is_empty() {
        anyhow::bail!("No se pudo determinar el commit HEAD");
    }
    
    println!("Rama por defecto: {} (commit: {})", default_branch, head_commit);
    
    // Solicitar el packfile
    println!("Descargando objetos...");
    let upload_pack_url = format!("{}/git-upload-pack", url);
    
    // Construir el cuerpo de la solicitud
    let body = format!(
        "0032want {}\n00000009done\n",
        head_commit
    );
    
    let pack_response = client.post(&upload_pack_url)
        .header("Content-Type", "application/x-git-upload-pack-request")
        .header("User-Agent", "git/2.0.0")
        .body(body)
        .send()?
        .bytes()?;
    
    // Procesar el packfile
    process_packfile(&pack_response, &git_dir)?;
    
    // Escribir HEAD
    fs::write(
        git_dir.join("HEAD"),
        format!("ref: {}\n", default_branch),
    )?;
    
    // Escribir la referencia de la rama por defecto
    let ref_path = git_dir.join(&default_branch);
    fs::create_dir_all(ref_path.parent().unwrap())?;
    fs::write(ref_path, format!("{}\n", head_commit))?;
    
    // Checkout del trabajo
    checkout_work_tree(&git_dir, &head_commit, target_dir)?;
    
    println!("Clonación completada con éxito");
    Ok(())
}

fn process_packfile(pack_data: &[u8], git_dir: &Path) -> Result<()> {
    let mut cursor = Cursor::new(pack_data);
    
    // Buscar el inicio del packfile (PACK signature)
    let mut buffer = [0u8; 4];
    let mut pack_start = 0;
    
    while cursor.read_exact(&mut buffer).is_ok() {
        if &buffer == b"PACK" {
            pack_start = cursor.position() - 4;
            break;
        }
        cursor.seek(SeekFrom::Current(-3))?;  // Retroceder 3 bytes para la siguiente búsqueda
    }
    
    // Si no encontramos la firma PACK, es un error
    if pack_start == 0 {
        anyhow::bail!("No se encontró la firma PACK en la respuesta");
    }
    
    // Posicionarnos al inicio del packfile
    cursor.seek(SeekFrom::Start(pack_start))?;
    
    // Leer la cabecera del packfile
    cursor.read_exact(&mut buffer)?;  // "PACK"
    if &buffer != b"PACK" {
        anyhow::bail!("Formato de packfile inválido");
    }
    
    let mut version_buf = [0u8; 4];
    cursor.read_exact(&mut version_buf)?;
    let version = u32::from_be_bytes(version_buf);
    if version != 2 {
        anyhow::bail!("Versión de packfile no soportada: {}", version);
    }
    
    let mut count_buf = [0u8; 4];
    cursor.read_exact(&mut count_buf)?;
    let object_count = u32::from_be_bytes(count_buf);
    
    println!("Procesando packfile: {} objetos", object_count);
    
    // Procesar cada objeto en el packfile
    let mut objects = HashMap::new();
    
    for _ in 0..object_count {
        let (obj_type, obj_data, obj_hash) = read_packed_object(&mut cursor, &objects)?;
        
        // Guardar el objeto en el mapa para referencias futuras
        objects.insert(obj_hash.clone(), (obj_type.clone(), obj_data.clone()));
        
        // Guardar el objeto en el sistema de archivos
        write_git_object(git_dir, &obj_type, &obj_data, &obj_hash)?;
    }
    
    println!("Objetos procesados: {}", objects.len());
    Ok(())
}

fn read_packed_object(cursor: &mut Cursor<&[u8]>, objects: &HashMap<String, (String, Vec<u8>)>) -> Result<(String, Vec<u8>, String)> {
    // Leer el byte de tipo y tamaño
    let mut type_byte = [0u8; 1];
    cursor.read_exact(&mut type_byte)?;
    
    // Los primeros 3 bits son el tipo de objeto
    let obj_type_num = (type_byte[0] >> 4) & 0x7;
    
    // Los 4 bits menos significativos son parte del tamaño
    let mut _size: u64 = (type_byte[0] & 0xF) as u64;
    
    // Si el bit más significativo está activado, hay más bytes para el tamaño
    let mut shift = 4;
    while (type_byte[0] & 0x80) != 0 {
        cursor.read_exact(&mut type_byte)?;
        _size |= ((type_byte[0] & 0x7F) as u64) << shift;
        shift += 7;
    }
    
    // Determinar el tipo de objeto
    let mut obj_type = match obj_type_num {
        1 => "commit".to_string(),
        2 => "tree".to_string(),
        3 => "blob".to_string(),
        4 => "tag".to_string(),
        6 => "ofs-delta".to_string(),
        7 => "ref-delta".to_string(),
        _ => anyhow::bail!("Tipo de objeto desconocido: {}", obj_type_num),
    };
    
    let mut obj_data = Vec::new();
    
    if obj_type == "ofs-delta" {
        // Implementación básica para delta basado en offset
        let mut offset: u64 = 0;
        let mut shift = 0;
        
        loop {
            cursor.read_exact(&mut type_byte)?;
            offset |= ((type_byte[0] & 0x7F) as u64) << shift;
            shift += 7;
            if (type_byte[0] & 0x80) == 0 {
                break;
            }
        }
        
        // Calcular el offset real
        let base_pos = cursor.position() - offset;
        let current_pos = cursor.position();
        
        // Leer el objeto base
        cursor.seek(SeekFrom::Start(base_pos))?;
        let (base_type, base_data, _) = read_packed_object(cursor, objects)?;
        
        // Volver a la posición actual
        cursor.seek(SeekFrom::Start(current_pos))?;
        
        // Leer los datos delta comprimidos
        let mut z = ZlibDecoder::new(cursor);
        let mut delta_data = Vec::new();
        z.read_to_end(&mut delta_data)?;
        
        // Aplicar el delta
        obj_data = apply_delta(&delta_data, &base_data)?;
        obj_type = base_type;
    } else if obj_type == "ref-delta" {
        // Implementación básica para delta basado en referencia
        let mut base_hash = [0u8; 20];
        cursor.read_exact(&mut base_hash)?;
        
        let base_hash_hex = base_hash.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        
        // Buscar el objeto base en nuestro mapa
        if let Some((base_type, base_data)) = objects.get(&base_hash_hex) {
            // Leer los datos delta comprimidos
            let mut z = ZlibDecoder::new(cursor);
            let mut delta_data = Vec::new();
            z.read_to_end(&mut delta_data)?;
            
            // Aplicar el delta
            obj_data = apply_delta(&delta_data, base_data)?;
            obj_type = base_type.clone();
        } else {
            anyhow::bail!("Objeto base no encontrado: {}", base_hash_hex);
        }
    } else {
        // Para objetos normales, simplemente descomprimir
        let mut z = ZlibDecoder::new(cursor);
        z.read_to_end(&mut obj_data)?;
    }
    
    // Calcular el hash del objeto
    let mut hasher = Sha1::new();
    hasher.update(format!("{} {}\0", obj_type, obj_data.len()));
    hasher.update(&obj_data);
    let hash = hasher.finalize();
    let hash_hex = hash.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    
    Ok((obj_type, obj_data, hash_hex))
}

fn apply_delta(delta: &[u8], base: &[u8]) -> Result<Vec<u8>> {
    let mut result = Vec::new();
    let mut i = 0;
    
    // Leer el tamaño del objeto base (formato variable)
    let mut base_size = 0;
    let mut shift = 0;
    while i < delta.len() {
        let byte = delta[i];
        i += 1;
        base_size |= ((byte & 0x7F) as usize) << shift;
        shift += 7;
        if (byte & 0x80) == 0 {
            break;
        }
    }
    
    // Verificar que el tamaño base coincide
    if base_size != base.len() {
        anyhow::bail!("Tamaño base incorrecto en delta");
    }
    
    // Leer el tamaño del objeto resultante
    let mut result_size = 0;
    shift = 0;
    while i < delta.len() {
        let byte = delta[i];
        i += 1;
        result_size |= ((byte & 0x7F) as usize) << shift;
        shift += 7;
        if (byte & 0x80) == 0 {
            break;
        }
    }
    
    // Reservar espacio para el resultado
    result.reserve(result_size);
    
    // Aplicar las instrucciones del delta
    while i < delta.len() {
        let instruction = delta[i];
        i += 1;
        
        if (instruction & 0x80) != 0 {
            // Instrucción de copia desde el objeto base
            let mut offset = 0;
            let mut size = 0;
            
            if (instruction & 0x01) != 0 {
                offset = delta[i] as usize;
                i += 1;
            }
            if (instruction & 0x02) != 0 {
                offset |= (delta[i] as usize) << 8;
                i += 1;
            }
            if (instruction & 0x04) != 0 {
                offset |= (delta[i] as usize) << 16;
                i += 1;
            }
            if (instruction & 0x08) != 0 {
                offset |= (delta[i] as usize) << 24;
                i += 1;
            }
            
            if (instruction & 0x10) != 0 {
                size = delta[i] as usize;
                i += 1;
            }
            if (instruction & 0x20) != 0 {
                size |= (delta[i] as usize) << 8;
                i += 1;
            }
            if (instruction & 0x40) != 0 {
                size |= (delta[i] as usize) << 16;
                i += 1;
            }
            
            // Si el tamaño es 0, usar 0x10000
            if size == 0 {
                size = 0x10000;
            }
            
            // Copiar datos desde el objeto base
            if offset + size > base.len() {
                anyhow::bail!("Delta fuera de límites: offset={}, size={}, base.len()={}", offset, size, base.len());
            }
            result.extend_from_slice(&base[offset..offset + size]);
        } else if instruction != 0 {
            // Instrucción de insertar datos literales
            let size = instruction as usize;
            if i + size > delta.len() {
                anyhow::bail!("Delta fuera de límites en datos literales");
            }
            result.extend_from_slice(&delta[i..i + size]);
            i += size;
        } else {
            anyhow::bail!("Instrucción delta inválida");
        }
    }
    
    if result.len() != result_size {
        anyhow::bail!("Tamaño resultante incorrecto: esperado={}, actual={}", result_size, result.len());
    }
    
    Ok(result)
}

fn write_git_object(git_dir: &Path, obj_type: &str, data: &[u8], hash: &str) -> Result<()> {
    let object_dir = git_dir.join("objects").join(&hash[0..2]);
    fs::create_dir_all(&object_dir)?;
    
    let object_path = object_dir.join(&hash[2..]);
    if object_path.exists() {
        return Ok(());  // El objeto ya existe, no hay que escribirlo
    }
    
    let mut object_file = File::create(&object_path)?;
    let mut encoder = ZlibEncoder::new(&mut object_file, Compression::default());
    
    // Escribir el encabezado
    write!(encoder, "{} {}\0", obj_type, data.len())?;
    
    // Escribir los datos
    encoder.write_all(data)?;
    encoder.finish()?;
    
    Ok(())
}

fn checkout_work_tree(git_dir: &Path, commit_hash: &str, target_dir: &Path) -> Result<()> {
    println!("Realizando checkout del commit {}", commit_hash);
    
    // Leer el objeto commit
    let commit_path = git_dir.join("objects").join(&commit_hash[0..2]).join(&commit_hash[2..]);
    let commit_file = File::open(commit_path)?;
    let mut z = ZlibDecoder::new(commit_file);
    let mut commit_data = String::new();
    z.read_to_string(&mut commit_data)?;
    
    // Extraer el hash del tree
    let tree_line = commit_data.lines()
        .find(|line| line.starts_with("tree "))
        .context("No se encontró la línea 'tree' en el commit")?;
    
    let tree_hash = tree_line.split_whitespace().nth(1).unwrap();
    
    // Checkout del tree
    checkout_tree(git_dir, tree_hash, target_dir, "")?;
    
    Ok(())
}

fn checkout_tree(git_dir: &Path, tree_hash: &str, target_dir: &Path, prefix: &str) -> Result<()> {
    // Leer el objeto tree
    let tree_path = git_dir.join("objects").join(&tree_hash[0..2]).join(&tree_hash[2..]);
    let tree_file = File::open(tree_path)?;
    let mut z = ZlibDecoder::new(BufReader::new(tree_file));
    
    // Leer y descartar el encabezado (tree <size>\0)
    let mut header = Vec::new();
    let mut buf_reader = BufReader::new(&mut z);
    buf_reader.read_until(0, &mut header)?;
    
    // Leer las entradas del tree
    loop {
        // Leer el modo y nombre
        let mut mode_name = Vec::new();
        let n = buf_reader.read_until(0, &mut mode_name)?;
        if n == 0 {
            break;  // Fin del tree
        }
        
        // Separar modo y nombre
        let mode_name = String::from_utf8_lossy(&mode_name[0..mode_name.len()-1]);
        let space_pos = mode_name.find(' ').unwrap();
        let mode = &mode_name[0..space_pos];
        let name = &mode_name[space_pos+1..];
        
        // Leer el hash del objeto
        let mut hash_bytes = [0u8; 20];
        buf_reader.read_exact(&mut hash_bytes)?;
        let hash = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        
        // Construir la ruta completa
        let path_str = if prefix.is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", prefix, name)
        };
        let path = target_dir.join(&path_str);
        
        if mode.starts_with("10") {
            // Es un archivo
            let blob_path = git_dir.join("objects").join(&hash[0..2]).join(&hash[2..]);
            let blob_file = File::open(blob_path)?;
            let mut blob_z = ZlibDecoder::new(blob_file);
            
            // Leer y descartar el encabezado
            let mut blob_header = Vec::new();
            let mut blob_buf_reader = BufReader::new(&mut blob_z);
            blob_buf_reader.read_until(0, &mut blob_header)?;
            
            // Leer el contenido del blob
            let mut content = Vec::new();
            blob_buf_reader.read_to_end(&mut content)?;
            
            // Crear directorios padre si es necesario
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Escribir el archivo
            fs::write(&path, content)?;
            
            // Establecer permisos si es ejecutable
            if mode == "100755" {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&path, perms)?;
                }
            }
        } else if mode == "40000" {
            // Es un directorio
            fs::create_dir_all(&path)?;
            checkout_tree(git_dir, &hash, target_dir, &path_str)?;
        } else if mode == "120000" {
            // Es un symlink
            let blob_path = git_dir.join("objects").join(&hash[0..2]).join(&hash[2..]);
            let blob_file = File::open(blob_path)?;
            let mut blob_z = ZlibDecoder::new(blob_file);
            
            // Leer y descartar el encabezado
            let mut blob_header = Vec::new();
            let mut blob_buf_reader = BufReader::new(&mut blob_z);
            blob_buf_reader.read_until(0, &mut blob_header)?;
            
            // Leer el contenido del blob (destino del symlink)
            let mut content = Vec::new();
            blob_buf_reader.read_to_end(&mut content)?;
            let target = String::from_utf8_lossy(&content);
            
            // Crear directorios padre si es necesario
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Crear el symlink
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(target.as_ref(), &path)?;
            }
            #[cfg(not(unix))]
            {
                // En sistemas no Unix, simplemente escribir el contenido
                fs::write(&path, content)?;
            }
        }
    }
    
    Ok(())
} 