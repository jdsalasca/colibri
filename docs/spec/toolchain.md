# Toolchain / CLI / Vite / SCSS — diseño

**Estado:** parcialmente implementado. Existen `check`, `render` (estático o `--bind`) y `build`/`eval` con codegen Rust. No existen `new`, `dev`, `preview`, `explain` ni `benchmark`.

## Objetivo de desarrollo
Una app usa workspace Rust con Cargo, archivos `.rs` y `.colibri`, CSS/SCSS, assets estáticos y, opcionalmente, Vite para servir/HMR y resolver recursos. Colibrí no obliga a instalar npm para compilar una biblioteca Rust que no usa Vite.

## CLI actual
~~~sh
colibri check <component.colibri>                  # estructura SFC
colibri render <component.colibri> [--bind k=v]    # HTML estático o con resolver
colibri build <component.colibri> [--out file.rs]  # programa Rust generado
colibri eval <component.colibri>                   # compila y ejecuta el Rust generado
~~~

## CLI objetivo (NO disponible)
~~~sh
colibri new app
colibri dev
colibri build --release
colibri preview
colibri explain ComponentName
colibri benchmark
~~~
Primero estabilizar `check`, luego `build` y `dev`. `explain` y `benchmark` son fases posteriores.

## Pipeline propuesto
- `.colibri` parse → AST → IR → Rust codegen.
- `.rs` → procedural macro → IR/codegen.
- Cargo genera server y/o `wasm32-unknown-unknown` según salida seleccionada.
- `wasm-bindgen` (o binding equivalente evaluado) produce adaptador JS mínimo cuando DOM/API lo exige.
- Vite recibe manifiesto de entradas WASM, CSS y assets, **sin descargar Cargo/parser al navegador**.
- Sass compila SCSS en build-time; scope CSS es transformación aparte, probada.
- Compresión, lazy modules y asset hashing en build release; no prometer un tamaño fijo.

## Límites de Vite
- El plugin de Vite gestiona desarrollo, invalidación y assets; no reimplementa Rust macros o semántica de templates.
- Cambiar SCSS puede hacer HMR CSS. Cambios Rust inicialmente desencadenan rebuild incremental + recarga completa.
- Preservación de estado en hot updates sólo después de definir identidad y cleanup; no exigirla en MVP.
- Diagnósticos deben referenciar archivo original y spans; recoger stderr Cargo y transformarlo con trazabilidad.

## Observabilidad de build
Separar tiempos de parse, codegen, cargo, wasm-opt y Vite, así como tamaño de bytes comprimidos de HTML, JS, WASM y CSS. No compararlos sin misma configuración/caché.

## Compatibilidad
- Rust estable (documentar MSRV cuando haya distribución).
- Navegadores con WebAssembly para modo client/islands; fallback HTML semántico donde sea posible.
- CSS tradicional por defecto; SCSS opcional vía Sass, no herramienta propia.
- TypeScript/JS puente interoperable si hace falta; no asumir compatibilidad automática con librerías React.
- Evitar paquetes Node obligatorios para ejecutar `colibri-core` o `colibri-ssr`.

## Tests de aceptación
- Proyecto de ejemplo compila con Cargo en CI.
- SFC change recompila resultado; errores aparecen con ruta y línea.
- SCSS se transforma a CSS y no aparece Sass en paquete de navegador.
- Navegación, reload, manifest assets y errores de dependencia funcionan en Chromium.
- Static app produce 0 bytes Colibrí de JS/WASM; Islands sólo carga lo requerido.
