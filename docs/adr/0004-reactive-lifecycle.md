# ADR 0004 — Ownership de estado y cleanup

Fecha: 2026-11-16
Estado: **Propuesta**, semántica detallada pendiente de M05.

## Contexto
Rust no recolecta automáticamente listeners, tareas asíncronas o ciclos lógicos retenidos por closures. Un framework reactivo necesita un criterio único y visible de ciclo de vida.

## Decisión propuesta
Cada instancia montada posee un Scope; signals, subscriptions, efectos, listeners y tareas asociadas se liberan/cancelan al desmontar. El Core mantiene su semántica sin conocer el DOM.

## Alternativas
- Cleanup sólo por GC/WeakRef: comportamiento menos determinista.
- Cleanup manual por el desarrollador: mayor carga y probabilidad de fugas.
- Scopes propietarios y cleanup explícito: complejidad interna a cambio de previsibilidad.

## Consecuencias
Hay que especificar reentrancia, orden LIFO, errores de callbacks, async cancel y ciclos de Rc. Evitar prometer thread-safety Wasm no validada.

## Validación
M05 pruebas de ciclo/reentrancia y M07 pruebas de 10.000 mounts/unmounts; no aprobar optimizaciones que pierdan cleanup.
