# colibri-codegen

Genera un programa Rust ejecutable desde un `.colibri`.

- El `<script lang="rust">` se compila con rustc dentro de `main`.
- Las expresiones `{expr}` se evalúan con `format!` y el tipado de Rust.
- El HTML final usa `colibri-ssr` (escape y validación incluidos).

```sh
cargo test -p colibri-codegen
```
