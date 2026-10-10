//! Headless tests for the personal library: Add to Library, Import from
//! Library…, and paste across projects.

mod common;

use egui::Vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::prefs::Prefs;
use tp_app::workspace::Workspace;
use tp_core::Project;
use tp_core::document::{
    CharStyle, Frame, Object, ObjectId, Paint, Rgba, ShapeKind, SwatchId, SymbolId, TextBlock,
};
use tp_core::kurbo::{Point, Size};

type H = Harness<'static, AppState>;

const VERT: Rgba = Rgba::rgb(0x1E, 0x8C, 0x3A);

/// Tall window with the Resources tab shown; a project for the sample
/// truck.
fn open() -> H {
    let mut prefs = Prefs::default();
    prefs.layout.left_tab = tp_app::layout::LeftTab::Resources;
    let mut h = common::builder()
        .with_size(Vec2::new(1440.0, 2400.0))
        .build_ui_state(
            |ui, state: &mut AppState| state.show(ui),
            AppState::with_prefs(prefs, None),
        );
    common::create_project(&mut h);
    h.run();
    h
}

fn ws(h: &H) -> &Workspace {
    h.state().workspace().expect("project open")
}

fn ws_mut(h: &mut H) -> &mut Workspace {
    h.state_mut().workspace_mut().expect("project open")
}

fn library(h: &mut H) -> Project {
    h.state_mut().library.library().clone()
}

fn names<T>(list: &[T], name: impl Fn(&T) -> &str) -> Vec<String> {
    list.iter().map(|x| name(x).to_owned()).collect()
}

struct Ardent {
    logo: SymbolId,
    vert: SwatchId,
}

/// The symbol "Logo Ardent": a stripe linked to the swatch "Vert Ardent"
/// and the text "ARDENT" following the text style "Titre". The project is
/// then marked saved.
fn ardent(h: &mut H) -> Ardent {
    let ws = ws_mut(h);
    let p = &mut ws.project;
    let (vert, _) = p.add_swatch(VERT, "Color");
    p.rename_swatch(vert, "Vert Ardent");
    let mut stripe = Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(600.0, 800.0), Size::new(400.0, 60.0), 0.0),
    );
    stripe.fill = Paint::Solid(VERT);
    stripe.fill_swatch = Some(vert);
    let stripe = p.add(stripe);
    let mut lettering = Object::new(
        ObjectId(0),
        ShapeKind::Text,
        Frame::new(Point::new(600.0, 700.0), Size::new(300.0, 80.0), 0.0),
    );
    lettering.text = Some(TextBlock::new("ARDENT", CharStyle::default()));
    let lettering = p.add(lettering);
    let titre = p.new_text_style(lettering, "Text style").unwrap();
    p.rename_style(titre, "Titre");
    let (logo, _) = p.convert_to_symbol(&[stripe, lettering], "Symbol").unwrap();
    p.rename_symbol(logo, "Logo Ardent");
    ws.relayout_all_texts();
    ws.selection.clear();
    ws.saved = Some(ws.snapshot());
    h.run();
    Ardent { logo, vert }
}

/// Opens the context menu of `row` (the first one listed: a swatch is in
/// the Resources tab and the inspector) and clicks `item`.
fn context_menu(h: &mut H, row: &str, item: &str) {
    h.get_all_by_label(row).next().expect(row).click_secondary();
    h.run();
    h.get_by_label(item).click();
    h.run();
}

// ── Add to Library ──────────────────────────────────────────────────────

#[test]
fn adding_a_symbol_brings_its_dependencies() {
    let mut h = open();
    let ids = ardent(&mut h);
    let steps = ws(&h).history.len();
    context_menu(&mut h, "Symbol Logo Ardent", "Add to Library");
    let l = library(&mut h);
    assert_eq!(names(&l.symbols, |s| &s.name), ["Logo Ardent"]);
    assert_eq!(names(&l.palette, |s| &s.name), ["Vert Ardent"]);
    assert_eq!(names(&l.text_styles, |s| &s.name), ["Titre"]);
    assert_eq!(
        ws(&h).hint.as_ref().map(|h| h.text.as_str()),
        Some("Added to the library")
    );
    // The links are saved with the project, but are no undo step.
    assert!(h.query_by_label("Unsaved changes").is_some());
    assert_eq!(ws(&h).history.len(), steps);
    let project = &ws(&h).project;
    assert!(project.symbol(ids.logo).unwrap().origin.is_some());
    assert!(project.swatch(ids.vert).unwrap().origin.is_some());
}

#[test]
fn updating_a_library_symbol_replaces_its_copy() {
    let mut h = open();
    let ids = ardent(&mut h);
    context_menu(&mut h, "Symbol Logo Ardent", "Add to Library");
    let entry = library(&mut h).symbols[0].id;
    ws_mut(&mut h).rename_symbol(ids.logo, "Logo Ardent 2026", 1.0);
    h.run();
    context_menu(&mut h, "Symbol Logo Ardent 2026", "Update in Library");
    let l = library(&mut h);
    assert_eq!(names(&l.symbols, |s| &s.name), ["Logo Ardent 2026"]);
    assert_eq!(l.symbols[0].id, entry, "the same entry, replaced");
    assert_eq!(l.palette.len(), 1);
    assert_eq!(
        ws(&h).hint.as_ref().map(|h| h.text.as_str()),
        Some("Library updated")
    );
}

#[test]
fn undo_keeps_the_library_links() {
    let mut h = open();
    let ids = ardent(&mut h);
    let mut circle = ws(&h).styled_shape(
        ShapeKind::Ellipse,
        Frame::new(Point::new(2000.0, 2000.0), Size::new(100.0, 100.0), 0.0),
    );
    circle.name = "Circle".into();
    ws_mut(&mut h).create_object(circle, 1.0);
    h.run();
    context_menu(&mut h, "Vert Ardent", "Add to Library");
    let key = ws(&h).project.swatch(ids.vert).unwrap().origin.clone();
    assert!(key.is_some());
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(
        ws(&h).project.surface().objects.len(),
        1,
        "the circle is gone"
    );
    assert_eq!(ws(&h).project.swatch(ids.vert).unwrap().origin, key);
    // The swatch's menu still offers to update it.
    h.get_all_by_label("Vert Ardent")
        .next()
        .unwrap()
        .click_secondary();
    h.run();
    assert!(h.query_by_label("Update in Library").is_some());
}

// ── Import from Library ─────────────────────────────────────────────────

fn run_command(h: &mut H, id: tp_app::commands::CommandId) {
    h.state_mut().queue.push(id);
    h.run_steps(3);
    h.run();
}

fn import_dialog(h: &mut H) {
    run_command(h, tp_app::commands::CommandId::ImportFromLibrary);
    assert!(
        matches!(
            h.state().modal,
            Some(tp_app::state::Modal::ImportFromLibrary(_))
        ),
        "the dialog is open"
    );
}

fn dialog_open(h: &H) -> bool {
    matches!(
        h.state().modal,
        Some(tp_app::state::Modal::ImportFromLibrary(_))
    )
}

/// "Logo Ardent" added to the library, then a new project.
fn library_with_ardent() -> H {
    let mut h = open();
    ardent(&mut h);
    context_menu(&mut h, "Symbol Logo Ardent", "Add to Library");
    common::create_project(&mut h);
    h
}

#[test]
fn importing_a_symbol_brings_its_dependencies_in_one_step() {
    let mut h = library_with_ardent();
    import_dialog(&mut h);
    assert!(h.query_by_label("Swatches").is_some(), "grouped");
    for name in ["Logo Ardent", "Vert Ardent", "Titre"] {
        let listed = h.query_by_role_and_label(egui::accesskit::Role::CheckBox, name);
        assert!(listed.is_some(), "{name} is listed");
    }
    let import = h.get_by_label("Import");
    assert!(import.accesskit_node().is_disabled(), "nothing checked");
    h.get_by_label("Logo Ardent").click();
    h.run();
    h.get_by_label("Import").click();
    h.run();
    assert!(!dialog_open(&h));
    let p = &ws(&h).project;
    assert_eq!(names(&p.symbols, |s| &s.name), ["Logo Ardent"]);
    assert_eq!(names(&p.palette, |s| &s.name), ["Vert Ardent"]);
    assert_eq!(names(&p.text_styles, |s| &s.name), ["Titre"]);
    assert_eq!(ws(&h).history.len(), 1);
    ws_mut(&mut h).undo();
    h.run();
    let p = &ws(&h).project;
    assert!(p.symbols.is_empty() && p.palette.is_empty() && p.text_styles.is_empty());
}

#[test]
fn elements_already_in_the_project_cant_be_checked() {
    let mut h = library_with_ardent();
    import_dialog(&mut h);
    h.get_by_label("Logo Ardent").click();
    h.run();
    h.get_by_label("Import").click();
    h.run();
    import_dialog(&mut h);
    assert_eq!(h.query_all_by_label("In this project").count(), 3);
    let logo = h.get_by_role_and_label(egui::accesskit::Role::CheckBox, "Logo Ardent");
    assert!(logo.accesskit_node().is_disabled());
    // Saved, closed and opened again, the project still knows them.
    h.key_press(egui::Key::Escape);
    h.run();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("b.truckpaint");
    tp_file::write(&ws(&h).project, &path).unwrap();
    let opened = tp_file::read(&path).unwrap().project;
    h.state_mut().open_project(opened);
    h.run();
    import_dialog(&mut h);
    assert_eq!(h.query_all_by_label("In this project").count(), 3);
}

#[test]
fn removing_from_the_library_keeps_the_projects_copy() {
    let mut h = library_with_ardent();
    import_dialog(&mut h);
    h.get_by_label("Logo Ardent").click();
    h.run();
    h.get_by_label("Import").click();
    h.run();
    import_dialog(&mut h);
    h.get_by_role_and_label(egui::accesskit::Role::CheckBox, "Logo Ardent")
        .click_secondary();
    h.run();
    h.get_by_label("Remove from Library").click();
    h.run();
    assert!(
        h.query_by_label(
            "Remove Logo Ardent from the library? Projects that imported it keep their copy."
        )
        .is_some()
    );
    h.get_by_label("Remove").click();
    h.run();
    let listed = h.query_by_role_and_label(egui::accesskit::Role::CheckBox, "Logo Ardent");
    assert!(listed.is_none());
    assert!(library(&mut h).symbols.is_empty());
    assert_eq!(names(&ws(&h).project.symbols, |s| &s.name), ["Logo Ardent"]);
}

#[test]
fn an_empty_library_explains_add_to_library() {
    let mut h = open();
    import_dialog(&mut h);
    assert!(h.query_by_label(
        "Your library is empty. Right-click a symbol, a swatch or a style in the Brand space or the Resources tab and choose Add to Library to use it in every project."
    )
    .is_some());
    assert!(h.query_by_label("Swatches").is_none());
}

#[test]
fn an_unreadable_library_is_set_aside() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(tp_file::library::FILE_NAME);
    std::fs::write(&path, b"PK\x03\x04 damaged").unwrap();
    let mut h = open();
    h.state_mut().library = tp_app::library::LibraryStore::new(Some(path.clone()));
    import_dialog(&mut h);
    let message = h
        .query_all_by_label_contains("The library could not be read: it starts empty.")
        .count();
    assert_eq!(message, 1);
    assert!(!path.exists(), "the damaged file was moved aside");
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1, "backup");
    // Said once.
    h.key_press(egui::Key::Escape);
    h.run();
    import_dialog(&mut h);
    assert_eq!(
        h.query_all_by_label_contains("could not be read").count(),
        0
    );
}

#[test]
fn the_library_survives_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(tp_file::library::FILE_NAME);
    let mut h = open();
    h.state_mut().library = tp_app::library::LibraryStore::new(Some(path.clone()));
    ardent(&mut h);
    context_menu(&mut h, "Vert Ardent", "Add to Library");
    h.state_mut().shutdown();
    assert!(path.exists());
    let mut again = open();
    again.state_mut().library = tp_app::library::LibraryStore::new(Some(path));
    import_dialog(&mut again);
    assert!(again.query_by_label("Vert Ardent").is_some());
}

#[test]
fn the_resources_tab_offers_import_from_library() {
    let mut h = open();
    let import = || "Import from Library…";
    assert_eq!(
        h.query_all_by_role_and_label(egui::accesskit::Role::Button, import())
            .count(),
        1,
        "one button, in the Resources tab"
    );
    h.get_by_role_and_label(egui::accesskit::Role::Button, import())
        .click();
    h.run();
    assert!(dialog_open(&h));
}

#[test]
fn import_from_library_is_disabled_while_editing_a_symbol() {
    let mut h = open();
    ardent(&mut h);
    let logo = ws(&h).project.symbols[0].id;
    ws_mut(&mut h).edit_symbol(logo, 0.0);
    h.run();
    let button = h.get_by_role_and_label(egui::accesskit::Role::Button, "Import from Library…");
    assert!(button.accesskit_node().is_disabled());
    button.hover();
    h.run();
    assert!(
        h.query_by_label_contains(&tp_i18n::tr("reason-editing-symbol"))
            .is_some(),
        "the tooltip says why"
    );
}

// ── Paste into another project ──────────────────────────────────────────

const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="40" height="20" fill="#1E8C3A"/></svg>"##;

/// An image and the text "ARDENT" linked to the swatch "Vert Ardent",
/// selected; returns the swatch.
fn logo_objects(h: &mut H) -> SwatchId {
    let ws = ws_mut(h);
    let p = &mut ws.project;
    let (vert, _) = p.add_swatch(VERT, "Color");
    p.rename_swatch(vert, "Vert Ardent");
    let (asset, _) = p.add_asset(
        "logo",
        tp_core::AssetKind::Svg,
        std::sync::Arc::from(SVG),
        Size::new(40.0, 20.0),
    );
    let picture = p.add(Object::new(
        ObjectId(0),
        ShapeKind::Image { asset },
        Frame::new(Point::new(600.0, 600.0), Size::new(400.0, 200.0), 0.0),
    ));
    let mut lettering = Object::new(
        ObjectId(0),
        ShapeKind::Text,
        Frame::new(Point::new(600.0, 900.0), Size::new(300.0, 80.0), 0.0),
    );
    lettering.text = Some(TextBlock::new("ARDENT", CharStyle::default()));
    lettering.fill = Paint::Solid(VERT);
    lettering.fill_swatch = Some(vert);
    let lettering = p.add(lettering);
    ws.relayout_all_texts();
    ws.selection = vec![picture, lettering];
    h.run();
    vert
}

fn copy_and_paste_in_a_new_project(h: &mut H) {
    run_command(h, tp_app::commands::CommandId::Copy);
    common::create_project(h);
    run_command(h, tp_app::commands::CommandId::Paste);
}

#[test]
fn pasting_a_logo_into_another_project() {
    let mut h = open();
    logo_objects(&mut h);
    copy_and_paste_in_a_new_project(&mut h);
    let p = &ws(&h).project;
    assert_eq!(names(&p.palette, |s| &s.name), ["Vert Ardent"]);
    let vert = p.palette[0].id;
    let pasted = ws(&h).selected_objects();
    assert_eq!(pasted.len(), 2);
    let ShapeKind::Image { asset } = pasted[0].kind else {
        panic!("an image");
    };
    assert_eq!(&*p.assets[&asset].bytes, SVG);
    assert_eq!(pasted[1].fill_swatch, Some(vert), "still linked");
    assert_eq!(pasted[1].fill, Paint::Solid(VERT));
}

#[test]
fn pasting_an_instance_after_closing_its_project() {
    let mut h = open();
    let ids = ardent(&mut h);
    let instance = ws(&h).project.surface().objects[0].id;
    ws_mut(&mut h).selection = vec![instance];
    h.run();
    run_command(&mut h, tp_app::commands::CommandId::Copy);
    h.state_mut().close_project();
    h.run();
    common::create_project(&mut h);
    run_command(&mut h, tp_app::commands::CommandId::Paste);
    let p = &ws(&h).project;
    assert_eq!(names(&p.symbols, |s| &s.name), ["Logo Ardent"]);
    let pasted = ws(&h).selected_objects();
    assert!(
        matches!(pasted[0].kind, ShapeKind::Instance { symbol, .. } if symbol == p.symbols[0].id)
    );
    assert_ne!(p.symbols[0].id, ids.logo, "a new id in this project");
}

#[test]
fn one_undo_removes_a_paste_and_what_came_with_it() {
    let mut h = open();
    logo_objects(&mut h);
    copy_and_paste_in_a_new_project(&mut h);
    assert_eq!(ws(&h).history.len(), 1);
    assert_eq!(ws(&h).history.undo_label(), Some("cmd-paste"));
    ws_mut(&mut h).undo();
    h.run();
    let p = &ws(&h).project;
    assert!(p.surface().objects.is_empty());
    assert!(p.palette.is_empty());
    assert!(p.assets.values().all(|a| &*a.bytes != SVG));
}

#[test]
fn pasting_into_the_same_project_opened_again_adds_no_swatch() {
    let mut h = open();
    logo_objects(&mut h);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a.truckpaint");
    tp_file::write(&ws(&h).project, &path).unwrap();
    run_command(&mut h, tp_app::commands::CommandId::Copy);
    h.state_mut().close_project();
    let opened = tp_file::read(&path).unwrap().project;
    h.state_mut().open_project(opened);
    h.run();
    run_command(&mut h, tp_app::commands::CommandId::Paste);
    let p = &ws(&h).project;
    assert_eq!(names(&p.palette, |s| &s.name), ["Vert Ardent"]);
    assert_eq!(p.surface().objects.len(), 4, "two objects, twice");
    let assets = p.assets.values().filter(|a| &*a.bytes == SVG).count();
    assert_eq!(assets, 1);
}
