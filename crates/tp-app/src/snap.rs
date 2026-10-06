//! Snapping: targets (grid, guides, artboard, other objects, anchor points)
//! built once per gesture, and nearest-target queries.

use std::collections::HashSet;

use tp_core::Axis;
use tp_core::document::{ObjectId, tree};
use tp_core::kurbo::{Point, Rect};

use crate::prefs::ViewAids;
use crate::workspace::Workspace;

/// What a position snapped to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Source {
    Grid,
    Guide,
    Artboard,
    /// An edge or center of another object, with its bounds.
    Object(Rect),
    /// An anchor point, or a corner or center of another object's bounds.
    Point,
}

/// One snap, for drawing alignment lines.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SnapHit {
    /// A line along `axis` at `value` (x = value for `Axis::Vertical`).
    /// `extent` is the span to draw along the line (document units); `None`
    /// means across the whole canvas.
    Line {
        axis: Axis,
        value: f64,
        extent: Option<(f64, f64)>,
        source: Source,
    },
    /// A snapped point.
    Point(Point),
}

/// Snap targets for one gesture.
#[derive(Clone, Debug, Default)]
pub struct Snapper {
    /// Vertical lines (x values), sorted.
    xs: Vec<(f64, Source)>,
    /// Horizontal lines (y values), sorted.
    ys: Vec<(f64, Source)>,
    points: Vec<Point>,
    grid: Option<f64>,
    side: f64,
}

fn sorted(mut v: Vec<(f64, Source)>) -> Vec<(f64, Source)> {
    v.sort_by(|a, b| a.0.total_cmp(&b.0));
    v
}

/// Nearest value of a sorted list to `v`.
fn nearest(list: &[(f64, Source)], v: f64) -> Option<(f64, Source)> {
    let i = list.partition_point(|(x, _)| *x < v);
    [i.checked_sub(1), Some(i)]
        .into_iter()
        .flatten()
        .filter_map(|j| list.get(j).copied())
        .min_by(|a, b| (a.0 - v).abs().total_cmp(&(b.0 - v).abs()))
}

impl Snapper {
    /// Targets of the active surface, leaving out `exclude` (and their
    /// descendants) and, when set, guide `skip_guide`.
    pub fn new(ws: &Workspace, exclude: &[ObjectId], skip_guide: Option<usize>) -> Self {
        let surface = ws.project.surface();
        let aids: ViewAids = ws.aids;
        let side = surface.size;
        let mut excluded: HashSet<ObjectId> = HashSet::new();
        for id in exclude {
            if let Some(o) = surface.get(*id) {
                excluded.insert(o.id);
                excluded.extend(o.shapes().iter().map(|s| s.id));
            }
        }
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        let mut points = Vec::new();
        for v in [0.0, side / 2.0, side] {
            xs.push((v, Source::Artboard));
            ys.push((v, Source::Artboard));
        }
        if aids.guides {
            for (i, g) in surface.guides.iter().enumerate() {
                if Some(i) == skip_guide {
                    continue;
                }
                match g.axis {
                    Axis::Vertical => xs.push((g.position, Source::Guide)),
                    Axis::Horizontal => ys.push((g.position, Source::Guide)),
                }
            }
        }
        for (o, _) in tree::draw_list(&surface.objects) {
            if excluded.contains(&o.id) {
                continue;
            }
            let b = o.bounding_box();
            let center = b.center();
            for x in [b.x0, center.x, b.x1] {
                xs.push((x, Source::Object(b)));
            }
            for y in [b.y0, center.y, b.y1] {
                ys.push((y, Source::Object(b)));
            }
            points.extend([
                Point::new(b.x0, b.y0),
                Point::new(b.x1, b.y0),
                Point::new(b.x1, b.y1),
                Point::new(b.x0, b.y1),
                center,
            ]);
            if let Some(path) = o.path_data() {
                let a = o.frame.affine();
                points.extend(
                    path.subpaths
                        .iter()
                        .flat_map(|s| &s.nodes)
                        .map(|n| a * n.point),
                );
            }
        }
        Self {
            xs: sorted(xs),
            ys: sorted(ys),
            points,
            grid: aids.grid.then_some(aids.grid_spacing),
            side,
        }
    }

    fn lines(&self, axis: Axis) -> &[(f64, Source)] {
        match axis {
            Axis::Vertical => &self.xs,
            Axis::Horizontal => &self.ys,
        }
    }

    /// Nearest target to `v` along `axis` (x values for `Axis::Vertical`).
    fn nearest_target(&self, axis: Axis, v: f64) -> Option<(f64, Source)> {
        let line = nearest(self.lines(axis), v);
        let grid = self.grid.and_then(|s| {
            let g = (v / s).round() * s;
            (0.0..=self.side).contains(&g).then_some((g, Source::Grid))
        });
        [line, grid]
            .into_iter()
            .flatten()
            .min_by(|a, b| (a.0 - v).abs().total_cmp(&(b.0 - v).abs()))
    }

    /// The smallest correction bringing one of `candidates` onto a target
    /// within `tolerance`, with its hit. `span` is the element's extent across
    /// the line (for drawing the alignment line).
    pub fn snap_axis(
        &self,
        axis: Axis,
        candidates: &[f64],
        span: (f64, f64),
        tolerance: f64,
    ) -> Option<(f64, SnapHit)> {
        candidates
            .iter()
            .filter_map(|v| {
                let (t, source) = self.nearest_target(axis, *v)?;
                let d = t - v;
                (d.abs() <= tolerance).then_some((d, t, source))
            })
            .min_by(|a, b| a.0.abs().total_cmp(&b.0.abs()))
            .map(|(d, value, source)| {
                let extent = match source {
                    Source::Object(b) => {
                        let (lo, hi) = match axis {
                            Axis::Vertical => (b.y0, b.y1),
                            Axis::Horizontal => (b.x0, b.x1),
                        };
                        Some((lo.min(span.0), hi.max(span.1)))
                    }
                    _ => None,
                };
                (
                    d,
                    SnapHit::Line {
                        axis,
                        value,
                        extent,
                        source,
                    },
                )
            })
    }

    /// Snaps a point: onto an anchor point or a bounds corner or center
    /// (both axes) if one is within `tolerance`, else each axis separately.
    pub fn snap_point(&self, p: Point, tolerance: f64) -> (Point, Vec<SnapHit>) {
        let best = self
            .points
            .iter()
            .map(|q| (q.distance(p), *q))
            .filter(|(d, _)| *d <= tolerance)
            .min_by(|a, b| a.0.total_cmp(&b.0));
        if let Some((_, q)) = best {
            return (q, vec![SnapHit::Point(q)]);
        }
        let mut out = p;
        let mut hits = Vec::new();
        if let Some((d, hit)) = self.snap_axis(Axis::Vertical, &[p.x], (p.y, p.y), tolerance) {
            out.x += d;
            hits.push(hit);
        }
        if let Some((d, hit)) = self.snap_axis(Axis::Horizontal, &[p.y], (p.x, p.x), tolerance) {
            out.y += d;
            hits.push(hit);
        }
        (out, hits)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use tp_core::document::{Frame, Object, ShapeKind};
    use tp_core::kurbo::Size;
    use tp_core::{Guide, Project, TextureResolution};

    use super::*;

    fn ws() -> Workspace {
        Workspace::new(Project::new("T", TextureResolution::R4096))
    }

    fn rect(ws: &mut Workspace, x: f64, y: f64, w: f64, h: f64) -> ObjectId {
        ws.project.add(Object::new(
            ObjectId(0),
            ShapeKind::rectangle(),
            Frame::new(Point::new(x, y), Size::new(w, h), 0.0),
        ))
    }

    #[test]
    fn nearest_target_within_tolerance_wins() {
        let mut ws = ws();
        ws.project.add_guide(Guide::new(Axis::Vertical, 1000.0));
        ws.project.add_guide(Guide::new(Axis::Vertical, 1010.0));
        let s = Snapper::new(&ws, &[], None);
        let (d, hit) = s
            .snap_axis(Axis::Vertical, &[1007.0], (0.0, 0.0), 6.0)
            .unwrap();
        assert_eq!(d, 3.0);
        assert!(matches!(
            hit,
            SnapHit::Line {
                value: 1010.0,
                source: Source::Guide,
                ..
            }
        ));
        assert!(
            s.snap_axis(Axis::Vertical, &[1030.0], (0.0, 0.0), 6.0)
                .is_none()
        );
        // Among candidates (left, center, right), the closest correction wins.
        let (d, _) = s
            .snap_axis(Axis::Vertical, &[995.0, 1095.0, 1195.0], (0.0, 0.0), 6.0)
            .unwrap();
        assert_eq!(d, 5.0);
    }

    #[test]
    fn hidden_guides_and_grid_do_not_snap() {
        let mut ws = ws();
        ws.project.add_guide(Guide::new(Axis::Horizontal, 777.0));
        ws.aids.guides = false;
        let s = Snapper::new(&ws, &[], None);
        assert!(
            s.snap_axis(Axis::Horizontal, &[775.0], (0.0, 0.0), 6.0)
                .is_none()
        );
        // Grid hidden: 64 is not a target.
        assert!(
            s.snap_axis(Axis::Horizontal, &[66.0], (0.0, 0.0), 6.0)
                .is_none()
        );
        ws.aids.grid = true;
        ws.aids.grid_spacing = 64.0;
        let s = Snapper::new(&ws, &[], None);
        let (d, hit) = s
            .snap_axis(Axis::Horizontal, &[66.0], (0.0, 0.0), 6.0)
            .unwrap();
        assert_eq!(d, -2.0);
        assert!(matches!(
            hit,
            SnapHit::Line {
                source: Source::Grid,
                extent: None,
                ..
            }
        ));
    }

    #[test]
    fn artboard_center_and_edited_objects() {
        let mut ws = ws();
        let a = rect(&mut ws, 500.0, 500.0, 200.0, 100.0);
        let s = Snapper::new(&ws, &[a], None);
        // The edited rectangle's own edges (400, 500, 600) are not targets.
        assert!(
            s.snap_axis(Axis::Vertical, &[402.0], (0.0, 0.0), 6.0)
                .is_none()
        );
        let (d, _) = s
            .snap_axis(Axis::Vertical, &[2045.0], (0.0, 0.0), 6.0)
            .unwrap();
        assert_eq!(d, 3.0, "artboard center 2048");
        let s = Snapper::new(&ws, &[], None);
        let (_, hit) = s
            .snap_axis(Axis::Vertical, &[402.0], (900.0, 1000.0), 6.0)
            .unwrap();
        assert!(
            matches!(
                hit,
                SnapHit::Line {
                    extent: Some((450.0, 1000.0)),
                    ..
                }
            ),
            "alignment line spans both elements: {hit:?}"
        );
    }

    #[test]
    fn point_targets_win_over_axis_targets() {
        let mut ws = ws();
        rect(&mut ws, 1000.0, 1000.0, 200.0, 200.0);
        ws.project.add_guide(Guide::new(Axis::Vertical, 903.0));
        let s = Snapper::new(&ws, &[], None);
        // Near the corner (900, 900) and the guide at x = 903.
        let (p, hits) = s.snap_point(Point::new(902.0, 902.0), 6.0);
        assert_eq!(p, Point::new(900.0, 900.0));
        assert_eq!(hits, vec![SnapHit::Point(Point::new(900.0, 900.0))]);
        // Far from points: each axis separately.
        let (p, hits) = s.snap_point(Point::new(905.0, 1500.0), 6.0);
        assert_eq!(p, Point::new(903.0, 1500.0));
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn queries_stay_fast_with_many_objects() {
        let mut ws = ws();
        for i in 0..5000 {
            let x = f64::from(i % 100) * 40.0;
            let y = f64::from(i / 100) * 80.0;
            rect(&mut ws, x, y, 30.0, 30.0);
        }
        let start = Instant::now();
        let s = Snapper::new(&ws, &[], None);
        let built = start.elapsed();
        let start = Instant::now();
        for i in 0..1000 {
            let v = f64::from(i) * 3.7;
            let _ = s.snap_axis(Axis::Vertical, &[v, v + 50.0, v + 100.0], (0.0, 0.0), 6.0);
            let _ = s.snap_point(Point::new(v, v), 6.0);
        }
        let per_query = start.elapsed() / 1000;
        println!("build {built:?}, query {per_query:?}");
        // Point queries scan linearly; generous bound for debug builds.
        assert!(
            per_query < std::time::Duration::from_millis(5),
            "{per_query:?}"
        );
    }
}
