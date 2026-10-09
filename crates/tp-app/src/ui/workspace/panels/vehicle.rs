//! The project's fleet as a tree, vehicle › Main textures / Accessories ›
//! texture (the Textures tab); what the Project space shares with it (a
//! vehicle's ⋯ menu, kind and package, a texture's part and flag, the Game
//! versions field); and the active texture's notices shown by the
//! inspector ([`texture_notices`]). The template's visibility and opacity
//! are in the status bar.

use egui::collapsing_header::CollapsingState;
use egui::{
    Align, Align2, Layout, Rect, RichText, Sense, Stroke, StrokeKind, TextEdit, Ui, Vec2,
    WidgetInfo, WidgetType,
};
use tp_core::{ProjectVehicle, TemplateStatus, TexturePart};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::theme::label_strong_style;
use tp_ui::tokens::{color, radius, space};
use tp_ui::widgets::{IconButton, MenuRow, more_menu};

use super::PanelEnv;
use crate::commands::CommandId;
use crate::state::VehicleRequest;
use crate::ui::CommandUi;

/// The fleet tree of the Textures tab: clicking a texture makes it active.
pub fn show(ui: &mut Ui, cmds: &mut CommandUi<'_>, env: &mut PanelEnv<'_>) {
    vehicles_section(ui, cmds, env);
}

/// The game versions the mod is made for, committed on Enter or when the
/// field is left (Escape restores them), with the versions every vehicle
/// supports under it (the Project space's Mod information column).
pub fn game_versions_field(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    use crate::game_versions::{FleetVersions, fleet, parse_list};
    let label = tr("project-game-versions");
    ui.label(RichText::new(&label).small().color(color::TEXT_SECONDARY));
    let id = ui.id().with("game_versions_field");
    let edit_id = id.with("edit");
    let mut buffer = ui
        .data(|d| d.get_temp::<String>(id))
        .unwrap_or_else(|| env.ws.project.game_versions.join(", "));
    tp_ui::widgets::remember_escape(ui, edit_id);
    let response = ui.add(
        TextEdit::singleline(&mut buffer)
            .id(edit_id)
            .hint_text("1.56.*, 1.57.*")
            .desired_width(f32::INFINITY)
            // As tall as the dialogs' fields.
            .margin(egui::Margin::symmetric(10, 0))
            .vertical_align(Align::Center)
            .min_size(egui::vec2(0.0, crate::ui::dialogs::FIELD_HEIGHT)),
    );
    response.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, &label));
    if response.has_focus() {
        ui.data_mut(|d| d.insert_temp(id, buffer.clone()));
    }
    if response.lost_focus() {
        ui.data_mut(|d| d.remove::<String>(id));
        if !tp_ui::widgets::take_escape(ui, edit_id) {
            env.ws.set_game_versions(parse_list(&buffer), env.now);
        }
    }
    let guide = match fleet(&env.ws.project) {
        FleetVersions::NoData => None,
        FleetVersions::Common(common) => Some(tr!(
            "project-game-versions-supported",
            versions = common.text().unwrap_or_else(|| tr("vehicles-any-version"))
        )),
        FleetVersions::Conflict { .. } => Some(tr("project-no-common-version")),
    };
    if let Some(text) = guide {
        let label = ui.label(RichText::new(&text).small().color(color::TEXT_SECONDARY));
        label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
    }
}

/// The fleet: a heading with Add Vehicle (+), then each vehicle with its
/// textures. Thumbnails left stale by an edit are rendered again.
fn vehicles_section(ui: &mut Ui, cmds: &mut CommandUi<'_>, env: &mut PanelEnv<'_>) {
    env.ws
        .thumbnails
        .update(ui.ctx(), &env.ws.project, &env.ws.text.fonts);
    let title = match env.ws.project.game() {
        Some(game) => tr!("vehicle-panel-game", game = game.to_uppercase()),
        None => tr("panel-vehicle"),
    };
    super::list_heading(ui, &title, |ui| {
        let add = tr("cmd-add-vehicle");
        if ui.add(IconButton::new(icons::ADD, &add)).clicked() {
            cmds.push(CommandId::AddVehicle);
        }
    });
    let vehicles = env.ws.project.vehicles.clone();
    let last = vehicles.len() <= 1;
    for vehicle in &vehicles {
        vehicle_group(ui, env, vehicle, last);
    }
}

/// A vehicle: its name groups its textures under Main textures and
/// Accessories; actions in its ⋯ menu, and an update button when a newer
/// version is installed.
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
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = space::XXS;
                vehicle_menu(ui, vehicle, update.is_some(), last, &mut request);
                if let Some(version) = &update {
                    let button = ui.add(
                        egui::Button::new((
                            icons::rich(icons::UPDATE).color(color::SIGNAL),
                            RichText::new(version.as_str()).small().color(color::SIGNAL),
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
                // Kind and package version, on the name's line.
                ui.add_space(space::XS);
                ui.label(
                    RichText::new(format!("{} · {}", kind_text(vehicle), vehicle.version))
                        .size(tp_ui::tokens::typography::CAPTION)
                        .color(color::TEXT_DISABLED),
                )
                .on_hover_text(package_text(env, vehicle));
                // The name takes the room left, on two lines if needed.
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(&vehicle.name).text_style(label_strong_style()),
                        )
                        .wrap(),
                    );
                });
            });
        })
        .body_unindented(|ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            // Surfaces are ordered main textures first, then accessories: a
            // heading before the first of each.
            let mut heading = None;
            for i in env.ws.project.vehicle_range(&vehicle.package_id) {
                let part = part_of(&env.ws.project, i);
                if heading != Some(part) {
                    heading = Some(part);
                    let title = match part {
                        TexturePart::Main => tr("vehicles-main-textures"),
                        TexturePart::Accessory => tr("vehicles-accessories"),
                    };
                    ui.add_space(space::XS);
                    ui.label(RichText::new(title).small().color(color::TEXT_SECONDARY));
                }
                texture_row(ui, env, i);
            }
        });
    ui.add_space(space::SM);
    if request.is_some() {
        *env.vehicle_request = request;
    }
}

/// The ⋯ actions menu of a vehicle (the Textures tab, the Project space's
/// cards): Textures…, Update Template… when a newer version is installed,
/// and Remove from Project, disabled for the `last` vehicle. The chosen
/// action is written to `request`.
pub fn vehicle_menu(
    ui: &mut Ui,
    vehicle: &ProjectVehicle,
    update: bool,
    last: bool,
    request: &mut Option<VehicleRequest>,
) {
    let name = tr!("vehicle-actions-named", name = vehicle.name.as_str());
    more_menu(ui, &name, |ui| {
        if ui.add(MenuRow::new(&tr("vehicles-textures"))).clicked() {
            *request = Some(VehicleRequest::Textures(vehicle.package_id.clone()));
            ui.close();
        }
        if update && ui.add(MenuRow::new(&tr("cmd-update-template"))).clicked() {
            *request = Some(VehicleRequest::Update(vehicle.package_id.clone()));
            ui.close();
        }
        let remove = ui
            .add_enabled(!last, MenuRow::new(&tr("vehicles-remove-from-project")))
            .on_disabled_hover_text(tr("reason-last-vehicle"));
        if remove.clicked() {
            *request = Some(VehicleRequest::Remove(vehicle.package_id.clone()));
            ui.close();
        }
    });
}

/// Whether surface `i` is a main texture or an accessory (a surface with no
/// template is a main texture).
pub fn part_of(project: &tp_core::Project, i: usize) -> TexturePart {
    project.surfaces[i]
        .template
        .as_ref()
        .map_or(TexturePart::Main, |t| t.part)
}

/// "Truck" or "Trailer".
pub fn kind_text(vehicle: &ProjectVehicle) -> String {
    tr(match vehicle.kind.as_str() {
        "trailer" => "vehicles-trailer",
        _ => "vehicles-truck",
    })
}

/// The game versions the recorded package version supports, or that it
/// isn't installed (the tooltip of a vehicle's version).
pub fn package_text(env: &PanelEnv<'_>, vehicle: &ProjectVehicle) -> String {
    match env.vehicles.recorded(vehicle) {
        Some(i) => tr!(
            "vehicle-panel-package",
            version = vehicle.version.as_str(),
            games = crate::ui::vehicle_dialogs::versions_text(&i.manifest.game.versions)
        ),
        None => tr!(
            "vehicle-panel-package-missing",
            version = vehicle.version.as_str()
        ),
    }
}

/// What an update flagged on a texture, if anything: "Layout changed" or
/// "Not in this version".
pub fn flag(status: Option<TemplateStatus>) -> Option<String> {
    match status? {
        TemplateStatus::LayoutChanged => Some(tr("vehicle-panel-layout-changed")),
        TemplateStatus::Removed => Some(tr("vehicle-panel-removed")),
        TemplateStatus::Current => None,
    }
}

/// Side of a texture's thumbnail in a row, in points.
const THUMB: f32 = 26.0;
/// Height of a texture row.
const ROW_HEIGHT: f32 = THUMB + 10.0;

/// A texture: thumbnail of its artwork, name, size and the update's flag;
/// clicking makes it active. The active texture's row is filled and its
/// name drawn in ink, as in the mockup.
fn texture_row(ui: &mut Ui, env: &mut PanelEnv<'_>, i: usize) {
    // No texture is highlighted while a symbol is edited.
    let active = env.ws.project.active_surface == i && !env.ws.is_editing_symbol();
    let surface = &env.ws.project.surfaces[i];
    let name = surface.name.clone();
    let size = format!("{}", surface.size);
    let flag = flag(surface.template.as_ref().map(|t| t.status));
    let label = env
        .ws
        .project
        .surface_names(i)
        .map_or_else(|| name.clone(), |(v, t)| format!("{v} › {t}"));
    let (rect, row) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), ROW_HEIGHT), Sense::click());
    row.widget_info(|| {
        WidgetInfo::selected(
            WidgetType::SelectableLabel,
            true,
            active,
            tr!("vehicle-panel-texture", name = label.as_str()),
        )
    });
    let painter = ui.painter().clone();
    if active {
        painter.rect_filled(rect, radius::MD, color::SURFACE_3);
    } else if row.hovered() {
        painter.rect_filled(rect, radius::MD, color::SURFACE_2);
    }
    tp_ui::widgets::paint_focus_ring(ui, rect, &row, radius::MD);

    // The thumbnail, over the artboard's color (as on the canvas).
    let thumb = Rect::from_min_size(
        egui::pos2(rect.left() + space::XS + 2.0, rect.center().y - THUMB / 2.0),
        Vec2::splat(THUMB),
    );
    painter.rect_filled(thumb, radius::SM, tp_ui::tokens::canvas::ARTBOARD);
    if let Some(texture) = env.ws.thumbnails.texture(i) {
        painter.image(
            texture.id(),
            thumb,
            Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }
    painter.rect_stroke(
        thumb,
        radius::SM,
        Stroke::new(1.0, color::OUTLINE),
        StrokeKind::Outside,
    );

    // Size and flag at the right end, the name in between.
    let mono = egui::TextStyle::Monospace.resolve(ui.style());
    let size_rect = painter.text(
        egui::pos2(rect.right() - space::SM, rect.center().y),
        Align2::RIGHT_CENTER,
        &size,
        egui::FontId::new(mono.size - 1.0, mono.family),
        color::TEXT_DISABLED,
    );
    let mut right = size_rect.left() - space::SM;
    if let Some(flag) = &flag {
        let icon =
            Rect::from_center_size(egui::pos2(right - 8.0, rect.center().y), Vec2::splat(16.0));
        painter.text(
            icon.center(),
            Align2::CENTER_CENTER,
            icons::WARNING,
            icons::font(14.0),
            color::WARNING,
        );
        ui.interact(icon, row.id.with("flag"), Sense::hover())
            .on_hover_text(flag.as_str())
            .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, flag.as_str()));
        right = icon.left() - space::XS;
    }
    let name_rect = Rect::from_min_max(
        egui::pos2(thumb.right() + space::SM, rect.top()),
        egui::pos2(right, rect.bottom()),
    );
    painter.with_clip_rect(name_rect).text(
        egui::pos2(name_rect.left(), rect.center().y),
        Align2::LEFT_CENTER,
        &name,
        egui::FontId::proportional(tp_ui::tokens::typography::CONTROL),
        if active {
            color::TEXT_PRIMARY
        } else {
            color::TEXT_SECONDARY
        },
    );
    if row.clicked() {
        env.ws.set_active_surface(i);
    }
}

/// What an update flagged on the active texture, in the inspector when
/// nothing is selected: "Layout changed" with Dismiss, or "Not in this
/// version".
pub fn texture_notices(ui: &mut Ui, env: &mut PanelEnv<'_>) {
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
    match template.status {
        TemplateStatus::LayoutChanged => {
            ui.add_space(space::SM);
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
            ui.add_space(space::SM);
            ui.label(
                RichText::new(tr!("texture-not-in-version", version = version.as_str()))
                    .small()
                    .color(color::TEXT_SECONDARY),
            );
        }
        TemplateStatus::Current => {}
    }
}
