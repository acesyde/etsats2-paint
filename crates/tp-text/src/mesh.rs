//! Triangulation of outlines (concave, with holes) for drawing.

use lyon_tessellation::path::Path;
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillRule, FillTessellator, FillVertex, LineCap, LineJoin,
    StrokeOptions, StrokeTessellator, StrokeVertex, VertexBuffers,
};
use tp_core::kurbo::{BezPath, PathEl};

/// Triangles: positions and indices (three per triangle).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

fn point(p: tp_core::kurbo::Point) -> lyon_tessellation::math::Point {
    lyon_tessellation::math::point(p.x as f32, p.y as f32)
}

fn to_lyon(path: &BezPath) -> Path {
    let mut builder = Path::builder();
    let mut open = false;
    for el in path.elements() {
        match *el {
            PathEl::MoveTo(p) => {
                // A subpath without `ClosePath` stays open.
                if open {
                    builder.end(false);
                }
                builder.begin(point(p));
                open = true;
            }
            PathEl::LineTo(p) if open => {
                builder.line_to(point(p));
            }
            PathEl::QuadTo(c, p) if open => {
                builder.quadratic_bezier_to(point(c), point(p));
            }
            PathEl::CurveTo(c0, c1, p) if open => {
                builder.cubic_bezier_to(point(c0), point(c1), point(p));
            }
            PathEl::ClosePath if open => {
                builder.end(true);
                open = false;
            }
            _ => {}
        }
    }
    if open {
        builder.end(false);
    }
    builder.build()
}

/// Fills `path` (non-zero rule) with triangles within `tolerance`.
pub fn fill(path: &BezPath, tolerance: f64) -> Mesh {
    let mut buffers: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
    let options =
        FillOptions::tolerance(tolerance.max(1e-3) as f32).with_fill_rule(FillRule::NonZero);
    let result = FillTessellator::new().tessellate_path(
        &to_lyon(path),
        &options,
        &mut BuffersBuilder::new(&mut buffers, |v: FillVertex| v.position().to_array()),
    );
    if let Err(err) = result {
        tracing::warn!(?err, "fill tessellation failed");
    }
    Mesh {
        vertices: buffers.vertices,
        indices: buffers.indices,
    }
}

/// Strokes `path` with a centered line of `width`.
pub fn stroke(path: &BezPath, width: f64, tolerance: f64) -> Mesh {
    stroke_with_caps(path, width, tolerance, false)
}

/// Like [`stroke`], with round caps on open ends when `round_caps`.
pub fn stroke_with_caps(path: &BezPath, width: f64, tolerance: f64, round_caps: bool) -> Mesh {
    let mut buffers: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
    let cap = if round_caps {
        LineCap::Round
    } else {
        LineCap::Butt
    };
    let options = StrokeOptions::tolerance(tolerance.max(1e-3) as f32)
        .with_line_width(width as f32)
        .with_line_join(LineJoin::MiterClip)
        .with_line_cap(cap)
        .with_miter_limit(4.0);
    let result = StrokeTessellator::new().tessellate_path(
        &to_lyon(path),
        &options,
        &mut BuffersBuilder::new(&mut buffers, |v: StrokeVertex| v.position().to_array()),
    );
    if let Err(err) = result {
        tracing::warn!(?err, "stroke tessellation failed");
    }
    Mesh {
        vertices: buffers.vertices,
        indices: buffers.indices,
    }
}

#[cfg(test)]
mod tests {
    use tp_core::kurbo::{Circle, Rect, Shape};

    use super::*;

    fn area(mesh: &Mesh) -> f64 {
        mesh.indices
            .chunks(3)
            .map(|t| {
                let [a, b, c] = [0, 1, 2].map(|i| mesh.vertices[t[i] as usize]);
                f64::from(((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])).abs())
                    / 2.0
            })
            .sum()
    }

    #[test]
    fn holes_are_not_filled() {
        let mut ring = Rect::new(0.0, 0.0, 100.0, 100.0).to_path(0.1);
        // Counter-wound inner square: a hole under the non-zero rule.
        ring.move_to((25.0, 25.0));
        ring.line_to((25.0, 75.0));
        ring.line_to((75.0, 75.0));
        ring.line_to((75.0, 25.0));
        ring.close_path();
        let mesh = fill(&ring, 0.1);
        assert!((area(&mesh) - 7500.0).abs() < 1.0, "{}", area(&mesh));
    }

    #[test]
    fn stroke_area_matches_width() {
        let circle = Circle::new((0.0, 0.0), 100.0).to_path(0.1);
        let mesh = stroke(&circle, 10.0, 0.05);
        let expected = std::f64::consts::TAU * 100.0 * 10.0;
        assert!((area(&mesh) - expected).abs() / expected < 0.02);
    }

    #[test]
    fn open_subpath_stroke_is_not_closed() {
        let mut v = BezPath::new();
        v.move_to((0.0, 0.0));
        v.line_to((100.0, 0.0));
        v.line_to((100.0, 100.0));
        // Two 100 px segments, 10 px wide (no closing diagonal).
        let butt = stroke(&v, 10.0, 0.05);
        assert!((area(&butt) - 2000.0).abs() < 60.0, "{}", area(&butt));
        // Round caps add a disc of radius 5 in total.
        let round = stroke_with_caps(&v, 10.0, 0.05, true);
        let caps = std::f64::consts::PI * 25.0;
        assert!((area(&round) - area(&butt) - caps).abs() < 3.0);
    }
}
