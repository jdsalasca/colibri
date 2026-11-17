//! Compile and run generated Rust against the real compiler.
#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::too_many_lines,
    clippy::unwrap_used
)]
// Integration test harness: failures are assertions and compiler diagnostics.

use colibri_codegen::generate;
use colibri_parser::parse;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn deps_dir() -> PathBuf {
    std::env::current_exe()
        .expect("test binary path")
        .parent()
        .expect("deps directory")
        .to_path_buf()
}

fn find_rlib(deps: &Path, crate_name: &str) -> PathBuf {
    let prefix = format!("lib{crate_name}-");
    let mut candidates: Vec<PathBuf> = fs::read_dir(deps)
        .expect("read deps")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            let name = path.file_name().unwrap().to_string_lossy();
            name.starts_with(&prefix) && name.ends_with(".rlib")
        })
        .collect();
    candidates.sort_by_key(|path| fs::metadata(path).and_then(|meta| meta.modified()).ok());
    candidates
        .pop()
        .unwrap_or_else(|| panic!("missing rlib for {crate_name} in {}", deps.display()))
}

fn compile_and_run(source: &str) -> String {
    let component = parse("integration.colibri", source).expect("valid SFC");
    let code = generate("Integration", &component).expect("codegen");
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let workdir =
        std::env::temp_dir().join(format!("colibri-codegen-{}-{stamp}", std::process::id()));
    fs::create_dir_all(&workdir).expect("create workdir");
    let source_path = workdir.join("main.rs");
    fs::write(&source_path, code).expect("write source");
    let exe = workdir.join(format!("app{}", std::env::consts::EXE_SUFFIX));
    let deps = deps_dir();

    let output = Command::new("rustc")
        .arg("--edition=2021")
        .arg(&source_path)
        .arg("-o")
        .arg(&exe)
        .arg("--extern")
        .arg(format!(
            "colibri_ir={}",
            find_rlib(&deps, "colibri_ir").display()
        ))
        .arg("--extern")
        .arg(format!(
            "colibri_ssr={}",
            find_rlib(&deps, "colibri_ssr").display()
        ))
        .arg("-L")
        .arg(format!("dependency={}", deps.display()))
        .output()
        .expect("run rustc");
    assert!(
        output.status.success(),
        "rustc failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = Command::new(&exe).output().expect("run generated program");
    assert!(
        output.status.success(),
        "generated program failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = fs::remove_dir_all(&workdir);
    String::from_utf8(output.stdout).expect("utf-8 output")
}

#[test]
fn runs_static_component() {
    assert_eq!(
        compile_and_run("<template><p>Título</p></template>"),
        "<p>Título</p>"
    );
}

#[test]
fn evaluates_script_and_dynamic_text() {
    let source = "<script lang=\"rust\">let name = String::from(\"Ada\");</script><template><p>Hola {name}</p></template>";
    assert_eq!(compile_and_run(source), "<p>Hola Ada</p>");
}

#[test]
fn escapes_dynamic_values_and_attributes() {
    let source = "<script lang=\"rust\">let user = \"<script>alert(1)</script>\";</script><template><p title={user}>{user}</p></template>";
    assert_eq!(
        compile_and_run(source),
        "<p title=\"&lt;script&gt;alert(1)&lt;/script&gt;\">&lt;script&gt;alert(1)&lt;/script&gt;</p>"
    );
}
