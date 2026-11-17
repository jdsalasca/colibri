# AGENTS.md — reglas de trabajo para agentes IA y contribuidores

**Propósito:** framework web Rust-first, ligero, accesible y medible. **Compute only what changes. Load only what matters.**

## Lectura obligatoria antes de tocar código
`docs/START_HERE.md` → `docs/STATUS.md` → `docs/architecture.md` → especificación correspondiente en `docs/spec/` → tarea en `docs/tasks/README.md` → `docs/agent-workflow.md`.

## Estado real
La base del repositorio es **experimental**. No existe compilador funcional de `.colibri` a Rust/WASM, ni macro `view!`, navegador DOM, Vite/HMR, SSR dinámico o benchmark competitivo validado. Nunca presentar una API diseñada como código que ya compila. Actualizar `docs/STATUS.md` exclusivamente con pruebas.

## Reglas técnicas inquebrantables
1. Rust puro (`.rs`, `#[component]`, `view!`) y SFC (`.colibri`) convergen sobre la **misma semántica/IR**. No duplicar runtimes.
2. `colibri-core` es independiente del DOM, Vite, Node, Sass y código de parser.
3. El compilador se ejecuta en build-time. Sólo código necesario en el navegador. Cero client runtime para páginas totalmente estáticas.
4. Las señales, effects, suscripciones, eventos y tareas pertenecen a scopes con limpieza definida.
5. Escapar texto/atributos por defecto, validar URLs, respetar HTML/ARIA/teclado, evitar `innerHTML` con datos no confiables.
6. Ninguna optimización puede romper consistencia o accesibilidad. No asumir WASM > JS: comparar por workload y Chromium real.
7. No copiar código upstream sin revisar licencia. Clones bajo `precedentes/upstream/` (gitignored).
8. Nuevas dependencias, unsafe y decisiones de API requieren justificación, tests y ADR. El workspace actualmente prohíbe `unsafe`.
9. PRs pequeños con cambios localizados, pruebas y handoff; no force-push ni ediciones concurrentes de archivos compartidos.
10. Documentación preferentemente en español; API y nombres de crates en inglés.

## Protocolo operativo
- **Selecciona una tarea:** `docs/tasks/README.md`. Comprueba bloqueos y no implementes más de una dependencia conceptual a la vez.
- **Antes de editar:** lee los archivos objetivo, confirma rama/base y acuerda ownership si existen agentes paralelos.
- **Implementa:** tests primero cuando sea posible; contempla errores, unicode, cleanup, reentrancia y seguridad.
- **Valida:** `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; tests de integración aplicables.
- **No mientas:** reporta cada comando ejecutado y resultado, CI pendiente y riesgos; no rellenes con "todo bien" sin evidencia.
- **Documenta:** modifica la tarea, `docs/STATUS.md` y un ADR cuando cambie un contrato.
- **Entrega:** PR con objetivo, archivos tocados, resultados verificables, limitaciones y siguiente paso desbloqueado. Plantilla `.github/PULL_REQUEST_TEMPLATE.md`.

## Definition of Done
Contrato documentado + casos felices/negativos + pruebas reproducibles + calidad/seguridad + ausencia de regresiones conocidas + handoff. Sin esto, marcar "parcial", no "finalizado".

## Calidad externa (teachers)

El baseline estricto de `C:\Users\jdsal\Documents\Programming-personal\teachers\Rust-teacher` aplica al código nuevo: `missing_docs`, sin `unwrap`/`expect`/`panic` en librería, ≤40 líneas/función, complejidad ≤8, ≤4 parámetros y ancho 100. Antes de publicar:

```powershell
cd C:\Users\jdsal\Documents\Programming-personal\teachers
python tools/lint.py --project "C:\Users\jdsal\Documents\Programming-personal\colibri"
```

Estado y excepciones: [`docs/quality.md`](docs/quality.md).

## Prioridad
M01 parser de template → M02 SFC a IR → M03 Rust a IR → M04 pruebas de paridad. El motor reactivo (M05) puede investigarse en paralelo de forma aislada. Vite/HMR (M08) viene más tarde. Ver `docs/roadmap.md`.
