use anyhow::{Context, Result};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub fn ejecutar(escribir: bool, ruta: &Path) -> Result<()> {
    // Leer el contenido del archivo
    let mut archivo = File::open(ruta).context("No se pudo abrir el archivo")?;
    let mut contenido = Vec::new();
    archivo.read_to_end(&mut contenido)?;
    
    // Calcular el hash SHA-1 del objeto
    let mut hasher = Sha1::new();
    hasher.update(format!("blob {}\0", contenido.len()));
    hasher.update(&contenido);
    let hash = hasher.finalize();
    let hash_str = format!("{:x}", hash);
    
    // Si la bandera -w está establecida, escribir el objeto en .git/objects
    if escribir {
        // Crear el directorio de objetos si no existe
        let objeto_dir = format!(".git/objects/{}", &hash_str[..2]);
        fs::create_dir_all(&objeto_dir)?;
        
        // Abrir el archivo de objeto para escritura
        let objeto_ruta = format!("{}/{}", objeto_dir, &hash_str[2..]);
        let objeto_archivo = File::create(&objeto_ruta)?;
        
        // Comprimir y escribir el objeto
        let mut encoder = ZlibEncoder::new(objeto_archivo, Compression::default());
        write!(encoder, "blob {}\0", contenido.len())?;
        encoder.write_all(&contenido)?;
        encoder.finish()?;
    }
    
    // Imprimir el hash
    println!("{}", hash_str);
    
    Ok(())
} 