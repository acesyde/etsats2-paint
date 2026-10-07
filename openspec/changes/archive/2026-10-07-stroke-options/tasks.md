## 1. Model and geometry (tp-core)

- [x] 1.1 Add `LineStyle`, `Dash`, `Cap`, `Join`, `StrokeAlign` and the new `StrokeStyle` fields, plus `PathData.line_style`, with defaults equal to today's rendering. Update every `StrokeStyle` construction. Verify that the existing tests pass unchanged (the defaults render as before).
- [x] 1.2 Add `i_overlay` and `stroke_region` (dash, expansion, Inside/Outside clipping, fast path) (D2). Verify unit tests:
  - Inside on a 400 px square stays within the square and covers its outer 30 px band;
  - Outside around an "O"-like shape (outer contour plus counter) covers both edges and never the filled area;
  - a dotted pattern gives separate dots (zero-length dashes with round caps);
  - Square, Butt and Round caps extend 0, w/2 and w/2 respectively;
  - Bevel and Round joins differ from Miter at a sharp corner;
  - degenerate inputs fall back safely.

## 2. Rendering

- [x] 2.1 Make `tp-render` fill stroke regions for non-default strokes and adapt the line casing to alignment and line style (D3). Verify pixel tests for:
  - an inside border;
  - an outside outline around a text (no pixel of the stroke color inside the letters);
  - a dashed pinstripe line without stroke;
  - dotted dots;
  - square caps;
  - existing render tests unchanged.
- [x] 2.2 Canvas: geometry cache keys include the styles; non-default strokes of shapes, paths and texts use region meshes; lines get the adapted casing. Verify that the existing screenshot and canvas tests are unchanged, and a cache test where changing the alignment rebuilds the mesh.

## 3. Persistence

- [x] 3.1 Add the optional `FileStroke` fields and `line_style` (D5) and regenerate the v2 fixture. Verify:
  - a round trip of every option;
  - a v2 document without the new fields opens with the defaults ("Old projects unchanged");
  - the fixture test.

## 4. UI

- [x] 4.1 Stroke panel: alignment, Dashed with Dash/Gap and presets, cap, join and miter limit (for shapes and texts; the current style with nothing selected; "Mixed"; one undo step each). Verify kittests "Outside outline in one click", "Dotted preset" and "Round corners".
- [x] 4.2 Properties line section: the shared line-style controls next to Width, and the remembered line style for new lines. Verify kittests "Dashed line from the Properties panel" and "Butt ends", and that the next line uses the last values.

## 5. Integration

- [x] 5.1 Add the `render_stroke_options` screenshots (outside-outlined lettering, a dashed pinstripe, inside border, both panels) at 100 % and 200 % UI scale, and review them. Re-run the 50-text panning benchmark with outside strokes, in release, and verify it stays under 16 ms per frame.
- [x] 5.2 Run `mise run checks`, the full test suite and the stress run (4 parallel × 5 rounds of the panels, text_images, vector_tools and new stroke binaries). Verify everything passes.
