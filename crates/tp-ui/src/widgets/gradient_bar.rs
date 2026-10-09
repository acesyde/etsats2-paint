//! Gradient bar: a preview of a gradient over a checkerboard with one marker
//! per color stop, to select, move, add and delete stops.

use egui::{
    Color32, CornerRadius, Id, Key, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, Vec2,
    WidgetInfo, WidgetType,
};
use tp_i18n::tr;

use super::color_widgets::{GradientPreview, paint_checkerboard};
use crate::tokens::{color, size, stroke};

/// Height of the gradient preview.
const BAR_HEIGHT: f32 = 20.0;
/// Width and height of a stop marker.
const MARKER: Vec2 = Vec2::new(12.0, 14.0);
/// Dragging a marker this far from the markers' row deletes its stop.
pub const DELETE_DISTANCE: f32 = 24.0;
/// Step of the arrow keys on a focused marker.
pub const KEY_STEP: f32 = 0.01;

/// Where the focused marker's id is remembered (see [`keyboard_claimed`]).
fn claim_id() -> Id {
    Id::new("gradient_bar_focused_marker")
}

/// Whether a gradient stop marker has keyboard focus: its arrow, Delete and
/// Backspace keys must then not trigger application shortcuts.
pub fn keyboard_claimed(ctx: &egui::Context) -> bool {
    let claimed: Option<Id> = ctx.data(|d| d.get_temp(claim_id()));
    claimed.is_some_and(|id| ctx.memory(|m| m.focused()) == Some(id))
}

/// What the user did with the bar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GradientBarEvent {
    /// Made a stop the selected one.
    Select(usize),
    /// Moved a stop to `offset`; `commit` ends the interaction (one undo step).
    Move {
        index: usize,
        offset: f32,
        commit: bool,
    },
    /// Clicked the bar away from markers at `offset`.
    Add { offset: f32 },
    /// Removed a stop (Delete key, or dragged off the bar).
    Delete(usize),
}

/// The bar for `preview`, with stop `selected` highlighted.
pub struct GradientBar<'a> {
    preview: &'a GradientPreview,
    selected: usize,
    id: Id,
}

impl<'a> GradientBar<'a> {
    pub fn new(
        preview: &'a GradientPreview,
        selected: usize,
        id_salt: impl std::hash::Hash + std::fmt::Debug,
    ) -> Self {
        Self {
            preview,
            selected,
            id: Id::new(("gradient_bar", id_salt)),
        }
    }

    /// Draws the bar; returns what happened this frame, if anything.
    pub fn show(self, ui: &mut Ui) -> Option<GradientBarEvent> {
        let stops = self.preview.stops();
        let width = ui.available_width().max(80.0);
        let (outer, bar_response) = ui.allocate_exact_size(
            Vec2::new(width, BAR_HEIGHT + MARKER.y + 2.0),
            Sense::click(),
        );
        // Markers' centers span the bar, inset by half a marker.
        let bar = Rect::from_min_size(
            outer.min + Vec2::new(MARKER.x / 2.0, 0.0),
            Vec2::new(width - MARKER.x, BAR_HEIGHT),
        );
        let to_offset = |x: f32| ((x - bar.left()) / bar.width()).clamp(0.0, 1.0);
        let painter = ui.painter_at(outer.expand(2.0));
        paint_checkerboard(&painter, bar, 5.0);
        self.preview.paint(&painter, bar);
        painter.rect_stroke(
            bar,
            CornerRadius::ZERO,
            Stroke::new(stroke::HAIRLINE, color::BORDER_STRONG),
            StrokeKind::Outside,
        );
        bar_response
            .widget_info(|| WidgetInfo::labeled(WidgetType::Other, true, tr("gradient-bar")));

        let mut event = None;
        let dragging_off = Id::new((self.id, "off"));
        let row_y = bar.bottom() + 1.0;
        for (i, (offset, c)) in stops.iter().enumerate() {
            let center_x = bar.left() + bar.width() * offset.clamp(0.0, 1.0);
            let rect = Rect::from_min_size(Pos2::new(center_x - MARKER.x / 2.0, row_y), MARKER);
            let hit =
                rect.expand2(Vec2::new(0.0, (size::HIT_MIN - MARKER.y) / 2.0).max(Vec2::ZERO));
            let response = ui.interact(hit, self.id.with(i), Sense::click_and_drag());
            let selected = i == self.selected;
            let name = tr!(
                "gradient-stop",
                index = i + 1,
                location = (offset * 100.0).round()
            );
            response
                .widget_info(|| WidgetInfo::selected(WidgetType::Button, true, selected, &name));
            if response.clicked() || response.drag_started() {
                response.request_focus();
                if !selected {
                    event = Some(GradientBarEvent::Select(i));
                }
            }
            let mut off = false;
            if response.dragged() || response.drag_stopped() {
                if let Some(p) = response.interact_pointer_pos() {
                    off = (p.y - (row_y + MARKER.y / 2.0)).abs() > DELETE_DISTANCE + MARKER.y;
                    if response.drag_stopped() {
                        event = Some(if off && stops.len() > 2 {
                            GradientBarEvent::Delete(i)
                        } else {
                            GradientBarEvent::Move {
                                index: i,
                                offset: to_offset(p.x),
                                commit: true,
                            }
                        });
                    } else if !off {
                        event = Some(GradientBarEvent::Move {
                            index: i,
                            offset: to_offset(p.x),
                            commit: false,
                        });
                    }
                }
                ui.data_mut(|d| d.insert_temp(dragging_off, off));
            }
            if response.has_focus() {
                ui.data_mut(|d| d.insert_temp(claim_id(), response.id));
                let (left, right, delete) = ui.input(|inp| {
                    (
                        inp.key_pressed(Key::ArrowLeft),
                        inp.key_pressed(Key::ArrowRight),
                        inp.key_pressed(Key::Delete) || inp.key_pressed(Key::Backspace),
                    )
                });
                if left || right {
                    let step = if right { KEY_STEP } else { -KEY_STEP };
                    event = Some(GradientBarEvent::Move {
                        index: i,
                        offset: (offset + step).clamp(0.0, 1.0),
                        commit: true,
                    });
                } else if delete && stops.len() > 2 {
                    event = Some(GradientBarEvent::Delete(i));
                }
            }
            // Marker: a small pentagon pointing up at the bar.
            let alpha = if off { 0.35 } else { 1.0 };
            let tip = Pos2::new(center_x, row_y);
            let body_top = row_y + 4.0;
            let points = vec![
                tip,
                Pos2::new(rect.right(), body_top),
                Pos2::new(rect.right(), rect.bottom()),
                Pos2::new(rect.left(), rect.bottom()),
                Pos2::new(rect.left(), body_top),
            ];
            let outline = if selected {
                Stroke::new(2.0, color::INDICATOR)
            } else {
                Stroke::new(stroke::HAIRLINE, color::BORDER_STRONG)
            };
            painter.add(Shape::convex_polygon(
                points,
                Color32::WHITE.gamma_multiply(alpha),
                Stroke::new(outline.width, outline.color.gamma_multiply(alpha)),
            ));
            let swatch = Rect::from_min_max(
                Pos2::new(rect.left() + 2.5, body_top + 1.5),
                Pos2::new(rect.right() - 2.5, rect.bottom() - 2.5),
            );
            paint_checkerboard(&painter, swatch, 2.0);
            painter.rect_filled(swatch, 0, c.gamma_multiply(alpha));
            if response.has_focus() {
                painter.rect_stroke(
                    rect.expand(2.0),
                    CornerRadius::same(2),
                    Stroke::new(stroke::FOCUS, color::FOCUS),
                    StrokeKind::Outside,
                );
            }
        }
        if event.is_none()
            && bar_response.clicked()
            && let Some(p) = bar_response.interact_pointer_pos()
            && p.y <= bar.bottom()
        {
            event = Some(GradientBarEvent::Add {
                offset: to_offset(p.x),
            });
        }
        event
    }
}

#[cfg(test)]
mod tests {
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable;

    use super::*;

    struct State {
        preview: GradientPreview,
        selected: usize,
        events: Vec<GradientBarEvent>,
    }

    fn harness() -> Harness<'static, State> {
        let state = State {
            preview: GradientPreview::new(
                false,
                &[(0.0, [255, 0, 0, 255]), (1.0, [0, 0, 255, 255])],
            ),
            selected: 0,
            events: Vec::new(),
        };
        Harness::builder()
            .with_size(Vec2::new(220.0, 60.0))
            .build_ui_state(
                |ui, s: &mut State| {
                    if let Some(e) = GradientBar::new(&s.preview, s.selected, "t").show(ui) {
                        if let GradientBarEvent::Select(i) = e {
                            s.selected = i;
                        }
                        s.events.push(e);
                    }
                },
                state,
            )
    }

    #[test]
    fn click_a_marker_selects_it() {
        let mut h = harness();
        h.get_by_label("Stop 2 at 100%").click();
        h.run();
        assert_eq!(h.state().selected, 1);
        assert_eq!(h.state().events[0], GradientBarEvent::Select(1));
    }

    #[test]
    fn arrows_move_and_delete_needs_three_stops() {
        let mut h = harness();
        h.get_by_label("Stop 1 at 0%").click();
        h.run();
        h.key_press(Key::ArrowRight);
        h.run();
        assert!(h.state().events.contains(&GradientBarEvent::Move {
            index: 0,
            offset: KEY_STEP,
            commit: true
        }));
        h.key_press(Key::Delete);
        h.run();
        assert!(
            !h.state()
                .events
                .iter()
                .any(|e| matches!(e, GradientBarEvent::Delete(_))),
            "two stops minimum"
        );
    }

    #[test]
    fn clicking_the_bar_adds_a_stop() {
        let mut h = harness();
        let bar = h.get_by_label("Gradient bar").rect();
        let at = Pos2::new(bar.center().x, bar.top() + 5.0);
        h.event(egui::Event::PointerMoved(at));
        for pressed in [true, false] {
            h.event(egui::Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
            h.run();
        }
        let added = h.state().events.iter().find_map(|e| match e {
            GradientBarEvent::Add { offset } => Some(*offset),
            _ => None,
        });
        let offset = added.expect("a stop is added");
        assert!((offset - 0.5).abs() < 0.03, "{offset}");
    }
}
