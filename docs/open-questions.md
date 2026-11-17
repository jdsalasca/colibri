# Registro de decisiones aún abiertas (no implementarlas por intuición)

| Pregunta | Alternativas | Decisión necesaria antes de |
|---|---|---|
| Sintaxis props de SFC | bloque Rust tipado / export macro / declaraciones en template | M03–M04 |
| Modelo de template expressions | Rust puro / DSL restringida | M02 |
| Identidad estable de componentes | keys explícitas y gen IDs / tree path | M04–M07 |
| Política de reentrancia de Signal | cola determinista / error ciclos / recursion bound | M05 |
| Memo equality | siempre invalidate / comparador opcional | M05 |
| Modelo de fallos de Effect | error tipado / policy de panic boundary | M05 |
| Surface de HTML seguro | escape exclusivo / SafeHtml auditado | M06 |
| URL allowlist | lista por atributo/namespace y contexto | M06 |
| Enlace DOM | web-sys + wasm-bindgen / alternativa | M07 |
| Scheduling UI | microtask / rAF / cooperative priority queue | M07 |
| Vite: Node obligatorio | opcional frontend plugin vs CLI standalone | M08 |
| Hot reload Rust | reload completo / preservación por IDs | M08 posterior |
| WASM module boundaries | por isla / por ruta / por app | M09 |
| Resumabilidad closures Rust | serialize state restringido / hydration selectiva | investigación futura |
| Compatibilidad React components | bridge explícito / no compatibilidad automática | post-preview |
| MSRV y licenciamiento | por adoptar ADR/gobernanza | antes de release público estable |

## Proceso de resolución
1. Escribir ADR en `docs/adr/` con problema, restricciones, opciones, trade-offs, benchmark si aplica y migración.
2. PR pequeño con tests de comportamiento e implicaciones para SFC y Rust.
3. Actualizar contratos afectados y registrar aceptación en changelog o STATUS.
4. No promover experimento a API pública sin aceptación documentada.
