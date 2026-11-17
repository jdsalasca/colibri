# ADR 0006 - Ciclos recuperables e instrumentación reactiva

Fecha: 2026-11-16
Estado: **Aceptada** para el cierre contractual de M05.

## Contexto
El prototipo M05 ya tenía tracking sincrónico, memos eager con igualdad, effects, batching y limpieza. Dos huecos impedían cerrar su contrato: un ciclo reactivo terminaba en `assert!` (panic no recuperable) y no existía instrumentación para demostrar que 10.000 ciclos no dejan observadores residuales. `Scope` tampoco ofrecía un desmontaje explícito e idempotente.

## Decisión

1. **Ciclo recuperable:** al superar `MAX_FLUSH_RUNS` (4.096) el motor deja de ejecutar, vacía las colas, resetea el flag `queued` de cada observador pendiente y guarda `ReactiveError::CycleLimit { runs }` en un slot thread-local. No hay panic.
2. **Diagnóstico pull:** `take_error()` devuelve y limpia el último error del hilo. Las actualizaciones normales siguen funcionando después de un ciclo; el observador que lo provocó permanece suscrito y volverá a intentarlo en el próximo cambio.
3. **Instrumentación:** `stats()` expone `signals`, `subscriptions`, `observers` y `queued` por hilo. Los contadores se actualizan en creación/destrucción de `Signal`, `Subscription` y observadores; `queued` se lee del motor.
4. **Scope explícito:** `Scope::dispose()` ejecuta cleanups LIFO y es idempotente; `Scope::own(valor)` permite apropiarse de effects, memos o cualquier valor. `Drop` llama a `dispose()`.
5. **Memos:** se mantienen eager y con `PartialEq`; no se cambia a lazy en este ADR.

## Alternativas
- Mantener `assert!`: simple, pero un ciclo mata el proceso y no permite diagnosticar ni recuperar.
- Devolver `Result` desde `set`: más explícito, pero rompe la API de señales y obliga a propagar errores por todos los callbacks.
- Contadores globales atómicos: innecesarios para un runtime single-thread y añaden coste.
- Cerrar M05 sin instrumentación: impediría demostrar la ausencia de residuos con evidencia reproducible.

## Consecuencias
- El error es pull-based: quien no llame a `take_error()` no verá el diagnóstico. Se documenta como contrato.
- Un ciclo deja a los observadores vivos pero inactivos hasta el siguiente cambio; no se intenta desuscribirlos automáticamente porque el grafo puede ser legítimo tras corregir datos.
- Los contadores son por hilo y no incluyen tareas en ejecución en `queued`; son para tests y diagnóstico, no para métricas de producción.
- Los pánicos de código de usuario dentro de un effect siguen propagándose; falta una política de panic boundary (M07).

## Validación
- `reactive_cycles_record_an_error_and_the_queue_recovers`: ciclo real, `take_error()` presente y actualizaciones posteriores funcionando.
- `ten_thousand_mixed_cycles_leave_no_residuals`: 10.000 ciclos signal+memo+effect y `stats()` vuelve al baseline.
- `scope_dispose_is_lifo_and_idempotent`.
- 13 pruebas de `colibri-core`, 58 de workspace; `clippy -D warnings` limpio.
