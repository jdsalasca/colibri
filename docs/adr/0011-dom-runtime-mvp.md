# ADR 0011 - Runtime DOM/WASM mínimo

Fecha: 2026-11-16
Estado: **Aceptada** para M07 parcial.

## Contexto
El framework ya tenía componentes funcionales, props tipadas y SSR, pero ningún estado cliente: `use_state` cambiaba una señal y el HTML no se actualizaba. Faltaba un runtime DOM real para considerar la API React-like.

## Decisión

1. Nuevo crate `colibri-dom` para `wasm32`, con `wasm-bindgen` y `web-sys`.
2. `View` de runtime: texto estático, texto dinámico, elemento (atributos/eventos/hijos) y fragmento.
3. Macro `dom!` con la misma sintaxis que `view!`, pero genera `View`:
   - `{expr}` → `DynamicText(Rc<dyn Fn() -> String>)`.
   - atributo `{expr}` → `Attribute::Dynamic`.
   - `on:evento={closure}` → `EventBinding`.
   - componentes y children funcionan con `#[component]` (el componente devuelve `View`).
4. `mount(view, container)` construye el DOM una vez:
   - texto/atributos dinámicos se actualizan con `use_effect` de `colibri-core`.
   - eventos usan `Closure` + `add_event_listener_with_callback`.
   - `MountHandle` guarda effects/listeners y los limpia en `Drop`.
5. El contador de demostración usa `use_state`, `use_memo`, un componente `Panel` con children y un `on:click`.

## Alternativas
- Re-render completo de `innerHTML`: simple, pero pierde foco, listeners y estado del DOM.
- Virtual DOM con diff: más general, pero mucho más código y sin necesidad para bindings puntuales.
- Runtime JS y componentes TS: rompe el objetivo Rust-first.
- HTMX/SSR-only: útil para formularios, pero no para estado local interactivo.

## Consecuencias
- Actualizaciones finas de texto/atributos sin reconstruir el árbol.
- La estructura es estática tras `mount`: no hay reconciliación para listas/condicionales que cambien de forma; hay que remontar o añadir diff en una fase posterior.
- No hay hidratación SSR → cliente; el demo es client-only.
- `use_effect`/`use_memo` funcionan en componentes que devuelven `View`; los componentes SSR (`Node`) siguen siendo one-shot.
- `wasm-bindgen`/`web-sys` sólo se compilan en `wasm32`; el host sigue limpio.
- Sin tests E2E en CI todavía; la validación es Chromium + Playwright local.

## Validación
- Build `wasm32-unknown-unknown` + `wasm-bindgen 0.2.130`.
- Chrome real: `0 → 1`, `doble 0 → 2`, sin errores de consola.
- Captura `crates/colibri-dom/demo/counter.png`.
