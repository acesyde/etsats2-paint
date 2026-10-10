//! The Workshop's inspector, on the right of the canvas area: the settings
//! of the selection, section by section (header, Layout, Text, Appearance,
//! Polygon, Style, Image), each shown only when it applies; with nothing
//! selected, the active texture's properties, what is on it (On this
//! texture) and the look of new objects.
//! Colors, stroke and shadow settings open in popovers anchored to their
//! rows.

use egui::{Frame, Margin, Panel, Rect, RichText, ScrollArea, Ui, WidgetInfo, WidgetType};
use tp_core::TexturePart;
use tp_core::document::{Object, ShapeKind, tree};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::{color, size, space, typography};
use tp_ui::widgets::{IconButton, PaintRow, Popover, SwatchColor, secondary_button};

use super::panels::{self, PanelEnv, properties, styles};
use crate::commands::CommandId;
use crate::layout::WorkspaceLayout;
use crate::ui::CommandUi;
use crate::workspace::{ColorTarget, Workspace};

/// The color popover of the Fill and Stroke rows.
pub fn color_popover() -> egui::Id {
    Popover::id("inspector_color")
}

/// The stroke popover of the Stroke row's summary.
pub fn stroke_popover() -> egui::Id {
    Popover::id("inspector_stroke")
}

/// The line settings popover (dashes, caps and joins of lines).
pub fn line_popover() -> egui::Id {
    Popover::id("inspector_line")
}

/// The shadow popover of the Shadow row.
pub fn shadow_popover() -> egui::Id {
    Popover::id("inspector_shadow")
}

/// The inspector, resizable by its left edge within bounds; its width is
/// written back to `layout`.
pub fn show(
    ui: &mut Ui,
    cmds: &mut CommandUi<'_>,
    layout: &mut WorkspaceLayout,
    env: &mut PanelEnv<'_>,
) {
    let range = size::INSPECTOR_MIN..=size::INSPECTOR_MAX;
    let panel = Panel::right(egui::Id::new(("inspector", layout.generation)))
        .resizable(true)
        .default_size(layout.inspector_width)
        .size_range(range.clone())
        .frame(Frame::new().fill(color::SURFACE_1))
        .show(ui, |ui| {
            ScrollArea::vertical()
                .id_salt("inspector_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    if env.ws.selection.is_empty() {
                        nothing_selected(ui, env);
                    } else {
                        selection(ui, cmds, env);
                    }
                });
        });
    let width = panel.response.rect.width().round();
    if (width - layout.inspector_width).abs() >= 1.0 {
        layout.inspector_width = width.clamp(*range.start(), *range.end());
    }
}

/// One section of the inspector: its heading (none for the header), its
/// body, and a hairline under it.
fn section(ui: &mut Ui, heading: Option<&str>, body: impl FnOnce(&mut Ui)) {
    let response = Frame::new()
        .inner_margin(Margin::same((space::MD + 2.0) as i8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = space::SM;
            if let Some(heading) = heading {
                panels::section_heading(ui, heading);
            }
            body(ui);
        })
        .response;
    let y = response.rect.bottom();
    ui.painter().hline(
        response.rect.x_range(),
        y,
        egui::Stroke::new(1.0, color::BORDER),
    );
}

/// A title (14/600), shortened when it does not fit.
fn title(ui: &mut Ui, text: &str) {
    let rich = RichText::new(text)
        .size(typography::HEADING - 1.0)
        .family(tp_ui::fonts::semibold_family())
        .color(color::TEXT_PRIMARY);
    ui.add(egui::Label::new(rich).truncate())
        .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
}

fn caption(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .size(typography::CAPTION + 1.0)
            .color(color::TEXT_SECONDARY),
    )
    .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
}

// --- Nothing selected -----------------------------------------------------------

/// The active texture (name, kind, size, its To check notice), On this
/// texture, the look of new objects, the Text section for new texts with
/// the Text tool, and a hint.
fn nothing_selected(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    section(ui, None, |ui| {
        ui.spacing_mut().item_spacing.y = space::XS;
        let index = env.ws.project.active_surface;
        let surface = env.ws.project.surface();
        let name = env
            .ws
            .project
            .surface_names(index)
            .map_or_else(|| surface.name.clone(), |(_, texture)| texture);
        let part = surface
            .template
            .as_ref()
            .map_or(TexturePart::Main, |t| t.part);
        let kind = tr(match part {
            TexturePart::Main => "inspector-main-texture",
            TexturePart::Accessory => "inspector-accessory-texture",
        });
        let size = surface.size.round();
        title(ui, &name);
        caption(
            ui,
            &tr!("inspector-texture-kind", kind = kind.as_str(), size = size),
        );
        panels::vehicle::texture_notices(ui, env);
    });
    on_this_texture(ui, env);
    section(ui, Some(&tr("inspector-new-objects")), |ui| {
        paint_rows(ui, env);
    });
    if env.ws.tool == crate::tool::Tool::Text {
        section(ui, Some(&tr("inspector-text")), |ui| {
            panels::character::show(ui, env);
        });
    }
    Frame::new()
        .inner_margin(Margin::same((space::MD + 2.0) as i8))
        .show(ui, |ui| {
            ui.label(
                RichText::new(tr("inspector-hint"))
                    .size(typography::CAPTION + 1.0)
                    .color(color::TEXT_DISABLED),
            );
        });
}

/// What the active texture holds: its top-level objects, its symbol
/// instances and its off-palette colors ("In this symbol", without
/// instances, while a symbol is edited). An off-palette count above zero
/// is a button selecting the unlocked objects using them.
fn on_this_texture(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    let editing = env.ws.is_editing_symbol();
    let objects = &env.ws.project.surface().objects;
    let count = objects.len();
    let instances = instance_count(objects);
    let off = tp_core::off_palette(objects);
    let (locked, unlocked): (Vec<_>, Vec<_>) = off
        .objects
        .iter()
        .partition(|id| tree::effective_flags(objects, **id).is_some_and(|(_, l)| l));
    let heading = tr(if editing {
        "inspector-in-symbol"
    } else {
        "inspector-on-texture"
    });
    let mut select = false;
    section(ui, Some(&heading), |ui| {
        count_row(
            ui,
            &tr("inspector-objects"),
            count,
            color::TEXT_PRIMARY,
            false,
        );
        if !editing {
            count_row(
                ui,
                &tr("inspector-instances"),
                instances,
                color::TEXT_PRIMARY,
                false,
            );
        }
        let colors = off.colors.len();
        let ink = if colors == 0 {
            color::TEXT_PRIMARY
        } else {
            color::SIGNAL
        };
        let clickable = colors > 0 && !unlocked.is_empty();
        if let Some(button) = count_row(ui, &tr("inspector-off-palette"), colors, ink, clickable) {
            let name = tr("inspector-off-palette-select");
            button.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, &name));
            let button = if locked.is_empty() {
                button.on_hover_text(&name)
            } else {
                button.on_hover_text(format!(
                    "{name}\n{}",
                    tr!("inspector-off-palette-locked", count = locked.len())
                ))
            };
            select = button.clicked();
        }
    });
    if select {
        env.ws.selection = unlocked.into_iter().copied().collect();
        env.ws.normalize_selection();
    }
}

/// The symbol instances among `objects`, inside groups too (not inside
/// instances: symbols don't nest).
fn instance_count(objects: &[std::sync::Arc<Object>]) -> usize {
    objects
        .iter()
        .map(|o| match o.kind {
            ShapeKind::Instance { .. } => 1,
            ShapeKind::Group => instance_count(&o.children),
            _ => 0,
        })
        .sum()
}

/// A row of On this texture: its label on the left, its count on the right
/// in the monospace face (12), named for assistive technologies as the
/// label with the count as its value. With `button`, the count is a
/// button (returned) with a hover fill.
fn count_row(
    ui: &mut Ui,
    label: &str,
    count: usize,
    ink: egui::Color32,
    button: bool,
) -> Option<egui::Response> {
    let body = egui::TextStyle::Body.resolve(ui.style());
    let mono = egui::FontId::monospace(typography::CAPTION + 1.0);
    let value = count.to_string();
    let painter = ui.painter();
    let name = painter.layout_no_wrap(label.to_owned(), body, color::TEXT_SECONDARY);
    let number = painter.layout_no_wrap(value.clone(), mono, ink);
    let height = name.size().y.max(number.size().y) + 2.0;
    let (rect, row) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::hover(),
    );
    row.widget_info(|| {
        // Not a Label: its name would become its value.
        let mut info = WidgetInfo::labeled(WidgetType::Other, true, label);
        info.current_text_value = Some(value.clone());
        info
    });
    let painter = ui.painter();
    painter.galley(
        egui::pos2(rect.left(), rect.center().y - name.size().y / 2.0),
        name,
        color::TEXT_SECONDARY,
    );
    let pad = space::XS;
    let target = Rect::from_min_max(
        egui::pos2(rect.right() - number.size().x - 2.0 * pad, rect.top()),
        rect.right_bottom(),
    );
    let response = button.then(|| {
        let response = ui.interact(target, row.id.with("button"), egui::Sense::click());
        if response.hovered() {
            ui.painter()
                .rect_filled(target, tp_ui::tokens::radius::SM, color::SURFACE_2);
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        tp_ui::widgets::paint_focus_ring(ui, target, &response, tp_ui::tokens::radius::SM);
        response
    });
    ui.painter().galley(
        egui::pos2(
            target.right() - pad - number.size().x,
            rect.center().y - number.size().y / 2.0,
        ),
        number,
        ink,
    );
    response
}

// --- Selection ------------------------------------------------------------------

fn selection(ui: &mut Ui, cmds: &mut CommandUi<'_>, env: &mut PanelEnv<'_>) {
    let objects = env.ws.selected_objects();
    if objects.is_empty() {
        return;
    }
    let instances_only = objects.iter().all(Object::is_instance);
    let shapes = env.ws.selected_shapes();
    let only = |f: fn(&ShapeKind) -> bool| !shapes.is_empty() && shapes.iter().all(|o| f(&o.kind));
    let only_images = objects
        .iter()
        .all(|o| matches!(o.kind, ShapeKind::Image { .. }));
    let only_texts = only(|k| *k == ShapeKind::Text);

    section(ui, None, |ui| {
        header(ui, cmds, env, &objects, instances_only)
    });
    section(ui, Some(&tr("inspector-layout")), |ui| {
        panels::transform::show(ui, cmds, env);
    });
    if only_texts {
        section(ui, Some(&tr("inspector-text")), |ui| {
            panels::character::show(ui, env);
        });
    }
    section(ui, Some(&tr("inspector-appearance")), |ui| {
        properties::opacity(ui, env, &objects);
        // Instances show their symbol's look; images have no fill nor
        // stroke, but a shadow.
        if instances_only {
            return;
        }
        if !only_images {
            paint_rows(ui, env);
        }
        shadow_row(ui, env);
        if only_images {
            return;
        }
        if let Some(lines) = properties::selected_lines(&objects) {
            line_settings(ui, env, &lines);
        }
        properties::corner_radius(ui, env, &objects);
    });
    if properties::selected_polygons(env.ws).is_some() {
        section(ui, Some(&tr("inspector-polygon")), |ui| {
            properties::polygon_settings(ui, env);
        });
    }
    if styles::selection_style(env).is_some() {
        section(ui, None, |ui| styles::inspector_row(ui, env));
    }
    if let [single] = objects.as_slice()
        && matches!(single.kind, ShapeKind::Image { .. })
    {
        section(ui, Some(&tr("inspector-image")), |ui| {
            properties::image_info(ui, env, single);
        });
    }
}

/// The name of `object` in the inspector's header: its own, or for a text
/// still named after its kind ("Text"), its first line, so that the header
/// doesn't read the kind twice.
pub fn display_name(object: &Object) -> String {
    if let Some(text) = &object.text
        && object.name == crate::workspace::object_name(object.kind)
        && let Some(line) = text.content.lines().map(str::trim).find(|l| !l.is_empty())
    {
        return line.to_owned();
    }
    object.name.clone()
}

/// What is selected (name and kind, or "N objects") and its symbol
/// actions: Convert to Symbol, or for instances their symbol, Edit Symbol
/// and Detach Instance.
fn header(
    ui: &mut Ui,
    cmds: &mut CommandUi<'_>,
    env: &mut PanelEnv<'_>,
    objects: &[Object],
    instances_only: bool,
) {
    let (name, kind) = match objects {
        [single] => {
            let (name, kind) = (
                display_name(single),
                crate::workspace::object_name(single.kind),
            );
            // A shape still named after its kind says it once.
            let kind = if name == kind { String::new() } else { kind };
            (name, kind)
        }
        many => (tr!("props-objects", count = many.len()), String::new()),
    };
    // The kind at the right end, the name shortened before it.
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !kind.is_empty() {
                caption(ui, &kind);
            }
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                title(ui, &name);
            });
        });
    });
    if !instances_only {
        let label = format!(
            "{} {}",
            icons::SYMBOL,
            cmds.label(CommandId::ConvertToSymbol)
        );
        command_button(ui, cmds, CommandId::ConvertToSymbol, &label);
        return;
    }
    let symbols: Vec<tp_core::document::SymbolId> = objects
        .iter()
        .filter_map(|o| match o.kind {
            ShapeKind::Instance { symbol, .. } => Some(symbol),
            _ => None,
        })
        .collect();
    let one_symbol = properties::common(symbols.iter().copied());
    if let Some(symbol) = one_symbol
        && let Some(name) = env.ws.project.symbol(symbol).map(|s| s.name.clone())
    {
        // The link color, with the symbol icon so it does not rely on
        // color alone.
        let text = tr!("props-instance-of", name = name.as_str());
        ui.label(RichText::new(format!("{} {name}", icons::SYMBOL)).color(color::LINK))
            .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
    }
    ui.horizontal_wrapped(|ui| {
        if let Some(symbol) = one_symbol
            && ui.add(secondary_button(&tr("cmd-edit-symbol"))).clicked()
        {
            env.ws.edit_symbol(symbol, env.now);
        }
        command_button(
            ui,
            cmds,
            CommandId::DetachInstance,
            &cmds.label(CommandId::DetachInstance),
        );
    });
    ui.label(
        RichText::new(tr("instance-look-in-symbol"))
            .size(typography::CAPTION + 1.0)
            .color(color::TEXT_SECONDARY),
    );
}

/// A secondary button running `id`, disabled with its reason as the
/// command is.
fn command_button(ui: &mut Ui, cmds: &mut CommandUi<'_>, id: CommandId, label: &str) {
    let enabled = cmds.enabled(id);
    let mut response = ui.add_enabled(enabled, secondary_button(label));
    response = response.on_hover_text(cmds.label(id));
    if !enabled && let Some(reason) = crate::state::disabled_reason_for(id, &cmds.edit) {
        response = response.on_disabled_hover_text(tr(reason));
    }
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, cmds.label(id)));
    if response.clicked() {
        cmds.push(id);
    }
}

// --- Fill and Stroke rows ---------------------------------------------------------

/// What a paint row says about `target`: the hex code of a solid color, the
/// linked swatch's name, "Linear" or "Radial", or "Mixed"; and whether it is
/// linked to the palette.
fn paint_value(ws: &Workspace, target: ColorTarget) -> (String, bool) {
    use tp_core::document::PaintKind;
    match panels::colors::current_kind(ws, target) {
        Some(PaintKind::Linear) => (tr("colors-linear"), false),
        Some(PaintKind::Radial) => (tr("colors-radial"), false),
        Some(PaintKind::Solid) => {
            if let Some(swatch) = ws
                .linked_swatch(target)
                .and_then(|id| ws.project.swatch(id))
            {
                return (swatch.name.clone(), true);
            }
            match panels::colors::current_color(ws, target) {
                Some(c) => (c.to_hex(), false),
                None => (tr("mixed"), false),
            }
        }
        // Kinds differ, or no selected object has a stroke (the Stroke
        // row's summary says so).
        None => match target {
            ColorTarget::Fill => (tr("mixed"), false),
            ColorTarget::Stroke => (String::new(), false),
        },
    }
}

/// The Fill and Stroke rows of the selection, or of the look of new
/// objects, with their popovers.
fn paint_rows(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    let ctx = ui.ctx().clone();
    let target = env.ws.panels.color_target;
    let (fill, stroke) = properties::selection_swatches(env.ws);

    let (value, linked) = paint_value(env.ws, ColorTarget::Fill);
    let fill_row = PaintRow::new(fill, &tr("props-fill"), &tr("props-fill-color"))
        .value(&value, linked)
        .target(target == ColorTarget::Fill)
        .show(ui);
    if fill_row.swatch.clicked() {
        open_color(&ctx, env, ColorTarget::Fill);
    }

    let (value, linked) = paint_value(env.ws, ColorTarget::Stroke);
    let value = if linked { value } else { String::new() };
    let summary = panels::stroke::summary(env.ws);
    let stroke_row = PaintRow::new(stroke, &tr("panel-stroke"), &tr("props-stroke-color"))
        .value(&value, linked)
        .target(target == ColorTarget::Stroke)
        .summary(&summary, &tr("stroke-options"))
        .show(ui);
    if stroke_row.swatch.clicked() {
        open_color(&ctx, env, ColorTarget::Stroke);
    }
    if stroke_row
        .summary
        .as_ref()
        .is_some_and(egui::Response::clicked)
    {
        Popover::toggle(&ctx, stroke_popover());
    }

    // X moves the color popover to the other row with the target.
    let anchor = match env.ws.panels.color_target {
        ColorTarget::Fill => fill_row.row,
        ColorTarget::Stroke => stroke_row.row,
    };
    Popover::new(color_popover(), popover_anchor(ui, anchor)).show(ui, |ui| {
        panels::colors::popover(ui, env);
    });
    Popover::new(stroke_popover(), popover_anchor(ui, stroke_row.row)).show(ui, |ui| {
        panels::stroke::popover(ui, env);
    });
}

/// The anchor of a popover opened from `row`: the row across the whole
/// inspector, so the popover opens beside the inspector, over the canvas.
fn popover_anchor(ui: &Ui, row: Rect) -> Rect {
    Rect::from_x_y_ranges(ui.clip_rect().x_range(), row.y_range())
}

/// Makes `target` the color target and opens the color popover on its
/// row; clicking the row it is open on closes it.
fn open_color(ctx: &egui::Context, env: &mut PanelEnv<'_>, target: ColorTarget) {
    let open_here = Popover::is_open(ctx, color_popover()) && env.ws.panels.color_target == target;
    env.ws.panels.color_target = target;
    if open_here {
        Popover::close(ctx);
    } else {
        Popover::open(ctx, color_popover());
    }
}

// --- Shadow row ---------------------------------------------------------------------

/// The Shadow row: the color and the summary ("8 / 8 · 8", or "Mixed",
/// the full values on hover) opening the shadow popover, and the remove
/// button at the row's right end; or "+ Add a shadow" when no selected
/// object has a shadow. Left out when no selected object can have a
/// shadow (groups, instances).
fn shadow_row(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    let shadows = env.ws.selected_shadows();
    if shadows.is_empty() {
        return;
    }
    if shadows.iter().all(Option::is_none) {
        let text = tr("inspector-add-shadow");
        let add = ui.add(
            egui::Button::new(
                RichText::new(&text)
                    .size(typography::CONTROL)
                    .color(color::TEXT_MUTED),
            )
            .frame(false)
            .min_size(egui::vec2(0.0, size::HIT_MIN)),
        );
        add.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, &text));
        if add.clicked() {
            env.ws.add_shadow(env.now);
        }
        return;
    }
    let colors = shadows.iter().map(|s| s.map(|s| s.color));
    let swatch = match properties::common(colors) {
        Some(Some(c)) => {
            SwatchColor::Solid(egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a))
        }
        _ => SwatchColor::Mixed,
    };
    let summary = panels::shadow::summary(&shadows);
    let full = panels::shadow::summary_full(&shadows);
    let name = tr("inspector-shadow");
    let remove_name = tr("inspector-shadow-remove");
    let open = Popover::is_open(ui.ctx(), shadow_popover());
    let row = PaintRow::new(swatch, &name, &name)
        .value(&summary, false)
        .hover_text(full.as_deref().unwrap_or(&name))
        .target(open)
        .action(icons::CLOSE, &remove_name)
        .show(ui);
    let remove = row.action.as_ref().is_some_and(|r| r.clicked());
    if row.swatch.clicked() {
        Popover::toggle(ui.ctx(), shadow_popover());
    }
    if remove {
        env.ws.remove_shadow(env.now);
    }
    Popover::new(shadow_popover(), popover_anchor(ui, row.row)).show(ui, |ui| {
        panels::shadow::popover(ui, env);
    });
}

/// The Width field of the selected lines, and the button opening their
/// dashes, caps and joins.
fn line_settings(
    ui: &mut Ui,
    env: &mut PanelEnv<'_>,
    lines: &[(f64, tp_core::document::LineStyle)],
) {
    let row = ui
        .horizontal(|ui| {
            properties::line_width(ui, env, lines);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let open = Popover::is_open(ui.ctx(), line_popover());
                if ui
                    .add(IconButton::new(icons::SETTINGS, &tr("line-settings")).selected(open))
                    .clicked()
                {
                    Popover::toggle(ui.ctx(), line_popover());
                }
            });
        })
        .response
        .rect;
    Popover::new(line_popover(), popover_anchor(ui, row)).show(ui, |ui| {
        properties::line_popover(ui, env);
    });
}
