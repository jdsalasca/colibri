# ADR 0001 — Dual Syntax, Single IR
Date: 2026-11-16
Status: Accepted as design direction; implementation pending.

Colibrí supports future Rust `#[component]`/`view!` and `.colibri` SFC components. Both must lower to the same semantic IR and lifecycle. One IR avoids duplicated runtimes and allows cross-syntax tests and optimization.

Costs: more demanding parsing, typed props, span/source maps and Rust macro/codegen work.

Acceptance: equivalent views written in both syntaxes produce equivalent static structure and observable dynamic behavior, including events and teardown.