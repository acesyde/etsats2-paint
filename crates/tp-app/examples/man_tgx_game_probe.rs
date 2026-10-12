//! Throwaway companion to experiments/man-tgx-game/extract.py.
//! Reuses TruckPaint's package reader, project format and mod exporter.
//! Run: cargo run -p tp-app --release --example man_tgx_game_probe -- OUTPUT

use std::path::PathBuf;

use tp_app::export::ExportOutcome;
use tp_app::mod_export::{self, ModJob};
use tp_core::document::{Frame, Object, ObjectId, ShapeKind};
use tp_core::kurbo::Point;
use tp_vehicles::Package;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(std::env::args().nth(1).ok_or("output directory required")?);
    let packed = tp_pack::pack_folder(&output.join("vehicle"))?;
    let package_path = output.join(packed.file_name());
    let package = Package::read(&packed.bytes)?;
    let vehicle_name = package
        .manifest
        .name
        .strip_suffix(" — Game extraction probe")
        .ok_or("expected a game extraction probe package")?;
    let probe_name = format!("{vehicle_name} GAME UV PROBE");
    let stem = probe_name.to_lowercase().replace(' ', "_");
    let textures: Vec<String> = package
        .manifest
        .paint_job
        .parts()
        .map(|(_, p)| p.id.clone())
        .collect();
    let mut project = tp_app::vehicle_project::fleet_project(&probe_name, &[(&package, &textures)])
        .map_err(|e| format!("cannot create probe project: {e:?}"))?;
    project.mod_settings.author = "TruckPaint local extraction experiment".into();
    project.mod_settings.description = format!(
        "Experimental {vehicle_name} UV calibration. Each texture has numbered cells and direction labels."
    );
    let version = package
        .manifest
        .game
        .versions
        .comparators
        .first()
        .ok_or("missing game version")?;
    project.game_versions = vec![format!(
        "{}.{}.*",
        version.major,
        version.minor.ok_or("missing game minor version")?
    )];
    for i in 0..project.surfaces.len() {
        project.active_surface = i;
        let surface = project.surface();
        let size = surface.size;
        let id = surface
            .template
            .as_ref()
            .ok_or("missing surface template")?
            .texture_id
            .clone();
        let file_name = format!("{id}.png");
        let imported = tp_app::import::read_bytes(
            &file_name,
            std::fs::read(output.join("controls").join(&file_name))?,
        )
        .map_err(|e| e.to_string())?;
        if imported.size != size {
            return Err(format!("control image has the wrong size: {id}").into());
        }
        let (asset, _) = project.add_asset(&id, imported.kind, imported.bytes, imported.size);
        project.add(Object::new(
            ObjectId(0),
            ShapeKind::Image { asset },
            Frame::new(Point::new(size.width / 2.0, size.height / 2.0), size, 0.0),
        ));
    }
    project.active_surface = 0;
    let bytes = tp_file::to_bytes(&project)?;
    let reopened = tp_file::from_bytes(&bytes)?;
    assert_eq!(reopened.project.surfaces.len(), textures.len());
    let plan = mod_export::plan(&project).map_err(|p| format!("invalid mod plan: {p:?}"))?;
    let mod_path = output.join(format!("{stem}.scs"));
    let outcome = ModJob::start(
        project,
        plan,
        tp_text::FontLibrary::bundled(),
        mod_path,
        || {},
    )
    .wait();
    match outcome {
        ExportOutcome::Written(path) => println!("mod: {}", path.display()),
        other => return Err(format!("mod export failed: {other:?}").into()),
    }
    std::fs::write(&package_path, &packed.bytes)?;
    let project_path = output.join(format!("{stem}.truckpaint"));
    std::fs::write(&project_path, bytes)?;
    println!("package: {}", package_path.display());
    println!("project: {}", project_path.display());
    Ok(())
}
