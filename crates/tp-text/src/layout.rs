//! Text layout (one style per text, no wrapping) and glyph outlines.

use std::collections::HashMap;
use std::sync::Arc;

use cosmic_text::{Attrs, Buffer, Family, Metrics, Shaping, Style, Weight, Wrap};
use skrifa::MetadataProvider;
use skrifa::instance::{LocationRef, Size as SkrifaSize};
use skrifa::outline::{DrawSettings, OutlinePen};
use tp_core::document::{CharStyle, TextAlign};
use tp_core::kurbo::{Affine, BezPath, Point, Rect, Size, Vec2};

use crate::fonts::FontLibrary;

/// One positioned glyph, in layout coordinates (origin at the top-left of
/// the layout box, y down; `y` is the baseline).
#[derive(Clone, Debug, PartialEq)]
pub struct PlacedGlyph {
    pub font_id: fontdb::ID,
    pub glyph_id: u16,
    pub weight: u16,
    pub size: f32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    /// Byte range of the source characters in the whole content.
    pub start: usize,
    pub end: usize,
}

/// One line of laid-out text.
#[derive(Clone, Debug, PartialEq)]
pub struct LineLayout {
    pub top: f64,
    pub baseline: f64,
    pub height: f64,
    /// Left edge after alignment.
    pub x: f64,
    pub width: f64,
    /// Byte range of the line in the content (without the newline).
    pub start: usize,
    pub end: usize,
}

/// Result of laying out a text.
#[derive(Clone, Debug, PartialEq)]
pub struct TextLayout {
    pub size: Size,
    pub lines: Vec<LineLayout>,
    pub glyphs: Vec<PlacedGlyph>,
    /// The requested family was not available (fallback used).
    pub missing_font: bool,
    pub align: TextAlign,
}

fn align_factor(align: TextAlign) -> f64 {
    match align {
        TextAlign::Left => 0.0,
        TextAlign::Center => 0.5,
        TextAlign::Right => 1.0,
    }
}

impl TextLayout {
    /// The anchor: first line's baseline at the left, center or right of
    /// the box (layout coordinates).
    pub fn anchor(&self) -> Point {
        let baseline = self.lines.first().map_or(0.0, |l| l.baseline);
        Point::new(self.size.width * align_factor(self.align), baseline)
    }

    /// Caret rectangle (zero width) before byte `offset`.
    pub fn caret(&self, offset: usize) -> Rect {
        let line = self.line_of(offset);
        let l = &self.lines[line];
        let x = self.x_at(line, offset);
        Rect::new(x, l.top, x, l.top + l.height)
    }

    /// Selection rectangles for the byte range `start..end`.
    pub fn selection_rects(&self, start: usize, end: usize) -> Vec<Rect> {
        let (start, end) = (start.min(end), start.max(end));
        if start == end {
            return Vec::new();
        }
        self.lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.end >= start && l.start <= end)
            .map(|(i, l)| {
                let x0 = self.x_at(i, start.max(l.start));
                let mut x1 = self.x_at(i, end.min(l.end));
                if end > l.end {
                    // Selection continues past the line: show the newline.
                    x1 += l.height * 0.25;
                }
                Rect::new(x0, l.top, x1, l.top + l.height)
            })
            .collect()
    }

    /// Index of the line containing byte `offset`.
    pub fn line_of(&self, offset: usize) -> usize {
        self.lines
            .iter()
            .position(|l| offset <= l.end)
            .unwrap_or(self.lines.len().saturating_sub(1))
    }

    /// x position of the caret before byte `offset` on `line`.
    fn x_at(&self, line: usize, offset: usize) -> f64 {
        let l = &self.lines[line];
        let mut line_glyphs = self
            .glyphs
            .iter()
            .filter(|g| g.start >= l.start && g.end <= l.end);
        if let Some(g) = line_glyphs.clone().find(|g| g.start >= offset) {
            return g.x;
        }
        match line_glyphs.next_back() {
            Some(last) => last.x + last.width,
            None => l.x,
        }
    }

    /// Byte offset nearest to `point` (layout coordinates).
    pub fn hit(&self, point: Point) -> usize {
        let line_index = self
            .lines
            .iter()
            .position(|l| point.y < l.top + l.height)
            .unwrap_or(self.lines.len().saturating_sub(1));
        let Some(l) = self.lines.get(line_index) else {
            return 0;
        };
        let mut best = (l.start, (self.x_at(line_index, l.start) - point.x).abs());
        for g in self
            .glyphs
            .iter()
            .filter(|g| g.start >= l.start && g.end <= l.end)
        {
            for (offset, x) in [(g.start, g.x), (g.end, g.x + g.width)] {
                let d = (x - point.x).abs();
                if d < best.1 {
                    best = (offset, d);
                }
            }
        }
        best.0
    }
}

/// Lays out `content` with `style` (no wrapping; lines split on '\n').
pub fn layout(fonts: &mut FontLibrary, content: &str, style: &CharStyle) -> TextLayout {
    let (family, missing_font) = fonts.resolve_family(&style.family);
    let family = family.to_owned();
    let weight = fonts.nearest_weight(&family, style.weight);
    let size = style.size.max(1.0) as f32;
    let line_height = (style.size * style.line_height / 100.0).max(1.0) as f32;
    let fs = &mut fonts.system;
    let mut buffer = Buffer::new(fs, Metrics::new(size, line_height));
    buffer.set_wrap(Wrap::None);
    buffer.set_size(None, None);
    let attrs = Attrs::new()
        .family(Family::Name(&family))
        .weight(Weight(weight))
        .style(if style.italic {
            Style::Italic
        } else {
            Style::Normal
        })
        .letter_spacing((style.letter_spacing / 1000.0) as f32);
    buffer.set_text(content, &attrs, Shaping::Advanced, None);
    buffer.shape_until_scroll(fs, false);

    // Byte offset where each source line starts.
    let mut line_starts = vec![0usize];
    for (i, c) in content.char_indices() {
        if c == '\n' {
            line_starts.push(i + 1);
        }
    }
    let line_end = |i: usize| {
        line_starts
            .get(i + 1)
            .map_or(content.len(), |next| next - 1)
    };

    let mut lines = Vec::new();
    let mut glyphs = Vec::new();
    for run in buffer.layout_runs() {
        let start = line_starts
            .get(run.line_i)
            .copied()
            .unwrap_or(content.len());
        lines.push(LineLayout {
            top: f64::from(run.line_top),
            baseline: f64::from(run.line_y),
            height: f64::from(run.line_height),
            x: 0.0,
            width: f64::from(run.line_w),
            start,
            end: line_end(run.line_i),
        });
        for g in run.glyphs {
            glyphs.push(PlacedGlyph {
                font_id: g.font_id,
                glyph_id: g.glyph_id,
                weight: g.font_weight.0,
                size: g.font_size,
                x: f64::from(g.x + g.x_offset),
                y: f64::from(run.line_y + g.y_offset),
                width: f64::from(g.w),
                start: start + g.start,
                end: start + g.end,
            });
        }
    }
    if lines.is_empty() {
        lines.push(LineLayout {
            top: 0.0,
            baseline: f64::from(line_height) * 0.8,
            height: f64::from(line_height),
            x: 0.0,
            width: 0.0,
            start: 0,
            end: 0,
        });
    }

    // Alignment: shift each line inside the widest line.
    let width = lines.iter().map(|l| l.width).fold(0.0, f64::max);
    let factor = align_factor(style.align);
    for line in &mut lines {
        let dx = (width - line.width) * factor;
        line.x = dx;
        for g in glyphs
            .iter_mut()
            .filter(|g| g.start >= line.start && g.end <= line.end)
        {
            g.x += dx;
        }
    }
    let height = lines
        .last()
        .map_or(f64::from(line_height), |l| l.top + l.height);
    TextLayout {
        size: Size::new(width.max(1.0), height.max(1.0)),
        lines,
        glyphs,
        missing_font,
        align: style.align,
    }
}

/// Maps layout coordinates (top-left origin, y down) of a text object to
/// the document: centered on the frame, scaled by the text's scale, then
/// rotated and placed by the frame. Shared by the canvas and exports.
pub fn layout_to_doc(object: &tp_core::document::Object) -> Affine {
    let Some(block) = &object.text else {
        return object.frame.affine();
    };
    let size = block.layout_size;
    object.frame.affine()
        * Affine::scale_non_uniform(block.scale.x, block.scale.y)
        * Affine::translate(Vec2::new(-size.width / 2.0, -size.height / 2.0))
}

/// Converts skrifa pen commands into a `kurbo` path at a glyph position.
struct KurboPen {
    path: BezPath,
    transform: Affine,
}

impl KurboPen {
    fn p(&self, x: f32, y: f32) -> Point {
        self.transform * Point::new(f64::from(x), f64::from(y))
    }
}

impl OutlinePen for KurboPen {
    fn move_to(&mut self, x: f32, y: f32) {
        let p = self.p(x, y);
        self.path.move_to(p);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.p(x, y);
        self.path.line_to(p);
    }
    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        let (c, p) = (self.p(cx0, cy0), self.p(x, y));
        self.path.quad_to(c, p);
    }
    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let (c0, c1, p) = (self.p(cx0, cy0), self.p(cx1, cy1), self.p(x, y));
        self.path.curve_to(c0, c1, p);
    }
    fn close(&mut self) {
        self.path.close_path();
    }
}

/// Size at which glyphs are cached; drawn outlines are scaled from it.
const CACHE_SIZE: f32 = 1000.0;

/// Cache of glyph outlines at [`CACHE_SIZE`], y down, origin at the
/// glyph's baseline origin.
#[derive(Default)]
pub struct GlyphCache {
    glyphs: HashMap<(fontdb::ID, u16, u16), Arc<BezPath>>,
    generation: u64,
}

impl GlyphCache {
    fn glyph(&mut self, fonts: &mut FontLibrary, g: &PlacedGlyph) -> Arc<BezPath> {
        if self.generation != fonts.generation() {
            self.glyphs.clear();
            self.generation = fonts.generation();
        }
        let key = (g.font_id, g.glyph_id, g.weight);
        if let Some(path) = self.glyphs.get(&key) {
            return path.clone();
        }
        let path = fonts
            .system
            .db()
            .with_face_data(g.font_id, |data, index| {
                let font = skrifa::FontRef::from_index(data, index).ok()?;
                let outline = font
                    .outline_glyphs()
                    .get(skrifa::GlyphId::new(u32::from(g.glyph_id)))?;
                let location = font.axes().location([("wght", f32::from(g.weight))]);
                let mut pen = KurboPen {
                    path: BezPath::new(),
                    // Font units are y-up: flip to y-down.
                    transform: Affine::scale_non_uniform(1.0, -1.0),
                };
                let settings = DrawSettings::unhinted(
                    SkrifaSize::new(CACHE_SIZE),
                    LocationRef::from(&location),
                );
                outline.draw(settings, &mut pen).ok()?;
                Some(pen.path)
            })
            .flatten()
            .unwrap_or_default();
        let path = Arc::new(path);
        self.glyphs.insert(key, path.clone());
        path
    }

    /// The whole text outline in layout coordinates.
    pub fn outline(&mut self, fonts: &mut FontLibrary, layout: &TextLayout) -> BezPath {
        let mut out = BezPath::new();
        for g in &layout.glyphs {
            let path = self.glyph(fonts, g);
            let scale = f64::from(g.size / CACHE_SIZE);
            let placed = Affine::translate(Vec2::new(g.x, g.y)) * Affine::scale(scale);
            out.extend((placed * (*path).clone()).elements().iter().copied());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use tp_core::kurbo::Shape;

    use super::*;

    fn style() -> CharStyle {
        CharStyle::default()
    }

    #[test]
    fn width_grows_with_content() {
        let mut fonts = FontLibrary::bundled();
        let a = layout(&mut fonts, "ACE", &style());
        let b = layout(&mut fonts, "ACE Logistics", &style());
        assert!(b.size.width > a.size.width * 2.0, "{a:?} {b:?}");
        assert!(!a.missing_font);
    }

    #[test]
    fn two_lines_are_stacked() {
        let mut fonts = FontLibrary::bundled();
        let l = layout(&mut fonts, "Line 1\nLine 2", &style());
        assert_eq!(l.lines.len(), 2);
        assert!(l.lines[1].top >= l.lines[0].top + l.lines[0].height - 0.01);
        assert_eq!((l.lines[1].start, l.lines[1].end), (7, 13));
        let one = layout(&mut fonts, "Line 1", &style());
        assert!((l.size.height - 2.0 * one.size.height).abs() < 1.0);
    }

    #[test]
    fn centered_lines_share_a_center() {
        let mut fonts = FontLibrary::bundled();
        let s = CharStyle {
            align: TextAlign::Center,
            ..style()
        };
        let l = layout(&mut fonts, "WIDE LINE\nab", &s);
        let c0 = l.lines[0].x + l.lines[0].width / 2.0;
        let c1 = l.lines[1].x + l.lines[1].width / 2.0;
        assert!((c0 - c1).abs() < 0.5);
        assert!((l.anchor().x - l.size.width / 2.0).abs() < 1e-9);
    }

    #[test]
    fn letter_spacing_widens_text() {
        let mut fonts = FontLibrary::bundled();
        let normal = layout(&mut fonts, "SPACING", &style());
        let wide = layout(
            &mut fonts,
            "SPACING",
            &CharStyle {
                letter_spacing: 200.0,
                ..style()
            },
        );
        assert!(wide.size.width > normal.size.width + 100.0);
    }

    #[test]
    fn missing_font_is_flagged() {
        let mut fonts = FontLibrary::bundled();
        let l = layout(
            &mut fonts,
            "Hi",
            &CharStyle {
                family: "Some Missing Font".into(),
                ..style()
            },
        );
        assert!(l.missing_font);
        assert!(!l.glyphs.is_empty());
    }

    #[test]
    fn letter_o_outline_has_a_hole() {
        let mut fonts = FontLibrary::bundled();
        let l = layout(&mut fonts, "O", &style());
        let path = GlyphCache::default().outline(&mut fonts, &l);
        let subpaths = path
            .elements()
            .iter()
            .filter(|e| matches!(e, tp_core::kurbo::PathEl::MoveTo(_)))
            .count();
        assert_eq!(subpaths, 2, "outer contour and counter");
        let b = path.bounding_box();
        assert!(
            b.height() > 100.0 && b.y1 <= l.lines[0].baseline + 5.0,
            "{b:?} {:?}",
            l.lines[0]
        );
    }

    #[test]
    fn caret_and_hit_round_trip() {
        let mut fonts = FontLibrary::bundled();
        let l = layout(&mut fonts, "AB\nCD", &style());
        let c = l.caret(1);
        assert!(c.x0 > 0.0);
        assert_eq!(l.hit(Point::new(c.x0 + 1.0, c.center().y)), 1);
        let second = l.caret(4);
        assert!(second.y0 > c.y0);
        assert_eq!(l.hit(Point::new(second.x0, second.center().y)), 4);
        assert_eq!(l.selection_rects(0, 5).len(), 2);
    }
}
