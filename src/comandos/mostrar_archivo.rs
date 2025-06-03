use crate::objetos::{Objeto, Tipo};
use anyhow::Result;
use std::io::Read;

pub fn ejecutar(mostrar_bonito: bool, hash_objeto: &str) -> Result<()> {
    if !mostrar_bonito {
        anyhow::bail!("Solo se admite la opción -p");
    }

    let mut objeto = Objeto::leer(hash_objeto)?;
    if objeto.tipo == Tipo::Blob {
        let mut contenido = Vec::new();
        objeto.lector.read_to_end(&mut contenido)?;
        std::io::stdout().write_all(&contenido)?;
    } else {
        anyhow::bail!("Solo se admiten objetos de tipo blob");
    }

    Ok(())
} 