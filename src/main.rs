#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use anyhow::Context;
use flate2::read::ZlibDecoder;
use std::ffi::CStr;

fn main() -> anyhow::Result<()> {
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
    //we need cat-file
    //also -p
    //https://www.youtube.com/watch?v=u0VotuGzD_w
    } else if args[1] == "cat-file" {
        if args[2] == "-p" {
            let hash: String = args[3].to_string();
            let file = std::fs::File::open(format!(
                ".git/objects/{}/{}", 
                &hash[..2], &
                hash[2..]
            )).context("open in .git/objects")?;

            //Decodess with ZLib
            let z = ZlibDecoder::new(file); //Zlib decoder
            let mut z = BufReader::new(z);
            let mut buf = Vec::new();
            
            //Until 0 or EOF, it will be appended to buff
            z.read_until(0, &mut buf)
                .context("read header from ./git/objects")?;

            let header = CStr::from_bytes_with_nul(&buf)
                .expect("know there is exaclty 1 nul and it's at the end");
            //dbg!(&hash);

            let header = header.to_str().context("header is not valid UTF-8")?;

            //Extracts header
            let Some(size) = header.strip_prefix("blob ") else{
                anyhow::bail!("header did not start with 'blob': '{header}'");
            };

            //Extracts size
            let size = size.parse::<usize>().context("file header has invalid size: {size}")?;

            //Reads Content
            buf.clear();
            buf.resize(size, 0);
            z.read_exact(&mut buf[..]).context("read true  contents did not match expectation")?;
            let n = z.read(&mut [0]).context("validate EOF in file")?;
            anyhow::ensure!(n == 0, "git file had {n} trailing bytes");

            let stdout = std::io::stdout();
            let mut stdout =  stdout.lock();
            stdout.write_all(&mut buf).context("write object contents to stdout")?;
        }
        else { 

            println!("unknown command: {}", args[1])
        }
    }
    //to see if it initialized a repo
    //1. cargo run init(Initialize a repo in github)
    //then ls -la to list the content from de directory
    //ls -ls .git to see the hidden git files
    //IMPORTANT do not initialized inside a git repo this will break the application

    Ok(())
}

//reading a blob object