# colibri-ssr

Render HTML seguro desde `colibri-ir`.

- `render_static`: rechaza nodos dinámicos con span.
- `render_dynamic`: resuelve expresiones con un contrato del host.
- Escape contextual, URL allowlist y rechazo de atributos/tags peligrosos.

```sh
cargo test -p colibri-ssr
```
