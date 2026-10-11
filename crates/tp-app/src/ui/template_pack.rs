//! Official vehicles in New Project and Add Vehicle…: picking one, the
//! template pack dialog that asks for the game's pack folder, and the
//! installation shown in the dialog that picked it.

use std::path::{Path, PathBuf};

use egui::{Align, Layout, RichText, Ui, WidgetInfo, WidgetType};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::theme::label_strong_style;
use tp_ui::tokens::{color, space};
use tp_ui::widgets::{IconButton, primary_button, secondary_button};
use tp_vehicles::catalog::Entry;

use super::origin::Origin;
use super::vehicle_dialogs::game_label;
use crate::official::{self, Finished, InstallJob, Installing, Pick};
use crate::state::{AppState, Modal};
use crate::vehicles::VehicleLibrary;

/// State of the template pack dialog.
pub struct TemplatePackDialog {
    /// The dialog it was opened from, shown again when it closes.
    pub origin: Origin,
    /// The official vehicle picked.
    pub entry: &'static Entry,
    /// The remembered pack folder no longer holds the vehicle.
    pub moved: bool,
    /// Why the last folder was refused.
    pub refusal: Option<String>,
}

/// Picks the official vehicle `entry` in `origin` (New Project or Add
/// Vehicle…, taken out of `state.modal`): chooses it when it is installed,
/// installs it when the pack folder holds it, or asks for the pack. The
/// origin, or the template pack dialog over it, is then in `state.modal`.
pub fn pick(ctx: &egui::Context, state: &mut AppState, mut origin: Origin, entry: &'static Entry) {
    match official::pick(&state.vehicles, &state.prefs, entry) {
        Pick::Choose(choice) => {
            origin.choose(choice);
            state.modal = Some(origin.into_modal());
        }
        Pick::Install(pack) => {
            origin.start_install(start(ctx, entry, pack));
            state.modal = Some(origin.into_modal());
        }
        Pick::AskPack { moved } => {
            state.modal = Some(Modal::TemplatePack(Box::new(TemplatePackDialog {
                origin,
                entry,
                moved,
                refusal: None,
            })));
        }
    }
}

/// Starts installing `entry` from `pack`, repainting as it goes.
fn start(ctx: &egui::Context, entry: &'static Entry, pack: PathBuf) -> InstallJob {
    let ctx = ctx.clone();
    InstallJob::start(entry, pack, move || ctx.request_repaint())
}

/// What the dialog asks for this frame.
enum Outcome {
    Pending,
    Cancel,
    ChooseFolder,
}

fn show(ctx: &egui::Context, dialog: &TemplatePackDialog) -> Outcome {
    let game = dialog.entry.game;
    let short = game_label(game);
    let mut outcome = Outcome::Pending;
    super::dialogs::modal("template_pack_modal").show(ctx, |ui| {
        ui.set_width(520.0);
        super::dialogs::title(
            ui,
            &tr!("pack-title", game = short),
            Some(&tr!(
                "pack-for-vehicle",
                vehicle = dialog.entry.name.as_str()
            )),
        );
        ui.add_space(space::MD);
        ui.spacing_mut().item_spacing.y = space::SM;
        if dialog.moved {
            super::dialogs::warning(ui, &tr!("pack-moved", game = short));
        }
        let explanation = tr!(
            "pack-explanation",
            game = super::vehicle_dialogs::game_name(game)
        );
        ui.add(egui::Label::new(RichText::new(&explanation).color(color::TEXT_PRIMARY)).wrap());
        ui.add_space(space::XS);
        step(ui, 1, &tr("pack-step-download"), |ui| {
            ui.horizontal(|ui| {
                let url = official::download_page(game);
                if ui
                    .add(secondary_button(&tr("pack-open-download")))
                    .clicked()
                {
                    ctx.open_url(egui::OpenUrl::new_tab(url));
                }
                copy_link(ui, url);
            });
        });
        step(ui, 2, &tr("pack-step-unzip"), |_| {});
        step(ui, 3, &tr("pack-step-choose"), |ui| {
            if ui.add(primary_button(&tr("pack-choose-folder"))).clicked() {
                outcome = Outcome::ChooseFolder;
            }
        });
        if let Some(refusal) = &dialog.refusal {
            super::dialogs::problem(ui, refusal);
        }
        super::dialogs::footer(ui, |ui| {
            if ui.add(secondary_button(&tr("button-cancel"))).clicked() {
                outcome = Outcome::Cancel;
            }
        });
    });
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        outcome = Outcome::Cancel;
    }
    outcome
}

/// How long the copy button shows that the link was copied, in seconds.
const COPIED_FOR: f64 = 2.0;

/// Copies `url`, for when no browser opens: the icon turns into a check
/// for a moment once copied.
fn copy_link(ui: &mut Ui, url: &str) {
    let id = ui.id().with("copied_at");
    let now = ui.input(|i| i.time);
    let copied = ui
        .data(|d| d.get_temp::<f64>(id))
        .is_some_and(|at| now - at < COPIED_FOR);
    let (icon, name) = if copied {
        (icons::CHECK, tr("pack-link-copied"))
    } else {
        (icons::COPY, tr("pack-copy-link"))
    };
    if ui.add(IconButton::new(icon, &name)).clicked() {
        ui.ctx().copy_text(url.to_owned());
        ui.data_mut(|d| d.insert_temp(id, now));
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs_f64(COPIED_FOR));
    }
}

/// A numbered step: its number, its text, and what it offers under it.
fn step(ui: &mut Ui, n: usize, text: &str, body: impl FnOnce(&mut Ui)) {
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM;
        let number = ui.label(
            RichText::new(format!("{n}."))
                .text_style(label_strong_style())
                .color(color::TEXT_SECONDARY),
        );
        // The texts line up whatever the number's width.
        ui.add_space((14.0 - number.rect.width()).max(0.0));
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = space::XS;
            ui.add(egui::Label::new(RichText::new(text).color(color::TEXT_PRIMARY)).wrap());
            body(ui);
        });
    });
}

/// Shows the template pack dialog held in `state.modal`, and puts the
/// dialog it came from back once it closes.
pub fn show_modal(ctx: &egui::Context, state: &mut AppState) {
    let Some(Modal::TemplatePack(dialog)) = state.modal.take() else {
        return;
    };
    match show(ctx, &dialog) {
        Outcome::Pending => state.modal = Some(Modal::TemplatePack(dialog)),
        Outcome::Cancel => state.modal = Some(dialog.origin.into_modal()),
        Outcome::ChooseFolder => {
            let title = tr!("pack-title", game = game_label(dialog.entry.game));
            match state.dialogs.pick_folder(&title) {
                Some(folder) => accept_folder(ctx, state, dialog, &folder),
                None => state.modal = Some(Modal::TemplatePack(dialog)),
            }
        }
    }
}

/// A folder dropped on the template pack dialog held in `state.modal`.
pub fn dropped(ctx: &egui::Context, state: &mut AppState, folder: &Path) {
    if let Some(Modal::TemplatePack(dialog)) = state.modal.take() {
        accept_folder(ctx, state, dialog, folder);
    }
}

/// Checks `folder`: when it holds the game's pack (or a folder directly
/// inside it does), remembers it and installs the vehicle back in the
/// origin; otherwise says why and keeps the dialog open.
fn accept_folder(
    ctx: &egui::Context,
    state: &mut AppState,
    mut dialog: Box<TemplatePackDialog>,
    folder: &Path,
) {
    let entry = dialog.entry;
    match tp_pack::catalog::find_pack(folder, entry.game) {
        Some(pack) => {
            state
                .prefs
                .set_template_pack(entry.game, Some(pack.clone()));
            let mut origin = dialog.origin;
            origin.start_install(start(ctx, entry, pack));
            state.modal = Some(origin.into_modal());
        }
        None => {
            dialog.refusal = Some(wrong_folder(entry.game));
            state.modal = Some(Modal::TemplatePack(dialog));
        }
    }
}

/// "This folder doesn't hold the ETS2 template pack."
pub fn wrong_folder(game: tp_vehicles::Game) -> String {
    tr!("pack-wrong-folder", game = game_label(game))
}

/// "Installing <vehicle>…" with its progress and Cancel, in the dialog
/// that picked the vehicle; returns whether Cancel was clicked.
pub fn installing_bar(ui: &mut Ui, job: &InstallJob) -> bool {
    let (done, total) = job.progress();
    let text = tr!("official-installing", name = job.entry().name.as_str());
    let mut cancel = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM;
        ui.spinner();
        ui.label(RichText::new(&text).color(color::TEXT_PRIMARY))
            .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            cancel |= ui.add(secondary_button(&tr("button-cancel"))).clicked();
            let fraction = if total == 0 {
                0.0
            } else {
                done as f32 / total as f32
            };
            ui.add(
                egui::ProgressBar::new(fraction)
                    .desired_width(ui.available_width().max(60.0))
                    .desired_height(6.0),
            );
        });
    });
    cancel
}

/// Ends the installation shown by a dialog once its build is done:
/// installs the package and tells how it ended.
pub fn poll(installing: &mut Option<Installing>, library: &mut VehicleLibrary) -> Option<Finished> {
    let job = installing.as_ref()?;
    let outcome = job.poll()?;
    let entry = job.entry();
    *installing = None;
    Some(official::finish(library, entry, outcome))
}

/// Cancels the installation shown by a dialog: nothing is installed.
pub fn cancel(installing: &mut Option<Installing>) {
    if let Some(job) = installing.take() {
        job.cancel();
    }
}
