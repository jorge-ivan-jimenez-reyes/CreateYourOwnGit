use flate2::read::ZlibDecoder;
use std::env;
use std::fs;
use std::io::Read;

fn main() {
    eprintln!("Logs from your program will appear aquí!");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Uso: cargo run -- <comando>");
        return;
    }

    match args[1].as_str() {
        "init" => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory");
        }
        "cat-file" => {
            if args.len() < 4 || args[2] != "-p" {
                eprintln!("Uso: cargo run -- cat-file -p <hash>");
                return;
            }

            let hash = &args[3];
            let path = format!(".git/objects/{}/{}", &hash[0..2], &hash[2..]);
            let object = fs::File::open(path).unwrap();
            let mut decoder = ZlibDecoder::new(object);
            let mut decoded = vec![];
            decoder.read_to_end(&mut decoded).unwrap();

            // Separar header y body
            let nul = decoded.iter().position(|&b| b == 0).unwrap();
            let header = std::str::from_utf8(&decoded[..nul]).unwrap();
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
                        // Leer modo
                        let mode_start = i;
                        let mode_end = body[i..]
                            .iter()
                            .position(|&b| b == b' ')
                            .map(|p| p + i)
                            .unwrap();
                        let mode = std::str::from_utf8(&body[mode_start..mode_end]).unwrap();

                        // Leer nombre
                        i = mode_end + 1;
                        let name_end = body[i..]
                            .iter()
                            .position(|&b| b == 0)
                            .map(|p| p + i)
                            .unwrap();
                        let name = std::str::from_utf8(&body[i..name_end]).unwrap();

                        // Leer SHA1 (20 bytes)
                        i = name_end + 1;
                        let hash_bytes = &body[i..i + 20];
                        let hash_hex: String =
                            hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();

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
}
