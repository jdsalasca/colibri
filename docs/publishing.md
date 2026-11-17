# Publishing and contribution workflow

The repository is hosted at https://github.com/jdsalasca/colibri.

- Develop changes on feature branches and review through pull requests.
- Run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`.
- No project license is selected yet; discuss licensing before accepting external code contributions.
- Upstream repositories are studied in `precedentes/` but never vendored by default.
- The initial foundation contains experimental code only. Documentation of planned features must not be mistaken for implemented features.
