# CreateYourOwnGit

Una implementación educativa de Git en Rust que recrea las funcionalidades básicas de Git para entender cómo funciona internamente.

## Descripción

Este proyecto implementa una versión simplificada de Git en Rust, replicando los comandos básicos como `init`, `hash-object`, `cat-file`, `write-tree`, `commit-tree` y `clone`. El objetivo es entender cómo funciona Git por dentro, recreando sus estructuras de datos y algoritmos fundamentales.

## Requisitos

- [Rust](https://www.rust-lang.org/) (versión 1.80 o superior)
- Cargo (viene incluido con Rust)

## Instalación

1. Clona este repositorio:
   ```
   git clone https://github.com/tu-usuario/CreateYourOwnGit.git
   cd CreateYourOwnGit
   ```

2. Compila el proyecto:
   ```
   cargo build --release
   ```

3. Ejecuta el programa:
   ```
   cargo run -- [comando] [argumentos]
   ```

## Comandos Disponibles

### Inicializar un Repositorio

```
cargo run -- iniciar
```

Este comando crea la estructura básica de un repositorio Git:
- Directorio `.git/`
- Subdirectorios `.git/objects/` y `.git/refs/`
- Archivo `.git/HEAD` apuntando a `refs/heads/main`

### Calcular Hash de un Objeto

```
cargo run -- hash-objeto [-w] <archivo>
```

Calcula el hash SHA-1 de un archivo y opcionalmente lo almacena en la base de datos de objetos si se usa la opción `-w`.

**Ejemplo:**
```
cargo run -- hash-objeto -w archivo.txt
```

### Mostrar Contenido de un Objeto

```
cargo run -- mostrar-archivo -p <hash-objeto>
```

Muestra el contenido de un objeto blob identificado por su hash.

**Ejemplo:**
```
cargo run -- mostrar-archivo -p a1b2c3d4e5f6...
```

### Listar Contenido de un Árbol

```
cargo run -- listar-arbol [--name-only] <hash-arbol>
```

Lista los elementos contenidos en un objeto árbol (tree). Con la opción `--name-only` solo muestra los nombres.

**Ejemplo:**
```
cargo run -- listar-arbol a1b2c3d4e5f6...
```

### Leer un Árbol al Directorio de Trabajo

```
cargo run -- leer-arbol <hash-arbol>
```

Extrae los archivos de un objeto árbol al directorio de trabajo.

**Ejemplo:**
```
cargo run -- leer-arbol a1b2c3d4e5f6...
```

### Crear un Árbol desde el Directorio de Trabajo

```
cargo run -- escribir-arbol
```

Crea un objeto árbol a partir del contenido actual del directorio de trabajo.

### Crear un Commit

```
cargo run -- commit-arbol <hash-arbol> -p <hash-padre> -m "<mensaje>"
```

Crea un objeto commit con el árbol especificado, opcionalmente referenciando un commit padre.

**Ejemplo:**
```
cargo run -- commit-arbol a1b2c3d4e5f6... -m "Commit inicial"
```

### Clonar un Repositorio Remoto

```
cargo run -- clonar <url> <directorio-destino>
```

Clona un repositorio Git remoto a un directorio local.

**Ejemplo:**
```
cargo run -- clonar https://github.com/usuario/repo.git mi-repo-clonado
```

## Ejemplo de Flujo de Trabajo Completo

1. **Iniciar un repositorio:**
   ```
   cargo run -- iniciar
   ```

2. **Crear algunos archivos:**
   ```
   echo "Hola mundo" > archivo1.txt
   echo "Otro archivo" > archivo2.txt
   mkdir carpeta
   echo "Contenido en carpeta" > carpeta/archivo3.txt
   ```

3. **Crear un árbol a partir del directorio de trabajo:**
   ```
   cargo run -- escribir-arbol
   ```
   Esto devolverá un hash, guárdalo (por ejemplo: `a1b2c3d4e5f6...`)

4. **Crear un commit con el árbol:**
   ```
   cargo run -- commit-arbol a1b2c3d4e5f6... -m "Mi primer commit"
   ```

## Arquitectura Interna

### Estructura de Datos

El proyecto implementa los tres tipos de objetos básicos de Git:

1. **Blobs**: Representan el contenido de archivos
2. **Trees (Árboles)**: Representan directorios y referencias a blobs o otros árboles
3. **Commits**: Contienen metadatos sobre cambios, referencias a árboles y otros commits

### Almacenamiento de Objetos

Todos los objetos se almacenan comprimidos con zlib en el directorio `.git/objects/` siguiendo el formato:
- Los primeros 2 caracteres del hash forman el nombre del subdirectorio
- Los 38 caracteres restantes forman el nombre del archivo

### Funciones Principales

- `ejecutar()`: Punto de entrada para cada comando
- `Objeto::leer()`: Lee un objeto Git de la base de datos
- `hash_objeto()`: Calcula el hash SHA-1 de un contenido y opcionalmente lo almacena
- `escribir_arbol_directorio()`: Genera un objeto árbol a partir de un directorio
- `crear_commit()`: Crea un objeto commit con los metadatos apropiados
- `procesar_packfile()`: Procesa packfiles durante la clonación
- `aplicar_delta()`: Aplica deltas para reconstruir objetos durante la clonación

## Detalles de Implementación

En esta sección explicamos en detalle cómo está implementado cada comando y por qué se tomaron ciertas decisiones de diseño.

### Arquitectura General

El proyecto está organizado en módulos:
- `main.rs`: Punto de entrada que parsea los comandos mediante Clap
- `objetos.rs`: Define la estructura de datos para objetos Git
- `comandos/*.rs`: Implementación específica de cada comando

### Comando `iniciar`

**Implementación**: [`src/init.rs`]

```rust
pub fn ejecutar() -> Result<()> {
    fs::create_dir(".git")?;
    fs::create_dir(".git/objects")?;
    fs::create_dir(".git/refs")?;
    fs::write(".git/HEAD", "ref: refs/heads/main\n")?;
    println!("Repositorio Git inicializado");
    Ok(())
}
```

**Explicación**: 
- Crea la estructura mínima necesaria para un repositorio Git.
- Usa `main` como rama predeterminada (igual que Git moderno) en lugar de `master`.
- No inicializa archivos adicionales como `config` o `description` para mantener la simplicidad.

### Comando `hash-objeto`

**Implementación**: [`src/comandos/hash_objeto.rs`]

Este comando implementa el cálculo de hashes SHA-1 de objetos:

```rust
pub fn ejecutar(escribir: bool, ruta: &Path) -> Result<()> {
    // Leer el contenido del archivo
    let mut archivo = File::open(ruta).context("No se pudo abrir el archivo")?;
    let mut contenido = Vec::new();
    archivo.read_to_end(&mut contenido)?;
    
    // Calcular el hash SHA-1 del objeto
    let mut hasher = Sha1::new();
    hasher.update(format!("blob {}\0", contenido.len()));
    hasher.update(&contenido);
    let hash = hasher.finalize();
    let hash_str = format!("{:x}", hash);
    
    // Si la bandera -w está establecida, escribir el objeto
    if escribir {
        // Crear el directorio y escribir el objeto comprimido...
    }
    
    println!("{}", hash_str);
    Ok(())
}
```

**Explicación**:
- El formato del hash sigue exactamente la especificación de Git: `<tipo> <tamaño>\0<contenido>`.
- Solo implementa objetos blob por simplicidad.
- La opción `-w` permite almacenar el objeto en la base de datos, igual que en Git real.
- Usa compresión zlib para almacenar objetos, igual que Git real.

### Comando `mostrar-archivo`

**Implementación**: [`src/comandos/mostrar_archivo.rs`]

```rust
pub fn ejecutar(mostrar_bonito: bool, hash_objeto: &str) -> Result<()> {
    if !mostrar_bonito {
        anyhow::bail!("Solo se admite la opción -p");
    }

    let mut objeto = Objeto::leer(hash_objeto)?;
    if objeto.tipo == Tipo::Blob {
        let mut contenido = Vec::new();
        objeto.lector.read_to_end(&mut contenido)?;
        std::io::stdout().write_all(&contenido)?;
    } else {
        anyhow::bail!("Solo se admiten objetos de tipo blob");
    }

    Ok(())
}
```

**Explicación**:
- Solo soporta la opción `-p` (pretty-print) por simplicidad.
- Usa la función `Objeto::leer` para leer el objeto de la base de datos.
- Verifica que el objeto sea de tipo blob.
- Solo implementa la visualización de blobs para mantener la simplicidad.

### Estructura de Objetos

**Implementación**: [`src/objetos.rs`]

```rust
pub(crate) enum Tipo {
    Blob,
    Arbol,
    Commit,
}

pub(crate) struct Objeto<R> {
    pub(crate) tipo: Tipo,
    pub(crate) tamaño_esperado: u64,
    pub(crate) lector: R,
}

impl Objeto<()> {
    pub(crate) fn leer(hash: &str) -> anyhow::Result<Objeto<impl BufRead>> {
        // Leer y descomprimir el objeto...
    }
}
```

**Explicación**:
- Define los tres tipos principales de objetos de Git.
- Usa genéricos para permitir flexibilidad en el tipo de lector.
- Implementa una función para leer objetos de la base de datos que:
  1. Encuentra el objeto por su hash
  2. Descomprime el contenido
  3. Parsea el encabezado
  4. Devuelve un objeto con tipo, tamaño y contenido

### Comando `escribir-arbol`

**Implementación**: [`src/comandos/escribir_arbol.rs`]

```rust
pub fn ejecutar() -> Result<()> {
    let hash = escribir_arbol_directorio(".")?;
    println!("{}", hash);
    Ok(())
}

fn escribir_arbol_directorio(ruta: &str) -> Result<String> {
    // Recorrer el directorio recursivamente
    // Crear entradas para cada archivo/directorio
    // Ordenar las entradas
    // Calcular el hash
    // Almacenar el objeto árbol
}
```

**Explicación**:
- Recorre recursivamente el directorio actual.
- Ignora archivos ocultos y el directorio `.git/`.
- Para cada archivo, calcula su hash usando `hash_objeto`.
- Para cada directorio, realiza una llamada recursiva.
- Genera un objeto tree con formato compatible con Git.
- Ordena las entradas por nombre, igual que Git real.
- Almacena el árbol en la base de datos de objetos.

### Comando `commit-arbol`

**Implementación**: [`src/comandos/commit_arbol.rs`]

```rust
pub fn ejecutar(hash_arbol: &str, hash_padre: Option<&str>, mensaje: &str) -> Result<()> {
    // Crear el contenido del commit
    let mut contenido = format!("tree {}\n", hash_arbol);
    
    // Agregar el hash del commit padre si existe
    if let Some(padre) = hash_padre {
        contenido.push_str(&format!("parent {}\n", padre));
    }
    
    // Agregar información del autor y committer...
    
    // Calcular hash, almacenar y actualizar HEAD...
}
```

**Explicación**:
- Crea un objeto commit con formato compatible con Git.
- Soporta referencias a commits padres con el parámetro `-p`.
- Obtiene la información del autor de la configuración global de Git.
- Calcula timestamps en formato Unix.
- Actualiza automáticamente HEAD para apuntar al nuevo commit.

### Comando `clonar`

**Implementación**: [`src/comandos/clonar.rs`]

El comando clonar es el más complejo e implementa:

1. **Solicitud HTTP para obtener referencias**:
   ```rust
   let respuesta_info_refs = cliente.get(&url_info_refs)
       .header("User-Agent", "git/2.0.0")
       .send()?
       .text()?;
   ```

2. **Procesamiento de packfiles**:
   ```rust
   fn procesar_packfile(datos_pack: &[u8], directorio_git: &Path) -> Result<()> {
       // Encontrar la firma PACK
       // Leer cabecera y verificar versión
       // Procesar cada objeto en el packfile
   }
   ```

3. **Manejo de deltas**:
   ```rust
   fn aplicar_delta(delta: &[u8], base: &[u8]) -> Result<Vec<u8>> {
       // Leer tamaños base y resultado
       // Aplicar instrucciones de copia e inserción
   }
   ```

4. **Checkout del árbol de trabajo**:
   ```rust
   fn checkout_arbol_trabajo(directorio_git: &Path, hash_commit: &str, directorio_destino: &Path) -> Result<()> {
       // Leer el commit para obtener el árbol
       // Extraer recursivamente el árbol al directorio
   }
   ```

**Explicación**:
- Implementa una versión simplificada pero funcional del protocolo Git HTTP.
- Soporta el procesamiento de packfiles, que es como Git transfiere objetos eficientemente.
- Implementa la decodificación de objetos delta, tanto para offset-deltas como ref-deltas.
- Reconstruye el árbol de trabajo a partir del árbol del commit HEAD.
- Maneja permisos de archivos y enlaces simbólicos en sistemas Unix.

### Manejo de Datos Binarios

Para trabajar con datos binarios (como en packfiles), el proyecto implementa:

1. **Decodificación de formato variable**:
   ```rust
   // Leer tamaño en formato de longitud variable
   let mut tamaño = 0;
   let mut desplazamiento = 0;
   while i < delta.len() {
       let byte = delta[i];
       i += 1;
       tamaño |= ((byte & 0x7F) as usize) << desplazamiento;
       desplazamiento += 7;
       if (byte & 0x80) == 0 {
           break;
       }
   }
   ```

2. **Manejo de instrucciones delta**:
   ```rust
   if (instruccion & 0x80) != 0 {
       // Instrucción de copia
       // ...
   } else if instruccion != 0 {
       // Instrucción de inserción
       // ...
   }
   ```

**Explicación**:
- Implementa el formato de longitud variable de Git, donde el bit más significativo indica continuación.
- Decodifica correctamente las instrucciones delta para reconstruir objetos.
- Maneja adecuadamente los datos binarios a nivel de bytes.

## Razones para Ciertas Decisiones de Diseño

### Uso de Rust

Rust fue elegido por:
- Seguridad de memoria sin recolector de basura
- Rendimiento cercano a C/C++
- Sistema de tipos expresivo que ayuda a modelar los conceptos de Git
- Manejo de errores mediante `Result` que facilita la propagación de errores

### Estructuración del Proyecto

- Se usó un diseño modular con cada comando en su propio archivo para facilitar la comprensión.
- Se implementaron los comandos como funciones puras que reciben argumentos explícitos en lugar de usar estado global.
- Se usaron tipos enumerados para representar conceptos como los tipos de objetos Git.

### Simplificaciones

Para mantener el proyecto educativo y comprensible:
- No se implementó un índice (staging area)
- No se implementaron ramas ni etiquetas
- Se simplificó el manejo de conflictos
- No se implementaron todas las opciones de cada comando

### Compatibilidad con Git Real

A pesar de las simplificaciones, el proyecto mantiene compatibilidad con Git real:
- Los objetos generados tienen el mismo formato y hash que Git real
- El protocolo de clonación sigue el estándar HTTP de Git
- Los árboles y commits son compatibles con Git real

## Limitaciones

Esta implementación es educativa y tiene las siguientes limitaciones:

- No soporta todas las características de Git (como ramas, tags, merge, etc.)
- El manejo de errores es básico
- No incluye optimizaciones de rendimiento presentes en Git real
- El comando `clone` solo admite el protocolo HTTP/HTTPS, no SSH

## Contribuciones

Las contribuciones son bienvenidas. Por favor, siente libre de mejorar este proyecto educativo.

## Licencia

Este proyecto está licenciado bajo [incluir licencia].

