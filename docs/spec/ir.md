# IR semántica compartida — contrato evolutivo

**Estado:** propuesta. `colibri-ir` actualmente tiene tipos mínimos, no una IR tipada ni lowering.

## Por qué
Ambas fuentes (`.rs` vía macro y `.colibri` vía parser) deben normalizarse en un **modelo semántico común**. Una IR de compilador no implica almacenar un Virtual DOM completo en el navegador.

## Capas (propuestas)
1. **SFC blocks / Rust tokens**: preservan texto original y offsets.
2. **Template AST**: nodos sintácticos, errores recuperables y spans.
3. **Semantic IR**: vistas validadas, componentes, imports, bindings, props, eventos y ownership.
4. **Lowered execution plan**: DOM operations / SSR instructions / static chunks, con análisis de dependencias.
5. **Codegen Rust**: generar la ruta server y WASM cuando sea necesario.

## Nodos candidatos
~~~text
ComponentDef { symbol, props, body, source_span }
StaticText { value, source_span }
DynamicText { expr_id, escape_context, source_span }
Element { namespace, tag, attrs, children, source_span }
ComponentCall { symbol, props, children, key, source_span }
Fragment { children, source_span }
Attribute { name, value_kind, escape_context }
EventBinding { event, handler_expr, lifecycle_owner }
Conditional / KeyedLoop (posteriores al MVP)
~~~

## Invariantes
- Spans preservan ubicación del archivo fuente (byte offset UTF-8 y ruta).
- Separar nombres declarados, símbolos resueltos y IDs locales; no guardar sólo strings de código fuente para siempre.
- Resolver tipos con Rust; no inventar un sistema de tipado Rust alternativo.
- Constantes y expresiones dinámicas no se confunden; el optimizer nunca evalúa efectos arbitrarios como constantes.
- Distinguir contenido HTML, atributo URL, booleano y eventos para seguridad.
- No guardar closures Rust serializables por defecto: state resumable exige diseño/ADR propio.
- IR estable para ambos caminos **semánticamente**; artefactos binarios no tienen por qué ser idénticos byte a byte.
- Identificadores de nodos estables en una compilación; hot reload es fase posterior.
- Clases, accesibilidad y atributos data/aria se representan sin perder semántica.

## Errores
- Ninguna fase posterior debe acceder a nodo inválido sin devolver diagnóstico.
- Adjuntar spans a parser errors, type errors y codegen errors.
- En caso de backend no soportado, error explícito ("SSR no soporta expresión X") en lugar de omitir contenido.

## Pruebas obligatorias
- Golden tests de AST→IR para tags anidados, texto unicode, atributos y bindings.
- Rust y SFC de la misma vista producen IR equivalente tras normalización.
- Errores de span apuntan a línea/columna correcta.
- Serialización para snapshots **sólo en tests**; no definir una ABI pública IR sin ADR.
- Comparar SSR HTML con equivalencia semántica (parser DOM en tests) y verificar que no hay scripts insertados.
