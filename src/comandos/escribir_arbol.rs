use std::os::unix::fs::PermissionsExt;
use anyhow::Result;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn ejecutar() -> Result<()> {
    let hash = escribir_arbol_directorio(".")?;
    println!("{}", hash);
    Ok(())
}

fn escribir_arbol_directorio(ruta: &str) -> Result<String> {
    let mut entradas = Vec::new();
    
    // Obtener todas las entradas en el directorio
    let ruta_path = Path::new(ruta);
    for entrada in fs::read_dir(ruta_path)? {
        let entrada = entrada?;
        let ruta_absoluta = entrada.path();
        let nombre = entrada.file_name();
        let nombre_str = nombre.to_string_lossy();

        // Evita procesar cualquier cosa dentro de .git/
        if ruta_absoluta.starts_with(".git") || ruta_absoluta.starts_with("target") {
            continue;
        }


        let tipo = entrada.file_type()?;
        let ruta_relativa = if ruta == "." {
            PathBuf::from(nombre.clone())
        } else {
            PathBuf::from(ruta).join(nombre.clone())
        };

        if tipo.is_dir() {
            let hash = escribir_arbol_directorio(&ruta_relativa.to_string_lossy())?;
            entradas.push((nombre_str.to_string(), "40000".to_string(), hash));
        } else if tipo.is_file() {
            let hash = hash_objeto(&ruta_relativa)?;
            let metadata = fs::metadata(&ruta_relativa)?;
            let es_ejecutable = metadata.permissions().mode() & 0o111 != 0;
            let modo = if es_ejecutable { "100755" } else { "100644" };
            entradas.push((nombre_str.to_string(), modo.to_string(), hash));
        }
    }

    
    // Ordenar las entradas por nombre
    entradas.sort_by(|a, b| a.0.cmp(&b.0));
    
    // Construir el contenido del tree
    let mut contenido = Vec::new();
    for (nombre, modo, hash) in entradas {
        // Convertir el hash de hex a bytes
        let mut hash_bytes = [0u8; 20];
        for i in 0..20 {
            let byte = u8::from_str_radix(&hash[i*2..i*2+2], 16)?;
            hash_bytes[i] = byte;
        }
        
        // Agregar la entrada al tree
        write!(contenido, "{} {}\0", modo, nombre)?;
        contenido.extend_from_slice(&hash_bytes);
    }
    
    // Calcular el hash del tree
    let mut hasher = Sha1::new();
    hasher.update(format!("tree {}\0", contenido.len()));
    hasher.update(&contenido);
    let hash = hasher.finalize();
    let hash_str = hash.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    
    // Escribir el objeto tree
    let objeto_dir = format!(".git/objects/{}", &hash_str[..2]);
    fs::create_dir_all(&objeto_dir)?;
    
    let objeto_ruta = format!("{}/{}", objeto_dir, &hash_str[2..]);
    if !Path::new(&objeto_ruta).exists() {
        let objeto_archivo = File::create(&objeto_ruta)?;
        let mut encoder = ZlibEncoder::new(objeto_archivo, Compression::default());
        
        // Escribir el encabezado
        write!(encoder, "tree {}\0", contenido.len())?;
        
        // Escribir el contenido
        encoder.write_all(&contenido)?;
        encoder.finish()?;
    }
    
    Ok(hash_str)
}

fn hash_objeto(ruta: &Path) -> Result<String> {
    // Leer el contenido del archivo
    let contenido = fs::read(ruta)?;
    
    // Calcular el hash SHA-1 del blob
    let mut hasher = Sha1::new();
    hasher.update(format!("blob {}\0", contenido.len()));
    hasher.update(&contenido);
    let hash = hasher.finalize();
    let hash_str = hash.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    
    // Verificar si el objeto ya existe
    let objeto_ruta = format!(".git/objects/{}/{}", &hash_str[..2], &hash_str[2..]);
    if !Path::new(&objeto_ruta).exists() {
        // Crear el directorio de objetos si no existe
        let objeto_dir = format!(".git/objects/{}", &hash_str[..2]);
        fs::create_dir_all(&objeto_dir)?;
        
        // Escribir el objeto blob
        let objeto_archivo = File::create(&objeto_ruta)?;
        let mut encoder = ZlibEncoder::new(objeto_archivo, Compression::default());
        
        // Escribir el encabezado
        write!(encoder, "blob {}\0", contenido.len())?;
        
        // Escribir el contenido
        encoder.write_all(&contenido)?;
        encoder.finish()?;
    }
    
    Ok(hash_str)
}