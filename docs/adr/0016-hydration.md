# ADR 0016 - Hidratación de DOM servido por SSR

Fecha: 2026-11-17
Estado: **Aceptada** como MVP de hidratación; sin estado serializado ni fragmentos dinámicos.

## Contexto
`mount` construye el DOM desde cero. Sobre una página SSR eso duplica el árbol, pierde el HTML ya pintado y descarta identidad/foco. Faltaba el puente SSR → WASM que el proyecto promete.

## Decisión

1. `colibri_dom::hydrate(view, container)` recorre los hijos existentes con un cursor y **adopta** cada nodo:
   - texto: exige un nodo de texto y corrige su contenido;
   - elemento: exige la misma etiqueta (sin distinguir mayúsculas), reaplica atributos y registra eventos;
   - `Fragment`: recorre en orden;
   - `Scoped`: mueve los effects al `MountHandle`;
   - `Dynamic`/`Keyed`: error explícito (no hidratables aún).
2. Al terminar, el cursor debe haber consumido todos los hijos; nodos sobrantes o faltantes son error de hidratación.
3. Los efectos dinámicos se ejecutan al adjuntarse, así que el valor SSR se corrige si no coincide.
4. Si falla, se sueltan listeners/effects ya creados y **no se borra** el DOM SSR.
5. El `MountHandle` resultante posee los nodos existentes: al desmontar se eliminan, igual que un montaje normal.

## Alternativas
- Reconstruir y reemplazar: simple, pero pierde SSR, foco y causa parpadeo.
- Diff genérico SSR/cliente: correcto a largo plazo; exige reconciliación estructural que aún no existe.
- Hidratar sólo islas con marcadores: menos código, pero no valida el árbol completo.

## Consecuencias
- Se conserva la identidad de nodos y el foco; el HTML SSR no se recrea.
- Los hooks se ejecutan otra vez en cliente: el estado arranca en sus valores iniciales. **No hay serialización de estado**; un SSR con `5` y estado inicial `0` se corrige a `0` al hidratar.
- El árbol debe coincidir exactamente (etiquetas, orden; los atributos se reaplican). Texto con espacios implícitos entre etiquetas rompe el cursor.
- Sin `Dynamic`/`Keyed`, sin streaming, sin Suspense y sin límites de error de hidratación.

## Validación
- E2E Playwright `hydrates server-rendered DOM without losing nodes`:
  - marca `data-*` puesta antes del click sobrevive a la hidratación y al update;
  - el valor SSR `0` pasa a `1` tras el click;
  - un solo `<section>` bajo `#hydrated`, sin duplicados ni errores de consola.
- Fixture: `crates/colibri-dom/demo/hydrate.html` con marcado estático equivalente al componente `HydratedCounter`.
