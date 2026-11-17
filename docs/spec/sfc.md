# Especificación propuesta SFC: archivos `.colibri`

**Estado:** propuesta pre-1.0. El parser actual detecta bloques y construye el AST del template ([contrato](template-ast.md)); todavía no compila a Rust ni renderiza HTML.

## Forma mínima
~~~html
<script lang="rust">
    let count = signal(0);
    let increment = move |_| count.update(|n| *n += 1);
</script>
<template>
    <button type="button" on:click={increment}>Count: {count}</button>
</template>
<style scoped lang="scss">
    button { color: rebeccapurple; }
</style>
~~~

## Gramática conceptual y reglas
- Extensión `.colibri` y texto UTF-8. Exactamente un `<template>` raíz (puede contener varios nodos hermanos); opcional un `<script lang="rust">` y opcional un `<style>`.
- `<style lang="css">` o `<style lang="scss">`; `scoped` opcional. Sass se compila en build-time y no se envía al cliente. Futuro: varios bloques por separado mediante ADR.
- Comentarios HTML fuera de bloques no generan elementos. Whitespace dentro de texto es relevante; no recortar el contenido del template indiscriminadamente.
- Los tags/atributos son case-insensitive sólo para HTML estándar donde lo defina el estándar; los nombres de componentes Rust respetan mayúsculas.
- El parser debe mantener ubicaciones (offset byte, línea/columna, ruta) y emitir errores de duplicado, bloques faltantes, cierre desbalanceado, strings mal formados y sintaxis prohibida.
- **Nunca** separar bloques mediante una simple búsqueda de `</script>`/`</template>` cuando eso pueda aparecer en comentarios o cadenas: usar scanner con estados apropiados, o declarar explícitamente la limitación hasta implantarlo.
- El parser SFC no ejecuta código; el bloque Rust se compila con toolchain Rust del proyecto. Nunca interpretar Rust o SCSS en el navegador.

## Template: primera versión
- Tags HTML semánticos en un subconjunto inicial, nodos de texto, atributos estáticos, interpolaciones `{expr}` y referencias a componentes.
- Expresiones son Rust, no JavaScript; deben parsearse como expresiones válidas en contexto de Rust. Evitar eval y JS implícito.
- Texto dinámico y atributos se escapan/validan según contexto. Sin `raw-html` implícito. Acceso al DOM a través del runtime.
- `on:event={handler}` para eventos conocidos, ligados al scope con cleanup; evitar sintaxis mágico-tolerante que acepte handlers inválidos silenciosamente.
- Componentes con props tipadas y children definidos en [components.md](components.md).
- Listas dinámicas deben tener identidad estable `key` cuando haya reordenamiento, según contrato de componentes.
- Condicionales, loops y slots avanzados **son fase posterior**; definir gramática y ownership por ADR antes de implementarlos.

## Scoping CSS
- El compilador deriva un identificador determinista por archivo+componente (no usar valores aleatorios por render); transforma selectores y añade un atributo de scope.
- No modificar semántica de `:global`/`:deep` sin contrato. En primera versión rechazar selectores no soportados con diagnóstico, no producir CSS incorrecto silenciosamente.
- Respetar CSS Modules existentes cuando se integren. No desarrollar un compilador Sass propio.

## Interoperabilidad
El SFC genera definiciones Rust y metadatos de props compatibles con el backend Rust. No compilar un runtime SFC paralelo. Referencias a otros componentes deben respetar resolución de módulos/visibilidad, por diseño documentado.

## Ejemplos de error esperados
- Falta `<template>`: error con línea de archivo y solución.
- `<script lang="js">`: error "sólo Rust en la fase inicial".
- `{unresolved}`: error rustc con span mapeado al SFC.
- Binding a atributo `onclick` string: diagnóstico por seguridad; ofrecer `on:click`.
- Evento duplicado, closing-tag inválido o comentario sin cerrar: error preciso, sin pánico.

## Aceptación del primer hito
1. AST anidado y posiciones robustas con casos Unicode.
2. Tests de errores: duplicados, cierres, escapes, comentarios, tag en literal, whitespace.
3. Dos componentes estáticos SFC equivalentes a Rust generan IR/HTML equivalente.
4. El build falla con diagnósticos claros; no ignora bloques desconocidos.
5. Ejemplos de documentación prueban realmente en CI antes de etiquetarse "ejecutables".
