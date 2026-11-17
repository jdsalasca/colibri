# Performance research — no results claimed

Hypotheses: (H1) fine-grained reactivity avoids unnecessary update work, (H2) static HTML reduces startup, (H3) WASM does not always outperform JS for browser DOM workloads.

Reference harness: https://github.com/krausest/js-framework-benchmark

Scenarios: keyed list operations; isolated and cascading reactive text; static documentation page; dashboard with lazy charts; 10,000 mount/update/unmount cycles; prolonged idle CPU/memory.

Protocol: identical application and data per framework; same Chromium, OS, machine, network and release flags. At least 20 repeats after warmup. Record raw runs, medians, P95 and spread; compressed transfer bytes, startup, scripting/layout/paint, interaction latency, CPU, process RSS, WASM heap, DOM nodes, leaks, cold/warm build.

Never compare an unoptimized reference implementation against optimized Colibrí or report unmeasured targets as results.