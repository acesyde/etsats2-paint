//! Vehicle panel: the project's vehicle and package version, its textures,
//! the active template's opacity and visibility, and available updates.

use egui::{RichText, Slider, Ui, WidgetInfo, WidgetType};
use tp_core::TemplateStatus;
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::theme::label_strong_style;
use tp_ui::tokens::{color, space};
use tp_ui::widgets::{EmptyState, secondary_button};

use super::PanelEnv;
use crate::commands::CommandId;
use crate::ui::CommandUi;

pub fn show(ui: &mut Ui, cmds: &mut CommandUi<'_>, env: &mut PanelEnv<'_>) {
    let Some(vehicle) = env.ws.project.vehicle.clone() else {
        EmptyState::new(
            icons::VEHICLE,
            &tr("vehicle-panel-none"),
            &tr("vehicle-panel-none-hint"),
        )
        .show(ui);
        ui.vertical_centered(|ui| {
            if ui
                .add(secondary_button(&tr("cmd-vehicle-library")))
                .clicked()
            {
                cmds.push(CommandId::VehicleLibrary);
            }
        });
        return;
    };
    // Vehicle and package.
    ui.label(RichText::new(&vehicle.name).text_style(label_strong_style()));
    let kind = tr(match vehicle.kind.as_str() {
        "trailer" => "vehicles-trailer",
        _ => "vehicles-truck",
    });
    let game = vehicle.game.to_uppercase();
    let variant = env
        .vehicles
        .get(&vehicle.package_id)
        .and_then(|v| {
            v.versions.iter().find_map(|i| {
                i.manifest
                    .variant(&vehicle.variant_id)
                    .map(|x| x.name.clone())
            })
        })
        .unwrap_or_else(|| vehicle.variant_id.clone());
    ui.label(
        RichText::new(format!("{} · {kind} · {game} · {variant}", vehicle.brand))
            .small()
            .color(color::TEXT_SECONDARY),
    );
    let installed = env.vehicles.get(&vehicle.package_id).and_then(|v| {
        v.versions
            .iter()
            .find(|i| i.manifest.version.to_string() == vehicle.version)
    });
    let package_line = match installed {
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
    if let Some(update) = env.vehicles.update_for(&vehicle) {
        ui.add_space(space::XS);
        ui.label(
            RichText::new(tr!(
                "vehicle-panel-update",
                version = update.manifest.version.to_string()
            ))
            .color(color::ACCENT),
        );
        if ui
            .add(secondary_button(&tr("cmd-update-template")))
            .clicked()
        {
            cmds.push(CommandId::UpdateTemplate);
        }
    }

    // Textures.
    ui.add_space(space::SM);
    ui.label(
        RichText::new(tr("vehicle-panel-textures"))
            .small()
            .color(color::TEXT_SECONDARY),
    );
    let active = env.ws.project.active_surface;
    let rows: Vec<(String, f64, Option<TemplateStatus>)> = env
        .ws
        .project
        .surfaces
        .iter()
        .map(|s| {
            (
                s.name.clone(),
                s.size,
                s.template.as_ref().map(|t| t.status),
            )
        })
        .collect();
    for (i, (name, size, status)) in rows.into_iter().enumerate() {
        ui.horizontal(|ui| {
            let text = format!("{name} · {size} px");
            let row = ui.selectable_label(i == active, text);
            row.widget_info(|| {
                WidgetInfo::selected(
                    WidgetType::SelectableLabel,
                    true,
                    i == active,
                    tr!("vehicle-panel-texture", name = name.as_str()),
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
                            tr!("vehicle-panel-dismiss-named", name = name.as_str()),
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

    // The active texture's template.
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
