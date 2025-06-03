use anyhow::Result;
use std::io::{BufReader, Read};
use flate2::read::ZlibDecoder;
use std::fs::File;

pub fn ejecutar(solo_nombres: bool, hash_arbol: &str) -> Result<()> {
    // Abrir el objeto tree
    let ruta_arbol = format!(".git/objects/{}/{}", &hash_arbol[..2], &hash_arbol[2..]);
    let archivo = File::open(&ruta_arbol)?;
    let decodificador = ZlibDecoder::new(archivo);
    let mut lector = BufReader::new(decodificador);
    
    // Leer y descartar el encabezado (tree <size>\0)
    let mut cabecera = Vec::new();
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
    
    // Leer las entradas del tree
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
        
        if solo_nombres {
            println!("{}", nombre_str);
        } else {
            println!("{} {} {}\t{}", modo_str, tipo_modo(&modo_str), hash_str, nombre_str);
        }
    }
    
    Ok(())
}

fn tipo_modo(modo: &str) -> &'static str {
    match modo {
        "100644" => "blob",
        "100755" => "blob ejecutable",
        "120000" => "symlink",
        "040000" => "tree",
        "160000" => "submodulo",
        _ => "desconocido",
    }
} 