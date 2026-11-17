//! Experimental SFC validation CLI; not a Rust/template/SCSS compiler.
use colibri_parser::parse;
use std::env;
use std::fs;
use std::process::ExitCode;

fn usage() {
    eprintln!("Usage: colibri-cli check <component.colibri>");
    eprintln!("Note: only basic SFC blocks are validated.");
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    if args.next().as_deref() != Some("check") {
        usage();
        return Err("unsupported command".into());
    }
    let path = args.next().ok_or("missing .colibri file path")?;
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    let source = fs::read_to_string(&path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let component = parse(&source).map_err(|e| format!("invalid {path}: {e}"))?;
    println!("SFC blocks valid: {path}");
    println!("  Rust script: {}", component.script.is_some());
    println!("  template bytes: {}", component.template.len());
    println!("  style: {}", component.style.is_some());
    println!("WARNING: template/Rust/SCSS compilation is not yet implemented");
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
