# Demo: contador WASM

Componente `Panel` con children, `use_state`, `use_memo` y `on:click`.

## Construir

Desde la raíz del repo:

```sh
cargo build -p colibri-dom --example counter --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir crates/colibri-dom/demo/pkg \
  target/wasm32-unknown-unknown/release/examples/counter.wasm
```

Requiere `rustup target add wasm32-unknown-unknown` y `wasm-bindgen` 0.2.130 en PATH.

## Servir y probar

```sh
cd crates/colibri-dom/demo
python -m http.server 3211 --bind 127.0.0.1
```

Abre <http://127.0.0.1:3211/>. Click en `+1`:

- valor: `0 → 1`
- doble: `0 → 2`
- async: `cargando catálogo… → 42 manuales listos`
- sin errores de consola.

Captura: [counter.png](counter.png).

`pkg/` es generado y no se versiona.
