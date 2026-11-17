# ADR 0018 - Estado SSR para hidratación

Fecha: 2026-11-17
Estado: **Aceptada** como siembra de estado explícita; sin serialización automática.

## Contexto
Al hidratar, los hooks se ejecutan otra vez y arrancan en su valor inicial. Si el SSR mostró `7`, el cliente lo corrige a `0` al hidratar: parpadeo y estado divergente. La hidratación necesitaba un canal de datos servidor → cliente.

## Decisión

1. **Servidor.** `colibri_ssr::state_script(id, json)` emite:
   ```html
   <script type="application/json" id="colibri-state">{"counter":7}</script>
   ```
   - el `id` se restringe a letras ASCII, dígitos, `-` y `_`;
   - `</` se escapa a `<\/` para que el JSON no pueda cerrar el `<script>`;
   - el JSON lo produce la app (por ejemplo con `serde_json`); el framework no impone serializador.
2. **Cliente.** `colibri_dom::use_hydrated_state(key, initial)`:
   - lee el script una sola vez y cachea el objeto (`js_sys::JSON::parse`);
   - busca `key` con `Reflect::get`;
   - convierte con el trait `Seed` (`i32`, `f64`, `bool`, `String`);
   - si falta la clave o el tipo no coincide, usa `initial` (el mismatch se reporta por consola).
3. El resto de hooks (`use_state`, `use_memo`) no cambia: la siembra es explícita, clave por clave.

## Alternativas
- Serialización automática por orden de hooks: frágil y contraria al diseño sin índice de hooks.
- `serde_json` dentro del framework: añade dependencia y acopla versiones; la app ya sabe serializar.
- `data-*` por nodo: sin objetos ni tipos anidados, y contamina el marcado.
- `eval` del estado: riesgo de XSS; descartado.

## Consecuencias
- Los valores sembrados no parpadean ni se reinician al hidratar.
- La clave es responsabilidad del desarrollador; dos componentes con la misma clave comparten valor inicial (útil para islas, peligroso si se abusa).
- Solo se soportan primitivos planos; arrays/objetos por clave requieren ampliar `Seed`.
- El estado se lee una vez: modificar el script después no afecta.
- No hay cifrado ni firma: el estado es público, como cualquier HTML. No poner secretos.

## Validación
- Host: `renders_state_script_and_escapes_script_end` y `rejects_unsafe_state_script_id`.
- E2E `hydrates server-rendered DOM without losing nodes`: SSR `contador: 7` se conserva y pasa a `8` al hacer click; `data-*` sobrevive.
- E2E `hydrates independent islands...`: contador `3` y toggle `ON` vienen del script y mantienen su nodo SSR.
