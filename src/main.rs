#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;

fn main() {
    eprintln!("Logs from your program will appear here!");

    let args: Vec<String> = env::args().collect();
    //making a cli app
    if args[1] == "init" {
        //creating a directory

        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
        println!("Initialized git directory")

    } else if args[1] == "cat-file" {
        println!("unknown command: {}", args[1])
    }
    //to see if it initialized a repo
    //1. cargo run init(Initialize a repo in github)
    //then ls -la to list the content from de directory
    //ls -ls .git to see the hidden git files
    //IMPORTANT do not initialized inside a git repo this will break the application
}

//reading a blob object