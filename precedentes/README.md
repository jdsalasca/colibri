# Precedentes: comparative research, not vendored dependencies

Study implementations rather than blindly reimplement features. The upstream repositories and their licenses belong to their respective maintainers. Files under `precedentes/upstream/` are local, ignored clones and are **not** dependencies of Colibrí.

| Project | Study | Avoid / investigate |
|---|---|---|
| React | declarative components, concurrency, DevTools, compiler | effect complexity, render work, hydration |
| Solid | fine-grained signals, direct DOM | dynamic tracking overhead, ownership |
| Svelte | compile-time specialization, component syntax | generated-code footprint, debugging |
| Qwik | resumability, lazy event handlers | serialization and network overhead |
| Leptos | Rust SSR, hydration, fine-grained signals | WASM startup, ergonomics |
| Dioxus | Rust RSX, cross-platform components | reconciliation cost, web bundle size |
| Astro | zero-client-JS default and islands | island boundaries and integrations |
| Vue | SFC tooling, props, scoped CSS | runtime overhead and compiler complexity |
| Yew | Rust component model, web tooling | bundle size, scheduler, diff costs |

See `frameworks.tsv`, `notes/`, and `fetch.sh`. Record upstream commit/version and benchmark methodology when writing comparisons; never equate design intentions with measurements.