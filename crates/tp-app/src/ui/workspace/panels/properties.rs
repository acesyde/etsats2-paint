//! Properties panel: selection summary, opacity, corner radius and
//! fill/stroke swatches.

use egui::{RichText, Slider, Ui, WidgetInfo, WidgetType};
use tp_core::document::{Object, ShapeKind};
use tp_ui::icons;
use tp_ui::theme::label_strong_style;
use tp_ui::tokens::{color, space};
use tp_ui::widgets::{ColorSwatch, NumericField, SwatchColor};

use super::{PanelEnv, apply_field, reveal};
use crate::layout::{PanelKind, WorkspaceLayout};
use crate::workspace::{ColorTarget, Workspace};

pub fn kind_icon(kind: ShapeKind) -> &'static str {
    match kind {
        ShapeKind::Rectangle { .. } => icons::RECTANGLE,
        ShapeKind::Ellipse => icons::ELLIPSE,
        ShapeKind::Group => icons::GROUP,
    }
}

/// The value shared by all items, or `None` when they differ (or are empty).
pub fn common<T: PartialEq + Copy>(mut values: impl Iterator<Item = T>) -> Option<T> {
    let first = values.next()?;
    values.all(|v| v == first).then_some(first)
}

fn swatch_of(color: Option<tp_core::document::Rgba>) -> SwatchColor {
    match color {
        Some(c) => SwatchColor::Solid(egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)),
        None => SwatchColor::Mixed,
    }
}

/// Fill and stroke swatches of the selection (or of the current style).
pub fn selection_swatches(ws: &Workspace) -> (SwatchColor, SwatchColor) {
    let shapes = ws.selected_shapes();
    if shapes.is_empty() && ws.selection.is_empty() {
        let fill = swatch_of(Some(ws.style.fill));
        let stroke = match ws.style.stroke() {
            Some(s) => swatch_of(Some(s.color)),
            None => SwatchColor::None,
        };
        return (fill, stroke);
    }
    let fill = swatch_of(common(shapes.iter().map(|o| o.fill)));
    let stroke = match common(shapes.iter().map(|o| o.stroke.map(|s| s.color))) {
        Some(Some(c)) => swatch_of(Some(c)),
        Some(None) => SwatchColor::None,
        None => SwatchColor::Mixed,
    };
    (fill, stroke)
}

fn set_opacity(ws: &mut Workspace, percent: f64) {
    let mut objects = ws.selected_objects();
    for o in &mut objects {
        o.opacity = (percent / 100.0).clamp(0.0, 1.0) as f32;
    }
    ws.live_edit("Change Opacity", |project, _| {
        project.surface_mut().replace(&objects)
    });
}

fn set_corner_radius(ws: &mut Workspace, radius: f64) {
    let mut objects = ws.selected_objects();
    for o in &mut objects {
        if let ShapeKind::Rectangle { corner_radius } = &mut o.kind {
            let max = o.frame.size.width.min(o.frame.size.height) / 2.0;
            *corner_radius = radius.clamp(0.0, max);
        }
    }
    ws.live_edit("Change Corner Radius", |project, _| {
        project.surface_mut().replace(&objects)
    });
}

fn summary(ui: &mut Ui, objects: &[Object], ws: &Workspace) {
    let (icon, title, subtitle) = match objects {
        [] => {
            let surface = ws.project.surface();
            (
                icons::VEHICLE,
                surface.name.clone(),
                format!("{0} × {0} px · Select an object to edit it", surface.size),
            )
        }
        [single] => (
            kind_icon(single.kind),
            single.name.clone(),
            single.kind.name().to_owned(),
        ),
        many => (
            icons::LAYERS,
            format!("{} objects", many.len()),
            "Multiple selection".to_owned(),
        ),
    };
    ui.horizontal(|ui| {
        ui.label(icons::rich(icon).color(color::TEXT_SECONDARY));
        let r = ui.label(RichText::new(&title).text_style(label_strong_style()));
        r.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &title));
    });
    ui.label(RichText::new(subtitle).small().color(color::TEXT_SECONDARY));
}

/// Fixed body height: the panel's content changes with the selection, and a
/// changing height would make the panels below it (Layers) jump under the
/// pointer between two clicks.
const BODY_HEIGHT: f32 = 136.0;

pub fn show(ui: &mut Ui, env: &mut PanelEnv<'_>, layout: &mut WorkspaceLayout) {
    let top = ui.cursor().top();
    ui.vertical(|ui| body(ui, env, layout));
    let used = ui.cursor().top() - top;
    if used < BODY_HEIGHT {
        ui.add_space(BODY_HEIGHT - used);
    }
}

fn body(ui: &mut Ui, env: &mut PanelEnv<'_>, layout: &mut WorkspaceLayout) {
    let objects = env.ws.selected_objects();
    summary(ui, &objects, env.ws);
    if objects.is_empty() {
        return;
    }
    ui.add_space(space::XS);

    // Opacity: field + slider.
    let opacity = common(objects.iter().map(|o| o.opacity)).map(|o| f64::from(o) * 100.0);
    ui.horizontal(|ui| {
        let e = NumericField::new("Opacity", "Opacity", opacity)
            .suffix("%")
            .range(0.0..=100.0)
            .width(44.0)
            .show(ui);
        apply_field(env, e, set_opacity);
        let mut value = opacity.unwrap_or(100.0);
        let slider = ui.add(Slider::new(&mut value, 0.0..=100.0).show_value(false));
        slider.widget_info(|| WidgetInfo::labeled(WidgetType::Slider, true, "Opacity slider"));
        if slider.changed() {
            set_opacity(env.ws, value);
        }
        if slider.drag_stopped() || (slider.changed() && !slider.dragged()) {
            env.ws.commit_pending(env.now);
        }
    });

    // Corner radius when every selected object is a rectangle.
    let radii: Vec<f64> = objects
        .iter()
        .filter_map(|o| match o.kind {
            ShapeKind::Rectangle { corner_radius } => Some(corner_radius),
            _ => None,
        })
        .collect();
    if radii.len() == objects.len() {
        let e = NumericField::new("Radius", "Corner radius", common(radii.iter().copied()))
            .suffix("px")
            .range(0.0..=100_000.0)
            .show(ui);
        apply_field(env, e, set_corner_radius);
    }

    // Fill / stroke swatches → Colors panel.
    let (fill, stroke) = selection_swatches(env.ws);
    let target = env.ws.panels.color_target;
    ui.horizontal(|ui| {
        ui.label(RichText::new("Fill").small().color(color::TEXT_SECONDARY));
        if ColorSwatch::new(fill, "Fill color")
            .selected(target == ColorTarget::Fill)
            .show(ui)
            .clicked()
        {
            env.ws.panels.color_target = ColorTarget::Fill;
            reveal(layout, PanelKind::Colors);
        }
        ui.add_space(space::SM);
        ui.label(RichText::new("Stroke").small().color(color::TEXT_SECONDARY));
        if ColorSwatch::new(stroke, "Stroke color")
            .selected(target == ColorTarget::Stroke)
            .show(ui)
            .clicked()
        {
            env.ws.panels.color_target = ColorTarget::Stroke;
            reveal(layout, PanelKind::Colors);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::common;

    #[test]
    fn common_values() {
        assert_eq!(common([1, 1, 1].into_iter()), Some(1));
        assert_eq!(common([1, 2].into_iter()), None);
        assert_eq!(common(std::iter::empty::<i32>()), None);
    }
}
