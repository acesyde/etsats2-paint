//! Headless tests for localization: languages, live switching, numbers,
//! plurals, shortcut notation, names of new objects, and that no screen
//! shows a raw message id.

mod common;

use egui::accesskit::Role;
use egui::{Key, Modifiers, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{By, NodeT, Queryable};
use tp_app::AppState;
use tp_app::prefs::{Prefs, PrefsStore};
use tp_app::state::Modal;
use tp_app::workspace::{ColorTarget, Workspace};
use tp_core::document::{Frame, Object, ObjectId, Rgba, ShapeKind, StrokeStyle};
use tp_core::kurbo::{Point, Size};
use tp_i18n::Language;

type H = Harness<'static, AppState>;

/// Tall window, a project open.
fn open() -> H {
    let prefs = Prefs::default();
    let mut h = common::builder()
        .with_size(Vec2::new(1440.0, 2400.0))
        .with_step_dt(1.0 / 4.0)
        .build_ui_state(
            |ui, state: &mut AppState| state.show(ui),
            AppState::with_prefs(prefs, None),
        );
    common::create_project(&mut h);
    h.run();
    h
}

fn set_language(h: &mut H, language: Language) {
    h.state_mut().prefs.set_language(Some(language));
    h.run();
}

fn ws(h: &H) -> &Workspace {
    h.state().workspace().expect("project open")
}

fn ws_mut(h: &mut H) -> &mut Workspace {
    h.state_mut().workspace_mut().expect("project open")
}

fn add_rect(h: &mut H, name: &str) -> ObjectId {
    let mut o = Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(500.0, 500.0), Size::new(200.0, 100.0), 0.0),
    );
    o.name = name.into();
    let id = ws_mut(h).project.add(o);
    ws_mut(h).selection = vec![id];
    // The new rectangle renders the texture's thumbnail again in the
    // background: let it finish so the next `run` settles.
    common::settle_renders(h);
    id
}

fn field_value(h: &H, name: &str) -> String {
    h.get_by_role_and_label(Role::TextInput, name)
        .value()
        .unwrap_or_default()
}

fn type_into(h: &mut H, name: &str, text: &str) {
    h.get_by_role_and_label(Role::TextInput, name).focus();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.get_by_role_and_label(Role::TextInput, name)
        .type_text(text);
    h.run();
    h.key_press(Key::Enter);
    h.run();
}

/// Whether `text` looks like a message id ("panel-colors", "undo-hide").
fn looks_like_id(text: &str) -> bool {
    text.len() > 3
        && text.contains('-')
        && text.starts_with(|c: char| c.is_ascii_lowercase())
        && text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Every accessible name and label value on screen; panics on raw ids.
fn assert_no_raw_ids(h: &H, screen: &str) {
    let leaks: Vec<String> = h
        .query_all(By::new().include_labels().predicate(|node| {
            [node.label(), node.value()]
                .into_iter()
                .flatten()
                .any(|t| looks_like_id(&t))
        }))
        .map(|n| {
            let node = n.accesskit_node();
            format!("{:?} / {:?}", node.label(), node.value())
        })
        .collect();
    assert!(
        leaks.is_empty(),
        "{screen}: raw message ids shown: {leaks:#?}"
    );
    // The check sees the texts (it would otherwise pass vacuously).
    let labelled = h
        .query_all(By::new().include_labels().predicate(|node| {
            node.label().is_some_and(|l| !l.is_empty())
                || node.value().is_some_and(|v| !v.is_empty())
        }))
        .count();
    assert!(labelled > 10, "{screen}: only {labelled} texts found");
    assert!(looks_like_id("panel-colors") && !looks_like_id("Colors"));
}

#[test]
fn no_screen_shows_a_raw_message_id() {
    for language in Language::ALL {
        let mut h = common::harness();
        h.state_mut().prefs.set_language(Some(language));
        h.run();
        assert_no_raw_ids(&h, "home");
        let mut h = open();
        set_language(&mut h, language);
        let r = add_rect(&mut h, "Box");
        ws_mut(&mut h).map_selected_shapes("undo-add-stroke", |o| {
            o.stroke = Some(StrokeStyle::default());
        });
        ws_mut(&mut h).commit_pending(1.0);
        let _ = r;
        h.run();
        assert_no_raw_ids(&h, "workspace");
        // Menus.
        for title in tp_app::ui::menu_bar::MENUS {
            h.get_by_label(&tp_i18n::tr(title)).click();
            h.run();
            assert_no_raw_ids(&h, title);
            h.key_press(Key::Escape);
            h.run();
        }
        for modal in [
            Modal::Preferences,
            Modal::KeyboardShortcuts,
            Modal::About,
            Modal::NewProject(Default::default()),
        ] {
            h.state_mut().modal = Some(modal);
            h.run();
            assert_no_raw_ids(&h, "dialog");
            h.state_mut().modal = None;
            h.run();
        }
    }
}

/// The message ids written as literals in `tr("…")`, `tr!("…", …)` and
/// `tr_args("…", …)` calls of `source`.
fn literal_ids(source: &str) -> Vec<String> {
    let mut ids = Vec::new();
    for call in ["tr(", "tr!(", "tr_args("] {
        for (at, _) in source.match_indices(call) {
            let before = source[..at].chars().next_back();
            if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                continue;
            }
            let rest = source[at + call.len()..].trim_start();
            if let Some(rest) = rest.strip_prefix('"')
                && let Some(end) = rest.find('"')
            {
                ids.push(rest[..end].to_owned());
            }
        }
    }
    ids
}

fn rust_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Every message id the code asks for by name exists (in English; the
/// tp-i18n tests check that every language has every English message).
#[test]
fn every_message_id_in_the_code_exists() {
    let crates = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    for krate in ["tp-app", "tp-ui"] {
        rust_files(&crates.join(krate).join("src"), &mut files);
    }
    let mut checked = 0;
    let mut missing = Vec::new();
    for file in files {
        let source = std::fs::read_to_string(&file).unwrap();
        for id in literal_ids(&source) {
            checked += 1;
            if !tp_i18n::exists(&id) {
                missing.push(format!("{id} ({})", file.display()));
            }
        }
    }
    assert!(checked > 300, "only {checked} ids found");
    assert!(missing.is_empty(), "unknown message ids: {missing:#?}");
}

#[test]
fn german_menu_bar() {
    let mut h = open();
    set_language(&mut h, Language::German);
    for title in [
        "Datei",
        "Bearbeiten",
        "Objekt",
        "Ebene",
        "Ansicht",
        "Fahrzeug",
        "Exportieren",
        "Hilfe",
    ] {
        assert!(h.query_by_label(title).is_some(), "{title}");
    }
}

#[test]
fn user_names_stay_as_typed() {
    let mut h = open();
    let id = add_rect(&mut h, "Stripes");
    set_language(&mut h, Language::German);
    assert!(h.query_by_label("Ebenen").is_some(), "Layers tab");
    assert_eq!(ws(&h).project.surface().get(id).unwrap().name, "Stripes");
    assert!(h.query_all_by_label_contains("Stripes").count() > 0);
}

#[test]
fn decimal_comma_display_and_point_input() {
    let mut h = open();
    add_rect(&mut h, "Box");
    ws_mut(&mut h).map_selected_shapes("undo-add-stroke", |o| {
        o.stroke = Some(StrokeStyle {
            width: 12.5,
            ..StrokeStyle::default()
        });
    });
    ws_mut(&mut h).commit_pending(1.0);
    h.run();
    common::open_stroke_popover(&mut h);
    set_language(&mut h, Language::French);
    // The Stroke row's summary too.
    let summary = h
        .get_by_label(&tp_i18n::tr("stroke-options"))
        .value()
        .unwrap_or_default();
    assert!(summary.starts_with("12,5 px"), "{summary}");
    let name = tp_i18n::tr("stroke-width");
    assert_eq!(field_value(&h, &name), "12,5");
    set_language(&mut h, Language::German);
    let name = tp_i18n::tr("stroke-width");
    type_into(&mut h, &name, "20.5");
    let o = ws(&h).selected_shapes()[0].clone();
    assert_eq!(o.stroke.unwrap().width, 20.5);
    assert_eq!(field_value(&h, &name), "20,5");
    type_into(&mut h, &name, "7,5");
    let o = ws(&h).selected_shapes()[0].clone();
    assert_eq!(o.stroke.unwrap().width, 7.5, "a comma works too");
}

#[test]
fn relative_times_use_each_languages_plurals() {
    let day = 86_400;
    tp_i18n::set_language(Language::Spanish);
    assert_eq!(
        tp_app::ui::home::relative_time(10 * day, 8 * day),
        "hace 2 días"
    );
    tp_i18n::set_language(Language::French);
    assert_eq!(
        tp_app::ui::home::relative_time(10 * day, 10 * day - 3_600),
        "il y a 1 heure"
    );
    tp_i18n::set_language(Language::English);
    assert_eq!(
        tp_app::ui::home::relative_time(10 * day, 5 * day),
        "5 days ago"
    );
}

#[test]
fn shortcut_notation_follows_the_language_on_windows() {
    let redo = egui::KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::Z);
    tp_i18n::set_language(Language::German);
    assert_eq!(
        tp_app::commands::format_shortcut(&redo, false, false),
        "Strg+Umschalt+Z"
    );
    tp_i18n::set_language(Language::French);
    assert_eq!(
        tp_app::commands::format_shortcut(&redo, false, false),
        "Ctrl+Maj+Z"
    );
    // macOS symbols do not change.
    assert_eq!(tp_app::commands::format_shortcut(&redo, true, true), "⇧⌘Z");
    tp_i18n::set_language(Language::English);
}

#[test]
fn new_layer_is_named_in_french_and_existing_names_stay() {
    let mut h = open();
    let rect = {
        ws_mut(&mut h).create_shape(
            ShapeKind::rectangle(),
            Frame::new(Point::new(300.0, 300.0), Size::new(50.0, 50.0), 0.0),
            1.0,
        )
    };
    assert_eq!(
        ws(&h).project.surface().get(rect).unwrap().name,
        "Rectangle"
    );
    set_language(&mut h, Language::French);
    ws_mut(&mut h).new_layer(2.0);
    let layer = ws(&h).selection[0];
    assert_eq!(
        ws(&h).project.surface().get(layer).unwrap().name,
        "Calque 1"
    );
    ws_mut(&mut h).new_layer(3.0);
    let layer = ws(&h).selection[0];
    assert_eq!(
        ws(&h).project.surface().get(layer).unwrap().name,
        "Calque 2"
    );
    set_language(&mut h, Language::German);
    assert_eq!(
        ws(&h).project.surface().get(rect).unwrap().name,
        "Rectangle"
    );
    let new = ws_mut(&mut h).create_shape(
        ShapeKind::rectangle(),
        Frame::new(Point::new(300.0, 300.0), Size::new(50.0, 50.0), 0.0),
        4.0,
    );
    assert_eq!(ws(&h).project.surface().get(new).unwrap().name, "Rechteck");
}

#[test]
fn live_switch_keeps_the_document() {
    let mut h = open();
    add_rect(&mut h, "Box");
    ws_mut(&mut h).apply_color(ColorTarget::Fill, Rgba::rgb(10, 20, 30));
    ws_mut(&mut h).commit_pending(1.0);
    let before = ws(&h).project.surface().objects.clone();
    let history = ws(&h).history.len();
    h.state_mut().modal = Some(Modal::Preferences);
    h.run();
    h.get_by_role_and_label(Role::ComboBox, "Language").click();
    h.run();
    h.get_by_label("Español").click();
    h.run();
    assert_eq!(h.state().language(), Language::Spanish);
    // The open dialog, the menu bar and the tabs are in Spanish.
    assert!(
        h.query_all_by_label("Idioma").count() > 0,
        "Preferences dialog"
    );
    assert!(h.query_by_label("Archivo").is_some(), "menu bar");
    assert!(h.query_by_label("Capas").is_some(), "Layers tab");
    assert_eq!(ws(&h).project.surface().objects, before);
    assert_eq!(ws(&h).history.len(), history);
    assert_eq!(ws(&h).save_state(), tp_app::workspace::SaveState::Unsaved);
}

#[test]
fn undo_label_follows_the_language() {
    let mut h = open();
    add_rect(&mut h, "Box");
    ws_mut(&mut h).apply_color(ColorTarget::Fill, Rgba::rgb(10, 20, 30));
    ws_mut(&mut h).commit_pending(1.0);
    h.get_by_label("Edit").click();
    h.run();
    assert!(h.query_by_label("Undo Change Fill").is_some());
    h.get_by_label("Edit").click();
    h.run();
    set_language(&mut h, Language::French);
    h.get_by_label("Édition").click();
    h.run();
    let labels: Vec<String> = h
        .query_all(By::new().label_contains("Annuler"))
        .filter_map(|n| n.accesskit_node().label())
        .collect();
    assert!(
        h.query_by_label("Annuler Modifier le fond").is_some(),
        "{labels:?}"
    );
}

#[test]
fn back_to_the_system_language() {
    let mut h = open();
    h.state_mut().system_language = Language::French;
    set_language(&mut h, Language::Spanish);
    h.state_mut().modal = Some(Modal::Preferences);
    h.run();
    h.get_by_role_and_label(Role::ComboBox, "Idioma").click();
    h.run();
    h.get_by_label("Idioma del sistema").click();
    h.run();
    assert_eq!(h.state().prefs.language, None, "no choice saved");
    assert_eq!(h.state().language(), Language::French);
    assert!(h.query_by_label("Fichier").is_some());
}

#[test]
fn language_survives_restart() {
    let dir = tempfile::tempdir().unwrap();
    let store = || Some(PrefsStore::new(dir.path()));
    let mut state = AppState::new(store());
    state.prefs.set_language(Some(Language::German));
    state.persist_prefs(f64::MAX, true);
    let mut reopened = AppState::new(store());
    // Whatever the system language.
    reopened.system_language = Language::French;
    assert_eq!(reopened.language(), Language::German);
}

#[test]
fn reset_to_defaults_keeps_the_language() {
    let mut prefs = Prefs::default();
    prefs.set_language(Some(Language::Spanish));
    prefs.ui_scale = 1.5;
    prefs.reset_scaling();
    assert_eq!(prefs.language(), Some(Language::Spanish));
}
