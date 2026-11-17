# Contrato de reactividad y ciclo de vida

**Estado:** propuesta. Existe sólo Signal explícito + Scope; autotracking, Memo, Effect y Scheduler pendientes.

## Modelo
- `Signal<T>`: lectura/actualización de estado; operaciones definidas según ownership de Rust.
- `Memo<T>`: deriva valor de dependencias; pureza del cálculo; invalidación lazy/eager decidida por política documentada.
- `Effect`: sincroniza con mundo externo; contiene cleanup explícito; no usar para derivaciones puras.
- `Scope`: propietario de subscriptions, effects, listeners y operaciones asíncronas; desmontaje determinista.
- `Scheduler`: cola deduplicada de cambios, prioridad de interacción visible, protección contra ciclos.

## Semántica que debe decidirse ANTES de codificar
1. **Orden de notificaciones:** registrar determinista (preferible FIFO de suscripciones); batch no debe emitir estados imposibles entre cambios agrupados.
2. **Reentrancia:** callbacks pueden invocar `set`; no mantener `RefCell` prestado a lo largo de callbacks. Definir límites / cola para impedir recursión explosiva.
3. **Tracking:** la expresión reactiva sólo se suscribe a señales realmente leídas; limpiar dependencias anteriores en branches dinámicas.
4. **Equality:** política explícita para evitar actualizaciones redundantes, sin imponer `PartialEq` a todos los tipos.
5. **Errores:** ciclos reactivos diagnostican o detienen la ejecución con un error controlado, no colgar el navegador.
6. **Memos:** invalidación topológica y cero glitches: ningún observer ve un memo obsoleto una vez confirmado el batch.
7. **Desmontaje:** cleanup LIFO; unsubscribe idempotente; tareas abortables o respuestas etiquetadas con versión.
8. **Ambiente:** single-threaded como MVP; no prometer `Send + Sync` universal ni threads WASM sin infraestructura.
9. **Ownership:** no introducir leaks por `Rc` cíclicos; preferir `Weak` para enlaces al padre.
10. **Observabilidad:** hooks de instrumentación para conteo de subscripciones, efectos y operaciones, sin coste alto en release.

## Contrato observable MVP
- `signal.get()` devuelve último valor confirmado.
- `signal.set(v)` cambia valor, informa a observadores activos y retorna sin mantener borrows pendientes.
- `subscription.drop()` cancela notificaciones futuras.
- `scope.drop()` limpia sus recursos una vez, incluyendo bajo desmontajes anidados.
- `memo` y `effect`, cuando existan, tienen orden y limpieza documentados.

## Pruebas de corrección mínimas
- Suscribir y desuscribir dentro de un callback.
- Cambiar un signal desde un callback de otro signal y desde sí mismo, sin stack overflow.
- Dependencias cambiantes condicionadas por una señal.
- Cancelar effect, borrar componente y verificar que no recibe eventos.
- Repetir 10.000 ciclos mount/update/unmount; contar listeners y subscriptions; memoria estable.
- Probar orden de efectos y ausencia de estados intermedios inconsistentes con batching.
- Test con closures que capturan valores y recursos Rust; verificar que Drop corre exactamente una vez.

## Plan de rendimiento
No asumir que más granularidad siempre es mejor. Comparar grafos grandes, cambios puntuales, cambios masivos y coste de memoria de cada subscription con Solid/Leptos y una implementación simple en JS.

## Límite
La implementación actual **no** cumple toda esta especificación. Cada feature exige tests y actualización de STATUS.
