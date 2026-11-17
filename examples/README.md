# Examples

`static-card.colibri` is a static template that compiles to HTML today:

```sh
cargo run -p colibri-cli -- render examples/static-card.colibri
```

`counter.colibri` and `counter.rs.example` illustrate **future intended APIs** for dynamic components; they do not compile to an interactive page yet. The CLI can validate their SFC structure with `check`, and `render` rejects their dynamic content with a precise span.
