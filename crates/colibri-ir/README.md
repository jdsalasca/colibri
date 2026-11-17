# colibri-ir

Modelo semántico compartido por las rutas SFC y Rust.

- `Node`, `Element`, `Attribute` y `SourceSpan`.
- Sin DOM, navegador ni sistema de archivos.
- `SourceSpan::text` permite mapear diagnósticos al fuente.

```sh
cargo test -p colibri-ir
```
