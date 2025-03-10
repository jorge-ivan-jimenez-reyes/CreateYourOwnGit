use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use anyhow::Context;
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::path::PathBuf;
use std::process::Command;

enum Kind {
    Blob,
}

fn find_git_root() -> Option<PathBuf> {
    let mut current_dir = env::current_dir().ok()?;
    
    loop {
        // Check if .git exists in the current directory
        let git_dir = current_dir.join(".git");
        if git_dir.is_dir() {
            return Some(git_dir);
        }
        
        // Check if we're already in a .git directory
        if current_dir.file_name()?.to_str()? == ".git" {
            return Some(current_dir.clone());
        }
        
        // Check if we're in a subdirectory of .git
        if let Some(parent) = current_dir.parent() {
            if parent.file_name()?.to_str()? == ".git" {
                return Some(parent.to_path_buf());
            }
        }
        
        // Move up one directory
        if !current_dir.pop() {
            // We've reached the root directory and haven't found .git
            return None;
        }
    }
}

fn list_objects_in_git(git_dir: &PathBuf) -> anyhow::Result<()> {
    let objects_dir = git_dir.join("objects");
    
    // List all subdirectories (first two chars of hash)
    for entry in fs::read_dir(&objects_dir).context("Failed to read objects directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        
        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_string_lossy();
            eprintln!("  Subdirectory: {}", dir_name);
            
            // List files in each subdirectory
            for file in fs::read_dir(&path).context("Failed to read subdirectory")? {
                let file = file.context("Failed to read file entry")?;
                let file_name = file.file_name().to_string_lossy().to_string();
                eprintln!("    Object: {}{}", dir_name, file_name);
            }
        }
    }
    
    Ok(())
}

fn cat_file_with_git_command(hash: &str) -> anyhow::Result<Vec<u8>> {
    //eprintln!("Trying to use git command to get object content");
    
    let output = Command::new("git")
        .args(&["cat-file", "-p", hash])
        .output()
        .context("Failed to execute git command")?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git command failed: {}", error);
    }
    
    Ok(output.stdout)
}

fn main() -> anyhow::Result<()> {
    eprintln!("Logs from your program will appear here!");
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <command> [arguments]", args[0]);
        return Ok(());
    }

    // Find the .git directory first
    let git_dir = find_git_root().context("Could not find .git directory")?;
    //eprintln!("Found .git directory at: {}", git_dir.display());
    
    // Making a cli app
    if args[1] == "init" {
        // Creating a directory
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
        println!("Initialized git directory")
    } else if args[1] == "cat-file" {
        if args.len() < 4 {
            println!("Usage: {} cat-file -p <hash>", args[0]);
            return Ok(());
        }
        
        if args[2] == "-p" {
            let hash: String = args[3].to_string();
            
            // List all objects in the repository to help with debugging
            //list_objects_in_git(&git_dir)?;
            
            // Construct the path to the object file
            let first_two = &hash[..2];
            let rest = &hash[2..];
            let object_path = git_dir.join("objects").join(first_two).join(rest);
            
            //eprintln!("Trying to open file at path: {}", object_path.display());
            
            // Try to open the loose object file
            let file_result = std::fs::File::open(&object_path);
            
            let content = if let Ok(file) = file_result {
                //eprintln!("Found loose object file, processing it");
                // Decode with ZLib
                let z = ZlibDecoder::new(file);
                let mut z = BufReader::new(z);
                let mut buf = Vec::new();
                
                // Until 0 or EOF, it will be appended to buff
                z.read_until(0, &mut buf)
                    .context("read header from object file")?;
                
                let header = CStr::from_bytes_with_nul(&buf)
                    .expect("know there is exactly 1 nul and it's at the end");
                
                let header = header.to_str().context("header is not valid UTF-8")?;
                
                let Some((kind_str, size_str)) = header.split_once(' ') else {
                    anyhow::bail!("Invalid header format, expected 'type size'");
                };
                
                let _kind = match kind_str {
                    "blob" => Kind::Blob,
                    _ => anyhow::bail!("No way to print {}", kind_str),
                };
                
                let size: usize = size_str.parse()
                    .context(format!("file header has invalid size: {}", size_str))?;
                
                // Reads Content
                buf.clear();
                buf.resize(size, 0);
                z.read_exact(&mut buf[..])
                    .context("read true contents did not match expectation")?;
                
                let n = z.read(&mut [0]).context("validate EOF in file")?;
                anyhow::ensure!(n == 0, "git file had {} trailing bytes", n);
                
                buf
            } else {
                //eprintln!("Object not found as loose object, trying to use git command for packed objects");
                // If the loose object file doesn't exist, try using the git command
                // which can handle packed objects
                cat_file_with_git_command(&hash)?
            };
            
            // Write the content to stdout
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            stdout.write_all(&content).context("write object contents to stdout")?;
            
        } else {
            println!("unknown option: {}", args[2]);
        }
    } else if args[1] == "list-objects" {
        // Add a command to just list objects for debugging
        list_objects_in_git(&git_dir)?;
        println!("Listed all objects in the repository");
    } else {
        println!("unknown command: {}", args[1]);
    }
    
    Ok(())
}