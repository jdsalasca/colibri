# ADR 0013 - Dynamic fragments y reconciliación coarse

Fecha: 2026-11-17
Estado: **Aceptada** como primera reconciliación estructural.

## Contexto
`View` era estático: los bindings de texto/atributos cambiaban, pero una lista o un condicional no podía añadir/quitar nodos. Terra señaló además el riesgo de desmontar la vista anterior antes de confirmar la nueva.

## Decisión

1. `View::DynamicFragment(Rc<dyn Fn() -> Vec<View>>)`.
2. `dom!` acepta `<Dynamic nodes={closure}/>`; la closure devuelve `Vec<View>`.
3. En montaje se crean dos comentarios ancla (`colibri:start`/`colibri:end`). En cada cambio:
   - se monta la nueva lista en un contenedor temporal **detached**,
   - si el montaje tiene éxito, se desmonta el anterior y se mueven los nuevos nodos antes del ancla `end`,
   - el handle adopta el padre real para el cleanup,
   - si falla, se conserva el anterior y se reporta por `console.error`.
4. `Fragment` y `Dynamic` rechazan `nodes` duplicado con span del segundo atributo.
5. `view!`/SSR rechaza `Dynamic`: no hay evaluación de closures en servidor en esta fase.

## Alternativas
- Reemplazar `innerHTML`: pierde listeners y foco, y no permite limpieza fina.
- Diff keyed completo: correcto a largo plazo, pero mucho más código y pruebas.
- Anclas/comment nodes sin wrapper: evita el `<div>`, pero requiere gestión de rangos y más pruebas de accesibilidad.

## Consecuencias
- Condicionales y listas ya cambian en cliente.
- La reconciliación es **coarse**: reconstruye el subárbol; no preserva foco, estado local ni identidad keyed.
- No hay wrapper `<div>`: se usan comentarios ancla, así que tablas, listas y `<select>` conservan su estructura.
- No hay hidratación SSR→cliente ni SFC cliente.
- Los errores de montaje dinámico se reportan por consola; no hay canal de error de aplicación todavía.

## Validación
- Demo Chrome: condicional `1 → 0 → 1`; contador `0 → 1`; doble `0 → 2`; async `42`; sin errores de consola.
- Pruebas host: 84; build wasm y CI.
- Revisión Terra: ver [`docs/reviews/2026-11-17-terra.md`](../reviews/2026-11-17-terra.md).
