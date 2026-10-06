## 1. Document model (tp-core)

- [x] 1.1 Add `ShapeKind::Polygon { sides, star }` and `ShapeKind::Path`, plus `PathData` / `Subpath` / `Node` and `Object.path`. Default names are "Polygon", "Path" and "Line". Add `Object::refit()` (D2). Verify with unit tests that `refit` centers the geometry, that the frame equals the exact curve bounds (including curve extrema), and that the minimum size is respected for a horizontal line.
- [x] 1.2 Implement polygon and star geometry (D4) and `local_path()` for polygons and paths. For paths, the fill path keeps only closed subpaths and the stroke path keeps all subpaths. Verify with tests that a triangle fills its 300×200 frame, that a hexagon has its first vertex at the top center, that a 5-point star has 10 vertices, and that the regular aspect helper keeps sides equal.
- [x] 1.3 Implement hit testing and marquee intersection for paths and polygons (D6). Verify tests for:
  - a star notch not hit;
  - a hole not hit;
  - an open "V" hit on its stroke but not between its arms;
  - a thin line hit within the tolerance;
  - a marquee touching a concave shape.
- [x] 1.4 Make the transforms exact for paths and polygons (D3): unfolded rotation and node transform for paths, up-axis rotation for polygons. Verify tests for:
  - an arrow path flipped horizontally points left;
  - a triangle flipped vertically points down;
  - a multi-selection resize of a rotated path matches the transformed points;
  - every existing transform test passes unchanged.
- [x] 1.5 Add the path operations module (D9): `move_nodes`, `set_handle` (aligned or broken), `toggle_smooth`, `insert_at`, `delete_nodes` with the survival rules, and `nearest_segment`. Verify tests for:
  - inserting on a curve keeps the flattened outline identical within 0.01 px;
  - deleting a corner of a closed square gives a triangle;
  - the subpath and object removal rules;
  - aligned handles keep their own length.
- [x] 1.6 Add Convert to Path for rectangles (with and without radius), ellipses and polygons (D10), keeping identity, name and style. Verify tests that the converted outline matches the original within 0.05 % of its size and that the id is unchanged.
- [x] 1.7 Add the point selection to `Snapshot`, ignored by `same_document`. Verify a test that `same_document` ignores the point selection and that restoring a snapshot returns it.

## 2. File format (tp-file)

- [x] 2.1 Freeze `v1` as read-only and add the `v2` module (D11), with `FORMAT_VERSION = 2`, conversion from `v1` and `migrate(1)`. Verify that the existing format tests pass, that the `v1` fixture opens with `migrated = true`, and that a round trip of a project with a star, a curved path with a hole and an open line is identical.
- [x] 2.2 Commit the `tests/fixtures/v2.truckpaint` fixture and add a test that opens it. Verify that a file declaring format 3 is refused as `NewerVersion`.

## 3. Rendering

- [x] 3.1 In `tp-render`, draw paths (closed subpaths filled, every subpath stroked with round caps and MiterClip) and polygons (D12). Verify pixel tests for:
  - an open "V" with no fill between its arms;
  - a hole showing the background;
  - a star notch transparent;
  - a mirrored path.
- [x] 3.2 Extend the canvas `GeometryCache` with lyon fill and stroke meshes for paths and polygons (D5), and paint them with `mesh_to_screen`. Rectangles and ellipses keep the convex path. Verify that the existing screenshot tests are unchanged and that a new cache test rebuilds the mesh when `path` changes.

## 4. Drawing tools (tp-app)

- [x] 4.1 Add the Polygon tool: `Gesture::Drawing` with a polygon kind, the Shift aspect from D4, Alt draws from the center, and `Workspace::polygon_style`. Verify the kittests "Default hexagon" and "Shift keeps it regular", and that a click without a drag creates nothing.
- [x] 4.2 Add the Line tool: a two-node path with Shift at 45° and Alt from the middle, and the default stroke for open paths (8 px in the fill color when there is no stroke). Verify the kittests "Horizontal line with Shift", "Line from the middle", "Line without a current stroke" and "Line uses the current stroke".
- [x] 4.3 Add the Pen session (D7):
  - click adds a corner point and dragging adds a smooth point;
  - the preview shows the next segment;
  - Shift constrains points and handles to 45°;
  - clicking the first point closes the path;
  - Enter, Escape, a tool change or a command finishes the path;
  - Backspace or Undo pops the last point;
  - a path with fewer than 2 points is discarded;
  - the finished path is one undo step.

  Verify the kittests for every scenario of "Pen tool point placement" and "Finishing, closing and cancelling a pen path".
- [x] 4.4 Remove the "not available yet" hint and add the cursors (crosshair for Polygon and Line, crosshair plus the Pen icon for Pen, the arrow for Direct Selection). Verify the kittest "Pen tool is available" (no hint is shown) and that no tool shows the hint any more.

## 5. Direct Selection and Convert to Path (tp-app)

- [x] 5.1 Add Direct Selection object picking (the innermost object, ignoring groups) and the point overlay: points, handles, and the hint for non-path shapes. Add the point-overlay sizes as `tp_ui` tokens. Verify the kittests "Points of a path inside a group" and "Rectangle hint".
- [x] 5.2 Add point selection (click, Shift+click, point marquee, Escape clearing first the points and then the objects; no history entries). Verify the kittests "Marquee selects points" and "Shift+click toggles a point".
- [x] 5.3 Add point and handle dragging (with Shift constraint, aligned smooth handles, Alt to break) and arrow nudges coalesced into one step. Verify the kittests "Move one point", "Smooth handle stays aligned", "Alt breaks the handle" and "Undo a point drag".
- [x] 5.4 Make double-click on a point toggle corner/smooth, double-click on a segment insert a point, and Delete remove the selected points (Delete removes objects when no point is selected). Verify the kittests "Insert keeps the shape", "Delete a point of a square" and "Corner to smooth".
- [x] 5.5 Add the `ConvertToPath` command in the Object menu, enabled by `selection_has_convertible`. Verify the kittests "Convert a rounded rectangle", "Convert inside a group" and "Disabled without convertible shapes", and the "Undo a point edit" scenario.

## 6. Properties panel

- [x] 6.1 Add the Polygon section (Sides 3–12, Star toggle, Inner radius 10–90 %, "Mixed" for differing values, one undo step per change, values remembered for new polygons). Verify the kittests "Make a five-point star" and "Sides are clamped", and that the next polygon drawn uses the last values.

## 7. Integration

- [x] 7.1 Add screenshot tests `render_vector_tools`: the pen preview while drawing, Direct Selection points and handles on a curved path, and stars and paths with holes on the canvas. Verify by reviewing the images that they match the export render.
- [x] 7.2 Run the save, reopen, export and recovery round trip on a project with paths in a kittest. Measure release timings (50 stars plus 50 paths of 40 nodes while panning; export at 4K and 8K) and record them in the PR description.
- [x] 7.3 Run `mise run checks` and the full test suite (including 4 parallel stress runs of the new kittest binaries). Verify that every check passes and that no `settle()` or run-step flakiness appears.

## 8. Lines drawn with the fill color (review feedback)

- [x] 8.1 Add `line_width` to `PathData` (default 8 px, kept by resizes) and to the v2 file format; regenerate the v2 fixture (format 2 is not released yet). Verify the format round trip and fixture tests with a 30 px line.
- [x] 8.2 Draw open subpaths as lines in the fill color at their width, outlined by the stroke (casing technique, D14), on the canvas meshes and in `tp-render`; hit test the line width. Verify pixel tests "Open path is a line" and "Outlined line", the hit test of a wide line, and the screenshots.
- [x] 8.3 New open paths use the current fill and stroke (no forced stroke) and `Workspace::line_width`; add the Width field (accessible name "Line width") to Properties for paths with open subpaths. Verify the kittests "Default line", "Line uses the current stroke as an outline", "Thicker line" and "Next line uses the last width".
- [x] 8.4 Run `mise run checks`, the full test suite and the stress run again. Verify everything passes.

