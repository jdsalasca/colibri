# colibri-dom

Runtime DOM/WASM mínimo para vistas Colibrí.

- `View`: texto, texto dinámico, elemento y fragmento.
- `dom!`: misma sintaxis que `view!`, con `on:evento={closure}`.
- `use_async`: `Signal<Resource<T>>` con `Loading/Ready/Failed` y `spawn_local`.
- `mount(view, container)`: construye el DOM y actualiza bindings con `use_effect`.
- `MountHandle`: limpia listeners y effects al hacer drop.
- Sólo compila en `wasm32`; en host el crate queda vacío.

```sh
cargo build -p colibri-dom --example counter --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir crates/colibri-dom/demo/pkg \
  target/wasm32-unknown-unknown/release/examples/counter.wasm
```

Ver [`demo/README.md`](demo/README.md).
