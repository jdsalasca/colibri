# M10 — Benchmark suite y prueba de hipótesis

**Prioridad:** P0 para diseñar metodología; P2 para comparativas finales. **Depende de:** M00 para baseline, M09 para alcance completo. **Área:** `benchmarks/`. **Spec:** `docs/benchmarks.md`.

## Objetivo
Convertir la afirmación de eficiencia en evidencia reproducible; permitir que futuros agentes identifiquen regresiones y expliquen por qué ocurren.

## Trabajo
1. Diseñar harness y fixtures funcionalmente idénticos sin favorecer un framework.
2. Establecer versión de Chromium, tooling y hardware, warmup, N corridas, raw results.
3. Comparadores: React, Solid, Svelte, Leptos, Dioxus, Vue, JS DOM puro.
4. Medir startup, p50/p95/p99, transfer compressed bytes, CPU, RSS browser, WASM heap y build times.
5. Publicar scripts/runs y analizar ruido/uncertainty.
6. Identificar escenarios donde Colibrí pierde y qué partes justifican investigación adicional.

## Aceptación
- [ ] Un tercero puede repetir un fixture y generar el mismo formato de resultados.
- [ ] Diferencias se explican con métricas, no con slogans.
- [ ] No hay afirmaciones de superioridad antes de resultados medidos.
- [ ] Existen resultados en una máquina con recursos modestos y una moderna.
- [ ] Las pruebas de corrección preceden cada benchmark.

## Handoff
Commit SHA de harness, raw data, entorno, comandos, conclusiones acotadas y limitaciones.
