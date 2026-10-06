## 1. Building blocks

- [x] 1.1 Add `PathData::from_bezpath` in `tp-core` (exact quad→cubic, closing-point merge, smooth detection). Verify unit tests:
  - a TrueType-like quad contour flattens identically to the source path;
  - a closed square has 4 nodes;
  - a smooth join is detected;
  - an open path stays open.
- [x] 1.2 Add `GlyphCache::glyph_outlines` in `tp-text`, returning per-glyph paths with their character range and skipping spaces. Verify unit tests:
  - "A C" yields 2 outlines, for "A" and "C";
  - "O" has 2 contours;
  - the union of the outlines equals `outline()`.
- [x] 1.3 Add `Project::replace_with_group` in `tp-core` (same id, fresh child ids, same position and parent, ancestor frames refreshed). Verify unit tests: a text inside a group is replaced at its index, ids stay unique, and undo through snapshots restores it.

## 2. Command

- [x] 2.1 Implement the conversion in `tp-app`:
  - letters in the text's rotation (D3), with the text's fill and stroke;
  - the group named after the first line (shortened to 32 characters);
  - texts in selected groups converted in place, texts without letters skipped;
  - one undo step.

  Verify unit tests "Outline a word" (names, order, selection), "Only spaces", and a rotated, stretched text whose letter rotation equals the text's.
- [x] 2.2 Add the `CreateOutlines` command (⌘⇧O, Object menu, `selection_has_text`, ends a text editing session first). Verify kittests: "Disabled without texts", "Undo restores the text", converting from the menu while editing a text, and the shortcut conflict test.

## 3. Fidelity

- [x] 3.1 Add a pixel test with `tp-render`: a rotated, stretched, stroked text and its outlines export the same image (identical: 0 pixels differ). Without a stroke, only a few anti-aliased edge pixels may differ by at most a quarter of their coverage: the font's quadratic curves become exact cubics (verified point for point within 0.001 px), and the renderer flattens the two curve types slightly differently. Verify "Same pixels on export" and "Counters stay holes" (the inside of an "O" is the background).
- [x] 3.2 Add a kittest "Reshape a letter": Direct Selection on an outlined letter shows its points, and moving one changes only that letter. Also verify a save and reopen round trip of an outlined text.

## 4. Integration

- [x] 4.1 Run `mise run checks`, the full test suite and the stress run (4 parallel × 5 rounds of the text_images, vector_tools, panels and new outline binaries). Verify everything passes.
