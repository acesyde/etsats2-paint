//! Headless tests for the Project space: the Vehicles header (counts and
//! texture filter), the vehicle cards and their textures with their
//! states, and the Mod information column with Before exporting.

mod common;

use egui::Vec2;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::layout::Space;
use tp_app::state::Modal;
use tp_app::workspace::Workspace;
use tp_core::TemplateStatus;
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
    h.get_by_label_contains(&format!("Texture {path},"))
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

fn surface(h: &H, name: &str) -> usize {
    ws(h)
        .project
        .surfaces
        .iter()
        .position(|s| s.name == name)
        .unwrap()
}

/// Draws a rectangle on texture `name` (it becomes Modified), as the
/// Workshop would, and runs a frame.
fn draw(h: &mut H, name: &str) {
    let i = surface(h, name);
    let ws = ws_mut(h);
    let active = ws.project.active_surface;
    ws.project.active_surface = i;
    ws.project.add(Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(200.0, 200.0), Size::new(100.0, 100.0), 0.0),
    ));
    ws.project.active_surface = active;
    h.run();
}

/// Flags texture `name` as an update would.
fn flag(h: &mut H, name: &str, status: TemplateStatus) {
    let i = surface(h, name);
    ws_mut(h).project.surfaces[i]
        .template
        .as_mut()
        .unwrap()
        .status = status;
    h.run();
}

/// The fleet of the Before exporting scenarios: the truck's accessories
/// and the trailer's Mudflaps Modified, Curtain body 13.6 m flagged
/// "Layout changed", the rest Empty.
fn fleet_with_work_left() -> H {
    let mut h = open(Some(WHOLE_TRUCK), Some(WHOLE_TRAILER));
    for name in ["Chassis", "Cab accessories", "Side skirts", "Mudflaps"] {
        draw(&mut h, name);
    }
    flag(&mut h, "Curtain body 13.6 m", TemplateStatus::LayoutChanged);
    h
}

fn tile(h: &H, path: &str) -> Option<String> {
    h.query_by_label_contains(&format!("Texture {path},"))
        .and_then(|n| n.accesskit_node().label())
}

fn choose(h: &mut H, filter: &str) {
    h.get_by_role_and_label(Role::RadioButton, filter).click();
    h.run();
}

#[test]
fn header_counts_the_vehicles_textures_and_states() {
    let h = fleet_with_work_left();
    assert!(h.query_by_label("Vehicles").is_some());
    assert!(
        h.query_by_label("2 vehicles · 9 textures · 4 modified · 1 to check")
            .is_some()
    );
    for filter in ["All", "To do", "To check"] {
        assert!(
            h.query_by_role_and_label(Role::RadioButton, filter)
                .is_some(),
            "{filter}"
        );
    }
    assert!(h.query_by_label("Add Vehicle…").is_some());
    // All is chosen when a project opens.
    assert_eq!(ws(&h).texture_filter, tp_core::TextureFilter::All);
}

#[test]
fn counts_follow_the_work() {
    let mut h = open(
        Some(&[
            "standard",
            "chassis",
            "cab_accessories",
            "side_skirts",
            "high_roof",
        ]),
        None,
    );
    assert!(h.query_by_label("1 vehicle · 5 textures").is_some());
    draw(&mut h, "Chassis");
    assert!(
        h.query_by_label("1 vehicle · 5 textures · 1 modified")
            .is_some()
    );
}

#[test]
fn rows_of_a_truck_are_empty() {
    let h = open(Some(WHOLE_TRUCK), None);
    for name in [
        "Standard cab",
        "High roof",
        "Chassis",
        "Cab accessories",
        "Side skirts",
    ] {
        assert_eq!(
            tile(&h, &format!("{TRUCK} › {name}")).as_deref(),
            Some(format!("Texture {TRUCK} › {name}, Empty").as_str())
        );
    }
}

#[test]
fn flagged_texture_reads_to_check() {
    let mut h = open(Some(WHOLE_TRUCK), None);
    flag(&mut h, "Standard cab", TemplateStatus::LayoutChanged);
    assert_eq!(
        tile(&h, &format!("{TRUCK} › Standard cab")).as_deref(),
        Some(format!("Texture {TRUCK} › Standard cab, To check").as_str())
    );
    // Hovering the state says why.
    assert!(h.query_by_label("Layout changed in 1.1.0").is_some());
    flag(&mut h, "High roof", TemplateStatus::Removed);
    assert!(h.query_by_label("Not in this version").is_some());
}

#[test]
fn only_what_is_left_to_do() {
    let mut h = open(Some(WHOLE_TRUCK), None);
    for name in ["Chassis", "Cab accessories", "Side skirts"] {
        draw(&mut h, name);
    }
    let counts = "1 vehicle · 5 textures · 3 modified";
    assert!(h.query_by_label(counts).is_some());
    choose(&mut h, "To do");
    assert!(tile(&h, &format!("{TRUCK} › Standard cab")).is_some());
    assert!(tile(&h, &format!("{TRUCK} › High roof")).is_some());
    assert!(tile(&h, &format!("{TRUCK} › Chassis")).is_none());
    assert_eq!(h.query_all_by_label("Main textures").count(), 1);
    assert_eq!(h.query_all_by_label("Accessories").count(), 0);
    // The filter doesn't change the counts.
    assert!(h.query_by_label(counts).is_some());
}

#[test]
fn nothing_to_do_on_a_vehicle() {
    let mut h = open(Some(WHOLE_TRUCK), Some(WHOLE_TRAILER));
    for name in [
        "Base",
        "Curtain body 13.6 m",
        "Curtain body 10.5 m",
        "Mudflaps",
    ] {
        draw(&mut h, name);
    }
    flag(&mut h, "Standard cab", TemplateStatus::LayoutChanged);
    let counts = "2 vehicles · 9 textures · 4 modified · 1 to check";
    assert!(h.query_by_label(counts).is_some());
    choose(&mut h, "To check");
    // The trailer keeps its card, saying there is nothing to do.
    assert!(
        h.query_by_label(&format!("Actions for {TRAILER}"))
            .is_some()
    );
    assert_eq!(h.query_all_by_label("Nothing to do").count(), 1);
    assert!(tile(&h, &format!("{TRAILER} › Base")).is_none());
    assert!(tile(&h, &format!("{TRUCK} › Standard cab")).is_some());
    assert!(tile(&h, &format!("{TRUCK} › High roof")).is_none());
    assert!(h.query_by_label(counts).is_some());
}

#[test]
fn filter_kept_across_spaces_and_mark_as_checked() {
    let mut h = open(Some(WHOLE_TRUCK), None);
    flag(&mut h, "Chassis", TemplateStatus::LayoutChanged);
    choose(&mut h, "To check");
    // Opening a To check texture and drawing on it doesn't clear it.
    h.get_by_label_contains(&format!("Texture {TRUCK} › Chassis,"))
        .click();
    h.run();
    assert_eq!(ws(&h).space, Space::Workshop);
    assert_eq!(ws(&h).project.surface().name, "Chassis");
    draw(&mut h, "Chassis");
    ws_mut(&mut h).space = Space::Project;
    h.run();
    assert_eq!(ws(&h).texture_filter, tp_core::TextureFilter::ToCheck);
    assert_eq!(
        tile(&h, &format!("{TRUCK} › Chassis")).as_deref(),
        Some(format!("Texture {TRUCK} › Chassis, To check").as_str())
    );
    // Marked as checked in the inspector: it leaves the list at once.
    ws_mut(&mut h).space = Space::Workshop;
    h.run();
    h.get_by_label(&format!("Mark {TRUCK} › Chassis as checked"))
        .click();
    h.run();
    ws_mut(&mut h).space = Space::Project;
    h.run();
    assert!(tile(&h, &format!("{TRUCK} › Chassis")).is_none());
    assert!(h.query_by_label("Nothing to do").is_some());
}

#[test]
fn before_exporting_lists_the_warnings() {
    let h = fleet_with_work_left();
    assert!(h.query_by_label("Before exporting").is_some());
    let check = h.get_by_label("Curtain body 13.6 m: layout changed").rect();
    let empty = h
        .get_by_label("4 textures empty, exported with the game's color")
        .rect();
    assert!(check.top() < empty.top());
    assert!(h.query_by_label("Nothing to check").is_none());
    // Collapsed: no Open for the empty textures yet.
    assert!(h.query_by_label("Open High roof").is_none());
}

#[test]
fn opening_a_texture_to_check() {
    let mut h = fleet_with_work_left();
    h.get_by_label("Open Curtain body 13.6 m").click();
    h.run();
    assert_eq!(ws(&h).space, Space::Workshop);
    assert_eq!(ws(&h).project.surface().name, "Curtain body 13.6 m");
}

#[test]
fn opening_an_empty_texture() {
    let mut h = fleet_with_work_left();
    h.get_by_label("4 textures empty, exported with the game's color")
        .click();
    h.run();
    for name in ["Standard cab", "High roof", "Base", "Curtain body 10.5 m"] {
        assert!(
            h.query_by_label(&format!("Open {name}")).is_some(),
            "{name}"
        );
    }
    h.get_by_label("Open High roof").click();
    h.run();
    assert_eq!(ws(&h).space, Space::Workshop);
    assert_eq!(ws(&h).project.surface().name, "High roof");
}

#[test]
fn nothing_to_check_when_every_texture_is_modified() {
    let mut h = open(None, Some(&["base", "mudflaps"]));
    draw(&mut h, "Base");
    assert!(
        h.query_by_label("Mudflaps is empty, exported with the game's color")
            .is_some()
    );
    assert!(h.query_by_label("Open Mudflaps").is_some());
    draw(&mut h, "Mudflaps");
    assert!(h.query_by_label("Nothing to check").is_some());
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
        h.get_by_label_contains(&format!("Texture {TRUCK} › {name},"))
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
    h.get_by_label_contains(&format!("Texture {TRAILER} › Mudflaps,"))
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
    assert_eq!(
        tile(&h, &format!("{TRUCK} › Chassis")).as_deref(),
        Some(format!("Texture {TRUCK} › Chassis, Modified").as_str())
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
    // Before exporting: its only texture is empty.
    assert!(h.query_by_label("Before exporting").is_some());
    assert!(
        h.query_by_label("Standard cab is empty, exported with the game's color")
            .is_some()
    );
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
