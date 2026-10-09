//! The stroke popover of the inspector's Stroke row: enable/disable the
//! stroke, its width and alignment, and the dashes, caps and joins of
//! outlines; and the row's summary ("6 px · Outside").

use egui::{Checkbox, Ui, WidgetInfo, WidgetType};
use tp_core::document::{LineStyle, Object, StrokeAlign, StrokeStyle};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::widgets::{NumericField, SegmentedControl};

use super::line_style::{self, Change, LineEdit};
use super::properties::common;
use super::{PanelEnv, apply_field};
use crate::workspace::Workspace;

/// Applies `f` to every selected stroke, or to the current style with
/// nothing selected.
fn edit_strokes(ws: &mut Workspace, label: &'static str, f: impl Fn(&mut StrokeStyle)) {
    if ws.selection.is_empty() {
        f(&mut ws.style.stroke);
        return;
    }
    ws.map_selected_shapes(label, |o| {
        if let Some(stroke) = &mut o.stroke {
            f(stroke);
        }
    });
}

fn set_width(ws: &mut Workspace, width: f64) {
    edit_strokes(ws, "undo-change-stroke-width", |s| s.width = width);
}

/// Whether the object's stroke follows an outline (shapes, texts, closed
/// paths) rather than outlining lines only.
fn outlines_a_shape(o: &Object) -> bool {
    o.path_data()
        .is_none_or(|p| p.subpaths.iter().any(|s| s.closed))
}

/// The Stroke row's summary: width and alignment of the selection's
/// strokes (or of the current style), "None" without a stroke, "Mixed" when
/// they differ.
pub fn summary(ws: &Workspace) -> String {
    let strokes: Vec<Option<StrokeStyle>> = if ws.selection.is_empty() {
        vec![ws.style.stroke()]
    } else {
        ws.selected_shapes().iter().map(|o| o.stroke).collect()
    };
    match common(strokes.iter().map(Option::is_some)) {
        Some(true) => {}
        Some(false) => return tr("stroke-none"),
        None => return tr("mixed"),
    }
    let strokes: Vec<StrokeStyle> = strokes.into_iter().flatten().collect();
    let width = common(strokes.iter().map(|s| s.width));
    let align = common(strokes.iter().map(|s| s.align));
    let width = width.map_or_else(
        || tr("mixed"),
        |w| tr!("stroke-width-px", width = tp_i18n::format_number(w, 1)),
    );
    let align = align.map_or_else(
        || tr("mixed"),
        |a| {
            tr(match a {
                StrokeAlign::Center => "stroke-center",
                StrokeAlign::Inside => "stroke-inside",
                StrokeAlign::Outside => "stroke-outside",
            })
        },
    );
    format!("{width} · {align}")
}

/// The stroke popover's content.
pub fn popover(ui: &mut Ui, env: &mut PanelEnv<'_>) {
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
    let response = ui.add(Checkbox::new(&mut checked, tr("panel-stroke")).indeterminate(mixed));
    response.widget_info(|| {
        WidgetInfo::selected(WidgetType::Checkbox, true, enabled, tr("stroke-enabled"))
    });
    if response.clicked() {
        let enable = mixed || !enabled;
        if empty {
            env.ws.style.stroke_enabled = enable;
        } else {
            let style = env.ws.style.stroke;
            let label = if enable {
                "undo-add-stroke"
            } else {
                "undo-remove-stroke"
            };
            env.ws.map_selected_shapes(label, |o| {
                o.stroke = if enable {
                    o.stroke.or(Some(style))
                } else {
                    None
                };
            });
            env.ws.commit_pending(env.now);
        }
    }

    let strokes: Vec<(StrokeStyle, bool)> = if empty {
        vec![(env.ws.style.stroke, true)]
    } else {
        shapes
            .iter()
            .filter_map(|o| o.stroke.map(|s| (s, outlines_a_shape(o))))
            .collect()
    };
    let width = common(strokes.iter().map(|(s, _)| s.width));
    let e = NumericField::new(&tr("field-width"), &tr("stroke-width"), width)
        .suffix("px")
        .decimals(1)
        .speed(0.5)
        .range(0.5..=500.0)
        .show(ui);
    apply_field(env, e, set_width);
    if strokes.is_empty() {
        return;
    }

    let align = common(strokes.iter().map(|(s, _)| s.align));
    let picked = ui
        .push_id("stroke_align", |ui| {
            SegmentedControl::new()
                .named_segment(
                    Some(StrokeAlign::Center),
                    icons::STROKE_CENTER,
                    &tr("stroke-center"),
                    &tr("stroke-center-name"),
                )
                .named_segment(
                    Some(StrokeAlign::Inside),
                    icons::STROKE_INSIDE,
                    &tr("stroke-inside"),
                    &tr("stroke-inside-name"),
                )
                .named_segment(
                    Some(StrokeAlign::Outside),
                    icons::STROKE_OUTSIDE,
                    &tr("stroke-outside"),
                    &tr("stroke-outside-name"),
                )
                .show(ui, align)
        })
        .inner;
    if let Some(Some(align)) = picked {
        edit_strokes(env.ws, "undo-change-stroke-alignment", |s| s.align = align);
        env.ws.commit_pending(env.now);
    }

    // Lines use their own dashes, caps and joins (inspector › Appearance).
    let outlines: Vec<LineStyle> = strokes
        .iter()
        .filter(|(_, outline)| *outline)
        .map(|(s, _)| s.line)
        .collect();
    if outlines.is_empty() {
        return;
    }
    match line_style::controls(ui, &outlines, env.ws.last_dash) {
        Some(Change::Apply { edit, commit }) => {
            remember_dash(env.ws, edit, &outlines);
            edit_strokes(env.ws, edit.label(false), |s| edit.apply(&mut s.line));
            if commit {
                env.ws.commit_pending(env.now);
            }
        }
        Some(Change::Revert) => env.ws.cancel_pending(),
        None => {}
    }
}

/// Keeps the dash lengths for the next time Dashed is turned on.
pub fn remember_dash(ws: &mut Workspace, edit: LineEdit, before: &[LineStyle]) {
    if let LineEdit::Dashed(_, lengths) = edit {
        ws.last_dash = lengths;
        return;
    }
    let mut style = before
        .iter()
        .find(|s| s.dash.is_some())
        .copied()
        .unwrap_or_default();
    edit.apply(&mut style);
    if let Some(dash) = style.dash {
        ws.last_dash = dash;
    }
}
