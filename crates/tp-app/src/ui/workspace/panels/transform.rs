//! The inspector's Layout section: X / Y (center), W / H, rotation and
//! scale, then the align, distribute, flip and combine buttons. Shown only
//! with a selection.

use egui::{RichText, Ui};
use tp_core::document::{
    Frame, Handle, Object, ResizeOptions, resize, rotate, selection_frame, translate,
};
use tp_core::kurbo::{Point, Vec2};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::{color, size, space};
use tp_ui::widgets::{FieldEvent, NumericField, toggle_icon_button};

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
    ws.live_edit("tool-move", |project, _| {
        project.surface_mut().replace(&moved)
    });
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
    ws.live_edit("undo-rotate", |project, _| {
        project.surface_mut().replace(&rotated)
    });
}

/// Height of the Layout section's fields and buttons.
const ROW: f32 = 28.0;
/// Height of the align buttons.
const BUTTON: f32 = 26.0;

/// A row of two even columns around a gutter as wide as a button (the
/// proportion lock sits in it, between W and H).
fn two_columns(
    ui: &mut Ui,
    left: impl FnOnce(&mut Ui),
    middle: impl FnOnce(&mut Ui),
    right: impl FnOnce(&mut Ui),
) {
    let column = ((ui.available_width() - size::HIT_MIN) / 2.0).floor();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for (width, body) in [
            (column, Box::new(left) as Box<dyn FnOnce(&mut Ui)>),
            (size::HIT_MIN, Box::new(middle)),
            (column, Box::new(right)),
        ] {
            ui.allocate_ui_with_layout(
                egui::vec2(width, ROW),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.set_width(width);
                    ui.set_min_height(ROW);
                    body(ui);
                },
            );
        }
    });
}

/// A field of the Layout section: an inset box filling its column, its
/// short label inside as a prefix.
fn field<'a>(label: &'a str, name: &'a str, value: f64) -> NumericField<'a> {
    NumericField::new(label, name, Some(value))
        .width(28.0)
        .fill()
        .inset(ROW)
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
        return;
    };
    let (w, h) = (bounds.size.width, bounds.size.height);
    let locked = env.ws.panels.lock_proportions;
    // Two even columns of inset fields (X Y, W H, rotation and scale); the
    // lock toggle sits between W and H. Positions and sizes are in pixels,
    // the unit of the whole section.
    let none = FieldEvent::None;
    let (mut x, mut y, mut width, mut height, mut rotation, mut scale) =
        (none, none, none, none, none, none);
    let mut toggle_lock = false;
    ui.spacing_mut().item_spacing.y = space::XS + 2.0;
    two_columns(
        ui,
        |ui| {
            x = field("X", &tr("transform-x"), bounds.center.x).show(ui);
        },
        |_| {},
        |ui| {
            y = field("Y", &tr("transform-y"), bounds.center.y).show(ui);
        },
    );
    two_columns(
        ui,
        |ui| {
            width = field(&tr("field-w"), &tr("field-width"), w)
                .range(1.0..=100_000.0)
                .show(ui);
        },
        |ui| {
            toggle_lock = toggle_icon_button(
                ui,
                locked,
                icons::LINKED,
                icons::UNLINKED,
                &tr("transform-unlock-proportions"),
                &tr("transform-lock-proportions"),
            );
        },
        |ui| {
            height = field(&tr("field-h"), &tr("field-height"), h)
                .range(1.0..=100_000.0)
                .show(ui);
        },
    );
    two_columns(
        ui,
        |ui| {
            rotation = NumericField::new(
                &tr("field-r"),
                &tr("transform-rotation"),
                common_rotation(&objects),
            )
            .width(28.0)
            .fill()
            .inset(ROW)
            .suffix("°")
            .decimals(1)
            .range(-360.0..=360.0)
            .show(ui);
        },
        |_| {},
        |ui| {
            scale = field(&tr("field-s"), &tr("transform-scale"), 100.0)
                .suffix("%")
                .range(1.0..=10_000.0)
                .show(ui);
        },
    );
    if toggle_lock {
        env.ws.panels.lock_proportions = !locked;
    }
    apply_field(env, x, |ws, v| move_center(ws, Some(v), None));
    apply_field(env, y, |ws, v| move_center(ws, None, Some(v)));
    apply_field(env, width, |ws, v| {
        let sx = v / w;
        scale_selection(ws, "undo-resize", sx, if locked { sx } else { 1.0 });
    });
    apply_field(env, height, |ws, v| {
        let sy = v / h;
        scale_selection(ws, "undo-resize", if locked { sy } else { 1.0 }, sy);
    });
    apply_field(env, rotation, set_rotation);
    // Scale is relative to the current size: applying live values every
    // frame would compound, so only the final value is applied.
    let scale = match scale {
        FieldEvent::Live(_) => FieldEvent::None,
        other => other,
    };
    apply_field(env, scale, |ws, v| {
        let s = v / 100.0;
        scale_selection(ws, "transform-scale", s, s);
    });
    align_rows(ui, cmds, env);
}

/// The Align to selector and the align buttons, then the distribute and
/// flip buttons, then the combine buttons.
fn align_rows(ui: &mut Ui, cmds: &mut crate::ui::CommandUi<'_>, env: &mut PanelEnv<'_>) {
    use tp_core::document::{DistributeAxis, DistributeMode, Edge};

    use crate::arrange::AlignTo;
    use crate::commands::CommandId;

    ui.add_space(space::SM);
    ui.horizontal(|ui| {
        let label = tr("transform-align-to");
        ui.label(RichText::new(&label).small().color(color::TEXT_SECONDARY));
        let current = env.ws.panels.align_to;
        let combo = egui::ComboBox::from_id_salt("align_to")
            .icon(tp_ui::widgets::dropdown_icon)
            .width(ui.available_width().min(140.0))
            .selected_text(tr(current.label()))
            .show_ui(ui, |ui| {
                for to in AlignTo::ALL {
                    ui.selectable_value(&mut env.ws.panels.align_to, to, tr(to.label()));
                }
            });
        combo
            .response
            .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, true, &label));
    });
    // Six even columns: the align buttons fill a row; the distribute
    // buttons take the first four of the next, the flip buttons (apart from
    // them) share one box over the last two.
    let gap = space::XS;
    let column = ((ui.available_width() - 5.0 * gap) / 6.0).floor();
    let button = egui::vec2(column, BUTTON);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for edge in Edge::ALL {
            cmds.framed_icon_button(ui, CommandId::Align(edge), button);
        }
    });
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for (axis, mode) in [
            (DistributeAxis::Horizontal, DistributeMode::Centers),
            (DistributeAxis::Vertical, DistributeMode::Centers),
            (DistributeAxis::Horizontal, DistributeMode::Spacing),
            (DistributeAxis::Vertical, DistributeMode::Spacing),
        ] {
            cmds.framed_icon_button(ui, CommandId::Distribute(axis, mode), button);
        }
        let flips = egui::vec2(2.0 * column + gap, BUTTON);
        let (rect, _) = ui.allocate_exact_size(flips, egui::Sense::hover());
        ui.painter()
            .rect_filled(rect, tp_ui::tokens::radius::MD, color::CONTROL);
        let mut inner = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        inner.spacing_mut().item_spacing.x = 0.0;
        for axis in tp_core::document::FlipAxis::ALL {
            cmds.framed_icon_button(
                &mut inner,
                CommandId::Flip(axis),
                egui::vec2(flips.x / 2.0, BUTTON),
            );
        }
    });
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for op in tp_core::document::BooleanOp::ALL {
            cmds.framed_icon_button(ui, CommandId::Combine(op), button);
        }
    });
}
