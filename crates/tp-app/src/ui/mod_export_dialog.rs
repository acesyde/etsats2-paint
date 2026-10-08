//! Export Mod dialog: the mod settings, its pictures, a summary of what
//! the mod holds, the problems that block it, and the export's progress.

use std::sync::mpsc::{self, Receiver};

use egui::{
    Align, Color32, CornerRadius, Key, Layout, Rect, RichText, Sense, Stroke, TextEdit, Ui, Vec2,
    WidgetInfo, WidgetType,
};
use tp_core::{ModSettings, Project};
use tp_i18n::tr;
use tp_text::FontLibrary;
use tp_ui::theme::{label_strong_style, title_style};
use tp_ui::tokens::{color, space};
use tp_ui::widgets::{FieldEvent, NumericField, primary_button, secondary_button};

use crate::export::ExportOutcome;
use crate::mod_export::{
    ICON_SIZE, IMAGE_SIZE, ModJob, Picture, PictureFile, Problem, VehicleSummary,
};
use crate::state::{AppState, Modal};

/// Previews of the shop icon and the Mod Manager image, rendered on a
/// worker thread.
struct PreviewJob {
    revision: u64,
    done: Receiver<[egui::ColorImage; 2]>,
}

/// State of the open Export Mod dialog.
pub struct ModExportDialog {
    /// The settings being edited (the project keeps its own until Export…).
    pub settings: ModSettings,
    pub icon: Picture,
    pub image: Picture,
    summary: Vec<VehicleSummary>,
    /// Why the last chosen picture couldn't be used.
    pub picture_error: Option<String>,
    /// Bumped when a picture changes.
    revision: u64,
    previews: Option<[egui::TextureHandle; 2]>,
    /// Revision of the shown previews.
    pub preview_revision: Option<u64>,
    preview_job: Option<PreviewJob>,
    pub job: Option<ModJob>,
}

impl ModExportDialog {
    /// The dialog for `project`, whose vehicles are summarized in `summary`.
    pub fn new(project: &Project, summary: Vec<VehicleSummary>) -> Self {
        let settings = project.mod_settings.clone();
        Self {
            icon: Picture::of(settings.icon),
            image: Picture::of(settings.image),
            settings,
            summary,
            picture_error: None,
            revision: 0,
            previews: None,
            preview_revision: None,
            preview_job: None,
            job: None,
        }
    }

    /// Whether the previews show the current pictures.
    pub fn preview_ready(&self) -> bool {
        self.previews.is_some() && self.preview_revision == Some(self.revision)
    }

    fn set_picture(&mut self, which: Which, picture: Picture) {
        match which {
            Which::Icon => self.icon = picture,
            Which::Image => self.image = picture,
        }
        self.revision += 1;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Which {
    Icon,
    Image,
}

/// Starts rendering both previews of `project` with `icon` and `image`.
fn start_previews(
    ctx: &egui::Context,
    project: Project,
    icon: Picture,
    image: Picture,
    mut fonts: FontLibrary,
    revision: u64,
) -> PreviewJob {
    let (tx, done) = mpsc::channel();
    let ctx = ctx.clone();
    let spawned = std::thread::Builder::new()
        .name("mod-preview".into())
        .spawn(move || {
            let mut render = |picture: &Picture, size: (u32, u32)| {
                let pixmap =
                    crate::mod_export::picture(&project, picture.bytes(&project), size, &mut fonts)
                        .unwrap_or_else(|_| tp_render::Pixmap::new(size.0, size.1).expect("size"));
                let rgba = tp_render::to_rgba(&pixmap);
                egui::ColorImage::from_rgba_unmultiplied(
                    [size.0 as usize, size.1 as usize],
                    rgba.as_raw(),
                )
            };
            let images = [render(&icon, ICON_SIZE), render(&image, IMAGE_SIZE)];
            let _ = tx.send(images);
            ctx.request_repaint();
        });
    if let Err(err) = spawned {
        tracing::error!(%err, "cannot start the mod previews");
    }
    PreviewJob { revision, done }
}

/// A problem in the current language.
pub fn problem_message(problem: &Problem) -> String {
    match problem {
        Problem::NameEmpty => tr("mod-problem-name-empty"),
        Problem::NameInvalid => tr("mod-problem-name-invalid"),
        Problem::VersionInvalid => tr("mod-problem-version-invalid"),
        Problem::AuthorInvalid => tr("mod-problem-author-invalid"),
        Problem::InternalNameEmpty => tr("mod-problem-internal-empty"),
        Problem::InternalNameInvalid => tr("mod-problem-internal-invalid"),
        Problem::InternalNameTooLong { max } => {
            tr!("mod-problem-internal-long", max = max.to_string())
        }
        Problem::PriceZero => tr("mod-problem-price"),
        Problem::SamePath {
            path,
            first,
            second,
        } => tr!(
            "mod-problem-same-path",
            path = path.as_str(),
            first = first.as_str(),
            second = second.as_str()
        ),
        Problem::MissingGameData { vehicle, version } => tr!(
            "mod-problem-game-data",
            vehicle = vehicle.as_str(),
            version = version.as_str()
        ),
    }
}

fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
}

/// A field's label in the grid's first column.
fn field_label(ui: &mut Ui, label: &str) {
    ui.label(RichText::new(label).color(color::TEXT_SECONDARY));
}

/// A labelled single-line text field.
fn text_field(ui: &mut Ui, label: &str, text: &mut String) -> bool {
    field_label(ui, label);
    let response = ui.add(TextEdit::singleline(text).desired_width(f32::INFINITY));
    response.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, label));
    ui.end_row();
    response.changed()
}

/// A labelled whole-number field.
fn number_field(ui: &mut Ui, label: &str, value: &mut u32, max: u32) {
    field_label(ui, label);
    let event = NumericField::new("", label, Some(f64::from(*value)))
        .range(0.0..=f64::from(max))
        .width(120.0)
        .show(ui);
    if let FieldEvent::Live(v) | FieldEvent::Commit(v) = event {
        *value = v.round() as u32;
    }
    ui.end_row();
}

/// The settings fields.
fn settings_ui(ui: &mut Ui, settings: &mut ModSettings, limit: usize) {
    egui::Grid::new("mod_settings_fields")
        .num_columns(2)
        .spacing([space::MD, space::XS])
        .min_col_width(120.0)
        .show(ui, |ui| {
            text_field(ui, &tr("mod-name"), &mut settings.name);
            text_field(ui, &tr("mod-version"), &mut settings.version);
            text_field(ui, &tr("mod-author"), &mut settings.author);
            let label = tr("mod-description");
            field_label(ui, &label);
            let response = ui.add(
                TextEdit::multiline(&mut settings.description)
                    .desired_rows(3)
                    .desired_width(f32::INFINITY),
            );
            response.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, &label));
            ui.end_row();
            number_field(ui, &tr("mod-price"), &mut settings.price, 100_000_000);
            number_field(ui, &tr("mod-unlock"), &mut settings.unlock_level, 1000);
            // The internal name follows the Name until the player types one.
            let mut internal = settings.internal_name(limit);
            if text_field(ui, &tr("mod-internal-name"), &mut internal) {
                settings.internal_name = Some(internal);
            }
            ui.label("");
            ui.label(
                RichText::new(tr("mod-internal-name-help"))
                    .small()
                    .color(color::TEXT_SECONDARY),
            );
            ui.end_row();
        });
}

/// One picture: its preview and its buttons. Returns the button clicked:
/// `Some(true)` to choose a file, `Some(false)` to use the generated one.
fn picture_ui(
    ui: &mut Ui,
    title: &str,
    preview: Option<&egui::TextureHandle>,
    size: (u32, u32),
    chosen: bool,
    choose: &str,
    generated: &str,
) -> Option<bool> {
    let mut clicked = None;
    ui.vertical(|ui| {
        ui.label(RichText::new(title).text_style(label_strong_style()));
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(size.0 as f32, size.1 as f32), Sense::hover());
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Image, true, title));
        match preview {
            Some(texture) => {
                ui.painter().image(
                    texture.id(),
                    rect,
                    Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            None => {
                egui::Spinner::new()
                    .paint_at(ui, Rect::from_center_size(rect.center(), Vec2::splat(20.0)));
            }
        }
        ui.painter().rect_stroke(
            rect,
            CornerRadius::ZERO,
            Stroke::new(1.0, color::BORDER_STRONG),
            egui::StrokeKind::Outside,
        );
        ui.add_space(space::XS);
        ui.horizontal(|ui| {
            if ui.add(secondary_button(choose)).clicked() {
                clicked = Some(true);
            }
            if chosen && ui.add(secondary_button(generated)).clicked() {
                clicked = Some(false);
            }
        });
    });
    clicked
}

/// The summary of what the mod holds.
fn summary_ui(ui: &mut Ui, summary: &[VehicleSummary]) {
    ui.label(RichText::new(tr("mod-summary")).text_style(label_strong_style()));
    for vehicle in summary {
        ui.label(RichText::new(&vehicle.name).color(color::TEXT_PRIMARY));
        for main in &vehicle.mains {
            let text = if !main.painted {
                tr!("mod-summary-not-painted", texture = main.name.as_str())
            } else if main.cabins.is_empty() {
                tr!("mod-summary-every-cabin", texture = main.name.as_str())
            } else {
                tr!(
                    "mod-summary-cabins",
                    texture = main.name.as_str(),
                    cabins = main.cabins.join(", ")
                )
            };
            let label = ui.label(RichText::new(&text).small().color(color::TEXT_SECONDARY));
            label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
        }
        if !vehicle.accessories.is_empty() {
            let text = tr!(
                "mod-summary-accessories",
                accessories = vehicle.accessories.join(", ")
            );
            let label = ui.label(RichText::new(&text).small().color(color::TEXT_SECONDARY));
            label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
        }
    }
}

/// Shows the dialog; returns false when it should close.
pub fn show(ctx: &egui::Context, state: &mut AppState, dialog: &mut ModExportDialog) -> bool {
    let now = ctx.input(|i| i.time);
    let Some(ws) = state.workspace_mut() else {
        return false;
    };

    // Previews: pick up a finished render, start one when a picture changed.
    if let Some(job) = &dialog.preview_job
        && let Ok([icon, image]) = job.done.try_recv()
    {
        let options = egui::TextureOptions::LINEAR;
        dialog.previews = Some([
            ctx.load_texture("mod_icon_preview", icon, options),
            ctx.load_texture("mod_image_preview", image, options),
        ]);
        dialog.preview_revision = Some(job.revision);
        dialog.preview_job = None;
    }
    let requested = dialog
        .preview_job
        .as_ref()
        .map(|j| j.revision)
        .or(dialog.preview_revision);
    if requested != Some(dialog.revision) {
        dialog.preview_job = Some(start_previews(
            ctx,
            ws.project.clone(),
            dialog.icon.clone(),
            dialog.image.clone(),
            ws.text.fonts.fork(),
            dialog.revision,
        ));
    }

    // A running export.
    if let Some(job) = &dialog.job
        && let Some(outcome) = job.poll()
    {
        dialog.job = None;
        match outcome {
            ExportOutcome::Written(path) => {
                ws.show_hint(tr!("mod-export-done", file = file_name(&path)), now);
                return false;
            }
            ExportOutcome::Cancelled => {}
            ExportOutcome::Failed { path, reason } => {
                state.modal = Some(Modal::Message {
                    title: tr("export-failed-title"),
                    text: tr!("export-failed", file = file_name(&path), reason = reason),
                });
                return false;
            }
        }
    }

    let limit = ws.project.internal_name_limit();
    let problems = crate::mod_export::problems_with(&ws.project, &dialog.settings);
    let exporting = dialog.job.as_ref().map(|j| (j.progress(), j.path.clone()));
    let mut keep = true;
    let mut start_export = false;
    let mut pick: Option<Which> = None;
    let mut generated: Option<Which> = None;
    super::dialogs::modal("mod_export_modal").show(ctx, |ui| {
        ui.set_width(760.0);
        ui.label(
            RichText::new(tr("dialog-export-mod"))
                .text_style(title_style())
                .color(color::TEXT_PRIMARY),
        );
        ui.add_space(space::LG);
        ui.add_enabled_ui(exporting.is_none(), |ui| {
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(420.0);
                    settings_ui(ui, &mut dialog.settings, limit);
                });
                ui.add_space(space::XL);
                ui.vertical(|ui| summary_ui(ui, &dialog.summary));
            });
            ui.add_space(space::LG);
            let previews = dialog.previews.as_ref().filter(|_| dialog.preview_ready());
            ui.horizontal_top(|ui| {
                let icon = picture_ui(
                    ui,
                    &tr("mod-icon"),
                    previews.map(|p| &p[0]),
                    ICON_SIZE,
                    dialog.icon != Picture::Generated,
                    &tr("mod-choose-icon"),
                    &tr("mod-generated-icon"),
                );
                ui.add_space(space::XL);
                let image = picture_ui(
                    ui,
                    &tr("mod-image"),
                    previews.map(|p| &p[1]),
                    IMAGE_SIZE,
                    dialog.image != Picture::Generated,
                    &tr("mod-choose-image"),
                    &tr("mod-generated-image"),
                );
                for (which, clicked) in [(Which::Icon, icon), (Which::Image, image)] {
                    match clicked {
                        Some(true) => pick = Some(which),
                        Some(false) => generated = Some(which),
                        None => {}
                    }
                }
            });
            if let Some(error) = &dialog.picture_error {
                ui.label(RichText::new(error).small().color(color::ERROR));
            }
        });
        ui.add_space(space::LG);
        for problem in &problems {
            let text = problem_message(problem);
            let label = ui.label(RichText::new(&text).small().color(color::ERROR));
            label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
        }
        ui.add_space(space::MD);
        match &exporting {
            Some((progress, path)) => {
                ui.add(
                    egui::ProgressBar::new(*progress)
                        .text(tr!("export-progress", file = file_name(path))),
                );
                ui.add_space(space::SM);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.add(secondary_button(&tr("button-cancel"))).clicked()
                        && let Some(job) = &dialog.job
                    {
                        job.cancel();
                    }
                });
            }
            None => {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    start_export |= ui
                        .add_enabled(problems.is_empty(), primary_button(&tr("export-start")))
                        .clicked();
                    if ui.add(secondary_button(&tr("button-cancel"))).clicked() {
                        keep = false;
                    }
                });
            }
        }
    });
    if exporting.is_none() && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape)) {
        keep = false;
    }

    if let Some(which) = generated {
        dialog.set_picture(which, Picture::Generated);
    }
    if let Some(which) = pick
        && let Some(path) = state.dialogs.pick_mod_image()
    {
        let name = path
            .file_stem()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        match std::fs::read(&path)
            .map_err(|e| e.to_string())
            .and_then(|bytes| PictureFile::read(&name, bytes))
        {
            Ok(file) => {
                dialog.picture_error = None;
                dialog.set_picture(which, Picture::File(file));
            }
            Err(reason) => {
                dialog.picture_error = Some(tr!(
                    "mod-picture-failed",
                    file = file_name(&path),
                    reason = reason
                ));
            }
        }
    }

    if start_export && problems.is_empty() {
        start(ctx, state, dialog, now);
    }
    keep
}

/// Records the settings, asks for the destination and starts the export.
fn start(ctx: &egui::Context, state: &mut AppState, dialog: &mut ModExportDialog, now: f64) {
    let Some(ws) = state.workspace_mut() else {
        return;
    };
    ws.set_mod_settings(dialog.settings.clone(), &dialog.icon, &dialog.image, now);
    // From now on the dialog edits the recorded settings.
    let settings = ws.project.mod_settings.clone();
    dialog.icon = Picture::of(settings.icon);
    dialog.image = Picture::of(settings.image);
    dialog.settings = settings;
    let Ok(plan) = crate::mod_export::plan(&ws.project) else {
        return;
    };
    let project = ws.project.clone();
    let fonts = ws.text.fonts.fork();
    let folder = project
        .game()
        .and_then(crate::mod_export::game_mod_folder)
        .or_else(|| state.last_mod_folder.clone());
    let suggested = crate::mod_export::suggested_name(&project.mod_settings.name);
    let Some(path) = state.dialogs.save_mod(&suggested, folder.as_deref()) else {
        return;
    };
    let path = crate::mod_export::with_extension(path);
    state.last_mod_folder = path.parent().map(std::path::Path::to_path_buf);
    let ctx = ctx.clone();
    dialog.job = Some(ModJob::start(project, plan, fonts, path, move || {
        ctx.request_repaint();
    }));
}
