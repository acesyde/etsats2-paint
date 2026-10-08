//! Text on the canvas: fonts, cached layouts, the anchor rule and cached
//! triangle meshes of glyph outlines.

use std::collections::HashMap;
use std::sync::Arc;

use tp_core::document::{CharStyle, Object, ObjectId, StrokeStyle, TextAlign, TextBlock};
use tp_core::kurbo::{Affine, BezPath, Point};
use tp_text::mesh::{self, Mesh};
use tp_text::{FontLibrary, GlyphCache, TextLayout};

/// Layout cache key: content and every style field.
fn layout_key(content: &str, style: &CharStyle) -> String {
    format!(
        "{}\u{1}{}\u{1}{}\u{1}{}\u{1}{}\u{1}{:?}\u{1}{}\u{1}{}",
        content,
        style.family,
        style.weight,
        style.italic,
        style.size,
        style.align,
        style.letter_spacing,
        style.line_height
    )
}

pub use tp_text::layout_to_doc;

/// Filled and stroked triangles of a text, in document space.
pub struct TextMesh {
    pub fill: Mesh,
    pub stroke: Option<Mesh>,
}

struct MeshEntry {
    ptr: usize,
    bucket: i32,
    frame: tp_core::document::Frame,
    mirrored: bool,
    block: TextBlock,
    stroke: Option<StrokeStyle>,
    mesh: Arc<TextMesh>,
    used: bool,
}

pub struct TextEngine {
    pub fonts: FontLibrary,
    glyphs: GlyphCache,
    layouts: HashMap<String, (Arc<TextLayout>, bool)>,
    meshes: HashMap<ObjectId, MeshEntry>,
    generation: u64,
    /// Font picker previews: each family name drawn in its own font.
    previews: HashMap<String, Arc<FontPreview>>,
    /// Meshes computed (cache misses), for tests and diagnostics.
    pub mesh_misses: u64,
}

/// A family name laid out in its own font, as triangles in layout space.
pub struct FontPreview {
    pub mesh: Mesh,
    pub size: tp_core::kurbo::Size,
    pub baseline: f64,
}

/// Size at which previews are laid out (scaled when drawn).
pub const PREVIEW_SIZE: f64 = 32.0;

impl Default for TextEngine {
    fn default() -> Self {
        Self::new(FontLibrary::bundled())
    }
}

impl TextEngine {
    pub fn new(fonts: FontLibrary) -> Self {
        let generation = fonts.generation();
        Self {
            fonts,
            glyphs: GlyphCache::default(),
            layouts: HashMap::new(),
            meshes: HashMap::new(),
            generation,
            previews: HashMap::new(),
            mesh_misses: 0,
        }
    }

    /// Picks up system fonts loaded in the background. Returns true when the
    /// font set changed (texts must be laid out again).
    pub fn poll_fonts(&mut self) -> bool {
        self.fonts.poll();
        if self.fonts.generation() == self.generation {
            return false;
        }
        self.generation = self.fonts.generation();
        self.layouts.clear();
        self.meshes.clear();
        self.previews.clear();
        true
    }

    pub fn layout(&mut self, content: &str, style: &CharStyle) -> Arc<TextLayout> {
        let key = layout_key(content, style);
        if let Some((layout, used)) = self.layouts.get_mut(&key) {
            *used = true;
            return layout.clone();
        }
        let layout = Arc::new(tp_text::layout(&mut self.fonts, content, style));
        self.layouts.insert(key, (layout.clone(), true));
        layout
    }

    pub fn block_layout(&mut self, block: &TextBlock) -> Arc<TextLayout> {
        self.layout(&block.content, &block.style)
    }

    /// The text anchor (first baseline at the left, center or right) in
    /// document space.
    pub fn anchor(&mut self, object: &Object) -> Option<Point> {
        let block = object.text.as_ref()?;
        let layout = self.block_layout(block);
        Some(layout_to_doc(object) * anchor_in(block, &layout))
    }

    /// Lays out `object` again after its content or style changed, keeping
    /// its anchor where it was with `previous` content and style.
    pub fn relayout(&mut self, object: &mut Object, previous: &TextBlock) {
        let Some(block) = object.text.clone() else {
            return;
        };
        let old = Object {
            text: Some(previous.clone()),
            ..object.clone()
        };
        let anchor = self.anchor(&old).unwrap_or(object.frame.center);
        self.place(object, &block, anchor);
    }

    /// Lays out `object` with its anchor at `anchor` (document space).
    pub fn place_at(&mut self, object: &mut Object, anchor: Point) {
        if let Some(block) = object.text.clone() {
            self.place(object, &block, anchor);
        }
    }

    fn place(&mut self, object: &mut Object, block: &TextBlock, anchor: Point) {
        let layout = self.block_layout(block);
        let size = layout.size;
        let mirror = if object.mirrored {
            Affine::scale_non_uniform(-1.0, 1.0)
        } else {
            Affine::IDENTITY
        };
        let text = object.text.as_mut().expect("text object");
        text.layout_size = size;
        object.frame.size = tp_core::kurbo::Size::new(
            size.width * text.scale.x.abs(),
            size.height * text.scale.y.abs(),
        );
        // As `layout_to_doc`, without the translation.
        let local = Affine::rotate(object.frame.rotation_deg.to_radians())
            * mirror
            * Affine::scale_non_uniform(text.scale.x, text.scale.y);
        let from_center =
            anchor_in(text, &layout) - Point::new(size.width / 2.0, size.height / 2.0);
        object.frame.center = anchor - (local * from_center.to_point()).to_vec2();
    }

    /// The text's glyph outlines in document space.
    pub fn outline(&mut self, object: &Object) -> BezPath {
        let Some(block) = &object.text else {
            return BezPath::new();
        };
        let layout = self.block_layout(block);
        let local = self.glyphs.outline(&mut self.fonts, &layout);
        layout_to_doc(object) * local
    }

    /// Triangles of the text for a zoom bucket.
    /// Each visible glyph of a text in document coordinates, in reading
    /// order, with the byte range of its characters.
    pub fn glyph_outlines(&mut self, object: &Object) -> Vec<tp_text::GlyphOutline> {
        let Some(block) = &object.text else {
            return Vec::new();
        };
        let layout = self.block_layout(block);
        let to_doc = layout_to_doc(object);
        self.glyphs
            .glyph_outlines(&mut self.fonts, &layout)
            .into_iter()
            .map(|g| tp_text::GlyphOutline {
                range: g.range,
                path: to_doc * g.path,
            })
            .collect()
    }

    pub fn mesh(&mut self, object: &Arc<Object>, bucket: i32, tolerance: f64) -> Arc<TextMesh> {
        let ptr = Arc::as_ptr(object) as usize;
        if let Some(e) = self.meshes.get_mut(&object.id)
            && e.ptr == ptr
            && e.bucket == bucket
            && e.frame == object.frame
            && e.mirrored == object.mirrored
            && e.stroke == object.stroke
            && object.text.as_ref() == Some(&e.block)
        {
            e.used = true;
            return e.mesh.clone();
        }
        self.mesh_misses += 1;
        let outline = self.outline(object);
        let mesh = Arc::new(TextMesh {
            fill: mesh::fill(&outline, tolerance),
            stroke: object
                .stroke
                .filter(|s| s.width > 0.0)
                .map(|s| crate::geometry_cache::stroke_mesh(&outline, &outline, &s, tolerance)),
        });
        self.meshes.insert(
            object.id,
            MeshEntry {
                ptr,
                bucket,
                frame: object.frame,
                mirrored: object.mirrored,
                block: object
                    .text
                    .clone()
                    .unwrap_or_else(|| TextBlock::new("", CharStyle::default())),
                stroke: object.stroke,
                mesh: mesh.clone(),
                used: true,
            },
        );
        mesh
    }

    /// The family name drawn in its own font, regular weight.
    pub fn preview(&mut self, family: &str) -> Arc<FontPreview> {
        self.styled_preview(family, family, 400, false)
    }

    /// `text` drawn in `family` at `weight`, upright or italic (a text
    /// style's name in its own look).
    pub fn styled_preview(
        &mut self,
        text: &str,
        family: &str,
        weight: u16,
        italic: bool,
    ) -> Arc<FontPreview> {
        let key = format!("{family}\u{1}{weight}\u{1}{italic}\u{1}{text}");
        if let Some(p) = self.previews.get(&key) {
            return p.clone();
        }
        let style = CharStyle {
            family: family.to_owned(),
            weight,
            italic,
            size: PREVIEW_SIZE,
            ..CharStyle::default()
        };
        let layout = tp_text::layout(&mut self.fonts, text, &style);
        let outline = self.glyphs.outline(&mut self.fonts, &layout);
        let preview = Arc::new(FontPreview {
            mesh: mesh::fill(&outline, 0.05),
            size: layout.size,
            baseline: layout.anchor().y,
        });
        self.previews.insert(key, preview.clone());
        preview
    }

    /// Drops cache entries not used since the previous call.
    pub fn prune(&mut self) {
        self.meshes.retain(|_, e| std::mem::take(&mut e.used));
        self.layouts.retain(|_, (_, used)| std::mem::take(used));
    }
}

/// Anchor of a block in layout coordinates.
fn anchor_in(block: &TextBlock, layout: &TextLayout) -> Point {
    let baseline = layout.anchor().y;
    let factor = match block.style.align {
        TextAlign::Left => 0.0,
        TextAlign::Center => 0.5,
        TextAlign::Right => 1.0,
    };
    Point::new(layout.size.width * factor, baseline)
}

#[cfg(test)]
mod tests {
    use tp_core::document::Frame;
    use tp_core::kurbo::Size;

    use super::*;

    fn text(content: &str, align: TextAlign) -> Object {
        let block = TextBlock::new(
            content,
            CharStyle {
                align,
                ..CharStyle::default()
            },
        );
        let mut o = Object::new(
            ObjectId(1),
            tp_core::document::ShapeKind::Text,
            Frame::new(Point::ORIGIN, Size::new(1.0, 1.0), 0.0),
        );
        o.text = Some(block);
        o
    }

    #[test]
    fn anchor_stays_fixed_when_typing() {
        let mut engine = TextEngine::default();
        for align in [TextAlign::Left, TextAlign::Center, TextAlign::Right] {
            let mut o = text("ACE", align);
            o.frame.rotation_deg = 30.0;
            engine.place_at(&mut o, Point::new(500.0, 400.0));
            let a = engine.anchor(&o).unwrap();
            assert!((a - Point::new(500.0, 400.0)).hypot() < 1e-6);
            let previous = o.text.clone().unwrap();
            o.text.as_mut().unwrap().content = "ACE Logistics".into();
            engine.relayout(&mut o, &previous);
            let b = engine.anchor(&o).unwrap();
            assert!((b - a).hypot() < 1e-6, "{align:?}: {a:?} {b:?}");
            assert!(o.frame.size.width > 400.0);
        }
    }

    #[test]
    fn mesh_is_cached_and_follows_stroke() {
        let mut engine = TextEngine::default();
        let mut o = text("O", TextAlign::Left);
        engine.place_at(&mut o, Point::new(100.0, 300.0));
        let o = Arc::new(o);
        let m = engine.mesh(&o, 0, 0.1);
        assert!(!m.fill.is_empty() && m.stroke.is_none());
        engine.mesh(&o, 0, 0.1);
        assert_eq!(engine.mesh_misses, 1);
        let mut stroked = (*o).clone();
        stroked.stroke = Some(StrokeStyle {
            paint: tp_core::document::Rgba::rgb(0, 0, 0).into(),
            width: 6.0,
            ..Default::default()
        });
        let m = engine.mesh(&Arc::new(stroked), 0, 0.1);
        assert!(m.stroke.as_ref().is_some_and(|s| !s.is_empty()));
    }
}
