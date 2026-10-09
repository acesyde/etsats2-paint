//! The Workshop's inspector, on the right of the canvas area: the settings
//! of the selection, section by section (header, Layout, Text, Appearance,
//! Polygon, Style, Image), each shown only when it applies; with nothing
//! selected, the active texture's properties and the look of new objects.
//! Colors and stroke settings open in popovers anchored to their rows.

use egui::{Frame, Margin, Panel, Rect, RichText, ScrollArea, Ui, WidgetInfo, WidgetType};
use tp_core::TexturePart;
use tp_core::document::{Object, ShapeKind};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::{color, size, space, typography};
use tp_ui::widgets::{IconButton, PaintRow, Popover, secondary_button};

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

/// The active texture (name, kind, size, the notices of a package update),
/// the look of new objects, the Text section for new texts with the Text
/// tool, and a hint.
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
        // stroke.
        if instances_only || only_images {
            return;
        }
        paint_rows(ui, env);
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
