//! The Project space: on the left, the Vehicles header and one card per
//! vehicle (its thumbnail, package, cabins, main texture mode, update,
//! actions and textures; clicking a texture shows it in the Workshop); on
//! the right, the Mod information column (the mod's pictures and settings,
//! read only, Edit in Export Mod…, and the editable Game versions field).

use egui::{
    Align, Align2, CentralPanel, Frame, Layout, Margin, Panel, Rect, Response, RichText,
    ScrollArea, Sense, Stroke, StrokeKind, Ui, UiBuilder, Vec2, WidgetInfo, WidgetType,
};
use tp_core::{Project, ProjectVehicle, TexturePart};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::{canvas, color, radius, space, typography};
use tp_ui::widgets::{paint_focus_ring, secondary_button};

use super::super::panels::{PanelEnv, vehicle};
use crate::commands::CommandId;
use crate::layout::Space;
use crate::mod_export::{ICON_SIZE, IMAGE_SIZE};
use crate::state::VehicleRequest;
use crate::ui::CommandUi;

/// Width of the Mod information column.
const COLUMN_WIDTH: f32 = 360.0;
/// Size of a vehicle card's thumbnail.
const VEHICLE_THUMB: Vec2 = Vec2::new(72.0, 48.0);
/// Padding of a vehicle card's header.
const HEADER_PAD: Vec2 = Vec2::new(16.0, 14.0);
/// Width of a texture tile, its padding and its square thumbnail.
const TILE_WIDTH: f32 = 136.0;
const TILE_PAD: f32 = 8.0;
const TILE_THUMB: f32 = TILE_WIDTH - 2.0 * TILE_PAD;
/// Height of a tile's text under the thumbnail: name, kind and size, and
/// the update's flag.
const TILE_TEXT: f32 = 58.0;

pub fn show(ui: &mut Ui, cmds: &mut CommandUi<'_>, env: &mut PanelEnv<'_>) {
    Panel::right("project_mod_information")
        .exact_size(COLUMN_WIDTH)
        .resizable(false)
        .frame(Frame::new().fill(color::SURFACE_1))
        .show(ui, |ui| mod_information(ui, cmds, env));
    CentralPanel::no_frame()
        .frame(Frame::new().fill(color::SURFACE_0))
        .show(ui, |ui| {
            ScrollArea::vertical()
                .id_salt("project_vehicles")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    Frame::new()
                        .inner_margin(Margin::symmetric(space::XXL as i8, 28))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.spacing_mut().item_spacing.y = space::LG + 4.0;
                            vehicles(ui, cmds, env);
                        });
                });
        });
}

/// The Vehicles header, then one card per vehicle in project order.
fn vehicles(ui: &mut Ui, cmds: &mut CommandUi<'_>, env: &mut PanelEnv<'_>) {
    let ctx = ui.ctx().clone();
    env.ws
        .thumbnails
        .update_with_templates(&ctx, &env.ws.project, &env.ws.text.fonts);
    let project = &env.ws.project;
    let counts = tr!(
        "project-vehicles-count",
        vehicles = project.vehicles.len(),
        textures = project.surfaces.len()
    );
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = space::XS + 2.0;
            super::title(ui, &tr("panel-vehicle"));
            ui.label(RichText::new(&counts).color(color::TEXT_SECONDARY));
        });
        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
            cmds.secondary_button(ui, CommandId::AddVehicle, &tr("cmd-add-vehicle"));
        });
    });

    let vehicles = env.ws.project.vehicles.clone();
    let last = vehicles.len() <= 1;
    for v in &vehicles {
        vehicle_card(ui, env, v, last);
    }
}

/// A vehicle's card: its header, a hairline, then its textures.
fn vehicle_card(ui: &mut Ui, env: &mut PanelEnv<'_>, v: &ProjectVehicle, last: bool) {
    let mut request = None;
    Frame::new()
        .fill(color::SURFACE_1)
        .stroke(Stroke::new(1.0, color::BORDER))
        .corner_radius(radius::LG)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.0;
            card_header(ui, env, v, last, &mut request);
            let rect = ui.available_rect_before_wrap();
            ui.painter()
                .hline(rect.x_range(), rect.top(), Stroke::new(1.0, color::BORDER));
            Frame::new()
                .inner_margin(Margin::same(space::LG as i8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    textures(ui, env, v);
                });
        });
    if request.is_some() {
        *env.vehicle_request = request;
    }
}

/// The card's header: thumbnail, name with kind and package, cabins and
/// main texture mode on the left; the update notice and the actions on the
/// right.
fn card_header(
    ui: &mut Ui,
    env: &mut PanelEnv<'_>,
    v: &ProjectVehicle,
    last: bool,
    request: &mut Option<VehicleRequest>,
) {
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), VEHICLE_THUMB.y + 2.0 * HEADER_PAD.y),
        Sense::hover(),
    );
    let inner = rect.shrink2(HEADER_PAD);
    let update = env
        .vehicles
        .update_for(v)
        .map(|u| u.manifest.version.to_string());

    // Actions, right to left; the information gets the room they leave.
    let mut actions = ui.new_child(
        UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::right_to_left(Align::Center)),
    );
    actions.spacing_mut().item_spacing.x = space::SM;
    vehicle::vehicle_menu(&mut actions, v, update.is_some(), last, request);
    let textures = actions.add(secondary_button(&tr("vehicles-textures")));
    textures.widget_info(|| {
        WidgetInfo::labeled(
            WidgetType::Button,
            true,
            tr!("vehicle-textures-named", name = v.name.as_str()),
        )
    });
    if textures.clicked() {
        *request = Some(VehicleRequest::Textures(v.package_id.clone()));
    }
    if let Some(version) = &update {
        let button = actions.add(secondary_button(&tr("cmd-update-template")));
        button.widget_info(|| {
            WidgetInfo::labeled(
                WidgetType::Button,
                true,
                tr!("vehicle-panel-update-named", name = v.name.as_str()),
            )
        });
        if button.clicked() {
            *request = Some(VehicleRequest::Update(v.package_id.clone()));
        }
        update_notice(
            &mut actions,
            &tr!("project-update-available", version = version.as_str()),
        );
    }
    let right = actions.min_rect().left() - space::LG;

    let info_rect = Rect::from_min_max(inner.min, egui::pos2(right.max(inner.left()), inner.max.y));
    let mut info = ui.new_child(
        UiBuilder::new()
            .max_rect(info_rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    info.set_clip_rect(info_rect.intersect(ui.clip_rect()));
    info.spacing_mut().item_spacing.x = space::LG;
    let (thumb, _) = info.allocate_exact_size(VEHICLE_THUMB, Sense::hover());
    if let Some(surface) = first_main(env, v) {
        paint_texture(&info, env, surface, thumb, cover_uv(thumb));
    } else {
        info.painter()
            .rect_filled(thumb, radius::LG, canvas::ARTBOARD);
    }

    // Name, with the kind and the package version under it.
    let kind = tr!(
        "project-kind-package",
        kind = vehicle::kind_text(v),
        version = v.version.as_str()
    );
    let package = vehicle::package_text(env, v);
    block(&mut info, 280.0, |ui| {
        ui.add(
            egui::Label::new(
                RichText::new(&v.name)
                    .size(typography::HEADING)
                    .family(tp_ui::fonts::semibold_family())
                    .color(color::TEXT_PRIMARY),
            )
            .truncate(),
        );
        ui.add(
            egui::Label::new(RichText::new(&kind).small().color(color::TEXT_SECONDARY)).truncate(),
        )
        .on_hover_text(package);
    });

    // Cabins (a truck's main textures and their cabins), then the main
    // texture mode, read only.
    if v.kind != "trailer" {
        let cabins = cabins_text(&env.ws.project, v);
        block(&mut info, 320.0, |ui| {
            caption(ui, &tr("project-cabins"));
            ui.add(egui::Label::new(&cabins).truncate());
        });
    }
    block(&mut info, 220.0, |ui| {
        caption(ui, &tr("project-main-texture"));
        ui.add(egui::Label::new(main_texture_mode(&env.ws.project, v)).truncate());
    });
}

/// Two lines of information, at most `max_width` wide.
fn block(ui: &mut Ui, max_width: f32, body: impl FnOnce(&mut Ui)) {
    ui.scope(|ui| {
        ui.set_max_width(max_width);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = space::XXS + 1.0;
            body(ui);
        });
    });
}

fn caption(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .size(typography::CAPTION)
            .color(color::TEXT_DISABLED),
    );
}

/// "Update 1.3.0 available": an outlined pill in the signal color, with an
/// icon, so that it doesn't rely on color alone.
fn update_notice(ui: &mut Ui, text: &str) -> Response {
    let font = egui::TextStyle::Small.resolve(ui.style());
    let label = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font, color::SIGNAL);
    let icon =
        ui.painter()
            .layout_no_wrap(icons::UPDATE.to_owned(), icons::font(13.0), color::SIGNAL);
    let pad = Vec2::new(space::SM + 2.0, space::XS);
    let gap = space::XS + 2.0;
    let size = Vec2::new(
        2.0 * pad.x + icon.size().x + gap + label.size().x,
        2.0 * pad.y + label.size().y.max(icon.size().y),
    );
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
    let painter = ui.painter();
    painter.rect_stroke(
        rect,
        rect.height() / 2.0,
        Stroke::new(1.0, color::SIGNAL),
        StrokeKind::Inside,
    );
    let icon_pos = egui::pos2(rect.left() + pad.x, rect.center().y - icon.size().y / 2.0);
    let label_pos = egui::pos2(
        icon_pos.x + icon.size().x + gap,
        rect.center().y - label.size().y / 2.0,
    );
    painter.galley(icon_pos, icon, color::SIGNAL);
    painter.galley(label_pos, label, color::SIGNAL);
    response
}

/// The vehicle's first main texture (its thumbnail is the vehicle's).
fn first_main(env: &PanelEnv<'_>, v: &ProjectVehicle) -> Option<usize> {
    let project = &env.ws.project;
    let mut range = project.vehicle_range(&v.package_id);
    range
        .clone()
        .find(|i| vehicle::part_of(project, *i) == TexturePart::Main)
        .or_else(|| range.next())
}

/// A truck's main textures, each followed by its cabins' internal names
/// when the package gives them: "Standard cab (standard), High roof
/// (high_roof)".
fn cabins_text(project: &Project, v: &ProjectVehicle) -> String {
    project
        .vehicle_range(&v.package_id)
        .filter(|i| vehicle::part_of(project, *i) == TexturePart::Main)
        .map(|i| {
            let surface = &project.surfaces[i];
            let cabins = surface
                .template
                .as_ref()
                .map(|t| t.game_ids.join(", "))
                .unwrap_or_default();
            if cabins.is_empty() {
                surface.name.clone()
            } else {
                format!("{} ({cabins})", surface.name)
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The main texture mode given by the package: one per cabin layout when
/// it has several main textures, one for every cabin for a truck with a
/// single one, a single main texture for a trailer.
fn main_texture_mode(project: &Project, v: &ProjectVehicle) -> String {
    if v.kind == "trailer" {
        return tr("project-mode-single");
    }
    // Files written before the game data was recorded: the main textures
    // the project paints.
    let count = v.game_data.as_ref().map_or_else(
        || {
            project
                .vehicle_range(&v.package_id)
                .filter(|i| vehicle::part_of(project, *i) == TexturePart::Main)
                .count()
        },
        |g| g.main_count,
    );
    tr(if count > 1 {
        "project-mode-per-layout"
    } else {
        "project-mode-every-cabin"
    })
}

/// A texture's kind and size: "Main · 4096", "Accessory · 1024".
fn texture_detail(project: &Project, i: usize) -> String {
    let kind = tr(match vehicle::part_of(project, i) {
        TexturePart::Main => "project-texture-main",
        TexturePart::Accessory => "project-texture-accessory",
    });
    format!("{kind} · {}", project.surfaces[i].size)
}

/// The part of a square texture that covers `rect` (centered).
fn cover_uv(rect: Rect) -> Rect {
    let aspect = rect.aspect_ratio();
    let (w, h) = if aspect >= 1.0 {
        (1.0, 1.0 / aspect)
    } else {
        (aspect, 1.0)
    };
    Rect::from_center_size(egui::pos2(0.5, 0.5), Vec2::new(w, h))
}

/// Paints the `uv` part of texture `surface`: its artwork over its
/// template, on the artboard's color.
fn paint_texture(ui: &Ui, env: &PanelEnv<'_>, surface: usize, rect: Rect, uv: Rect) {
    let painter = ui.painter();
    painter.rect_filled(rect, radius::SM, canvas::ARTBOARD);
    let template = env.ws.project.surfaces[surface]
        .template
        .as_ref()
        .and_then(|t| env.ws.thumbnails.template(t.asset));
    for texture in [template, env.ws.thumbnails.texture(surface)]
        .into_iter()
        .flatten()
    {
        painter.image(texture.id(), rect, uv, egui::Color32::WHITE);
    }
}

/// The vehicle's textures in project order, under Main textures and
/// Accessories (a heading is left out when it would list nothing).
fn textures(ui: &mut Ui, env: &mut PanelEnv<'_>, v: &ProjectVehicle) {
    let range = env.ws.project.vehicle_range(&v.package_id);
    let mut first = true;
    for part in [TexturePart::Main, TexturePart::Accessory] {
        let items: Vec<usize> = range
            .clone()
            .filter(|i| vehicle::part_of(&env.ws.project, *i) == part)
            .collect();
        if items.is_empty() {
            continue;
        }
        if !first {
            ui.add_space(space::MD);
        }
        first = false;
        let title = tr(match part {
            TexturePart::Main => "vehicles-main-textures",
            TexturePart::Accessory => "vehicles-accessories",
        });
        ui.label(RichText::new(title).small().color(color::TEXT_SECONDARY));
        ui.add_space(space::SM);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(space::MD);
            for i in items {
                texture_tile(ui, env, i);
            }
        });
    }
}

/// A texture: thumbnail of its artwork over its template, name, kind and
/// size, and the update's flag. Clicking it (or Enter / Space) makes it
/// the active texture and shows the Workshop. The active texture's tile is
/// outlined, marked by an indicator bar and its thumbnail ringed.
fn texture_tile(ui: &mut Ui, env: &mut PanelEnv<'_>, i: usize) {
    // No texture is highlighted while a symbol is edited.
    let active = env.ws.project.active_surface == i && !env.ws.is_editing_symbol();
    let project = &env.ws.project;
    let surface = &project.surfaces[i];
    let name = surface.name.clone();
    let detail = texture_detail(project, i);
    let flag = vehicle::flag(surface.template.as_ref().map(|t| t.status));
    let label = project
        .surface_names(i)
        .map_or_else(|| name.clone(), |(v, t)| format!("{v} › {t}"));

    let size = Vec2::new(TILE_WIDTH, TILE_PAD + TILE_THUMB + TILE_TEXT);
    let (rect, tile) = ui.allocate_exact_size(size, Sense::click());
    tile.widget_info(|| {
        WidgetInfo::selected(
            WidgetType::SelectableLabel,
            true,
            active,
            tr!("vehicle-panel-texture", name = label.as_str()),
        )
    });
    if !ui.is_rect_visible(rect) {
        return;
    }
    let painter = ui.painter().clone();
    if active {
        painter.rect_filled(rect, radius::LG, color::SURFACE_3);
        painter.rect_stroke(
            rect,
            radius::LG,
            Stroke::new(1.0, color::BORDER_STRONG),
            StrokeKind::Inside,
        );
    } else if tile.hovered() {
        painter.rect_filled(rect, radius::LG, color::SURFACE_2);
        painter.rect_stroke(
            rect,
            radius::LG,
            Stroke::new(1.0, color::BORDER_STRONG),
            StrokeKind::Inside,
        );
    }
    paint_focus_ring(ui, rect, &tile, radius::LG);

    let thumb = Rect::from_min_size(rect.min + Vec2::splat(TILE_PAD), Vec2::splat(TILE_THUMB));
    paint_texture(
        ui,
        env,
        i,
        thumb,
        Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
    );
    let (ring, width) = if active {
        (color::ACCENT_PRIMARY, 1.5)
    } else {
        (color::BORDER, 1.0)
    };
    painter.rect_stroke(
        thumb,
        radius::SM,
        Stroke::new(width, ring),
        StrokeKind::Outside,
    );

    // Name, kind and size, then the flag.
    let text = Rect::from_min_max(
        egui::pos2(thumb.left() + 2.0, thumb.bottom() + space::SM),
        egui::pos2(thumb.right(), rect.bottom() - TILE_PAD),
    );
    let clipped = painter.with_clip_rect(text.intersect(ui.clip_rect()));
    let body = egui::TextStyle::Body.resolve(ui.style());
    let line = body.size + space::XS;
    clipped.text(
        text.left_top(),
        Align2::LEFT_TOP,
        &name,
        body,
        if active {
            color::TEXT_PRIMARY
        } else {
            color::TEXT_SECONDARY
        },
    );
    clipped.text(
        text.left_top() + Vec2::new(0.0, line),
        Align2::LEFT_TOP,
        &detail,
        egui::FontId::monospace(typography::CAPTION),
        color::TEXT_DISABLED,
    );
    if let Some(flag) = &flag {
        let top = text.top() + 2.0 * line;
        let icon = clipped.text(
            egui::pos2(text.left(), top),
            Align2::LEFT_TOP,
            icons::WARNING,
            icons::font(typography::CAPTION + 1.0),
            color::WARNING,
        );
        clipped.text(
            egui::pos2(icon.right() + space::XS, top),
            Align2::LEFT_TOP,
            flag,
            egui::FontId::proportional(typography::CAPTION),
            color::WARNING,
        );
        let area = Rect::from_min_max(egui::pos2(text.left(), top), text.right_bottom());
        ui.interact(area, tile.id.with("flag"), Sense::hover())
            .on_hover_text(flag.as_str())
            .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, flag.as_str()));
    }
    if tile.clicked() {
        env.ws.set_active_surface(i);
        env.ws.space = Space::Workshop;
    }
}

/// The Mod information column: the mod's pictures, its settings read only,
/// Edit in Export Mod…, and the Game versions field with its hint.
fn mod_information(ui: &mut Ui, cmds: &mut CommandUi<'_>, env: &mut PanelEnv<'_>) {
    let ctx = ui.ctx().clone();
    env.ws
        .mod_previews
        .update(&ctx, &env.ws.project, &env.ws.text.fonts);
    let previews = env.ws.mod_previews.textures().cloned();
    let settings = env.ws.project.mod_settings.clone();
    ScrollArea::vertical()
        .id_salt("project_mod_information")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            Frame::new().inner_margin(Margin::same(18)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing.y = space::MD;
                super::caps_heading(ui, &tr("project-mod-information"));
                picture(
                    ui,
                    &tr("mod-icon"),
                    previews.as_ref().map(|p| &p[0]),
                    ICON_SIZE,
                );
                picture(
                    ui,
                    &tr("mod-image"),
                    previews.as_ref().map(|p| &p[1]),
                    IMAGE_SIZE,
                );
                super::read_only(ui, &tr("mod-name"), &settings.name, false, false);
                ui.horizontal_top(|ui| {
                    let gap = space::SM + 2.0;
                    let version_width = 96.0;
                    let author_width = ui.available_width() - version_width - gap;
                    ui.spacing_mut().item_spacing.x = gap;
                    for (label, value, mono, width) in [
                        (tr("mod-author"), &settings.author, false, author_width),
                        (tr("mod-version"), &settings.version, true, version_width),
                    ] {
                        ui.allocate_ui_with_layout(
                            Vec2::new(width, 0.0),
                            Layout::top_down(Align::Min),
                            |ui| {
                                ui.set_width(width);
                                super::read_only(ui, &label, value, mono, false);
                            },
                        );
                    }
                });
                super::read_only(
                    ui,
                    &tr("mod-description"),
                    &settings.description,
                    false,
                    true,
                );
                cmds.secondary_button(ui, CommandId::ExportMod, &tr("project-edit-in-export"));
                ui.add_space(space::SM);
                vehicle::game_versions_field(ui, env);
            });
        });
}

/// A picture of the mod at its size in pixels, under its title; an empty
/// box until it is rendered (no spinner: the render wakes the UI when it
/// ends, nothing needs to repaint meanwhile).
fn picture(ui: &mut Ui, title: &str, texture: Option<&egui::TextureHandle>, size: (u32, u32)) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = space::XS;
        super::field_label(ui, title);
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(size.0 as f32, size.1 as f32), Sense::hover());
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Image, true, title));
        match texture {
            Some(texture) => {
                ui.painter().image(
                    texture.id(),
                    rect,
                    Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
            None => {
                ui.painter().rect_filled(rect, radius::SM, color::SURFACE_2);
            }
        }
        ui.painter().rect_stroke(
            rect,
            radius::SM,
            Stroke::new(1.0, color::BORDER_STRONG),
            StrokeKind::Outside,
        );
    });
}

#[cfg(test)]
mod tests {
    use tp_vehicles::Package;

    use super::*;

    /// The sample truck painting both cabins and every accessory.
    fn truck() -> Project {
        let package = Package::read(crate::vehicles::SAMPLES[0].bytes).unwrap();
        let mut textures = crate::vehicle_project::default_textures(&package.manifest);
        textures.push("high_roof".into());
        crate::vehicle_project::fleet_project("F", &package, &textures).unwrap()
    }

    #[test]
    fn a_truck_card_reads_its_cabins_and_mode() {
        let project = truck();
        let v = &project.vehicles[0];
        assert_eq!(
            cabins_text(&project, v),
            "Standard cab (standard), High roof (high_roof)"
        );
        assert_eq!(main_texture_mode(&project, v), "One per cabin layout");
        let mut single = v.clone();
        single.game_data.as_mut().unwrap().main_count = 1;
        assert_eq!(main_texture_mode(&project, &single), "One for every cabin");
        let mut trailer = v.clone();
        trailer.kind = "trailer".into();
        assert_eq!(main_texture_mode(&project, &trailer), "Single main texture");
    }

    #[test]
    fn texture_rows_read_kind_and_size() {
        let project = truck();
        let details: Vec<(String, String)> = (0..project.surfaces.len())
            .map(|i| {
                (
                    project.surfaces[i].name.clone(),
                    texture_detail(&project, i),
                )
            })
            .collect();
        let expected = [
            ("Standard cab", "Main · 4096"),
            ("High roof", "Main · 4096"),
            ("Chassis", "Accessory · 4096"),
            ("Cab accessories", "Accessory · 1024"),
            ("Side skirts", "Accessory · 1024"),
        ];
        assert_eq!(details, expected.map(|(n, d)| (n.to_owned(), d.to_owned())));
    }

    #[test]
    fn cover_keeps_the_middle_of_a_square() {
        let uv = cover_uv(Rect::from_min_size(egui::Pos2::ZERO, VEHICLE_THUMB));
        assert_eq!(uv.width(), 1.0);
        assert!((uv.height() - 48.0 / 72.0).abs() < 1e-6);
        assert!((uv.center().y - 0.5).abs() < 1e-6);
    }
}
