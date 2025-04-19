#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;

fn main() {
    eprintln!("Logs from your program will appear here!");

    let args: Vec<String> = env::args().collect();
    if args[1] == "init" {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
        println!("Initialized git directory");
    } else if args[1] == "cat-file" {
        if args[2] == "-p" {
            let hash: String = args[3].clone();
            let folder_name = hash[0..2].to_string();
            let file_name = hash[2..].to_string();
            dbg!(folder_name, file_name);
        }
    } else {
        println!("Unknown command");
    }
}
