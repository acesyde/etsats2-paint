//! Headless tests for official vehicles: the Official and Installed
//! sections of New Project and Add Vehicle…, the template pack dialog,
//! installing from the pack, the pack folders in Preferences, and the
//! credits in About. Packs are temporary folders holding blank PNGs of the
//! catalog's sizes.

mod common;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::Vec2;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::file_dialogs::ScriptedDialogs;
use tp_app::prefs::{Prefs, PrefsStore};
use tp_app::state::{Modal, NewProjectDraft};
use tp_app::vehicles::VehicleLibrary;
use tp_vehicles::Game;
use tp_vehicles::catalog::Entry;

type H = Harness<'static, AppState>;

const DAF: &str = "official.ets2.daf.2021";
const KRONE: &str = "official.ets2.krone.coolliner";

fn entry(id: &str) -> &'static Entry {
    tp_vehicles::catalog::by_package_id(id).expect("catalog entry")
}

/// A blank PNG of `width` × `height`.
fn png(width: u32, height: u32) -> Vec<u8> {
    let mut out = Vec::new();
    image::GrayImage::new(width, height)
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

/// The path of pack file `file` (`/`-separated) of `entry` in `pack`.
fn pack_file(pack: &Path, entry: &Entry, file: &str) -> PathBuf {
    file.split('/')
        .fold(pack.join(&entry.folder), |p, part| p.join(part))
}

/// Writes the files of the ETS2 entries at their sizes in `pack`, as the
/// pack holds them.
fn write_pack(pack: &Path) {
    for e in tp_vehicles::catalog::entries(Game::Ets2) {
        for t in e.textures() {
            let path = pack_file(pack, e, t.file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, png(t.size.width, t.size.height)).unwrap();
        }
    }
    std::fs::write(
        pack.join("Info and links.txt"),
        "current for game version 1.59, updated 2026-05-28",
    )
    .unwrap();
}

/// The app with an empty library in `dir/library`, the ETS2 pack folder
/// `pack` remembered when given.
fn app_with(dir: &Path, pack: Option<&Path>) -> H {
    let mut prefs = Prefs::default();
    if let Some(pack) = pack {
        prefs.set_template_pack(Game::Ets2, Some(pack.to_path_buf()));
    }
    let mut state = AppState::with_prefs(prefs, None);
    state.vehicles = VehicleLibrary::open(&dir.join("library"));
    let mut h = common::builder()
        .with_size(Vec2::new(1440.0, 1100.0))
        .with_step_dt(1.0 / 4.0)
        .build_ui_state(|ui, state: &mut AppState| state.show(ui), state);
    h.run();
    h
}

fn app(dir: &Path) -> H {
    app_with(dir, None)
}

/// Lets the UI settle (modals re-center over a few frames, templates
/// render in the background).
fn settle(h: &mut H) {
    h.run_steps(8);
    for _ in 0..1000 {
        if h.try_run().is_ok() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("the UI keeps repainting");
}

fn script_folders(h: &mut H, folders: &[&Path]) {
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        folders: folders.iter().map(|p| p.to_path_buf()).collect(),
        ..Default::default()
    });
}

fn draft(h: &H) -> &NewProjectDraft {
    match &h.state().modal {
        Some(Modal::NewProject(d)) => d,
        _ => panic!("New Project should be open"),
    }
}

/// Whether an official vehicle is being installed by the open dialog.
fn installing(h: &H) -> bool {
    match &h.state().modal {
        Some(Modal::NewProject(d)) => d.installing.is_some(),
        Some(Modal::AddVehicle(d)) => d.installing.is_some(),
        _ => false,
    }
}

/// Runs frames until the installation started by the open dialog ends.
fn wait_install(h: &mut H) {
    for _ in 0..2000 {
        h.step();
        if !installing(h) {
            settle(h);
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("the installation never ended");
}

fn new_project(h: &mut H) {
    h.get_by_label("New Project").click();
    settle(h);
}

/// Clicks the vehicle `name` in the list (an installation may then
/// spin: a few frames only).
fn pick(h: &mut H, name: &str) {
    h.get_by_role_and_label(Role::CheckBox, name).click();
    h.run_steps(2);
}

fn pack_dialog_open(h: &H) -> bool {
    matches!(h.state().modal, Some(Modal::TemplatePack(_)))
}

/// The top of each listed vehicle row named in `names`, in order.
fn row_tops(h: &H, names: &[&str]) -> Vec<f64> {
    names
        .iter()
        .map(|n| {
            h.get_by_role_and_label(Role::CheckBox, n)
                .accesskit_node()
                .bounding_box()
                .expect("on screen")
                .y0
        })
        .collect()
}

// --- Lists -------------------------------------------------------------

#[test]
fn official_vehicles_first() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.state_mut().vehicles = common::sample_library();
    new_project(&mut h);
    assert_eq!(draft(&h).game(), Game::Ets2);
    for label in ["Official", "Installed"] {
        assert!(h.query_by_label(label).is_some(), "{label} header");
    }
    let tops = row_tops(
        &h,
        &[
            "DAF NGD",
            "Krone Cool Liner",
            "TruckPaint Sample Trailer",
            "TruckPaint Sample Truck",
        ],
    );
    assert!(tops.windows(2).all(|w| w[0] < w[1]), "{tops:?}");
    let official = h.get_by_label("Official").accesskit_node().bounding_box();
    let installed = h.get_by_label("Installed").accesskit_node().bounding_box();
    assert!(official.unwrap().y0 < tops[0]);
    assert!(tops[1] < installed.unwrap().y0 && installed.unwrap().y0 < tops[2]);
    // The rows' cells: kind, brand, and "Official" as the package.
    let library = &h.state().vehicles;
    let filter = draft(&h).filter.clone();
    let rows = tp_app::ui::vehicle_dialogs::vehicle_rows(library, &filter, &[]);
    let cells: Vec<(String, String, String)> = rows
        .official
        .iter()
        .map(|r| (r.entry.name.clone(), r.entry.brand.clone(), r.package()))
        .collect();
    assert_eq!(
        cells,
        [
            ("DAF NGD".into(), "DAF".into(), "Official".into()),
            ("Krone Cool Liner".into(), "Krone".into(), "Official".into()),
        ]
    );
    assert_eq!(
        rows.official[0].summary(),
        "DAF · Truck · ETS2 · Official",
        "kind in the summary"
    );
}

#[test]
fn official_vehicle_filtered_by_kind() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.state_mut().vehicles = common::sample_library();
    new_project(&mut h);
    h.get_by_label("Trailers").click();
    settle(&mut h);
    assert!(h.query_by_label("Krone Cool Liner").is_some());
    assert!(h.query_by_label("DAF NGD").is_none());
    assert!(h.query_by_label("TruckPaint Sample Truck").is_none());
    // The search applies to both sections.
    h.get_by_label("Trucks").click();
    settle(&mut h);
    assert!(h.query_by_label("DAF NGD").is_some());
    assert!(h.query_by_label("Krone Cool Liner").is_none());
}

#[test]
fn no_network_needed() {
    // Offline, with nothing installed and no pack: the catalog is data
    // built into the app.
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    new_project(&mut h);
    for name in ["DAF NGD", "Krone Cool Liner"] {
        assert!(
            h.query_by_role_and_label(Role::CheckBox, name).is_some(),
            "{name}"
        );
    }
    // A single section: no headers. The sample vehicles stay offered.
    assert!(h.query_by_label("Installed").is_none());
    assert!(h.query_by_label("Install the sample vehicles").is_some());
    // No crate that lists or builds official vehicles depends on a network
    // client.
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for krate in ["tp-app", "tp-pack", "tp-vehicles"] {
        let manifest = std::fs::read_to_string(crates.join(krate).join("Cargo.toml")).unwrap();
        for client in ["reqwest", "ureq", "hyper", "curl", "attohttpc", "isahc"] {
            assert!(!manifest.contains(client), "{krate} depends on {client}");
        }
    }
}

#[test]
fn no_vehicle_installed_for_ats() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    new_project(&mut h);
    h.get_by_label("American Truck Simulator").click();
    settle(&mut h);
    for label in ["Install…", "Install the sample vehicles", "Custom vehicle…"] {
        assert!(h.query_by_label(label).is_some(), "{label}");
    }
    assert!(h.query_by_label("DAF NGD").is_none());
    assert!(
        h.get_by_label("Create Project")
            .accesskit_node()
            .is_disabled()
    );
}

#[test]
fn an_installed_official_vehicle_is_listed_once() {
    let dir = tempfile::tempdir().unwrap();
    let pack = dir.path().join("pack");
    write_pack(&pack);
    let mut h = app_with(dir.path(), Some(&pack));
    new_project(&mut h);
    pick(&mut h, "Krone Cool Liner");
    wait_install(&mut h);
    assert!(h.state().vehicles.get(KRONE).is_some());
    assert_eq!(
        h.query_all_by_role_and_label(Role::CheckBox, "Krone Cool Liner")
            .count(),
        1
    );
    // Only the Official section: no headers.
    assert!(h.query_by_label("Installed").is_none());
    let rows = tp_app::ui::vehicle_dialogs::vehicle_rows(
        &h.state().vehicles,
        &draft(&h).filter.clone(),
        &[],
    );
    assert!(rows.installed.is_empty());
    assert_eq!(rows.official[1].package(), "Official 1.0.0");
}

// --- The template pack dialog -------------------------------------------

#[test]
fn first_official_vehicle_asks_for_the_pack() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    new_project(&mut h);
    pick(&mut h, "DAF NGD");
    settle(&mut h);
    assert!(pack_dialog_open(&h));
    assert!(h.query_by_label("ETS2 template pack").is_some());
    assert!(h.query_by_label("Open the download page").is_some());
    assert!(h.query_by_label("Choose the pack folder…").is_some());
    assert!(h.state().vehicles.vehicles().is_empty());
}

#[test]
fn download_page() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    new_project(&mut h);
    pick(&mut h, "DAF NGD");
    settle(&mut h);
    h.get_by_label("Open the download page").click();
    h.step();
    let urls: Vec<String> = h
        .output()
        .platform_output
        .commands
        .iter()
        .filter_map(|c| match c {
            egui::OutputCommand::OpenUrl(open) => Some(open.url.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        urls,
        ["https://forum.scssoft.com/viewtopic.php?f=33&t=272386"]
    );
    assert!(pack_dialog_open(&h));
}

#[test]
fn copy_download_link() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    new_project(&mut h);
    pick(&mut h, "DAF NGD");
    settle(&mut h);
    h.get_by_label("Copy the link").click();
    h.step();
    let copied: Vec<String> = h
        .output()
        .platform_output
        .commands
        .iter()
        .filter_map(|c| match c {
            egui::OutputCommand::CopyText(text) => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        copied,
        ["https://forum.scssoft.com/viewtopic.php?f=33&t=272386"]
    );
    h.step();
    assert!(h.query_by_label("Link copied").is_some());
    assert!(pack_dialog_open(&h));
}

#[test]
fn folder_unzipped_inside_another() {
    let dir = tempfile::tempdir().unwrap();
    let outer = dir.path().join("Downloads");
    let inner = outer.join("ETS2 templates");
    write_pack(&inner);
    let mut h = app(dir.path());
    new_project(&mut h);
    pick(&mut h, "DAF NGD");
    settle(&mut h);
    script_folders(&mut h, &[&outer]);
    h.get_by_label("Choose the pack folder…").click();
    // The DAF then installs from it: wait for the end of its spinner.
    wait_install(&mut h);
    assert_eq!(
        h.state().prefs.template_pack(Game::Ets2),
        Some(inner.as_path())
    );
    wait_install(&mut h);
    assert_eq!(draft(&h).vehicles.last().map(|c| c.id.as_str()), Some(DAF));
}

#[test]
fn wrong_folder() {
    let dir = tempfile::tempdir().unwrap();
    let downloads = dir.path().join("Downloads");
    std::fs::create_dir_all(downloads.join("photos")).unwrap();
    let mut h = app(dir.path());
    new_project(&mut h);
    pick(&mut h, "DAF NGD");
    settle(&mut h);
    script_folders(&mut h, &[&downloads]);
    h.get_by_label("Choose the pack folder…").click();
    settle(&mut h);
    assert!(pack_dialog_open(&h));
    assert!(
        h.query_by_label("This folder doesn't hold the ETS2 template pack.")
            .is_some()
    );
    assert!(h.state().prefs.template_packs.is_empty());
}

/// A folder dropped from the operating system.
#[derive(Debug)]
struct DroppedFolder(PathBuf);

impl egui::DroppedFile for DroppedFolder {
    fn path(&self) -> &Path {
        &self.0
    }

    fn bytes(&self) -> Result<Vec<u8>, String> {
        Err("a folder".into())
    }
}

#[test]
fn dropped_folder() {
    let dir = tempfile::tempdir().unwrap();
    let pack = dir.path().join("templates");
    write_pack(&pack);
    let mut h = app(dir.path());
    new_project(&mut h);
    pick(&mut h, "Krone Cool Liner");
    settle(&mut h);
    assert!(pack_dialog_open(&h));
    h.input_mut().dropped_files =
        vec![Arc::new(DroppedFolder(pack.clone())) as Arc<dyn egui::DroppedFile + Send + Sync>];
    h.step();
    h.input_mut().dropped_files.clear();
    assert_eq!(
        h.state().prefs.template_pack(Game::Ets2),
        Some(pack.as_path())
    );
    wait_install(&mut h);
    assert_eq!(
        draft(&h).vehicles.last().map(|c| c.id.as_str()),
        Some(KRONE)
    );
    // Nothing was placed or opened besides.
    assert!(!h.state().has_project());
}

#[test]
fn pack_moved() {
    let dir = tempfile::tempdir().unwrap();
    let gone = dir.path().join("deleted pack");
    let mut h = app_with(dir.path(), Some(&gone));
    new_project(&mut h);
    pick(&mut h, "Krone Cool Liner");
    settle(&mut h);
    assert!(pack_dialog_open(&h));
    assert!(
        h.query_by_label("The ETS2 template pack folder was moved or deleted. Choose it again.")
            .is_some()
    );
}

#[test]
fn cancel_returns_to_new_project_as_it_was() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.state_mut().vehicles = common::sample_library();
    new_project(&mut h);
    pick(&mut h, "TruckPaint Sample Truck");
    if let Some(Modal::NewProject(d)) = &mut h.state_mut().modal {
        d.name = "ACE".into();
        d.filter.query = "a".into();
    }
    settle(&mut h);
    let before = draft(&h).clone();
    pick(&mut h, "DAF NGD");
    settle(&mut h);
    assert!(pack_dialog_open(&h));
    h.get_by_label("Cancel").click();
    settle(&mut h);
    let after = draft(&h);
    assert_eq!(after.name, before.name);
    assert_eq!(after.filter.query, before.filter.query);
    assert_eq!(after.vehicles, before.vehicles);
    assert!(after.installing.is_none());
    assert!(h.state().prefs.template_packs.is_empty());
}

// --- Installing -----------------------------------------------------------

#[test]
fn pick_and_install() {
    let dir = tempfile::tempdir().unwrap();
    let pack = dir.path().join("pack");
    write_pack(&pack);
    let mut h = app_with(dir.path(), Some(&pack));
    new_project(&mut h);
    tp_app::official::hold_builds_from(&pack, true);
    pick(&mut h, "DAF NGD");
    h.run_steps(2);
    assert!(h.query_by_label("Installing DAF NGD…").is_some());
    assert!(draft(&h).vehicles.is_empty(), "the previous choice is kept");
    assert!(!pack_dialog_open(&h));
    tp_app::official::hold_builds_from(&pack, false);
    wait_install(&mut h);
    let choice = draft(&h).vehicles.last().cloned().expect("chosen");
    assert_eq!(choice.id, DAF);
    assert_eq!(
        choice.textures,
        [
            "xg_plus",
            "sun_visor",
            "sideskirts",
            "mirrors_standard",
            "mirrors_camera"
        ]
    );
    let m = choice.manifest(&h.state().vehicles).unwrap();
    assert_eq!(m.name, "DAF NGD");
    assert_eq!(m.version, entry(DAF).version);
}

#[test]
fn installed_without_the_pack() {
    let dir = tempfile::tempdir().unwrap();
    let pack = dir.path().join("pack");
    write_pack(&pack);
    let mut h = app_with(dir.path(), Some(&pack));
    new_project(&mut h);
    pick(&mut h, "DAF NGD");
    wait_install(&mut h);
    std::fs::remove_dir_all(&pack).unwrap();
    pick(&mut h, "Krone Cool Liner");
    settle(&mut h);
    // The Krone needs the pack: asked for, then cancelled.
    assert!(pack_dialog_open(&h));
    h.get_by_label("Cancel").click();
    settle(&mut h);
    // The DAF stays checked; unchecked, then checked again without the
    // pack.
    assert_eq!(draft(&h).vehicles.last().map(|c| c.id.as_str()), Some(DAF));
    pick(&mut h, "DAF NGD");
    settle(&mut h);
    assert!(draft(&h).vehicles.is_empty());
    pick(&mut h, "DAF NGD");
    settle(&mut h);
    assert!(!pack_dialog_open(&h));
    assert!(!installing(&h));
    assert_eq!(draft(&h).vehicles.last().map(|c| c.id.as_str()), Some(DAF));
}

#[test]
fn cancel_the_installation() {
    let dir = tempfile::tempdir().unwrap();
    let pack = dir.path().join("pack");
    write_pack(&pack);
    let mut h = app_with(dir.path(), Some(&pack));
    new_project(&mut h);
    tp_app::official::hold_builds_from(&pack, true);
    pick(&mut h, "Krone Cool Liner");
    h.run_steps(2);
    assert!(h.query_by_label("Installing Krone Cool Liner…").is_some());
    // The installation's Cancel comes before the dialog's.
    h.get_all_by_label("Cancel").next().unwrap().click();
    h.run();
    assert!(matches!(h.state().modal, Some(Modal::NewProject(_))));
    assert!(!installing(&h));
    assert!(h.query_by_label("Installing Krone Cool Liner…").is_none());
    tp_app::official::hold_builds_from(&pack, false);
    std::thread::sleep(std::time::Duration::from_millis(100));
    settle(&mut h);
    assert!(draft(&h).vehicles.is_empty());
    assert!(h.state().vehicles.get(KRONE).is_none());
    assert!(!dir.path().join("library").join(KRONE).exists());
}

#[test]
fn missing_file() {
    let dir = tempfile::tempdir().unwrap();
    let pack = dir.path().join("pack");
    write_pack(&pack);
    std::fs::remove_file(pack.join("DAF NGD/Cabin B (XG).png")).unwrap();
    let mut h = app_with(dir.path(), Some(&pack));
    new_project(&mut h);
    pick(&mut h, "DAF NGD");
    wait_install(&mut h);
    let messages = &draft(&h).messages;
    assert_eq!(messages.len(), 1);
    let Err(message) = &messages[0] else {
        panic!("refused: {messages:?}");
    };
    assert!(message.contains("DAF NGD"), "{message}");
    assert!(message.contains("Cabin B (XG).png"), "{message}");
    assert!(h.query_by_label(message).is_some());
    assert!(h.state().vehicles.vehicles().is_empty());
    assert!(draft(&h).vehicles.is_empty());
}

#[test]
fn pack_of_another_version() {
    let dir = tempfile::tempdir().unwrap();
    let pack = dir.path().join("pack");
    write_pack(&pack);
    std::fs::write(
        pack.join("Krone Cool Liner/Krone Cool Liner.png"),
        png(4096, 1024),
    )
    .unwrap();
    let mut h = app_with(dir.path(), Some(&pack));
    new_project(&mut h);
    pick(&mut h, "Krone Cool Liner");
    wait_install(&mut h);
    let Some(Err(message)) = draft(&h).messages.first() else {
        panic!("refused");
    };
    assert!(message.contains("Krone Cool Liner.png"), "{message}");
    assert!(message.contains("4096 × 1024"), "{message}");
    assert!(message.contains("another game version"), "{message}");
    assert!(h.state().vehicles.vehicles().is_empty());
}

#[test]
fn project_from_an_official_vehicle() {
    let dir = tempfile::tempdir().unwrap();
    let pack = dir.path().join("pack");
    write_pack(&pack);
    let mut h = app_with(dir.path(), Some(&pack));
    new_project(&mut h);
    pick(&mut h, "DAF NGD");
    wait_install(&mut h);
    h.get_by_label("XF").click();
    settle(&mut h);
    h.get_by_label("Create Project").click();
    h.run();
    let ws = h.state().workspace().expect("project open");
    assert_eq!(ws.project.name, "DAF NGD");
    let names: Vec<&str> = ws
        .project
        .surfaces
        .iter()
        .map(|s| s.name.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "XG+",
            "XF",
            "Sun Visor",
            "Sideskirts",
            "Mirrors (standard)",
            "Mirrors (camera)"
        ]
    );
    assert!(ws.project.surfaces.iter().all(|s| s.template.is_some()));
    let xf = &ws.project.surfaces[1];
    assert_eq!((xf.size.width, xf.size.height), (4096.0, 4096.0));
}

#[test]
fn official_trailer_added_to_a_fleet() {
    let dir = tempfile::tempdir().unwrap();
    let pack = dir.path().join("pack");
    write_pack(&pack);
    let mut h = app_with(dir.path(), Some(&pack));
    common::create_project(&mut h);
    h.state_mut()
        .queue
        .push(tp_app::commands::CommandId::AddVehicle);
    settle(&mut h);
    assert!(h.query_by_label("Official").is_some());
    pick(&mut h, "Krone Cool Liner");
    wait_install(&mut h);
    match &h.state().modal {
        Some(Modal::AddVehicle(d)) => {
            assert_eq!(d.choices.last().map(|c| c.id.as_str()), Some(KRONE));
        }
        _ => panic!("Add Vehicle should be open"),
    }
    h.get_by_label("Add").click();
    settle(&mut h);
    let ws = h.state().workspace().expect("project open");
    assert_eq!(ws.project.vehicles.len(), 2);
    assert_eq!(ws.project.vehicles[1].package_id, KRONE);
    let added: Vec<&str> = ws.project.surfaces[4..]
        .iter()
        .map(|s| s.name.as_str())
        .collect();
    assert_eq!(
        added,
        [
            "Body",
            "Chassis front pallet box",
            "Chassis front side skirt",
            "Chassis front simple side skirt",
            "Chassis rear pallet box"
        ]
    );
    assert_eq!(ws.project.active_surface, 4);
}

/// vehicle-projects "Several vehicles in one step".
#[test]
fn several_vehicles_in_one_step() {
    let dir = tempfile::tempdir().unwrap();
    let pack = dir.path().join("pack");
    write_pack(&pack);
    let mut h = app_with(dir.path(), Some(&pack));
    common::create_project(&mut h);
    h.state_mut()
        .queue
        .push(tp_app::commands::CommandId::AddVehicle);
    settle(&mut h);
    assert!(
        h.get_by_label("Add").accesskit_node().is_disabled(),
        "nothing checked"
    );
    pick(&mut h, "TruckPaint Sample Trailer");
    settle(&mut h);
    // Its textures under its row.
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Mudflaps")
            .is_some()
    );
    pick(&mut h, "Krone Cool Liner");
    wait_install(&mut h);
    let checked: Vec<String> = match &h.state().modal {
        Some(Modal::AddVehicle(d)) => d.choices.iter().map(|c| c.id.clone()).collect(),
        _ => panic!("Add Vehicle should be open"),
    };
    assert_eq!(checked, [tp_app::vehicles::SAMPLE_TRAILER_ID, KRONE]);
    for name in ["TruckPaint Sample Trailer", "Krone Cool Liner"] {
        let row = h.get_by_role_and_label(Role::CheckBox, name);
        assert_eq!(
            row.accesskit_node().toggled(),
            Some(egui::accesskit::Toggled::True),
            "{name}"
        );
    }
    h.get_by_label("Add").click();
    settle(&mut h);
    assert!(h.state().modal.is_none());
    let ws = h.state().workspace().expect("project open");
    let vehicles: Vec<&str> = ws
        .project
        .vehicles
        .iter()
        .map(|v| v.package_id.as_str())
        .collect();
    assert_eq!(
        vehicles,
        [
            tp_app::vehicles::SAMPLE_ID,
            tp_app::vehicles::SAMPLE_TRAILER_ID,
            KRONE
        ]
    );
    let added: Vec<&str> = ws.project.surfaces[4..]
        .iter()
        .map(|s| s.name.as_str())
        .collect();
    assert_eq!(
        added,
        [
            "Base",
            "Curtain body 13.6 m",
            "Curtain body 10.5 m",
            "Mudflaps",
            "Body",
            "Chassis front pallet box",
            "Chassis front side skirt",
            "Chassis front simple side skirt",
            "Chassis rear pallet box"
        ]
    );
    assert_eq!(ws.project.active_surface, 4, "the sample trailer's Base");
    h.state_mut().workspace_mut().unwrap().undo();
    h.run();
    let ws = h.state().workspace().unwrap();
    assert_eq!(ws.project.vehicles.len(), 1, "one Undo removes both");
    assert_eq!(ws.project.surfaces.len(), 4);
}

// --- Preferences ------------------------------------------------------------

fn open_preferences(h: &mut H) {
    h.state_mut().modal = Some(Modal::Preferences);
    settle(h);
}

#[test]
fn pack_folder_survives_restart() {
    let dir = tempfile::tempdir().unwrap();
    let pack = dir.path().join("pack");
    write_pack(&pack);
    let config = dir.path().join("config");
    let store = || Some(PrefsStore::new(&config));
    let mut state = AppState::new(store());
    state
        .prefs
        .set_template_pack(Game::Ets2, Some(pack.clone()));
    state.persist_prefs(f64::MAX, true);
    let mut state = AppState::new(store());
    state.dialogs = Box::new(ScriptedDialogs::default());
    state.system_fonts = false;
    state.vehicles = VehicleLibrary::open(&dir.path().join("library"));
    let mut h = common::builder()
        .with_size(Vec2::new(1440.0, 1100.0))
        .with_step_dt(1.0 / 4.0)
        .build_ui_state(|ui, state: &mut AppState| state.show(ui), state);
    open_preferences(&mut h);
    assert!(h.query_by_label(&pack.display().to_string()).is_some());
    h.get_by_label("Done").click();
    settle(&mut h);
    new_project(&mut h);
    pick(&mut h, "DAF NGD");
    assert!(!pack_dialog_open(&h));
    wait_install(&mut h);
    assert_eq!(draft(&h).vehicles.last().map(|c| c.id.as_str()), Some(DAF));
}

#[test]
fn change_the_pack_folder() {
    let dir = tempfile::tempdir().unwrap();
    let old = dir.path().join("old");
    write_pack(&old);
    let newer = dir.path().join("1.60");
    write_pack(&newer.join("ETS2 templates"));
    let mut h = app_with(dir.path(), Some(&old));
    h.state_mut().vehicles = common::sample_library();
    let installed = h.state().vehicles.vehicles().len();
    open_preferences(&mut h);
    assert!(h.query_by_label("Template packs").is_some());
    script_folders(&mut h, &[&newer]);
    h.get_by_label("Choose…").click();
    settle(&mut h);
    let remembered = newer.join("ETS2 templates");
    assert_eq!(
        h.state().prefs.template_pack(Game::Ets2),
        Some(remembered.as_path())
    );
    assert!(
        h.query_by_label(&remembered.display().to_string())
            .is_some()
    );
    assert_eq!(h.state().vehicles.vehicles().len(), installed);
    // Clear forgets it.
    h.get_by_label("Clear").click();
    settle(&mut h);
    assert_eq!(h.state().prefs.template_pack(Game::Ets2), None);
    assert!(h.query_by_label("Not set").is_some());
}

#[test]
fn refused_pack_folder() {
    let dir = tempfile::tempdir().unwrap();
    let old = dir.path().join("old");
    write_pack(&old);
    let empty = dir.path().join("empty");
    std::fs::create_dir_all(&empty).unwrap();
    let mut h = app_with(dir.path(), Some(&old));
    open_preferences(&mut h);
    script_folders(&mut h, &[&empty]);
    h.get_by_label("Choose…").click();
    settle(&mut h);
    assert!(
        h.query_by_label("This folder doesn't hold the ETS2 template pack.")
            .is_some()
    );
    assert_eq!(
        h.state().prefs.template_pack(Game::Ets2),
        Some(old.as_path())
    );
    // Closing the dialog forgets the message.
    h.get_by_label("Done").click();
    settle(&mut h);
    open_preferences(&mut h);
    assert!(
        h.query_by_label("This folder doesn't hold the ETS2 template pack.")
            .is_none()
    );
}

#[test]
fn template_packs_list_only_games_with_official_vehicles() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    open_preferences(&mut h);
    assert!(h.query_by_label("Template packs").is_some());
    assert!(h.query_by_label("ETS2").is_some());
    assert!(h.query_by_label("Not set").is_some());
    assert!(h.query_by_label("ATS").is_none());
    assert_eq!(h.query_all_by_label("Choose…").count(), 1);
}

// --- About --------------------------------------------------------------------

#[test]
fn credits() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.state_mut().modal = Some(Modal::About);
    settle(&mut h);
    let credits = "Vehicle data from Paintjob Packer by Carsmaniac (MIT license). Template packs by Carsmaniac.";
    assert!(h.query_by_label(credits).is_some());
    assert!(
        h.query_by_label(tp_vehicles::catalog::PAINTJOB_PACKER_LICENSE)
            .is_none()
    );
    h.get_by_label("Licenses…").click();
    settle(&mut h);
    // The notice shown is Paintjob Packer's MIT license.
    let notice = tp_vehicles::catalog::PAINTJOB_PACKER_LICENSE;
    assert!(h.query_by_label(notice).is_some());
    assert!(notice.starts_with("MIT License"));
    assert!(notice.contains("Copyright (c) 2020 Carsmaniac"));
}

/// The real ETS2 template pack, when `TRUCKPAINT_ETS2_PACK` points to it:
/// both vehicles install from it and a fleet of the two plans a mod with
/// every game id of the catalog.
#[test]
#[ignore = "needs the ETS2 template pack: TRUCKPAINT_ETS2_PACK=<folder>"]
fn installs_and_exports_from_the_real_pack() {
    let Some(folder) = std::env::var_os("TRUCKPAINT_ETS2_PACK") else {
        panic!("set TRUCKPAINT_ETS2_PACK to the unzipped ETS2 template pack");
    };
    let pack = tp_pack::catalog::find_pack(Path::new(&folder), Game::Ets2).expect("an ETS2 pack");
    let dir = tempfile::tempdir().unwrap();
    let mut library = VehicleLibrary::open(&dir.path().join("library"));
    let mut packages = Vec::new();
    for id in [DAF, KRONE] {
        let packed = tp_pack::catalog::build(entry(id), &pack).unwrap();
        library.install_bytes(&packed.bytes).unwrap();
        let version = &library.get(id).unwrap().newest().manifest.version;
        packages.push(library.load(id, version).unwrap());
    }
    // The DAF on XG+ with every accessory, the Krone with all its textures.
    let all = |p: &tp_vehicles::Package| tp_app::vehicle_project::default_textures(&p.manifest);
    let project =
        tp_app::vehicle_project::fleet_project("ACE", &[(&packages[0], &all(&packages[0]))])
            .unwrap();
    let mut ws = tp_app::workspace::Workspace::new(project);
    ws.add_vehicle(&packages[1], &all(&packages[1]), 0.0)
        .unwrap();
    let mut project = ws.project.clone();
    project.mod_settings.internal_name = Some("ace".into());
    let plan = tp_app::mod_export::plan(&project).unwrap();

    let daf = plan
        .text("def/vehicle/truck/daf.2021/paint_job/ace_a.sii")
        .unwrap();
    assert!(
        daf.contains("suitable_for[]: \"xg_plus.daf.2021.cabin\""),
        "{daf}"
    );
    let daf_acc = plan
        .text("def/vehicle/truck/daf.2021/paint_job/accessory/ace_a.sii")
        .unwrap();
    let krone_acc = plan
        .text("def/vehicle/trailer_owned/krone.coolliner/paint_job/accessory/ace.sii")
        .unwrap();
    assert!(krone_acc.contains(
        "paint_job_mask: \"/vehicle/trailer_owned/upgrade/paintjob/ace/krone.coolliner/body.tobj\""
    ));
    // Every id the catalog covers is in the mod, the left-out ones are not.
    for (file, id) in [(daf_acc, DAF), (krone_acc, KRONE)] {
        let e = entry(id);
        for t in e.accessories.iter() {
            for game_id in &t.ids {
                assert!(
                    file.contains(&format!("acc_list[]: \"{game_id}\"")),
                    "{game_id}"
                );
            }
        }
        for m in e.main.iter() {
            for game_id in &m.also_paints {
                assert!(
                    file.contains(&format!("acc_list[]: \"{game_id}\"")),
                    "{game_id}"
                );
            }
        }
        for left in e.left_out.iter() {
            for game_id in &left.ids {
                assert!(!file.contains(game_id.as_str()), "{game_id}");
            }
        }
    }
    let krone_body = plan
        .files
        .get("vehicle/trailer_owned/upgrade/paintjob/ace/krone.coolliner/body.dds")
        .unwrap();
    assert!(matches!(
        krone_body,
        tp_app::mod_export::Entry::Texture {
            width: 8192,
            height: 1024,
            ..
        }
    ));
}

/// Writes the mod for the in-game check to `TRUCKPAINT_CHECK_MOD` (an
/// `.scs` path): the DAF NGD (every cabin and accessory) and the Krone Cool
/// Liner from the real pack, each texture filled with its own color, with a
/// white square at its top left and a black one at its bottom right.
#[test]
#[ignore = "writes the in-game check mod: TRUCKPAINT_ETS2_PACK and TRUCKPAINT_CHECK_MOD"]
fn writes_the_in_game_check_mod() {
    use tp_core::document::{Frame, Object, ObjectId, Paint, Rgba, ShapeKind};
    use tp_core::kurbo::{Point, Size};

    let folder = std::env::var_os("TRUCKPAINT_ETS2_PACK").expect("TRUCKPAINT_ETS2_PACK");
    let out =
        PathBuf::from(std::env::var_os("TRUCKPAINT_CHECK_MOD").expect("TRUCKPAINT_CHECK_MOD"));
    let pack = tp_pack::catalog::find_pack(Path::new(&folder), Game::Ets2).expect("an ETS2 pack");
    let packages: Vec<_> = [DAF, KRONE]
        .iter()
        .map(|id| {
            let packed = tp_pack::catalog::build(entry(id), &pack).unwrap();
            tp_vehicles::Package::read(&packed.bytes).unwrap()
        })
        .collect();
    let all = |p: &tp_vehicles::Package| -> Vec<String> {
        p.manifest
            .paint_job
            .parts()
            .map(|(_, part)| part.id.clone())
            .collect()
    };
    let project = tp_app::vehicle_project::fleet_project(
        "TruckPaint check",
        &[(&packages[0], &all(&packages[0]))],
    )
    .unwrap();
    let mut ws = tp_app::workspace::Workspace::new(project);
    ws.add_vehicle(&packages[1], &all(&packages[1]), 0.0)
        .unwrap();
    let mut project = ws.project.clone();
    project.mod_settings.internal_name = Some("tpcheck".into());
    project.mod_settings.name = "TruckPaint check".into();

    const COLORS: [(Rgba, &str); 12] = [
        (Rgba::rgb(0xE5, 0x39, 0x35), "red"),
        (Rgba::rgb(0x1E, 0x88, 0xE5), "blue"),
        (Rgba::rgb(0x43, 0xA0, 0x47), "green"),
        (Rgba::rgb(0xFD, 0xD8, 0x35), "yellow"),
        (Rgba::rgb(0x8E, 0x24, 0xAA), "purple"),
        (Rgba::rgb(0xFB, 0x8C, 0x00), "orange"),
        (Rgba::rgb(0x00, 0xAC, 0xC1), "cyan"),
        (Rgba::rgb(0xD8, 0x1B, 0x60), "pink"),
        (Rgba::rgb(0x6D, 0x4C, 0x41), "brown"),
        (Rgba::rgb(0xC0, 0xCA, 0x33), "lime"),
        (Rgba::rgb(0x3F, 0x51, 0xB5), "indigo"),
        (Rgba::rgb(0x75, 0x75, 0x75), "grey"),
    ];
    let rect = |x: f64, y: f64, w: f64, h: f64, color: Rgba| {
        let mut o = Object::new(
            ObjectId(1),
            ShapeKind::rectangle(),
            Frame::new(Point::new(x + w / 2.0, y + h / 2.0), Size::new(w, h), 0.0),
        );
        o.fill = Paint::Solid(color);
        o
    };
    let mut legend = String::new();
    for i in 0..project.surfaces.len() {
        project.active_surface = i;
        let size = project.surfaces[i].size;
        let mark = size.width.min(size.height) / 4.0;
        let (color, color_name) = COLORS[i % COLORS.len()];
        project.add(rect(0.0, 0.0, size.width, size.height, color));
        project.add(rect(0.0, 0.0, mark, mark, Rgba::rgb(0xFF, 0xFF, 0xFF)));
        project.add(rect(
            size.width - mark,
            size.height - mark,
            mark,
            mark,
            Rgba::rgb(0, 0, 0),
        ));
        legend.push_str(&format!(
            "{} · {}: {color_name}\n",
            tp_app::mod_export::texture_label(&project, i),
            project.surfaces[i].size_label(),
        ));
    }
    let plan = tp_app::mod_export::plan(&project).unwrap();
    let job = tp_app::mod_export::ModJob::start(
        project,
        plan,
        tp_text::FontLibrary::bundled(),
        out.clone(),
        || {},
    );
    let outcome = job.wait();
    assert!(
        matches!(outcome, tp_app::export::ExportOutcome::Written(_)),
        "{outcome:?}"
    );
    std::fs::write(out.with_extension("txt"), legend).unwrap();
}
