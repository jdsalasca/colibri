# ADR 0019 - Contexto, `use_reducer` y refs de DOM

Fecha: 2026-11-17
Estado: **Aceptada** para paridad React básica.

## Contexto
Faltaban tres primitivas que cualquier app React usa a diario: contexto para evitar prop drilling, `useReducer` para estado con acciones, y refs para tocar el nodo una vez montado.

## Decisión

### Contexto
1. `provide_context<T: Clone + 'static>(value)` apila el valor en un TLS tipado (`TypeId`).
2. `use_context<T>()` busca el valor más reciente del tipo.
3. `#[component]` envuelve cada cuerpo en `with_context_scope`: los `provide` mueren al terminar el componente, también con panic.
4. `Dynamic` y `Keyed` abren un scope por render, así un `provide_context` dentro de su closure no se filtra.
5. El provider es una llamada al inicio del cuerpo, no un elemento `<Provider>`: los children se construyen antes de llamar al componente padre, así que un elemento Provider no podría verlos.

### `use_reducer`
`use_reducer(initial, |state, action| next)` devuelve `ReducerState` clonable con `get()`, `dispatch(action)` y `signal()`. Sigue el mismo motor de señales: memos y effects reaccionan igual.

### Refs
1. `ref={move |element| ...}` en `dom!` (solo backend DOM; `view!` lo rechaza).
2. Los callbacks se encolan al crear el elemento y se ejecutan al final del montaje, cuando el árbol ya está conectado. Así `focus()` funciona.
3. En `Dynamic`/`Keyed` se ejecutan al insertar el subárbol en un padre conectado; en el render inicial se dejan al montaje raíz.
4. Si el montaje falla, los refs encolados se descartan. No hay cleanup de ref.

## Alternativas
- `<Provider>` como elemento: imposible con children eager; exigiría cambiar todo el modelo a closures.
- Contexto global mutable: sin scoping, propenso a fugas entre componentes.
- `use_reducer` como wrapper de `use_state`: es exactamente eso, pero con API de acciones.
- Refs con `use_ref` + objeto mutable: más ceremonia; el callback cubre focus, medidas y canvas.

## Consecuencias
- Contexto funciona igual en SSR (`view!`/`Node`) y en DOM; no hay hidratación de contexto (se reconstruye al ejecutar componentes).
- Un `provide_context` en una closure normal (no componente ni `Dynamic`) se filtra hasta el siguiente scope; documentado.
- Los refs no se re-ejecutan si el nodo se reutiliza (Keyed); corren una vez por montaje/hidratación.
- `ReducerState` no implementa `PartialEq` ni `Display`; usa `get()` o `signal()`.

## Validación
- Core: contexto (3 pruebas: nearest, panic, tipos) y reducer (dispatch + effect).
- Macros SSR: `context_flows_to_children_and_is_scoped` (`dark` dentro, `light` fuera).
- E2E `context theme and ref focus work like React primitives`: el input recibe foco al montar y el tema cambia claro ↔ oscuro mostrando el contexto.
- Suite: 98 pruebas de host + 7 E2E.
