# Visión, personas usuarias y límites de producto

## Problema
Las interfaces web modernas pueden gastar memoria, CPU, red y energía en trabajo que no contribuye a la interacción que una persona está realizando. Colibrí investiga cómo disminuir ese coste sin obligar a abandonar HTML, CSS, accesibilidad o flujos de desarrollo agradables.

## Norte
**Compute only what changes. Load only what matters.** Rust-first con:
- componentes `.rs` y `.colibri`;
- un mismo motor semántico para ambos;
- contenido estático sin runtime cliente;
- interactividad granular con activación bajo demanda;
- soporte para CSS/SCSS y herramientas familiares como Vite;
- mediciones transparentes en Chromium y máquinas de bajos recursos.

## Público prioritario
1. Personas que usan PCs modestos (2–4 GB RAM, CPU de bajo consumo) y redes limitadas.
2. Equipos que necesitan accesibilidad, SEO/SSR y buen tiempo de respuesta.
3. Desarrolladores Rust y frontend que no desean gestionar manualmente cada optimización.
4. Comunidades e instituciones que publican aplicaciones web abiertas y duraderas.

## No objetivos de las primeras versiones
- Reemplazar Chromium o el motor DOM/CSS del navegador.
- Prometer 0 MB RAM, 0 CPU o latencias instantáneas bajo toda carga.
- Implementar un GC propio (Rust no lo requiere en su modelo convencional).
- Prometer compatibilidad directa con todos los componentes React/JSX.
- Reescribir desde cero Sass, Vite, HTML, CSS o el compilador de Rust.
- Implementar por adelantado todas las plataformas de escritorio/móvil.
- Elegir WASM para todo sin medir el coste de arranque/puente JS.

## Principios de producto
- **Accesibilidad y seguridad son requisitos**, no mejoras opcionales.
- **Progresividad**: HTML antes de JS; interacción antes que animación; contenido antes que frameworks.
- **Performance verificable**: pruebas publicadas con hardware, versiones y datos reproducibles.
- **Ergonomía sin trabajo oculto**: herramientas que expliquen el coste de código e islas.
- **Compatibilidad con web abierta**: semántica HTML estándar y CSS del navegador.

## Objetivos de investigación, no resultados
Reducir trabajo de actualización, huella de memoria y bytes descargados frente a implementaciones funcionalmente equivalentes. Las hipótesis y métodos se recogen en `docs/benchmarks.md`.

## Criterio de éxito inicial
Dos contadores equivalentes (Rust/SFC), compilados y correctamente actualizados, con cleanup verificable. **Antes de proclamar rendimiento, demostrar semántica y pruebas.**
