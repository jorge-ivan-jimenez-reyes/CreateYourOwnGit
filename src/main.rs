mod init;
mod commit;

use init::init;
use commit::run_commit;

use flate2::read::ZlibDecoder;
use std::env;
use std::fs;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Uso: cargo run -- <comando>");
        return Ok(());
    }

    match args[1].as_str() {
        "init" => {
            init()?;
        }
        "commit" => {
            let message_index = args.iter().position(|s| s == "-m");
            if let Some(i) = message_index {
                if let Some(msg) = args.get(i + 1) {
                    run_commit(msg)?;
                } else {
                    eprintln!("Falta el mensaje de commit.");
                }
            } else {
                eprintln!("Uso: cargo run -- commit -m \"mensaje\"");
            }
        }
        "cat-file" => {
            if args.len() < 4 || args[2] != "-p" {
                eprintln!("Uso: cargo run -- cat-file -p <hash>");
                return Ok(());
            }

            let hash = &args[3];
            let path = format!(".git/objects/{}/{}", &hash[0..2], &hash[2..]);
            let object = fs::File::open(path)?;
            let mut decoder = ZlibDecoder::new(object);
            let mut decoded = vec![];
            decoder.read_to_end(&mut decoded)?;

            let nul = decoded.iter().position(|&b| b == 0).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid object format")
            })?;
            let header = std::str::from_utf8(&decoded[..nul])
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            let body = &decoded[nul + 1..];

            let mut parts = header.split(' ');
            let obj_type = parts.next().unwrap_or("unknown");

            match obj_type {
                "blob" => {
                    let content = String::from_utf8_lossy(body);
                    println!("{}", content);
                }
                "tree" => {
                    let mut i = 0;
                    while i < body.len() {
                        let mode_end = body[i..].iter().position(|&b| b == b' ').map(|p| p + i)
                            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid tree format"))?;
                        let mode = std::str::from_utf8(&body[i..mode_end])
                            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                        i = mode_end + 1;
                        let name_end = body[i..].iter().position(|&b| b == 0).map(|p| p + i)
                            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid tree format"))?;
                        let name = std::str::from_utf8(&body[i..name_end])
                            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                        i = name_end + 1;
                        let hash_bytes = &body[i..i + 20];
                        let hash_hex: String = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();

                        println!("{} {} {}", mode, name, hash_hex);
                        i += 20;
                    }
                }
                other => {
                    eprintln!("Tipo de objeto no soportado: {}", other);
                }
            }
        }
        _ => {
            eprintln!("Comando desconocido: {}", args[1]);
        }
    }

    Ok(())
}
