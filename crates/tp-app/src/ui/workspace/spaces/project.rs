//! The Project space: on the left, the Vehicles header (counts, texture
//! filter, Add Vehicle…) and one card per vehicle (its thumbnail, package,
//! cabins, main texture mode, update, actions and textures with their
//! states; clicking a texture shows it in the Workshop); on the right, the
//! Mod information column, where the mod's pictures and settings are
//! edited (each committed edit is one undo step), with the problems that
//! block the export under their fields and in Before exporting.

use egui::{
    Align, Align2, CentralPanel, Frame, Layout, Margin, Panel, Rect, Response, RichText,
    ScrollArea, Sense, Stroke, StrokeKind, Ui, UiBuilder, Vec2, WidgetInfo, WidgetType,
};
use tp_core::{Project, ProjectVehicle, TextureFilter, TexturePart, TextureState};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::{canvas, color, radius, space, typography};
use tp_ui::widgets::{
    FieldEvent, NumericField, SegmentedControl, paint_focus_ring, secondary_button,
};

use super::super::panels::{PanelEnv, vehicle};
use crate::commands::CommandId;
use crate::layout::Space;
use crate::mod_export::{ModField, ModPicture, Problem, ProblemPlace, Warning};
use crate::state::VehicleRequest;
use crate::ui::CommandUi;
use crate::ui::dialogs::TextKind;

/// Width of the Mod information column.
const COLUMN_WIDTH: f32 = 360.0;
/// Size of a vehicle card's thumbnail.
const VEHICLE_THUMB: Vec2 = Vec2::new(72.0, 48.0);
/// Padding of a vehicle card's header.
const HEADER_PAD: Vec2 = Vec2::new(16.0, 14.0);
/// A texture tile's thumbnail (the middle of the square texture), its
/// padding and its width.
const TILE_THUMB: Vec2 = Vec2::new(150.0, 108.0);
const TILE_PAD: f32 = 8.0;
const TILE_WIDTH: f32 = TILE_THUMB.x + 2.0 * TILE_PAD;
/// Height of a tile's text under the thumbnail: name, kind and size, and
/// the state.
const TILE_TEXT: f32 = 68.0;

pub fn show(ui: &mut Ui, cmds: &mut CommandUi<'_>, env: &mut PanelEnv<'_>) {
    // Where Show on a problem leads, taken once.
    let reveal = env.ws.reveal.take();
    Panel::right("project_mod_information")
        .exact_size(COLUMN_WIDTH)
        .resizable(false)
        .frame(Frame::new().fill(color::SURFACE_1))
        .show(ui, |ui| mod_information(ui, env, reveal.as_ref()));
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
                            vehicles(ui, cmds, env, reveal.as_ref());
                        });
                });
        });
}

/// "2 vehicles · 9 textures · 4 modified · 1 to check" (a state's part is
/// left out when it is zero).
fn header_counts(project: &Project) -> String {
    let (mut modified, mut check) = (0, 0);
    for surface in &project.surfaces {
        match surface.state() {
            TextureState::Modified => modified += 1,
            TextureState::ToCheck(_) => check += 1,
            TextureState::Empty => {}
        }
    }
    let counts = tr!(
        "project-vehicles-count",
        vehicles = project.vehicles.len(),
        textures = project.surfaces.len()
    );
    let states = tr!("project-texture-states", modified = modified, check = check);
    format!("{counts}{states}")
}

/// The Vehicles header, then one card per vehicle in project order; the
/// card `reveal` names is scrolled into view.
fn vehicles(
    ui: &mut Ui,
    cmds: &mut CommandUi<'_>,
    env: &mut PanelEnv<'_>,
    reveal: Option<&ProblemPlace>,
) {
    let ctx = ui.ctx().clone();
    env.ws
        .thumbnails
        .update_with_templates(&ctx, &env.ws.project, &env.ws.text.fonts);
    let counts = header_counts(&env.ws.project);
    let (all, to_do, to_check) = (
        tr("project-filter-all"),
        tr("project-filter-to-do"),
        tr("project-filter-to-check"),
    );
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = space::XS + 2.0;
            super::title(ui, &tr("panel-vehicle"));
            ui.label(RichText::new(&counts).color(color::TEXT_SECONDARY));
        });
        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
            ui.spacing_mut().item_spacing.x = space::SM;
            cmds.secondary_icon_button(
                ui,
                CommandId::AddVehicle,
                icons::ADD,
                &tr("cmd-add-vehicle"),
            );
            // The filter, left of Add Vehicle….
            if let Some(filter) = SegmentedControl::new()
                .segment(TextureFilter::All, "", &all, None)
                .segment(TextureFilter::ToDo, "", &to_do, None)
                .segment(TextureFilter::ToCheck, "", &to_check, None)
                .show(ui, env.ws.texture_filter)
            {
                env.ws.texture_filter = filter;
            }
        });
    });

    let vehicles = env.ws.project.vehicles.clone();
    let last = vehicles.len() <= 1;
    for v in &vehicles {
        let rect = vehicle_card(ui, env, v, last);
        if reveal.is_some_and(|r| *r == ProblemPlace::Vehicle(v.package_id.clone())) {
            ui.scroll_to_rect(rect, Some(Align::TOP));
        }
    }
}

/// A vehicle's card: its header, a hairline, then its textures. Returns
/// where it is.
fn vehicle_card(ui: &mut Ui, env: &mut PanelEnv<'_>, v: &ProjectVehicle, last: bool) -> Rect {
    let mut request = None;
    let card = Frame::new()
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
    card.response.rect
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
        let ids = cabin_ids_text(&env.ws.project, v);
        block(&mut info, 320.0, |ui| {
            caption(ui, &tr("project-cabins"));
            let label = ui.add(egui::Label::new(&cabins).truncate());
            if let Some(ids) = ids {
                label.on_hover_text(ids);
            }
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

/// A truck's main textures by name: "Standard cab, High roof".
fn cabins_text(project: &Project, v: &ProjectVehicle) -> String {
    main_surfaces(project, v)
        .map(|i| project.surfaces[i].name.clone())
        .collect::<Vec<_>>()
        .join(", ")
}

/// The cabins' internal names of a truck's main textures, when the package
/// gives them (the tooltip of the Cabins value): "Standard cab: standard;
/// High roof: high_roof".
fn cabin_ids_text(project: &Project, v: &ProjectVehicle) -> Option<String> {
    let ids: Vec<String> = main_surfaces(project, v)
        .filter_map(|i| {
            let surface = &project.surfaces[i];
            let ids = surface.template.as_ref()?.game_ids.join(", ");
            (!ids.is_empty()).then(|| format!("{}: {ids}", surface.name))
        })
        .collect();
    (!ids.is_empty()).then(|| tr!("project-cabin-ids", ids = ids.join("; ")))
}

/// The indices of a vehicle's main textures.
fn main_surfaces<'a>(project: &'a Project, v: &ProjectVehicle) -> impl Iterator<Item = usize> + 'a {
    project
        .vehicle_range(&v.package_id)
        .filter(|i| vehicle::part_of(project, *i) == TexturePart::Main)
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

/// The vehicle's textures that match the texture filter, in project order,
/// under Main textures and Accessories (a heading is left out when it would
/// list nothing); "Nothing to do" when none matches.
fn textures(ui: &mut Ui, env: &mut PanelEnv<'_>, v: &ProjectVehicle) {
    let range = env.ws.project.vehicle_range(&v.package_id);
    let filter = env.ws.texture_filter;
    let mut first = true;
    for part in [TexturePart::Main, TexturePart::Accessory] {
        let project = &env.ws.project;
        let items: Vec<usize> = range
            .clone()
            .filter(|i| {
                vehicle::part_of(project, *i) == part
                    && filter.matches(project.surfaces[*i].state())
            })
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
    if first {
        ui.label(RichText::new(tr("project-nothing-to-do")).color(color::TEXT_MUTED));
    }
}

/// A texture: thumbnail of its artwork over its template, name, kind and
/// size, and its state (a dot and its label, the reason on hover for To
/// check). Clicking it (or Enter / Space) makes it
/// the active texture and shows the Workshop. The active texture's tile is
/// outlined, marked by an indicator bar and its thumbnail ringed.
fn texture_tile(ui: &mut Ui, env: &mut PanelEnv<'_>, i: usize) {
    // No texture is highlighted while a symbol is edited.
    let active = env.ws.project.active_surface == i && !env.ws.is_editing_symbol();
    let project = &env.ws.project;
    let surface = &project.surfaces[i];
    let name = surface.name.clone();
    let detail = texture_detail(project, i);
    let state = surface.state();
    let state_text = vehicle::state_label(state);
    let reason = vehicle::state_tooltip(state, &vehicle::vehicle_version(project, i));
    let label = vehicle::texture_name(project, i);

    let size = Vec2::new(TILE_WIDTH, TILE_PAD + TILE_THUMB.y + TILE_TEXT);
    let (rect, tile) = ui.allocate_exact_size(size, Sense::click());
    tile.widget_info(|| {
        WidgetInfo::selected(WidgetType::SelectableLabel, true, active, label.as_str())
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

    let thumb = Rect::from_min_size(rect.min + Vec2::splat(TILE_PAD), TILE_THUMB);
    paint_texture(ui, env, i, thumb, cover_uv(thumb));
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

    // Name, kind and size, then the state.
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
    let detail_rect = clipped.text(
        text.left_top() + Vec2::new(0.0, line),
        Align2::LEFT_TOP,
        &detail,
        egui::FontId::monospace(typography::CAPTION),
        color::TEXT_DISABLED,
    );
    // The muted label lacks contrast on the hovered and active fills, the
    // signal color on the active one (its dot keeps it).
    let (dot, mut ink) = vehicle::state_colors(state, active || tile.hovered());
    if active && state.is_to_check() {
        ink = color::TEXT_PRIMARY;
    }
    let top = detail_rect.bottom() + space::XXS + 1.0;
    let font = egui::FontId::proportional(typography::CAPTION);
    let height = ui.fonts_mut(|f| f.row_height(&font));
    let r = vehicle::STATE_DOT / 2.0;
    clipped.circle_filled(egui::pos2(text.left() + r, top + height / 2.0), r, dot);
    clipped.text(
        egui::pos2(text.left() + vehicle::STATE_DOT + space::XS + 2.0, top),
        Align2::LEFT_TOP,
        &state_text,
        font,
        ink,
    );
    if let Some(reason) = &reason {
        let area = Rect::from_min_max(egui::pos2(text.left(), top), text.right_bottom());
        ui.interact(area, tile.id.with("state"), Sense::hover())
            .on_hover_text(reason.as_str())
            .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, reason.as_str()));
    }
    if tile.clicked() {
        env.ws.set_active_surface(i);
        env.ws.space = Space::Workshop;
    }
}

/// What the Mod information column draws its fields with this frame.
struct Column<'a> {
    problems: &'a [Problem],
    /// Where Show on a problem leads, this frame only.
    reveal: Option<&'a ProblemPlace>,
}

impl Column<'_> {
    /// Whether Show leads to `field` this frame.
    fn reveals(&self, field: ModField) -> bool {
        self.reveal == Some(&ProblemPlace::Field(field))
    }
}

/// The id of the column's Advanced state (open or not).
fn advanced_id() -> egui::Id {
    egui::Id::new("project_mod_advanced")
}

/// The Mod information column, from top to bottom: the mod's pictures
/// with their controls, the settings as fields (Name; Author and Version;
/// Game versions; Description; Price and Unlock level), Advanced (the
/// internal name) and Before exporting. A problem about a setting is shown
/// under its field. `reveal` scrolls to a field and gives it the focus.
fn mod_information(ui: &mut Ui, env: &mut PanelEnv<'_>, reveal: Option<&ProblemPlace>) {
    let ctx = ui.ctx().clone();
    env.ws
        .mod_previews
        .update(&ctx, &env.ws.project, &env.ws.text.fonts);
    let problems = crate::mod_export::problems(&env.ws.project);
    let column = Column {
        problems: &problems,
        reveal,
    };
    ScrollArea::vertical()
        .id_salt("project_mod_information")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            Frame::new().inner_margin(Margin::same(18)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing.y = space::MD;
                super::caps_heading(ui, &tr("project-mod-information"));
                pictures(ui, env);
                text_setting(ui, env, &column, ModField::Name, TextKind::Single);
                field_problems(ui, &column, ModField::Name);
                side_by_side(ui, 96.0, |ui, column_index| match column_index {
                    0 => text_setting(ui, env, &column, ModField::Author, TextKind::Single),
                    _ => text_setting(ui, env, &column, ModField::Version, TextKind::Mono),
                });
                // Under both, so that they have the column's width.
                field_problems(ui, &column, ModField::Author);
                field_problems(ui, &column, ModField::Version);
                let field =
                    vehicle::game_versions_field(ui, env, column.reveals(ModField::GameVersions));
                if column.reveals(ModField::GameVersions) {
                    field.scroll_to_me(Some(Align::Center));
                }
                field_problems(ui, &column, ModField::GameVersions);
                text_setting(ui, env, &column, ModField::Description, TextKind::Multiline);
                let half = (ui.available_width() - FIELD_GAP) / 2.0;
                side_by_side(ui, half, |ui, column_index| match column_index {
                    0 => number_setting(ui, env, &column, ModField::Price, 100_000_000),
                    _ => number_setting(ui, env, &column, ModField::UnlockLevel, 1000),
                });
                field_problems(ui, &column, ModField::Price);
                advanced(ui, env, &column);
                ui.add_space(space::SM);
                before_exporting(ui, env, &problems);
            });
        });
}

/// The gap between two fields side by side.
const FIELD_GAP: f32 = space::SM + 2.0;

/// Two columns side by side, the second `second` wide; `body` draws
/// column 0 then column 1.
fn side_by_side(ui: &mut Ui, second: f32, mut body: impl FnMut(&mut Ui, usize)) {
    ui.horizontal_top(|ui| {
        let first = ui.available_width() - second - FIELD_GAP;
        ui.spacing_mut().item_spacing.x = FIELD_GAP;
        for (i, width) in [first, second].into_iter().enumerate() {
            ui.allocate_ui_with_layout(Vec2::new(width, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(width);
                ui.spacing_mut().item_spacing.y = space::MD;
                body(ui, i);
            });
        }
    });
}

/// The problems about `field`, under it.
fn field_problems(ui: &mut Ui, column: &Column<'_>, field: ModField) {
    for problem in column.problems {
        if problem.place() == ProblemPlace::Field(field) {
            crate::ui::dialogs::problem(ui, &problem.message());
        }
    }
}

/// The value of text setting `field`.
fn text_value(project: &Project, field: ModField) -> String {
    let s = &project.mod_settings;
    match field {
        ModField::Name => s.name.clone(),
        ModField::Author => s.author.clone(),
        ModField::Version => s.version.clone(),
        ModField::Description => s.description.clone(),
        ModField::InternalName => s.internal_name(project.internal_name_limit()),
        ModField::Price | ModField::UnlockLevel | ModField::GameVersions => String::new(),
    }
}

/// A text setting under its label, committed when it is left (see
/// [`crate::ui::dialogs::committed_text`]): one "Edit Mod Settings" step,
/// nothing when unchanged. What is typed is mirrored in the workspace so
/// that a command can commit it first.
fn text_setting(
    ui: &mut Ui,
    env: &mut PanelEnv<'_>,
    column: &Column<'_>,
    field: ModField,
    kind: TextKind,
) {
    let label = field.label();
    let current = text_value(&env.ws.project, field);
    let id = egui::Id::new(("project_mod_setting", field));
    let edited = crate::ui::dialogs::labelled(ui, &label, |ui| {
        crate::ui::dialogs::committed_text(ui, id, &label, &current, kind, "", None)
    });
    if let Some(text) = edited.typing {
        env.ws.mod_draft = Some((field, text));
    }
    if edited.left && env.ws.mod_draft.as_ref().is_some_and(|(f, _)| *f == field) {
        env.ws.mod_draft = None;
    }
    if let Some(text) = edited.committed {
        env.ws.commit_mod_field(field, text, env.now);
    }
    if column.reveals(field) {
        edited.response.scroll_to_me(Some(Align::Center));
        edited.response.request_focus();
    }
}

/// A whole-number setting (Price, Unlock level) from 0 to `max`, in the
/// monospace face, as tall as the text fields: one "Edit Mod Settings"
/// step when a number is committed; anything else restores the value.
fn number_setting(
    ui: &mut Ui,
    env: &mut PanelEnv<'_>,
    column: &Column<'_>,
    field: ModField,
    max: u32,
) {
    let label = field.label();
    let s = &env.ws.project.mod_settings;
    let value = match field {
        ModField::Price => s.price,
        _ => s.unlock_level,
    };
    let event = crate::ui::dialogs::labelled(ui, &label, |ui| {
        let shown = ui.scope(|ui| {
            let event = NumericField::new("", &label, Some(f64::from(value)))
                .range(0.0..=f64::from(max))
                .fill()
                .inset(crate::ui::dialogs::FIELD_HEIGHT)
                .show(ui);
            // Asked once it is shown: on the frame after a dialog closed,
            // egui still drops the focus of a field behind it while drawn.
            if column.reveals(field) {
                // The id NumericField gives its text box.
                let text_id = ui.id().with(("numeric_field", label.as_str())).with("text");
                ui.memory_mut(|m| m.request_focus(text_id));
            }
            event
        });
        if column.reveals(field) {
            ui.scroll_to_rect(shown.response.rect, Some(Align::Center));
        }
        shown.inner
    });
    if let FieldEvent::Commit(v) = event {
        let v = v.round() as u32;
        env.ws.set_mod_setting(env.now, |s| match field {
            ModField::Price => s.price = v,
            _ => s.unlock_level = v,
        });
    }
}

/// Advanced: a header that opens and closes it, then the internal name
/// (it follows the Name until a value is committed) and its help. It opens
/// by itself while a problem concerns the internal name.
fn advanced(ui: &mut Ui, env: &mut PanelEnv<'_>, column: &Column<'_>) {
    let id = advanced_id();
    let forced = column.reveals(ModField::InternalName)
        || column
            .problems
            .iter()
            .any(|p| p.place() == ProblemPlace::Field(ModField::InternalName));
    let mut open = forced || ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
    let title = tr("mod-advanced");
    let caret = if open {
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
        info.selected = Some(open);
        info
    });
    if response.clicked() && !forced {
        open = !open;
    }
    ui.data_mut(|d| d.insert_temp(id, open));
    if open {
        ui.indent("project_mod_advanced", |ui| {
            ui.spacing_mut().item_spacing.y = space::SM;
            text_setting(ui, env, column, ModField::InternalName, TextKind::Single);
            field_problems(ui, column, ModField::InternalName);
            ui.add(
                egui::Label::new(
                    RichText::new(tr("mod-internal-name-help"))
                        .small()
                        .color(color::TEXT_SECONDARY),
                )
                .selectable(false)
                .wrap(),
            );
        });
    }
}

/// Both pictures of the mod, each under its title with its size and its
/// controls. Files dragged over the window are dropped on their previews
/// (see `AppState::after_frame`).
fn pictures(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    let previews = env.ws.mod_previews.textures().cloned();
    // A generated picture of a blank first texture would be blank: a
    // placeholder says where it comes from instead.
    let blank = env
        .ws
        .project
        .surfaces
        .first()
        .is_none_or(|s| s.objects.is_empty());
    for (i, which) in [ModPicture::Icon, ModPicture::Image]
        .into_iter()
        .enumerate()
    {
        let settings = &env.ws.project.mod_settings;
        let chosen = match which {
            ModPicture::Icon => settings.icon.is_some(),
            ModPicture::Image => settings.image.is_some(),
        };
        let (title, choose, generated) = match which {
            ModPicture::Icon => ("mod-icon", "mod-choose-icon", "mod-generated-icon"),
            ModPicture::Image => ("mod-image", "mod-choose-image", "mod-generated-image"),
        };
        let shown = picture(
            ui,
            &tr(title),
            previews.as_ref().map(|p| &p[i]),
            which.size(),
            blank && !chosen,
        );
        env.ws.picture_drop_zones[i] = Some(shown.rect);
        let (w, h) = which.size();
        let clicked = ui
            .horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(space::SM, space::XS);
                ui.add(
                    egui::Label::new(
                        RichText::new(format!("{w} × {h}"))
                            .monospace()
                            .size(typography::CAPTION)
                            .color(color::TEXT_MUTED),
                    )
                    .selectable(false),
                );
                let choose = ui.add(secondary_button(&tr(choose))).clicked();
                let generated = chosen && ui.add(secondary_button(&tr(generated))).clicked();
                (choose, generated)
            })
            .inner;
        if clicked.0 {
            env.ws.picture_request = Some(which);
        }
        if clicked.1 {
            env.ws.picture_error = None;
            env.ws
                .set_mod_picture(which, &crate::mod_export::Picture::Generated, env.now);
        }
        if let Some((_, error)) = env.ws.picture_error.as_ref().filter(|(w, _)| *w == which) {
            crate::ui::dialogs::problem(ui, error);
        }
    }
}

/// Before exporting: the problems that block the export first (an error
/// icon and the error color, each with Show, which leads to where it is
/// fixed), then the project's warnings (see
/// [`crate::mod_export::warnings`]), each with a dot in its state's color:
/// a To check line with Open, then the Empty line, expanding to its
/// textures with Open (Open on the line itself for one). "Nothing to
/// check" when there is neither. Open makes the texture active and shows
/// the Workshop.
fn before_exporting(ui: &mut Ui, env: &mut PanelEnv<'_>, problems: &[Problem]) {
    super::caps_heading(ui, &tr("project-before-exporting"));
    let project = &env.ws.project;
    let warnings = crate::mod_export::warnings(project);
    if warnings.is_empty() && problems.is_empty() {
        ui.label(
            RichText::new(tr("project-nothing-to-check"))
                .small()
                .color(color::TEXT_MUTED),
        );
        return;
    }
    let mut show = None;
    for problem in problems {
        let name = crate::ui::mod_export_dialog::show_name(project, problem);
        let row = warning_row(
            ui,
            Marker::Error,
            &problem.message(),
            None,
            Some(Action::Show(&name)),
        );
        if row.opened {
            show = Some(problem.place());
        }
    }
    let label = |i: usize| crate::mod_export::texture_label(project, i);
    let (empty_dot, _) = vehicle::state_colors(TextureState::Empty, false);
    let mut open = None;
    for warning in &warnings {
        let text = warning.message(project);
        match warning {
            Warning::ToCheck { surface, .. } => {
                let row = warning_row(
                    ui,
                    Marker::Dot(color::SIGNAL, color::TEXT_PRIMARY),
                    &text,
                    None,
                    Some(Action::Open(&label(*surface))),
                );
                if row.opened {
                    open = Some(*surface);
                }
            }
            Warning::Empty { surfaces } => {
                let marker = Marker::Dot(empty_dot, color::TEXT_SECONDARY);
                if let [one] = surfaces.as_slice() {
                    let row =
                        warning_row(ui, marker, &text, None, Some(Action::Open(&label(*one))));
                    if row.opened {
                        open = Some(*one);
                    }
                    continue;
                }
                let id = ui.id().with("before_exporting_empty");
                let expanded = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
                if warning_row(ui, marker, &text, Some(expanded), None).toggled {
                    ui.data_mut(|d| d.insert_temp(id, !expanded));
                }
                if expanded {
                    for &i in surfaces {
                        let indent = CARET_WIDTH + space::SM;
                        ui.horizontal_top(|ui| {
                            ui.add_space(indent);
                            let row = warning_row(
                                ui,
                                marker,
                                &label(i),
                                None,
                                Some(Action::Open(&label(i))),
                            );
                            if row.opened {
                                open = Some(i);
                            }
                        });
                    }
                }
            }
        }
    }
    if let Some(place) = show {
        // Taken on the next frame by the column or the Vehicles area.
        env.ws.reveal = Some(place);
        ui.ctx().request_repaint();
    }
    if let Some(i) = open {
        env.ws.set_active_surface(i);
        env.ws.space = Space::Workshop;
    }
}

/// Width of the caret of an expanding warning line.
const CARET_WIDTH: f32 = 14.0;

/// What was clicked on a warning line.
struct RowClick {
    /// The line itself (it expands).
    toggled: bool,
    /// Its Open or Show button.
    opened: bool,
}

/// How a Before exporting line is marked.
#[derive(Clone, Copy)]
enum Marker {
    /// A warning: a dot and the text in these colors.
    Dot(egui::Color32, egui::Color32),
    /// A problem: the error icon and the error color.
    Error,
}

/// The button at the right of a Before exporting line.
#[derive(Clone, Copy)]
enum Action<'a> {
    /// Open texture <name>.
    Open(&'a str),
    /// Show where a problem is fixed, named so.
    Show(&'a str),
}

impl Action<'_> {
    fn text(self) -> String {
        tr(match self {
            Action::Open(_) => "project-open",
            Action::Show(_) => "project-show",
        })
    }

    /// The accessible name.
    fn name(self) -> String {
        match self {
            Action::Open(name) => tr!("project-open-named", name = name),
            Action::Show(name) => name.to_owned(),
        }
    }
}

/// A Before exporting line: its marker and the wrapped text, with a caret
/// when it expands (`caret`: whether it is expanded, the line is then a
/// button), and an Open or Show button at its right.
fn warning_row(
    ui: &mut Ui,
    marker: Marker,
    text: &str,
    caret: Option<bool>,
    action: Option<Action<'_>>,
) -> RowClick {
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM;
        let font = egui::TextStyle::Small.resolve(ui.style());
        let row = ui.fonts_mut(|f| f.row_height(&font));
        let button = action.map_or(0.0, |a| action_width(ui, a) + space::SM);
        let width = ui.available_width() - button;
        let caret_width = if caret.is_some() { CARET_WIDTH } else { 0.0 };
        let (mark_width, ink) = match marker {
            Marker::Dot(_, ink) => (vehicle::STATE_DOT, ink),
            Marker::Error => (ERROR_ICON, color::ERROR),
        };
        let wrap = width - mark_width - space::SM - caret_width;
        let galley = ui.painter().layout(text.to_owned(), font, ink, wrap);
        // At least the interaction height, the first line centered in it.
        let lead = ((ui.spacing().interact_size.y - row) / 2.0).max(0.0);
        let height = galley.size().y + 2.0 * lead;
        let sense = if caret.is_some() {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), sense);
        response.widget_info(|| match caret {
            Some(expanded) => {
                let mut info = WidgetInfo::labeled(WidgetType::CollapsingHeader, true, text);
                info.selected = Some(expanded);
                info
            }
            None => WidgetInfo::labeled(WidgetType::Label, true, text),
        });
        let top = rect.top() + lead;
        let painter = ui.painter();
        match marker {
            Marker::Dot(dot, _) => {
                let r = vehicle::STATE_DOT / 2.0;
                painter.circle_filled(egui::pos2(rect.left() + r, top + row / 2.0), r, dot);
            }
            Marker::Error => {
                painter.text(
                    egui::pos2(rect.left(), top + row / 2.0),
                    Align2::LEFT_CENTER,
                    icons::WARNING,
                    icons::font(ERROR_ICON),
                    color::ERROR,
                );
            }
        }
        let mut x = rect.left() + mark_width + space::SM;
        if let Some(expanded) = caret {
            let icon = if expanded {
                icons::EXPANDED
            } else {
                icons::COLLAPSED
            };
            painter.text(
                egui::pos2(x, top + row / 2.0),
                Align2::LEFT_CENTER,
                icon,
                icons::font(12.0),
                color::TEXT_SECONDARY,
            );
            x += caret_width;
            paint_focus_ring(ui, rect, &response, radius::SM);
        }
        painter.galley(egui::pos2(x, top), galley, ink);
        let opened = action.is_some_and(|a| action_button(ui, a, height, lead));
        RowClick {
            toggled: response.clicked(),
            opened,
        }
    })
    .inner
}

/// Size of a problem's error icon in Before exporting.
const ERROR_ICON: f32 = 12.0;

/// The width of an Open or Show button.
fn action_width(ui: &Ui, action: Action<'_>) -> f32 {
    let font = egui::TextStyle::Small.resolve(ui.style());
    ui.painter()
        .layout_no_wrap(action.text(), font, color::TEXT_SECONDARY)
        .size()
        .x
}

/// A small Open or Show button, `height` tall with its text `lead` below
/// its top (on its line's first line); whether it was clicked. Its text
/// brightens on hover.
fn action_button(ui: &mut Ui, action: Action<'_>, height: f32, lead: f32) -> bool {
    let font = egui::TextStyle::Small.resolve(ui.style());
    let (rect, button) =
        ui.allocate_exact_size(Vec2::new(action_width(ui, action), height), Sense::click());
    button.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, action.name()));
    let ink = if button.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        color::TEXT_PRIMARY
    } else {
        color::TEXT_SECONDARY
    };
    ui.painter().text(
        egui::pos2(rect.left(), rect.top() + lead),
        Align2::LEFT_TOP,
        action.text(),
        font,
        ink,
    );
    paint_focus_ring(ui, rect, &button, radius::SM);
    button.clicked()
}

/// A picture of the mod at its size in pixels, under its title; an empty
/// box until it is rendered (no spinner: the render wakes the UI when it
/// ends, nothing needs to repaint meanwhile). A `blank` picture (generated
/// from a texture with nothing on it) shows a dashed placeholder saying
/// where it comes from and that a file can be dropped on it. While files
/// are dragged over the window, the preview shows that it takes them, and
/// the one under the pointer is highlighted. Returns its preview.
fn picture(
    ui: &mut Ui,
    title: &str,
    texture: Option<&egui::TextureHandle>,
    size: (u32, u32),
    blank: bool,
) -> Response {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = space::XS;
        super::field_label(ui, title);
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(size.0 as f32, size.1 as f32), Sense::hover());
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Image, true, title));
        let painter = ui.painter();
        if blank {
            let text = format!(
                "{}\n{}",
                tr("project-picture-generated"),
                tr("project-picture-drop")
            );
            painter.rect_filled(rect, radius::LG, color::CANVAS);
            dashed_rect(painter, rect, color::BORDER_STRONG);
            let mut job = egui::text::LayoutJob::simple(
                text.clone(),
                egui::FontId::proportional(typography::CAPTION),
                color::TEXT_MUTED,
                rect.width() - 2.0 * space::MD,
            );
            job.halign = Align::Center;
            let galley = painter.layout_job(job);
            painter.galley(
                egui::pos2(rect.center().x, rect.center().y - galley.size().y / 2.0),
                galley,
                color::TEXT_MUTED,
            );
            ui.interact(rect, response.id.with("blank"), Sense::hover())
                .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
        } else {
            match texture {
                Some(texture) => {
                    painter.image(
                        texture.id(),
                        rect,
                        Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
                None => {
                    painter.rect_filled(rect, radius::SM, color::CONTROL);
                }
            }
            painter.rect_stroke(
                rect,
                radius::SM,
                Stroke::new(1.0, color::OUTLINE),
                StrokeKind::Outside,
            );
        }
        // Files dragged over the window: every preview takes them, the one
        // under the pointer is highlighted.
        let (dragging, pointer) = ui
            .ctx()
            .input(|i| (!i.raw.hovered_files.is_empty(), i.pointer.latest_pos()));
        if dragging {
            let under = pointer.is_some_and(|p| rect.contains(p));
            if under {
                painter.rect_filled(rect, radius::LG, color::SURFACE_2.gamma_multiply(0.85));
            }
            let outline = if under {
                color::TEXT_SECONDARY
            } else {
                color::BORDER_STRONG
            };
            dashed_rect(painter, rect, outline);
        }
        response
    })
    .inner
}

/// A dashed hairline around `rect`.
fn dashed_rect(painter: &egui::Painter, rect: Rect, ink: egui::Color32) {
    let rect = rect.shrink(0.5);
    let corners = [
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
        rect.left_top(),
    ];
    for pair in corners.windows(2) {
        painter.extend(egui::Shape::dashed_line(
            pair,
            Stroke::new(1.0, ink),
            5.0,
            4.0,
        ));
    }
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
        assert_eq!(cabins_text(&project, v), "Standard cab, High roof");
        assert_eq!(
            cabin_ids_text(&project, v).as_deref(),
            Some("Internal names: Standard cab: standard; High roof: high_roof")
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
