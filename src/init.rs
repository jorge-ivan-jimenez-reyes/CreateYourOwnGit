use std::fs;
use std::io;

pub fn init() -> io::Result<()> {
    fs::create_dir(".git")?;
    fs::create_dir(".git/objects")?;
    fs::create_dir(".git/refs")?;
    fs::write(".git/HEAD", "ref: refs/heads/main\n")?;
    println!("Initialized git directory");
    Ok(())
}
