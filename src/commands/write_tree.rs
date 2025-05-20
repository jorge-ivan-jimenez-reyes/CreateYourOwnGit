use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::{BufReader, Read, Write, Seek};
use std::path::{Path, PathBuf};

pub(crate) fn invoke() -> anyhow::Result<()> {
    // Leer el índice para obtener los archivos a incluir en el tree
    let index_path = Path::new(".git/index");
    if !index_path.exists() {
        anyhow::bail!("No existe el archivo .git/index. Debes agregar archivos primero.");
    }

    let file = File::open(index_path)?;
    let file_size = file.metadata()?.len();

    if file_size < 12 {
        anyhow::bail!("El archivo .git/index es demasiado pequeño");
    }

    let mut file = BufReader::new(file);

    let mut signature = [0u8; 4];
    file.read_exact(&mut signature)?;
    if &signature != b"DIRC" {
        anyhow::bail!("Índice inválido");
    }

    let mut version_bytes = [0u8; 4];
    file.read_exact(&mut version_bytes)?;
    let _version = u32::from_be_bytes(version_bytes);

    let mut num_entries_bytes = [0u8; 4];
    file.read_exact(&mut num_entries_bytes)?;
    let num_entries = u32::from_be_bytes(num_entries_bytes);

    println!("Leyendo {} entradas del índice...", num_entries);

    // Estructura para almacenar las entradas del tree
    struct TreeEntry {
        mode: String,
        name: String,
        hash: [u8; 20],
    }

    // Mapa para agrupar entradas por directorio
    let mut entries_by_dir: std::collections::HashMap<PathBuf, Vec<TreeEntry>> = std::collections::HashMap::new();
    let root_dir = PathBuf::from("");

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

        let file_name = match std::str::from_utf8(&name_bytes) {
            Ok(name) if !name.trim().is_empty() => name,
            _ => continue,
        };

        println!("Archivo: {} (hash: {})", file_name, hex::encode(&sha1));

        // Determinar el directorio padre y el nombre base
        let path = PathBuf::from(file_name);
        let parent = path.parent().unwrap_or(&root_dir).to_path_buf();
        let name = path.file_name().unwrap().to_str().unwrap().to_string();

        // Crear la entrada del tree
        let entry = TreeEntry {
            mode: "100644".to_string(), // Modo regular file
            name,
            hash: sha1,
        };

        // Agregar la entrada al directorio correspondiente
        entries_by_dir.entry(parent).or_default().push(entry);

        // Saltar el padding
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

    // Función recursiva para crear trees
    fn create_tree(
        path: &Path,
        entries: &[TreeEntry],
        dir_entries: &std::collections::HashMap<PathBuf, Vec<TreeEntry>>,
    ) -> anyhow::Result<String> {
        let mut tree_data = Vec::new();

        // Primero agregar subdirectorios
        for (dir_path, _) in dir_entries.iter() {
            if let Some(parent) = dir_path.parent() {
                if parent == path {
                    let dir_name = dir_path.file_name().unwrap().to_str().unwrap();
                    let subdir_entries = dir_entries.get(dir_path).unwrap();
                    let hash_hex = create_tree(dir_path, subdir_entries, dir_entries)?;
                    let hash_bytes = hex::decode(&hash_hex)?;
                    
                    // Agregar la entrada para el subdirectorio
                    tree_data.extend(format!("40000 {}\0", dir_name).as_bytes());
                    tree_data.extend(&hash_bytes);
                }
            }
        }

        // Luego agregar archivos
        for entry in entries {
            tree_data.extend(format!("{} {}\0", entry.mode, entry.name).as_bytes());
            tree_data.extend(&entry.hash);
        }

        // Crear el objeto tree
        let tree_header = format!("tree {}\0", tree_data.len());
        let mut tree_object = Vec::new();
        tree_object.extend(tree_header.as_bytes());
        tree_object.extend(&tree_data);

        // Calcular el hash
        let tree_hash = Sha1::digest(&tree_object);
        let tree_hash_hex: String = tree_hash.iter().map(|b| format!("{:02x}", b)).collect();

        // Escribir el objeto
        let tree_path = format!(".git/objects/{}/{}", &tree_hash_hex[0..2], &tree_hash_hex[2..]);
        fs::create_dir_all(format!(".git/objects/{}", &tree_hash_hex[0..2]))?;
        let tree_file = File::create(tree_path)?;
        let mut encoder = ZlibEncoder::new(tree_file, Compression::default());
        encoder.write_all(&tree_object)?;
        encoder.finish()?;

        if path.as_os_str().is_empty() {
            println!("Objeto tree raíz creado: {}", tree_hash_hex);
        } else {
            println!("Objeto tree para '{}' creado: {}", path.display(), tree_hash_hex);
        }

        Ok(tree_hash_hex)
    }

    // Crear el tree raíz
    let empty_vec = Vec::new();
    let root_entries = entries_by_dir.get(&root_dir).unwrap_or(&empty_vec);
    let root_hash = create_tree(&root_dir, root_entries, &entries_by_dir)?;
    
    println!("{}", root_hash);
    
    Ok(())
} 