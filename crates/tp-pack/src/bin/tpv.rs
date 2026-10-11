//! `tpv`: builds and checks TruckPaint vehicle packages.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
tpv: build and check TruckPaint vehicle packages (.tpv)

Usage:
  tpv pack <folder> [-o <file.tpv>]   Pack a folder holding a vehicle.json and its
                                      templates (PNG, SVG or DDS, converted to PNG).
                                      Default output: <id>-<version>.tpv
  tpv check <file.tpv>                Validate a package and print its contents.
  tpv catalog check <pack> [--game ets2|ats] [--catalog <dir>]
                                      Check the official vehicles' catalog against an
                                      unzipped template pack (ETS2 by default): folders,
                                      files, sizes and coverage of the game ids.
  tpv catalog sync <paintjob-packer folder> --commit <commit> [--catalog <dir>]
                                      Replace each catalog entry's game data with that
                                      of a local copy of Paintjob Packer.
  tpv catalog match <pack> <paintjob-packer folder> --commit <commit>
                    [--game ets2|ats] [--vehicle <path>]... [--catalog <dir>]
                                      Propose an entry for each official vehicle the
                                      catalog doesn't have, from the pack's file names,
                                      and print what to check by hand. Existing entries
                                      are never changed.
                                      Default catalog: ./catalog.
  tpv --help                          Show this help.

Each texture's size is [width, height] in pixels, each side a power of two
from 64 to 8192: [4096, 4096], [8192, 1024]. A single number is refused.

See docs/vehicle-package-format.md.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("pack") => pack(&args[1..]),
        Some("check") => check(&args[1..]),
        Some("catalog") => catalog(&args[1..]),
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

fn catalog(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("check") => catalog_check(&args[1..]),
        Some("sync") => catalog_sync(&args[1..]),
        Some("match") => catalog_match(&args[1..]),
        _ => Err(format!("catalog needs check, sync or match\n\n{USAGE}")),
    }
}

/// The positional argument and the values of `options` (`--name value`).
fn parse_options<'a>(
    args: &'a [String],
    options: &[&str],
) -> Result<(Option<&'a str>, Vec<Option<&'a str>>), String> {
    let mut positional = None;
    let mut values = vec![None; options.len()];
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        if let Some(i) = options.iter().position(|o| o == arg) {
            let value = rest.next().ok_or_else(|| format!("{arg} needs a value"))?;
            values[i] = Some(value.as_str());
        } else if positional.is_none() && !arg.starts_with('-') {
            positional = Some(arg.as_str());
        } else {
            return Err(format!("unexpected argument \"{arg}\"\n\n{USAGE}"));
        }
    }
    Ok((positional, values))
}

fn catalog_check(args: &[String]) -> Result<(), String> {
    let (pack, options) = parse_options(args, &["--game", "--catalog"])?;
    let pack = pack.ok_or_else(|| format!("catalog check needs a pack folder\n\n{USAGE}"))?;
    let game = match options[0] {
        None => tp_vehicles::Game::Ets2,
        Some(code) => tp_vehicles::Game::from_code(code)
            .ok_or_else(|| format!("unknown game \"{code}\": use ets2 or ats"))?,
    };
    let entries = match options[1] {
        Some(dir) => tp_pack::catalog::read_catalog(Path::new(dir))?
            .into_iter()
            .map(|(_, e)| e)
            .collect(),
        None => tp_vehicles::catalog::embedded().to_vec(),
    };
    let entries: Vec<&tp_vehicles::catalog::Entry> =
        entries.iter().filter(|e| e.game == game).collect();
    if !Path::new(pack).is_dir() {
        return Err(format!("{pack}: not a folder"));
    }
    let report = tp_pack::catalog::check(Path::new(pack), &entries);
    print!("{}", report.text());
    if report.problems.is_empty() {
        Ok(())
    } else {
        Err(format!("{} problem(s) found", report.problems.len()))
    }
}

fn catalog_sync(args: &[String]) -> Result<(), String> {
    let (ppk, options) = parse_options(args, &["--commit", "--catalog"])?;
    let ppk =
        ppk.ok_or_else(|| format!("catalog sync needs a Paintjob Packer folder\n\n{USAGE}"))?;
    let commit = options[0].ok_or("catalog sync needs --commit <commit>")?;
    let dir = options[1].unwrap_or("catalog");
    let report = tp_pack::catalog::sync(Path::new(ppk), commit, Path::new(dir))?;
    print!("{}", report.text());
    Ok(())
}

fn catalog_match(args: &[String]) -> Result<(), String> {
    let mut positional = Vec::new();
    let mut commit = None;
    let mut game = tp_vehicles::Game::Ets2;
    let mut only = Vec::new();
    let mut dir = "catalog".to_owned();
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        let mut value = || {
            rest.next()
                .cloned()
                .ok_or_else(|| format!("{arg} needs a value"))
        };
        match arg.as_str() {
            "--commit" => commit = Some(value()?),
            "--game" => {
                let code = value()?;
                game = tp_vehicles::Game::from_code(&code)
                    .ok_or_else(|| format!("unknown game \"{code}\": use ets2 or ats"))?;
            }
            "--vehicle" => only.push(value()?),
            "--catalog" => dir = value()?,
            other if !other.starts_with('-') && positional.len() < 2 => {
                positional.push(other.to_owned());
            }
            other => return Err(format!("unexpected argument \"{other}\"\n\n{USAGE}")),
        }
    }
    let [pack, ppk] = &positional[..] else {
        return Err(format!(
            "catalog match needs a pack folder and a Paintjob Packer folder\n\n{USAGE}"
        ));
    };
    let commit = commit.ok_or("catalog match needs --commit <commit>")?;
    let dir = Path::new(&dir);
    let existing: Vec<tp_vehicles::catalog::Entry> = tp_pack::catalog::read_catalog(dir)?
        .into_iter()
        .map(|(_, e)| e)
        .collect();
    let report = tp_pack::catalog::matching::match_vehicles(
        Path::new(pack),
        Path::new(ppk),
        game,
        &commit,
        &only,
        &existing,
    )?;
    for m in &report.vehicles {
        let path = dir.join(m.entry.catalog_path());
        if path.exists() {
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        std::fs::write(&path, m.entry.to_json()).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    print!("{}", report.text());
    Ok(())
}
