use anyhow::Result;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn ejecutar(hash_arbol: &str, hash_padre: Option<&str>, mensaje: &str) -> Result<()> {
    // Crear el contenido del commit
    let mut contenido = format!("tree {}\n", hash_arbol);
    
    // Agregar el hash del commit padre si existe
    if let Some(padre) = hash_padre {
        contenido.push_str(&format!("parent {}\n", padre));
    }
    
    // Agregar información del autor y committer
    let autor = obtener_autor()?;
    let timestamp = obtener_timestamp()?;
    contenido.push_str(&format!("author {} {}\n", autor, timestamp));
    contenido.push_str(&format!("committer {} {}\n", autor, timestamp));
    contenido.push_str("\n");
    contenido.push_str(mensaje);
    contenido.push_str("\n");
    
    // Calcular el hash SHA-1 del commit
    let mut hasher = Sha1::new();
    hasher.update(format!("commit {}\0", contenido.len()));
    hasher.update(&contenido);
    let hash = hasher.finalize();
    let hash_str = hash.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    
    // Escribir el objeto commit
    let objeto_dir = format!(".git/objects/{}", &hash_str[..2]);
    fs::create_dir_all(&objeto_dir)?;
    
    let objeto_ruta = format!("{}/{}", objeto_dir, &hash_str[2..]);
    if !Path::new(&objeto_ruta).exists() {
        let objeto_archivo = File::create(&objeto_ruta)?;
        let mut encoder = ZlibEncoder::new(objeto_archivo, Compression::default());
        
        // Escribir el encabezado
        write!(encoder, "commit {}\0", contenido.len())?;
        
        // Escribir el contenido
        encoder.write_all(contenido.as_bytes())?;
        encoder.finish()?;
    }
    
    // Actualizar la referencia HEAD
    actualizar_head(&hash_str)?;
    
    // Imprimir el hash del commit
    println!("{}", hash_str);
    
    Ok(())
}

fn obtener_autor() -> Result<String> {
    // Intentar leer el autor del archivo de configuración de Git
    let config_global = dirs::home_dir()
        .map(|home| home.join(".gitconfig"))
        .filter(|path| path.exists());
    
    if let Some(config_path) = config_global {
        if let Ok(contenido) = fs::read_to_string(config_path) {
            // Buscar nombre y email en el archivo de configuración
            let nombre = contenido.lines()
                .skip_while(|line| !line.contains("[user]"))
                .take_while(|line| !line.contains("["))
                .find_map(|line| {
                    if line.contains("name =") {
                        Some(line.split('=').nth(1)?.trim().to_string())
                    } else {
                        None
                    }
                });
            
            let email = contenido.lines()
                .skip_while(|line| !line.contains("[user]"))
                .take_while(|line| !line.contains("["))
                .find_map(|line| {
                    if line.contains("email =") {
                        Some(line.split('=').nth(1)?.trim().to_string())
                    } else {
                        None
                    }
                });
            
            if let (Some(nombre), Some(email)) = (nombre, email) {
                return Ok(format!("{} <{}>", nombre, email));
            }
        }
    }
    
    // Si no se puede obtener del archivo de configuración, usar un valor predeterminado
    Ok("Usuario Git <usuario@ejemplo.com>".to_string())
}

fn obtener_timestamp() -> Result<String> {
    let segundos = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    
    // Calcular la zona horaria (esto es simplificado, idealmente deberíamos usar una biblioteca para ello)
    let zona_horaria = "+0000";
    
    Ok(format!("{} {}", segundos, zona_horaria))
}

fn actualizar_head(hash_commit: &str) -> Result<()> {
    // Leer el archivo HEAD para determinar la rama actual
    if let Ok(head_contenido) = fs::read_to_string(".git/HEAD") {
        if head_contenido.starts_with("ref: ") {
            // HEAD apunta a una referencia (rama)
            let ref_path = head_contenido.trim_start_matches("ref: ").trim();
            let ref_dir = Path::new(ref_path).parent().unwrap();
            
            // Crear el directorio si no existe
            fs::create_dir_all(format!(".git/{}", ref_dir.display()))?;
            
            // Escribir el hash del commit en la referencia
            fs::write(format!(".git/{}", ref_path), format!("{}\n", hash_commit))?;
        } else {
            // HEAD está en estado detached, solo actualizar HEAD
            fs::write(".git/HEAD", format!("{}\n", hash_commit))?;
        }
    }
    
    Ok(())
} 