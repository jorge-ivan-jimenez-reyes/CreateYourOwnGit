use anyhow::Result;
use std::fs;

pub fn ejecutar() -> Result<()> {
    fs::create_dir(".git")?;
    fs::create_dir(".git/objects")?;
    fs::create_dir(".git/refs")?;
    fs::write(".git/HEAD", "ref: refs/heads/main\n")?;
    println!("Repositorio Git inicializado");
    Ok(())
}
