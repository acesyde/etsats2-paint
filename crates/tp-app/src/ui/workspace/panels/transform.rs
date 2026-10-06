//! Transform panel: X / Y (center), W / H, rotation and scale.

use egui::{Grid, Ui};
use tp_core::document::{
    Frame, Handle, Object, ResizeOptions, resize, rotate, selection_frame, translate,
};
use tp_core::kurbo::{Point, Vec2};
use tp_ui::icons;
use tp_ui::tokens::space;
use tp_ui::widgets::{EmptyState, NumericField, toggle_icon_button};

use super::{PanelEnv, apply_field};
use crate::workspace::Workspace;

/// Scales the selection by (sx, sy) around the bounds center, in the bounds'
/// own axes.
fn scale_selection(ws: &mut Workspace, label: &'static str, sx: f64, sy: f64) {
    let objects = ws.selected_objects();
    let Some(bounds) = selection_frame(&objects) else {
        return;
    };
    let half = Vec2::new(bounds.size.width / 2.0, bounds.size.height / 2.0);
    let pointer = bounds.affine() * Point::new(sx * half.x, sy * half.y);
    let options = ResizeOptions {
        proportional: false,
        from_center: true,
    };
    let scaled = resize(&objects, bounds, Handle { x: 1, y: 1 }, pointer, options);
    ws.live_edit(label, |project, _| project.surface_mut().replace(&scaled));
}

fn move_center(ws: &mut Workspace, x: Option<f64>, y: Option<f64>) {
    let objects = ws.selected_objects();
    let Some(bounds) = selection_frame(&objects) else {
        return;
    };
    let delta = Vec2::new(
        x.map_or(0.0, |x| x - bounds.center.x),
        y.map_or(0.0, |y| y - bounds.center.y),
    );
    let moved = translate(&objects, delta, false);
    ws.live_edit("Move", |project, _| project.surface_mut().replace(&moved));
}

fn set_rotation(ws: &mut Workspace, degrees: f64) {
    let objects = ws.selected_objects();
    let rotated: Vec<Object> = match objects.as_slice() {
        [single] => rotate(
            &objects,
            single.frame.center,
            degrees - single.frame.rotation_deg,
            false,
        ),
        many => many
            .iter()
            .flat_map(|o| {
                rotate(
                    std::slice::from_ref(o),
                    o.frame.center,
                    degrees - o.frame.rotation_deg,
                    false,
                )
            })
            .collect(),
    };
    ws.live_edit("Rotate", |project, _| {
        project.surface_mut().replace(&rotated)
    });
}

/// Rotation shown by the panel: the shared value, or `None` ("Mixed").
fn common_rotation(objects: &[Object]) -> Option<f64> {
    let first = objects.first()?.frame.rotation_deg;
    objects
        .iter()
        .all(|o| (o.frame.rotation_deg - first).abs() < 1e-6)
        .then_some(first)
}

pub fn show(ui: &mut Ui, cmds: &mut crate::ui::CommandUi<'_>, env: &mut PanelEnv<'_>) {
    let objects = env.ws.selected_objects();
    let Some(bounds): Option<Frame> = selection_frame(&objects) else {
        EmptyState::new(
            icons::TRANSFORM,
            "Nothing to transform",
            "Select an object to edit position, size and rotation.",
        )
        .show(ui);
        return;
    };
    let (w, h) = (bounds.size.width, bounds.size.height);
    let locked = env.ws.panels.lock_proportions;

    Grid::new("transform_grid")
        .num_columns(3)
        .spacing([space::SM, space::XS + 2.0])
        .show(ui, |ui| {
            let e = NumericField::new("X", "X position", Some(bounds.center.x))
                .suffix("px")
                .show(ui);
            apply_field(env, e, |ws, v| move_center(ws, Some(v), None));
            ui.label("");
            let e = NumericField::new("Y", "Y position", Some(bounds.center.y))
                .suffix("px")
                .show(ui);
            apply_field(env, e, |ws, v| move_center(ws, None, Some(v)));
            ui.end_row();

            let e = NumericField::new("W", "Width", Some(w))
                .suffix("px")
                .range(1.0..=100_000.0)
                .show(ui);
            apply_field(env, e, |ws, v| {
                let sx = v / w;
                scale_selection(ws, "Resize", sx, if locked { sx } else { 1.0 });
            });
            if toggle_icon_button(
                ui,
                locked,
                icons::LINKED,
                icons::UNLINKED,
                "Unlock proportions",
                "Lock proportions",
            ) {
                env.ws.panels.lock_proportions = !locked;
            }
            let e = NumericField::new("H", "Height", Some(h))
                .suffix("px")
                .range(1.0..=100_000.0)
                .show(ui);
            apply_field(env, e, |ws, v| {
                let sy = v / h;
                scale_selection(ws, "Resize", if locked { sy } else { 1.0 }, sy);
            });
            ui.end_row();

            let e = NumericField::new("R", "Rotation", common_rotation(&objects))
                .suffix("°")
                .decimals(1)
                .range(-360.0..=360.0)
                .show(ui);
            apply_field(env, e, set_rotation);
            ui.label("");
            let e = NumericField::new("S", "Scale", Some(100.0))
                .suffix("%")
                .range(1.0..=10_000.0)
                .show(ui);
            // Scale is relative to the current size: applying live values
            // every frame would compound, so only the final value is applied.
            let e = match e {
                tp_ui::widgets::FieldEvent::Live(_) => tp_ui::widgets::FieldEvent::None,
                other => other,
            };
            apply_field(env, e, |ws, v| {
                let s = v / 100.0;
                scale_selection(ws, "Scale", s, s);
            });
            ui.end_row();
        });
    align_rows(ui, cmds, env);
}

/// Align buttons with the Align to selector, then distribute buttons.
fn align_rows(ui: &mut Ui, cmds: &mut crate::ui::CommandUi<'_>, env: &mut PanelEnv<'_>) {
    use tp_core::document::{DistributeAxis, DistributeMode, Edge};

    use crate::arrange::AlignTo;
    use crate::commands::CommandId;

    ui.add_space(space::SM);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for edge in Edge::ALL {
            cmds.icon_button(ui, CommandId::Align(edge), false);
        }
        ui.add_space(space::XS);
        let current = env.ws.panels.align_to;
        let combo = egui::ComboBox::from_id_salt("align_to")
            .width(88.0)
            .selected_text(current.label())
            .show_ui(ui, |ui| {
                for to in AlignTo::ALL {
                    ui.selectable_value(&mut env.ws.panels.align_to, to, to.label());
                }
            });
        combo.response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, true, "Align to")
        });
    });
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for (axis, mode) in [
            (DistributeAxis::Horizontal, DistributeMode::Centers),
            (DistributeAxis::Vertical, DistributeMode::Centers),
            (DistributeAxis::Horizontal, DistributeMode::Spacing),
            (DistributeAxis::Vertical, DistributeMode::Spacing),
        ] {
            cmds.icon_button(ui, CommandId::Distribute(axis, mode), false);
        }
    });
}
