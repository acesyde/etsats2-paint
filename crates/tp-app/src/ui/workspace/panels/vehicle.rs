//! Vehicles panel: the fleet's vehicles, their variants and textures as a
//! tree, package versions and updates, and the active template's opacity
//! and visibility.

use egui::{CollapsingHeader, RichText, Slider, Ui, WidgetInfo, WidgetType};
use tp_core::{ProjectVehicle, TemplateStatus};
use tp_i18n::tr;
use tp_ui::theme::label_strong_style;
use tp_ui::tokens::{color, space};
use tp_ui::widgets::secondary_button;

use super::PanelEnv;
use crate::commands::CommandId;
use crate::state::VehicleRequest;
use crate::ui::CommandUi;

pub fn show(ui: &mut Ui, cmds: &mut CommandUi<'_>, env: &mut PanelEnv<'_>) {
    if let Some(game) = env.ws.project.game() {
        ui.label(
            RichText::new(tr!("vehicles-fleet-game", game = game.to_uppercase()))
                .small()
                .color(color::TEXT_SECONDARY),
        );
    }
    let vehicles = env.ws.project.vehicles.clone();
    let last = vehicles.len() <= 1;
    for vehicle in &vehicles {
        vehicle_section(ui, env, vehicle, last);
    }
    ui.add_space(space::SM);
    if ui.add(secondary_button(&tr("cmd-add-vehicle"))).clicked() {
        cmds.push(CommandId::AddVehicle);
    }
    template_controls(ui, env);
}

/// A vehicle: its package, update notice, actions and variants.
fn vehicle_section(ui: &mut Ui, env: &mut PanelEnv<'_>, vehicle: &ProjectVehicle, last: bool) {
    let active_vehicle = env
        .ws
        .project
        .vehicle_of(env.ws.project.active_surface)
        .is_some_and(|v| v.package_id == vehicle.package_id);
    let header =
        CollapsingHeader::new(RichText::new(&vehicle.name).text_style(label_strong_style()))
            .id_salt(("fleet_vehicle", &vehicle.package_id))
            .default_open(true)
            .open(active_vehicle.then_some(true));
    header.show(ui, |ui| {
        let kind = tr(match vehicle.kind.as_str() {
            "trailer" => "vehicles-trailer",
            _ => "vehicles-truck",
        });
        ui.label(
            RichText::new(format!("{} · {kind}", vehicle.brand))
                .small()
                .color(color::TEXT_SECONDARY),
        );
        let package_line = match env.vehicles.recorded(vehicle) {
            Some(i) => tr!(
                "vehicle-panel-package",
                version = vehicle.version.as_str(),
                games = i.manifest.game_versions.to_string()
            ),
            None => tr!(
                "vehicle-panel-package-missing",
                version = vehicle.version.as_str()
            ),
        };
        ui.label(
            RichText::new(package_line)
                .small()
                .color(color::TEXT_SECONDARY),
        );
        if let Some(update) = env.vehicles.update_for(vehicle) {
            ui.label(
                RichText::new(tr!(
                    "vehicle-panel-update",
                    version = update.manifest.version.to_string()
                ))
                .color(color::ACCENT),
            );
            let button = ui.add(secondary_button(&tr("cmd-update-template")));
            button.widget_info(|| {
                WidgetInfo::labeled(
                    WidgetType::Button,
                    true,
                    tr!("vehicle-panel-update-named", name = vehicle.name.as_str()),
                )
            });
            if button.clicked() {
                *env.vehicle_request = Some(VehicleRequest::Update(vehicle.package_id.clone()));
            }
        }
        ui.horizontal(|ui| {
            let variants = ui.small_button(tr("vehicles-variants"));
            variants.widget_info(|| {
                WidgetInfo::labeled(
                    WidgetType::Button,
                    true,
                    tr!("vehicles-variants-named", name = vehicle.name.as_str()),
                )
            });
            if variants.clicked() {
                *env.vehicle_request = Some(VehicleRequest::Variants(vehicle.package_id.clone()));
            }
            let remove = ui
                .add_enabled(
                    !last,
                    egui::Button::new(tr("vehicles-remove-from-project")).small(),
                )
                .on_disabled_hover_text(tr("reason-last-vehicle"));
            remove.widget_info(|| {
                WidgetInfo::labeled(
                    WidgetType::Button,
                    !last,
                    tr!("vehicles-remove-named", name = vehicle.name.as_str()),
                )
            });
            if remove.clicked() {
                *env.vehicle_request = Some(VehicleRequest::Remove(vehicle.package_id.clone()));
            }
        });
        for variant in &vehicle.variants {
            let range = env
                .ws
                .project
                .variant_range(&vehicle.package_id, &variant.id);
            let active_variant = range.contains(&env.ws.project.active_surface);
            CollapsingHeader::new(&variant.name)
                .id_salt(("fleet_variant", &vehicle.package_id, &variant.id))
                .default_open(true)
                .open(active_variant.then_some(true))
                .show(ui, |ui| {
                    for i in range {
                        texture_row(ui, env, i);
                    }
                });
        }
    });
}

/// A texture: selects it; shows its update badges.
fn texture_row(ui: &mut Ui, env: &mut PanelEnv<'_>, i: usize) {
    let active = env.ws.project.active_surface == i;
    let surface = &env.ws.project.surfaces[i];
    let name = surface.name.clone();
    let size = surface.size;
    let status = surface.template.as_ref().map(|t| t.status);
    let label = env
        .ws
        .project
        .surface_names(i)
        .map_or_else(|| name.clone(), |(v, x, t)| format!("{v} › {x} › {t}"));
    ui.horizontal(|ui| {
        let row = ui.selectable_label(active, format!("{name} · {size} px"));
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
        match status {
            Some(TemplateStatus::LayoutChanged) => {
                ui.label(
                    RichText::new(tr("vehicle-panel-layout-changed"))
                        .small()
                        .color(color::WARNING),
                );
                let dismiss = ui.small_button(tr("vehicle-panel-dismiss"));
                dismiss.widget_info(|| {
                    WidgetInfo::labeled(
                        WidgetType::Button,
                        true,
                        tr!("vehicle-panel-dismiss-named", name = label.as_str()),
                    )
                });
                if dismiss.clicked() {
                    env.ws.dismiss_layout_change(i, env.now);
                }
            }
            Some(TemplateStatus::Removed) => {
                ui.label(
                    RichText::new(tr("vehicle-panel-removed"))
                        .small()
                        .color(color::TEXT_SECONDARY),
                );
            }
            _ => {}
        }
    });
}

/// The active texture's template opacity and visibility.
fn template_controls(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    let Some(template) = env.ws.project.surface().template.clone() else {
        return;
    };
    if template.status == TemplateStatus::Removed {
        return;
    }
    ui.add_space(space::SM);
    let mut visible = template.visible;
    if ui.checkbox(&mut visible, tr("cmd-show-template")).changed() {
        if let Some(t) = env.ws.project.surface_mut().template.as_mut() {
            t.visible = visible;
        }
        env.ws.settings_changed = true;
    }
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(tr("vehicle-panel-opacity"))
                .small()
                .color(color::TEXT_SECONDARY),
        );
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
    });
}
