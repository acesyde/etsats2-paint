//! The area a stroke paints: dashing, expansion to an outline (caps, joins,
//! miter limit) and, for Inside / Outside strokes, clipping against the
//! filled area. Drawn by both the canvas and the export.

use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::single::SingleFloatOverlay;
use kurbo::{BezPath, PathEl, Point};

use super::object::{Cap, Join, LineStyle, StrokeAlign};

fn kurbo_stroke(width: f64, line: &LineStyle) -> kurbo::Stroke {
    let cap = match line.cap {
        Cap::Butt => kurbo::Cap::Butt,
        Cap::Round => kurbo::Cap::Round,
        Cap::Square => kurbo::Cap::Square,
    };
    let join = match line.join {
        Join::Miter => kurbo::Join::Miter,
        Join::Round => kurbo::Join::Round,
        Join::Bevel => kurbo::Join::Bevel,
    };
    kurbo::Stroke::new(width)
        .with_caps(cap)
        .with_join(join)
        .with_miter_limit(line.miter_limit.max(1.0))
}

/// `path` dashed by `line`'s pattern (unchanged when solid).
pub fn dashed(path: &BezPath, line: &LineStyle) -> BezPath {
    match line.dash {
        Some(d) if d.gap > 0.0 => {
            // A zero-length dash still has to exist (dots with round caps):
            // give it a tiny length so the stroker draws its caps, taken
            // from the gap so the period stays exact.
            let on = d.dash.max(1e-3);
            let off = (d.gap - (on - d.dash)).max(1e-3);
            BezPath::from_iter(kurbo::dash(path.iter(), 0.0, &[on, off]))
        }
        _ => path.clone(),
    }
}

/// The outline of `path` stroked `width` wide with `line`'s caps and
/// joins (after dashing), as a path to fill with the non-zero rule.
pub fn expand(path: &BezPath, width: f64, line: &LineStyle, tolerance: f64) -> BezPath {
    let path = dashed(path, line);
    kurbo::stroke(
        path.iter(),
        &kurbo_stroke(width, line),
        &kurbo::StrokeOpts::default(),
        tolerance.max(1e-3),
    )
}

/// Widths of a line's casing (stroke color) and body (fill color) for a
/// line `width` wide outlined by a stroke `s` wide: the stroke straddles
/// the line's edges (Center), lies outside them, or inside them.
pub fn line_widths(width: f64, s: f64, align: StrokeAlign) -> (f64, f64) {
    match align {
        StrokeAlign::Center => (width + s, width - s),
        StrokeAlign::Outside => (width + 2.0 * s, width),
        StrokeAlign::Inside => (width, width - 2.0 * s),
    }
}

type Contours = Vec<Vec<[f64; 2]>>;

fn contours(path: &BezPath, tolerance: f64) -> Contours {
    let mut out: Contours = Vec::new();
    kurbo::flatten(path.iter(), tolerance.max(1e-3), |el| match el {
        PathEl::MoveTo(p) => out.push(vec![[p.x, p.y]]),
        PathEl::LineTo(p) => {
            if let Some(c) = out.last_mut() {
                c.push([p.x, p.y]);
            }
        }
        _ => {}
    });
    out.retain(|c| c.len() >= 3);
    out
}

fn to_path(shapes: &[Vec<Vec<[f64; 2]>>]) -> BezPath {
    let mut out = BezPath::new();
    for shape in shapes {
        for contour in shape {
            let mut points = contour.iter().map(|p| Point::new(p[0], p[1]));
            let Some(first) = points.next() else {
                continue;
            };
            out.move_to(first);
            for p in points {
                out.line_to(p);
            }
            out.close_path();
        }
    }
    out
}

/// The area painted by a stroke of `width` following `center`, as a path
/// filled with the non-zero rule. `fill` is the object's filled area, used
/// to clip Inside and Outside strokes (lines have none). Returns `None`
/// for the default stroke (centered, solid, round caps, miter joins), which
/// callers draw directly as before.
pub fn stroke_region(
    center: &BezPath,
    fill: Option<&BezPath>,
    width: f64,
    align: StrokeAlign,
    line: &LineStyle,
    tolerance: f64,
) -> Option<BezPath> {
    let clipped = fill.is_some() && align != StrokeAlign::Center;
    if !clipped && line.is_default() {
        return None;
    }
    let expansion = if clipped { 2.0 * width } else { width };
    let outline = expand(center, expansion, line, tolerance);
    let Some(fill) = fill.filter(|_| clipped) else {
        return Some(outline);
    };
    let rule = match align {
        StrokeAlign::Inside => OverlayRule::Intersect,
        _ => OverlayRule::Difference,
    };
    let subject = contours(&outline, tolerance);
    let clip = contours(fill, tolerance);
    if subject.is_empty() {
        return Some(BezPath::new());
    }
    let shapes = subject.overlay(&clip, rule, FillRule::NonZero);
    Some(to_path(&shapes))
}

#[cfg(test)]
mod tests {
    use kurbo::{Rect, Shape};

    use super::*;
    use crate::document::object::Dash;

    fn inside(path: &BezPath, p: (f64, f64)) -> bool {
        path.winding(Point::new(p.0, p.1)) != 0
    }

    fn square(x0: f64, y0: f64, side: f64) -> BezPath {
        Rect::new(x0, y0, x0 + side, y0 + side).to_path(0.1)
    }

    #[test]
    fn default_stroke_takes_the_fast_path() {
        let s = square(0.0, 0.0, 100.0);
        assert!(
            stroke_region(
                &s,
                Some(&s),
                10.0,
                StrokeAlign::Center,
                &LineStyle::default(),
                0.1
            )
            .is_none()
        );
    }

    #[test]
    fn inside_border_stays_within() {
        let s = square(0.0, 0.0, 400.0);
        let r = stroke_region(
            &s,
            Some(&s),
            30.0,
            StrokeAlign::Inside,
            &LineStyle::default(),
            0.05,
        )
        .unwrap();
        assert!(inside(&r, (15.0, 200.0)), "the outer 30 px band");
        assert!(inside(&r, (29.0, 200.0)));
        assert!(!inside(&r, (31.0, 200.0)), "not deeper than 30 px");
        assert!(!inside(&r, (-1.0, 200.0)), "nothing outside the square");
        let b = r.bounding_box();
        assert!(b.x0 >= -1e-6 && b.x1 <= 400.0 + 1e-6);
    }

    #[test]
    fn outside_around_a_counter() {
        // An "O": outer square with an opposite-wound counter.
        let mut o = square(0.0, 0.0, 300.0);
        o.move_to((100.0, 100.0));
        o.line_to((100.0, 200.0));
        o.line_to((200.0, 200.0));
        o.line_to((200.0, 100.0));
        o.close_path();
        let r = stroke_region(
            &o,
            Some(&o),
            20.0,
            StrokeAlign::Outside,
            &LineStyle::default(),
            0.05,
        )
        .unwrap();
        assert!(inside(&r, (-10.0, 150.0)), "outside the outer edge");
        assert!(
            inside(&r, (110.0, 150.0)),
            "inside the counter, along its edge"
        );
        assert!(!inside(&r, (50.0, 150.0)), "never over the filled area");
        assert!(!inside(&r, (150.0, 150.0)), "only 20 px into the counter");
    }

    #[test]
    fn dotted_pattern_gives_separate_dots() {
        let mut line = BezPath::new();
        line.move_to((0.0, 0.0));
        line.line_to((200.0, 0.0));
        let style = LineStyle {
            dash: Some(Dash {
                dash: 0.0,
                gap: 20.0,
            }),
            cap: Cap::Round,
            ..LineStyle::default()
        };
        let r = stroke_region(&line, None, 10.0, StrokeAlign::Center, &style, 0.05).unwrap();
        assert!(inside(&r, (0.0, 0.0)) && inside(&r, (20.0, 0.0)) && inside(&r, (40.0, 0.0)));
        assert!(!inside(&r, (10.0, 0.0)), "gap between dots");
        assert!(inside(&r, (20.0, 4.0)), "round dot of radius 5");
    }

    #[test]
    fn caps_extend_as_expected() {
        let mut line = BezPath::new();
        line.move_to((0.0, 0.0));
        line.line_to((100.0, 0.0));
        let with = |cap| LineStyle {
            cap,
            ..LineStyle::default()
        };
        let butt = expand(&line, 20.0, &with(Cap::Butt), 0.05).bounding_box();
        let square = expand(&line, 20.0, &with(Cap::Square), 0.05).bounding_box();
        let round = expand(&line, 20.0, &with(Cap::Round), 0.05).bounding_box();
        assert!((butt.x1 - 100.0).abs() < 1e-6);
        assert!((square.x1 - 110.0).abs() < 1e-6);
        assert!((round.x1 - 110.0).abs() < 0.1);
        // Square caps fill the corner, round caps do not.
        let sq = expand(&line, 20.0, &with(Cap::Square), 0.05);
        let rd = expand(&line, 20.0, &with(Cap::Round), 0.05);
        assert!(inside(&sq, (109.0, 9.0)) && !inside(&rd, (109.0, 9.0)));
    }

    #[test]
    fn joins_differ_at_a_sharp_corner() {
        let mut v = BezPath::new();
        v.move_to((0.0, 0.0));
        v.line_to((50.0, 100.0));
        v.line_to((100.0, 0.0));
        let at = |join| {
            let s = LineStyle {
                join,
                miter_limit: 10.0,
                ..LineStyle::default()
            };
            expand(&v, 20.0, &s, 0.05).bounding_box().y1
        };
        let (miter, round, bevel) = (at(Join::Miter), at(Join::Round), at(Join::Bevel));
        assert!(miter > round && round > bevel, "{miter} {round} {bevel}");
    }

    #[test]
    fn degenerate_inputs_do_not_panic() {
        let empty = BezPath::new();
        let style = LineStyle {
            dash: Some(Dash {
                dash: 5.0,
                gap: 5.0,
            }),
            ..LineStyle::default()
        };
        let r =
            stroke_region(&empty, Some(&empty), 10.0, StrokeAlign::Inside, &style, 0.1).unwrap();
        assert!(r.elements().is_empty());
        let mut dot = BezPath::new();
        dot.move_to((5.0, 5.0));
        dot.line_to((5.0, 5.0));
        let _ = stroke_region(&dot, None, 10.0, StrokeAlign::Center, &style, 0.1);
        let tiny = square(0.0, 0.0, 1.0);
        let r = stroke_region(
            &tiny,
            Some(&tiny),
            4.0,
            StrokeAlign::Outside,
            &LineStyle::default(),
            0.05,
        )
        .unwrap();
        assert!(inside(&r, (-1.0, 0.5)) && !inside(&r, (0.5, 0.5)));
    }
}
