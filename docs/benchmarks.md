# Plan científico de rendimiento — sin resultados atribuidos

## Preguntas orientadoras
1. ¿Reduce un grafo reactivo granular el trabajo de DOM y CPU en cambios locales de UI?
2. ¿Cuándo compensa el costo de arranque WASM frente a JS optimizado o HTML estático?
3. ¿Cuánto memoria consume el propio framework en sesiones de larga duración y máquinas de bajos recursos?
4. ¿Puede un pipeline static-first servir páginas complejas con 0 runtime cliente?
5. ¿Cuál es el costo para el desarrollador (cold/warm build, recompilación, bundle) de las optimizaciones?

## Hipótesis falsables
- H1: al cambiar 1 binding entre 1.000, Colibrí hace menos trabajo de re-render que una implementación React idiomática, con latencia menor bajo entorno controlado.
- H2: en páginas sin interactividad, el runtime cliente generado es 0 bytes y la inicialización inferior a una SPA equivalente.
- H3: WASM no es necesariamente más rápido para cambios DOM pequeños por puente JS/WASM.
- H4: después de 10.000 montajes/desmontajes no queda crecimiento atribuible a listeners/subscriptions retenidos, dentro del ruido del entorno.

## Comparadores
React (versión actual fijada), Solid, Svelte, Leptos, Dioxus, Vue, implementación DOM en JS puro y Colibrí. Estudiar `https://github.com/krausest/js-framework-benchmark` sin optimizar injustamente un único competidor.

## Cargas comparables
- Static landing (contenido igual, CSS igual).
- Keyed table 1.000/10.000 filas: create, update 1, update 10 %, reorder, delete, replace.
- Counter 1 y 1.000 bindings con cambios locales vs globales.
- Dashboard con charts diferidos y eventos rápidos.
- Mount/unmount x 10.000, navegar hacia/desde vista, 30 minutos idle.
- Entrada texto de alta frecuencia con filtros async; time to input responsiveness.
- Página renderizada server con y sin islas.

## Ambiente y métricas
Registrar: commit SHA, versiones de Rust/Node/Chromium, CPU/RAM/OS, frecuencia/power mode, resolución, red/latencia, caché frío/caliente, formato de compresión (brotli/gzip), profile release, workers y flags.

Medir: transfer bytes (HTML/CSS/JS/WASM separados), LCP/INP donde corresponda, JS/WASM startup, tiempo scripting/layout/paint, latencia input p50/p95/p99, CPU total y en reposo, memoria de procesos Chromium/RSS/heap WASM, nodos DOM, subscriptions residuales, allocs, picos de memoria y builds cold/warm. P95 necesita suficientes muestras y dispersión.

## Protocolo
1. Mismo diseño/HTML visible/funcionalidad y datasets; semántica y accesibilidad equivalentes.
2. Optimizar todas las variantes razonablemente y registrar configuraciones.
3. Calentar antes de medir; realizar >=20 corridas por escenario y ordenar aleatoriamente variantes cuando sea posible.
4. Reportar mediana, percentiles e intervalos/dispersión; incluir datos crudos, scripts y condiciones exactas.
5. Evitar medir con DevTools abierto si altera resultados; usar herramientas equivalentes para todas las variantes.
6. Repetir en al menos una máquina modesta y una moderna; no extrapolar resultados de desktop potente a 2 GB RAM.
7. Reportar regresiones y casos donde Colibrí pierda. **No publicar un "ganador" general sin definir workloads.**

## Gates
Antes de optimizar, obtener baseline reproducible. Cambios de alto riesgo requieren demostrar beneficio superior a ruido de medición, con correctness intacta. No establecer un umbral universal de bytes, FPS o RAM sin estudio.

## Repositorio de evidencia (futuro)
`benchmarks/fixtures/`, `benchmarks/harness/`, `benchmarks/results/YYYY-MM-DD/<commit>/`; README con condiciones y scripts. No subir datos de personas reales o secretos.

## Declaración obligatoria en PR de rendimiento
"Implementación comparada, versión/commit, máquina, comando, N corridas, métricas y limitaciones". Ausencia de evidencia = hipótesis, nunca hecho.
