# Architecture v0.1 — Dual Syntax, Single Engine

> Design target, not a complete implementation.

```text
Rust #[component] / view!     .colibri (Rust + template + SCSS)
            \                  /
         AST and type validation
                    |
         Shared semantic Colibrí IR
                    |
    Static analysis and code generation
       /            |             \
HTML + CSS        Rust SSR      Rust -> WASM
(no runtime)      (server)      (islands/client)
                                  |
                          minimal browser bridge
```

## What exists
- `colibri-core`: explicit subscription-based synchronous `Signal<T>` and cleanup `Scope`, **not** dependency autotracking, memoization or a scheduler.
- `colibri-parser`: limited SFC top-level block parser for Rust script, template and style. It does **not** parse nested HTML or Rust expressions.
- `colibri-ir`: minimal static/dynamic node types with no parser-to-IR lowering yet.
- `colibri-ssr`: restrictive static subset renderer with HTML escaping. No server-side expression evaluation.
- `colibri-cli`: `check` performs limited block validation, not compilation.

## Core architectural decisions
1. **Common IR**: `.rs` and `.colibri` must have the same semantics for props, events, state, ownership and lifecycle.
2. **Separate compilation from runtime**: client does not download compiler, Sass, Vite or Rust source.
3. **Static-first**: all-static pages require no JS/WASM; interactive islands load only when needed.
4. **Signals and scopes**: local reactive dependencies plus deterministic cleanup. Avoid a mandatory VDOM.
5. **WASM is optional by workload**: measure bridge overhead before selecting WASM or minimal JS.
6. **Accessibility/security**: semantic DOM, safe escaping, unsafe URL rejection, no implicit raw HTML.
7. **Dev tools**: Vite plugin, CSS modules, SCSS, scoped CSS, sourcemaps and incremental Rust builds are *planned*, not implemented.
8. **Benchmarks before claims**: performance targets are hypotheses.

## Future compiler stages
SFC tokenization -> HTML template AST -> Rust expression parsing (syn) -> typed props/dependencies -> semantic IR -> static optimizer -> Rust code generation -> Cargo/WASM + SSR/SSG + CSS extraction.

## Open questions
Typed props and generic closures; hydration and serializable state; stable IDs under hot reload; effects and re-entrancy; batching; WASM module splitting; SSR safety; deterministic DOM updates. Each requires tests/ADRs.

See `docs/adr/`, `docs/roadmap.md` and `docs/benchmarks.md`.