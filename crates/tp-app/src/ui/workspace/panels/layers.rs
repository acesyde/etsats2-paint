//! Layers panel: the object tree with selection, rename, visibility, lock,
//! drag and drop, and grouping commands.

use std::collections::HashSet;
use std::sync::Arc;

use egui::{
    Align2, CornerRadius, Margin, Pos2, Rect, Sense, Stroke, StrokeKind, TextEdit, Ui, Vec2,
    WidgetInfo, WidgetType,
};
use tp_core::document::tree::{self, Placement};
use tp_core::document::{Object, ObjectId};
use tp_ui::icons;
use tp_ui::tokens::{color, size, space, stroke};
use tp_ui::widgets::{EmptyState, MenuRow, toggle_icon_button};

use super::PanelEnv;
use super::properties::kind_icon;
use crate::commands::CommandId;
use crate::ui::CommandUi;

const ROW_HEIGHT: f32 = 26.0;
const INDENT: f32 = 14.0;

/// One visible row of the tree.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub id: ObjectId,
    pub depth: usize,
    pub is_group: bool,
    pub expanded: bool,
}

/// Visible rows, topmost first, descending into expanded groups.
pub fn visible_rows(list: &[Arc<Object>], expanded: &HashSet<ObjectId>) -> Vec<Row> {
    fn walk(list: &[Arc<Object>], expanded: &HashSet<ObjectId>, depth: usize, out: &mut Vec<Row>) {
        for o in list.iter().rev() {
            let open = o.is_group() && expanded.contains(&o.id);
            out.push(Row {
                id: o.id,
                depth,
                is_group: o.is_group(),
                expanded: open,
            });
            if open {
                walk(&o.children, expanded, depth + 1, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(list, expanded, 0, &mut out);
    out
}

/// Where a drop at `fraction` (0 = top, 1 = bottom of the row) lands. The
/// middle of a group row drops into the group.
pub fn drop_placement(row: &Row, fraction: f32) -> Placement {
    if row.is_group && (0.3..=0.7).contains(&fraction) {
        Placement::IntoTop(row.id)
    } else if fraction < 0.5 {
        Placement::Above(row.id)
    } else {
        Placement::Below(row.id)
    }
}

/// Ids of the rows between `a` and `b` (inclusive), in row order.
pub fn row_range(rows: &[Row], a: ObjectId, b: ObjectId) -> Vec<ObjectId> {
    let ia = rows.iter().position(|r| r.id == a);
    let ib = rows.iter().position(|r| r.id == b);
    match (ia, ib) {
        (Some(ia), Some(ib)) => {
            let (lo, hi) = if ia <= ib { (ia, ib) } else { (ib, ia) };
            rows[lo..=hi].iter().map(|r| r.id).collect()
        }
        _ => vec![b],
    }
}

/// Expands every ancestor of the selection so selected rows are visible.
fn reveal_selection(env: &mut PanelEnv<'_>) {
    let objects = &env.ws.project.surface().objects;
    for id in env.ws.selection.clone() {
        let mut current = id;
        while let Some(Some(parent)) = tree::parent_of(objects, current) {
            env.ws.panels.expanded.insert(parent);
            current = parent;
        }
    }
}

pub fn show(ui: &mut Ui, cmds: &mut CommandUi<'_>, env: &mut PanelEnv<'_>) {
    if env.ws.project.surface().objects.is_empty() {
        EmptyState::new(
            icons::LAYERS,
            "No layers yet",
            "Shapes you draw and layers you add appear here.",
        )
        .show(ui);
        footer(ui, cmds);
        return;
    }
    reveal_selection(env);
    let rows = visible_rows(&env.ws.project.surface().objects, &env.ws.panels.expanded);
    let mut row_rects: Vec<(Row, Rect)> = Vec::with_capacity(rows.len());
    ui.spacing_mut().item_spacing.y = 0.0;
    for row in &rows {
        let rect = row_ui(ui, cmds, env, row, &rows);
        row_rects.push((row.clone(), rect));
    }
    drag_and_drop(ui, env, &row_rects);
    ui.add_space(space::XS);
    footer(ui, cmds);
}

fn footer(ui: &mut Ui, cmds: &mut CommandUi<'_>) {
    ui.horizontal(|ui| {
        cmds.icon_button(ui, CommandId::NewLayer, false);
        cmds.icon_button(ui, CommandId::Group, false);
        cmds.icon_button(ui, CommandId::Ungroup, false);
        cmds.icon_button(ui, CommandId::Delete, false);
    });
}

/// Draws one row; returns its rectangle.
fn row_ui(
    ui: &mut Ui,
    cmds: &mut CommandUi<'_>,
    env: &mut PanelEnv<'_>,
    row: &Row,
    rows: &[Row],
) -> Rect {
    let object = env
        .ws
        .project
        .surface()
        .get(row.id)
        .cloned()
        .expect("row object exists");
    let (hidden, locked) =
        tree::effective_flags(&env.ws.project.surface().objects, row.id).unwrap_or((false, false));
    let width = ui.available_width();
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(width, ROW_HEIGHT), Sense::click_and_drag());
    let selected = env.ws.selection.contains(&row.id);
    response.widget_info(|| {
        WidgetInfo::selected(WidgetType::SelectableLabel, true, selected, &object.name)
    });

    // Background and selection indicator.
    let painter = ui.painter();
    if selected {
        painter.rect_filled(rect, 0, color::ACCENT_SUBTLE);
        painter.rect_filled(
            Rect::from_min_size(rect.min, Vec2::new(stroke::INDICATOR, rect.height())),
            0,
            color::ACCENT,
        );
    } else if response.hovered() {
        painter.rect_filled(rect, 0, color::SURFACE_3);
    }

    let mut x = rect.left() + space::SM + row.depth as f32 * INDENT;
    // Disclosure caret for groups.
    if row.is_group {
        let caret_rect =
            Rect::from_center_size(Pos2::new(x + 6.0, rect.center().y), Vec2::splat(18.0));
        let caret = ui.interact(caret_rect, ui.id().with(("caret", row.id)), Sense::click());
        let name = if row.expanded {
            format!("Collapse {}", object.name)
        } else {
            format!("Expand {}", object.name)
        };
        caret.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, &name));
        ui.painter().text(
            caret_rect.center(),
            Align2::CENTER_CENTER,
            if row.expanded {
                icons::EXPANDED
            } else {
                icons::COLLAPSED
            },
            icons::font(size::ICON - 4.0),
            color::TEXT_SECONDARY,
        );
        if caret.clicked() {
            if row.expanded {
                env.ws.panels.expanded.remove(&row.id);
            } else {
                env.ws.panels.expanded.insert(row.id);
            }
        }
    }
    x += 14.0;
    let dim = hidden || locked;
    let text_color = if dim {
        color::TEXT_DISABLED
    } else {
        color::TEXT_PRIMARY
    };
    ui.painter().text(
        Pos2::new(x + 7.0, rect.center().y),
        Align2::CENTER_CENTER,
        kind_icon(object.kind),
        icons::font(size::ICON - 2.0),
        if dim {
            color::TEXT_DISABLED
        } else {
            color::TEXT_SECONDARY
        },
    );
    x += 14.0 + space::SM;

    // Name, or the inline rename editor.
    let toggles_width = 2.0 * size::HIT_MIN + space::XS;
    let name_rect = Rect::from_min_max(
        Pos2::new(x, rect.top()),
        Pos2::new(rect.right() - toggles_width - space::XS, rect.bottom()),
    );
    let renaming = matches!(&env.ws.panels.renaming, Some((id, _)) if *id == row.id);
    if renaming {
        rename_editor(ui, env, row.id, name_rect);
    } else {
        ui.painter().with_clip_rect(name_rect).text(
            Pos2::new(name_rect.left(), rect.center().y),
            Align2::LEFT_CENTER,
            &object.name,
            egui::TextStyle::Body.resolve(ui.style()),
            text_color,
        );
    }

    // Visibility and lock toggles.
    let eye_rect = Rect::from_min_size(
        Pos2::new(
            rect.right() - toggles_width,
            rect.center().y - size::HIT_MIN / 2.0,
        ),
        Vec2::splat(size::HIT_MIN),
    );
    let lock_rect = eye_rect.translate(Vec2::new(size::HIT_MIN + space::XS, 0.0));
    let visible = object.visible;
    let unlocked = !object.locked;
    let mut eye_clicked = false;
    let mut lock_clicked = false;
    ui.scope_builder(egui::UiBuilder::new().max_rect(eye_rect), |ui| {
        eye_clicked = toggle_icon_button(
            ui,
            visible,
            icons::VISIBLE,
            icons::HIDDEN,
            &format!("Hide {}", object.name),
            &format!("Show {}", object.name),
        );
    });
    ui.scope_builder(egui::UiBuilder::new().max_rect(lock_rect), |ui| {
        lock_clicked = toggle_icon_button(
            ui,
            unlocked,
            icons::UNLOCKED,
            icons::LOCKED,
            &format!("Lock {}", object.name),
            &format!("Unlock {}", object.name),
        );
    });
    if eye_clicked {
        env.ws.set_visible(row.id, !visible, env.now);
    }
    if lock_clicked {
        env.ws.set_locked(row.id, unlocked, env.now);
    }

    // Selection.
    if response.clicked() && !eye_clicked && !lock_clicked {
        let modifiers = ui.input(|i| i.modifiers);
        select_row(env, rows, row.id, locked, modifiers);
    }
    if response.double_clicked() && !locked {
        env.ws.panels.renaming = Some((row.id, object.name.clone()));
    }
    if response.secondary_clicked() && !selected && !locked {
        env.ws.selection = vec![row.id];
    }
    if response.drag_started() && !locked {
        let dragged = if selected {
            env.ws.selection.clone()
        } else {
            vec![row.id]
        };
        env.ws.panels.layers_drag = Some(dragged);
    }
    response.context_menu(|ui| {
        if ui.add(MenuRow::new("Rename")).clicked() {
            env.ws.panels.renaming = Some((row.id, object.name.clone()));
            ui.close();
        }
        for id in [
            CommandId::Duplicate,
            CommandId::Delete,
            CommandId::Group,
            CommandId::Ungroup,
            CommandId::NewLayer,
        ] {
            cmds.menu_item(ui, id);
        }
    });
    rect
}

fn select_row(
    env: &mut PanelEnv<'_>,
    rows: &[Row],
    id: ObjectId,
    locked: bool,
    modifiers: egui::Modifiers,
) {
    if locked {
        return;
    }
    let ws = &mut env.ws;
    if modifiers.command {
        if let Some(i) = ws.selection.iter().position(|s| *s == id) {
            ws.selection.remove(i);
        } else {
            ws.selection.push(id);
        }
    } else if modifiers.shift
        && let Some(anchor) = ws.panels.layers_anchor
    {
        let objects = &ws.project.surface().objects;
        ws.selection = row_range(rows, anchor, id)
            .into_iter()
            .filter(|r| tree::effective_flags(objects, *r) == Some((false, false)))
            .collect();
    } else {
        ws.selection = vec![id];
    }
    if !modifiers.shift {
        ws.panels.layers_anchor = Some(id);
    }
    ws.normalize_selection();
}

fn rename_editor(ui: &mut Ui, env: &mut PanelEnv<'_>, id: ObjectId, rect: Rect) {
    let Some((_, buffer)) = &mut env.ws.panels.renaming else {
        return;
    };
    let mut text = buffer.clone();
    let edit_id = ui.id().with(("rename", id));
    tp_ui::widgets::remember_escape(ui, edit_id);
    let response = ui.put(
        rect.shrink2(Vec2::new(0.0, 2.0)),
        TextEdit::singleline(&mut text)
            .id(edit_id)
            .margin(Margin::symmetric(4, 1)),
    );
    response.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, "Layer name"));
    if !response.has_focus() && !response.lost_focus() {
        response.request_focus();
    }
    *buffer = text.clone();
    if response.lost_focus() {
        let cancelled = tp_ui::widgets::take_escape(ui, edit_id);
        env.ws.panels.renaming = None;
        if !cancelled {
            env.ws.rename(id, &text, env.now);
        }
    }
}

fn drag_and_drop(ui: &mut Ui, env: &mut PanelEnv<'_>, rows: &[(Row, Rect)]) {
    let Some(dragged) = env.ws.panels.layers_drag.clone() else {
        return;
    };
    let pointer = ui.ctx().pointer_latest_pos();
    let target = pointer.and_then(|p| {
        rows.iter()
            .find(|(_, r)| r.y_range().contains(p.y))
            .map(|(row, r)| {
                let fraction = (p.y - r.top()) / r.height();
                (drop_placement(row, fraction), *r)
            })
    });
    if let Some((placement, r)) = target {
        let painter = ui.painter();
        match placement {
            Placement::IntoTop(_) => {
                painter.rect_stroke(
                    r.shrink(1.0),
                    CornerRadius::same(2),
                    Stroke::new(stroke::FOCUS, color::ACCENT),
                    StrokeKind::Inside,
                );
            }
            Placement::Above(_) => {
                painter.hline(r.x_range(), r.top(), Stroke::new(2.0, color::ACCENT));
            }
            Placement::Below(_) | Placement::Top => {
                painter.hline(r.x_range(), r.bottom(), Stroke::new(2.0, color::ACCENT));
            }
        }
    }
    if ui.input(|i| i.pointer.any_released()) {
        env.ws.panels.layers_drag = None;
        if let Some((placement, _)) = target {
            env.ws.move_objects(&dragged, placement, env.now);
        }
    }
}

#[cfg(test)]
mod tests {
    use tp_core::document::{Frame, ShapeKind};
    use tp_core::kurbo::{Point, Size};

    use super::*;

    fn rect(id: u64) -> Arc<Object> {
        Arc::new(Object::new(
            ObjectId(id),
            ShapeKind::rectangle(),
            Frame::new(Point::ORIGIN, Size::new(10.0, 10.0), 0.0),
        ))
    }

    fn tree() -> Vec<Arc<Object>> {
        vec![
            rect(1),
            Arc::new(Object::group(ObjectId(10), vec![rect(2), rect(3)])),
            rect(4),
        ]
    }

    #[test]
    fn rows_are_topmost_first_and_respect_expansion() {
        let collapsed = visible_rows(&tree(), &HashSet::new());
        assert_eq!(
            collapsed.iter().map(|r| r.id.0).collect::<Vec<_>>(),
            vec![4, 10, 1]
        );
        let expanded = visible_rows(&tree(), &HashSet::from([ObjectId(10)]));
        assert_eq!(
            expanded
                .iter()
                .map(|r| (r.id.0, r.depth))
                .collect::<Vec<_>>(),
            vec![(4, 0), (10, 0), (3, 1), (2, 1), (1, 0)]
        );
    }

    #[test]
    fn drop_zones() {
        let group = Row {
            id: ObjectId(10),
            depth: 0,
            is_group: true,
            expanded: false,
        };
        let shape = Row {
            id: ObjectId(1),
            depth: 0,
            is_group: false,
            expanded: false,
        };
        assert_eq!(
            drop_placement(&group, 0.5),
            Placement::IntoTop(ObjectId(10))
        );
        assert_eq!(drop_placement(&group, 0.1), Placement::Above(ObjectId(10)));
        assert_eq!(drop_placement(&group, 0.9), Placement::Below(ObjectId(10)));
        assert_eq!(drop_placement(&shape, 0.45), Placement::Above(ObjectId(1)));
        assert_eq!(drop_placement(&shape, 0.55), Placement::Below(ObjectId(1)));
    }

    #[test]
    fn ranges_follow_row_order() {
        let rows = visible_rows(&tree(), &HashSet::from([ObjectId(10)]));
        let ids = |v: Vec<ObjectId>| v.into_iter().map(|i| i.0).collect::<Vec<_>>();
        assert_eq!(
            ids(row_range(&rows, ObjectId(1), ObjectId(3))),
            vec![3, 2, 1]
        );
        assert_eq!(ids(row_range(&rows, ObjectId(4), ObjectId(4))), vec![4]);
    }
}
