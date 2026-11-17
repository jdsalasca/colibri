# Colibrí roadmap

## Milestone 0 — Foundation
- [x] Cargo workspace / crate boundaries
- [x] Limited SFC block parser
- [x] Minimal shared IR types
- [x] Explicit signal subscriptions and cleanup scopes
- [x] Restrictive static SSR renderer
- [x] CLI prototype, docs, precedents, proposed CI
- [ ] Verify CI on GitHub

## Milestone 1 — Two syntaxes, one compiler
- [ ] Real HTML template lexer and AST with source spans
- [ ] Safe SFC -> semantic IR lowering
- [ ] Rust codegen and compiler diagnostics
- [ ] Rust `view!` macro -> same IR
- [ ] Differential tests: Rust and SFC render the same UI

## Milestone 2 — Reactive engine
- [ ] Fine-grained dependency graph with dynamic tracking
- [ ] Memo, Effect, Scope and deterministic Scheduler
- [ ] Scoped DOM subscriptions, cleanup and reentrancy tests
- [ ] Memory regression suite

## Milestone 3 — Browser tooling
- [ ] WASM + minimal JS DOM bridge
- [ ] CLI dev/build, Vite plugin, Sass/SCSS/scoped CSS
- [ ] Reliable incremental rebuild, later state-preserving HMR

## Milestone 4 — Delivery and research
- [ ] Static site generation and SSR
- [ ] Selective islands, lazy load and experimental resumability
- [ ] Reproducible Chromium benchmarks on low-end machines
- [ ] Benchmark JS versus WASM strategies

Every milestone needs correctness tests, documented limitations and repeatable measurements.