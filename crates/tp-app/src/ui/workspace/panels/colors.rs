//! Colors panel: fill/stroke target, paint kind and gradient stops, picker,
//! color models, hex, None, recent colors and project palette.

use egui::{
    Color32, Grid, Margin, RichText, Stroke, StrokeKind, TextEdit, Ui, WidgetInfo, WidgetType,
};
use tp_core::document::{
    ColorStop, Frame, Gradient, GradientKind, Hsla, Hsva, MAX_STOPS, Paint, PaintKind, Rgba,
};
use tp_ui::icons;
use tp_ui::tokens::{color, space};
use tp_ui::widgets::{
    ColorSwatch, FieldEvent, FillOrStroke, FillStrokeSwatches, GradientBar, GradientBarEvent, Hsv,
    IconButton, NumericField, SegmentedControl, SwatchColor, alpha_slider, hue_slider, sv_square,
};

use super::PanelEnv;
use super::properties::{common, gradient_preview, selection_swatches};
use crate::workspace::{ColorModel, ColorTarget, Workspace, paint_of};

/// The color being edited: the selected stop of the gradient being edited,
/// else the solid color shared by the selection (or the current style).
/// `None` when the selection's colors differ or no object has a stroke.
pub fn current_color(ws: &Workspace, target: ColorTarget) -> Option<Rgba> {
    if let Some(Paint::Gradient(g)) = ws.edited_paint(target) {
        return Some(g.stops()[selected_stop(ws, &g)].color);
    }
    if ws.selection.is_empty() {
        return ws.edited_paint(target).map(|p| p.first_color());
    }
    let shapes = ws.selected_shapes();
    common(
        shapes
            .iter()
            .filter_map(|o| paint_of(o, target))
            .map(|p| p.solid_color()),
    )
    .flatten()
}

/// The kind of paint shared by the selection (or of the current style);
/// `None` when kinds differ or no object has a stroke.
pub fn current_kind(ws: &Workspace, target: ColorTarget) -> Option<PaintKind> {
    if ws.selection.is_empty() {
        return ws.edited_paint(target).map(|p| p.kind());
    }
    common(
        ws.selected_shapes()
            .iter()
            .filter_map(|o| paint_of(o, target))
            .map(|p| p.kind()),
    )
}

/// The selected stop of `g`, clamped.
fn selected_stop(ws: &Workspace, g: &Gradient) -> usize {
    ws.panels.gradient_stop.min(g.stops().len() - 1)
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

    paint_kind(ui, env);
    if let Some(Paint::Gradient(g)) = env.ws.edited_paint(target) {
        gradient_editor(ui, env, &g);
    }
    let current = current_color(env.ws, target);

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

/// Solid / Linear / Radial control for the current target.
fn paint_kind(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    let target = env.ws.panels.color_target;
    if env.ws.edited_paint(target).is_none() {
        return;
    }
    let kind = current_kind(env.ws, target);
    let picked = ui
        .push_id("paint_kind", |ui| {
            SegmentedControl::new()
                .named_segment(
                    Some(PaintKind::Solid),
                    icons::PAINT_SOLID,
                    "Solid",
                    "Solid paint",
                )
                .named_segment(
                    Some(PaintKind::Linear),
                    icons::PAINT_LINEAR,
                    "Linear",
                    "Linear gradient",
                )
                .named_segment(
                    Some(PaintKind::Radial),
                    icons::PAINT_RADIAL,
                    "Radial",
                    "Radial gradient",
                )
                .show(ui, kind)
        })
        .inner;
    if let Some(Some(kind)) = picked {
        env.ws.panels.picker = None;
        env.ws.set_paint_kind(target, kind);
        env.ws.commit_pending(env.now);
    }
}

fn gradient_label(target: ColorTarget) -> &'static str {
    match target {
        ColorTarget::Fill => "Change Fill Gradient",
        ColorTarget::Stroke => "Change Stroke Gradient",
    }
}

/// Applies `base` changed by `f` (given each object's frame) to the target
/// of every selected shape, or of the current style; `commit` records it.
fn edit_gradient(
    env: &mut PanelEnv<'_>,
    base: &Gradient,
    commit: bool,
    f: impl Fn(&mut Gradient, &Frame),
) {
    let target = env.ws.panels.color_target;
    env.ws
        .map_paints(target, gradient_label(target), |_, frame| {
            let mut g = *base;
            f(&mut g, frame);
            Paint::Gradient(g)
        });
    if commit {
        env.ws.commit_pending(env.now);
    }
}

/// `g` with stop `index` moved to `offset`, and the stop's new index.
pub fn move_stop(g: &Gradient, index: usize, offset: f32) -> (Gradient, usize) {
    let mut stops = g.stops().to_vec();
    let mut moved = stops.remove(index);
    moved.offset = offset.clamp(0.0, 1.0);
    let at = stops.iter().filter(|s| s.offset <= moved.offset).count();
    stops.insert(at, moved);
    let mut out = *g;
    out.set_stops(&stops);
    (out, at)
}

/// Gradient bar, stop location, angle, aspect and Reverse.
fn gradient_editor(ui: &mut Ui, env: &mut PanelEnv<'_>, g: &Gradient) {
    let target = env.ws.panels.color_target;
    let selected = selected_stop(env.ws, g);
    let preview = gradient_preview(g);
    let event = GradientBar::new(&preview, selected, target == ColorTarget::Fill).show(ui);
    match event {
        Some(GradientBarEvent::Select(i)) => {
            env.ws.panels.gradient_stop = i;
            env.ws.panels.picker = None;
        }
        Some(GradientBarEvent::Move {
            index,
            offset,
            commit,
        }) => {
            let (moved, at) = move_stop(g, index, offset);
            env.ws.panels.gradient_stop = at;
            edit_gradient(env, g, commit, |out, _| out.set_stops(moved.stops()));
        }
        Some(GradientBarEvent::Add { offset }) if g.stops().len() < MAX_STOPS => {
            let mut stops = g.stops().to_vec();
            let stop = ColorStop::new(offset, g.color_at(offset));
            let at = stops.iter().filter(|s| s.offset <= offset).count();
            stops.insert(at, stop);
            env.ws.panels.gradient_stop = at;
            env.ws.panels.picker = None;
            edit_gradient(env, g, true, |out, _| out.set_stops(&stops));
        }
        Some(GradientBarEvent::Delete(i)) if g.stops().len() > 2 => {
            let mut stops = g.stops().to_vec();
            stops.remove(i);
            env.ws.panels.gradient_stop = i.min(stops.len() - 1);
            env.ws.panels.picker = None;
            edit_gradient(env, g, true, |out, _| out.set_stops(&stops));
        }
        _ => {}
    }
    // The frame the gradient is measured in: the first selected shape's.
    let frame = env
        .ws
        .selected_shapes()
        .iter()
        .find(|o| paint_of(o, target).is_some())
        .map_or_else(
            || Frame::from_rect(tp_core::kurbo::Rect::new(0.0, 0.0, 1.0, 1.0)),
            |o| o.frame,
        );
    let g = match env.ws.edited_paint(target) {
        Some(Paint::Gradient(g)) => g,
        _ => return,
    };
    let selected = selected_stop(env.ws, &g);
    ui.horizontal(|ui| {
        let location = f64::from(g.stops()[selected].offset) * 100.0;
        let e = NumericField::new("Location", "Stop location", Some(location))
            .suffix("%")
            .decimals(0)
            .range(0.0..=100.0)
            .width(40.0)
            .show(ui);
        let at = |v: f64| (v / 100.0) as f32;
        match e {
            FieldEvent::Live(v) | FieldEvent::Commit(v) => {
                let (moved, index) = move_stop(&g, selected, at(v));
                env.ws.panels.gradient_stop = index;
                let commit = matches!(e, FieldEvent::Commit(_));
                edit_gradient(env, &g, commit, |out, _| out.set_stops(moved.stops()));
            }
            FieldEvent::Revert => env.ws.cancel_pending(),
            FieldEvent::None => {}
        }
        let e = NumericField::new("Angle", "Gradient angle", Some(g.angle(&frame)))
            .suffix("°")
            .decimals(0)
            .range(-180.0..=180.0)
            .width(40.0)
            .show(ui);
        match e {
            FieldEvent::Live(v) | FieldEvent::Commit(v) => {
                let commit = matches!(e, FieldEvent::Commit(_));
                edit_gradient(env, &g, commit, |out, frame| out.set_angle(frame, v));
            }
            FieldEvent::Revert => env.ws.cancel_pending(),
            FieldEvent::None => {}
        }
        if ui
            .add(IconButton::new(icons::REVERSE, "Reverse gradient"))
            .clicked()
        {
            edit_gradient(env, &g, true, |out, _| out.reverse());
        }
    });
    if g.kind == GradientKind::Radial {
        let e = NumericField::new(
            "Aspect",
            "Gradient aspect ratio",
            Some(g.aspect(&frame) * 100.0),
        )
        .suffix("%")
        .decimals(0)
        .range(1.0..=1000.0)
        .width(40.0)
        .show(ui);
        match e {
            FieldEvent::Live(v) | FieldEvent::Commit(v) => {
                let commit = matches!(e, FieldEvent::Commit(_));
                edit_gradient(env, &g, commit, |out, frame| {
                    out.set_aspect(frame, v / 100.0);
                });
            }
            FieldEvent::Revert => env.ws.cancel_pending(),
            FieldEvent::None => {}
        }
    }
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
