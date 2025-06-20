use anyhow::{Context, Result};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) struct DatosCommit {
    pub(crate) hash_arbol: String,
    pub(crate) hash_padre: Option<String>,
    pub(crate) mensaje: String,
}

pub(crate) fn crear_commit(datos: &DatosCommit) -> Result<String> {
    // Generar el contenido del commit
    let mut contenido = format!("tree {}\n", datos.hash_arbol);
    
    // Agregar la referencia al padre si existe
    if let Some(ref hash_padre) = datos.hash_padre {
        contenido.push_str(&format!("parent {}\n", hash_padre));
    }
    
    // Información del autor y committer
    let autor = obtener_autor()?;
    let timestamp = obtener_timestamp()?;
    
    contenido.push_str(&format!("author {} {}\n", autor, timestamp));
    contenido.push_str(&format!("committer {} {}\n", autor, timestamp));
    contenido.push_str("\n");
    contenido.push_str(&datos.mensaje);
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
    
    Ok(hash_str)
}

fn obtener_autor() -> Result<String> {
    // Intentar obtener el nombre y email de la configuración de Git
    let config_global = dirs::home_dir()
        .map(|home| home.join(".gitconfig"))
        .filter(|path| path.exists());
    
    if let Some(config_path) = config_global {
        if let Ok(contenido) = fs::read_to_string(config_path) {
            // Buscar la sección [user]
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
    
    // Si no se pudo obtener de la configuración, usar un valor predeterminado
    Ok("Usuario Git <usuario@ejemplo.com>".to_string())
}

fn obtener_timestamp() -> Result<String> {
    let segundos = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    
    // Por simplicidad, usamos UTC (zona horaria +0000)
    let zona_horaria = "+0000";
    
    Ok(format!("{} {}", segundos, zona_horaria))
}

fn actualizar_head(hash_commit: &str) -> Result<()> {
    // Leer el archivo HEAD para determinar a qué referencia apunta
    let head_contenido = fs::read_to_string(".git/HEAD")
        .context("No se pudo leer el archivo .git/HEAD")?;
    
    if head_contenido.starts_with("ref: ") {
        // HEAD apunta a una referencia (rama)
        let ref_path = head_contenido.trim_start_matches("ref: ").trim();
        let ref_dir = Path::new(ref_path).parent()
            .context("Formato de referencia inválido en HEAD")?;
        
        // Crear el directorio si no existe
        fs::create_dir_all(format!(".git/{}", ref_dir.display()))?;
        
        // Escribir el hash del commit en la referencia
        fs::write(format!(".git/{}", ref_path), format!("{}\n", hash_commit))?;
    } else {
        // HEAD está en estado detached, solo actualizar HEAD
        fs::write(".git/HEAD", format!("{}\n", hash_commit))?;
    }
    
    Ok(())
}
