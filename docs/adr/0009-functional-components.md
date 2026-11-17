# ADR 0009 - Componentes funcionales con props tipadas

Fecha: 2026-11-16
Estado: **Aceptada** para el subconjunto SSR.

## Contexto
`view!` sólo construía IR estático y rechazaba dinámicos. No existía una forma de declarar props, componer componentes ni aprovechar el tipado de Rust para una API tipo React. El codegen SFC ya demostró que se puede evaluar Rust sin intérprete.

## Decisión

1. `#[component]` envuelve una función `PascalCase` con parámetros owned y retorno `Node`.
   - Genera `<Nombre>Props` con campos `pub` y una función pública del mismo nombre que recibe el struct.
   - El cuerpo original queda en una función privada.
   - Rechaza referencias, genéricos, `async`, `const`, `unsafe`, receivers y retorno vacío.
2. `view!` acepta:
   - texto literal,
   - interpolación `{expr}` con `format!("{}", expr)`,
   - atributos estáticos, booleanos y `{expr}`,
   - llamadas a componentes `PascalCase` con props.
3. La composición es directa: un componente devuelve `Node` y se inserta en el árbol.
4. Children: si un componente tiene contenido, `view!` añade `children: vec![...]` al struct de props. El componente declara `children: Vec<Node>` y lo inserta con el componente reservado `<Fragment nodes={children}/>`. No hay slots nombrados.
5. El render sigue siendo `colibri_ssr::render_static`; no hay runtime cliente.
6. `Option<T>` es la representación de ausencia; no se introduce `null`.

## Alternativas
- Props como argumentos posicionales: impide construir el componente desde `view!` sin conocer la firma.
- Struct de props manual por el usuario: funciona, pero duplica código y permite inconsistencias.
- Macros de string/HTML builder: otro modelo, rompe el IR compartido.
- Componentes con closures/vida dinámica: exige runtime cliente (M07).

## Consecuencias
- API ergonómica y fuertemente tipada; los errores de props son errores de rustc.
- El macro genera structs públicas; los nombres `<Nombre>Props` quedan reservados.
- Los props deben ser owned y `Display` sólo si se interpolan.
- `Fragment` es un nombre reservado para insertar listas de nodos.
- Sin slots nombrados, eventos, estado cliente ni hidratación todavía.
- La paridad SFC no cubre componentes: el SFC aún no resuelve símbolos Rust.

## Validación
- Tests de integración: props tipadas, composición, `Option`/null safety y render SSR.
- Doc-test del macro con componente y `view!`.
- `cargo clippy --workspace --all-targets -- -D warnings` limpio.
