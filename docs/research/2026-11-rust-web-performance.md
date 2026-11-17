# Investigación técnica: Rust/WASM y rendimiento web (noviembre de 2026)

**Tipo:** síntesis de fuentes primarias + hipótesis y plan de experimentos. **Estado:** no hay benchmarks Colibrí vs frameworks en este documento. No convertir hipótesis en resultados. Evidencia consultada el 16 de noviembre de 2026.

## 0. Pregunta y objetivo social

¿Cómo minimizar *bytes transferidos + tiempo de arranque + CPU + memoria y energía* en aplicaciones accesibles, particularmente en computadores de 2–4 GB RAM? Colibrí adopta Rust-first sin convertir WebAssembly en religión. El límite inferior real lo imponen HTML/CSS/DOM, layout, pintura, tipografías, imágenes, red, V8, procesos Chromium y el trabajo funcional de la app.

### Hipótesis medibles
- **H1 — localización:** una señal con un único consumidor cambia un text node sin reconstruir subárboles; menor trabajo que un baseline React comparable cuando hay UI grande y cambio pequeño.
- **H2 — estático:** página sin interactividad debe emitir 0 bytes de runtime de Colibrí en navegador, sea JS o WASM.
- **H3 — puente JS/WASM:** una operación DOM de pocos pasos puede ser más rápida en JS directo que en WASM si cruza demasiado la frontera; medir por tamaño de lote.
- **H4 — memoria:** después de 10.000 mount/unmount, las suscripciones y callbacks activos vuelven a su baseline; RSS de Chromium se estabiliza dentro del ruido medido.
- **H5 — memo:** derivaciones costosas que tienen varios consumidores ganan con cache/equality; expresiones triviales pueden ser más lentas si siempre se memoizan.
- **H6 — paquete:** `opt-level=s/z`, LTO, `codegen-units=1`, `wasm-opt`, strip y compresión deben compararse contra `opt-level=3` en bytes y latencia, nunca optimizar tamaño ignorando CPU.

## 1. Evidencia externa confirmada

### Chromium / V8 — el binario WASM no es gratis
- V8 compila WASM en distintos niveles: baseline **Liftoff** para arrancar rápido y optimización **Turboshaft** para código caliente. El tiempo total de arranque también incluye obtención, validación, inicialización y carga de bindings.
- La documentación de V8 describe streaming/compilation tiering; *el hecho de compilar WASM no demuestra menor latencia end-to-end respecto a JS*.
- Fuente primaria: https://chromium.googlesource.com/v8/v8/+/main/docs/wasm/architecture.md
- Experimento: performance marks para fetch/compile/instantiate y primera interacción; comparar WebAssembly.instantiateStreaming con JS mínimo bajo diferentes cachés y redes.

### Rust ↔ JS ↔ DOM
- `wasm-bindgen` permite invocar Web APIs/JS desde Rust; `web-sys` expone API web. La frontera y las conversiones de strings/objects tienen coste y exigen medición.
- Fuentes: https://rustwasm.github.io/docs/wasm-bindgen/ y https://wasm-bindgen.github.io/wasm-bindgen/benchmarks/
- Experimento: 1, 10, 100, 1.000 actualizaciones de texto, atributos y clases desde JS directo, Rust+WASM+bridge, y Rust que minimiza viajes; trace scripting, bindings, layout y GC.

### Tamaño de paquete
- El libro Rust/WASM recomienda medir LTO, `opt-level=s` y `z`, `wasm-opt -Os/-Oz/-O`; advierte de trade-offs de build speed vs tamaño vs runtime. No implica que una combinación sea siempre la mejor.
- Fuente: https://rustwasm.github.io/docs/book/reference/code-size.html
- Experimento: usar misma app, registrar bytes `.wasm`/JS glue br/gzip, memoria al arranque, latency p95, builds cold/warm para cada configuración. No eliminar símbolos de depuración sin sourcemaps/proceso de diagnóstico.

### React — aciertos y restricciones
- `useState` se integra con reconciliación; `useMemo` evita recomputaciones pero su caché es una optimización, no un fundamento semántico. React Compiler puede memoizar muchos casos automáticamente. `useEffect` existe para sincronización externa, no para expresar estado derivado; el cleanup es crucial.
- Fuentes: https://react.dev/reference/react/useState ; https://react.dev/reference/react/useMemo ; https://react.dev/reference/react/useEffect
- Aprender: ergonomía declarativa, tooling y scheduling; evitar introducir arrays de dependencias y render general obligatorio si el grafo fino ofrece menor coste medido.

### Solid y Svelte 5 — grafo fino y dependencias dinámicas
- Solid `createSignal/createMemo/createEffect` detecta señales leídas síncronamente; elimina dependencias de ramas no recorridas; memos pueden filtrar cambios por igualdad. Effects se ejecutan después de cálculos puros.
- Svelte 5 `$state/$derived/$effect` también distingue derivación pura de efectos y advierte contra usar efectos para derivar estado. Usa un enfoque push/pull para derivaciones y batching del runtime.
- Fuentes: https://docs.solidjs.com/advanced-concepts/fine-grained-reactivity ; https://docs.solidjs.com/reference/basic-reactivity/create-memo ; https://docs.solidjs.com/reference/basic-reactivity/create-effect ; https://svelte.dev/docs/svelte/$derived ; https://svelte.dev/docs/svelte/$effect
- Aprender: tracking dinámico, ownership y *glitch avoidance*; medir el precio de almacenar un observador por dependencia, no asumir superioridad universal.

### Leptos — referencia Rust directa
- Leptos 0.8 describe signals, memos, effects y ownership de recursos. Su `Memo` es lazy y evita avisar a dependientes si el resultado no cambia; recomienda valores derivados sencillos cuando memo agrega más overhead que beneficio.
- El `Owner` administra efectos, limpieza y arena; la API de Leptos sirve para estudiar cómo compatibilizar Rust ownership con vistas reactivas.
- Fuentes: https://docs.rs/leptos/latest/leptos/reactive/index.html ; https://docs.rs/leptos/latest/leptos/prelude/struct.Memo.html ; https://docs.rs/leptos/latest/leptos/prelude/struct.Owner.html
- Pregunta para Colibrí: ¿memo eager o lazy? Nuestro prototipo usa eager y puede recalcular sin lectores. Medir en escenarios *unused derived* y *multiple consumers* antes de estabilizar API.

### Dioxus, Qwik, Astro
- Dioxus documenta web Rust/WASM con tamaño de bundle web del mismo orden que React en un ejemplo concreto: **Rust no equivale a bundle pequeño automáticamente**.
- Qwik: resumabilidad, estado serializado y lazy loading pueden reducir el trabajo cliente inicial; Rust closures arbitrarios no son automáticamente serializables.
- Astro: static-first y arquitectura de islas permiten evitar hidratación global donde no hace falta.
- Fuentes: https://dioxuslabs.com/learn/0.7/guides/platforms/web/ ; https://qwik.dev/docs/concepts/resumable/ ; https://docs.astro.build/en/concepts/islands/
- Pregunta: ¿qué combinación de SSR estático + islands ofrece mejor web en CPU limitada sin demasiado chunking y descarga?

## 2. Análisis crítico de Colibrí Reactive Core (prototipo M05)

- `use_state` es un `Signal<T>` independiente del orden de llamadas, no un Hook React.
- `use_memo` compara resultados `PartialEq` y los mantiene **eager**; podría desperdiciar CPU si se actualiza una dependencia pero nadie lee el memo.
- `use_effect` registra fuentes leídas **sincrónicamente**; al rerun limpia viejas dependencias. No rastrea `async` o closures diferidas.
- `batch` difiere la propagación automática hasta el final del bloque; suscriptores manuales aún reciben cambios intermedios.
- Orden: se procesan memos antes de efectos, con cola y deduplicación; no es un scheduler concurrente, ni garantía formal de ausencia de glitches bajo cualquier grafo con feedback.
- Ciclos: el límite `MAX_FLUSH_RUNS` interrumpe loops de efectos con pánico diagnóstico; se necesita API de error controlado para release.
- `Scope` permite ownership de `Effect`; quien no retenga el handle pierde la suscripción. No hay memoria de arena, IDs generacionales ni optimizaciones verificadas.
- No usar este prototipo como evidencia de rendimiento. Es baseline funcional/correctness, sin tests Chromium.

## 3. Diseño de benchmark falsable

### Escenarios
| Escenario | Medida principal | Benchmark contrastable |
|---|---|---|
| 10k signals con 1 consumidor | memoria por nodo y µs de cambio | Rust core vs Leptos reactive_graph |
| 1k observadores / 1 señal | coste de fan-out, allocations | misma semántica y warmup |
| Branch switch | subscribers residuales | calidad/corrección |
| 10k mount/unmount | ciclos retenidos y heap/RSS | release; sin DevTools abierto |
| Memo no leído | CPU de invalidación | eager Colibrí vs lazy Leptos |
| 1k list keyed | JS/WASM/DOM P95, memory | js-framework-benchmark compatible |
| Página 0 interactividad | transfer/CPU idle | Astro/plain HTML/Colibrí static |
| Dashboard dinámico | input delay, memory, bytes | React/Solid/Svelte/Leptos/Dioxus |

### Protocolo de reproducibilidad
1. Registrar SHA, navegador exacto, SO, CPU/RAM, plan de energía, modo de compilación, red y caché; equiparar HTML/CSS/acciones.
2. Usar ≥20 corridas tras warmup y reportar mediana, p95 y dispersión con raw results; no comparar con DevTools pesado abierto.
3. Separar RSS navegador/JS heap/WASM linear memory; una disminución de JS heap puede coexistir con aumento de WASM memory.
4. Analizar por separado CPU scripting, style/layout, paint, startup, bundle size, memory peak, idle y build time.
5. Medir cold vs warm, offline/cache y CPU throttling de Chromium; contrastar en 2–4 GB RAM y un ordenador moderno.
6. Preservar resultados donde Colibrí *pierda*; exigir equivalencia funcional y accesibilidad previa.
7. Benchmark de referencia: https://github.com/krausest/js-framework-benchmark ; resultados publicados https://krausest.github.io/js-framework-benchmark/ (comparar siempre misma versión Chrome, no diferentes releases).

## 4. Experimentos prioritarios para otros agentes

**E1 — correctness del núcleo (P0):** ejecutar CI y todos los tests de reentrancia, ciclo de vida y diamond graph. Añadir pruebas de panic recovery y observadores dinámicos autodesuscritos.

**E2 — grafo memory cost (P0):** instrumentar bytes por subscription, callbacks, Vec/HashSet/queues y allocations; comparar BTreeMap vs slab/índices generacionales con cargas realistas.

**E3 — modelo memo lazy/eager (P1):** benchmark con derivaciones raramente consumidas, ramas condicionales, memo chain y 10k derived. Propuesta ADR antes de cambiar API.

**E4 — política equality (P1):** comparar siempre-notify, `PartialEq`, comparator explícito, eventos con `Rc` y estructuras grandes. Evitar clones completos de objetos gigantes.

**E5 — frontera JS/WASM (P1):** trazar coste de web-sys/wasm-bindgen para batches de cambios DOM; no asumir superioridad wasm frente a JS para render granular.

**E6 — bundle size y startup (P1):** grid de cargo profiles, wasm-opt y split por isla; combinar con SSR/SSG sin cliente cuando no sea necesario.

## 5. Qué NO hemos demostrado
No hay ganador Colibrí/React/Solid/Leptos. No hay medición experimental de ahorro RAM/CPU, costo de energía o inicialización WASM. No hay backend DOM y el núcleo reactivo corre sólo en Rust nativo por ahora. Este documento reúne **fuentes y experimentos**, no resultados.

## Bibliografía primaria seleccionada
- V8 WASM architecture: https://chromium.googlesource.com/v8/v8/+/main/docs/wasm/architecture.md
- wasm-bindgen guide: https://rustwasm.github.io/docs/wasm-bindgen/
- wasm-bindgen crossing benchmarks: https://wasm-bindgen.github.io/wasm-bindgen/benchmarks/
- Rust/WASM code size: https://rustwasm.github.io/docs/book/reference/code-size.html
- React useEffect: https://react.dev/reference/react/useEffect
- React useMemo: https://react.dev/reference/react/useMemo
- Solid fine-grained reactivity: https://docs.solidjs.com/advanced-concepts/fine-grained-reactivity
- Svelte derived/effect: https://svelte.dev/docs/svelte/$derived , https://svelte.dev/docs/svelte/$effect
- Leptos 0.8 reactive graph: https://docs.rs/leptos/latest/leptos/reactive/index.html
- Dioxus web: https://dioxuslabs.com/learn/0.7/guides/platforms/web/
- JS framework benchmark docs: https://github.com/krausest/js-framework-benchmark/blob/master/README.md
