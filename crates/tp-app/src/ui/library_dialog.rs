//! Import from Library… dialog: the library's symbols, swatches, graphic
//! styles and text styles, to check and import into the project, or to
//! remove from the library.

use std::collections::{BTreeSet, HashMap};
use std::sync::mpsc::{self, Receiver};

use egui::{Align, Layout, Rect, RichText, ScrollArea, Sense, Ui, Vec2, WidgetInfo, WidgetType};
use tp_core::document::Rgba;
use tp_core::import::Picks;
use tp_core::{LibraryKey, Look, Project};
use tp_i18n::tr;
use tp_text::FontLibrary;
use tp_ui::theme::title_style;
use tp_ui::tokens::{color, radius, space};
use tp_ui::widgets::{MenuRow, primary_button, secondary_button};

use crate::state::{AppState, Screen};

/// Side of a symbol's thumbnail, in points.
const THUMBNAIL: f32 = 28.0;
/// Side it is rendered at, in pixels (sharp on high-density screens).
const THUMBNAIL_PIXELS: u32 = 64;
const ROW_HEIGHT: f32 = 32.0;

/// State of the open dialog.
#[derive(Default)]
pub struct LibraryDialog {
    /// Checked entries.
    pub checked: BTreeSet<LibraryKey>,
    /// Entry waiting for the removal confirmation.
    pub confirm_remove: Option<LibraryKey>,
    /// Why the library is empty when it could not be read.
    pub issue: Option<String>,
    thumbnails: HashMap<LibraryKey, egui::TextureHandle>,
    /// Symbols being rendered, and the job rendering them.
    rendering: BTreeSet<LibraryKey>,
    job: Option<Receiver<Vec<(LibraryKey, egui::ColorImage)>>>,
}

impl std::fmt::Debug for LibraryDialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LibraryDialog")
            .field("checked", &self.checked)
            .field("confirm_remove", &self.confirm_remove)
            .field("issue", &self.issue)
            .finish_non_exhaustive()
    }
}

impl LibraryDialog {
    /// Whether every symbol's thumbnail is shown.
    pub fn thumbnails_ready(&self, library: &Project) -> bool {
        library
            .symbols
            .iter()
            .filter_map(|s| s.origin.as_ref())
            .all(|k| self.thumbnails.contains_key(k))
    }
}

/// The message about an unreadable library.
pub fn issue_message(issue: &crate::library::LibraryIssue) -> String {
    match &issue.backup {
        Some(path) => {
            let file = path.file_name().map_or_else(
                || path.display().to_string(),
                |n| n.to_string_lossy().into(),
            );
            tr!("library-unreadable", file = file)
        }
        None => tr("library-unreadable-no-backup"),
    }
}

/// What a row previews.
enum Preview {
    Symbol,
    Color(Rgba),
    Look(Box<Look>),
}

/// A library entry, as a row.
struct Entry {
    key: LibraryKey,
    name: String,
    preview: Preview,
}

/// The library's entries in four groups: (title, entries).
fn groups(library: &Project) -> [(&'static str, Vec<Entry>); 4] {
    let entry = |origin: &Option<LibraryKey>, name: &str, preview| {
        origin.clone().map(|key| Entry {
            key,
            name: name.to_owned(),
            preview,
        })
    };
    [
        (
            "library-symbols",
            library
                .symbols
                .iter()
                .filter_map(|s| entry(&s.origin, &s.name, Preview::Symbol))
                .collect(),
        ),
        (
            "library-swatches",
            library
                .palette
                .iter()
                .filter_map(|s| entry(&s.origin, &s.name, Preview::Color(s.color)))
                .collect(),
        ),
        (
            "library-graphic-styles",
            library
                .graphic_styles
                .iter()
                .filter_map(|s| entry(&s.origin, &s.name, Preview::Look(Box::new(s.look))))
                .collect(),
        ),
        (
            "library-text-styles",
            library
                .text_styles
                .iter()
                .filter_map(|s| entry(&s.origin, &s.name, Preview::Look(Box::new(s.look))))
                .collect(),
        ),
    ]
}

/// The library's elements linked to the `checked` entries.
pub fn picks(library: &Project, checked: &BTreeSet<LibraryKey>) -> Picks {
    let has = |o: &Option<LibraryKey>| o.as_ref().is_some_and(|k| checked.contains(k));
    Picks {
        symbols: library
            .symbols
            .iter()
            .filter(|s| has(&s.origin))
            .map(|s| s.id)
            .collect(),
        swatches: library
            .palette
            .iter()
            .filter(|s| has(&s.origin))
            .map(|s| s.id)
            .collect(),
        graphic_styles: library
            .graphic_styles
            .iter()
            .filter(|s| has(&s.origin))
            .map(|s| s.id)
            .collect(),
        text_styles: library
            .text_styles
            .iter()
            .filter(|s| has(&s.origin))
            .map(|s| s.id)
            .collect(),
        objects: Vec::new(),
    }
}

/// Renders the thumbnails of `symbols` of `library` on a worker thread.
fn start_thumbnails(
    ctx: &egui::Context,
    library: &Project,
    symbols: Vec<LibraryKey>,
    mut fonts: FontLibrary,
) -> Receiver<Vec<(LibraryKey, egui::ColorImage)>> {
    // One single-surface project per symbol: the renderer draws surfaces.
    let jobs: Vec<(LibraryKey, Project)> = symbols
        .into_iter()
        .filter_map(|key| {
            let symbol = library
                .symbols
                .iter()
                .find(|s| s.origin.as_ref() == Some(&key))?;
            let mut p = library.clone();
            p.symbols.clear();
            p.surfaces = vec![symbol.surface.clone()];
            p.active_surface = 0;
            p.editing_symbol = None;
            Some((key, p))
        })
        .collect();
    let (tx, rx) = mpsc::channel();
    let ctx = ctx.clone();
    let spawned = std::thread::Builder::new()
        .name("library-thumbnails".into())
        .spawn(move || {
            let images = jobs
                .into_iter()
                .map(|(key, p)| {
                    let side = THUMBNAIL_PIXELS;
                    let pixmap = tp_render::render_cover(&p, 0, side, side, None, &mut fonts);
                    let rgba = tp_render::to_rgba(&pixmap);
                    let image = egui::ColorImage::from_rgba_unmultiplied(
                        [side as usize, side as usize],
                        rgba.as_raw(),
                    );
                    (key, image)
                })
                .collect();
            let _ = tx.send(images);
            ctx.request_repaint();
        });
    if let Err(err) = spawned {
        tracing::error!(%err, "cannot render the library thumbnails");
    }
    rx
}

/// Picks up finished thumbnails and starts rendering the missing ones.
fn update_thumbnails(
    ctx: &egui::Context,
    dialog: &mut LibraryDialog,
    library: &Project,
    fonts: &FontLibrary,
) {
    if let Some(job) = &dialog.job
        && let Ok(images) = job.try_recv()
    {
        for (key, image) in images {
            let name = format!("library_thumbnail_{}", key.0);
            let texture = ctx.load_texture(name, image, egui::TextureOptions::LINEAR);
            dialog.thumbnails.insert(key, texture);
        }
        dialog.job = None;
        dialog.rendering.clear();
    }
    if dialog.job.is_some() {
        return;
    }
    let missing: Vec<LibraryKey> = library
        .symbols
        .iter()
        .filter_map(|s| s.origin.clone())
        .filter(|k| !dialog.thumbnails.contains_key(k) && !dialog.rendering.contains(k))
        .collect();
    if !missing.is_empty() {
        dialog.rendering.extend(missing.iter().cloned());
        dialog.job = Some(start_thumbnails(ctx, library, missing, fonts.fork()));
    }
}

fn preview(ui: &mut Ui, dialog: &LibraryDialog, entry: &Entry) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(THUMBNAIL), Sense::hover());
    match &entry.preview {
        Preview::Symbol => {
            ui.painter().rect_filled(rect, radius::SM, color::SURFACE_0);
            if let Some(texture) = dialog.thumbnails.get(&entry.key) {
                ui.painter().image(
                    texture.id(),
                    rect.shrink(2.0),
                    Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
        }
        Preview::Color(c) => chip(ui, rect, *c, None),
        Preview::Look(look) => {
            let ring = look.stroke.map(|s| s.paint.first_color());
            chip(ui, rect, look.fill.first_color(), ring);
        }
    }
}

/// A color chip, ringed by `ring` (a style's stroke); the row's checkbox
/// names it.
fn chip(ui: &Ui, rect: Rect, fill: Rgba, ring: Option<Rgba>) {
    let color32 = |c: Rgba| egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a);
    let chip = Rect::from_center_size(rect.center(), Vec2::splat(20.0));
    let painter = ui.painter();
    painter.rect_filled(chip, radius::SM, color32(fill));
    painter.rect_stroke(
        chip,
        radius::SM,
        egui::Stroke::new(1.0, color::BORDER_STRONG),
        egui::StrokeKind::Outside,
    );
    if let Some(c) = ring {
        painter.rect_stroke(
            chip.shrink(1.0),
            radius::SM,
            egui::Stroke::new(3.0, color32(c)),
            egui::StrokeKind::Inside,
        );
    }
}

/// What the user asked for in a frame.
#[derive(Default)]
struct Actions {
    import: bool,
    remove: Option<LibraryKey>,
    close: bool,
}

fn entry_row(
    ui: &mut Ui,
    dialog: &mut LibraryDialog,
    project: &Project,
    entry: &Entry,
    actions: &mut Actions,
) {
    let in_project = project.has_origin(&entry.key);
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), ROW_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            preview(ui, dialog, entry);
            let mut checked = in_project || dialog.checked.contains(&entry.key);
            let response =
                ui.add_enabled(!in_project, egui::Checkbox::new(&mut checked, &entry.name));
            if response.changed() {
                if checked {
                    dialog.checked.insert(entry.key.clone());
                } else {
                    dialog.checked.remove(&entry.key);
                }
            }
            if in_project {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(tr("library-in-project"))
                            .small()
                            .color(color::TEXT_SECONDARY),
                    );
                });
            }
            // A disabled checkbox ignores clicks: its row has the menu.
            let menu = if in_project {
                ui.interact(
                    ui.min_rect(),
                    ui.id().with(("library_row", &entry.key.0)),
                    Sense::click(),
                )
            } else {
                response
            };
            menu.context_menu(|ui| {
                if ui.add(MenuRow::new(&tr("library-remove"))).clicked() {
                    dialog.confirm_remove = Some(entry.key.clone());
                    ui.close();
                }
            });
        },
    );
    if dialog.confirm_remove.as_ref() == Some(&entry.key) {
        let text = tr!("library-remove-confirm", name = entry.name.as_str());
        ui.label(RichText::new(&text).small().color(color::WARNING))
            .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
        ui.horizontal(|ui| {
            if ui
                .add(primary_button(&tr("library-remove-button")))
                .clicked()
            {
                actions.remove = Some(entry.key.clone());
            }
            if ui.add(secondary_button(&tr("button-cancel"))).clicked() {
                dialog.confirm_remove = None;
            }
        });
    }
}

/// Shows the dialog; returns whether it stays open.
pub fn show(ctx: &egui::Context, state: &mut AppState, dialog: &mut LibraryDialog) -> bool {
    let now = ctx.input(|i| i.time);
    let AppState {
        library, screen, ..
    } = state;
    let Screen::Workspace(ws) = screen else {
        return false;
    };
    if let Some(issue) = library.take_issue() {
        dialog.issue = Some(issue_message(&issue));
    }
    let lib = library.library();
    update_thumbnails(ctx, dialog, lib, &ws.text.fonts);
    let mut actions = Actions::default();
    crate::ui::dialogs::modal("import_from_library_modal").show(ctx, |ui| {
        ui.set_width(460.0);
        ui.label(
            RichText::new(tr("cmd-import-from-library").trim_end_matches('…'))
                .text_style(title_style())
                .color(color::TEXT_PRIMARY),
        );
        ui.add_space(space::XS);
        if let Some(issue) = &dialog.issue {
            ui.label(RichText::new(issue).small().color(color::WARNING))
                .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, issue));
            ui.add_space(space::XS);
        }
        let groups = groups(lib);
        let empty = groups.iter().all(|(_, entries)| entries.is_empty());
        let hint = tr(if empty {
            "library-empty"
        } else {
            "library-hint"
        });
        ui.label(RichText::new(&hint).small().color(color::TEXT_SECONDARY))
            .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &hint));
        if !empty {
            ui.add_space(space::SM);
            ScrollArea::vertical()
                .max_height(420.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for (title, entries) in &groups {
                        if entries.is_empty() {
                            continue;
                        }
                        ui.add_space(space::XS);
                        ui.label(
                            RichText::new(tr(title))
                                .small()
                                .color(color::TEXT_SECONDARY),
                        );
                        for entry in entries {
                            entry_row(ui, dialog, &ws.project, entry, &mut actions);
                        }
                    }
                });
        }
        ui.add_space(space::LG);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let any = !dialog.checked.is_empty();
            actions.import |= ui
                .add_enabled(any, primary_button(&tr("library-import")))
                .clicked();
            actions.close |= ui.add(secondary_button(&tr("button-cancel"))).clicked();
        });
    });
    if actions.import {
        let picks = picks(lib, &dialog.checked);
        let lib = lib.clone();
        ws.import_from_library(&lib, &picks, now);
        return false;
    }
    if let Some(key) = actions.remove {
        library.remove(&key);
        dialog.checked.remove(&key);
        dialog.confirm_remove = None;
        dialog.thumbnails.remove(&key);
    }
    // Entries imported or removed meanwhile are no longer checked.
    let lib = library.library();
    dialog
        .checked
        .retain(|k| lib.has_origin(k) && !ws.project.has_origin(k));
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        actions.close = true;
    }
    !actions.close
}
