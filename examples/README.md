# Examples

`static-card.colibri` is a static template that compiles to HTML today:

```sh
cargo run -p colibri-cli -- render examples/static-card.colibri
```

`greeting.colibri` has dynamic expressions resolved by the host or evaluated by Rust:

```sh
cargo run -p colibri-cli -- render examples/greeting.colibri --bind name=Ada --bind message=Bienvenida
cargo run -p colibri-cli -- eval examples/greeting.colibri
cargo run -p colibri-cli -- build examples/greeting.colibri --out greeting.rs
```

`eval` compiles the `<script lang="rust">` with rustc; it is not a sandbox. `build` only emits the generated Rust program.

`counter.colibri` and `counter.rs.example` illustrate **future intended APIs** for interactive components; they do not compile to an interactive page yet. The CLI can validate their SFC structure with `check`; `render` reports dynamic values or events with a precise span.
