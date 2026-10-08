//! The sidebar's content: the project's properties (read only for now) and
//! its fleet as a tree, vehicle › variant › texture, the only place to
//! switch textures. The active texture's template settings live in the
//! Properties panel ([`texture_section`]).

use egui::collapsing_header::CollapsingState;
use egui::{Align, Layout, RichText, Slider, TextEdit, Ui, WidgetInfo, WidgetType};
use tp_core::{ProjectVehicle, TemplateStatus, TexturePart};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::theme::label_strong_style;
use tp_ui::tokens::{color, space};
use tp_ui::widgets::IconButton;

use super::PanelEnv;
use crate::commands::CommandId;
use crate::state::VehicleRequest;
use crate::ui::CommandUi;

/// The Project and Vehicles sections.
pub fn show(ui: &mut Ui, cmds: &mut CommandUi<'_>, env: &mut PanelEnv<'_>) {
    let name = env.ws.project.name.clone();
    project_section(ui, cmds, &name);
    ui.add_space(space::SM);
    vehicles_section(ui, cmds, env);
}

/// A section title with an action button at its right end.
fn section_header(ui: &mut Ui, title: &str, action: impl FnOnce(&mut Ui)) {
    ui.label(RichText::new(title).text_style(label_strong_style()));
    ui.with_layout(Layout::right_to_left(Align::Center), action);
}

/// Project properties, read only: they become editable with a later
/// feature. Its header holds the button that reduces the sidebar.
fn project_section(ui: &mut Ui, cmds: &mut CommandUi<'_>, name: &str) {
    let id = ui.make_persistent_id("sidebar_project");
    CollapsingState::load_with_default_open(ui.ctx(), id, true)
        .show_header(ui, |ui| {
            section_header(ui, &tr("sidebar-project"), |ui| {
                let hide = tr("sidebar-hide");
                if ui
                    .add(IconButton::new(icons::HIDE_SIDEBAR, &hide))
                    .on_hover_text(&hide)
                    .clicked()
                {
                    cmds.push(CommandId::ToggleVehicles);
                }
            });
        })
        .body(|ui| {
            for (label, value) in [
                (tr("project-name"), name.to_owned()),
                (tr("project-version"), String::new()),
                (tr("project-game-versions"), String::new()),
            ] {
                ui.label(RichText::new(&label).small().color(color::TEXT_SECONDARY));
                let mut text = value;
                let field = ui.add_enabled(
                    false,
                    TextEdit::singleline(&mut text).desired_width(f32::INFINITY),
                );
                field.widget_info(|| WidgetInfo::text_edit(false, String::new(), &text, &label));
            }
        });
}

/// The fleet: each vehicle with its variants and textures.
fn vehicles_section(ui: &mut Ui, cmds: &mut CommandUi<'_>, env: &mut PanelEnv<'_>) {
    let title = match env.ws.project.game() {
        Some(game) => tr!("sidebar-vehicles-game", game = game.to_uppercase()),
        None => tr("panel-vehicle"),
    };
    let id = ui.make_persistent_id("sidebar_vehicles");
    CollapsingState::load_with_default_open(ui.ctx(), id, true)
        .show_header(ui, |ui| {
            section_header(ui, &title, |ui| {
                let add = tr("cmd-add-vehicle");
                if ui
                    .add(IconButton::new(icons::ADD, &add))
                    .on_hover_text(&add)
                    .clicked()
                {
                    cmds.push(CommandId::AddVehicle);
                }
            });
        })
        .body(|ui| {
            let vehicles = env.ws.project.vehicles.clone();
            let last = vehicles.len() <= 1;
            for vehicle in &vehicles {
                vehicle_group(ui, env, vehicle, last);
            }
        });
}

/// A vehicle: its name groups its textures under Main textures and
/// Accessories; actions in its ⋯ menu.
fn vehicle_group(ui: &mut Ui, env: &mut PanelEnv<'_>, vehicle: &ProjectVehicle, last: bool) {
    let id = ui.make_persistent_id(("fleet_vehicle", &vehicle.package_id));
    let active = env
        .ws
        .project
        .vehicle_of(env.ws.project.active_surface)
        .is_some_and(|v| v.package_id == vehicle.package_id);
    let mut state = CollapsingState::load_with_default_open(ui.ctx(), id, true);
    if active {
        state.set_open(true);
    }
    let update = env
        .vehicles
        .update_for(vehicle)
        .map(|u| u.manifest.version.to_string());
    let mut request = None;
    state
        .show_header(ui, |ui| {
            ui.label(RichText::new(&vehicle.name).text_style(label_strong_style()));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let menu = ui.menu_button(icons::rich(icons::MORE), |ui| {
                    if ui.button(tr("vehicles-textures")).clicked() {
                        request = Some(VehicleRequest::Textures(vehicle.package_id.clone()));
                        ui.close();
                    }
                    if update.is_some() && ui.button(tr("cmd-update-template")).clicked() {
                        request = Some(VehicleRequest::Update(vehicle.package_id.clone()));
                        ui.close();
                    }
                    let remove = ui
                        .add_enabled(!last, egui::Button::new(tr("vehicles-remove-from-project")))
                        .on_disabled_hover_text(tr("reason-last-vehicle"));
                    if remove.clicked() {
                        request = Some(VehicleRequest::Remove(vehicle.package_id.clone()));
                        ui.close();
                    }
                });
                menu.response.widget_info(|| {
                    WidgetInfo::labeled(
                        WidgetType::Button,
                        true,
                        tr!("vehicle-actions-named", name = vehicle.name.as_str()),
                    )
                });
                if let Some(version) = &update {
                    let button = ui.add(
                        egui::Button::new((
                            icons::rich(icons::UPDATE).color(color::ACCENT),
                            RichText::new(version.as_str()).small().color(color::ACCENT),
                        ))
                        .small(),
                    );
                    button.widget_info(|| {
                        WidgetInfo::labeled(
                            WidgetType::Button,
                            true,
                            tr!("vehicle-panel-update-named", name = vehicle.name.as_str()),
                        )
                    });
                    if button
                        .on_hover_text(tr!("vehicle-panel-update", version = version.as_str()))
                        .clicked()
                    {
                        request = Some(VehicleRequest::Update(vehicle.package_id.clone()));
                    }
                }
            });
        })
        .body(|ui| {
            let kind = tr(match vehicle.kind.as_str() {
                "trailer" => "vehicles-trailer",
                _ => "vehicles-truck",
            });
            let package = match env.vehicles.recorded(vehicle) {
                Some(i) => tr!(
                    "vehicle-panel-package",
                    version = vehicle.version.as_str(),
                    games = crate::ui::vehicle_dialogs::versions_text(&i.manifest.game.versions)
                ),
                None => tr!(
                    "vehicle-panel-package-missing",
                    version = vehicle.version.as_str()
                ),
            };
            ui.label(
                RichText::new(format!("{kind} · {}", vehicle.version))
                    .small()
                    .color(color::TEXT_SECONDARY),
            )
            .on_hover_text(package);
            // Surfaces are ordered main textures first, then accessories: a
            // heading before the first of each.
            let mut heading = None;
            for i in env.ws.project.vehicle_range(&vehicle.package_id) {
                let part = env.ws.project.surfaces[i]
                    .template
                    .as_ref()
                    .map_or(TexturePart::Main, |t| t.part);
                if heading != Some(part) {
                    heading = Some(part);
                    let title = match part {
                        TexturePart::Main => tr("vehicles-main-textures"),
                        TexturePart::Accessory => tr("vehicles-accessories"),
                    };
                    ui.label(RichText::new(title).small().color(color::TEXT_SECONDARY));
                }
                texture_row(ui, env, i);
            }
        });
    if request.is_some() {
        *env.vehicle_request = request;
    }
}

/// Whether a template needs the painter's attention.
fn flagged(status: Option<TemplateStatus>) -> bool {
    matches!(
        status,
        Some(TemplateStatus::LayoutChanged | TemplateStatus::Removed)
    )
}

/// A texture: name and size; clicking makes it active.
fn texture_row(ui: &mut Ui, env: &mut PanelEnv<'_>, i: usize) {
    // No texture is highlighted while a symbol is edited.
    let active = env.ws.project.active_surface == i && !env.ws.is_editing_symbol();
    let surface = &env.ws.project.surfaces[i];
    let name = surface.name.clone();
    let size = surface.size;
    let warn = flagged(surface.template.as_ref().map(|t| t.status));
    let label = env
        .ws
        .project
        .surface_names(i)
        .map_or_else(|| name.clone(), |(v, t)| format!("{v} › {t}"));
    ui.horizontal(|ui| {
        let row = ui.selectable_label(active, &name);
        row.widget_info(|| {
            WidgetInfo::selected(
                WidgetType::SelectableLabel,
                true,
                active,
                tr!("vehicle-panel-texture", name = label.as_str()),
            )
        });
        if row.clicked() {
            env.ws.set_active_surface(i);
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{size}"))
                    .small()
                    .color(color::TEXT_SECONDARY),
            );
            if warn {
                ui.label(icons::rich(icons::WARNING).color(color::WARNING))
                    .on_hover_text(tr("vehicle-panel-needs-check"));
            }
        });
    });
}

/// The active texture's template, in the Properties panel when nothing is
/// selected: what an update flagged, visibility and opacity.
pub fn texture_section(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    let Some(template) = env.ws.project.surface().template.clone() else {
        return;
    };
    let index = env.ws.project.active_surface;
    let name = env
        .ws
        .project
        .surface_names(index)
        .map_or_else(String::new, |(v, t)| format!("{v} › {t}"));
    let version = env
        .ws
        .project
        .vehicle_of(index)
        .map(|v| v.version.clone())
        .unwrap_or_default();
    ui.add_space(space::SM);
    ui.label(RichText::new(tr("vehicle-panel-opacity")).text_style(label_strong_style()));
    match template.status {
        TemplateStatus::LayoutChanged => {
            ui.label(
                RichText::new(tr!("texture-layout-changed", version = version.as_str()))
                    .small()
                    .color(color::WARNING),
            );
            let dismiss = ui.small_button(tr("vehicle-panel-dismiss"));
            dismiss.widget_info(|| {
                WidgetInfo::labeled(
                    WidgetType::Button,
                    true,
                    tr!("vehicle-panel-dismiss-named", name = name.as_str()),
                )
            });
            if dismiss.clicked() {
                env.ws.dismiss_layout_change(index, env.now);
            }
        }
        TemplateStatus::Removed => {
            ui.label(
                RichText::new(tr!("texture-not-in-version", version = version.as_str()))
                    .small()
                    .color(color::TEXT_SECONDARY),
            );
            return;
        }
        TemplateStatus::Current => {}
    }
    let mut visible = template.visible;
    if ui.checkbox(&mut visible, tr("cmd-show-template")).changed() {
        if let Some(t) = env.ws.project.surface_mut().template.as_mut() {
            t.visible = visible;
        }
        env.ws.settings_changed = true;
    }
    let mut percent = (template.opacity * 100.0).round();
    let slider = ui.add(
        Slider::new(&mut percent, 0.0..=100.0)
            .suffix("%")
            .custom_formatter(|v, _| tp_i18n::format_number(v, 0)),
    );
    slider.widget_info(|| {
        WidgetInfo::labeled(WidgetType::Slider, true, tr("vehicle-panel-opacity-name"))
    });
    if slider.changed() {
        if let Some(t) = env.ws.project.surface_mut().template.as_mut() {
            t.opacity = (percent / 100.0).clamp(0.0, 1.0);
        }
        env.ws.settings_changed = true;
    }
}
