//! Triangulation of outlines (concave, with holes) for drawing.

use lyon_tessellation::path::Path;
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillRule, FillTessellator, FillVertex, LineJoin, StrokeOptions,
    StrokeTessellator, StrokeVertex, VertexBuffers,
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
                if open {
                    builder.end(true);
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
        builder.end(true);
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
    let mut buffers: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
    let options = StrokeOptions::tolerance(tolerance.max(1e-3) as f32)
        .with_line_width(width as f32)
        .with_line_join(LineJoin::MiterClip)
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
}
