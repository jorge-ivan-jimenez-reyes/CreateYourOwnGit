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

A continuación se muestra un flujo de trabajo completo utilizando todos los comandos disponibles:

### 1. Crear un Repositorio Desde Cero

```bash
# Crear directorio para el proyecto
mkdir mi-proyecto
cd mi-proyecto

# Inicializar un repositorio Git
cargo run -- iniciar

# Verificar que se creó la estructura .git
ls -la
```

### 2. Crear y Manipular Archivos

```bash
# Crear algunos archivos
echo "# Mi Proyecto" > README.md
echo "console.log('Hola mundo');" > app.js
mkdir src
echo "function saludar() { return 'Hola'; }" > src/funciones.js

# Calcular el hash del README sin guardarlo
cargo run -- hash-objeto README.md
# El resultado será algo como: 8e95bfba2d81498f65541e95ecdf8a05f9d6e2b5

# Calcular y guardar el hash del README
cargo run -- hash-objeto -w README.md
# Guardar este hash para uso posterior: 8e95bfba2d81498f65541e95ecdf8a05f9d6e2b5

# Calcular y guardar el hash de app.js
cargo run -- hash-objeto -w app.js
# Guardar este hash: 7d108c946fe3b3d768fc1b8559cbead2e4998827

# Calcular y guardar el hash de src/funciones.js
cargo run -- hash-objeto -w src/funciones.js
# Guardar este hash: 1af17e73721dbe0c40011b82ed4bb1a7dbe3ce29
```

### 3. Visualizar Contenido de Objetos

```bash
# Ver el contenido del README usando su hash
cargo run -- mostrar-archivo -p 8e95bfba2d81498f65541e95ecdf8a05f9d6e2b5
# Debería mostrar: # Mi Proyecto

# Ver el contenido de app.js usando su hash
cargo run -- mostrar-archivo -p 7d108c946fe3b3d768fc1b8559cbead2e4998827
# Debería mostrar: console.log('Hola mundo');
```

### 4. Crear un Árbol (Snapshot del Directorio)

```bash
# Crear un árbol a partir del directorio actual
cargo run -- escribir-arbol
# Esto devolverá un hash, guárdalo (por ejemplo: c68d233a33c5930ef3a38968a47477fd53ff8f42)

# Listar el contenido del árbol
cargo run -- listar-arbol c68d233a33c5930ef3a38968a47477fd53ff8f42
# Mostrará algo como:
# 100644 blob 7d108c946fe3b3d768fc1b8559cbead2e4998827    app.js
# 100644 blob 8e95bfba2d81498f65541e95ecdf8a05f9d6e2b5    README.md
# 040000 tree f68b86fb5c797961937d71e25c697bc863988e7d    src

# Listar solo los nombres del árbol
cargo run -- listar-arbol --name-only c68d233a33c5930ef3a38968a47477fd53ff8f42
# Mostrará:
# app.js
# README.md
# src

# Opcionalmente, podemos listar el contenido del directorio src
cargo run -- listar-arbol f68b86fb5c797961937d71e25c697bc863988e7d
# Mostrará:
# 100644 blob 1af17e73721dbe0c40011b82ed4bb1a7dbe3ce29    funciones.js
```

### 5. Crear un Commit

```bash
# Crear un commit con el árbol que creamos
cargo run -- commit-arbol c68d233a33c5930ef3a38968a47477fd53ff8f42 -m "Commit inicial"
# Esto devolverá un hash de commit, guárdalo (por ejemplo: a7d9a15f9e1655cd7e47e51d3b25307e11775b49)
```

### 6. Hacer Cambios y Crear un Segundo Commit

```bash
# Modificar un archivo
echo "console.log('Hola mundo actualizado');" > app.js

# Calcular y guardar el hash del archivo modificado
cargo run -- hash-objeto -w app.js
# Guardar este nuevo hash: 9c5b3ce3e9eeb7ef7da7b620bb36c6794da69a3b

# Crear un nuevo árbol
cargo run -- escribir-arbol
# Guardar este nuevo hash de árbol: f7b877f1151eb2815da8c75b79b07a782a8d5cc5

# Crear un segundo commit referenciando el primero como padre
cargo run -- commit-arbol f7b877f1151eb2815da8c75b79b07a782a8d5cc5 -p a7d9a15f9e1655cd7e47e51d3b25307e11775b49 -m "Actualización de app.js"
# Esto devolverá un nuevo hash de commit: b2e5c96a7e2135dc3893bba7b8103683cfc8a32b
```

### 7. Experimentar con Árboles y Directorios

```bash
# Vamos a crear un directorio temporal para experimentar
mkdir ../temp-test
cd ../temp-test

# Extraer el árbol del primer commit al directorio actual
cargo run -- leer-arbol c68d233a33c5930ef3a38968a47477fd53ff8f42

# Verificar que se extrajeron los archivos correctamente
ls -la
cat README.md
cat app.js
cat src/funciones.js
```

### 8. Clonar un Repositorio Remoto

```bash
# Volver al directorio principal
cd ..

# Clonar un repositorio remoto
cargo run -- clonar https://github.com/rust-lang/rust-by-example.git rust-example

# Explorar el repositorio clonado
cd rust-example
ls -la
```

### 9. Trabajar con el Repositorio Clonado

```bash
# Crear un nuevo archivo en el repo clonado
echo "// Mis notas sobre Rust" > mis-notas.rs

# Calcular y guardar el hash del nuevo archivo
cargo run -- hash-objeto -w mis-notas.rs
# Guardar este hash: 3a8f2af030b7e218f2e5c7e19d0f68616736a5b3

# Crear un nuevo árbol con nuestros cambios
cargo run -- escribir-arbol
# Guardar este hash: d42fb816e2e9734ec93ed931f4eaa4c193147f38

# Obtener el hash del commit HEAD actual
cat .git/refs/heads/main
# Por ejemplo: 6c2dc2a236457d439eb51ac3cc4743bca190887e

# Crear un nuevo commit con nuestros cambios
cargo run -- commit-arbol d42fb816e2e9734ec93ed931f4eaa4c193147f38 -p 6c2dc2a236457d439eb51ac3cc4743bca190887e -m "Agregué mis notas personales"
# Esto creará un nuevo commit: 1e9f8aec3df5b8ac9016842863612a0ff3230fff
```

### 10. Explorar un Commit en Detalle

```bash
# Obtener el árbol del commit
# Primero necesitamos leer el objeto commit para extraer el hash del árbol
# (Esta funcionalidad no está implementada directamente, pero podríamos verlo con cat-file)
# Usaremos el árbol que conocemos: d42fb816e2e9734ec93ed931f4eaa4c193147f38

# Listar el contenido del árbol del commit
cargo run -- listar-arbol d42fb816e2e9734ec93ed931f4eaa4c193147f38

# Ver el contenido de nuestro archivo
cargo run -- mostrar-archivo -p 3a8f2af030b7e218f2e5c7e19d0f68616736a5b3
# Debería mostrar: // Mis notas sobre Rust
```

### 11. Resumen del Flujo de Trabajo

Este flujo de trabajo muestra cómo:

1. Inicializar un repositorio Git desde cero
2. Calcular hashes de archivos y almacenarlos como objetos blob
3. Visualizar el contenido de objetos almacenados
4. Crear árboles (snapshots del directorio)
5. Crear commits vinculados a árboles
6. Crear una secuencia de commits conectados (historia)
7. Extraer el contenido de un árbol a un directorio
8. Clonar un repositorio remoto
9. Hacer cambios y crear commits en un repositorio existente
10. Examinar la estructura interna de Git

Este flujo cubre todos los comandos disponibles en nuestra implementación y demuestra cómo funcionan juntos para crear un sistema de control de versiones.

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

