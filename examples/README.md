# Examples

`static-card.colibri` is a static template that compiles to HTML today:

```sh
cargo run -p colibri-cli -- render examples/static-card.colibri
```

`greeting.colibri` has dynamic expressions resolved by the host (no Rust evaluation yet):

```sh
cargo run -p colibri-cli -- render examples/greeting.colibri --bind name=Ada --bind message=Bienvenida
```

`counter.colibri` and `counter.rs.example` illustrate **future intended APIs** for interactive components; they do not compile to an interactive page yet. The CLI can validate their SFC structure with `check`; `render` reports dynamic values or events with a precise span.
