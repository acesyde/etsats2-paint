//! Colors panel: fill/stroke target, picker, color models, hex, None,
//! recent colors and project palette.

use egui::{
    Color32, Grid, Margin, RichText, Stroke, StrokeKind, TextEdit, Ui, WidgetInfo, WidgetType,
};
use tp_core::document::{Hsla, Hsva, Rgba};
use tp_ui::icons;
use tp_ui::tokens::{color, space};
use tp_ui::widgets::{
    ColorSwatch, FieldEvent, FillOrStroke, FillStrokeSwatches, Hsv, IconButton, NumericField,
    SegmentedControl, SwatchColor, alpha_slider, hue_slider, sv_square,
};

use super::PanelEnv;
use super::properties::{common, selection_swatches};
use crate::workspace::{ColorModel, ColorTarget, Workspace};

/// The color being edited: shared by the selection, or the current style.
/// `None` when the selection's colors differ or no object has a stroke.
pub fn current_color(ws: &Workspace, target: ColorTarget) -> Option<Rgba> {
    if ws.selection.is_empty() {
        return Some(match target {
            ColorTarget::Fill => ws.style.fill,
            ColorTarget::Stroke => ws.style.stroke.color,
        });
    }
    let shapes = ws.selected_shapes();
    match target {
        ColorTarget::Fill => common(shapes.iter().map(|o| o.fill)),
        ColorTarget::Stroke => common(shapes.iter().filter_map(|o| o.stroke.map(|s| s.color))),
    }
}

fn to_widget_hsv(c: Rgba) -> Hsv {
    let h = Hsva::from(c);
    Hsv::new(h.h / 360.0, h.s, h.v, h.a)
}

fn from_color32(c: Color32) -> Rgba {
    let [r, g, b, a] = c.to_srgba_unmultiplied();
    Rgba::with_alpha(r, g, b, a)
}

fn label_for(c: Rgba) -> String {
    c.to_hex()
}

/// Applies a color and commits it as one step.
fn apply_now(env: &mut PanelEnv<'_>, color: Rgba) {
    let target = env.ws.panels.color_target;
    env.ws.panels.picker = None;
    env.ws.apply_color(target, color);
    env.ws.commit_pending(env.now);
}

pub fn show(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    let target = env.ws.panels.color_target;
    let current = current_color(env.ws, target);

    // Target swatches (+ None for strokes).
    let (fill, stroke) = selection_swatches(env.ws);
    ui.horizontal(|ui| {
        let pair = FillStrokeSwatches {
            fill,
            stroke,
            active: match target {
                ColorTarget::Fill => FillOrStroke::Fill,
                ColorTarget::Stroke => FillOrStroke::Stroke,
            },
        };
        if let Some(which) = pair.show(ui) {
            env.ws.panels.color_target = match which {
                FillOrStroke::Fill => ColorTarget::Fill,
                FillOrStroke::Stroke => ColorTarget::Stroke,
            };
        }
        if target == ColorTarget::Stroke {
            ui.add_space(space::SM);
            if ColorSwatch::new(SwatchColor::None, "No stroke")
                .show(ui)
                .clicked()
            {
                remove_stroke(env);
            }
        }
    });

    // Picker.
    let base = current.unwrap_or(Rgba::rgb(128, 128, 128));
    let mut hsv = match env.ws.panels.picker {
        Some((shown, hsv)) if Some(shown) == current => hsv,
        _ => to_widget_hsv(base),
    };
    let responses = [
        sv_square(ui, &mut hsv, 120.0),
        hue_slider(ui, &mut hsv),
        alpha_slider(ui, &mut hsv),
    ];
    if responses.iter().any(|r| r.changed()) {
        let color = from_color32(hsv.to_color32());
        env.ws.panels.picker = Some((color, hsv));
        env.ws.apply_color(target, color);
    }
    let finished = responses
        .iter()
        .any(|r| r.drag_stopped() || (r.changed() && !r.dragged()));
    if finished {
        env.ws.commit_pending(env.now);
    }

    // Color model.
    let model = env.ws.panels.color_model;
    if let Some(picked) = SegmentedControl::new()
        .segment(ColorModel::Rgb, icons::COLORS, "RGB", None)
        .segment(ColorModel::Hsv, icons::COLORS, "HSV", None)
        .segment(ColorModel::Hsl, icons::COLORS, "HSL", None)
        .show(ui, model)
    {
        env.ws.panels.color_model = picked;
    }
    channel_fields(ui, env, current, base);
    hex_field(ui, env, current);
    recent_colors(ui, env);
    palette(ui, env, current);
}

fn remove_stroke(env: &mut PanelEnv<'_>) {
    if env.ws.selection.is_empty() {
        env.ws.style.stroke_enabled = false;
        return;
    }
    env.ws
        .map_selected_shapes("Remove Stroke", |o| o.stroke = None);
    env.ws.commit_pending(env.now);
}

fn channel_fields(ui: &mut Ui, env: &mut PanelEnv<'_>, current: Option<Rgba>, base: Rgba) {
    let target = env.ws.panels.color_target;
    let model = env.ws.panels.color_model;
    let mut changes: Vec<ChannelChange> = Vec::new();
    let alpha = current.map(|c| f64::from(c.a) / 2.55);
    Grid::new("color_channels")
        .num_columns(4)
        .spacing([space::SM, space::XS])
        .show(ui, |ui| match model {
            ColorModel::Rgb => {
                for (label, name, value, set) in [
                    ("R", "Red", current.map(|c| f64::from(c.r)), 0usize),
                    ("G", "Green", current.map(|c| f64::from(c.g)), 1),
                    ("B", "Blue", current.map(|c| f64::from(c.b)), 2),
                ] {
                    let e = NumericField::new(label, name, value)
                        .range(0.0..=255.0)
                        .width(36.0)
                        .show(ui);
                    changes.push((
                        e,
                        Box::new(move |v| {
                            let v = v.round() as u8;
                            let mut c = base;
                            match set {
                                0 => c.r = v,
                                1 => c.g = v,
                                _ => c.b = v,
                            }
                            c
                        }),
                    ));
                }
                alpha_field(ui, &mut changes, alpha, base);
            }
            ColorModel::Hsv => {
                let h = current.map(Hsva::from);
                for (label, name, value, set, max) in [
                    ("H", "Hue", h.map(|h| f64::from(h.h)), 0usize, 360.0),
                    (
                        "S",
                        "Saturation",
                        h.map(|h| f64::from(h.s) * 100.0),
                        1,
                        100.0,
                    ),
                    ("V", "Value", h.map(|h| f64::from(h.v) * 100.0), 2, 100.0),
                ] {
                    let e = NumericField::new(label, name, value)
                        .range(0.0..=max)
                        .width(36.0)
                        .show(ui);
                    changes.push((
                        e,
                        Box::new(move |v| {
                            let mut h = Hsva::from(base);
                            match set {
                                0 => h.h = v as f32,
                                1 => h.s = (v / 100.0) as f32,
                                _ => h.v = (v / 100.0) as f32,
                            }
                            Rgba::from(h)
                        }),
                    ));
                }
                alpha_field(ui, &mut changes, alpha, base);
            }
            ColorModel::Hsl => {
                let h = current.map(Hsla::from);
                for (label, name, value, set, max) in [
                    ("H", "Hue", h.map(|h| f64::from(h.h)), 0usize, 360.0),
                    (
                        "S",
                        "Saturation",
                        h.map(|h| f64::from(h.s) * 100.0),
                        1,
                        100.0,
                    ),
                    (
                        "L",
                        "Lightness",
                        h.map(|h| f64::from(h.l) * 100.0),
                        2,
                        100.0,
                    ),
                ] {
                    let e = NumericField::new(label, name, value)
                        .range(0.0..=max)
                        .width(36.0)
                        .show(ui);
                    changes.push((
                        e,
                        Box::new(move |v| {
                            let mut h = Hsla::from(base);
                            match set {
                                0 => h.h = v as f32,
                                1 => h.s = (v / 100.0) as f32,
                                _ => h.l = (v / 100.0) as f32,
                            }
                            Rgba::from(h)
                        }),
                    ));
                }
                alpha_field(ui, &mut changes, alpha, base);
            }
        });
    for (event, make) in changes {
        match event {
            FieldEvent::Live(v) => {
                env.ws.panels.picker = None;
                env.ws.apply_color(target, make(v));
            }
            FieldEvent::Commit(v) => apply_now(env, make(v)),
            FieldEvent::Revert => env.ws.cancel_pending(),
            FieldEvent::None => {}
        }
    }
}

type ChannelChange = (FieldEvent, Box<dyn Fn(f64) -> Rgba>);

fn alpha_field(ui: &mut Ui, changes: &mut Vec<ChannelChange>, alpha: Option<f64>, base: Rgba) {
    let e = NumericField::new("A", "Alpha", alpha)
        .suffix("%")
        .range(0.0..=100.0)
        .width(36.0)
        .show(ui);
    changes.push((
        e,
        Box::new(move |v| {
            let mut c = base;
            c.a = (v * 2.55).round().clamp(0.0, 255.0) as u8;
            c
        }),
    ));
    ui.end_row();
}

fn hex_field(ui: &mut Ui, env: &mut PanelEnv<'_>, current: Option<Rgba>) {
    let id = ui.id().with("hex_field");
    let error_id = id.with("error");
    let mut buffer = ui
        .data(|d| d.get_temp::<String>(id))
        .unwrap_or_else(|| current.map(Rgba::to_hex).unwrap_or_default());
    let mut error = ui.data(|d| d.get_temp::<bool>(error_id)).unwrap_or(false);
    ui.horizontal(|ui| {
        ui.label(RichText::new("Hex").small().color(color::TEXT_SECONDARY));
        tp_ui::widgets::remember_escape(ui, id.with("edit"));
        let response = ui.add(
            TextEdit::singleline(&mut buffer)
                .id(id.with("edit"))
                .desired_width(96.0)
                .margin(Margin::symmetric(6, 3))
                .hint_text(if current.is_none() { "Mixed" } else { "" }),
        );
        response.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, "Hex color"));
        if response.changed() {
            error = false;
        }
        if response.has_focus() {
            ui.data_mut(|d| d.insert_temp(id, buffer.clone()));
        }
        if response.lost_focus() {
            ui.data_mut(|d| d.remove::<String>(id));
            if !tp_ui::widgets::take_escape(ui, id.with("edit")) {
                match Rgba::from_hex(&buffer) {
                    Some(c) => {
                        error = false;
                        apply_now(env, c);
                    }
                    None => error = !buffer.trim().is_empty(),
                }
            }
        }
        if error {
            ui.painter().rect_stroke(
                response.rect.expand(1.0),
                3,
                Stroke::new(1.5, color::ERROR),
                StrokeKind::Outside,
            );
            ui.label(icons::rich(icons::WARNING).color(color::ERROR))
                .on_hover_text("Invalid hex color. Use #RGB, #RRGGBB or #RRGGBBAA.");
        }
    });
    ui.data_mut(|d| d.insert_temp(error_id, error));
}

fn recent_colors(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    if env.recent_colors.is_empty() {
        return;
    }
    ui.label(RichText::new("Recent").small().color(color::TEXT_SECONDARY));
    let recent: Vec<Rgba> = env
        .recent_colors
        .iter()
        .map(|[r, g, b, a]| Rgba::with_alpha(*r, *g, *b, *a))
        .collect();
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = space::XXS;
        for c in recent {
            let name = format!("Recent color {}", label_for(c));
            let swatch = SwatchColor::Solid(Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a));
            if ColorSwatch::new(swatch, &name)
                .size(18.0)
                .show(ui)
                .clicked()
            {
                apply_now(env, c);
            }
        }
    });
}

fn palette(ui: &mut Ui, env: &mut PanelEnv<'_>, current: Option<Rgba>) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Palette")
                .small()
                .color(color::TEXT_SECONDARY),
        );
        let add = ui.add_enabled(
            current.is_some(),
            IconButton::new(icons::ADD, "Add to Palette")
                .disabled_reason("Colors differ in the selection."),
        );
        if add.clicked()
            && let Some(c) = current
        {
            env.ws.edit("Add to Palette", env.now, false, |project, _| {
                project.add_to_palette(c);
            });
        }
    });
    let palette = env.ws.project.palette.clone();
    if palette.is_empty() {
        ui.label(
            RichText::new("Save colors you reuse with +.")
                .small()
                .color(color::TEXT_DISABLED),
        );
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = space::XXS;
        for c in palette {
            let name = format!("Palette color {}", label_for(c));
            let swatch = SwatchColor::Solid(Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a));
            let response = ColorSwatch::new(swatch, &name).size(18.0).show(ui);
            if response.clicked() {
                apply_now(env, c);
            }
            response.context_menu(|ui| {
                if ui
                    .add(tp_ui::widgets::MenuRow::new("Remove from Palette"))
                    .clicked()
                {
                    env.ws
                        .edit("Remove from Palette", env.now, false, |project, _| {
                            project.palette.retain(|p| *p != c);
                        });
                    ui.close();
                }
            });
        }
    });
}
