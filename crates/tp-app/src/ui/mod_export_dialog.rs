//! Export Mod dialog: the mod settings labelled above their fields, the
//! internal name under Advanced, the mod's pictures, a summary of what the
//! mod holds, the problems that block it just above the buttons, and the
//! export's progress.

use std::sync::mpsc::{self, Receiver};

use egui::{
    Color32, CornerRadius, Key, Rect, RichText, Sense, Stroke, TextEdit, Ui, Vec2, WidgetInfo,
    WidgetType,
};
use tp_core::{ModSettings, Project};
use tp_i18n::tr;
use tp_text::FontLibrary;
use tp_ui::icons;
use tp_ui::theme::label_strong_style;
use tp_ui::tokens::{color, radius, space};
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
    /// The Advanced section (the internal name) is open. It opens by
    /// itself while a problem concerns the internal name.
    pub advanced: bool,
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
            advanced: false,
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
            let images = crate::mod_previews::render(&project, &icon, &image, &mut fonts);
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
        Problem::BadGameVersion { version } => {
            tr!("mod-problem-bad-game-version", version = version.as_str())
        }
        Problem::UnsupportedGameVersion { version, vehicle } => tr!(
            "mod-problem-unsupported-game-version",
            version = version.as_str(),
            vehicle = vehicle.vehicle.as_str(),
            range = vehicle.range.as_str()
        ),
        Problem::NoCommonGameVersion { first, second } => tr!(
            "mod-problem-no-common-game-version",
            first = first.vehicle.as_str(),
            first_range = first.range.as_str(),
            second = second.vehicle.as_str(),
            second_range = second.range.as_str()
        ),
    }
}

fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
}

/// A labelled whole-number field filling its column, as tall as the text
/// fields.
fn number_field(ui: &mut Ui, label: &str, value: &mut u32, max: u32) {
    super::dialogs::labelled(ui, label, |ui| {
        let event = NumericField::new("", label, Some(f64::from(*value)))
            .range(0.0..=f64::from(max))
            .fill()
            .inset(super::dialogs::FIELD_HEIGHT)
            .show(ui);
        if let FieldEvent::Live(v) | FieldEvent::Commit(v) = event {
            *value = v.round() as u32;
        }
    });
}

/// Whether `problem` concerns the internal name (the Advanced section).
fn is_internal_name_problem(problem: &Problem) -> bool {
    matches!(
        problem,
        Problem::InternalNameEmpty
            | Problem::InternalNameInvalid
            | Problem::InternalNameTooLong { .. }
    )
}

/// The settings fields, each labelled above: Name and Version (in the
/// monospace face), Author with Price and Unlock level, then Description.
fn settings_ui(ui: &mut Ui, settings: &mut ModSettings) {
    use super::dialogs::{columns, labelled, mono_text_field, text_field};
    columns(ui, &[2.0, 1.0], |ui, column| match column {
        0 => {
            text_field(ui, &tr("mod-name"), &mut settings.name);
        }
        _ => {
            mono_text_field(ui, &tr("mod-version"), &mut settings.version);
        }
    });
    columns(ui, &[2.0, 1.0, 1.0], |ui, column| match column {
        0 => {
            text_field(ui, &tr("mod-author"), &mut settings.author);
        }
        1 => number_field(ui, &tr("mod-price"), &mut settings.price, 100_000_000),
        _ => number_field(ui, &tr("mod-unlock"), &mut settings.unlock_level, 1000),
    });
    let label = tr("mod-description");
    labelled(ui, &label, |ui| {
        let response = ui.add(
            TextEdit::multiline(&mut settings.description)
                .desired_rows(2)
                .margin(egui::Margin::symmetric(10, 8))
                .desired_width(f32::INFINITY),
        );
        response.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, &label));
    });
}

/// The Advanced section: a header that opens and closes it, then the
/// internal name and its help when open.
fn advanced_ui(ui: &mut Ui, settings: &mut ModSettings, limit: usize, open: &mut bool) {
    let title = tr("mod-advanced");
    let caret = if *open {
        icons::EXPANDED
    } else {
        icons::COLLAPSED
    };
    let response = ui.add(
        egui::Button::new((
            icons::rich(caret).color(color::TEXT_SECONDARY),
            RichText::new(&title).color(color::TEXT_SECONDARY),
        ))
        .frame(false)
        .min_size(Vec2::new(0.0, tp_ui::tokens::size::HIT_MIN)),
    );
    response.widget_info(|| {
        let mut info = WidgetInfo::labeled(WidgetType::CollapsingHeader, true, &title);
        info.selected = Some(*open);
        info
    });
    if response.clicked() {
        *open = !*open;
    }
    if *open {
        ui.indent("mod_export_advanced", |ui| {
            // The internal name follows the Name until the player types one.
            let mut internal = settings.internal_name(limit);
            if super::dialogs::text_field(ui, &tr("mod-internal-name"), &mut internal).changed() {
                settings.internal_name = Some(internal);
            }
            ui.add(
                egui::Label::new(
                    RichText::new(tr("mod-internal-name-help"))
                        .small()
                        .color(color::TEXT_SECONDARY),
                )
                .wrap(),
            );
        });
    }
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
        ui.spacing_mut().item_spacing.y = space::XS;
        super::dialogs::field_label(ui, title);
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
            Stroke::new(1.0, color::OUTLINE),
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

/// The summary of what the mod holds, in a box on the control surface.
fn summary_ui(ui: &mut Ui, summary: &[VehicleSummary]) {
    egui::Frame::new()
        .fill(color::CONTROL)
        .stroke(Stroke::new(1.0, color::BORDER))
        .corner_radius(radius::CARD)
        .inner_margin(egui::Margin::same(space::MD as i8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = space::XS;
            ui.label(RichText::new(tr("mod-summary")).text_style(label_strong_style()));
            for vehicle in summary {
                ui.label(RichText::new(&vehicle.name).color(color::TEXT_PRIMARY));
                let line = |ui: &mut Ui, text: &str| {
                    let label = ui.add(
                        egui::Label::new(RichText::new(text).color(color::TEXT_SECONDARY)).wrap(),
                    );
                    label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
                };
                ui.indent(("mod_summary", &vehicle.name), |ui| {
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
                        line(ui, &text);
                    }
                    if !vehicle.accessories.is_empty() {
                        line(
                            ui,
                            &tr!(
                                "mod-summary-accessories",
                                accessories = vehicle.accessories.join(", ")
                            ),
                        );
                    }
                });
            }
        });
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
    if problems.iter().any(is_internal_name_problem) {
        dialog.advanced = true;
    }
    let game = match ws.project.game() {
        Some("ats") => tp_vehicles::Game::Ats,
        _ => tp_vehicles::Game::Ets2,
    };
    let subtitle = format!(
        "{} · {}",
        ws.project.name,
        super::vehicle_dialogs::game_name(game)
    );
    // Sized to its content; the fields only scroll when the window is too
    // short for them, the title, the problems and the buttons.
    // The chrome: the dialog's margins, title, footer and a little room
    // around it.
    let chrome = 2.0 * space::XL + 70.0 + 64.0 + 2.0 * space::LG;
    let body_height =
        (ctx.content_rect().height() - chrome - 24.0 * problems.len() as f32).max(240.0);
    super::dialogs::modal("mod_export_modal").show(ctx, |ui| {
        // 640 points wide with the dialog's margins.
        ui.set_width(640.0 - 2.0 * space::XL);
        super::dialogs::title(ui, &tr("dialog-export-mod"), Some(&subtitle));
        ui.add_space(space::LG);
        ui.add_enabled_ui(exporting.is_none(), |ui| {
            egui::ScrollArea::vertical()
                .id_salt("mod_export_fields")
                .max_height(body_height)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.spacing_mut().item_spacing.y = space::SM + 2.0;
                    settings_ui(ui, &mut dialog.settings);
                    advanced_ui(ui, &mut dialog.settings, limit, &mut dialog.advanced);
                    let previews = dialog.previews.as_ref().filter(|_| dialog.preview_ready());
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = space::XL;
                        let icon = picture_ui(
                            ui,
                            &tr("mod-icon"),
                            previews.map(|p| &p[0]),
                            ICON_SIZE,
                            dialog.icon != Picture::Generated,
                            &tr("mod-choose-icon"),
                            &tr("mod-generated-icon"),
                        );
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
                        super::dialogs::problem(ui, error);
                    }
                    summary_ui(ui, &dialog.summary);
                });
        });
        // The problems that block the export, just above the buttons.
        if !problems.is_empty() {
            ui.add_space(space::MD);
            for problem in &problems {
                super::dialogs::problem(ui, &problem_message(problem));
            }
        }
        match &exporting {
            Some((progress, path)) => {
                ui.add_space(space::LG);
                ui.add(
                    egui::ProgressBar::new(*progress)
                        .text(tr!("export-progress", file = file_name(path))),
                );
                super::dialogs::footer(ui, |ui| {
                    if ui.add(secondary_button(&tr("button-cancel"))).clicked()
                        && let Some(job) = &dialog.job
                    {
                        job.cancel();
                    }
                });
            }
            None => {
                super::dialogs::footer(ui, |ui| {
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
