//! Properties panel: selection summary, opacity, corner radius, polygon
//! settings, line width, dashes, caps and joins, and fill/stroke swatches.

use egui::{RichText, Slider, Ui, WidgetInfo, WidgetType};
use tp_core::AssetKind;
use tp_core::document::{GradientKind, LineStyle, Object, Paint, ShapeKind};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::theme::label_strong_style;
use tp_ui::tokens::{color, space};
use tp_ui::widgets::{ColorSwatch, NumericField, SwatchColor, secondary_button};

use super::line_style::{Change, LineEdit};
use super::{PanelEnv, apply_field, reveal};
use crate::layout::{PanelKind, WorkspaceLayout};
use crate::workspace::{ColorTarget, Workspace};

pub fn kind_icon(kind: ShapeKind) -> &'static str {
    match kind {
        ShapeKind::Rectangle { .. } => icons::RECTANGLE,
        ShapeKind::Ellipse => icons::ELLIPSE,
        ShapeKind::Polygon { .. } => icons::POLYGON,
        ShapeKind::Path => icons::PEN,
        ShapeKind::Group => icons::GROUP,
        ShapeKind::Text => icons::TEXT,
        ShapeKind::Image { .. } => icons::IMAGE,
    }
}

/// The value shared by all items, or `None` when they differ (or are empty).
pub fn common<T: PartialEq + Copy>(mut values: impl Iterator<Item = T>) -> Option<T> {
    let first = values.next()?;
    values.all(|v| v == first).then_some(first)
}

/// The preview of a gradient for swatches and the gradient bar.
pub fn gradient_preview(g: &tp_core::document::Gradient) -> tp_ui::widgets::GradientPreview {
    let stops: Vec<(f32, [u8; 4])> = g
        .stops()
        .iter()
        .map(|s| (s.offset, [s.color.r, s.color.g, s.color.b, s.color.a]))
        .collect();
    tp_ui::widgets::GradientPreview::new(g.kind == GradientKind::Radial, &stops)
}

fn swatch_of(paint: Option<Paint>) -> SwatchColor {
    match paint {
        Some(Paint::Solid(c)) => {
            SwatchColor::Solid(egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a))
        }
        Some(Paint::Gradient(g)) => SwatchColor::Gradient(gradient_preview(&g)),
        None => SwatchColor::Mixed,
    }
}

/// Fill and stroke swatches of the selection (or of the current style).
pub fn selection_swatches(ws: &Workspace) -> (SwatchColor, SwatchColor) {
    let shapes = ws.selected_shapes();
    if shapes.is_empty() && ws.selection.is_empty() {
        let fill = swatch_of(Some(ws.style.fill));
        let stroke = match ws.style.stroke() {
            Some(s) => swatch_of(Some(s.paint)),
            None => SwatchColor::None,
        };
        return (fill, stroke);
    }
    let fill = swatch_of(common(shapes.iter().map(|o| o.fill)));
    let stroke = match common(shapes.iter().map(|o| o.stroke.map(|s| s.paint))) {
        Some(Some(p)) => swatch_of(Some(p)),
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
    ws.live_edit("undo-change-opacity", |project, _| {
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
    ws.live_edit("undo-change-corner-radius", |project, _| {
        project.surface_mut().replace(&objects)
    });
}

/// Applies `f` to every selected polygon's `(sides, star)` as a live edit;
/// the result also becomes the setting for new polygons.
fn edit_polygons(ws: &mut Workspace, label: &'static str, f: impl Fn(&mut u8, &mut Option<f64>)) {
    let mut objects = ws.selected_objects();
    for o in &mut objects {
        if let ShapeKind::Polygon { sides, star } = &mut o.kind {
            f(sides, star);
            ws.polygon_style.sides = *sides;
            ws.polygon_style.star = star.is_some();
            if let Some(inner) = star {
                ws.polygon_style.inner = *inner;
            }
        }
    }
    ws.live_edit(label, |project, _| project.surface_mut().replace(&objects));
}

fn set_sides(ws: &mut Workspace, value: f64) {
    let sides = value.round().clamp(3.0, 12.0) as u8;
    edit_polygons(ws, "undo-change-sides", |s, _| *s = sides);
}

fn set_inner_radius(ws: &mut Workspace, percent: f64) {
    let inner = (percent / 100.0).clamp(0.1, 0.9);
    edit_polygons(ws, "undo-change-inner-radius", |_, star| {
        if star.is_some() {
            *star = Some(inner);
        }
    });
}

fn set_line_width(ws: &mut Workspace, width: f64) {
    let width = width.clamp(0.5, 1000.0);
    let mut objects = ws.selected_objects();
    for o in &mut objects {
        if o.has_open_path() {
            o.edit_path(|p| p.line_width = width);
        }
    }
    ws.line_width = width;
    ws.live_edit("undo-change-line-width", |project, _| {
        project.surface_mut().replace(&objects)
    });
}

/// Applies a dash, cap or join change to the selected lines; the result
/// also becomes the line style for new lines.
fn edit_line_style(ws: &mut Workspace, edit: LineEdit) {
    let mut objects = ws.selected_objects();
    for o in &mut objects {
        if o.has_open_path() {
            o.edit_path(|p| edit.apply(&mut p.line_style));
        }
    }
    edit.apply(&mut ws.line_style);
    ws.live_edit(edit.label(true), |project, _| {
        project.surface_mut().replace(&objects)
    });
}

/// Sides, Star and Inner radius of the selected polygons.
fn polygon_settings(ui: &mut Ui, env: &mut PanelEnv<'_>, polygons: &[(u8, Option<f64>)]) {
    let sides = common(polygons.iter().map(|(s, _)| *s)).map(f64::from);
    let stars = common(polygons.iter().map(|(_, star)| star.is_some()));
    ui.horizontal(|ui| {
        let e = NumericField::new(&tr("props-sides"), &tr("props-sides"), sides)
            .range(3.0..=12.0)
            .width(44.0)
            .show(ui);
        apply_field(env, e, set_sides);
        ui.add_space(space::SM);
        let mut checked = stars == Some(true);
        let response = ui.add(
            egui::Checkbox::new(&mut checked, tr("props-star")).indeterminate(stars.is_none()),
        );
        response.widget_info(|| {
            WidgetInfo::selected(WidgetType::Checkbox, true, checked, tr("props-star"))
        });
        if response.changed() {
            let inner = env.ws.polygon_style.inner;
            edit_polygons(env.ws, "undo-change-star", |_, star| {
                *star = checked.then_some(inner);
            });
            env.ws.commit_pending(env.now);
        }
    });
    if stars == Some(true) {
        let inner = common(polygons.iter().map(|(_, star)| star.unwrap_or(0.5)))
            .map(|r| (r * 100.0).round());
        let e = NumericField::new(&tr("props-inner"), &tr("props-inner-radius"), inner)
            .suffix("%")
            .range(10.0..=90.0)
            .width(44.0)
            .show(ui);
        apply_field(env, e, set_inner_radius);
    }
}

fn summary(ui: &mut Ui, objects: &[Object], ws: &Workspace) {
    let (icon, title, subtitle) = match objects {
        [] => {
            let surface = ws.project.surface();
            (
                icons::VEHICLE,
                surface.name.clone(),
                tr!("props-surface-summary", size = surface.size),
            )
        }
        [single] => (
            kind_icon(single.kind),
            single.name.clone(),
            crate::workspace::object_name(single.kind),
        ),
        many => (
            icons::LAYERS,
            tr!("props-objects", count = many.len()),
            tr("props-multiple"),
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
    // The body only grows (texts show more settings), so rows of the panels
    // below do not move between two clicks.
    let id = ui.id().with("properties_height");
    let height = ui
        .data(|d| d.get_temp::<f32>(id))
        .unwrap_or(BODY_HEIGHT)
        .max(used);
    ui.data_mut(|d| d.insert_temp(id, height));
    if used < height {
        ui.add_space(height - used);
    }
}

fn image_info(ui: &mut Ui, env: &mut PanelEnv<'_>, object: &Object) {
    let ShapeKind::Image { asset } = object.kind else {
        return;
    };
    let Some(asset) = env.ws.project.assets.get(&asset).cloned() else {
        return;
    };
    let source = match asset.kind {
        AssetKind::Svg => tr("props-source-svg"),
        AssetKind::Raster => tr!(
            "props-source-image",
            width = asset.size.width.round(),
            height = asset.size.height.round()
        ),
    };
    ui.label(
        RichText::new(tr!(
            "props-source",
            name = asset.name.as_str(),
            source = source
        ))
        .small()
        .color(color::TEXT_SECONDARY),
    );
    if ui.add(secondary_button(&tr("undo-reset-size"))).clicked() {
        env.ws.reset_image_size(env.now);
    }
}

fn body(ui: &mut Ui, env: &mut PanelEnv<'_>, layout: &mut WorkspaceLayout) {
    let objects = env.ws.selected_objects();
    summary(ui, &objects, env.ws);
    if objects.is_empty() {
        if env.ws.tool == crate::tool::Tool::Text {
            ui.add_space(space::XS);
            super::character::show(ui, env);
        }
        return;
    }
    let shapes = env.ws.selected_shapes();
    let only = |f: fn(&ShapeKind) -> bool| !shapes.is_empty() && shapes.iter().all(|o| f(&o.kind));
    let only_images = only(|k| matches!(k, ShapeKind::Image { .. }));
    let only_texts = only(|k| *k == ShapeKind::Text);
    ui.add_space(space::XS);

    // Opacity: field + slider.
    let opacity = common(objects.iter().map(|o| o.opacity)).map(|o| f64::from(o) * 100.0);
    ui.horizontal(|ui| {
        let e = NumericField::new(&tr("props-opacity"), &tr("props-opacity"), opacity)
            .suffix("%")
            .range(0.0..=100.0)
            .width(44.0)
            .show(ui);
        apply_field(env, e, set_opacity);
        let mut value = opacity.unwrap_or(100.0);
        let slider = ui.add(Slider::new(&mut value, 0.0..=100.0).show_value(false));
        slider.widget_info(|| {
            WidgetInfo::labeled(WidgetType::Slider, true, tr("props-opacity-slider"))
        });
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
        let e = NumericField::new(
            &tr("props-radius"),
            &tr("props-corner-radius"),
            common(radii.iter().copied()),
        )
        .suffix("px")
        .range(0.0..=100_000.0)
        .show(ui);
        apply_field(env, e, set_corner_radius);
    }

    // Sides and star settings when every selected object is a polygon.
    let polygons: Vec<(u8, Option<f64>)> = objects
        .iter()
        .filter_map(|o| match o.kind {
            ShapeKind::Polygon { sides, star } => Some((sides, star)),
            _ => None,
        })
        .collect();
    if polygons.len() == objects.len() {
        polygon_settings(ui, env, &polygons);
    }

    // Line width when every selected object is a path with open subpaths.
    let widths: Vec<f64> = objects
        .iter()
        .filter(|o| o.has_open_path())
        .filter_map(|o| o.path_data().map(|p| p.line_width))
        .collect();
    if widths.len() == objects.len() {
        let e = NumericField::new(
            &tr("field-width"),
            &tr("props-line-width"),
            common(widths.iter().copied()),
        )
        .suffix("px")
        .range(0.5..=1000.0)
        .show(ui);
        apply_field(env, e, set_line_width);
        let styles: Vec<LineStyle> = objects
            .iter()
            .filter_map(|o| o.path_data().map(|p| p.line_style))
            .collect();
        match super::line_style::controls(ui, &styles, env.ws.last_dash) {
            Some(Change::Apply { edit, commit }) => {
                super::stroke::remember_dash(env.ws, edit, &styles);
                edit_line_style(env.ws, edit);
                if commit {
                    env.ws.commit_pending(env.now);
                }
            }
            Some(Change::Revert) => env.ws.cancel_pending(),
            None => {}
        }
    }

    if let [single] = objects.as_slice()
        && only_images
    {
        image_info(ui, env, single);
    }
    if only_texts {
        super::character::show(ui, env);
    }
    if only_images {
        // Fill and stroke do not apply to images.
        return;
    }

    // Fill / stroke swatches → Colors panel.
    let (fill, stroke) = selection_swatches(env.ws);
    let target = env.ws.panels.color_target;
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(tr("props-fill"))
                .small()
                .color(color::TEXT_SECONDARY),
        );
        if ColorSwatch::new(fill, &tr("props-fill-color"))
            .selected(target == ColorTarget::Fill)
            .show(ui)
            .clicked()
        {
            env.ws.panels.color_target = ColorTarget::Fill;
            reveal(layout, PanelKind::Colors);
        }
        ui.add_space(space::SM);
        ui.label(
            RichText::new(tr("panel-stroke"))
                .small()
                .color(color::TEXT_SECONDARY),
        );
        if ColorSwatch::new(stroke, &tr("props-stroke-color"))
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
