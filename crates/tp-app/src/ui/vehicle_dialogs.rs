//! Vehicle Library and Update Template dialogs, and the vehicle list shared
//! with the New Project wizard.

use egui::{Align, Layout, RichText, ScrollArea, TextEdit, Ui, WidgetInfo, WidgetType};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::theme::{label_strong_style, title_style};
use tp_ui::tokens::{color, space};
use tp_ui::widgets::{EmptyState, IconButton, primary_button, secondary_button};
use tp_vehicles::{Game, Kind, Manifest, Package};

use crate::state::AppState;
use crate::vehicle_project::{TextureChange, UpdatePlan};
use crate::vehicles::{InstalledVehicle, VehicleLibrary};

/// Search and filters of a vehicle list.
#[derive(Clone, Debug, Default)]
pub struct VehicleFilter {
    pub query: String,
    pub game: Option<Game>,
    pub kind: Option<Kind>,
}

impl VehicleFilter {
    pub fn matches(&self, m: &Manifest) -> bool {
        let q = self.query.trim().to_lowercase();
        (q.is_empty() || m.name.to_lowercase().contains(&q) || m.brand.to_lowercase().contains(&q))
            && self.game.is_none_or(|g| g == m.game)
            && self.kind.is_none_or(|k| k == m.kind)
    }

    /// Search field and game / kind filters.
    pub fn show(&mut self, ui: &mut Ui, id: &str) {
        ui.horizontal(|ui| {
            let search = ui.add(
                TextEdit::singleline(&mut self.query)
                    .hint_text(format!("{} {}", icons::SEARCH, tr("vehicles-search")))
                    .desired_width(200.0),
            );
            search.widget_info(|| {
                WidgetInfo::labeled(WidgetType::TextEdit, true, tr("vehicles-search"))
            });
            let game_name = |g: Option<Game>| match g {
                None => tr("vehicles-all-games"),
                Some(Game::Ets2) => "Euro Truck Simulator 2".to_owned(),
                Some(Game::Ats) => "American Truck Simulator".to_owned(),
            };
            let combo = egui::ComboBox::from_id_salt((id, "game"))
                .selected_text(game_name(self.game))
                .show_ui(ui, |ui| {
                    for g in [None, Some(Game::Ets2), Some(Game::Ats)] {
                        ui.selectable_value(&mut self.game, g, game_name(g));
                    }
                });
            combo.response.widget_info(|| {
                WidgetInfo::labeled(WidgetType::ComboBox, true, tr("vehicles-game-filter"))
            });
            let kind_name = |k: Option<Kind>| match k {
                None => tr("vehicles-all-kinds"),
                Some(k) => kind_label(k),
            };
            let combo = egui::ComboBox::from_id_salt((id, "kind"))
                .selected_text(kind_name(self.kind))
                .show_ui(ui, |ui| {
                    for k in [None, Some(Kind::Truck), Some(Kind::Trailer)] {
                        ui.selectable_value(&mut self.kind, k, kind_name(k));
                    }
                });
            combo.response.widget_info(|| {
                WidgetInfo::labeled(WidgetType::ComboBox, true, tr("vehicles-kind-filter"))
            });
        });
    }
}

pub fn kind_label(kind: Kind) -> String {
    tr(match kind {
        Kind::Truck => "vehicles-truck",
        Kind::Trailer => "vehicles-trailer",
    })
}

pub fn game_label(game: Game) -> &'static str {
    match game {
        Game::Ets2 => "ETS2",
        Game::Ats => "ATS",
    }
}

/// "Volvo · Truck · ETS2 · 1.3.0".
pub fn summary(m: &Manifest) -> String {
    format!(
        "{} · {} · {} · {}",
        m.brand,
        kind_label(m.kind),
        game_label(m.game),
        m.version
    )
}

/// The installed vehicles passing `filter`.
pub fn filtered<'a>(
    library: &'a VehicleLibrary,
    filter: &VehicleFilter,
) -> Vec<&'a InstalledVehicle> {
    library
        .vehicles()
        .iter()
        .filter(|v| filter.matches(&v.newest().manifest))
        .collect()
}

/// Installs `paths`; returns one line per file (success or why it failed).
pub fn install(state: &mut AppState, paths: &[std::path::PathBuf]) -> Vec<Result<String, String>> {
    let files = paths
        .iter()
        .map(|path| {
            let name = path.file_name().map_or_else(
                || path.display().to_string(),
                |n| n.to_string_lossy().into_owned(),
            );
            (name, std::fs::read(path).map_err(|e| e.to_string()))
        })
        .collect();
    install_named(state, files)
}

/// Installs files given by name and content.
pub fn install_named(
    state: &mut AppState,
    files: Vec<(String, Result<Vec<u8>, String>)>,
) -> Vec<Result<String, String>> {
    files
        .into_iter()
        .map(|(file, bytes)| {
            let result = match bytes {
                Ok(bytes) => state.vehicles.install_bytes(&bytes),
                Err(e) => Err(crate::vehicles::InstallError::Io(std::io::Error::other(e))),
            };
            match result {
                Ok(m) => Ok(tr!(
                    "vehicles-installed",
                    name = m.name.as_str(),
                    version = m.version.to_string()
                )),
                Err(err) => Err(crate::vehicles::install_error_message(&err, &file)),
            }
        })
        .collect()
}

/// Installs the built-in sample vehicle; returns its result line.
pub fn install_sample(state: &mut AppState) -> Vec<Result<String, String>> {
    install_named(
        state,
        vec![(
            crate::vehicles::SAMPLE_FILE.to_owned(),
            Ok(crate::vehicles::SAMPLE.to_vec()),
        )],
    )
}

/// State of the Vehicle Library dialog.
#[derive(Clone, Debug, Default)]
pub struct LibraryDialog {
    pub filter: VehicleFilter,
    /// Version waiting for removal confirmation: id, version, name.
    pub confirm_remove: Option<(String, semver::Version, String)>,
    /// Results of the last installation.
    pub messages: Vec<Result<String, String>>,
}

/// Shows the Vehicle Library dialog; returns false once it is closed.
pub fn library(ctx: &egui::Context, state: &mut AppState, dialog: &mut LibraryDialog) -> bool {
    let mut keep = true;
    let mut install_clicked = false;
    let mut sample_clicked = false;
    let mut remove = None;
    super::dialogs::modal("vehicle_library_modal").show(ctx, |ui| {
        ui.set_width(640.0);
        ui.label(
            RichText::new(tr("cmd-vehicle-library").trim_end_matches('…'))
                .text_style(title_style())
                .color(color::TEXT_PRIMARY),
        );
        ui.add_space(space::SM);
        for message in &dialog.messages {
            let (text, tint) = match message {
                Ok(text) => (text, color::SUCCESS),
                Err(text) => (text, color::ERROR),
            };
            ui.label(RichText::new(text).small().color(tint));
        }
        if state.vehicles.vehicles().is_empty() {
            EmptyState::new(
                icons::VEHICLE,
                &tr("vehicles-empty"),
                &tr("vehicles-empty-hint"),
            )
            .show(ui);
            ui.vertical_centered(|ui| {
                sample_clicked |= ui
                    .add(primary_button(&tr("vehicles-install-sample")))
                    .clicked();
            });
        } else {
            dialog.filter.show(ui, "library");
            ui.add_space(space::SM);
            ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                for vehicle in filtered(&state.vehicles, &dialog.filter) {
                    let m = &vehicle.newest().manifest;
                    ui.label(RichText::new(&m.name).text_style(label_strong_style()));
                    ui.label(
                        RichText::new(summary(m))
                            .small()
                            .color(color::TEXT_SECONDARY),
                    );
                    let variants: Vec<&str> = m.variants.iter().map(|v| v.name.as_str()).collect();
                    ui.label(
                        RichText::new(tr!(
                            "vehicles-details",
                            versions = m.game_versions.to_string(),
                            variants = variants.join(", ")
                        ))
                        .small()
                        .color(color::TEXT_SECONDARY),
                    );
                    for v in &vehicle.versions {
                        ui.horizontal(|ui| {
                            let version = v.manifest.version.to_string();
                            ui.label(
                                RichText::new(tr!("vehicles-version", version = version.as_str()))
                                    .small(),
                            );
                            let name = tr!(
                                "vehicles-remove-version",
                                name = m.name.as_str(),
                                version = version.as_str()
                            );
                            if ui.add(IconButton::new(icons::REMOVE, &name)).clicked() {
                                remove = Some((
                                    vehicle.id.clone(),
                                    v.manifest.version.clone(),
                                    format!("{} {version}", m.name),
                                ));
                            }
                        });
                    }
                    ui.separator();
                }
            });
        }
        if let Some((_, _, name)) = &dialog.confirm_remove {
            ui.add_space(space::SM);
            ui.label(
                RichText::new(tr!("vehicles-remove-confirm", name = name.as_str()))
                    .color(color::WARNING),
            );
        }
        ui.add_space(space::LG);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            match &dialog.confirm_remove {
                Some(_) => {
                    if ui.add(primary_button(&tr("vehicles-remove"))).clicked()
                        && let Some((id, version, _)) = dialog.confirm_remove.take()
                        && let Err(err) = state.vehicles.remove(&id, &version)
                    {
                        dialog.messages = vec![Err(err.to_string())];
                    }
                    if ui.add(secondary_button(&tr("button-cancel"))).clicked() {
                        dialog.confirm_remove = None;
                    }
                }
                None => {
                    keep &= !ui.add(primary_button(&tr("button-close"))).clicked();
                    install_clicked |= ui.add(secondary_button(&tr("vehicles-install"))).clicked();
                }
            }
        });
    });
    if let Some(r) = remove {
        dialog.confirm_remove = Some(r);
    }
    if sample_clicked {
        dialog.messages = install_sample(state);
    }
    if install_clicked {
        let paths = state.dialogs.pick_packages();
        if !paths.is_empty() {
            dialog.messages = install(state, &paths);
        }
    }
    let escape = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
    if escape {
        if dialog.confirm_remove.is_some() {
            dialog.confirm_remove = None;
        } else {
            keep = false;
        }
    }
    keep
}

/// State of the Update Template dialog.
#[derive(Clone, Debug)]
pub struct UpdateDialog {
    pub package: Box<Package>,
    pub plan: UpdatePlan,
}

/// One line describing a change.
pub fn change_text(change: &TextureChange) -> String {
    match change {
        TextureChange::Replaced {
            name,
            layout_changed,
            resized,
        } => match (layout_changed, resized) {
            (_, Some((old, new))) => tr!(
                "update-resized",
                name = name.as_str(),
                old = *old,
                new = *new
            ),
            (true, None) => tr!("update-layout-changed", name = name.as_str()),
            (false, None) => tr!("update-replaced", name = name.as_str()),
        },
        TextureChange::Added { name } => tr!("update-added", name = name.as_str()),
        TextureChange::Removed { name } => tr!("update-removed", name = name.as_str()),
    }
}

/// Shows the Update Template dialog; returns false once it is closed.
pub fn update(ctx: &egui::Context, state: &mut AppState, dialog: &UpdateDialog) -> bool {
    let mut keep = true;
    let mut apply = false;
    super::dialogs::modal("update_template_modal").show(ctx, |ui| {
        ui.set_width(480.0);
        ui.label(
            RichText::new(tr("undo-update-template"))
                .text_style(title_style())
                .color(color::TEXT_PRIMARY),
        );
        ui.add_space(space::SM);
        ui.label(tr!(
            "update-versions",
            name = dialog.package.manifest.name.as_str(),
            from = dialog.plan.from.as_str(),
            to = dialog.plan.to.as_str()
        ));
        ui.add_space(space::SM);
        for change in &dialog.plan.changes {
            ui.label(RichText::new(format!("• {}", change_text(change))).small());
        }
        ui.add_space(space::SM);
        ui.label(
            RichText::new(tr("update-artwork-kept"))
                .small()
                .color(color::TEXT_SECONDARY),
        );
        ui.add_space(space::LG);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            apply |= ui.add(primary_button(&tr("update-apply"))).clicked();
            keep &= !ui.add(secondary_button(&tr("button-cancel"))).clicked();
        });
    });
    if apply {
        let now = ctx.input(|i| i.time);
        if let Some(ws) = state.workspace_mut() {
            ws.apply_update(&dialog.package, now);
        }
        keep = false;
    }
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        keep = false;
    }
    keep
}
