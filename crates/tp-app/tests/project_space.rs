//! Headless tests for the Project space: the Vehicles header, the vehicle
//! cards and their textures, and the Mod information column.

mod common;

use egui::Vec2;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::layout::Space;
use tp_app::state::Modal;
use tp_app::workspace::Workspace;
use tp_core::document::{Frame, Object, ObjectId, Rgba, ShapeKind};
use tp_core::kurbo::{Point, Size};
use tp_vehicles::Package;

type H = Harness<'static, AppState>;

const TRUCK: &str = "TruckPaint Sample Truck";
const TRAILER: &str = "TruckPaint Sample Trailer";

fn ws(h: &H) -> &Workspace {
    h.state().workspace().expect("project open")
}

fn ws_mut(h: &mut H) -> &mut Workspace {
    h.state_mut().workspace_mut().expect("project open")
}

fn sample(i: usize) -> Package {
    Package::read(tp_app::vehicles::SAMPLES[i].bytes).unwrap()
}

fn ids(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

/// A tall window showing, in the Project space, a project named "ACE
/// Logistics" for the sample truck painting `truck` (if any) and the
/// sample trailer painting `trailer` (if any).
fn open(truck: Option<&[&str]>, trailer: Option<&[&str]>) -> H {
    let mut h = Harness::builder()
        .with_size(Vec2::new(1440.0, 1800.0))
        .build_ui_state(
            |ui, state: &mut AppState| state.show(ui),
            AppState::with_prefs(Default::default(), None),
        );
    h.state_mut().vehicles = common::sample_library();
    let (first, then) = match (truck, trailer) {
        (Some(t), r) => ((sample(0), t), r.map(|r| (sample(1), r))),
        (None, Some(r)) => ((sample(1), r), None),
        (None, None) => unreachable!(),
    };
    let project =
        tp_app::vehicle_project::fleet_project("ACE Logistics", &first.0, &ids(first.1)).unwrap();
    h.state_mut().open_project(project);
    if let Some((package, textures)) = then {
        ws_mut(&mut h)
            .add_vehicle(&package, &ids(textures), 0.0)
            .unwrap();
    }
    ws_mut(&mut h).space = Space::Project;
    settle(&mut h);
    h
}

/// Runs frames until the thumbnails and the mod's pictures are rendered.
fn settle(h: &mut H) {
    common::settle_renders(h);
}

/// Whether the tile of texture `path` is highlighted.
fn highlighted(h: &H, path: &str) -> bool {
    h.get_by_label(&format!("Texture {path}"))
        .accesskit_node()
        .toggled()
        == Some(egui::accesskit::Toggled::True)
}

const WHOLE_TRUCK: &[&str] = &[
    "standard",
    "high_roof",
    "chassis",
    "cab_accessories",
    "side_skirts",
];
const WHOLE_TRAILER: &[&str] = &["base", "body_13_6", "body_10_5", "mudflaps"];

#[test]
fn header_counts_the_vehicles_and_textures() {
    let h = open(Some(WHOLE_TRUCK), Some(WHOLE_TRAILER));
    assert!(h.query_by_label("Vehicles").is_some());
    assert!(h.query_by_label("2 vehicles · 9 textures").is_some());
    assert!(h.query_by_label("Add Vehicle…").is_some());
}

#[test]
fn one_vehicle_and_add_vehicle() {
    let mut h = open(None, Some(&["base", "mudflaps"]));
    assert!(h.query_by_label("1 vehicle · 2 textures").is_some());
    h.get_by_label("Add Vehicle…").click();
    settle(&mut h);
    assert!(matches!(h.state().modal, Some(Modal::AddVehicle(_))));
    assert!(
        h.query_by_role_and_label(Role::RadioButton, TRUCK)
            .is_some()
    );
    assert!(
        h.query_by_role_and_label(Role::RadioButton, TRAILER)
            .is_none(),
        "only vehicles that aren't in the project"
    );
}

#[test]
fn truck_and_trailer_cards() {
    let h = open(
        Some(&["standard", "high_roof"]),
        Some(&["base", "mudflaps"]),
    );
    assert!(h.query_by_label("Truck · package 1.1.0").is_some());
    // The cabins by name; their internal names are in the tooltip only.
    assert!(h.query_by_label("Standard cab, High roof").is_some());
    assert!(h.query_by_label_contains("high_roof").is_none());
    assert!(h.query_by_label("One per cabin layout").is_some());
    assert!(h.query_by_label("Trailer · package 1.0.0").is_some());
    assert!(h.query_by_label("Single main texture").is_some());
    // A trailer has no Cabins line; no update is installed.
    assert_eq!(h.query_all_by_label("Cabins").count(), 1);
    assert!(h.query_by_label_contains("available").is_none());
    // The actions are named with the vehicle's name.
    for name in [TRUCK, TRAILER] {
        assert!(h.query_by_label(&format!("Actions for {name}")).is_some());
        assert!(h.query_by_label(&format!("Textures of {name}")).is_some());
    }
}

#[test]
fn textures_listed_under_main_textures_and_accessories() {
    let h = open(Some(WHOLE_TRUCK), Some(&["base"]));
    // The trailer paints no accessory: no Accessories heading for it.
    assert_eq!(h.query_all_by_label("Main textures").count(), 2);
    assert_eq!(h.query_all_by_label("Accessories").count(), 1);
    let top = |name: &str| {
        h.get_by_label(&format!("Texture {TRUCK} › {name}"))
            .rect()
            .top()
    };
    assert!(top("Standard cab") <= top("High roof"));
    assert!(top("High roof") < top("Chassis"), "accessories after");
    assert_eq!(top("Chassis"), top("Side skirts"), "one row of tiles");
}

#[test]
fn clicking_a_texture_shows_it_in_the_workshop() {
    let mut h = open(Some(&["standard"]), Some(&["base", "mudflaps"]));
    let id = ws_mut(&mut h).project.add(Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(500.0, 500.0), Size::new(100.0, 100.0), 0.0),
    ));
    ws_mut(&mut h).selection = vec![id];
    h.run();
    // Adding the trailer made its Base active: its tile is highlighted.
    assert!(highlighted(&h, &format!("{TRAILER} › Base")));
    assert!(!highlighted(&h, &format!("{TRUCK} › Standard cab")));
    h.get_by_label(&format!("Texture {TRAILER} › Mudflaps"))
        .click();
    h.run();
    let ws = ws(&h);
    assert_eq!(ws.space, Space::Workshop);
    assert_eq!(ws.project.surface().name, "Mudflaps");
    assert!(ws.selection.is_empty());
}

#[test]
fn thumbnails_follow_the_artwork() {
    let mut h = open(Some(WHOLE_TRUCK), None);
    let chassis = ws(&h)
        .project
        .surfaces
        .iter()
        .position(|s| s.name == "Chassis")
        .unwrap();
    let ws = ws_mut(&mut h);
    let side = ws.project.surfaces[chassis].size;
    let mut red = Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(
            Point::new(side / 2.0, side / 2.0),
            Size::new(side, side),
            0.0,
        ),
    );
    red.fill = Rgba::rgb(255, 0, 0).into();
    ws.project.active_surface = chassis;
    ws.project.add(red);
    settle(&mut h);
    assert_eq!(
        ws_mut(&mut h).thumbnails.color_at(chassis, [0.5, 0.5]),
        Some(Rgba::rgb(255, 0, 0))
    );
}

#[test]
fn mod_information_of_a_new_project() {
    let h = open(Some(&["standard"]), None);
    // The name is in the top bar and the column.
    assert!(h.query_all_by_label("ACE Logistics").count() >= 2);
    assert!(h.query_by_label("1.0").is_some());
    assert_eq!(h.query_all_by_label("Not set").count(), 2);
    for picture in ["Shop icon", "Mod Manager image"] {
        assert!(
            h.query_by_role_and_label(Role::Image, picture).is_some(),
            "{picture}"
        );
    }
    assert!(ws(&h).mod_previews.textures().is_some(), "both rendered");
    let field = h.get_by_role_and_label(Role::TextInput, "Game versions");
    assert_eq!(field.value().unwrap_or_default(), "");
}

#[test]
fn edit_in_export_mod_opens_the_dialog() {
    let mut h = open(Some(&["standard"]), None);
    h.get_by_label("Edit in Export Mod…").click();
    h.run_steps(3);
    assert!(matches!(h.state().modal, Some(Modal::ExportMod(_))));
}

#[test]
fn edit_in_export_mod_is_disabled_while_editing_a_symbol() {
    let mut h = open(Some(&["standard"]), None);
    let ws = ws_mut(&mut h);
    let id = ws.project.add(Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(500.0, 500.0), Size::new(100.0, 100.0), 0.0),
    ));
    ws.selection = vec![id];
    let symbol = ws.convert_to_symbol(0.0).unwrap();
    ws.edit_symbol(symbol, 0.0);
    ws.space = Space::Project;
    h.run();
    assert!(
        h.get_by_label("Edit in Export Mod…")
            .accesskit_node()
            .is_disabled()
    );
    // No texture is highlighted while a symbol is edited.
    assert!(!highlighted(&h, &format!("{TRUCK} › Standard cab")));
}

#[test]
fn a_chosen_picture_renders_the_previews_again() {
    let mut h = open(Some(&["standard"]), None);
    let before = ws(&h).mod_previews.renders;
    // Unchanged: nothing rendered again.
    settle(&mut h);
    assert_eq!(ws(&h).mod_previews.renders, before);
    let mut png = Vec::new();
    image::RgbaImage::from_pixel(8, 8, image::Rgba([0, 0, 255, 255]))
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .unwrap();
    let ws = ws_mut(&mut h);
    let (asset, _) = ws.project.add_asset(
        "picture",
        tp_core::AssetKind::Raster,
        png.into(),
        Size::new(8.0, 8.0),
    );
    ws.project.mod_settings.image = Some(asset);
    settle(&mut h);
    assert_eq!(ws_mut(&mut h).mod_previews.renders, before + 1);
}
