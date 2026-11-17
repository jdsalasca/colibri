//! Experimental SFC tooling; not a Rust/SCSS compiler.
use colibri_parser::{lower, parse};
use colibri_ssr::render_static;
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

fn usage() {
    eprintln!("Usage: colibri-cli check <component.colibri>");
    eprintln!("       colibri-cli render <component.colibri>");
    eprintln!("Note: render only supports static templates.");
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args.next().ok_or("missing command")?;
    if !matches!(command.as_str(), "check" | "render") {
        usage();
        return Err(format!("unsupported command: {command}"));
    }
    let path = args.next().ok_or("missing .colibri file path")?;
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    let source = fs::read_to_string(&path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let component = parse(&path, &source).map_err(|error| error.to_string())?;

    match command.as_str() {
        "check" => {
            println!("SFC valid: {path}");
            println!("  Rust script: {}", component.script.is_some());
            println!("  template nodes: {}", component.template.nodes.len());
            println!("  style: {}", component.style.is_some());
            println!("WARNING: Rust/SCSS compilation is not yet implemented");
        }
        "render" => {
            let name = Path::new(&path)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .ok_or("cannot derive component name from path")?;
            let ir = lower(name, &component).map_err(|error| error.to_string())?;
            let html = render_static(&ir.root).map_err(|error| error.to_string())?;
            println!("{html}");
        }
        _ => unreachable!(),
    }
    Ok(())
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
