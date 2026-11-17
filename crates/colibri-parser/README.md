# colibri-parser

Parser del formato `.colibri` y lowering a IR.

- Scanner de bloques `script`/`template`/`style` con límites de tamaño y profundidad.
- AST de template con spans, atributos e interpolaciones.
- `lower()` separa estructura estática de expresiones dinámicas.

```sh
cargo test -p colibri-parser
```
