//! Stroke panel: enable/disable the stroke and set its width.

use egui::{Checkbox, Ui, WidgetInfo, WidgetType};
use tp_core::document::StrokeStyle;
use tp_ui::widgets::NumericField;

use super::properties::common;
use super::{PanelEnv, apply_field};
use crate::workspace::Workspace;

fn set_width(ws: &mut Workspace, width: f64) {
    if ws.selection.is_empty() {
        ws.style.stroke.width = width;
        return;
    }
    ws.map_selected_shapes("Change Stroke Width", |o| {
        if let Some(stroke) = &mut o.stroke {
            stroke.width = width;
        }
    });
}

pub fn show(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    let shapes = env.ws.selected_shapes();
    let empty = env.ws.selection.is_empty();
    // Enabled state: all / none / mixed.
    let (enabled, mixed) = if empty {
        (env.ws.style.stroke_enabled, false)
    } else {
        match common(shapes.iter().map(|o| o.stroke.is_some())) {
            Some(all) => (all, false),
            None => (false, true),
        }
    };
    let mut checked = enabled;
    let response = ui.add(Checkbox::new(&mut checked, "Stroke").indeterminate(mixed));
    response.widget_info(|| {
        WidgetInfo::selected(WidgetType::Checkbox, true, enabled, "Stroke enabled")
    });
    if response.clicked() {
        let enable = mixed || !enabled;
        if empty {
            env.ws.style.stroke_enabled = enable;
        } else {
            let style = env.ws.style.stroke;
            let label = if enable {
                "Add Stroke"
            } else {
                "Remove Stroke"
            };
            env.ws.map_selected_shapes(label, |o| {
                o.stroke = if enable {
                    o.stroke.or(Some(StrokeStyle {
                        color: style.color,
                        width: style.width,
                    }))
                } else {
                    None
                };
            });
            env.ws.commit_pending(env.now);
        }
    }

    let width = if empty {
        Some(env.ws.style.stroke.width)
    } else {
        common(shapes.iter().filter_map(|o| o.stroke.map(|s| s.width)))
    };
    let e = NumericField::new("Width", "Stroke width", width)
        .suffix("px")
        .decimals(1)
        .speed(0.5)
        .range(0.5..=500.0)
        .show(ui);
    apply_field(env, e, set_width);
}
