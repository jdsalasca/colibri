# AGENTS.md — Colibrí

## Mission
Build an accessible, resource-efficient Rust-first web framework. **Compute only what changes; load only what matters.** Optimize for low-end devices without sacrificing correctness, accessibility or developer experience.

## Current status
Experimental foundation, **not a working frontend framework**. Core, parser, IR, SSR and CLI are independent prototypes. No complete compiler, `.colibri` code generation, production browser runtime, WASM DOM backend, Vite integration or benchmark victories exist yet. Never imply otherwise.

## Architectural invariants
1. Two planned syntaxes — Rust `#[component]` / `view!` and `.colibri` SFC — must converge on a **single semantic IR and runtime**.
2. Keep `colibri-core` independent of DOM, browser, Vite, Node and third-party UI frameworks.
3. Separate parser, AST, type checking, IR, code generation, renderer and tooling; compiler dependencies must not ship to the browser.
4. Static-first delivery: send **zero client runtime** for fully static pages.
5. Use fine-grained reactivity where it saves real work; do not impose a Virtual DOM or mandatory WASM out of ideology.
6. Scope and release effects, subscriptions, listeners and async work. Rust prevents some memory errors, not logical resource leaks.
7. Escape HTML and attributes; validate URL schemes and element/attribute names; forbid untrusted HTML/script injection.
8. Measure WASM/JavaScript boundary cost and total Chromium memory. Define comparisons before claiming gains.
9. Preserve standards: semantic HTML, CSS, keyboard navigation and progressive enhancement.
10. Keep conceptual code clearly labeled as not yet implemented.

## Working agreements
- Start with a failing behavioral test; implement the smallest change; then evaluate performance regressions.
- Before review run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- Add tests for malformed inputs, Unicode, lifecycle cleanup, reentrancy, security and render parity.
- Explain new dependencies, their costs, and any unsafe code. Workspace currently forbids unsafe code.
- Keep all upstream framework clones under ignored `precedentes/upstream/`; respect upstream licenses.
- Use feature branches and PRs; record consequential decisions in `docs/adr/`; do not force-push main.
- Changes to shared IR require eventual equivalence tests for Rust and SFC sources.
- Prefer Spanish project documentation; stable English code/API names.

## Benchmark discipline
Use the same UI, Chromium version, dataset, release settings and hardware for Colibrí and comparison frameworks. Record compressed bytes, startup, interaction latency P95, CPU, total browser RSS, WASM heap, allocations, DOM mutations, long sessions and cold/warm build. Publish scripts and raw results, not just a winner.

## Next verifiable target
Parse `<template>` into a real AST, lower a static `.colibri` component to IR and render it safely. Then implement the equivalent Rust macro path and prove observable equivalence. HMR, hybrid backends and resumability come later.