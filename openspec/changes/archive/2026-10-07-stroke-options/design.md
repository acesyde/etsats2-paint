## Context

How strokes are drawn today:

| What | Canvas | Export (`tp-render`) |
|---|---|---|
| Rectangles, ellipses | egui `closed_line` | tiny-skia `stroke_path` (MiterClip, limit 4, round caps) |
| Polygons, closed paths | lyon stroke meshes (`stroke_with_caps`) | tiny-skia, as above |
| Lines (open paths) | casing technique: a band of the line width plus the stroke width in the stroke color, then the line narrowed by the stroke width in the fill color (round caps) | same |
| Texts | lyon stroke of the glyph outlines | tiny-skia, as above |

Two other facts shape the design:
- `StrokeStyle { color, width }` is `Copy`, and is saved in the v2 format, which is unreleased.
- kurbo 0.13 provides `dash()` and `stroke()` (stroke expansion to a fillable outline with caps, joins and miter limit), and `flo_curves` is already a dependency.

## Goals / Non-Goals

**Goals:**
- One geometry pipeline for all strokes, used by the canvas and the export alike, so they match.
- Defaults that reproduce today's rendering.
- Cached geometry, so panning stays fast.

**Non-Goals:**
- Variable width, arrowheads, multiple strokes, outlining strokes into editable paths.

## Decisions

### D1. Data model

```rust
pub struct LineStyle { pub dash: Option<Dash>, pub cap: Cap, pub join: Join, pub miter_limit: f64 }
pub struct Dash { pub dash: f64, pub gap: f64 }
pub enum Cap { Butt, Round, Square }        // default Round
pub enum Join { Miter, Round, Bevel }       // default Miter, miter_limit 4
pub enum StrokeAlign { Center, Inside, Outside }  // default Center
pub struct StrokeStyle { pub color, pub width, pub align: StrokeAlign, pub line: LineStyle }
```

- `PathData` gains `line_style: LineStyle` for line bodies.
- A line's outline uses `PathData.line_style`; shapes and texts use `stroke.line`. This is the rule from the spec: the stroke's own dash, cap and join don't apply to lines.
- Everything stays `Copy`/`PartialEq`. `Default` values reproduce today's rendering.
- **Dash pattern** for kurbo: `[dash, gap]`. When `dash = 0` with round caps (Dotted), kurbo emits zero-length dashes; stroking a zero-length segment with round caps is verified to give a dot. If it doesn't, those dashes are replaced by circles of the stroke width.

### D2. `tp-core::document::stroke_region`

```rust
pub fn stroke_region(center: &BezPath, fill: Option<&BezPath>, width: f64, align: StrokeAlign, line: &LineStyle, tolerance: f64) -> Option<BezPath>
```

1. **Dash:** `kurbo::dash(center, 0.0, &[dash, gap])` when dashed.
2. **Expand:** `kurbo::stroke(path, &Stroke { width: w', join, cap, miter_limit }, opts, tolerance)`. The expansion width `w'` is `width` for Center and `2·width` for Inside and Outside.
3. **Clip** (shapes and texts only):
   - Inside: region ∩ fill;
   - Outside: region − fill.

   This uses `i_overlay` (new dependency 9.0.0, MIT/Apache-2.0), with both operands flattened at `tolerance` and the non-zero fill rule for both.
4. The result is a path filled with the non-zero rule.

The **fast path:** Center alignment with default joins and no dash returns `None`, so callers keep their current drawing (egui `closed_line`, lyon or tiny-skia strokes). This keeps the defaults pixel-identical and avoids a cost increase for unchanged documents.

Why `i_overlay` rather than `flo_curves` here: this region is only drawn, never edited, so robustness and speed matter more than keeping curves. Stroke outlines from kurbo contain many small cubics, and polygon booleans on flattened input are the robust choice. Flattening uses the drawing tolerance (the zoom bucket on the canvas, 0.05 px in the export), so it's invisible.

### D3. Lines

The casing technique stays, with widths adapted to the alignment:

| Alignment | Casing (stroke color) | Body (fill color) |
|---|---|---|
| Center | `W + s` | `W − s` |
| Outside | `W + 2s` | `W` |
| Inside | `W` | `W − 2s` |

Both are stroked with the line's `LineStyle` (dash, cap, join), so the outline follows the dashes. A line without a stroke draws its body with its `LineStyle`. The dash pattern applies to the center line, so the casing and the body dashes coincide.

### D4. Canvas and export integration

- `GeometryCache` keys gain the stroke style and the line style. When `stroke_region` returns a region, it's tessellated with lyon `fill` (non-zero) into the stroke mesh. Rectangles and ellipses with a non-default stroke move from `closed_line` to that mesh.
- Text: `TextEngine::mesh` computes the stroke from `stroke_region(glyph outline, Some(glyph outline), …)` when the style isn't default.
- `tp-render`: when `stroke_region` returns a region, it's filled with the stroke color instead of `stroke_path`. The default path is unchanged.

### D5. Persistence

`FileStroke` gains optional fields (`align`, `dash`, `cap`, `join`, `miter_limit`), and `FileObject` gains optional `line_style`, all with `#[serde(default)]` matching the defaults. Format 2 is unreleased and the change is additive, so there's no version bump (same reasoning as for guides). The v2 fixture is regenerated with a dashed outside stroke and a dotted line.

### D6. UI

- **Stroke panel** (for shapes and texts):
  - a segmented control for alignment (Center, Inside, Outside, with icons and labels);
  - a Dashed checkbox with Dash and Gap fields and a preset combo box;
  - segmented controls for cap and join;
  - a Miter limit field, shown when the join is Miter.
- **Properties, next to Width** (lines only): the same dash, cap and join controls bound to `PathData.line_style`, extracted into a shared `line_style_controls(ui, value: Mixed<LineStyle>) -> Option<Change>` widget so the two panels stay identical.
- **Workspace:** remembers the line style for new lines (`ws.line_style`), like `line_width`.

## Risks / Trade-offs

- [`kurbo::stroke` output on degenerate input (zero-length, cusps)] → Tests for zero-length dashes, sharp corners and a 1 px square. An empty or failed region falls back to the centered stroke.
- [Performance of Inside/Outside on texts with many glyphs] → The region is cached per object, zoom bucket and style. The release timing check repeats the 50-text panning benchmark with outside strokes.
- [Canvas/export mismatch] → Both use `stroke_region` with their own tolerance. A pixel test compares the export of an outside-stroked text against expectations (covered pixels outside the letters only).
