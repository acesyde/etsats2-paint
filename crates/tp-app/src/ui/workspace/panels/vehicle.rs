//! The project's fleet as a tree, vehicle › Main textures / Accessories ›
//! texture (the Textures tab); what the Project space shares with it (a
//! vehicle's ⋯ menu, kind and package, a texture's part and state, the
//! Game versions field); and the active texture's To check notice shown by
//! the inspector ([`texture_notices`]). The template's visibility and opacity
//! are in the status bar.

use egui::collapsing_header::CollapsingState;
use egui::{
    Align, Align2, Layout, Rect, RichText, Sense, Stroke, StrokeKind, Ui, Vec2, WidgetInfo,
    WidgetType,
};
use tp_core::{CheckReason, Project, ProjectVehicle, TexturePart, TextureState};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::theme::label_strong_style;
use tp_ui::tokens::{color, radius, space};
use tp_ui::widgets::{IconButton, MenuRow, more_menu};

use super::PanelEnv;
use crate::commands::CommandId;
use crate::state::VehicleRequest;
use crate::ui::CommandUi;

/// The fleet tree of the Textures tab under the Search textures field:
/// clicking a texture makes it active.
pub fn show(ui: &mut Ui, cmds: &mut CommandUi<'_>, env: &mut PanelEnv<'_>) {
    if let Some(query) = search_field(ui, cmds) {
        *env.palette_request = Some(crate::state::PaletteOpen {
            textures_only: true,
            query,
        });
    }
    ui.add_space(space::SM);
    vehicles_section(ui, cmds, env);
}

/// The id of the Search textures field.
pub fn search_field_id() -> egui::Id {
    egui::Id::new("textures_search_field")
}

/// Whether the Search textures field has the keyboard focus: what is typed
/// goes to the palette it opens, not to single-key shortcuts.
pub fn search_field_focused(ctx: &egui::Context) -> bool {
    ctx.memory(|m| m.has_focus(search_field_id()))
}

/// Height of the Search textures field.
const SEARCH_HEIGHT: f32 = 28.0;

/// The Search textures field: it looks like a field, with the Command
/// Palette's shortcut at its right end, but holds no text. Clicking it, or
/// Enter or typing while it has the focus, opens the palette limited to
/// textures: returns what was typed.
fn search_field(ui: &mut Ui, cmds: &CommandUi<'_>) -> Option<String> {
    let label = tr("textures-search");
    let rect = Rect::from_min_size(
        ui.cursor().min,
        Vec2::new(ui.available_width(), SEARCH_HEIGHT),
    );
    let response = ui.interact(rect, search_field_id(), Sense::click());
    ui.advance_cursor_after_rect(rect);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, &label));
    let painter = ui.painter();
    painter.rect(
        rect,
        radius::LG,
        color::FIELD,
        Stroke::new(1.0, color::BORDER),
        StrokeKind::Inside,
    );
    tp_ui::widgets::paint_focus_ring(ui, rect, &response, radius::LG);
    let y = rect.center().y;
    let icon = painter.text(
        egui::pos2(rect.left() + space::SM + 2.0, y),
        Align2::LEFT_CENTER,
        icons::SEARCH,
        icons::font(13.0),
        color::TEXT_MUTED,
    );
    let mut right = rect.right() - space::SM - 2.0;
    if let Some(shortcut) = cmds.shortcuts.command(CommandId::CommandPalette) {
        let key = painter.text(
            egui::pos2(right, y),
            Align2::RIGHT_CENTER,
            shortcut,
            egui::FontId::monospace(tp_ui::tokens::typography::MONO),
            color::TEXT_MUTED,
        );
        right = key.left() - space::SM;
    }
    let text = Rect::from_min_max(
        egui::pos2(icon.right() + space::SM, rect.top()),
        egui::pos2(right, rect.bottom()),
    );
    painter.with_clip_rect(text.intersect(ui.clip_rect())).text(
        egui::pos2(text.left(), y),
        Align2::LEFT_CENTER,
        &label,
        egui::FontId::proportional(tp_ui::tokens::typography::CONTROL),
        color::TEXT_MUTED,
    );
    if response.clicked() {
        return Some(String::new());
    }
    if response.has_focus() {
        let (enter, typed) = ui.input_mut(|i| {
            let enter = i.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
            let mut typed = String::new();
            i.events.retain(|e| match e {
                egui::Event::Text(t) => {
                    typed.push_str(t);
                    false
                }
                _ => true,
            });
            (enter, typed)
        });
        if enter || !typed.is_empty() {
            return Some(typed);
        }
    }
    None
}

/// The id of the Game versions field's text field (+ Add once clicked).
pub fn game_versions_add_id() -> egui::Id {
    egui::Id::new("project_game_versions_add")
}

/// The game versions the mod is made for, one chip per version with its
/// remove button, then + Add, which turns into a short text field: Enter
/// or leaving it adds what was typed, Escape closes it (the Project space's
/// Mod information column). A badly written version's chip is marked. The
/// versions every vehicle supports are shown under the chips. `focus_add`
/// opens + Add's field with the keyboard focus (Show on a problem).
pub fn game_versions_field(ui: &mut Ui, env: &mut PanelEnv<'_>, focus_add: bool) -> egui::Response {
    use crate::game_versions::{FleetVersions, fleet};
    use crate::mod_export::ModField;
    let label = tr("project-game-versions");
    let add_id = game_versions_add_id();
    let adding_id = add_id.with("open");
    // The field takes the focus once shown (so that the focused widget is
    // always in the accessibility tree).
    let pending_id = add_id.with("focus");
    if focus_add {
        ui.data_mut(|d| {
            d.insert_temp(adding_id, true);
            d.insert_temp(pending_id, true);
        });
    }
    let mut adding = ui.data(|d| d.get_temp::<bool>(adding_id)).unwrap_or(false);
    let focus_pending = ui
        .data_mut(|d| d.remove_temp::<bool>(pending_id))
        .unwrap_or(false);
    let mut remove = None;
    let mut add = None;
    let mut open_add = false;
    let mut close_add = false;
    let versions = env.ws.project.game_versions.clone();
    let field = ui
        .vertical(|ui| {
            ui.spacing_mut().item_spacing.y = space::XS;
            crate::ui::dialogs::field_label(ui, &label);
            egui::Frame::new()
                .fill(color::FIELD)
                .stroke(Stroke::new(1.0, color::BORDER))
                .corner_radius(radius::MD)
                .inner_margin(egui::Margin::same(5))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::splat(6.0);
                        for (i, version) in versions.iter().enumerate() {
                            if version_chip(ui, version) {
                                remove = Some(i);
                            }
                        }
                        if adding {
                            let had_focus = ui.memory(|m| m.has_focus(add_id));
                            let field = crate::ui::dialogs::committed_text(
                                ui,
                                add_id,
                                &label,
                                "",
                                crate::ui::dialogs::TextKind::Mono,
                                "1.56.*",
                                Some(96.0),
                            );
                            if let Some(text) = field.typing {
                                env.ws.mod_draft = Some((ModField::GameVersions, text));
                            }
                            if field.left {
                                close_add = true;
                                if env
                                    .ws
                                    .mod_draft
                                    .as_ref()
                                    .is_some_and(|(f, _)| *f == ModField::GameVersions)
                                {
                                    env.ws.mod_draft = None;
                                }
                            }
                            add = field.committed;
                            if focus_pending {
                                field.response.request_focus();
                            } else if !had_focus && !field.left && !field.response.has_focus() {
                                // Opened, but the focus went elsewhere first.
                                close_add = true;
                            }
                        } else {
                            let text = tr("project-game-versions-add");
                            let button = ui.add(
                                egui::Button::new((
                                    icons::rich(icons::ADD).color(color::TEXT_SECONDARY),
                                    RichText::new(&text).color(color::TEXT_SECONDARY),
                                ))
                                .frame(false)
                                .min_size(Vec2::new(0.0, tp_ui::tokens::size::HIT_MIN)),
                            );
                            button.widget_info(|| {
                                WidgetInfo::labeled(WidgetType::Button, true, &text)
                            });
                            if button.clicked() {
                                open_add = true;
                            }
                        }
                    });
                });
        })
        .response;
    if open_add {
        adding = true;
        ui.data_mut(|d| d.insert_temp(pending_id, true));
        ui.ctx().request_repaint();
    }
    if close_add && !focus_add {
        adding = false;
    }
    ui.data_mut(|d| d.insert_temp(adding_id, adding));
    if let Some(text) = add {
        env.ws
            .commit_mod_field(ModField::GameVersions, text, env.now);
    }
    if let Some(i) = remove {
        env.ws.remove_game_version(i, env.now);
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
        let label = ui.add(
            egui::Label::new(RichText::new(&text).small().color(color::TEXT_SECONDARY))
                .selectable(false),
        );
        label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
    }
    field
}

/// A game version's chip: the version in the monospace face and its
/// remove button, named "Remove <version>"; a badly written version is
/// outlined in the error color with a warning icon, and says why on hover.
/// Whether its remove button was clicked.
fn version_chip(ui: &mut Ui, version: &str) -> bool {
    let bad = crate::game_versions::of_listed(version).is_none();
    let ink = if bad {
        color::ERROR
    } else {
        color::TEXT_PRIMARY
    };
    let font = egui::FontId::monospace(tp_ui::tokens::typography::MONO);
    let text = ui.painter().layout_no_wrap(version.to_owned(), font, ink);
    let warning = bad.then(|| {
        ui.painter()
            .layout_no_wrap(icons::WARNING.to_owned(), icons::font(12.0), color::ERROR)
    });
    let hit = tp_ui::tokens::size::HIT_MIN;
    let pad = space::SM;
    let lead = warning.as_ref().map_or(0.0, |w| w.size().x + space::XS);
    let size = Vec2::new(pad + lead + text.size().x + hit - space::XS, hit);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    let close = Rect::from_min_size(egui::pos2(rect.right() - hit, rect.top()), Vec2::splat(hit));
    let painter = ui.painter();
    let pill = rect.height() / 2.0;
    painter.rect_filled(rect, pill, color::CHIP);
    if bad {
        painter.rect_stroke(
            rect,
            pill,
            Stroke::new(1.0, color::ERROR),
            StrokeKind::Inside,
        );
    }
    let mut x = rect.left() + pad;
    if let Some(w) = warning {
        let y = rect.center().y - w.size().y / 2.0;
        let width = w.size().x;
        painter.galley(egui::pos2(x, y), w, color::ERROR);
        x += width + space::XS;
    }
    let y = rect.center().y - text.size().y / 2.0;
    painter.galley(egui::pos2(x, y), text, ink);
    let chip = ui.interact(
        Rect::from_min_max(rect.min, egui::pos2(close.left(), rect.bottom())),
        response.id.with("chip"),
        Sense::hover(),
    );
    chip.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, version));
    if bad {
        chip.on_hover_text(
            crate::mod_export::Problem::BadGameVersion {
                version: version.to_owned(),
            }
            .message(),
        );
    }
    let remove = ui.interact(close, response.id.with("remove"), Sense::click());
    let name = tr!("project-game-version-remove", version = version);
    remove.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, &name));
    let icon_ink = if remove.hovered() {
        color::TEXT_PRIMARY
    } else {
        color::TEXT_SECONDARY
    };
    ui.painter().text(
        close.center() - Vec2::new(space::XXS, 0.0),
        Align2::CENTER_CENTER,
        icons::CLOSE,
        icons::font(11.0),
        icon_ink,
    );
    tp_ui::widgets::paint_focus_ring(ui, close.shrink(3.0), &remove, radius::SM);
    remove.clicked()
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

/// A texture state's label: "Empty", "Modified" or "To check".
pub fn state_label(state: TextureState) -> String {
    tr(match state {
        TextureState::Empty => "texture-state-empty",
        TextureState::Modified => "texture-state-modified",
        TextureState::ToCheck(_) => "texture-state-to-check",
    })
}

/// Why a texture is To check, the package `version` of its vehicle given:
/// "Layout changed in 1.3.0" or "Not in this version" (none for the other
/// states).
pub fn state_tooltip(state: TextureState, version: &str) -> Option<String> {
    match state {
        TextureState::ToCheck(CheckReason::LayoutChanged) => {
            Some(tr!("texture-reason-layout-changed", version = version))
        }
        TextureState::ToCheck(CheckReason::NotInVersion) => {
            Some(tr("texture-reason-not-in-version"))
        }
        _ => None,
    }
}

/// The package version of the vehicle surface `i` belongs to (empty when
/// unknown).
pub fn vehicle_version(project: &Project, i: usize) -> String {
    project
        .vehicle_of(i)
        .map(|v| v.version.clone())
        .unwrap_or_default()
}

/// A texture's name for assistive technologies: "Texture <vehicle> ›
/// <texture>, <state>".
pub fn texture_name(project: &Project, i: usize) -> String {
    let path = project.surface_names(i).map_or_else(
        || project.surfaces[i].name.clone(),
        |(v, t)| format!("{v} › {t}"),
    );
    tr!(
        "vehicle-panel-texture",
        name = path.as_str(),
        state = state_label(project.surfaces[i].state())
    )
}

/// The dot of a state and its label's color, in the Project space's tiles
/// and Before exporting: muted for Empty, primary for Modified, the signal
/// color for To check. `hover_fill` is true on a hovered or active tile,
/// where muted text lacks contrast.
pub fn state_colors(state: TextureState, hover_fill: bool) -> (egui::Color32, egui::Color32) {
    match state {
        TextureState::Empty if hover_fill => (color::BORDER_STRONG, color::TEXT_SECONDARY),
        TextureState::Empty => (color::BORDER_STRONG, color::TEXT_MUTED),
        TextureState::Modified => (color::TEXT_SECONDARY, color::TEXT_PRIMARY),
        TextureState::ToCheck(_) => (color::SIGNAL, color::SIGNAL),
    }
}

/// Diameter of a state's dot, in points.
pub const STATE_DOT: f32 = 6.0;

/// The marker of a state in the Textures tab, centered at `center`: a
/// hollow ring for Empty, a filled dot for Modified and a warning icon in
/// the signal color for To check, so the states differ by shape.
pub fn paint_state_marker(painter: &egui::Painter, center: egui::Pos2, state: TextureState) {
    let r = STATE_DOT / 2.0;
    match state {
        TextureState::Empty => {
            painter.circle_stroke(center, r - 0.5, Stroke::new(1.0, color::TEXT_MUTED));
        }
        TextureState::Modified => {
            painter.circle_filled(center, r, color::TEXT_SECONDARY);
        }
        TextureState::ToCheck(_) => {
            painter.text(
                center,
                Align2::CENTER_CENTER,
                icons::WARNING,
                icons::font(14.0),
                color::SIGNAL,
            );
        }
    }
}

/// The marker of `state` drawn in `rect` (16 pt square), saying `hover`
/// (its state, and why To check) on hover and to assistive technologies.
pub fn state_marker(ui: &mut Ui, rect: Rect, id: egui::Id, state: TextureState, hover: &str) {
    paint_state_marker(ui.painter(), rect.center(), state);
    ui.interact(rect, id, Sense::hover())
        .on_hover_text(hover)
        .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, hover));
}

/// Side of a texture's thumbnail in a row, in points.
const THUMB: f32 = 26.0;
/// Height of a texture row.
const ROW_HEIGHT: f32 = THUMB + 10.0;

/// A texture: thumbnail of its artwork, name, size and its state's marker;
/// clicking makes it active. The active texture's row is filled and its
/// name drawn in ink, as in the mockup.
fn texture_row(ui: &mut Ui, env: &mut PanelEnv<'_>, i: usize) {
    // No texture is highlighted while a symbol is edited.
    let active = env.ws.project.active_surface == i && !env.ws.is_editing_symbol();
    let surface = &env.ws.project.surfaces[i];
    let name = surface.name.clone();
    let size = format!("{}", surface.size);
    let state = surface.state();
    let label = texture_name(&env.ws.project, i);
    let hover = match state_tooltip(state, &vehicle_version(&env.ws.project, i)) {
        Some(reason) => format!("{}\n{reason}", state_label(state)),
        None => state_label(state),
    };
    let (rect, row) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), ROW_HEIGHT), Sense::click());
    row.widget_info(|| {
        WidgetInfo::selected(WidgetType::SelectableLabel, true, active, label.as_str())
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

    // Size and state at the right end, the name in between.
    let mono = egui::TextStyle::Monospace.resolve(ui.style());
    let size_rect = painter.text(
        egui::pos2(rect.right() - space::SM, rect.center().y),
        Align2::RIGHT_CENTER,
        &size,
        egui::FontId::new(mono.size - 1.0, mono.family),
        color::TEXT_DISABLED,
    );
    let right = size_rect.left() - space::SM;
    let marker =
        Rect::from_center_size(egui::pos2(right - 8.0, rect.center().y), Vec2::splat(16.0));
    state_marker(ui, marker, row.id.with("state"), state, &hover);
    let right = marker.left() - space::XS;
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

/// The active texture's To check notice, in the inspector when nothing is
/// selected: "To check" and why, "Layout changed in <version>" with Mark
/// as Checked, or "Not in this version, left out of the mod" with no
/// action.
pub fn texture_notices(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    let TextureState::ToCheck(reason) = env.ws.project.surface().state() else {
        return;
    };
    let index = env.ws.project.active_surface;
    let name = env
        .ws
        .project
        .surface_names(index)
        .map_or_else(String::new, |(v, t)| format!("{v} › {t}"));
    let version = vehicle_version(&env.ws.project, index);
    ui.add_space(space::SM);
    let to_check = state_label(TextureState::ToCheck(reason));
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::XS;
        ui.label(icons::rich(icons::WARNING).color(color::SIGNAL));
        ui.label(
            RichText::new(&to_check)
                .text_style(label_strong_style())
                .color(color::SIGNAL),
        );
    });
    let why = match reason {
        CheckReason::LayoutChanged => {
            tr!("texture-reason-layout-changed", version = version.as_str())
        }
        CheckReason::NotInVersion => tr("texture-notice-not-in-version"),
    };
    let text = format!("{to_check}: {why}");
    ui.add(egui::Label::new(RichText::new(&why).small().color(color::TEXT_SECONDARY)).wrap())
        .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text.as_str()));
    if reason == CheckReason::LayoutChanged {
        let mark = ui.small_button(tr("texture-mark-checked"));
        mark.widget_info(|| {
            WidgetInfo::labeled(
                WidgetType::Button,
                true,
                tr!("texture-mark-checked-named", name = name.as_str()),
            )
        });
        if mark.clicked() {
            env.ws.dismiss_layout_change(index, env.now);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn states_read_their_label_and_reason() {
        let changed = TextureState::ToCheck(CheckReason::LayoutChanged);
        let removed = TextureState::ToCheck(CheckReason::NotInVersion);
        assert_eq!(state_label(TextureState::Empty), "Empty");
        assert_eq!(state_label(TextureState::Modified), "Modified");
        assert_eq!(state_label(changed), "To check");
        assert_eq!(state_label(removed), "To check");
        assert_eq!(state_tooltip(TextureState::Empty, "1.3.0"), None);
        assert_eq!(state_tooltip(TextureState::Modified, "1.3.0"), None);
        assert_eq!(
            state_tooltip(changed, "1.3.0").as_deref(),
            Some("Layout changed in 1.3.0")
        );
        assert_eq!(
            state_tooltip(removed, "1.3.0").as_deref(),
            Some("Not in this version")
        );
    }
}
