//! Color widgets: swatches, fill/stroke pair, saturation/value square, hue
//! and alpha sliders.

use egui::{
    Color32, CornerRadius, Key, Mesh, Painter, Pos2, Rect, Response, Sense, Shape, Stroke,
    StrokeKind, Ui, Vec2, WidgetInfo, WidgetType,
};

use super::paint_focus_ring;
use crate::tokens::{color, radius, stroke};

/// What a swatch shows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SwatchColor {
    Solid(Color32),
    /// No color (e.g. no stroke): white with a red diagonal.
    None,
    /// Differing values in a multi-selection.
    Mixed,
}

/// Paints a checkerboard (for transparency) inside `rect`.
pub fn paint_checkerboard(painter: &Painter, rect: Rect, cell: f32) {
    painter.rect_filled(rect, 0, Color32::from_gray(200));
    let cols = (rect.width() / cell).ceil() as i32;
    let rows = (rect.height() / cell).ceil() as i32;
    for y in 0..rows {
        for x in 0..cols {
            if (x + y) % 2 == 0 {
                let min = rect.min + Vec2::new(x as f32 * cell, y as f32 * cell);
                let r = Rect::from_min_size(min, Vec2::splat(cell)).intersect(rect);
                painter.rect_filled(r, 0, Color32::from_gray(150));
            }
        }
    }
}

fn paint_swatch(painter: &Painter, rect: Rect, swatch: SwatchColor) {
    match swatch {
        SwatchColor::Solid(c) => {
            if c.a() < 255 {
                paint_checkerboard(painter, rect, 4.0);
            }
            painter.rect_filled(rect, 0, c);
        }
        SwatchColor::None => {
            painter.rect_filled(rect, 0, Color32::WHITE);
            painter.line_segment(
                [rect.left_bottom(), rect.right_top()],
                Stroke::new(2.0, Color32::from_rgb(0xE0, 0x30, 0x30)),
            );
        }
        SwatchColor::Mixed => {
            painter.rect_filled(rect, 0, color::SURFACE_3);
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "?",
                egui::FontId::proportional(rect.height() * 0.6),
                color::TEXT_SECONDARY,
            );
        }
    }
    painter.rect_stroke(
        rect,
        0,
        Stroke::new(1.0, color::BORDER_STRONG),
        StrokeKind::Inside,
    );
}

/// A clickable color square.
pub struct ColorSwatch<'a> {
    color: SwatchColor,
    name: &'a str,
    size: f32,
    selected: bool,
}

impl<'a> ColorSwatch<'a> {
    pub fn new(color: SwatchColor, name: &'a str) -> Self {
        Self {
            color,
            name,
            size: 20.0,
            selected: false,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, response) =
            ui.allocate_exact_size(Vec2::splat(self.size.max(20.0)), Sense::click());
        let swatch_rect = Rect::from_center_size(rect.center(), Vec2::splat(self.size));
        response.widget_info(|| {
            WidgetInfo::selected(WidgetType::Button, true, self.selected, self.name)
        });
        let painter = ui.painter();
        paint_swatch(painter, swatch_rect, self.color);
        if self.selected || response.hovered() {
            painter.rect_stroke(
                swatch_rect.expand(2.0),
                CornerRadius::same(radius::SM),
                Stroke::new(
                    if self.selected {
                        stroke::FOCUS + 0.5
                    } else {
                        1.0
                    },
                    color::ACCENT,
                ),
                StrokeKind::Outside,
            );
        }
        paint_focus_ring(ui, swatch_rect, &response, radius::SM);
        response.on_hover_text(self.name)
    }
}

/// Which swatch of a [`FillStrokeSwatches`] pair was clicked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillOrStroke {
    Fill,
    Stroke,
}

/// Illustrator-style overlapping fill (solid) and stroke (ring) swatches; the
/// active one is drawn in front with an accent outline.
pub struct FillStrokeSwatches {
    pub fill: SwatchColor,
    pub stroke: SwatchColor,
    pub active: FillOrStroke,
}

impl FillStrokeSwatches {
    /// Returns the swatch clicked this frame, if any.
    pub fn show(self, ui: &mut Ui) -> Option<FillOrStroke> {
        let size = 30.0;
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(size + 14.0), Sense::hover());
        let fill_rect = Rect::from_min_size(rect.min, Vec2::splat(size));
        let stroke_rect = Rect::from_min_size(rect.min + Vec2::splat(14.0), Vec2::splat(size));
        let fill_resp = ui.interact(fill_rect, ui.id().with("fill_swatch"), Sense::click());
        let stroke_resp = ui.interact(stroke_rect, ui.id().with("stroke_swatch"), Sense::click());
        let fill_active = self.active == FillOrStroke::Fill;
        fill_resp.widget_info(|| {
            WidgetInfo::selected(WidgetType::RadioButton, true, fill_active, "Fill")
        });
        stroke_resp.widget_info(|| {
            WidgetInfo::selected(WidgetType::RadioButton, true, !fill_active, "Stroke")
        });

        let painter = ui.painter();
        let draw_fill = |p: &Painter, active: bool| {
            paint_swatch(p, fill_rect, self.fill);
            if active {
                p.rect_stroke(
                    fill_rect.expand(1.5),
                    0,
                    Stroke::new(2.0, color::ACCENT),
                    StrokeKind::Outside,
                );
            }
        };
        let draw_stroke = |p: &Painter, active: bool| {
            // A ring: the stroke color around a hollow center.
            paint_swatch(p, stroke_rect, self.stroke);
            p.rect_filled(stroke_rect.shrink(8.0), 0, color::SURFACE_1);
            p.rect_stroke(
                stroke_rect.shrink(8.0),
                0,
                Stroke::new(1.0, color::BORDER_STRONG),
                StrokeKind::Inside,
            );
            if active {
                p.rect_stroke(
                    stroke_rect.expand(1.5),
                    0,
                    Stroke::new(2.0, color::ACCENT),
                    StrokeKind::Outside,
                );
            }
        };
        if fill_active {
            draw_stroke(painter, false);
            draw_fill(painter, true);
        } else {
            draw_fill(painter, false);
            draw_stroke(painter, true);
        }
        let fill_clicked = fill_resp.on_hover_text("Fill (X to switch)").clicked();
        let stroke_clicked = stroke_resp.on_hover_text("Stroke (X to switch)").clicked();
        // The front swatch wins where they overlap.
        match (fill_clicked, stroke_clicked) {
            (true, true) => Some(self.active),
            (true, false) => Some(FillOrStroke::Fill),
            (false, true) => Some(FillOrStroke::Stroke),
            _ => None,
        }
    }
}

/// sRGB hue/saturation/value/alpha, all in 0..=1 (the picker's model; it
/// matches the HSV values shown in panels, unlike egui's linear `Hsv`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hsv {
    pub h: f32,
    pub s: f32,
    pub v: f32,
    pub a: f32,
}

impl Hsv {
    pub fn new(h: f32, s: f32, v: f32, a: f32) -> Self {
        Self { h, s, v, a }
    }

    /// The color as sRGB with straight alpha.
    pub fn to_color32(self) -> Color32 {
        let chroma = self.v * self.s;
        let h = (self.h.rem_euclid(1.0)) * 6.0;
        let x = chroma * (1.0 - (h % 2.0 - 1.0).abs());
        let (r, g, b) = match h as u32 {
            0 => (chroma, x, 0.0),
            1 => (x, chroma, 0.0),
            2 => (0.0, chroma, x),
            3 => (0.0, x, chroma),
            4 => (x, 0.0, chroma),
            _ => (chroma, 0.0, x),
        };
        let m = self.v - chroma;
        let q = |c: f32| ((c + m).clamp(0.0, 1.0) * 255.0).round() as u8;
        Color32::from_rgba_unmultiplied(
            q(r),
            q(g),
            q(b),
            (self.a.clamp(0.0, 1.0) * 255.0).round() as u8,
        )
    }
}

fn opaque(mut hsv: Hsv) -> Hsv {
    hsv.a = 1.0;
    hsv
}

fn handle(painter: &Painter, at: Pos2) {
    painter.circle_stroke(at, 6.0, Stroke::new(3.0, Color32::BLACK));
    painter.circle_stroke(at, 6.0, Stroke::new(1.5, Color32::WHITE));
}

fn keyboard_step(ui: &Ui, response: &Response) -> (f32, f32) {
    if !response.has_focus() {
        return (0.0, 0.0);
    }
    ui.input(|i| {
        let step = if i.modifiers.shift { 0.1 } else { 0.01 };
        let mut d = (0.0, 0.0);
        if i.key_pressed(Key::ArrowLeft) {
            d.0 -= step;
        }
        if i.key_pressed(Key::ArrowRight) {
            d.0 += step;
        }
        if i.key_pressed(Key::ArrowUp) {
            d.1 += step;
        }
        if i.key_pressed(Key::ArrowDown) {
            d.1 -= step;
        }
        d
    })
}

/// Saturation (x) / value (y) square for the current hue.
pub fn sv_square(ui: &mut Ui, hsva: &mut Hsv, height: f32) -> Response {
    let width = ui.available_width();
    let (rect, mut response) =
        ui.allocate_exact_size(Vec2::new(width, height), Sense::click_and_drag());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Slider, true, "Saturation and value"));
    let painter = ui.painter_at(rect.expand(8.0));

    // White → hue horizontally, then transparent → black vertically.
    let hue = opaque(Hsv::new(hsva.h, 1.0, 1.0, 1.0)).to_color32();
    let mut mesh = Mesh::default();
    for (pos, col) in [
        (rect.left_top(), Color32::WHITE),
        (rect.right_top(), hue),
        (rect.right_bottom(), hue),
        (rect.left_bottom(), Color32::WHITE),
    ] {
        mesh.colored_vertex(pos, col);
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    let base = mesh.vertices.len() as u32;
    for (pos, col) in [
        (rect.left_top(), Color32::TRANSPARENT),
        (rect.right_top(), Color32::TRANSPARENT),
        (rect.right_bottom(), Color32::BLACK),
        (rect.left_bottom(), Color32::BLACK),
    ] {
        mesh.colored_vertex(pos, col);
    }
    mesh.add_triangle(base, base + 1, base + 2);
    mesh.add_triangle(base, base + 2, base + 3);
    painter.add(Shape::mesh(mesh));
    painter.rect_stroke(
        rect,
        0,
        Stroke::new(1.0, color::BORDER_STRONG),
        StrokeKind::Inside,
    );

    if let Some(pos) = response.interact_pointer_pos()
        && (response.dragged() || response.clicked() || response.drag_started())
    {
        hsva.s = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        hsva.v = (1.0 - (pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
        response.mark_changed();
    }
    let (dx, dy) = keyboard_step(ui, &response);
    if dx != 0.0 || dy != 0.0 {
        hsva.s = (hsva.s + dx).clamp(0.0, 1.0);
        hsva.v = (hsva.v + dy).clamp(0.0, 1.0);
        response.mark_changed();
    }
    handle(
        &painter,
        Pos2::new(
            rect.left() + hsva.s * rect.width(),
            rect.top() + (1.0 - hsva.v) * rect.height(),
        ),
    );
    paint_focus_ring(ui, rect, &response, 0);
    response
}

fn slider_strip(ui: &mut Ui, name: &str) -> (Rect, Response) {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 14.0),
        Sense::click_and_drag(),
    );
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Slider, true, name));
    (rect, response)
}

fn strip_value(ui: &Ui, response: &mut Response, rect: Rect, value: &mut f32) {
    if let Some(pos) = response.interact_pointer_pos()
        && (response.dragged() || response.clicked() || response.drag_started())
    {
        *value = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        response.mark_changed();
    }
    let (dx, _) = keyboard_step(ui, response);
    if dx != 0.0 {
        *value = (*value + dx).clamp(0.0, 1.0);
        response.mark_changed();
    }
}

fn strip_handle(painter: &Painter, rect: Rect, t: f32) {
    let x = rect.left() + t * rect.width();
    let r = Rect::from_center_size(
        Pos2::new(x, rect.center().y),
        Vec2::new(6.0, rect.height() + 4.0),
    );
    painter.rect(
        r,
        2,
        Color32::WHITE,
        Stroke::new(1.0, Color32::BLACK),
        StrokeKind::Inside,
    );
}

/// Hue slider (0..1 maps to 0..360°).
pub fn hue_slider(ui: &mut Ui, hsva: &mut Hsv) -> Response {
    let (rect, mut response) = slider_strip(ui, "Hue");
    let painter = ui.painter();
    let mut mesh = Mesh::default();
    const STEPS: u32 = 6;
    for i in 0..=STEPS {
        let t = i as f32 / STEPS as f32;
        let c = Hsv::new(t, 1.0, 1.0, 1.0).to_color32();
        let x = rect.left() + t * rect.width();
        mesh.colored_vertex(Pos2::new(x, rect.top()), c);
        mesh.colored_vertex(Pos2::new(x, rect.bottom()), c);
        if i > 0 {
            let b = 2 * i;
            mesh.add_triangle(b - 2, b - 1, b);
            mesh.add_triangle(b - 1, b, b + 1);
        }
    }
    painter.add(Shape::mesh(mesh));
    strip_value(ui, &mut response, rect, &mut hsva.h);
    strip_handle(ui.painter(), rect, hsva.h);
    paint_focus_ring(ui, rect, &response, 0);
    response
}

/// Alpha slider over a checkerboard.
pub fn alpha_slider(ui: &mut Ui, hsva: &mut Hsv) -> Response {
    let (rect, mut response) = slider_strip(ui, "Opacity of color");
    let painter = ui.painter();
    paint_checkerboard(painter, rect, 7.0);
    let opaque_color = opaque(*hsva).to_color32();
    let mut mesh = Mesh::default();
    mesh.colored_vertex(rect.left_top(), Color32::TRANSPARENT);
    mesh.colored_vertex(rect.right_top(), opaque_color);
    mesh.colored_vertex(rect.right_bottom(), opaque_color);
    mesh.colored_vertex(rect.left_bottom(), Color32::TRANSPARENT);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(Shape::mesh(mesh));
    strip_value(ui, &mut response, rect, &mut hsva.a);
    strip_handle(ui.painter(), rect, hsva.a);
    paint_focus_ring(ui, rect, &response, 0);
    response
}
