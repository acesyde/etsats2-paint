//! Rulers, grid and guides: geometry and painting.

use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};
use tp_core::{Axis, Guide};
use tp_ui::tokens::canvas as tokens;

use crate::viewport::ScreenMap;

/// Ruler graduations: labelled step and the step of minor ticks, in texture
/// pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RulerSteps {
    pub major: f64,
    pub minor: Option<f64>,
}

/// Smallest step in {1, 2, 5}·10ᵏ texture pixels whose labels are at least
/// `RULER_LABEL_GAP` points apart at `scale` (points per texture pixel),
/// with minor ticks at a tenth, fifth or half of it when they stay 5 points
/// apart.
pub fn ruler_steps(scale: f64) -> RulerSteps {
    let gap = f64::from(tokens::RULER_LABEL_GAP);
    let mut base = 10f64
        .powi((gap / scale).log10().floor() as i32 - 1)
        .max(1e-3);
    let major = loop {
        let found = [1.0, 2.0, 5.0]
            .iter()
            .map(|m| base * m)
            .find(|s| s * scale >= gap);
        if let Some(s) = found {
            break s;
        }
        base *= 10.0;
    };
    let minor = [10.0, 5.0, 2.0]
        .iter()
        .map(|d| major / d)
        .find(|s| s * scale >= 5.0);
    RulerSteps { major, minor }
}

/// Distance between drawn grid lines for a grid of `spacing` at `scale`:
/// `spacing · 2ᵏ` with the smallest k keeping lines `GRID_MIN_GAP` apart.
pub fn grid_step(spacing: f64, scale: f64) -> f64 {
    let spacing = spacing.max(1e-3);
    let mut step = spacing;
    while step * scale < f64::from(tokens::GRID_MIN_GAP) {
        step *= 2.0;
    }
    step
}

/// Screen coordinate of a guide (y for horizontal, x for vertical).
pub fn guide_screen(guide: &Guide, map: &ScreenMap) -> f32 {
    match guide.axis {
        Axis::Horizontal => guide.position as f32 * map.scale + map.offset.y,
        Axis::Vertical => guide.position as f32 * map.scale + map.offset.x,
    }
}

/// Index of the guide within `GUIDE_HIT` points of `pointer`, nearest first.
pub fn guide_at(guides: &[Guide], map: &ScreenMap, pointer: Pos2) -> Option<usize> {
    guides
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let along = match g.axis {
                Axis::Horizontal => pointer.y,
                Axis::Vertical => pointer.x,
            };
            (i, (guide_screen(g, map) - along).abs())
        })
        .filter(|(_, d)| *d <= tokens::GUIDE_HIT)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// Grid lines over the artboard (`artboard` in texture pixels), clipped to
/// the visible `area`.
pub fn paint_grid(painter: &Painter, map: &ScreenMap, area: Rect, side: f64, spacing: f64) {
    let scale = f64::from(map.scale);
    let step = grid_step(spacing, scale);
    let top_left = map.to_screen(tp_core::kurbo::Point::ORIGIN);
    let bottom_right = map.to_screen(tp_core::kurbo::Point::new(side, side));
    let board = Rect::from_min_max(top_left, bottom_right).intersect(area);
    if !board.is_positive() {
        return;
    }
    let visible = |from: f32, to: f32, offset: f32| {
        let a = (f64::from(from - offset) / scale / step).floor().max(0.0) as i64;
        let b = (f64::from(to - offset) / scale / step)
            .ceil()
            .min(side / step) as i64;
        a..=b
    };
    let color = |pos: f64| {
        let index = (pos / spacing).round() as i64;
        if index % 8 == 0 {
            tokens::GRID_MAJOR
        } else {
            tokens::GRID_MINOR
        }
    };
    for i in visible(board.left(), board.right(), map.offset.x) {
        let pos = i as f64 * step;
        let x = pos as f32 * map.scale + map.offset.x;
        if x < board.left() - 0.5 || x > board.right() + 0.5 {
            continue;
        }
        painter.vline(x, board.y_range(), Stroke::new(1.0, color(pos)));
    }
    for i in visible(board.top(), board.bottom(), map.offset.y) {
        let pos = i as f64 * step;
        let y = pos as f32 * map.scale + map.offset.y;
        if y < board.top() - 0.5 || y > board.bottom() + 0.5 {
            continue;
        }
        painter.hline(board.x_range(), y, Stroke::new(1.0, color(pos)));
    }
}

/// A guide across the whole canvas `area`.
pub fn paint_guide(painter: &Painter, map: &ScreenMap, area: Rect, guide: &Guide, color: Color32) {
    let at = guide_screen(guide, map);
    match guide.axis {
        Axis::Horizontal => {
            painter.hline(area.x_range(), at, Stroke::new(1.0, color));
        }
        Axis::Vertical => {
            painter.vline(at, area.y_range(), Stroke::new(1.0, color));
        }
    }
}

/// Paints a ruler along `rect`: horizontal for `Axis::Horizontal` (top
/// ruler, graduating x), vertical otherwise; `pointer` is marked.
pub fn paint_ruler(
    painter: &Painter,
    rect: Rect,
    axis: Axis,
    map: &ScreenMap,
    pointer: Option<Pos2>,
) {
    painter.rect_filled(rect, 0, tokens::RULER_BG);
    let scale = f64::from(map.scale);
    let steps = ruler_steps(scale);
    let (from, to, offset) = match axis {
        Axis::Horizontal => (rect.left(), rect.right(), map.offset.x),
        Axis::Vertical => (rect.top(), rect.bottom(), map.offset.y),
    };
    let doc = |s: f32| f64::from(s - offset) / scale;
    let screen = |d: f64| d as f32 * map.scale + offset;
    let font = FontId::proportional(9.0);
    let tick = |painter: &Painter, at: f32, len: f32| match axis {
        Axis::Horizontal => {
            painter.vline(
                at,
                (rect.bottom() - len)..=rect.bottom(),
                Stroke::new(1.0, tokens::RULER_TICK),
            );
        }
        Axis::Vertical => {
            painter.hline(
                (rect.right() - len)..=rect.right(),
                at,
                Stroke::new(1.0, tokens::RULER_TICK),
            );
        }
    };
    if let Some(minor) = steps.minor {
        let first = (doc(from) / minor).floor() as i64;
        let last = (doc(to) / minor).ceil() as i64;
        for i in first..=last {
            tick(painter, screen(i as f64 * minor), 4.0);
        }
    }
    let first = (doc(from) / steps.major).floor() as i64;
    let last = (doc(to) / steps.major).ceil() as i64;
    for i in first..=last {
        let value = i as f64 * steps.major;
        let at = screen(value);
        tick(painter, at, rect_thickness(rect, axis));
        let label = format_ruler(value);
        match axis {
            Axis::Horizontal => {
                painter.text(
                    Pos2::new(at + 3.0, rect.top() + 2.0),
                    Align2::LEFT_TOP,
                    label,
                    font.clone(),
                    tokens::RULER_TEXT,
                );
            }
            Axis::Vertical => {
                // Rotated text is not available: stack short labels.
                painter.text(
                    Pos2::new(rect.left() + 2.0, at + 3.0),
                    Align2::LEFT_TOP,
                    label,
                    font.clone(),
                    tokens::RULER_TEXT,
                );
            }
        }
    }
    if let Some(p) = pointer {
        let marker = Stroke::new(1.0, tokens::RULER_MARKER);
        match axis {
            Axis::Horizontal if (from..=to).contains(&p.x) => {
                painter.vline(p.x, rect.y_range(), marker);
            }
            Axis::Vertical if (from..=to).contains(&p.y) => {
                painter.hline(rect.x_range(), p.y, marker);
            }
            _ => {}
        }
    }
    let edge = match axis {
        Axis::Horizontal => [rect.left_bottom(), rect.right_bottom()],
        Axis::Vertical => [rect.right_top(), rect.right_bottom()],
    };
    painter.line_segment(edge, Stroke::new(1.0, tokens::RULER_TICK));
}

fn rect_thickness(rect: Rect, axis: Axis) -> f32 {
    match axis {
        Axis::Horizontal => rect.height(),
        Axis::Vertical => rect.width(),
    }
}

fn format_ruler(value: f64) -> String {
    if (value - value.round()).abs() < 1e-6 {
        format!("{}", value.round() as i64)
    } else {
        tp_i18n::localize_number(&format!("{value:.1}")).into_owned()
    }
}

/// Ruler value under a screen position, for the pointer marker tests.
pub fn ruler_value(axis: Axis, map: &ScreenMap, at: Pos2) -> f64 {
    let doc = map.to_doc(at);
    match axis {
        Axis::Horizontal => doc.x,
        Axis::Vertical => doc.y,
    }
}

/// The three regions of the canvas: top ruler, left ruler, inner canvas.
pub fn split(area: Rect) -> (Rect, Rect, Rect) {
    let r = tokens::RULER_SIZE;
    let top = Rect::from_min_max(
        area.min + Vec2::new(r, 0.0),
        Pos2::new(area.right(), area.top() + r),
    );
    let left = Rect::from_min_max(
        area.min + Vec2::new(0.0, r),
        Pos2::new(area.left() + r, area.bottom()),
    );
    let inner = Rect::from_min_max(area.min + Vec2::splat(r), area.max);
    (top, left, inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ruler_labels_are_far_enough_apart() {
        for scale in [0.01, 0.05, 0.18, 0.5, 1.0, 3.0, 16.0, 64.0] {
            let s = ruler_steps(scale);
            assert!(
                s.major * scale >= f64::from(tokens::RULER_LABEL_GAP),
                "{scale}: {s:?}"
            );
            // And not absurdly far: the next smaller nice step would be too close.
            assert!(
                s.major * scale < f64::from(tokens::RULER_LABEL_GAP) * 2.6,
                "{scale}: {s:?}"
            );
            if let Some(minor) = s.minor {
                assert!(minor * scale >= 5.0);
            }
        }
        assert_eq!(ruler_steps(0.18).major, 500.0);
        assert_eq!(ruler_steps(1.0).major, 100.0);
    }

    #[test]
    fn grid_is_thinned_when_zoomed_out() {
        // 16 px grid on a 4096 artboard at 10 %.
        let step = grid_step(16.0, 0.1);
        assert!(step * 0.1 >= 8.0);
        assert_eq!(step, 128.0);
        assert_eq!(grid_step(64.0, 1.0), 64.0);
    }

    #[test]
    fn guide_hit_and_ruler_value() {
        let map = ScreenMap {
            scale: 0.5,
            offset: Vec2::new(100.0, 50.0),
        };
        let guides = [
            Guide::new(Axis::Vertical, 1000.0),
            Guide::new(Axis::Horizontal, 400.0),
        ];
        // x = 1000 → screen 600; y = 400 → screen 250.
        assert_eq!(guide_at(&guides, &map, Pos2::new(603.0, 10.0)), Some(0));
        assert_eq!(guide_at(&guides, &map, Pos2::new(10.0, 248.0)), Some(1));
        assert_eq!(guide_at(&guides, &map, Pos2::new(610.0, 10.0)), None);
        assert_eq!(
            ruler_value(Axis::Horizontal, &map, Pos2::new(700.0, 0.0)),
            1200.0
        );
        assert_eq!(
            ruler_value(Axis::Vertical, &map, Pos2::new(0.0, 450.0)),
            800.0
        );
    }
}
