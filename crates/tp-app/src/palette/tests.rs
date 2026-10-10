use tp_core::document::FlipAxis;
use tp_i18n::{Language, set_language, tr};

use super::*;
use crate::menus::{catalog, menus};

/// Every command of the palette in `language`, enabled, in menu order.
fn commands(language: Language) -> Vec<Entry> {
    set_language(language);
    catalog()
        .into_iter()
        .map(|c| Entry {
            target: Target::Command(c.id),
            label: tr(c.id.meta().label),
            context: context(&c.path),
            enabled: true,
        })
        .collect()
}

fn entry(label: &str, context: &str) -> Entry {
    Entry {
        target: Target::Texture(0),
        label: label.to_owned(),
        context: context.to_owned(),
        enabled: true,
    }
}

fn labels(entries: &[Entry], query: &str) -> Vec<String> {
    rank(entries, query)
        .into_iter()
        .map(|(i, _)| entries[i].label.clone())
        .collect()
}

fn score_of(query: &str, label: &str, context: &str) -> i32 {
    score(&words(query), label, context).unwrap().score
}

#[test]
fn folding_ignores_case_and_accents() {
    let s = |t: &str| fold(t).chars.into_iter().collect::<String>();
    assert_eq!(s("Tout sélectionner"), "tout selectionner");
    assert_eq!(s("TOUT SELECTIONNER"), "tout selectionner");
    assert_eq!(s("Œuvre"), "oeuvre");
    assert_eq!(s("Ñandú"), "nandu");
    let strasse = fold("Straße");
    assert_eq!(strasse.chars.iter().collect::<String>(), "strasse");
    // Both `s` of `ß` come from it.
    assert_eq!(strasse.origin, [0, 1, 2, 3, 4, 4, 5]);
}

#[test]
fn every_label_folds_to_plain_letters() {
    for language in Language::ALL {
        set_language(language);
        let mut texts: Vec<String> = CommandId::all()
            .into_iter()
            .map(|id| tr(id.meta().label))
            .collect();
        for menu in menus() {
            texts.push(tr(menu.title));
        }
        for key in [
            "menu-combine",
            "menu-align",
            "palette-group-tools",
            "palette-group-colors",
            "palette-group-symbol",
        ] {
            texts.push(tr(key));
        }
        for text in texts {
            let folded = fold(&text);
            assert!(
                folded
                    .chars
                    .iter()
                    .all(|c| !c.is_alphabetic() || c.is_ascii_lowercase()),
                "{language:?}: {text:?} folds to {:?}",
                folded.chars
            );
        }
    }
}

#[test]
fn letters_in_order() {
    let all = commands(Language::English);
    assert_eq!(labels(&all, "flp hor")[0], "Flip Horizontal");
}

#[test]
fn words_in_any_order() {
    let all = commands(Language::English);
    assert!(labels(&all, "horizontal flip").contains(&"Flip Horizontal".to_owned()));
}

#[test]
fn case_and_accents_ignored() {
    let all = commands(Language::French);
    let found = rank(&all, "tout selectionner");
    let select_all = found
        .iter()
        .map(|(i, _)| &all[*i])
        .find(|e| e.target == Target::Command(CommandId::SelectAll))
        .expect("Select All");
    assert_eq!(select_all.label, "Tout sélectionner");
    assert_eq!(select_all.context, "Édition");
}

#[test]
fn interface_language() {
    let all = commands(Language::German);
    let found: Vec<Target> = rank(&all, "spiegeln")
        .into_iter()
        .map(|(i, _)| all[i].target)
        .collect();
    // The shorter "Vertikal spiegeln" comes first.
    for axis in FlipAxis::ALL {
        assert!(
            found[..2].contains(&Target::Command(CommandId::Flip(axis))),
            "{found:?}"
        );
    }
    assert_eq!(all[rank(&all, "spiegeln")[0].0].context, "Objekt");
}

#[test]
fn match_in_the_context() {
    set_language(Language::English);
    let mut all = commands(Language::English);
    all.push(entry("Chassis", "TruckPaint Sample Truck › Accessories"));
    all.push(entry(
        "Standard cab",
        "TruckPaint Sample Truck › Main textures",
    ));
    assert_eq!(labels(&all, "accessories chassis"), ["Chassis"]);
}

#[test]
fn nothing_matches() {
    let all = commands(Language::English);
    assert!(rank(&all, "zzzz").is_empty());
    assert!(rows(&all, "zzzz", &[]).is_empty());
}

#[test]
fn word_start_beats_inside_word() {
    assert!(score_of("ex", "Export", "") > score_of("ex", "Indexe", ""));
}

#[test]
fn contiguous_beats_scattered() {
    assert!(score_of("grid", "Gridxx", "") > score_of("grid", "Gxrixd", ""));
}

#[test]
fn label_beats_context() {
    assert!(score_of("snap", "Snap", "") > score_of("snap", "Xxxx", "Snap"));
}

#[test]
fn shorter_label_wins_a_tie() {
    let list = [entry("Grid lines", ""), entry("Grid", "")];
    assert_eq!(labels(&list, "grid"), ["Grid", "Grid lines"]);
}

#[test]
fn equal_scores_keep_list_order() {
    let mut list = vec![entry("Base", "Trailer"), entry("Base", "Truck")];
    list[1].target = Target::Texture(1);
    let found: Vec<usize> = rank(&list, "base").into_iter().map(|(i, _)| i).collect();
    assert_eq!(found, [0, 1]);
}

#[test]
fn disabled_after_enabled() {
    let mut list = vec![entry("Group", ""), entry("Ungroup selection", "")];
    list[0].enabled = false;
    assert_eq!(labels(&list, "group"), ["Ungroup selection", "Group"]);
}

#[test]
fn matched_letters_are_highlighted() {
    let m = score(&words("flp"), "Flip Horizontal", "Object").unwrap();
    assert_eq!(m.label_hits, [0, 1, 3]);
    // A context-only match highlights nothing in the label.
    let m = score(&words("object"), "Flip Horizontal", "Object").unwrap();
    assert!(m.label_hits.is_empty());
    // `ss` matching `ß` highlights that one char.
    let m = score(&words("strasse"), "Straße", "").unwrap();
    assert_eq!(m.label_hits, [0, 1, 2, 3, 4, 5]);
}

#[test]
fn empty_query_lists_recent_then_textures_then_commands() {
    let mut list = commands(Language::English);
    let first_command = list.len();
    list.push(Entry {
        target: Target::Texture(0),
        ..entry("Standard cab", "Truck › Main textures")
    });
    let grid = CommandId::ShowGrid;
    let flip = CommandId::Flip(FlipAxis::Horizontal);
    let mut recent = Vec::new();
    push_recent(&mut recent, flip);
    push_recent(&mut recent, grid);
    let rows = rows(&list, "", &recent);
    let target = |row: &Row| row.entry().map(|i| list[i].target);
    assert_eq!(rows[0], Row::Heading("palette-heading-recent"));
    assert_eq!(target(&rows[1]), Some(Target::Command(grid)));
    assert_eq!(target(&rows[2]), Some(Target::Command(flip)));
    assert_eq!(rows[3], Row::Heading("palette-heading-textures"));
    assert_eq!(rows[4].entry(), Some(first_command));
    assert_eq!(rows[5], Row::Heading("palette-heading-commands"));
    // Every command once.
    let commands: Vec<usize> = rows.iter().filter_map(Row::entry).collect();
    assert_eq!(commands.len(), list.len());
    // First use: no Recent heading.
    let rows = super::rows(&list, "  ", &[]);
    assert_eq!(rows[0], Row::Heading("palette-heading-textures"));
}

#[test]
fn recent_keeps_five_without_duplicates() {
    let mut recent = Vec::new();
    for id in [
        CommandId::Save,
        CommandId::ShowGrid,
        CommandId::Save,
        CommandId::ZoomIn,
        CommandId::ZoomOut,
        CommandId::FitToScreen,
        CommandId::ActualSize,
    ] {
        push_recent(&mut recent, id);
    }
    assert_eq!(
        recent,
        [
            CommandId::ActualSize,
            CommandId::FitToScreen,
            CommandId::ZoomOut,
            CommandId::ZoomIn,
            CommandId::Save,
        ]
    );
}
