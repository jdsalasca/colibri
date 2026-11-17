# colibri-cli

Herramientas de desarrollo de Colibrí.

```sh
colibri check page.colibri
colibri render page.colibri [--bind key=value]...
colibri build page.colibri [--out page.rs]
colibri eval page.colibri
```

`build` emite un programa Rust; `eval` lo compila y ejecuta. No es un compilador completo de `.colibri` a WASM.
