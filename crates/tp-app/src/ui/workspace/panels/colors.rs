//! The color popover of the inspector's Fill and Stroke rows: paint kind
//! and gradient stops, picker, color models, hex, None, brand palette and
//! recent colors; and the palette lists shared with the Resources tab and
//! the Brand space.

use egui::{
    Color32, Grid, Margin, RichText, Stroke, StrokeKind, TextEdit, Ui, WidgetInfo, WidgetType,
};
use tp_core::document::{
    ColorStop, Frame, Gradient, GradientKind, Hsla, Hsva, MAX_STOPS, Paint, PaintKind, Rgba,
};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::{color, space};
use tp_ui::widgets::{
    ColorSwatch, FieldEvent, GradientBar, GradientBarEvent, Hsv, IconButton, NumericField,
    SegmentedControl, SwatchColor, alpha_slider, hue_slider, sv_square,
};

use super::PanelEnv;
use super::properties::{common, gradient_preview};
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

/// The picker's HSV for `c`.
pub(crate) fn to_widget_hsv(c: Rgba) -> Hsv {
    let h = Hsva::from(c);
    Hsv::new(h.h / 360.0, h.s, h.v, h.a)
}

/// The color a picker shows.
pub(crate) fn from_color32(c: Color32) -> Rgba {
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

/// The color popover's content for the current target: the paint kind
/// and the gradient editor, the picker, the color models and the hex code,
/// the brand palette, then the recent colors; "No stroke" for the stroke.
pub fn popover(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    let target = env.ws.panels.color_target;
    ui.horizontal(|ui| {
        ui.set_min_height(tp_ui::tokens::size::HIT_MIN);
        let title = tr(match target {
            ColorTarget::Fill => "props-fill",
            ColorTarget::Stroke => "panel-stroke",
        });
        ui.label(
            RichText::new(&title)
                .text_style(tp_ui::theme::label_strong_style())
                .color(color::TEXT_PRIMARY),
        );
        if target == ColorTarget::Stroke {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ColorSwatch::new(SwatchColor::None, &tr("colors-no-stroke"))
                    .show(ui)
                    .clicked()
                {
                    remove_stroke(env);
                }
            });
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
        sv_square(ui, &mut hsv, 128.0),
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
        .segment(ColorModel::Rgb, "", "RGB", None)
        .segment(ColorModel::Hsv, "", "HSV", None)
        .segment(ColorModel::Hsl, "", "HSL", None)
        .track(color::FIELD)
        .show(ui, model)
    {
        env.ws.panels.color_model = picked;
    }
    channel_fields(ui, env, current, base);
    hex_field(ui, env, current);
    palette(ui, env, current);
    recent_colors(ui, env);
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
            // Text only, spread over the popover, on a sunken track (the
            // popover is on the control surface).
            SegmentedControl::new()
                .named_segment(
                    Some(PaintKind::Solid),
                    "",
                    &tr("colors-solid"),
                    &tr("colors-solid-paint"),
                )
                .named_segment(
                    Some(PaintKind::Linear),
                    "",
                    &tr("colors-linear"),
                    &tr("colors-linear-gradient"),
                )
                .named_segment(
                    Some(PaintKind::Radial),
                    "",
                    &tr("colors-radial"),
                    &tr("colors-radial-gradient"),
                )
                .fill()
                .track(color::FIELD)
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
        ColorTarget::Fill => "undo-change-fill-gradient",
        ColorTarget::Stroke => "undo-change-stroke-gradient",
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
        let e = NumericField::new(
            &tr("colors-location"),
            &tr("colors-stop-location"),
            Some(location),
        )
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
        let e = NumericField::new(
            &tr("colors-angle"),
            &tr("colors-gradient-angle"),
            Some(g.angle(&frame)),
        )
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
            .add(IconButton::new(icons::REVERSE, &tr("colors-reverse")))
            .clicked()
        {
            edit_gradient(env, &g, true, |out, _| out.reverse());
        }
    });
    if g.kind == GradientKind::Radial {
        let e = NumericField::new(
            &tr("colors-aspect"),
            &tr("colors-aspect-ratio"),
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
        .map_selected_shapes("undo-remove-stroke", |o| o.stroke = None);
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
                    ("R", "colors-red", current.map(|c| f64::from(c.r)), 0usize),
                    ("G", "colors-green", current.map(|c| f64::from(c.g)), 1),
                    ("B", "colors-blue", current.map(|c| f64::from(c.b)), 2),
                ] {
                    let e = NumericField::new(label, &tr(name), value)
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
                    ("H", "colors-hue", h.map(|h| f64::from(h.h)), 0usize, 360.0),
                    (
                        "S",
                        "colors-saturation",
                        h.map(|h| f64::from(h.s) * 100.0),
                        1,
                        100.0,
                    ),
                    (
                        "V",
                        "colors-value",
                        h.map(|h| f64::from(h.v) * 100.0),
                        2,
                        100.0,
                    ),
                ] {
                    let e = NumericField::new(label, &tr(name), value)
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
                    ("H", "colors-hue", h.map(|h| f64::from(h.h)), 0usize, 360.0),
                    (
                        "S",
                        "colors-saturation",
                        h.map(|h| f64::from(h.s) * 100.0),
                        1,
                        100.0,
                    ),
                    (
                        "L",
                        "colors-lightness",
                        h.map(|h| f64::from(h.l) * 100.0),
                        2,
                        100.0,
                    ),
                ] {
                    let e = NumericField::new(label, &tr(name), value)
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
    let e = NumericField::new("A", &tr("colors-alpha"), alpha)
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
    hex_field_with(ui, env, current, apply_now);
}

/// The hex code of `current`, giving a valid code typed in to `apply`.
fn hex_field_with(
    ui: &mut Ui,
    env: &mut PanelEnv<'_>,
    current: Option<Rgba>,
    apply: fn(&mut PanelEnv<'_>, Rgba),
) {
    let id = ui.id().with("hex_field");
    let error_id = id.with("error");
    let mut buffer = ui
        .data(|d| d.get_temp::<String>(id))
        .unwrap_or_else(|| current.map(Rgba::to_hex).unwrap_or_default());
    let mut error = ui.data(|d| d.get_temp::<bool>(error_id)).unwrap_or(false);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(tr("colors-hex"))
                .small()
                .color(color::TEXT_SECONDARY),
        );
        tp_ui::widgets::remember_escape(ui, id.with("edit"));
        let response = ui.add(
            TextEdit::singleline(&mut buffer)
                .id(id.with("edit"))
                .font(egui::TextStyle::Monospace)
                .desired_width(96.0)
                .margin(Margin::symmetric(6, 3))
                .hint_text(if current.is_none() {
                    tr("mixed")
                } else {
                    String::new()
                }),
        );
        response.widget_info(|| {
            WidgetInfo::labeled(WidgetType::TextEdit, true, tr("colors-hex-color"))
        });
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
                        apply(env, c);
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
                .on_hover_text(tr("colors-hex-invalid"));
        }
    });
    ui.data_mut(|d| d.insert_temp(error_id, error));
}

fn recent_colors(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    if env.recent_colors.is_empty() {
        return;
    }
    ui.label(
        RichText::new(tr("colors-recent"))
            .small()
            .color(color::TEXT_MUTED),
    );
    let recent: Vec<Rgba> = env
        .recent_colors
        .iter()
        .map(|[r, g, b, a]| Rgba::with_alpha(*r, *g, *b, *a))
        .collect();
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = space::XXS;
        for c in recent {
            let name = tr!("colors-recent-item", color = label_for(c));
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

/// The brand palette with its heading and a "+ Add" link (Add to
/// Palette), in the link color as everything tied to the brand.
fn palette(ui: &mut Ui, env: &mut PanelEnv<'_>, current: Option<Rgba>) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(tr("colors-brand-palette"))
                .small()
                .color(color::TEXT_MUTED),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let name = tr("colors-add-to-palette");
            let ink = if current.is_some() {
                color::LINK
            } else {
                color::TEXT_DISABLED
            };
            let add = ui
                .add_enabled(
                    current.is_some(),
                    egui::Button::new(RichText::new(tr("colors-add-short")).small().color(ink))
                        .frame(false)
                        .min_size(egui::vec2(0.0, tp_ui::tokens::size::HIT_MIN)),
                )
                .on_hover_text(&name)
                .on_disabled_hover_text(tr("colors-differ"));
            add.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, current.is_some(), &name)
            });
            if add.clicked()
                && let Some(c) = current
            {
                add_color(env, c);
            }
        });
    });
    if env.ws.project.palette.is_empty() {
        palette_empty_hint(ui);
        return;
    }
    palette_swatches(ui, env, 22.0, true);
    linked_swatch_label(ui, env);
}

/// "Add to Palette" (+): saves the current target's color, or the selected
/// stop's, as a new swatch and links the target to it. Disabled while the
/// selection's colors differ.
pub fn add_to_palette_button(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    let current = current_color(env.ws, env.ws.panels.color_target);
    add_to_palette_button_for(ui, env, current);
}

fn add_to_palette_button_for(ui: &mut Ui, env: &mut PanelEnv<'_>, current: Option<Rgba>) {
    let add = ui.add_enabled(
        current.is_some(),
        IconButton::new(icons::ADD, &tr("colors-add-to-palette"))
            .disabled_reason(&tr("colors-differ")),
    );
    if add.clicked()
        && let Some(c) = current
    {
        add_color(env, c);
    }
}

/// The color Add to Palette saves: the current target's, or the selected
/// stop's; `None` while the selection's colors differ.
pub fn color_to_add(env: &PanelEnv<'_>) -> Option<Rgba> {
    current_color(env.ws, env.ws.panels.color_target)
}

/// Saves `c` as a new swatch and links the current target to it.
pub fn add_color(env: &mut PanelEnv<'_>, c: Rgba) {
    env.ws.panels.picker = None;
    env.ws
        .add_to_palette(env.ws.panels.color_target, c, env.now);
}

/// What an empty palette says.
pub fn palette_empty_hint(ui: &mut Ui) {
    ui.label(
        RichText::new(tr("colors-palette-empty"))
            .small()
            .color(color::TEXT_DISABLED),
    );
}

/// The project palette's swatches, `size` points wide, wrapped. Clicking a
/// swatch applies its color to the current target, or to the selected stop
/// when the target is a gradient, and links it to the swatch (one undo
/// step); the swatch the target is linked to has a ring. Each swatch's
/// context menu offers Edit Swatch…, Add to / Update in Library and Delete
/// Swatch. With `usage`, a swatch's tooltip also shows its usage. Shared by
/// every place that lists the palette.
pub fn palette_swatches(ui: &mut Ui, env: &mut PanelEnv<'_>, size: f32, usage: bool) {
    let target = env.ws.panels.color_target;
    let linked = env.ws.linked_swatch(target);
    swatches(ui, env, size, usage, linked, |env, id| {
        env.ws.panels.picker = None;
        env.ws.apply_swatch(target, id);
        env.ws.commit_pending(env.now);
    });
}

/// The palette's swatches, the one `linked` has a ring; a click calls
/// `pick` with the swatch.
fn swatches(
    ui: &mut Ui,
    env: &mut PanelEnv<'_>,
    size: f32,
    usage: bool,
    linked: Option<tp_core::document::SwatchId>,
    mut pick: impl FnMut(&mut PanelEnv<'_>, tp_core::document::SwatchId),
) {
    let palette = env.ws.project.palette.clone();
    let usages: Vec<Option<String>> = palette
        .iter()
        .map(|s| usage.then(|| crate::brand_ops::usage_label(env.ws.usage().swatch(s.id))))
        .collect();
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(space::XS, space::XS);
        for (s, usage) in palette.iter().zip(&usages) {
            let c = s.color;
            let swatch = SwatchColor::Solid(Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a));
            let mut widget = ColorSwatch::new(swatch, &s.name)
                .size(size)
                .selected(linked == Some(s.id));
            if let Some(usage) = usage {
                widget = widget.detail(usage);
            }
            let response = widget.show(ui);
            if response.clicked() {
                pick(env, s.id);
            }
            response.context_menu(|ui| swatch_menu(ui, env, s.id));
        }
    });
}

/// The menu of a palette swatch: Edit Swatch…, Add to / Update in Library
/// and Delete Swatch.
pub fn swatch_menu(ui: &mut Ui, env: &mut PanelEnv<'_>, id: tp_core::document::SwatchId) {
    if ui
        .add(tp_ui::widgets::MenuRow::new(&tr("colors-edit-swatch")))
        .clicked()
    {
        env.ws.start_swatch_edit(id);
        ui.close();
    }
    super::library_item(ui, env, crate::library::Element::Swatch(id));
    if ui
        .add(tp_ui::widgets::MenuRow::new(&tr("colors-delete-swatch")))
        .clicked()
    {
        env.ws.delete_swatch(id, env.now);
        ui.close();
    }
}

/// "Linked to <swatch>" with a link icon, when the current target (or the
/// selected stop) is linked to a swatch.
pub fn linked_swatch_label(ui: &mut Ui, env: &PanelEnv<'_>) {
    let linked = env.ws.linked_swatch(env.ws.panels.color_target);
    linked_label(ui, env, linked);
}

/// "Linked to <swatch>" with a link icon, for swatch `linked`.
fn linked_label(ui: &mut Ui, env: &PanelEnv<'_>, linked: Option<tp_core::document::SwatchId>) {
    let Some(s) = linked.and_then(|id| env.ws.project.swatch(id)) else {
        return;
    };
    let text = tr!("colors-linked-to", name = s.name.as_str());
    ui.label(
        RichText::new(format!("{} {text}", icons::LINKED))
            .small()
            .color(color::TEXT_SECONDARY),
    )
    .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
}

/// The color of the selected shadows in the shadow popover: the picker,
/// the hex code and the brand palette, whose swatches link the shadows as
/// they link a fill. `current` is the shadows' color (`None`: they
/// differ), `linked` the swatch they share.
pub fn shadow_color(
    ui: &mut Ui,
    env: &mut PanelEnv<'_>,
    current: Option<Rgba>,
    linked: Option<tp_core::document::SwatchId>,
) {
    let base = current.unwrap_or(Rgba::rgb(0, 0, 0));
    let mut hsv = match env.ws.panels.picker {
        Some((shown, hsv)) if Some(shown) == current => hsv,
        _ => to_widget_hsv(base),
    };
    let responses = [
        sv_square(ui, &mut hsv, 128.0),
        hue_slider(ui, &mut hsv),
        alpha_slider(ui, &mut hsv),
    ];
    if responses.iter().any(|r| r.changed()) {
        let color = from_color32(hsv.to_color32());
        env.ws.panels.picker = Some((color, hsv));
        env.ws.set_shadow_color(color);
    }
    if responses
        .iter()
        .any(|r| r.drag_stopped() || (r.changed() && !r.dragged()))
    {
        env.ws.commit_pending(env.now);
    }
    hex_field_with(ui, env, current, |env, c| {
        env.ws.panels.picker = None;
        env.ws.set_shadow_color(c);
        env.ws.commit_pending(env.now);
    });
    ui.label(
        RichText::new(tr("colors-brand-palette"))
            .small()
            .color(color::TEXT_MUTED),
    );
    if env.ws.project.palette.is_empty() {
        palette_empty_hint(ui);
        return;
    }
    swatches(ui, env, 22.0, true, linked, |env, id| {
        env.ws.panels.picker = None;
        env.ws.link_shadow(id);
        env.ws.commit_pending(env.now);
    });
    linked_label(ui, env, linked);
}
