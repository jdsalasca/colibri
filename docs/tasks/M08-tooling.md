# M08 — CLI, Sass/SCSS y Vite plugin

**Prioridad:** P1. **Depende de:** M07 y contrato compiler. **Área:** `colibri-cli`, `packages/vite-plugin-colibri`. **Spec:** `docs/spec/toolchain.md`.

## Objetivo
Una aplicación de ejemplo debe compilar y poder abrirse desde un servidor de desarrollo que combine Rust, WASM, HTML y CSS/SCSS.

## Trabajo
1. `colibri build`: invocar compilador, Cargo y empaquetado definido.
2. `colibri dev`: servidor/estrategia integración Vite documentada.
3. Plugin JS/TS orquesta, no reimplementa parser ni compila Rust por duplicado.
4. Pipeline Sass a CSS, scoped CSS y clases; incluir sourcemaps y manejo de errores.
5. Rebuild Rust incremental con reload completo inicialmente; HMR para CSS cuando Vite lo soporta.
6. Separar dependencias de tooling Node/Sass de runtime y SSR.

## Aceptación
- [ ] App end-to-end construye y se abre en Chromium.
- [ ] Editar SCSS recompila CSS; Rust recompila y recarga sin estado corrupto.
- [ ] Diagnósticos muestran la ruta fuente real SFC/Rust.
- [ ] Static-only output no contiene runtime cliente.
- [ ] Registros de tiempos de build y tamaños por asset.

## Handoff
Pasos para clonación, bootstrap, dev, build, preview; limitaciones HMR y lista de dependencias.
