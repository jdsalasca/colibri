#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$ROOT/upstream"
mkdir -p "$OUT"
declare -A REPOS=(
  [react]="https://github.com/facebook/react.git"
  [solid]="https://github.com/solidjs/solid.git"
  [svelte]="https://github.com/sveltejs/svelte.git"
  [qwik]="https://github.com/QwikDev/qwik.git"
  [leptos]="https://github.com/leptos-rs/leptos.git"
  [dioxus]="https://github.com/DioxusLabs/dioxus.git"
  [astro]="https://github.com/withastro/astro.git"
  [vue]="https://github.com/vuejs/core.git"
  [yew]="https://github.com/yewstack/yew.git"
)
for name in "${@:-}"; do
  if [[ -z "${REPOS[$name]:-}" ]]; then echo "Unknown framework: $name" >&2; exit 1; fi
  if [[ ! -d "$OUT/$name/.git" ]]; then
    git clone --depth 1 "${REPOS[$name]}" "$OUT/$name"
  else
    echo "Already fetched: $name"
  fi
done
# Example: bash precedentes/fetch.sh react solid leptos
