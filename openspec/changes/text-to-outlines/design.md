## Context

- **Text rendering:** texts are drawn from glyph outlines. `tp_text::layout()` places glyphs (`PlacedGlyph` with font, glyph id, size, position and character byte range). `GlyphCache::outline()` concatenates every glyph's `BezPath`, in layout coordinates, and `layout_to_doc(object)` maps layout space to the document (frame, rotation, text scale).
- **Glyph contours:** they come from skrifa as move/line/quad/cubic/close commands. TrueType fonts use quadratic curves.
- **Paths:** `PathData` (`tp-core`) stores cubic nodes in frame-local space, and `Object::from_path` fits an unrotated frame.
- **Convert to Path** already converts shapes in place, keeping their ids.

## Goals / Non-Goals

**Goals:**
- Pixel-identical result.
- Every letter reshapeable.
- One undo step.

**Non-Goals:**
- Merging letters that touch.
- Keeping kerning information.
- Outlining part of a text.

## Decisions

### D1. Per-glyph outlines from the text engine

`GlyphCache::glyph_outlines(fonts, layout) -> Vec<GlyphOutline { range, path }>` reuses the cached glyph paths and the placement transform of `outline()`, without concatenating them, and skips empty paths (spaces).

The outlines come from the same `TextEngine` layout used for drawing (same fonts and fallback), so what you see is what you get.

Alternative considered: re-tessellating from the canvas mesh. Rejected because it would lose the curves.

### D2. `PathData::from_bezpath`

Walks the elements:
- `MoveTo` starts a subpath;
- `LineTo` adds a corner node;
- `QuadTo(c, p)` becomes a cubic, with handles `p0 + 2/3 (c − p0)` and `p + 2/3 (c − p)`, set as the previous node's `handle_out` and the new node's `handle_in`;
- `CurveTo` sets both handles;
- `ClosePath` marks the subpath closed.

When the last node of a closed subpath repeats the first one (within 1e-9), it's merged: the first node takes its incoming handle. A node is marked smooth when both handles exist and are collinear with opposite directions (cross product within tolerance).

The conversion is exact: the flattened outline is identical within 1e-6.

### D3. Letters keep the text's rotation

For each glyph:
1. The document-space path is `layout_to_doc(text) * glyph.path`.
2. The letter is built in the text's rotation, the same way `create_line` builds a rotated line: the nodes are expressed in a frame rotated by the text's rotation, and `refit` centers that frame.
3. The text's non-uniform scale is baked into the nodes, exactly.

So the letters' rotation in the Transform panel matches the text's.

### D4. Replacing the text in place

`Project::replace_with_group(id, children) -> Option<ObjectId>`:
1. Builds a group with the text's id, name, visibility, lock and opacity, and the letters as children (each with fill and stroke from the text, opacity 1).
2. Assigns fresh ids to the children.
3. Swaps it into the tree at the text's path, then refreshes the ancestors' frames.

The selection maps each outlined text id to its group, which has the same id, so the selection is unchanged.

**Texts in selected groups:** `ws.selected_objects()` is walked with `shapes()` to collect the text ids. Each one is replaced individually, inside a single `ws.edit("Create Outlines")`.

### D5. Command

- `CommandId::CreateOutlines`: ⌘⇧O, Object menu after Convert to Path.
- Enabled by `EditContext.selection_has_text` (any text in the selected subtrees).
- Dispatch first ends a text editing session (the existing rule for commands that don't keep it), then converts.
- ⌘⇧O is free (⌘O is Open; the registry's conflict test confirms it).
