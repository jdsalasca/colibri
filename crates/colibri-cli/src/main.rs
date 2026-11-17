//! Experimental SFC tooling: check, render, generate Rust and evaluate it.
use colibri_codegen::generate;
use colibri_parser::{lower, parse, SingleFileComponent};
use colibri_ssr::{render_dynamic, render_static};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{SystemTime, UNIX_EPOCH};

fn usage() {
    eprintln!("Usage: colibri-cli check <component.colibri>");
    eprintln!("       colibri-cli render <component.colibri> [--bind key=value]...");
    eprintln!("       colibri-cli build <component.colibri> [--out file.rs]");
    eprintln!("       colibri-cli eval <component.colibri>");
    eprintln!("Note: render resolves --bind values; eval compiles the Rust script with rustc.");
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args.next().ok_or("missing command")?;
    if !matches!(command.as_str(), "check" | "render" | "build" | "eval") {
        usage();
        return Err(format!("unsupported command: {command}"));
    }
    let path = args.next().ok_or("missing .colibri file path")?;
    let source = fs::read_to_string(&path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let component = parse(&path, &source).map_err(|error| error.to_string())?;

    match command.as_str() {
        "check" => check(&path, &component, args),
        "render" => render(&path, &component, args),
        "build" => build(&path, &component, args),
        "eval" => eval(&path, &component, args),
        _ => unreachable!(),
    }
}

fn check(
    path: &str,
    component: &SingleFileComponent,
    mut args: impl Iterator<Item = String>,
) -> Result<(), String> {
    if args.next().is_some() {
        return Err("check does not accept extra arguments".into());
    }
    println!("SFC valid: {path}");
    println!("  Rust script: {}", component.script.is_some());
    println!("  template nodes: {}", component.template.nodes.len());
    println!("  style: {}", component.style.is_some());
    println!("WARNING: Rust/SCSS compilation is not yet implemented");
    Ok(())
}

fn render(
    path: &str,
    component: &SingleFileComponent,
    args: impl Iterator<Item = String>,
) -> Result<(), String> {
    let bindings = parse_bindings(args)?;
    let name = component_name(path)?;
    let ir = lower(name, component).map_err(|error| error.to_string())?;
    let html = if bindings.is_empty() {
        render_static(&ir.root)
    } else {
        render_dynamic(&ir.root, |expression| bindings.get(expression).cloned())
    }
    .map_err(|error| error.to_string())?;
    println!("{html}");
    Ok(())
}

fn parse_bindings(
    mut args: impl Iterator<Item = String>,
) -> Result<HashMap<String, String>, String> {
    let mut bindings = HashMap::new();
    while let Some(argument) = args.next() {
        if argument != "--bind" {
            return Err(format!("unexpected argument: {argument}"));
        }
        let pair = args.next().ok_or("--bind requires key=value")?;
        let (key, value) = pair
            .split_once('=')
            .ok_or_else(|| format!("invalid binding `{pair}`; expected key=value"))?;
        bindings.insert(key.to_owned(), value.to_owned());
    }
    Ok(bindings)
}

fn build(
    path: &str,
    component: &SingleFileComponent,
    args: impl Iterator<Item = String>,
) -> Result<(), String> {
    let mut output = None;
    let mut args = args;
    while let Some(argument) = args.next() {
        if argument != "--out" {
            return Err(format!("unexpected argument: {argument}"));
        }
        output = Some(args.next().ok_or("--out requires a file path")?);
    }
    let code = generate(component_name(path)?, component).map_err(|error| error.to_string())?;
    if let Some(output) = output {
        fs::write(&output, code).map_err(|e| format!("cannot write {output}: {e}"))?;
        println!("Wrote {output}");
    } else {
        print!("{code}");
    }
    Ok(())
}

fn eval(
    path: &str,
    component: &SingleFileComponent,
    mut args: impl Iterator<Item = String>,
) -> Result<(), String> {
    if args.next().is_some() {
        return Err("eval does not accept extra arguments".into());
    }
    let code = generate(component_name(path)?, component).map_err(|error| error.to_string())?;
    run_generated(&code)
}

fn component_name(path: &str) -> Result<&str, String> {
    Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| "cannot derive component name from path".to_owned())
}

fn run_generated(code: &str) -> Result<(), String> {
    let directory = temp_project_dir();
    let src = directory.join("src");
    fs::create_dir_all(&src).map_err(|e| format!("cannot create {}: {e}", src.display()))?;
    fs::write(src.join("main.rs"), code).map_err(|e| format!("cannot write main.rs: {e}"))?;

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|e| format!("cannot locate workspace: {e}"))?;
    let manifest = format!(
        "[package]\nname = \"colibri-eval\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[dependencies]\ncolibri-ir = {{ path = \"{}\" }}\ncolibri-ssr = {{ path = \"{}\" }}\n",
        toml_path(&root.join("crates/colibri-ir")),
        toml_path(&root.join("crates/colibri-ssr"))
    );
    fs::write(directory.join("Cargo.toml"), manifest)
        .map_err(|e| format!("cannot write Cargo.toml: {e}"))?;

    let status = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(directory.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", directory.join("target"))
        .status()
        .map_err(|e| format!("cannot run cargo: {e}"))?;
    let _ = fs::remove_dir_all(&directory);
    if !status.success() {
        return Err("generated program failed".into());
    }
    Ok(())
}

fn temp_project_dir() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    env::temp_dir().join(format!("colibri-eval-{}-{stamp}", std::process::id()))
}

fn toml_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    let text = text.strip_prefix(r"\\?\").unwrap_or(&text);
    text.replace('\\', "/")
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}
