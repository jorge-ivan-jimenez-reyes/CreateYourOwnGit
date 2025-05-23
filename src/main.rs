use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

pub(crate) mod commands;
pub(crate) mod objects;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Doc comment
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,
        object_hash: String,
    },
    HashObject {
        #[clap(short = 'w')]
        write: bool,
        file: PathBuf,
    },
    LsTree {
        #[clap(long)]
        name_only: bool,
        tree_hash: String,
    },
    ReadTree {
        tree_hash: String,
    },
    WriteTree,
    CommitTree {
        tree_hash: String,
        #[clap(short = 'p')]
        parent: Option<String>,
        #[clap(short = 'm')]
        message: String,
    },
    Clone {
        url: String,
        target_dir: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    println!("{:?}", std::fs::canonicalize(".git"));

    let args = Args::parse();
    match args.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory")
        }
        Command::CatFile {
            pretty_print,
            object_hash,
        } => commands::cat_file::invoke(pretty_print, &object_hash)?,
        Command::HashObject { write, file } => commands::hash_object::invoke(write, &file)?,
        Command::LsTree {
            name_only,
            tree_hash,
        } => commands::ls_tree::invoke(name_only, &tree_hash)?,
        Command::ReadTree { tree_hash } => commands::read_tree::invoke(&tree_hash)?,
        Command::WriteTree => commands::write_tree::invoke()?,
        Command::CommitTree { tree_hash, parent, message } => 
            commands::commit_tree::invoke(&tree_hash, parent.as_deref(), &message)?,
        Command::Clone { url, target_dir } => 
            commands::clone::invoke(&url, &target_dir)?,
    }
    Ok(())
}
