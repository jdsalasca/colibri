# ADR 0002 — Static-first and selective runtime
Date: 2026-11-16
Status: Accepted as design goal; implementation pending.

Deliver HTML and CSS without client JavaScript or WASM when interaction is unnecessary. For interactive experiences, consider islands or dedicated client modules and measure startup, memory, latency and transfer overhead.

Risks: serializing Rust closures; hydration differences; too many small WASM modules; session-dependent content.

Acceptance: truly static page transfers 0 client-runtime bytes; interactive islands preserve accessibility and cleanup with benchmark evidence.