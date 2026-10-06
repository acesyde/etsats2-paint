## Why

Livery lettering is often customized beyond what a font gives: a stretched "S", a letter cut by a stripe, a team name with a swoosh tail. Texts can be resized and styled, but their letters can't be reshaped. Turning a text into editable paths unlocks the Pen, Direct Selection and Convert to Path tools on lettering. It also removes the dependency on the font: a project using a font installed on one computer looks the same everywhere once its texts are outlined.

## What Changes

- **Object › Create Outlines** (⌘⇧O / Ctrl+Shift+O) turns each selected text, including texts inside selected groups, into a **group of letter paths**:
  - one path per letter, holes included (the counters of A, O, R);
  - the group is named after the text and takes its place in the stacking order;
  - the group keeps the text's identity, visibility, lock and opacity.
- **What's kept:**
  - letters have the text's fill and stroke, and look exactly like the text did: same glyphs, size, spacing, alignment, line breaks, rotation and scale;
  - with a missing font, the outlines use the fallback font that was being shown.
- **Behavior:**
  - texts without visible letters (only spaces) are left unchanged;
  - the conversion is one undo step;
  - a text being edited is committed first.
- **After conversion**, letters are ordinary paths: editable with Direct Selection, and saved and exported like any path.

Not in this change: outlining a single selected range of characters, converting outlined letters back into text, keeping a hidden copy of the original text.

## Capabilities

### New Capabilities

- `text-outlines`: the Create Outlines command (where it applies, the resulting group and letter paths, appearance and identity, undo).

### Modified Capabilities

(none: Object menu items are listed by the command system, and paths and groups are already specified)

## Impact

- **`tp-text`:** `GlyphCache::glyph_outlines(fonts, layout)` returns the outline of each placed glyph with its character range, so each letter can become its own path. Empty glyphs (spaces) are skipped.
- **`tp-core`:**
  - `PathData::from_bezpath` converts a kurbo path into subpaths. It raises quadratic curves to cubics exactly and merges a closing point that repeats the first.
  - `Project::replace_with_group(id, children)` replaces an object with a group that keeps its id, with fresh ids for the children.
- **`tp-app`:**
  - the `CreateOutlines` command (⌘⇧O), enabled when the selection holds texts;
  - the Object menu entry;
  - the conversion, which uses the text engine's layout.
