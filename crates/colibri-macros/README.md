# colibri-macros

Macros de la ruta Rust.

- `#[component]`: genera `<Nombre>Props` y una función pública con props tipadas.
- `view!`: IR desde sintaxis tipo JSX, con texto/atributos dinámicos, componentes y children.
- Los eventos siguen bloqueados hasta el runtime cliente.

```rust
#[component]
fn Greeting(name: String) -> Node {
    view! { <p>"Hola " {name}</p> }
}
```
