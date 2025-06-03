use anyhow::Result;
use flate2::read::ZlibDecoder;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

pub fn ejecutar(hash_arbol: &str) -> Result<()> {
    // Leer el objeto tree
    let ruta_arbol = format!(".git/objects/{}/{}", &hash_arbol[..2], &hash_arbol[2..]);
    let archivo = File::open(&ruta_arbol)?;
    let decodificador = ZlibDecoder::new(archivo);
    let mut lector = BufReader::new(decodificador);
    
    // Leer y descartar el encabezado (tree <size>\0)
    let mut cabecera: Vec<u8> = Vec::new();
    while let Ok(byte) = lector.by_ref().bytes().next().transpose() {
        if let Some(byte) = byte {
            cabecera.push(byte);
            if byte == 0 {
                break;
            }
        } else {
            break;
        }
    }
    
    // Verificar que es un tree
    let cabecera = String::from_utf8_lossy(&cabecera[..cabecera.len()-1]);
    if !cabecera.starts_with("tree ") {
        anyhow::bail!("El objeto no es un tree: {}", cabecera);
    }
    
    // Leer las entradas del tree y extraerlas
    extraer_arbol(&mut lector, Path::new("."))?;
    
    Ok(())
}

fn extraer_arbol(lector: &mut impl Read, base_path: &Path) -> Result<()> {
    let mut buffer = [0u8; 1];
    
    while lector.read_exact(&mut buffer).is_ok() {
        let mut modo = Vec::new();
        modo.push(buffer[0]);
        
        // Leer el resto del modo
        while let Ok(byte) = lector.by_ref().bytes().next().transpose() {
            if let Some(byte) = byte {
                if byte == b' ' {
                    break;
                }
                modo.push(byte);
            } else {
                break;
            }
        }
        
        // Leer el nombre
        let mut nombre = Vec::new();
        while let Ok(byte) = lector.by_ref().bytes().next().transpose() {
            if let Some(byte) = byte {
                if byte == 0 {
                    break;
                }
                nombre.push(byte);
            } else {
                break;
            }
        }
        
        // Leer el hash (20 bytes)
        let mut hash = [0u8; 20];
        if lector.read_exact(&mut hash).is_err() {
            break;
        }
        
        let modo_str = String::from_utf8_lossy(&modo);
        let nombre_str = String::from_utf8_lossy(&nombre);
        let hash_str = hash.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        
        let ruta = base_path.join(&*nombre_str);
        
        if modo_str == "40000" {
            // Es un directorio, extraer recursivamente
            fs::create_dir_all(&ruta)?;
            
            // Leer el objeto tree
            let ruta_objeto = format!(".git/objects/{}/{}", &hash_str[..2], &hash_str[2..]);
            let archivo = File::open(&ruta_objeto)?;
            let decodificador = ZlibDecoder::new(archivo);
            let mut lector_sub = BufReader::new(decodificador);
            
            // Leer y descartar el encabezado
            let mut cabecera: Vec<u8> = Vec::new();
            while let Ok(byte) = lector_sub.by_ref().bytes().next().transpose() {
                if let Some(byte) = byte {
                    if byte == 0 {
                        break;
                    }
                } else {
                    break;
                }
            }
            
            extraer_arbol(&mut lector_sub, &ruta)?;
        } else {
            // Es un archivo, extraer el contenido
            extraer_blob(&hash_str, &ruta)?;
        }
    }
    
    Ok(())
}

fn extraer_blob(hash: &str, ruta: &Path) -> Result<()> {
    let ruta_objeto = format!(".git/objects/{}/{}", &hash[..2], &hash[2..]);
    let archivo = File::open(&ruta_objeto)?;
    let decodificador = ZlibDecoder::new(archivo);
    let mut lector = BufReader::new(decodificador);
    
    // Leer y descartar el encabezado
    let mut cabecera: Vec<u8> = Vec::new();
    while let Ok(byte) = lector.by_ref().bytes().next().transpose() {
        if let Some(byte) = byte {
            if byte == 0 {
                break;
            }
        } else {
            break;
        }
    }
    
    // Leer el contenido y escribirlo al archivo
    let mut contenido = Vec::new();
    lector.read_to_end(&mut contenido)?;
    fs::write(ruta, contenido)?;
    
    Ok(())
} 