> For now, this guide is only in Spanish, but you can translate it and contribute.

# Cómo ayudar en Browseraptor

Gracias por querer ayudar en Browseraptor.

Este archivo explica cómo puedes ayudar, cómo probar el proyecto en tu computadora y cómo crear plugins. Está escrito con palabras simples. La idea es que también lo pueda entender alguien que está empezando a programar y que todavía no conoce Rust.

## Qué es Browseraptor

Browseraptor es un programa que te ayuda a elegir con qué navegador abrir un enlace (link). Por ejemplo, puedes abrir unos enlaces con Chrome y otros con Firefox.

## Índice

- [Código de conducta](#código-de-conducta)
- [Cómo puedes ayudar](#cómo-puedes-ayudar)
  - [Avisar de un error](#avisar-de-un-error)
  - [Proponer una idea](#proponer-una-idea)
  - [Enviar cambios](#enviar-cambios)
- [Preparar tu computadora](#preparar-tu-computadora)
  - [Qué necesitas instalar](#qué-necesitas-instalar)
  - [Descargar el proyecto](#descargar-el-proyecto)
  - [Compilar y ejecutar](#compilar-y-ejecutar)
- [Cómo está organizado el proyecto](#cómo-está-organizado-el-proyecto)
- [Plugins de Browseraptor](#plugins-de-browseraptor)
  - [Qué es un plugin](#qué-es-un-plugin)
  - [Qué puede hacer un plugin](#qué-puede-hacer-un-plugin)
  - [Qué archivos necesita un plugin](#qué-archivos-necesita-un-plugin)
  - [Cómo se comunica Browseraptor con un plugin](#cómo-se-comunica-browseraptor-con-un-plugin)
  - [Respuesta del plugin al revisar un enlace](#respuesta-del-plugin-al-revisar-un-enlace)
  - [Archivo manifest.json](#archivo-manifestjson)
  - [Crear tu primer plugin paso a paso](#crear-tu-primer-plugin-paso-a-paso)
  - [Compilar el plugin](#compilar-el-plugin)
  - [Instalar un plugin en Browseraptor](#instalar-un-plugin-en-browseraptor)
  - [Plugins con configuración](#plugins-con-configuración)
  - [Plugins de traducción](#plugins-de-traducción)
  - [Problemas comunes](#problemas-comunes)
- [Reglas para el código](#reglas-para-el-código)
- [Trabajos automáticos (Workflows)](#trabajos-automáticos-workflows)

## Código de conducta

Este proyecto sigue el [Código de conducta de Rust](https://www.rust-lang.org/policies/code-of-conduct). Al participar, se espera que seas respetuoso con los demás.

## Cómo puedes ayudar

### Avisar de un error

Antes de avisar de un error:

- Revisa si alguien ya lo reportó en [Issues](https://github.com/livrasand/Browseraptor/issues).
- Anota datos de tu computadora: sistema (Windows, macOS o Linux), versión del sistema y versión de Browseraptor.

Cuando abras un reporte, incluye:

- Un título claro.
- Los pasos para ver el error (qué hiciste, en orden).
- Qué esperabas que pasara y qué pasó en realidad.
- Una foto o video de la pantalla, si ayuda.
- Si viste algún mensaje de error, cópialo.

### Proponer una idea

Si tienes una idea para mejorar Browseraptor:

- Explica para qué serviría.
- Explica cómo te imaginas que funcionaría.
- Escucha otras ideas y comentarios.

### Enviar cambios

1. Haz una copia (fork) del repositorio y crea una rama desde `master`.
2. Prueba tus cambios. Estos comandos deben funcionar sin problemas:
   - `cargo build --locked`
   - `cargo clippy`
3. Cambia solo lo necesario para resolver el problema o agregar la mejora.
4. Si tu cambio cambia cómo se usa algo, actualiza la documentación.
5. Abre un pull request con un título y una descripción claros.

## Preparar tu computadora

### Qué necesitas instalar

Browseraptor está hecho con **Rust**. Rust es un lenguaje de programación. Para trabajar con Rust necesitas una herramienta llamada **Cargo**. Cargo sirve para descargar lo que el proyecto necesita, compilarlo (convertir el código en un programa) y ejecutarlo.

- Instala Rust estable (incluye Cargo):

```sh
rustup toolchain install stable
```

- **macOS:** instala Xcode (y Metal Toolchain) para poder compilar la app en tu Mac.
- **Linux:** instala las librerías del sistema que pide el [README](./README.md#linux).
- **Windows:** instala Visual Studio Build Tools con la opción "Desktop development with C++".

### Descargar el proyecto

```sh
git clone https://github.com/livrasand/Browseraptor.git
cd Browseraptor
```

### Compilar y ejecutar

Compilar el proyecto:

```sh
cargo build --release
```

Ejecutar Browseraptor:

```sh
cargo run
```

Abrir un enlace con el selector:

```sh
cargo run -- open https://example.com
```

Ver los navegadores que Browseraptor encontró en tu computadora:

```sh
cargo run -- list
```

Crear el paquete `.app` en macOS:

```sh
./bundle_macos.sh
# Resultado: dist/Browseraptor.app
```

## Cómo está organizado el proyecto

Estas son las carpetas y archivos más importantes:

```
src/
  main.rs               — Punto de inicio del programa
  app.rs                — Mensajes y tipos que usan varias partes
  tray.rs               — Icono y menú en la barra del sistema
  hotkey.rs             — Atajos de teclado
  single_instance.rs    — Evita que se abran varias copias a la vez
  default_browser.rs    — Revisa y cambia el navegador por defecto

  browser/
    mod.rs              — Tipos de navegadores
    detector.rs         — Busca navegadores instalados
    launcher.rs         — Abre enlaces en un navegador

  config/
    mod.rs              — Guarda y carga la configuración y los plugins instalados
    rules.rs            — Reglas para decidir qué navegador usar

  plugin/
    mod.rs              — Sistema de plugins: cargar y ejecutar plugins locales

  ui/
    mod.rs              — Partes de la interfaz
    selector.rs         — Ventana para elegir navegador y manejar plugins
    default_prompt.rs   — Ventana para poner Browseraptor como navegador por defecto

assets/
  icons/                — Iconos de la interfaz
  os/                   — Iconos usados en las descargas del release
  browseraptor-logo.png
  screenshot.png
```

## Plugins de Browseraptor

Esta es la parte más importante si quieres crear un plugin.

### Qué es un plugin

Un plugin es como una pieza extra que le agrega una función a Browseraptor. No tienes que cambiar el código principal de Browseraptor para usarlo.

Un plugin de Browseraptor es un archivo especial llamado **WASM** (WebAssembly). Puedes pensarlo como un programita que corre dentro de una caja segura. Browseraptor le da información (por ejemplo, un enlace) y el plugin responde.

Los plugins se instalan de forma local en tu computadora (por ejemplo, plugins que estás creando y probando). No hay una tienda ni un repositorio de plugins dentro de la app: tú eliges el archivo del plugin desde tu computadora.

### Qué puede hacer un plugin

Un plugin puede:

- Decidir qué navegador debe abrir un enlace.
- Elegir un perfil del navegador (si el navegador usa perfiles).
- Cancelar la apertura de un enlace.
- Tener una pantalla de configuración (si el plugin la ofrece).
- Traducir los textos de la interfaz de Browseraptor.

### Qué archivos necesita un plugin

Cada plugin necesita dos archivos en la misma carpeta:

1. **El archivo `.wasm`**: es el plugin ya compilado (listo para usar).
2. **`manifest.json`**: es un archivo de texto con información del plugin (nombre, versión, etc.).

Ejemplo de carpeta:

```
mi-plugin/
  manifest.json
  mi_plugin.wasm
```

### Cómo se comunica Browseraptor con un plugin

Browseraptor y el plugin se pasan datos usando la memoria del plugin. No te preocupes si esto suena raro: si usas el ejemplo de este documento, ya tendrás la base lista.

Tu plugin debe incluir una sección de memoria (`memory`) y estas funciones obligatorias:

| Función | Para qué sirve |
| --- | --- |
| `alloc(size)` | Pide un espacio en memoria de cierto tamaño. Devuelve dónde empieza ese espacio. |
| `dealloc(ptr, size)` | Libera un espacio de memoria que ya no se usa. |
| `evaluate(ptr, len)` | Browseraptor envía un enlace (URL) y el plugin responde qué hacer con ese enlace. |

Cómo funciona `evaluate`:

1. Browseraptor escribe el enlace como texto (UTF-8) en la memoria del plugin.
2. Llama a `evaluate(ptr, len)`, donde `ptr` es la posición y `len` es el tamaño del texto.
3. El plugin devuelve una posición en memoria con la respuesta.
4. La respuesta tiene este formato: **4 bytes** con el tamaño + **los datos en JSON**.

Ejemplo en Rust de las funciones obligatorias (puedes copiarlo como base):

```rust
use serde::Serialize;
use std::slice;
use std::str;

#[derive(Serialize)]
struct PluginResult {
    browser: Option<String>,
    profile: Option<String>,
    cancel: bool,
}

#[no_mangle]
pub extern "C" fn alloc(size: i32) -> i32 {
    let mut buf = Vec::<u8>::with_capacity(size as usize);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr as i32
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: i32, size: i32) {
    unsafe {
        let _ = Vec::from_raw_parts(ptr as *mut u8, 0, size as usize);
    }
}

#[no_mangle]
pub extern "C" fn evaluate(ptr: i32, len: i32) -> i32 {
    let url = unsafe {
        let bytes = slice::from_raw_parts(ptr as *const u8, len as usize);
        str::from_utf8(bytes).unwrap_or("")
    };

    let result = PluginResult {
        browser: None,
        profile: None,
        cancel: false,
    };

    write_json_result(&result)
}

fn write_json_result<T: Serialize>(value: &T) -> i32 {
    let json = serde_json::to_vec(value).unwrap_or_else(|_| b"{}".to_vec());
    let total_len = 4 + json.len();
    let mut out = Vec::<u8>::with_capacity(total_len);
    out.extend_from_slice(&(json.len() as i32).to_le_bytes());
    out.extend_from_slice(&json);
    let ptr = out.as_mut_ptr();
    std::mem::forget(out);
    ptr as i32
}
```

### Respuesta del plugin al revisar un enlace

La función `evaluate` debe devolver un JSON con esta forma:

```json
{
  "browser": "Firefox",
  "profile": "Work",
  "cancel": false
}
```

Qué significa cada campo:

- `browser`: nombre del navegador que debe abrir el enlace. Usa `null` si no quieres elegir navegador.
- `profile`: perfil del navegador. Usa `null` si no aplica.
- `cancel`: si es `true`, Browseraptor cancela la apertura del enlace.

Ejemplos:

- No hacer nada y dejar que Browseraptor decida:

```json
{ "browser": null, "profile": null, "cancel": false }
```

- Abrir en un navegador:

```json
{ "browser": "Google Chrome", "profile": null, "cancel": false }
```

- Bloquear el enlace:

```json
{ "browser": null, "profile": null, "cancel": true }
```

### Archivo manifest.json

Cada plugin debe tener un `manifest.json` en la misma carpeta que el `.wasm`.

```json
{
  "id": "my-plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "Opens work links in a specific browser"
}
```

Campos:

- `id`: identificador del plugin (usa letras, números y guiones, sin espacios).
- `name`: nombre que se muestra.
- `version`: versión del plugin (por ejemplo `1.0.0`).
- `author`: quién lo hizo.
- `description`: para qué sirve el plugin.

Browseraptor lee estos campos del manifest.

### Crear tu primer plugin paso a paso

Este ejemplo hace un plugin simple: si el enlace contiene `github.com`, lo abre con Google Chrome. Si no, no hace nada.

1. Instala la herramienta para compilar a WASM:

```sh
rustup target add wasm32-unknown-unknown
```

2. Crea un proyecto nuevo de biblioteca (library):

```sh
cargo init --lib mi-plugin
cd mi-plugin
```

3. Cambia `Cargo.toml` para que quede así:

```toml
[package]
name = "mi-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

4. Reemplaza el contenido de `src/lib.rs` con este código:

```rust
use serde::Serialize;
use std::slice;
use std::str;

#[derive(Serialize)]
struct PluginResult {
    browser: Option<String>,
    profile: Option<String>,
    cancel: bool,
}

#[no_mangle]
pub extern "C" fn alloc(size: i32) -> i32 {
    let mut buf = Vec::<u8>::with_capacity(size as usize);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr as i32
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: i32, size: i32) {
    unsafe {
        let _ = Vec::from_raw_parts(ptr as *mut u8, 0, size as usize);
    }
}

#[no_mangle]
pub extern "C" fn evaluate(ptr: i32, len: i32) -> i32 {
    let url = unsafe {
        let bytes = slice::from_raw_parts(ptr as *const u8, len as usize);
        str::from_utf8(bytes).unwrap_or("")
    };

    let result = if url.contains("github.com") {
        PluginResult {
            browser: Some("Google Chrome".to_string()),
            profile: None,
            cancel: false,
        }
    } else {
        PluginResult {
            browser: None,
            profile: None,
            cancel: false,
        }
    };

    write_json_result(&result)
}

fn write_json_result<T: Serialize>(value: &T) -> i32 {
    let json = serde_json::to_vec(value).unwrap_or_else(|_| b"{}".to_vec());
    let total_len = 4 + json.len();
    let mut out = Vec::<u8>::with_capacity(total_len);
    out.extend_from_slice(&(json.len() as i32).to_le_bytes());
    out.extend_from_slice(&json);
    let ptr = out.as_mut_ptr();
    std::mem::forget(out);
    ptr as i32
}
```

5. Crea un archivo `manifest.json` en la carpeta del plugin:

```json
{
  "id": "mi-plugin",
  "name": "Mi Plugin",
  "version": "1.0.0",
  "author": "Tu Nombre",
  "description": "Abre enlaces de GitHub en Google Chrome"
}
```

### Compilar el plugin

Dentro de la carpeta de tu plugin, ejecuta:

```sh
cargo build --release --target wasm32-unknown-unknown
```

Esto crea el archivo `.wasm` en:

```
target/wasm32-unknown-unknown/release/mi_plugin.wasm
```

(El nombre puede cambiar según el nombre de tu proyecto; los guiones suelen cambiar por guión bajo `_`.)

### Instalar un plugin en Browseraptor

1. Abre Browseraptor.
2. Abre el panel de plugins.
3. Haz clic en **+ Dev**.
4. Elige el archivo `.wasm` de tu plugin.
5. Browseraptor revisa que el plugin sea válido y lo guarda en la configuración local.
6. Si quieres quitarlo, puedes desinstalarlo desde el mismo panel.

Importante: el archivo `manifest.json` debe estar en la misma carpeta que el `.wasm` que seleccionas.

### Plugins con configuración

Si tu plugin necesita opciones (por ejemplo, reglas o un navegador por defecto), puede exportar estas funciones extra:

| Función | Para qué sirve |
| --- | --- |
| `get_config()` | Devuelve la configuración actual del plugin en JSON. Browseraptor la usa para mostrar la pantalla de configuración. |
| `set_config(ptr, len)` | Recibe una nueva configuración en JSON y devuelve una respuesta en JSON. |

`get_config` debe devolver un JSON con la configuración. `set_config` debe devolver una respuesta así:

- Si todo salió bien:

```json
{ "ok": true }
```

- Si hubo un problema:

```json
{ "ok": false, "error": "Mensaje del error" }
```

Si tu plugin exporta estas funciones y es un plugin local con archivo `.wasm`, Browseraptor puede mostrar el botón **Configure** para editar su configuración.

### Plugins de traducción

Un plugin de traducción cambia los textos de la interfaz de Browseraptor a otro idioma.

Para hacer uno, el plugin debe exportar la función:

| Función | Para qué sirve |
| --- | --- |
| `get_ui_strings()` | Devuelve un objeto JSON con las traducciones: clave y texto. |

Ejemplo de lo que debe devolver:

```json
{
  "plugins.title": "PLUGINS",
  "plugins.empty": "No plugins installed yet.",
  "plugins.configure": "Configure",
  "plugins.save": "Save",
  "plugins.cancel": "Cancel"
}
```

Cómo funciona:

- Cada clave (por ejemplo `plugins.save`) es el nombre interno de un texto de la interfaz.
- El valor es el texto que quieres mostrar.
- Browseraptor carga las traducciones al iniciar y también cuando instalas o desinstalas un plugin.
- Si una clave no está en el plugin, Browseraptor muestra el texto original en inglés.
- Si hay varios plugins instalados, Browseraptor usa las traducciones del primer plugin local instalado que devuelva datos válidos de `get_ui_strings`.

Claves comunes que puedes traducir:

- Títulos y mensajes: `plugins.title`, `plugins.empty`, `plugins.path`, `plugins.error`
- Botones y acciones: `plugins.open_folder`, `plugins.configure`, `plugins.uninstall`, `plugins.add_rule`, `plugins.copy`, `plugins.save`, `plugins.cancel`
- Pantalla de configuración: `plugins.configure_title`, `plugins.default_browser`, `plugins.default_profile`, `plugins.rules`, `plugins.domain`, `plugins.profile`, `plugins.select`, `plugins.select_browser`, `plugins.default`
- Errores: `plugins.invalid_json`, `plugins.invalid_response`, `plugins.read_config_error`, `plugins.load_error`, `plugins.save_error`

Consejo: puedes empezar traduciendo solo algunas claves y agregar más después.

### Problemas comunes

- **No puedo instalar el plugin:** revisa que `manifest.json` esté en la misma carpeta que el archivo `.wasm`.
- **No aparece el botón Configure:** tu plugin debe exportar `get_config` y `set_config`.
- **La traducción no se ve:** revisa que el plugin exporte `get_ui_strings` y que devuelva un JSON válido con claves y textos.
- **El plugin no hace lo que quiero con un enlace:** revisa el JSON que devuelve `evaluate` y que el nombre del navegador sea correcto.
- **Compiló pero no encuentro el `.wasm`:** busca en `target/wasm32-unknown-unknown/release/`.

## Reglas para el código

Si vas a cambiar el código de Browseraptor (no solo crear un plugin):

- Usa `rustfmt` para dar formato al código.
- Ejecuta `cargo clippy` y corrige las advertencias.
- Evita agregar cosas que no se necesitan.
- Borra código que ya no se use.
- No pongas rutas o valores fijos si deberían poder cambiarse.
- Para mensajes internos del programa se usa `tracing`, no `println!`.
- Usa mensajes de commit claros, por ejemplo:

```
feat: add browser icon cache for macOS
fix: prevent crash when no browsers are detected
docs: update plugin documentation
```

Prefijos comunes: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`, `style:`, `ci:`.

## Trabajos automáticos (Workflows)

Este proyecto usa GitHub Actions. Los archivos están en `.github/workflows/`:

- **`build-binaries.yml`**: compila Browseraptor para Linux, macOS y Windows y puede crear un release en GitHub.
- **`ci.yml`**: se ejecuta cuando se sube una etiqueta `v*` o cuando se ejecuta manualmente.

Al ejecutarlo manualmente se pueden usar las opciones `publish-release`, `tag` y `prerelease`.

Antes de abrir un pull request, revisa que:

- `cargo build --release --locked` funcione en tu sistema.
- Hayas ejecutado `cargo clippy` y corregido las advertencias.
- Si agregaste una dependencia nueva, expliques por qué es necesaria.
