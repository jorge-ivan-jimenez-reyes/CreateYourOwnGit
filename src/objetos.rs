use anyhow::Context;
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::fmt;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Tipo {
    Blob,
    Arbol,
    Commit,
}

impl fmt::Display for Tipo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tipo::Blob => write!(f, "blob"),
            Tipo::Arbol => write!(f, "tree"),
            Tipo::Commit => write!(f, "commit"),
        }
    }
}

pub(crate) struct Objeto<R> {
    pub(crate) tipo: Tipo,
    pub(crate) tamaño_esperado: u64,
    pub(crate) lector: R,
}

impl Objeto<()> {
    pub(crate) fn leer(hash: &str) -> anyhow::Result<Objeto<impl BufRead>> {
        let f = std::fs::File::open(format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
            .context("abrir en .git/objects")?;
        let z = ZlibDecoder::new(f);
        let mut z = BufReader::new(z);
        let mut buf = Vec::new();
        z.read_until(0, &mut buf)
            .context("leer cabecera desde .git/objects")?;
        let cabecera = CStr::from_bytes_with_nul(&buf)
            .expect("sabemos que hay exactamente un nul, y está al final");
        let cabecera = cabecera
            .to_str()
            .context("la cabecera del archivo .git/objects no es UTF-8 válido")?;
        let Some((tipo, tamaño)) = cabecera.split_once(' ') else {
            anyhow::bail!("La cabecera del archivo .git/objects no comenzó con un tipo conocido: '{cabecera}'");
        };
        let tipo = match tipo {
            "blob" => Tipo::Blob,
            "tree" => Tipo::Arbol,
            "commit" => Tipo::Commit,
            _ => anyhow::bail!("¿Qué es un '{tipo}'?"),
        };
        let tamaño = tamaño
            .parse::<u64>()
            .context("La cabecera del archivo .git/objects tiene un tamaño inválido: {tamaño}")?;
        // NOTA: esto no dará error si el archivo descomprimido es demasiado largo, pero al menos
        // no spameará stdout y será vulnerable a un zipbomb.
        let z = z.take(tamaño);
        Ok(Objeto {
            tipo,
            tamaño_esperado: tamaño,
            lector: z,
        })
    }
} 