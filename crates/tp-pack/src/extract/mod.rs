//! Extracts official truck definitions and painted geometry from ETS2.

mod geometry;
mod source;
mod syntax;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use source::Sources;
use syntax::Section;

/// Inputs supplied explicitly by the package author.
pub struct Options {
    pub installation: PathBuf,
    pub converter: PathBuf,
    pub vehicle: Option<String>,
    pub output: PathBuf,
    pub package_version: semver::Version,
}

/// A report is still delivered when individual vehicles fail.
pub struct Batch {
    pub directory: PathBuf,
    pub failed: usize,
}

fn units(section: &Section) -> &[Section] {
    if let Some(wrapper) = section.children.iter().find(|s| s.header == "SiiNunit") {
        &wrapper.children
    } else {
        &section.children
    }
}

fn array(section: &Section, field: &str) -> Vec<String> {
    section
        .fields
        .get(field)
        .into_iter()
        .flatten()
        .map(|s| s.trim_matches('"').to_owned())
        .collect()
}

fn reserve(parent: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    for attempt in 0..100 {
        let path = parent.join(format!("ets2-{stamp}-{}-{attempt}", std::process::id()));
        match std::fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(format!("{}: {e}", path.display())),
        }
    }
    Err("cannot reserve unique batch directory".into())
}

fn publish(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let temporary = path.with_extension("tmp");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| format!("{}: {e}", temporary.display()))?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result.map_err(|e| format!("{}: {e}", path.display()))
}

fn definition(
    sources: &mut Sources,
    path: &str,
    evidence: &mut BTreeSet<String>,
) -> Result<Section, String> {
    let text = sources.resolved(path, &mut BTreeSet::new(), evidence)?;
    syntax::parse(&text).map_err(|e| format!("{path}: {e}"))
}

fn label(value: &str) -> String {
    value
        .strip_prefix("@@")
        .and_then(|v| v.strip_suffix("@@"))
        .map(|v| {
            v.trim_start_matches("brand_")
                .trim_start_matches("series_")
                .trim_start_matches("cabin_")
                .replace('_', " ")
                .to_uppercase()
        })
        .unwrap_or_else(|| value.to_owned())
}

struct Painted {
    id: String,
    group: String,
    name: String,
    cabins: BTreeSet<String>,
    mesh: Vec<geometry::Mesh>,
    selection: Vec<Value>,
    source: String,
}

fn crop_size(mask: &str) -> Option<[u32; 2]> {
    let (_, rest) = mask.split_once("_size_")?;
    let (width, height) = rest.split_once('x')?;
    let height = height
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>();
    Some([width.parse().ok()?, height.parse().ok()?])
}

fn build_vehicle(
    sources: &mut Sources,
    vehicle: &str,
    game_version: &str,
    package_version: &semver::Version,
    evidence: &mut BTreeSet<String>,
    stage: &mut &'static str,
) -> Result<(crate::Packed, Value), String> {
    *stage = "definitions";
    let root = format!("/def/vehicle/truck/{vehicle}/");
    let data = definition(sources, &format!("{root}data.sii"), evidence)?;
    let truck = units(&data)
        .iter()
        .find(|u| u.header.starts_with("accessory_truck_data :"))
        .ok_or("missing truck metadata")?;
    let info = array(truck, "info[]");
    if info.len() < 2 {
        return Err("missing truck brand or series".into());
    }
    let brand = label(&info[0]);
    let series = label(&info[1]);
    let name = if series.starts_with(&format!("{brand} ")) {
        series
    } else {
        format!("{brand} {series}")
    };
    let paths = sources
        .files
        .keys()
        .filter(|p| p.starts_with(&root) && p.ends_with(".sii"))
        .cloned()
        .collect::<Vec<_>>();
    let mut sizes: BTreeMap<String, BTreeSet<[u32; 2]>> = BTreeMap::new();
    let mut independent = BTreeSet::new();
    for path in paths
        .iter()
        .filter(|p| p.starts_with(&format!("{root}paint_job/accessory/")))
    {
        let data = definition(sources, path, evidence)?;
        for unit in units(&data)
            .iter()
            .filter(|u| u.header.starts_with("simple_paint_job_data :"))
        {
            // Flake-only overrides also list accessories, but assign no independent texture.
            if !unit.fields.contains_key("paint_job_mask") {
                continue;
            }
            let ids = array(unit, "acc_list[]");
            for id in ids {
                independent.insert(id.clone());
                let mask = unit.field("paint_job_mask")?;
                if let Some(size) = crop_size(mask) {
                    sizes.entry(id).or_default().insert(size);
                }
            }
        }
    }
    let model_export = sources.work.join("models");
    std::fs::create_dir_all(&model_export).map_err(|e| e.to_string())?;
    let mut painted = BTreeMap::new();
    let mut cabins = BTreeMap::new();
    let mut seen_units = BTreeSet::new();
    for path in paths.iter().filter(|p| {
        ["cabin/", "chassis/", "accessory/"]
            .iter()
            .any(|group| p.starts_with(&format!("{root}{group}")))
    }) {
        let data = definition(sources, path, evidence)?;
        for unit in units(&data)
            .iter()
            .filter(|u| u.header.starts_with("accessory_"))
        {
            let (_, unit_id) = unit
                .header
                .split_once(':')
                .ok_or("invalid accessory unit")?;
            let Some((unit_name, group)) = unit_id.trim().split_once(&format!(".{vehicle}."))
            else {
                continue;
            };
            let id = format!("{group}.{unit_name}");
            if !seen_units.insert(id.clone()) {
                return Err(format!("duplicate accessory unit {id}"));
            }
            if group == "cabin" {
                cabins.insert(
                    unit_name.to_owned(),
                    unit.field("name")
                        .map(label)
                        .unwrap_or_else(|_| unit_name.to_owned()),
                );
            }
            let mut mesh = Vec::new();
            let mut selections = Vec::new();
            let mut selected_models = BTreeSet::new();
            for field in ["model", "exterior_model", "model_uk", "exterior_model_uk"] {
                let Some(_) = unit.fields.get(field) else {
                    continue;
                };
                let model = source::virtual_path(path, unit.field(field)?)?;
                if !model.ends_with(".pmd") {
                    return Err(format!("unsupported model {model}"));
                }
                let uk = field.ends_with("_uk");
                let look = unit
                    .field(if uk && unit.fields.contains_key("look_uk") {
                        "look_uk"
                    } else {
                        "look"
                    })
                    .unwrap_or("default");
                let variant = unit
                    .field(if uk && unit.fields.contains_key("variant_uk") {
                        "variant_uk"
                    } else {
                        "variant"
                    })
                    .unwrap_or("default");
                if !selected_models.insert((model.clone(), look.to_owned(), variant.to_owned())) {
                    continue;
                }
                // Read the PMD to resolve and record its binary source; conversion uses the same mount order.
                sources.locate(&model)?;
                evidence.insert(model.clone());
                let local = model_export
                    .join(model.trim_start_matches('/'))
                    .with_extension("pim");
                let pit = local.with_extension("pit");
                if !local.is_file() || !pit.is_file() {
                    *stage = "model-conversion";
                    let mounts = (0..sources.archives.len()).collect::<Vec<_>>();
                    sources.command(
                        &mounts,
                        &model_export,
                        &["-m", model.trim_end_matches(".pmd")],
                    )?;
                }
                let pim = std::fs::read_to_string(&local)
                    .map_err(|e| format!("{}: {e}", local.display()))?;
                let pit =
                    std::fs::read_to_string(&pit).map_err(|e| format!("{}: {e}", pit.display()))?;
                *stage = "geometry-selection";
                mesh.extend(
                    geometry::painted(&pim, &pit, look, variant)
                        .map_err(|e| format!("{path}, {model}, {look}/{variant}: {e}"))?,
                );
                selections.push(json!({"model": model, "look": look, "variant": variant, "uv_alias": "_TEXCOORD1"}));
            }
            // A UK variant may reuse the EU model rather than provide a model_uk field.
            if unit.fields.contains_key("variant_uk")
                && !unit.fields.contains_key("model_uk")
                && !unit.fields.contains_key("exterior_model_uk")
            {
                for field in ["model", "exterior_model"] {
                    if let Ok(model) = unit.field(field) {
                        let local = model_export
                            .join(model.trim_start_matches('/'))
                            .with_extension("pim");
                        let look = unit
                            .field("look_uk")
                            .or_else(|_| unit.field("look"))
                            .unwrap_or("default");
                        let variant = unit.field("variant_uk")?;
                        mesh.extend(geometry::painted(
                            &std::fs::read_to_string(&local).map_err(|e| e.to_string())?,
                            &std::fs::read_to_string(local.with_extension("pit"))
                                .map_err(|e| e.to_string())?,
                            look,
                            variant,
                        )?);
                        selections.push(json!({"model": model, "look": look, "variant": variant, "uv_alias": "_TEXCOORD1"}));
                    }
                }
            }
            if !mesh.is_empty() {
                let compatible = array(unit, "suitable_for[]")
                    .into_iter()
                    .filter_map(|s| {
                        s.strip_suffix(&format!(".{vehicle}.cabin"))
                            .map(str::to_owned)
                    })
                    .collect();
                painted.insert(
                    id.clone(),
                    Painted {
                        id,
                        group: group.to_owned(),
                        name: unit_name.to_owned(),
                        cabins: compatible,
                        mesh,
                        selection: selections,
                        source: path.clone(),
                    },
                );
            }
        }
    }
    *stage = "coverage";
    if cabins.is_empty() {
        return Err("truck has no cabins".into());
    }
    for id in &independent {
        if !painted.contains_key(id) {
            return Err(format!(
                "paint override has no painted exterior model: {id}"
            ));
        }
    }
    let mut main = Vec::new();
    let mut accessories = Vec::new();
    let mut templates = BTreeMap::new();
    let mut coverage = Vec::new();
    let mut part_ids = BTreeSet::new();
    let mut covered = BTreeSet::new();
    for (cabin, cabin_name) in &cabins {
        if !painted.contains_key(&format!("cabin.{cabin}")) {
            return Err(format!("cabin has no painted geometry: {cabin}"));
        }
        let contributors = painted
            .values()
            .filter(|p| {
                !independent.contains(&p.id)
                    && (p.group != "cabin" || p.name == *cabin)
                    && (p.cabins.is_empty() || p.cabins.contains(cabin))
            })
            .collect::<Vec<_>>();
        let meshes = contributors
            .iter()
            .flat_map(|p| p.mesh.clone())
            .collect::<Vec<_>>();
        let (svg, edges) = geometry::svg(&meshes, [4096, 4096])?;
        let path = format!("templates/{cabin}.svg");
        if !part_ids.insert(cabin.clone()) {
            return Err(format!("duplicate part id {cabin}"));
        }
        templates.insert(path.clone(), svg);
        main.push(json!({"id": cabin, "name": cabin_name, "game_ids": [cabin], "texture": {"size": [4096,4096], "template": path, "layout_version": 1}}));
        for part in &contributors {
            covered.insert(part.id.clone());
        }
        coverage.push(json!({"part": cabin, "size": [4096,4096], "edges": edges, "contributors": contributors.iter().map(|p| json!({"id":p.id,"definition":p.source,"selections":p.selection})).collect::<Vec<_>>()}));
    }
    for id in independent {
        let part = &painted[&id];
        let dimensions = sizes
            .get(&id)
            .ok_or_else(|| format!("missing crop dimensions for {id}"))?;
        if dimensions.len() != 1 {
            return Err(format!(
                "ambiguous crop dimensions for {id}: {dimensions:?}"
            ));
        }
        let size = *dimensions.first().unwrap();
        let part_id = id.replace('.', "_");
        if !part_ids.insert(part_id.clone()) {
            return Err(format!("colliding part id {part_id}"));
        }
        let path = format!("templates/{part_id}.svg");
        let (svg, edges) = geometry::svg(&part.mesh, size)?;
        templates.insert(path.clone(), svg);
        accessories.push(json!({"id": part_id, "name": id, "game_ids": [id], "texture": {"size": size, "template": path, "layout_version": 1}}));
        covered.insert(id.clone());
        coverage.push(json!({"part":part_id,"size":size,"edges":edges,"contributors":[{"id":id,"definition":part.source,"selections":part.selection}]}));
    }
    if let Some(id) = painted.keys().find(|id| !covered.contains(*id)) {
        return Err(format!(
            "painted definition has no compatible texture: {id}"
        ));
    }
    let game = game_version.split('.').collect::<Vec<_>>();
    if game.len() < 2 {
        return Err("invalid game version".into());
    }
    let minor = game[1]
        .parse::<u32>()
        .map_err(|_| "invalid game minor version")?;
    let manifest = json!({"format":1,"id":format!("official.ets2.{vehicle}"),"version":package_version,"name":name,"brand":brand,"kind":"truck","description":format!("Generated from ETS2 {game_version}; technical validation only."),"game":{"id":"ets2","path":vehicle,"versions":format!(">=1.{minor}, <1.{}",minor.checked_add(1).ok_or("invalid game version")?),"alt_uv":false,"colour_picker":false},"paint_job":{"main":main,"accessories":accessories}});
    *stage = "package-validation";
    let packed = crate::pack_entries(manifest, templates).map_err(|e| e.to_string())?;
    Ok((packed, json!(coverage)))
}

/// Extracts a batch, continuing after per-vehicle refusals.
pub fn run(options: &Options) -> Result<Batch, String> {
    let preliminary = reserve(Path::new("target/game-extraction"))?;
    let mut sources = Sources::new(&options.installation, &options.converter, &preliminary)?;
    let mut evidence = BTreeSet::new();
    let version = definition(&mut sources, "/version.sii", &mut evidence)?;
    let version = units(&version)
        .iter()
        .find(|s| s.header.starts_with("fs_pack_set :"))
        .ok_or("missing game version unit")?;
    if version.field("application")? != "eut2" {
        return Err("only ETS2 installations are supported".into());
    }
    let game_version = version.field("version")?.to_owned();
    sources.inventory()?;
    let discovered = sources
        .files
        .keys()
        .filter_map(|p| {
            p.strip_prefix("/def/vehicle/truck/")?
                .strip_suffix("/data.sii")
        })
        .filter(|p| !p.contains('/') && tp_vehicles::is_valid_game_name(p))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if discovered.is_empty() {
        return Err("no official ETS2 trucks discovered".into());
    }
    let vehicles = if let Some(vehicle) = &options.vehicle {
        if !discovered.contains(vehicle) {
            return Err(format!("unknown or excluded truck {vehicle}"));
        }
        vec![vehicle.clone()]
    } else {
        discovered.iter().cloned().collect()
    };
    let output = reserve(&options.output)?;
    let mut results = Vec::new();
    let mut failed = 0;
    for vehicle in &vehicles {
        eprintln!("extracting {vehicle}");
        let mut evidence = BTreeSet::new();
        let mut stage = "definitions";
        let result = build_vehicle(
            &mut sources,
            vehicle,
            &game_version,
            &options.package_version,
            &mut evidence,
            &mut stage,
        )
        .and_then(|(packed, coverage)| {
            stage = "publication";
            let filename = packed.file_name();
            publish(&output.join(&filename), &packed.bytes)?;
            Ok((filename, coverage))
        });
        match result {
            Ok((filename, coverage)) => results.push(json!({"vehicle":vehicle,"status":"success","package":filename,"technical_validation":"passed","in_game_validation":"required","textures":coverage,"sources":sources.provenance(&evidence)})),
            Err(error) => { failed += 1; results.push(json!({"vehicle":vehicle,"status":"failure","stage":stage,"error":error,"sources":sources.provenance(&evidence)})); }
        }
    }
    let report = json!({"format":1,"installation":options.installation,"converter":{"executable":sources.converter,"version":null,"version_status":"not exposed by CLI"},"tool_version":env!("CARGO_PKG_VERSION"),"game_version":game_version,"package_version":options.package_version,"archives":sources.archives,"archive_precedence":"last mounted wins; base source order and sorted DLC as observed in ETS2 mount log","requested_vehicle":options.vehicle,"discovered_vehicles":discovered,"processed_vehicles":vehicles,"work_directory":preliminary,"results":results,"failed":failed,"limits":["Ordinary eut2.truckpaint.tsnmapuv exterior models only","Alternate UV mappings are refused","Main templates union compatible painted options","Cabin size 4096 square; accessory dimensions from crop names","Package validation does not establish correctness in game"]});
    publish(
        &output.join("report.json"),
        &serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )?;
    Ok(Batch {
        directory: output,
        failed,
    })
}
