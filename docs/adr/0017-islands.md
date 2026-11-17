# ADR 0017 - Islas hidratables sobre una página SSR

Fecha: 2026-11-17
Estado: **Aceptada** como activación selectiva mínima (M09 parcial).

## Contexto
`hydrate` adopta un árbol SSR, pero cablear cada región a mano obliga a repetir búsqueda de contenedor, manejo de ausencia y ownership de handles. Una página SSR real suele ser mayormente estática con unos pocos widgets interactivos.

## Decisión

1. `colibri_dom::hydrate_island(selector, view)`:
   - busca el primer nodo que casa con `selector`;
   - si no existe, devuelve `Ok(None)` (página sin esa isla);
   - si existe, hidrata `view()` dentro y devuelve `Ok(Some(MountHandle))`.
2. El llamador posee los handles (por ejemplo, un `Vec<MountHandle>` en el entrypoint WASM) y decide cuándo desmontarlos.
3. Cada isla es independiente: sus listeners, effects y nodos se limpian al soltar su handle sin tocar las demás.
4. No hay registro, manifiesto ni chunking por isla: un único bundle WASM sirve todas las islas de la página.

## Alternativas
- `mount` de toda la app: reconstruye el SSR y pierde la ventaja estática.
- Registro declarativo (`islands!` con tabla de selectores): útil cuando exista generación de manifiesto (M08/M09 completo); posponer.
- Hidratación automática por atributos `data-*`: requiere que el compilador emita marcadores; no existe aún.

## Consecuencias
- Activar una isla no duplica contenido ni pierde nodos del SSR (`hydrate` estricto).
- Desmontar una isla elimina su subárbol; las demás siguen intactas.
- Se heredan los límites de `hydrate`: sin `Dynamic`/`Keyed`, sin estado serializado, árbol debe coincidir.
- El coste de arranque WASM se paga una vez por página, no por isla; el bundle no se divide.
- Un selector inexistente es silencioso; un árbol que no coincide devuelve error con `hydration mismatch`.

## Validación
- E2E Playwright `hydrates independent islands on a server-rendered page` (`/islands.html`):
  - dos islas SSR (contador con texto coalescido y toggle con `class`/texto dinámicos);
  - cada una responde a su click sin afectar a la otra;
  - los `data-*` puestos antes de hidratar sobreviven;
  - `unmount_islands()` deja ambos contenedores vacíos y sin errores de consola.
