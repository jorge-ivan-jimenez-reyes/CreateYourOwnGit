use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

pub(crate) mod comandos;
pub(crate) mod objetos;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Argumentos {
    #[command(subcommand)]
    comando: Comando,
}

#[derive(Debug, Subcommand)]
enum Comando {
    /// Inicializa un repositorio git
    Iniciar,
    MostrarArchivo {
        #[clap(short = 'p')]
        mostrar_bonito: bool,
        hash_objeto: String,
    },
    HashObjeto {
        #[clap(short = 'w')]
        escribir: bool,
        archivo: PathBuf,
    },
    ListarArbol {
        #[clap(long)]
        solo_nombres: bool,
        hash_arbol: String,
    },
    LeerArbol {
        hash_arbol: String,
    },
    EscribirArbol,
    CommitArbol {
        hash_arbol: String,
        #[clap(short = 'p')]
        padre: Option<String>,
        #[clap(short = 'm')]
        mensaje: String,
    },
    Clonar {
        url: String,
        directorio_destino: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    println!("{:?}", std::fs::canonicalize(".git"));

    let argumentos = Argumentos::parse();
    match argumentos.comando {
        Comando::Iniciar => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Repositorio Git inicializado")
        }
        Comando::MostrarArchivo {
            mostrar_bonito,
            hash_objeto,
        } => comandos::mostrar_archivo::ejecutar(mostrar_bonito, &hash_objeto)?,
        Comando::HashObjeto { escribir, archivo } => comandos::hash_objeto::ejecutar(escribir, &archivo)?,
        Comando::ListarArbol {
            solo_nombres,
            hash_arbol,
        } => comandos::listar_arbol::ejecutar(solo_nombres, &hash_arbol)?,
        Comando::LeerArbol { hash_arbol } => comandos::leer_arbol::ejecutar(&hash_arbol)?,
        Comando::EscribirArbol => comandos::escribir_arbol::ejecutar()?,
        Comando::CommitArbol { hash_arbol, padre, mensaje } => 
            comandos::commit_arbol::ejecutar(&hash_arbol, padre.as_deref(), &mensaje)?,
        Comando::Clonar { url, directorio_destino } => 
            comandos::clonar::ejecutar(&url, &directorio_destino)?,
    }
    Ok(())
}
