# Roadmap Colibrí: hitos por evidencia

**Estado:** plan; no fechas prometidas. El trabajo se ejecuta por contratos y condiciones de aceptación, no por cantidad de archivos. Tareas concretas: [tasks/README.md](tasks/README.md).

| Fase | Objetivo demostrable | Dependencias |
|---|---|---|
| M00 | CI y baseline de workspace verdes y reproducibles | fundación |
| M01 | Template AST estructurada con spans y errores | M00 |
| M02 | `.colibri` estático → IR validada | M01 |
| M03 | macro `view!`/componente Rust → IR compatible | M02 contrato IR |
| M04 | suite de paridad Rust/SFC + HTML estático | M02, M03 |
| M05 | señales/scopes, reentrancia, memo/effect y scheduler mínimo | M00; compatible con IR |
| M06 | SSR seguro y dinámico sobre IR con tests XSS | M02, M05 para dinámico |
| M07 | DOM browser/WASM y cleanup montable | M04, M05 |
| M08 | CLI build/dev, SCSS y plugin Vite | M07; CSS preliminar paralelo |
| M09 | static-first, SSR/islands, assets diferidos | M06, M07, M08 |
| M10 | benchmarks reproducibles y análisis de coste | baseline temprano; completo tras M09 |

## Versiones como capas de madurez
- **Foundation (v0.0.1):** prototipos aislados. No vender como framework completo.
- **Compiler alpha:** M01–M04: dos sintaxis producen la misma salida estática.
- **Runtime alpha:** M05–M07: interactividad DOM real, con cleanup.
- **Developer preview:** M08: flujo end-to-end con Vite/SCSS.
- **Research beta:** M09–M10: SSG/SSR/islands y benchmarks publicados.
- **1.0 candidata:** compatibilidad, seguridad, accesibilidad, DX, documentación, política semver y mantenimiento.

## Plan de trabajo paralelo
- Parser y fixtures M01 bajo ownership exclusivo; core M05 por un segundo agente en crate separado.
- SSR security tests y benchmark design pueden investigarse en paralelo sin cambiar contratos.
- M02 y M03 deben compartir el *mismo* modelo IR y no tomar decisiones incompatibles sin ADR.
- M08 no inicia transformación de templates por su cuenta; consume CLI/codegen.

## Condición para avanzar
Tests unitarios, negativos y de equivalencia del hito pasan en CI; `docs/STATUS.md` refleja qué es realidad; limitaciones y PR de handoff documentados; hay ADR para cambios semánticos.

## Riesgos principales
Rust macro hygiene/spans; parsing SFC con strings complejos; graph reentrancy; streaming SSR seguro; estado/closure serializable; coste WASM startup; integraciones Vite con Cargo; regresiones de memoria por listeners.

## Decisiones abiertas
Ver [open-questions.md](open-questions.md) para no improvisar APIs públicas.
