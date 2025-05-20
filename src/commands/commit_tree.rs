use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn invoke(tree_hash: &str, parent: Option<&str>, message: &str) -> anyhow::Result<()> {
    // Verificar que el tree_hash existe
    let tree_path = format!(".git/objects/{}/{}", &tree_hash[0..2], &tree_hash[2..]);
    if !std::path::Path::new(&tree_path).exists() {
        anyhow::bail!("El objeto tree {} no existe", tree_hash);
    }

    // Verificar que el parent existe, si se proporciona
    if let Some(parent_hash) = parent {
        let parent_path = format!(".git/objects/{}/{}", &parent_hash[0..2], &parent_hash[2..]);
        if !std::path::Path::new(&parent_path).exists() {
            anyhow::bail!("El objeto commit padre {} no existe", parent_hash);
        }
    }

    // Obtener timestamp actual
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let timestamp = now.as_secs();
    
    // Información del autor y committer (hardcodeada como se sugiere)
    let author = format!("Author Name <author@example.com> {} -0600", timestamp);
    let committer = format!("Committer Name <committer@example.com> {} -0600", timestamp);
    
    // Construir el contenido del commit
    let mut commit_content = format!("tree {}\n", tree_hash);
    
    // Agregar parent si existe
    if let Some(parent_hash) = parent {
        commit_content.push_str(&format!("parent {}\n", parent_hash));
    }
    
    // Agregar autor, committer y mensaje
    commit_content.push_str(&format!("author {}\n", author));
    commit_content.push_str(&format!("committer {}\n", committer));
    commit_content.push_str("\n");
    commit_content.push_str(message);
    commit_content.push_str("\n");
    
    // Crear el objeto commit
    let commit_header = format!("commit {}\0", commit_content.len());
    let mut commit_object = Vec::new();
    commit_object.extend(commit_header.as_bytes());
    commit_object.extend(commit_content.as_bytes());
    
    // Calcular el hash SHA-1
    let commit_hash = Sha1::digest(&commit_object);
    let commit_hash_hex: String = commit_hash.iter().map(|b| format!("{:02x}", b)).collect();
    
    // Crear el directorio para el objeto
    let object_dir = format!(".git/objects/{}", &commit_hash_hex[0..2]);
    fs::create_dir_all(&object_dir)?;
    
    // Crear el archivo del objeto
    let object_path = format!("{}/{}", object_dir, &commit_hash_hex[2..]);
    let object_file = File::create(object_path)?;
    
    // Comprimir y escribir el contenido
    let mut encoder = ZlibEncoder::new(object_file, Compression::default());
    encoder.write_all(&commit_object)?;
    encoder.finish()?;
    
    // Imprimir el hash del commit
    println!("{}", commit_hash_hex);
    
    Ok(())
} 