use std::fs::{self, File};
use std::io::{self, BufReader, Read, Write, Seek};
use std::path::Path;
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};

pub fn run_commit(message: &str) -> io::Result<()> {
    println!("Mensaje de commit: {}", message);
    println!("Leyendo entradas del índice...");

    let index_path = Path::new(".git/index");
    let file = File::open(index_path)?;
    let file_size = file.metadata()?.len();

    if file_size < 12 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "El archivo .git/index es demasiado pequeño",
        ));
    }

    let mut file = BufReader::new(file);

    let mut signature = [0u8; 4];
    file.read_exact(&mut signature)?;
    if &signature != b"DIRC" {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Índice inválido"));
    }

    let mut version_bytes = [0u8; 4];
    file.read_exact(&mut version_bytes)?;
    let _version = u32::from_be_bytes(version_bytes);

    let mut num_entries_bytes = [0u8; 4];
    file.read_exact(&mut num_entries_bytes)?;
    let num_entries = u32::from_be_bytes(num_entries_bytes);

    let mut tree_entries = Vec::new();

    for _ in 0..num_entries {
        let start_pos = match file.stream_position() {
            Ok(pos) => pos,
            Err(_) => break,
        };

        let mut metadata = [0u8; 62];
        if file.read_exact(&mut metadata).is_err() {
            continue;
        }

        let mut name_bytes = Vec::new();
        let mut name_buf = [0u8; 1];
        while let Ok(_) = file.read_exact(&mut name_buf) {
            if name_buf[0] == 0 {
                break;
            }
            name_bytes.push(name_buf[0]);
        }

        let mut sha1 = [0u8; 20];
        if file.read_exact(&mut sha1).is_err() {
            continue;
        }

        let file_name = match str::from_utf8(&name_bytes) {
            Ok(name) if !name.trim().is_empty() => name,
            _ => continue,
        };

        // println!("Archivo: {} (hash: {:02x?})", file_name, sha1); ← Puedes descomentar si quieres verlos

        let mut entry = Vec::new();
        write!(entry, "100644 {}", file_name)?;
        entry.push(0);
        entry.extend(&sha1);
        tree_entries.push(entry);

        let end_pos = match file.stream_position() {
            Ok(pos) => pos,
            Err(_) => break,
        };

        let entry_len = (end_pos - start_pos) as usize;
        let padding = (8 - (entry_len % 8)) % 8;
        let mut skip = vec![0u8; padding];
        if file.read_exact(&mut skip).is_err() {
            continue;
        }
    }

    let tree_data: Vec<u8> = tree_entries.concat();
    let tree_header = format!("tree {}\0", tree_data.len());
    let mut tree_object = Vec::new();
    tree_object.extend(tree_header.as_bytes());
    tree_object.extend(&tree_data);

    let tree_hash = Sha1::digest(&tree_object);
    let tree_hash_hex: String = tree_hash.iter().map(|b| format!("{:02x}", b)).collect();

    let tree_path = format!(".git/objects/{}/{}", &tree_hash_hex[0..2], &tree_hash_hex[2..]);
    fs::create_dir_all(format!(".git/objects/{}", &tree_hash_hex[0..2]))?;
    let tree_file = File::create(tree_path)?;
    let mut encoder = ZlibEncoder::new(tree_file, Compression::default());
    encoder.write_all(&tree_object)?;
    encoder.finish()?;

    println!("Objeto tree creado: {}", tree_hash_hex);

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let timestamp = now.as_secs();
    let author = format!("abraham <abraham@example.com> {} -0600", timestamp);

    let commit_body = format!(
        "tree {}\nauthor {}\ncommitter {}\n\n{}\n",
        tree_hash_hex, author, author, message
    );

    let commit_header = format!("commit {}\0", commit_body.len());
    let mut commit_object = Vec::new();
    commit_object.extend(commit_header.as_bytes());
    commit_object.extend(commit_body.as_bytes());

    let commit_hash = Sha1::digest(&commit_object);
    let commit_hash_hex: String = commit_hash.iter().map(|b| format!("{:02x}", b)).collect();

    let commit_path = format!(".git/objects/{}/{}", &commit_hash_hex[0..2], &commit_hash_hex[2..]);
    fs::create_dir_all(format!(".git/objects/{}", &commit_hash_hex[0..2]))?;
    let commit_file = File::create(commit_path)?;
    let mut encoder = ZlibEncoder::new(commit_file, Compression::default());
    encoder.write_all(&commit_object)?;
    encoder.finish()?;

    println!("Commit creado: {}", commit_hash_hex);

    fs::create_dir_all(".git/refs/heads")?;
    fs::write(".git/refs/heads/master", format!("{}\n", commit_hash_hex))?;

    Ok(())
}
