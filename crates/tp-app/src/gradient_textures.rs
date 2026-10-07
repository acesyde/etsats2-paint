//! Gradient ramp textures for the canvas: a gradient is drawn by texturing
//! the object's triangle meshes with a ramp whose texture coordinates follow
//! gradient space. Coordinates are affine in position, so the GPU
//! interpolates them exactly.

use std::cell::RefCell;
use std::collections::HashMap;

use egui::{Color32, ColorImage, TextureHandle, TextureId, TextureOptions};
use tp_core::document::{Frame, Gradient, GradientKind, Paint, Rgba};
use tp_core::kurbo::{Affine, Point};

/// Width of linear ramps (texels).
pub const LINEAR_SIZE: usize = 2048;
/// Side of radial ramps (texels), covering gradient space [-1, 1]².
pub const RADIAL_SIZE: usize = 512;

fn color32(c: Rgba) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
}

/// The ramp image of `g`. Linear: texel `i` holds the color at
/// `i / (LINEAR_SIZE - 1)`. Radial: texel `(i, j)` holds the color at the
/// distance from the center of gradient point
/// `(-1 + 2i / (RADIAL_SIZE - 1), -1 + 2j / (RADIAL_SIZE - 1))`, capped at 1.
pub fn ramp_image(g: &Gradient) -> ColorImage {
    match g.kind {
        GradientKind::Linear => {
            let pixels = (0..LINEAR_SIZE)
                .map(|i| color32(g.color_at(i as f32 / (LINEAR_SIZE - 1) as f32)))
                .collect();
            ColorImage::new([LINEAR_SIZE, 1], pixels)
        }
        GradientKind::Radial => {
            let n = RADIAL_SIZE;
            let at = |i: usize| -1.0 + 2.0 * i as f32 / (n - 1) as f32;
            let mut pixels = Vec::with_capacity(n * n);
            for j in 0..n {
                for i in 0..n {
                    let d = at(i).hypot(at(j)).min(1.0);
                    pixels.push(color32(g.color_at(d)));
                }
            }
            ColorImage::new([n, n], pixels)
        }
    }
}

/// Maps gradient space to texture coordinates (texel centers at the ramp's
/// defining points).
fn gradient_to_uv(kind: GradientKind) -> Affine {
    match kind {
        GradientKind::Linear => {
            let w = LINEAR_SIZE as f64;
            // u = (0.5 + t (W - 1)) / W, v = 0.5.
            Affine::new([(w - 1.0) / w, 0.0, 0.0, 0.0, 0.5 / w, 0.5])
        }
        GradientKind::Radial => {
            let n = RADIAL_SIZE as f64;
            // u = (0.5 + (g + 1) / 2 (N - 1)) / N, same for v.
            let s = (n - 1.0) / (2.0 * n);
            let t = (0.5 + (n - 1.0) / 2.0) / n;
            Affine::new([s, 0.0, 0.0, s, t, t])
        }
    }
}

/// Maps document points to texture coordinates of `g`'s ramp on an object
/// with `frame`. `None` when the gradient is degenerate.
pub fn doc_to_uv(g: &Gradient, frame: &Frame) -> Option<Affine> {
    let to_doc = g.to_document(frame)?;
    Some(gradient_to_uv(g.kind) * to_doc.inverse())
}

/// How a mesh is colored.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MeshPaint {
    Solid(Color32),
    /// Textured with a ramp; `tint` carries the opacity (premultiplied).
    Ramp {
        texture: TextureId,
        doc_to_uv: Affine,
        tint: Color32,
    },
}

impl MeshPaint {
    /// Whether nothing would be visible.
    pub fn is_invisible(&self) -> bool {
        match self {
            MeshPaint::Solid(c) => c.a() == 0,
            MeshPaint::Ramp { tint, .. } => tint.a() == 0,
        }
    }

    /// Color and texture coordinates of a vertex at document point `p`.
    pub fn vertex(&self, p: Point) -> (Color32, egui::Pos2) {
        match self {
            MeshPaint::Solid(c) => (*c, egui::epaint::WHITE_UV),
            MeshPaint::Ramp {
                doc_to_uv, tint, ..
            } => {
                let uv = *doc_to_uv * p;
                (*tint, egui::pos2(uv.x as f32, uv.y as f32))
            }
        }
    }

    pub fn texture_id(&self) -> TextureId {
        match self {
            MeshPaint::Solid(_) => TextureId::default(),
            MeshPaint::Ramp { texture, .. } => *texture,
        }
    }
}

/// Identity of a ramp: kind and stops.
#[derive(Clone, PartialEq, Eq, Hash)]
struct Key {
    radial: bool,
    stops: Vec<(u32, [u8; 4])>,
}

impl Key {
    fn of(g: &Gradient) -> Self {
        Self {
            radial: g.kind == GradientKind::Radial,
            stops: g
                .stops()
                .iter()
                .map(|s| {
                    (
                        s.offset.to_bits(),
                        [s.color.r, s.color.g, s.color.b, s.color.a],
                    )
                })
                .collect(),
        }
    }
}

struct Entry {
    texture: TextureHandle,
    used: bool,
}

fn texture_options() -> TextureOptions {
    TextureOptions {
        magnification: egui::TextureFilter::Linear,
        minification: egui::TextureFilter::Linear,
        wrap_mode: egui::TextureWrapMode::ClampToEdge,
        mipmap_mode: None,
    }
}

/// Ramp textures of the gradients drawn, dropped when unused for a frame.
/// Shared through `&self` (drawing code holds the workspace immutably).
#[derive(Default)]
pub struct GradientTextures {
    entries: RefCell<HashMap<Key, Entry>>,
}

impl GradientTextures {
    /// The ramp texture of `g`, created on first use.
    pub fn texture(&self, ctx: &egui::Context, g: &Gradient) -> TextureId {
        let mut entries = self.entries.borrow_mut();
        let entry = entries.entry(Key::of(g)).or_insert_with(|| Entry {
            texture: ctx.load_texture("gradient", ramp_image(g), texture_options()),
            used: false,
        });
        entry.used = true;
        entry.texture.id()
    }

    /// How to color a mesh painted with `paint` on an object with `frame`
    /// at `opacity`. A degenerate gradient is its last stop's color.
    pub fn mesh_paint(
        &self,
        ctx: &egui::Context,
        paint: &Paint,
        frame: &Frame,
        opacity: f32,
    ) -> MeshPaint {
        match paint {
            Paint::Solid(c) => MeshPaint::Solid(color32(c.with_opacity(opacity))),
            Paint::Gradient(g) => match doc_to_uv(g, frame) {
                Some(doc_to_uv) => MeshPaint::Ramp {
                    texture: self.texture(ctx, g),
                    doc_to_uv,
                    tint: Color32::from_white_alpha((opacity.clamp(0.0, 1.0) * 255.0).round() as u8),
                },
                None => MeshPaint::Solid(color32(g.last_color().with_opacity(opacity))),
            },
        }
    }

    /// Drops the textures not used since the last call.
    pub fn prune(&mut self) {
        self.entries
            .get_mut()
            .retain(|_, e| std::mem::take(&mut e.used));
    }

    pub fn len(&self) -> usize {
        self.entries.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.borrow().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use tp_core::document::ColorStop;
    use tp_core::kurbo::Size;

    use super::*;

    const RED: Rgba = Rgba::rgb(255, 0, 0);
    const BLUE: Rgba = Rgba::rgb(0, 0, 255);

    fn red_blue(kind: GradientKind) -> Gradient {
        Gradient::new(kind, &[ColorStop::new(0.0, RED), ColorStop::new(1.0, BLUE)])
    }

    #[test]
    fn linear_ramp_matches_color_at() {
        let g = red_blue(GradientKind::Linear);
        let img = ramp_image(&g);
        assert_eq!(img.size, [LINEAR_SIZE, 1]);
        for i in [0, 1, 700, LINEAR_SIZE - 1] {
            let want = color32(g.color_at(i as f32 / (LINEAR_SIZE - 1) as f32));
            assert_eq!(img.pixels[i], want, "texel {i}");
        }
    }

    #[test]
    fn radial_ramp_matches_color_at() {
        let g = red_blue(GradientKind::Radial);
        let img = ramp_image(&g);
        let n = RADIAL_SIZE;
        assert_eq!(img.size, [n, n]);
        // Corners lie beyond the radius: the last stop.
        assert_eq!(img.pixels[0], color32(BLUE));
        // The middle row at the left edge: distance 1.
        assert_eq!(img.pixels[(n / 2) * n], color32(BLUE));
        let center = img.pixels[(n / 2) * n + n / 2];
        assert!(center.r() > 250 && center.b() < 5, "{center:?}");
    }

    #[test]
    fn uv_maps_the_ends_to_the_end_texels() {
        let frame = Frame::new(Point::new(200.0, 50.0), Size::new(400.0, 100.0), 0.0);
        let g = red_blue(GradientKind::Linear);
        let m = doc_to_uv(&g, &frame).unwrap();
        let w = LINEAR_SIZE as f64;
        let start = m * Point::new(0.0, 50.0);
        let end = m * Point::new(400.0, 77.0);
        assert!((start.x - 0.5 / w).abs() < 1e-9 && (start.y - 0.5).abs() < 1e-9);
        assert!((end.x - (w - 0.5) / w).abs() < 1e-9 && (end.y - 0.5).abs() < 1e-9);
        let r = red_blue(GradientKind::Radial);
        let m = doc_to_uv(&r, &frame).unwrap();
        let n = RADIAL_SIZE as f64;
        let center = m * Point::new(200.0, 50.0);
        assert!((center.x - 0.5).abs() < 1e-9 && (center.y - 0.5).abs() < 1e-9);
        // The radius point (right edge) is at the center of the last texel.
        let edge = m * Point::new(400.0, 50.0);
        assert!((edge.x - (n - 0.5) / n).abs() < 1e-9);
    }

    #[test]
    fn textures_are_reused_and_pruned() {
        let ctx = egui::Context::default();
        let mut cache = GradientTextures::default();
        let g = red_blue(GradientKind::Linear);
        let a = cache.texture(&ctx, &g);
        let b = cache.texture(&ctx, &g);
        assert_eq!(a, b);
        assert_eq!(cache.len(), 1);
        cache.prune();
        assert_eq!(cache.len(), 1, "used since the last prune");
        cache.prune();
        assert!(cache.is_empty());
    }

    #[test]
    fn degenerate_gradient_is_a_solid_last_color() {
        let ctx = egui::Context::default();
        let cache = GradientTextures::default();
        let mut g = red_blue(GradientKind::Linear);
        g.end = g.start;
        let frame = Frame::new(Point::ORIGIN, Size::new(10.0, 10.0), 0.0);
        let p = cache.mesh_paint(&ctx, &Paint::Gradient(g), &frame, 1.0);
        assert_eq!(p, MeshPaint::Solid(color32(BLUE)));
    }
}
