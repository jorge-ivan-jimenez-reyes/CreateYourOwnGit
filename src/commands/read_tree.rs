use crate::objects::{Kind, Object};
use anyhow::Context;
use std::{
    ffi::CStr,
    io::{BufRead, Read},
};

pub(crate) fn invoke(tree_hash: &str) -> anyhow::Result<()> {
    let mut object = Object::read(tree_hash).context("parse out tree object file")?;
    
    if object.kind != Kind::Tree {
        anyhow::bail!("El objeto {} no es un árbol, es un {}", tree_hash, object.kind);
    }
    
    println!("Leyendo objeto tree: {}", tree_hash);
    println!("Tamaño: {} bytes", object.expected_size);
    println!("Contenido:");
    
    let mut buf = Vec::new();
    let mut hashbuf = [0; 20];
    let mut entry_count = 0;
    
    loop {
        buf.clear();
        let n = object
            .reader
            .read_until(0, &mut buf)
            .context("leer entrada del objeto tree")?;
        
        if n == 0 {
            break;
        }
        
        object
            .reader
            .read_exact(&mut hashbuf[..])
            .context("leer hash del objeto en la entrada del tree")?;
        
        let mode_and_name = CStr::from_bytes_with_nul(&buf).context("entrada de tree inválida")?;
        let mut bits = mode_and_name.to_bytes().splitn(2, |&b| b == b' ');
        let mode = bits.next().expect("split always yields once");
        let name = bits
            .next()
            .ok_or_else(|| anyhow::anyhow!("la entrada del tree no tiene nombre de archivo"))?;
        
        let mode_str = std::str::from_utf8(mode).context("el modo siempre es UTF-8 válido")?;
        let name_str = std::str::from_utf8(name).context("el nombre debería ser UTF-8 válido")?;
        let hash_hex = hex::encode(&hashbuf);
        
        entry_count += 1;
        
        // Determinar el tipo de objeto
        let object_type = match Object::read(&hash_hex) {
            Ok(obj) => format!("{}", obj.kind),
            Err(_) => "desconocido".to_string(),
        };
        
        println!("Entrada {}: ", entry_count);
        println!("  Modo: {}", mode_str);
        println!("  Tipo: {}", object_type);
        println!("  Hash: {}", hash_hex);
        println!("  Nombre: {}", name_str);
    }
    
    println!("Total de entradas: {}", entry_count);
    
    Ok(())
} 