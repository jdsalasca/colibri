# ADR 0014 - Keyed views con identidad y foco

Fecha: 2026-11-17
Estado: **Aceptada** para reconciliación keyed inicial.

## Contexto
`Dynamic` reconstruye todo el subárbol. En una lista, reordenar destruye nodos, listeners y foco. Terra lo señaló como el siguiente salto de reconciliación.

## Decisión

1. `KeyedView { key: String, view: View }`.
2. `View::KeyedFragment(Rc<dyn Fn() -> Vec<KeyedView>>)`.
3. `dom!` acepta `<Keyed nodes={closure}/>`; la closure devuelve `Vec<KeyedView>`.
4. Montaje:
   - anclas de comentario `colibri:keyed:start/end`;
   - `HashMap<key, { MountHandle, roots }>`;
   - una key existente **reutiliza sus nodos** y los mueve antes del ancla `end` en el nuevo orden;
   - una key nueva se monta en un contenedor detached, se mueven sus nodos y el handle adopta el padre;
   - una key eliminada se desmonta;
   - keys duplicadas se reportan por consola y se descartan.
5. Antes de mover se guarda `document.activeElement`; si sigue conectado, se restaura el foco.

## Alternativas
- Reconstruir todo (`Dynamic`): simple, pero pierde foco/identidad.
- Diff por índice: rompe identidad al reordenar.
- Component instances con estado keyed: requiere un runtime de componentes; fuera de esta fase.

## Consecuencias
- Reordenar preserva nodos, listeners y foco de las keys existentes.
- **No se re-renderiza el contenido de una key existente.** Si el contenido cambia, debe venir de señales reactivas o la key debe incluir la versión del dato.
- No hay diff de contenido ni estado local por key; sigue siendo una reconciliación de nivel lista.
- No hay hidratación SSR ni SFC cliente.
- Sin tests de memoria a largo plazo; E2E cubre orden, identidad y foco.

## Validación
- E2E Playwright/Chromium:
  - lista `A,B,C` → `C,B,A` tras reordenar;
  - el nodo marcado de `B` se conserva;
  - el foco permanece en el botón de `B`;
  - sin wrappers `div` ni errores.
- CI `validate + wasm + e2e`; 84 pruebas de host.
