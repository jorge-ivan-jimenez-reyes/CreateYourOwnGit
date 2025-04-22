use create_your_own_git::init;
use flate2::read::ZlibDecoder;
use std::env;
use std::fs;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    eprintln!("Logs from your program will appear here!");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run -- <command>");
        return Ok(());
    }

    match args[1].as_str() {
        "init" => {
            init()?;
        }
        "cat-file" => {
            if args.len() < 4 || args[2] != "-p" {
                eprintln!("Usage: cargo run -- cat-file -p <hash>");
                return Ok(());
            }

            let hash = &args[3];
            let path = format!(".git/objects/{}/{}", &hash[0..2], &hash[2..]);
            let object = fs::File::open(path)?;
            let mut decoder = ZlibDecoder::new(object);
            let mut decoded = vec![];
            decoder.read_to_end(&mut decoded)?;

            // Separate header and body
            let nul = decoded.iter().position(|&b| b == 0).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid object format")
            })?;
            let header = std::str::from_utf8(&decoded[..nul]).map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, e)
            })?;
            let body = &decoded[nul + 1..];

            let mut parts = header.split(' ');
            let obj_type = parts.next().unwrap_or("unknown");
            let _size = parts.next().unwrap_or("0");

            match obj_type {
                "blob" => {
                    let content = String::from_utf8_lossy(body);
                    println!("{}", content);
                }
                "tree" => {
                    let mut i = 0;
                    while i < body.len() {
                        // Read mode
                        let mode_start = i;
                        let mode_end = body[i..]
                            .iter()
                            .position(|&b| b == b' ')
                            .map(|p| p + i)
                            .ok_or_else(|| {
                                io::Error::new(io::ErrorKind::InvalidData, "Invalid tree format")
                            })?;
                        let mode = std::str::from_utf8(&body[mode_start..mode_end]).map_err(|e| {
                            io::Error::new(io::ErrorKind::InvalidData, e)
                        })?;

                        // Read name
                        i = mode_end + 1;
                        let name_end = body[i..]
                            .iter()
                            .position(|&b| b == 0)
                            .map(|p| p + i)
                            .ok_or_else(|| {
                                io::Error::new(io::ErrorKind::InvalidData, "Invalid tree format")
                            })?;
                        let name = std::str::from_utf8(&body[i..name_end]).map_err(|e| {
                            io::Error::new(io::ErrorKind::InvalidData, e)
                        })?;

                        // Read SHA1 (20 bytes)
                        i = name_end + 1;
                        let hash_bytes = &body[i..i + 20];
                        let hash_hex: String =
                            hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();

                        println!("{} {} {}", mode, name, hash_hex);
                        i += 20;
                    }
                }
                other => {
                    eprintln!("Unsupported object type: {}", other);
                }
            }
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
        }
    }
    Ok(())
}
