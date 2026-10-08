//! CPU rendering of a surface with tiny-skia.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use resvg::tiny_skia::{
    self, Color, FillRule, FilterQuality, GradientStop, LineCap, LineJoin, LinearGradient, Paint,
    PathBuilder, Pixmap, PixmapPaint, RadialGradient, Shader, SpreadMode, Stroke, Transform,
};
use resvg::usvg;
use tp_core::document::{
    self as doc, AssetId, Frame, GradientKind, LineStyle, Object, Rgba, ShapeKind, stroke_region,
    tree,
};
use tp_core::kurbo::{Affine, BezPath, PathEl, Rect, Shape, Vec2};
use tp_core::{Asset, AssetKind, Project};
use tp_text::{FontLibrary, GlyphCache, layout_to_doc};

/// What to render.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderOptions {
    /// Output side in pixels (the surface is square).
    pub size: u32,
    /// Opaque or translucent background; `None` keeps it transparent.
    pub background: Option<Rgba>,
}

/// The render was cancelled through the progress callback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cancelled;

/// Fonts available to texts inside SVG files (the bundled fonts).
pub fn svg_options() -> &'static usvg::Options<'static> {
    static OPTIONS: OnceLock<usvg::Options<'static>> = OnceLock::new();
    OPTIONS.get_or_init(|| {
        let mut db = usvg::fontdb::Database::new();
        for data in tp_text::bundled_font_data() {
            db.load_font_data(data.to_vec());
        }
        db.set_sans_serif_family(tp_text::FALLBACK_FAMILY);
        usvg::Options {
            fontdb: Arc::new(db),
            ..usvg::Options::default()
        }
    })
}

fn color(c: Rgba, opacity: f32) -> Color {
    Color::from_rgba8(c.r, c.g, c.b, (f32::from(c.a) * opacity).round() as u8)
}

fn to_skia(path: &BezPath) -> Option<tiny_skia::Path> {
    let mut b = PathBuilder::new();
    let f = |v: f64| v as f32;
    for el in path.elements() {
        match *el {
            PathEl::MoveTo(p) => b.move_to(f(p.x), f(p.y)),
            PathEl::LineTo(p) => b.line_to(f(p.x), f(p.y)),
            PathEl::QuadTo(c, p) => b.quad_to(f(c.x), f(c.y), f(p.x), f(p.y)),
            PathEl::CurveTo(c0, c1, p) => {
                b.cubic_to(f(c0.x), f(c0.y), f(c1.x), f(c1.y), f(p.x), f(p.y));
            }
            PathEl::ClosePath => b.close(),
        }
    }
    b.finish()
}

fn skia_transform(a: Affine) -> Transform {
    let [sx, ky, kx, sy, tx, ty] = a.as_coeffs().map(|v| v as f32);
    Transform::from_row(sx, ky, kx, sy, tx, ty)
}

/// Document-space geometry of a shape: the filled area, the outline the
/// stroke follows, and the lines of a path's open subpaths with their width.
struct Outline {
    fill: BezPath,
    stroke: BezPath,
    lines: Option<(BezPath, f64)>,
}

impl Outline {
    fn same(path: BezPath) -> Self {
        Self {
            fill: path.clone(),
            stroke: path,
            lines: None,
        }
    }

    fn of(object: &Object) -> Self {
        Self {
            fill: object.fill_path(),
            stroke: object.stroke_path(),
            lines: object.line_path(),
        }
    }
}

/// Flattening tolerance of stroke regions, in output pixels.
const REGION_TOLERANCE: f64 = 0.05;

/// The tiny-skia paint of `p` on an object with `frame` at `opacity`,
/// `scale` mapping the document to pixels. A degenerate gradient paints its
/// last stop's color.
fn skia_paint(p: &doc::Paint, frame: &Frame, opacity: f32, scale: f64) -> Paint<'static> {
    let shader = match p {
        doc::Paint::Solid(c) => Shader::SolidColor(color(*c, opacity)),
        doc::Paint::Gradient(g) => {
            let stops: Vec<GradientStop> = g
                .stops()
                .iter()
                .map(|s| GradientStop::new(s.offset, color(s.color, opacity)))
                .collect();
            let shader = g.to_document(frame).and_then(|to_doc| {
                let t = skia_transform(Affine::scale(scale) * to_doc);
                let origin = tiny_skia::Point::from_xy(0.0, 0.0);
                match g.kind {
                    GradientKind::Linear => LinearGradient::new(
                        origin,
                        tiny_skia::Point::from_xy(1.0, 0.0),
                        stops,
                        SpreadMode::Pad,
                        t,
                    ),
                    GradientKind::Radial => {
                        RadialGradient::new(origin, 0.0, origin, 1.0, stops, SpreadMode::Pad, t)
                    }
                }
            });
            shader.unwrap_or_else(|| Shader::SolidColor(color(g.last_color(), opacity)))
        }
    };
    Paint {
        anti_alias: true,
        shader,
        ..Paint::default()
    }
}

/// Fills `path` (document space, `scale` mapping to pixels), non-zero.
fn fill_with(pixmap: &mut Pixmap, path: &BezPath, paint: &Paint, scale: f64) {
    if let Some(path) = to_skia(&(Affine::scale(scale) * path.clone())) {
        pixmap.fill_path(&path, paint, FillRule::Winding, Transform::identity(), None);
    }
}

/// A centered stroke `width` wide along `path`: tiny-skia for the default
/// style, the filled expanded outline otherwise.
fn stroke_with(
    pixmap: &mut Pixmap,
    path: &BezPath,
    paint: &Paint,
    width: f64,
    line: &LineStyle,
    scale: f64,
) {
    if !line.is_default() {
        let region = stroke_region::expand(path, width, line, REGION_TOLERANCE / scale);
        fill_with(pixmap, &region, paint, scale);
        return;
    }
    let Some(path) = to_skia(&(Affine::scale(scale) * path.clone())) else {
        return;
    };
    let stroke = Stroke {
        width: (width * scale) as f32,
        line_join: LineJoin::MiterClip,
        line_cap: LineCap::Round,
        miter_limit: 4.0,
        ..Stroke::default()
    };
    pixmap.stroke_path(&path, paint, &stroke, Transform::identity(), None);
}

/// Draws a shape, `scale` mapping to pixels: the fill, then the lines of
/// open subpaths (fill color, outlined by the stroke), then the stroke.
fn draw_path(pixmap: &mut Pixmap, outline: &Outline, object: &Object, opacity: f32, scale: f64) {
    let stroke = object
        .stroke
        .filter(|s| s.width > 0.0 && s.paint.is_visible());
    let fill = object
        .fill
        .is_visible()
        .then(|| skia_paint(&object.fill, &object.frame, opacity, scale));
    let stroke_paint = stroke.map(|s| skia_paint(&s.paint, &object.frame, opacity, scale));
    if let Some(fill) = &fill {
        fill_with(pixmap, &outline.fill, fill, scale);
    }
    // Lines: a casing in the stroke color, then the line body in the fill
    // color, their widths set by the stroke's alignment; both follow the
    // line's own dash, caps and joins.
    if let Some((lines, width)) = &outline.lines {
        let line = object
            .path
            .as_ref()
            .map(|p| p.line_style)
            .unwrap_or_default();
        let body = match (stroke, &stroke_paint) {
            (Some(s), Some(paint)) => {
                let (casing, body) = stroke_region::line_widths(*width, s.width, s.align);
                if casing > 0.0 {
                    stroke_with(pixmap, lines, paint, casing, &line, scale);
                }
                body
            }
            _ => *width,
        };
        if let Some(fill) = fill.as_ref().filter(|_| body > 0.0) {
            stroke_with(pixmap, lines, fill, body, &line, scale);
        }
    }
    if let (Some(s), Some(c)) = (stroke, &stroke_paint) {
        match stroke_region(
            &outline.stroke,
            Some(&outline.fill),
            s.width,
            s.align,
            &s.line,
            REGION_TOLERANCE / scale,
        ) {
            Some(region) if !region.elements().is_empty() => fill_with(pixmap, &region, c, scale),
            // Default stroke, or a degenerate region: a centered stroke.
            _ => stroke_with(
                pixmap,
                &outline.stroke,
                c,
                s.width,
                &LineStyle::default(),
                scale,
            ),
        }
    }
}

/// Decoded assets for the duration of a render.
#[derive(Default)]
struct AssetCache {
    raster: HashMap<AssetId, Option<Arc<image::RgbaImage>>>,
    svg: HashMap<AssetId, Option<Arc<usvg::Tree>>>,
}

impl AssetCache {
    fn raster(&mut self, asset: &Asset) -> Option<Arc<image::RgbaImage>> {
        self.raster
            .entry(asset.id)
            .or_insert_with(|| {
                image::load_from_memory(&asset.bytes)
                    .ok()
                    .map(|i| Arc::new(i.to_rgba8()))
            })
            .clone()
    }

    fn svg(&mut self, asset: &Asset) -> Option<Arc<usvg::Tree>> {
        self.svg
            .entry(asset.id)
            .or_insert_with(|| {
                usvg::Tree::from_data(&asset.bytes, svg_options())
                    .ok()
                    .map(Arc::new)
            })
            .clone()
    }
}

/// Straight RGBA image → premultiplied pixmap.
fn pixmap_from_rgba(image: &image::RgbaImage) -> Option<Pixmap> {
    let size = tiny_skia::IntSize::from_wh(image.width(), image.height())?;
    let mut data = image.as_raw().clone();
    for px in data.as_chunks_mut::<4>().0 {
        let a = u16::from(px[3]);
        for c in &mut px[..3] {
            *c = ((u16::from(*c) * a + 127) / 255) as u8;
        }
    }
    Pixmap::from_vec(data, size)
}

/// Draws an image object through a layer covering its pixel bounds, so the
/// object's opacity applies to the image as a whole.
fn draw_image(
    pixmap: &mut Pixmap,
    object: &Object,
    asset: &Asset,
    opacity: f32,
    scale: f64,
    cache: &mut AssetCache,
) {
    let out = Rect::new(
        0.0,
        0.0,
        f64::from(pixmap.width()),
        f64::from(pixmap.height()),
    );
    let bounds = (Affine::scale(scale) * object.frame.bounding_box().to_path(0.1))
        .bounding_box()
        .intersect(out)
        .expand();
    if bounds.width() < 1.0 || bounds.height() < 1.0 {
        return;
    }
    let Some(mut layer) = Pixmap::new(bounds.width() as u32, bounds.height() as u32) else {
        return;
    };
    let frame_size = object.frame.size;
    // Image space (0..w, 0..h) → layer pixels.
    let place = |w: f64, h: f64| {
        Affine::translate(Vec2::new(-bounds.x0, -bounds.y0))
            * Affine::scale(scale)
            * object.content_affine()
            * Affine::translate(Vec2::new(-frame_size.width / 2.0, -frame_size.height / 2.0))
            * Affine::scale_non_uniform(frame_size.width / w, frame_size.height / h)
    };
    match asset.kind {
        AssetKind::Svg => {
            let Some(tree) = cache.svg(asset) else {
                return;
            };
            let s = tree.size();
            let t = place(f64::from(s.width()), f64::from(s.height()));
            resvg::render(&tree, skia_transform(t), &mut layer.as_mut());
        }
        AssetKind::Raster => {
            let Some(source) = cache.raster(asset) else {
                return;
            };
            // Target size in output pixels; pre-shrink large sources so the
            // bicubic sampling does not alias.
            let target_w = (frame_size.width * scale).max(1.0);
            let target_h = (frame_size.height * scale).max(1.0);
            let shrunk;
            let img: &image::RgbaImage = if f64::from(source.width()) > target_w * 2.0
                && f64::from(source.height()) > target_h * 2.0
            {
                shrunk = image::imageops::resize(
                    &*source,
                    (target_w * 2.0).ceil() as u32,
                    (target_h * 2.0).ceil() as u32,
                    image::imageops::FilterType::Lanczos3,
                );
                &shrunk
            } else {
                &source
            };
            let Some(src) = pixmap_from_rgba(img) else {
                return;
            };
            let t = place(f64::from(img.width()), f64::from(img.height()));
            let paint = PixmapPaint {
                quality: FilterQuality::Bicubic,
                ..PixmapPaint::default()
            };
            layer.draw_pixmap(0, 0, src.as_ref(), &paint, skia_transform(t), None);
        }
    }
    let paint = PixmapPaint {
        opacity,
        ..PixmapPaint::default()
    };
    pixmap.draw_pixmap(
        bounds.x0 as i32,
        bounds.y0 as i32,
        layer.as_ref(),
        &paint,
        Transform::identity(),
        None,
    );
}

/// Renders surface `surface` of `project`. `progress(done, total)` is called
/// after each object; returning false cancels.
pub fn render(
    project: &Project,
    surface: usize,
    options: RenderOptions,
    fonts: &mut FontLibrary,
    progress: &mut dyn FnMut(usize, usize) -> bool,
) -> Result<Pixmap, Cancelled> {
    let size = options.size.max(1);
    let mut pixmap = Pixmap::new(size, size).expect("non-zero size");
    if let Some(bg) = options.background {
        pixmap.fill(color(bg, 1.0));
    }
    let Some(surface) = project.surfaces.get(surface) else {
        return Ok(pixmap);
    };
    let scale = f64::from(size) / surface.size;
    let objects = tree::draw_list(&surface.objects);
    let total = objects.len();
    let mut glyphs = GlyphCache::default();
    let mut assets = AssetCache::default();
    for (i, (object, opacity)) in objects.iter().enumerate() {
        match object.kind {
            ShapeKind::Rectangle { .. } | ShapeKind::Ellipse | ShapeKind::Polygon { .. } => {
                draw_path(
                    &mut pixmap,
                    &Outline::same(object.path()),
                    object,
                    *opacity,
                    scale,
                );
            }
            ShapeKind::Path => {
                draw_path(&mut pixmap, &Outline::of(object), object, *opacity, scale);
            }
            ShapeKind::Text => {
                if let Some(block) = &object.text {
                    let layout = tp_text::layout(fonts, &block.content, &block.style);
                    let outline = layout_to_doc(object) * glyphs.outline(fonts, &layout);
                    draw_path(
                        &mut pixmap,
                        &Outline::same(outline),
                        object,
                        *opacity,
                        scale,
                    );
                }
            }
            ShapeKind::Image { asset } => {
                if let Some(asset) = project.assets.get(&asset) {
                    draw_image(&mut pixmap, object, asset, *opacity, scale, &mut assets);
                }
            }
            ShapeKind::Group | ShapeKind::Instance { .. } => {}
        }
        if !progress(i + 1, total) {
            return Err(Cancelled);
        }
    }
    Ok(pixmap)
}

/// Straight (non-premultiplied) RGBA8 pixels of a pixmap.
pub fn to_rgba(pixmap: &Pixmap) -> image::RgbaImage {
    let mut out = image::RgbaImage::new(pixmap.width(), pixmap.height());
    for (dst, src) in out.pixels_mut().zip(pixmap.pixels()) {
        let c = src.demultiply();
        *dst = image::Rgba([c.red(), c.green(), c.blue(), c.alpha()]);
    }
    out
}
