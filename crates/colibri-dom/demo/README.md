# Demo: contador WASM

Componente `Panel` con children, `use_state`, `use_memo`, `use_async`, `Dynamic`, `Keyed` y `on:click`.

La página expone `window.colibri` con `mount_app`, `unmount_app` y `stats_json` para probar teardown y contadores del runtime desde Playwright.

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
- toggle: condicional `1 → 0 → 1`
- keyed: `A,B,C → C,B,A` conservando el nodo y el foco de `B`
- teardown: 25 ciclos mount/unmount dejan `stats()` en cero y `#app` vacío
- teclado: Enter/Space activan, `Shift+Tab` mueve el foco
- hidratación: `/hydrate.html` adopta el HTML SSR y conserva los nodos
- islas: `/islands.html` hidrata dos widgets independientes y los desmonta sin afectarse
- sin errores de consola ni wrappers `div`.

E2E automatizado:

```sh
npm ci
npx playwright install chromium
npx playwright test
```

Captura: [counter.png](counter.png).

`pkg/` es generado y no se versiona.
