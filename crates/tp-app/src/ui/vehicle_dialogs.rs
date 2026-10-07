//! Vehicle Library, Update Template, Add Vehicle, Textures and Remove from
//! Project dialogs, and the vehicle list and texture checkboxes shared with
//! the New Project wizard.

use egui::{Align, Checkbox, Layout, RichText, ScrollArea, TextEdit, Ui, WidgetInfo, WidgetType};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::theme::{label_strong_style, title_style};
use tp_ui::tokens::{color, space};
use tp_ui::widgets::{EmptyState, IconButton, primary_button, secondary_button};
use tp_vehicles::{Game, Kind, Manifest, Package, Part};

use crate::state::AppState;
use crate::vehicle_project::{TextureChange, UpdatePlan, default_textures, is_always_painted};
use crate::vehicles::{InstalledVehicle, VehicleLibrary};

/// Search and filters of a vehicle list.
#[derive(Clone, Debug, Default)]
pub struct VehicleFilter {
    pub query: String,
    pub game: Option<Game>,
    pub kind: Option<Kind>,
    /// The game is set by the project: no game filter is shown.
    pub game_locked: bool,
}

impl VehicleFilter {
    pub fn matches(&self, m: &Manifest) -> bool {
        let q = self.query.trim().to_lowercase();
        (q.is_empty() || m.name.to_lowercase().contains(&q) || m.brand.to_lowercase().contains(&q))
            && self.game.is_none_or(|g| g == m.game.id)
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
            if !self.game_locked {
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
            }
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

/// Supported game versions: "any version" when unlimited.
pub fn versions_text(versions: &semver::VersionReq) -> String {
    if *versions == semver::VersionReq::STAR {
        tr("vehicles-any-version")
    } else {
        versions.to_string()
    }
}

/// "Volvo · Truck · ETS2 · 1.3.0".
pub fn summary(m: &Manifest) -> String {
    format!(
        "{} · {} · {} · {}",
        m.brand,
        kind_label(m.kind),
        game_label(m.game.id),
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

/// A vehicle and the textures checked for it.
#[derive(Clone, Debug, PartialEq)]
pub struct VehicleChoice {
    pub id: String,
    pub version: semver::Version,
    /// Checked main textures and accessories (part ids).
    pub textures: Vec<String>,
}

impl VehicleChoice {
    /// The newest version of `vehicle` with its default textures checked.
    pub fn of(vehicle: &InstalledVehicle) -> Self {
        let m = &vehicle.newest().manifest;
        Self {
            id: vehicle.id.clone(),
            version: m.version.clone(),
            textures: default_textures(m),
        }
    }

    /// The chosen version's manifest.
    pub fn manifest<'a>(&self, library: &'a VehicleLibrary) -> Option<&'a Manifest> {
        library
            .get(&self.id)?
            .versions
            .iter()
            .find(|v| v.manifest.version == self.version)
            .map(|v| &v.manifest)
    }

    /// Ready to create or add: a main texture painted.
    pub fn is_complete(&self, library: &VehicleLibrary) -> bool {
        self.manifest(library).is_some_and(|m| {
            m.paint_job
                .main
                .iter()
                .any(|p| self.textures.contains(&p.id) || is_always_painted(m, &p.id))
        })
    }

    /// The textures that will be painted, in package order, with their
    /// sizes.
    pub fn painted<'a>(&self, m: &'a Manifest) -> Vec<&'a Part> {
        m.paint_job
            .parts()
            .map(|(_, p)| p)
            .filter(|p| self.textures.contains(&p.id) || is_always_painted(m, &p.id))
            .collect()
    }
}

/// Checkboxes for the main textures and accessories of `m`, keeping the
/// package's order in `chosen`. A single main texture is always painted,
/// and the last checked main texture can't be unchecked.
pub fn texture_checkboxes(ui: &mut Ui, m: &Manifest, chosen: &mut Vec<String>) {
    let job = &m.paint_job;
    let checked_main = job.main.iter().filter(|p| chosen.contains(&p.id)).count();
    let order: Vec<&str> = job.parts().map(|(_, p)| p.id.as_str()).collect();
    let mut toggle = |ui: &mut Ui, part: &Part, locked: bool| {
        let always = is_always_painted(m, &part.id);
        let mut on = always || chosen.contains(&part.id);
        let enabled = !always && !(locked && on);
        let response = ui.add_enabled(enabled, Checkbox::new(&mut on, &part.name));
        response
            .widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, enabled, on, &part.name));
        let response = response.on_hover_text(format!("{0} × {0}", part.texture.size));
        if response.changed() {
            if on {
                chosen.push(part.id.clone());
                chosen.sort_by_key(|id| order.iter().position(|o| o == id));
            } else {
                chosen.retain(|id| *id != part.id);
            }
        }
    };
    let heading = |ui: &mut Ui, text: String| {
        ui.label(RichText::new(text).small().color(color::TEXT_SECONDARY));
    };
    heading(ui, tr("vehicles-main-textures"));
    for part in &job.main {
        toggle(ui, part, checked_main <= 1);
    }
    if !job.accessories.is_empty() {
        heading(ui, tr("vehicles-accessories"));
        for part in &job.accessories {
            toggle(ui, part, false);
        }
    }
}

/// The installed vehicles passing `filter` (minus `exclude`), one row each;
/// the chosen one shows its textures as checkboxes.
pub fn vehicle_list(
    ui: &mut Ui,
    library: &VehicleLibrary,
    filter: &VehicleFilter,
    exclude: &[String],
    choice: &mut Option<VehicleChoice>,
) {
    for vehicle in filtered(library, filter) {
        if exclude.contains(&vehicle.id) {
            continue;
        }
        let m = &vehicle.newest().manifest;
        let selected = choice.as_ref().is_some_and(|c| c.id == vehicle.id);
        let text = format!("{}   {}", m.name, summary(m));
        let row = ui.selectable_label(selected, text);
        row.widget_info(|| WidgetInfo::selected(WidgetType::RadioButton, true, selected, &m.name));
        if row.clicked() && !selected {
            *choice = Some(VehicleChoice::of(vehicle));
        }
        if selected && let Some(c) = choice.as_mut() {
            ui.indent(("textures", &vehicle.id), |ui| {
                texture_checkboxes(ui, m, &mut c.textures);
            });
        }
    }
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

/// Installs the built-in sample vehicles; returns their result lines.
pub fn install_samples(state: &mut AppState) -> Vec<Result<String, String>> {
    install_named(
        state,
        crate::vehicles::SAMPLES
            .iter()
            .map(|s| (s.file.to_owned(), Ok(s.bytes.to_vec())))
            .collect(),
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
    let mut custom_clicked = false;
    let mut new_version = None;
    let mut export = None;
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
                custom_clicked |= ui
                    .add(secondary_button(&tr("custom-open-library")))
                    .clicked();
            });
        } else {
            dialog.filter.show(ui, "library");
            ui.add_space(space::SM);
            ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                for vehicle in filtered(&state.vehicles, &dialog.filter) {
                    let m = &vehicle.newest().manifest;
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&m.name).text_style(label_strong_style()));
                        if tp_pack::custom::is_custom(&vehicle.id) {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let name = tr!("vehicles-new-version-of", name = m.name.as_str());
                                let button = ui.add(secondary_button(&tr("vehicles-new-version")));
                                button.widget_info(|| {
                                    WidgetInfo::labeled(WidgetType::Button, true, &name)
                                });
                                if button.clicked() {
                                    new_version = Some(vehicle.id.clone());
                                }
                            });
                        }
                    });
                    ui.label(
                        RichText::new(summary(m))
                            .small()
                            .color(color::TEXT_SECONDARY),
                    );
                    let main: Vec<&str> =
                        m.paint_job.main.iter().map(|p| p.name.as_str()).collect();
                    ui.label(
                        RichText::new(tr!(
                            "vehicles-details",
                            versions = versions_text(&m.game.versions),
                            main = main.join(", "),
                            accessories = m.paint_job.accessories.len()
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
                            let name = tr!(
                                "vehicles-export-version",
                                name = m.name.as_str(),
                                version = version.as_str()
                            );
                            if ui.add(IconButton::new(icons::EXPORT, &name)).clicked() {
                                export = Some((
                                    v.path.clone(),
                                    format!("{}-{version}.{}", vehicle.id, tp_vehicles::EXTENSION),
                                ));
                            }
                        });
                    }
                    ui.separator();
                }
            });
            ui.label(
                RichText::new(tr("custom-scs-reminder"))
                    .small()
                    .color(color::TEXT_SECONDARY),
            );
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
                    if !state.vehicles.vehicles().is_empty() {
                        custom_clicked |= ui
                            .add(secondary_button(&tr("custom-open-library")))
                            .clicked();
                    }
                }
            }
        });
    });
    if let Some(r) = remove {
        dialog.confirm_remove = Some(r);
    }
    if sample_clicked {
        dialog.messages = install_samples(state);
    }
    if let Some((from, suggested)) = export
        && let Some(to) = state.dialogs.save_package(&suggested)
    {
        let written = std::fs::read(&from)
            .map_err(|e| e.to_string())
            .and_then(|bytes| tp_file::write_atomic(&to, &bytes).map_err(|e| e.to_string()));
        let file = to.display().to_string();
        dialog.messages = vec![match written {
            Ok(()) => Ok(tr!("vehicles-exported", file = file.as_str())),
            Err(reason) => Err(tr!(
                "vehicles-export-failed",
                file = file.as_str(),
                reason = reason.as_str()
            )),
        }];
    }
    if custom_clicked {
        super::custom_vehicle::open(
            state,
            super::custom_vehicle::Origin::Library(std::mem::take(dialog)),
        );
        return true;
    }
    if let Some(id) = new_version {
        super::custom_vehicle::open_new_version(state, &id, std::mem::take(dialog));
        return true;
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
    /// New textures checked to be added.
    pub added: Vec<String>,
}

impl UpdateDialog {
    /// The dialog for `plan`, with the new textures checked by default.
    pub fn new(package: Package, plan: UpdatePlan) -> Self {
        let added = plan
            .new
            .iter()
            .filter(|n| n.checked_by_default())
            .map(|n| n.id.clone())
            .collect();
        Self {
            package: Box::new(package),
            plan,
            added,
        }
    }
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
        TextureChange::Removed { name } => tr!("update-removed", name = name.as_str()),
    }
}

/// Shows the Update Template dialog; returns false once it is closed.
pub fn update(ctx: &egui::Context, state: &mut AppState, dialog: &mut UpdateDialog) -> bool {
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
        if !dialog.plan.new.is_empty() {
            ui.add_space(space::SM);
            ui.label(RichText::new(tr("update-new-textures")).small());
            for new in &dialog.plan.new {
                let mut on = new.always || dialog.added.contains(&new.id);
                let response = ui.add_enabled(!new.always, Checkbox::new(&mut on, &new.name));
                response.widget_info(|| {
                    WidgetInfo::selected(WidgetType::Checkbox, !new.always, on, &new.name)
                });
                if response.changed() {
                    if on {
                        dialog.added.push(new.id.clone());
                    } else {
                        dialog.added.retain(|id| *id != new.id);
                    }
                }
            }
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
            ws.apply_update(&dialog.package, &dialog.added, now);
        }
        keep = false;
    }
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        keep = false;
    }
    keep
}

/// State of the Add Vehicle dialog.
#[derive(Clone, Debug, Default)]
pub struct AddVehicleDialog {
    pub filter: VehicleFilter,
    pub choice: Option<VehicleChoice>,
    pub messages: Vec<Result<String, String>>,
}

impl AddVehicleDialog {
    /// The dialog for a project of `game`.
    pub fn for_game(game: Option<&str>) -> Self {
        let game = match game {
            Some("ats") => Some(Game::Ats),
            Some(_) => Some(Game::Ets2),
            None => None,
        };
        Self {
            filter: VehicleFilter {
                game,
                game_locked: game.is_some(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/// Shows the Add Vehicle dialog; returns false once it is closed.
pub fn add_vehicle(
    ctx: &egui::Context,
    state: &mut AppState,
    dialog: &mut AddVehicleDialog,
) -> bool {
    let mut keep = true;
    let mut add = false;
    let mut install_clicked = false;
    let mut custom_clicked = false;
    let exclude: Vec<String> = state
        .workspace()
        .map(|ws| {
            ws.project
                .vehicles
                .iter()
                .map(|v| v.package_id.clone())
                .collect()
        })
        .unwrap_or_default();
    super::dialogs::modal("add_vehicle_modal").show(ctx, |ui| {
        ui.set_width(560.0);
        ui.label(
            RichText::new(tr("cmd-add-vehicle").trim_end_matches('…'))
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
        dialog.filter.show(ui, "add_vehicle");
        ui.add_space(space::SM);
        let available = filtered(&state.vehicles, &dialog.filter)
            .iter()
            .any(|v| !exclude.contains(&v.id));
        if available {
            // A fixed height, as in the New Project wizard.
            ScrollArea::vertical()
                .max_height(320.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    vehicle_list(
                        ui,
                        &state.vehicles,
                        &dialog.filter,
                        &exclude,
                        &mut dialog.choice,
                    );
                });
        } else {
            ui.label(
                RichText::new(tr("add-vehicle-none"))
                    .small()
                    .color(color::TEXT_SECONDARY),
            );
        }
        ui.add_space(space::LG);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let ready = dialog
                .choice
                .as_ref()
                .is_some_and(|c| c.is_complete(&state.vehicles));
            add |= ui
                .add_enabled(ready, primary_button(&tr("add-vehicle-add")))
                .clicked();
            keep &= !ui.add(secondary_button(&tr("button-cancel"))).clicked();
            install_clicked |= ui.add(secondary_button(&tr("vehicles-install"))).clicked();
            custom_clicked |= ui.add(secondary_button(&tr("custom-open"))).clicked();
        });
    });
    if custom_clicked {
        super::custom_vehicle::open(
            state,
            super::custom_vehicle::Origin::AddVehicle(std::mem::take(dialog)),
        );
        return true;
    }
    if install_clicked {
        let paths = state.dialogs.pick_packages();
        if !paths.is_empty() {
            dialog.messages = install(state, &paths);
        }
    }
    if add && let Some(choice) = dialog.choice.clone() {
        let now = ctx.input(|i| i.time);
        match state.vehicles.load(&choice.id, &choice.version) {
            Ok(package) => {
                if let Some(ws) = state.workspace_mut() {
                    match ws.add_vehicle(&package, &choice.textures, now) {
                        Ok(()) => keep = false,
                        Err(err) => dialog.messages = vec![Err(fleet_error_message(err))],
                    }
                }
            }
            Err(err) => {
                dialog.messages = vec![Err(crate::vehicles::install_error_message(
                    &err,
                    &format!("{} {}", choice.id, choice.version),
                ))];
            }
        }
    }
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        keep = false;
    }
    keep
}

/// Why a fleet change was refused, in the current language.
pub fn fleet_error_message(err: crate::vehicle_project::FleetError) -> String {
    use crate::vehicle_project::FleetError;
    tr(match err {
        FleetError::OtherGame => "fleet-error-other-game",
        FleetError::AlreadyThere => "fleet-error-already-there",
        FleetError::BadTextures => "fleet-error-textures",
        FleetError::WrongVersion => "fleet-error-version",
        FleetError::LastVehicle => "reason-last-vehicle",
        FleetError::Unknown => "fleet-error-unknown",
    })
}

/// State of the Textures dialog of a project's vehicle.
#[derive(Clone, Debug)]
pub struct TexturesDialog {
    pub package_id: String,
    pub name: String,
    pub version: String,
    /// The textures the vehicle paints (part ids).
    pub checked: Vec<String>,
    /// Textures with artwork about to be removed: asks for confirmation.
    pub confirm: Option<Vec<String>>,
}

impl TexturesDialog {
    pub fn new(project: &tp_core::Project, vehicle: &tp_core::ProjectVehicle) -> Self {
        let checked = project.surfaces[project.vehicle_range(&vehicle.package_id)]
            .iter()
            .filter_map(|s| s.template.as_ref().map(|t| t.texture_id.clone()))
            .collect();
        Self {
            package_id: vehicle.package_id.clone(),
            name: vehicle.name.clone(),
            version: vehicle.version.clone(),
            checked,
            confirm: None,
        }
    }
}

/// Shows the Textures dialog; returns false once it is closed.
pub fn textures(ctx: &egui::Context, state: &mut AppState, dialog: &mut TexturesDialog) -> bool {
    let mut keep = true;
    let mut apply = false;
    let mut update = false;
    let installed = state
        .workspace()
        .and_then(|ws| ws.project.vehicle(&dialog.package_id))
        .and_then(|v| state.vehicles.recorded(v))
        .map(|i| i.manifest.clone());
    super::dialogs::modal("textures_modal").show(ctx, |ui| {
        ui.set_width(440.0);
        ui.label(
            RichText::new(tr!("textures-title", name = dialog.name.as_str()))
                .text_style(title_style())
                .color(color::TEXT_PRIMARY),
        );
        ui.add_space(space::SM);
        match &installed {
            Some(m) => {
                if dialog.confirm.is_none() {
                    texture_checkboxes(ui, m, &mut dialog.checked);
                }
            }
            None => {
                ui.label(
                    RichText::new(tr!(
                        "textures-missing-version",
                        version = dialog.version.as_str()
                    ))
                    .color(color::WARNING),
                );
            }
        }
        if let Some(names) = &dialog.confirm {
            ui.label(
                RichText::new(tr!("textures-remove-confirm", textures = names.join(", ")))
                    .color(color::WARNING),
            );
        }
        ui.add_space(space::LG);
        ui.with_layout(
            Layout::right_to_left(Align::Center),
            |ui| match &installed {
                Some(_) => {
                    let label = if dialog.confirm.is_some() {
                        tr("textures-remove")
                    } else {
                        tr("textures-apply")
                    };
                    apply |= ui.add(primary_button(&label)).clicked();
                    keep &= !ui.add(secondary_button(&tr("button-cancel"))).clicked();
                }
                None => {
                    update |= ui.add(primary_button(&tr("cmd-update-template"))).clicked();
                    keep &= !ui.add(secondary_button(&tr("button-cancel"))).clicked();
                }
            },
        );
    });
    if apply && let Some(m) = &installed {
        // Textures with artwork being removed ask first.
        let removed_with_art: Vec<String> = state
            .workspace()
            .map(|ws| {
                let p = &ws.project;
                p.surfaces[p.vehicle_range(&dialog.package_id)]
                    .iter()
                    .filter(|s| {
                        s.template.as_ref().is_some_and(|t| {
                            m.paint_job.part(&t.texture_id).is_some()
                                && !dialog.checked.contains(&t.texture_id)
                                && !is_always_painted(m, &t.texture_id)
                        }) && !s.objects.is_empty()
                    })
                    .map(|s| s.name.clone())
                    .collect()
            })
            .unwrap_or_default();
        if dialog.confirm.is_none() && !removed_with_art.is_empty() {
            dialog.confirm = Some(removed_with_art);
        } else {
            let now = ctx.input(|i| i.time);
            let loaded = state.vehicles.load(&m.id, &m.version);
            if let (Ok(package), Some(ws)) = (loaded, state.workspace_mut()) {
                let _ = ws.set_textures(&package, &dialog.checked, now);
            }
            keep = false;
        }
    }
    if update {
        state.modal = None;
        state.open_update_dialog(Some(&dialog.package_id));
        return false;
    }
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        if dialog.confirm.is_some() {
            dialog.confirm = None;
        } else {
            keep = false;
        }
    }
    keep
}

/// Asks before removing a vehicle with artwork; returns false once closed.
pub fn remove_vehicle(
    ctx: &egui::Context,
    state: &mut AppState,
    package_id: &str,
    name: &str,
) -> bool {
    let mut keep = true;
    let mut remove = false;
    super::dialogs::modal("remove_vehicle_modal").show(ctx, |ui| {
        ui.set_width(420.0);
        ui.label(
            RichText::new(tr("vehicles-remove-from-project"))
                .text_style(title_style())
                .color(color::TEXT_PRIMARY),
        );
        ui.add_space(space::SM);
        ui.label(tr!("remove-vehicle-confirm", name = name));
        ui.add_space(space::LG);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            remove |= ui.add(primary_button(&tr("vehicles-remove"))).clicked();
            keep &= !ui.add(secondary_button(&tr("button-cancel"))).clicked();
        });
    });
    if remove {
        let now = ctx.input(|i| i.time);
        if let Some(ws) = state.workspace_mut() {
            let _ = ws.remove_vehicle(package_id, now);
        }
        keep = false;
    }
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        keep = false;
    }
    keep
}
