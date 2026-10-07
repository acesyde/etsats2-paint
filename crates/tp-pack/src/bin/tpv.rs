//! `tpv`: builds and checks TruckPaint vehicle packages.

use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "\
tpv: build and check TruckPaint vehicle packages (.tpv)

Usage:
  tpv pack <folder> [-o <file.tpv>]   Pack a folder holding a vehicle.json and its
                                      templates (PNG, SVG or DDS, converted to PNG).
                                      Default output: <id>-<version>.tpv
  tpv check <file.tpv>                Validate a package and print its contents.
  tpv --help                          Show this help.

See docs/vehicle-package-format.md.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("pack") => pack(&args[1..]),
        Some("check") => check(&args[1..]),
        Some("-h" | "--help" | "help") => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        _ => Err(format!("missing or unknown command\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn pack(args: &[String]) -> Result<(), String> {
    let mut folder = None;
    let mut output = None;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "-o" | "--output" => {
                output = Some(PathBuf::from(rest.next().ok_or("-o needs a file name")?));
            }
            other if folder.is_none() && !other.starts_with('-') => {
                folder = Some(PathBuf::from(other));
            }
            other => return Err(format!("unexpected argument \"{other}\"\n\n{USAGE}")),
        }
    }
    let folder = folder.ok_or_else(|| format!("pack needs a folder\n\n{USAGE}"))?;
    let packed = tp_pack::pack_folder(&folder).map_err(|e| e.to_string())?;
    let output = output.unwrap_or_else(|| PathBuf::from(packed.file_name()));
    for (dds, png) in &packed.converted {
        println!("converted {dds} -> {png}");
    }
    for file in &packed.ignored {
        println!("ignored   {file}");
    }
    std::fs::write(&output, &packed.bytes).map_err(|e| format!("{}: {e}", output.display()))?;
    println!(
        "packed    {} ({} KB)",
        output.display(),
        packed.bytes.len().div_ceil(1024)
    );
    print!("{}", tp_pack::summary(&packed.manifest));
    Ok(())
}

fn check(args: &[String]) -> Result<(), String> {
    let [file] = args else {
        return Err(format!("check needs one file\n\n{USAGE}"));
    };
    let bytes = std::fs::read(file).map_err(|e| format!("{file}: {e}"))?;
    let manifest = tp_pack::check(&bytes).map_err(|e| format!("{file}: {e}"))?;
    print!("{}", tp_pack::summary(&manifest));
    Ok(())
}
