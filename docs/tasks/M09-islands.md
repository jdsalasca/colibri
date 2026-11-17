# M09 — Static-first y activación selectiva

**Prioridad:** P2. **Depende de:** M06 + M07 + M08. **Área:** compiler delivery + SSR/router manifests. **Spec:** `docs/spec/rendering.md`.

## Objetivo
Entregar HTML/ CSS legible sin runtime cuando no hay interactividad; en páginas mixtas activar sólo islas necesarias. Resumabilidad de closures Rust es una investigación separada.

## Trabajo
1. Manifiesto de dependencias de cliente de cada región interactiva.
2. Bundling razonable por ruta o grupo de islas, no necesariamente un WASM por componente.
3. Activación controlada (interacción/visible/idle) con accesibilidad y manejo de eventos.
4. SSR/cliente no muestran contenido divergente o cambios bruscos injustificados.
5. Comparar tamaño extra de módulos, memoria, RTT y startup con estrategia app-wide.

## Aceptación
- [ ] Página 100 % estática descarga 0 JS y 0 WASM propios.
- [ ] Página mixta carga sólo bytes necesarios para su interacción.
- [ ] Navegación, teclado, focus y formulario funcionan con progressive enhancement.
- [x] Tests muestran cleanup y ausencia de estado duplicado tras activaciones (dos islas E2E, desmontaje independiente).
- [ ] Métricas reportadas con equipos modestos, no sólo laptop moderna.

## Implementado (parcial)

- `colibri_dom::hydrate_island(selector, view)`: hidrata la primera coincidencia y devuelve `Option<MountHandle>`; ausencia = `None`. Ver [ADR 0017](../adr/0017-islands.md).
- Demo `islands.html`: contador y toggle SSR independientes; `unmount_islands()` los limpia por separado.
- Pendiente: manifiesto de dependencias, chunking por ruta/isla, activación por interacción/visibilidad/idle y métricas.

## Handoff
Estrategia de chunks/manifest, comportamiento de fallos de carga y benchmarks antes/después.
