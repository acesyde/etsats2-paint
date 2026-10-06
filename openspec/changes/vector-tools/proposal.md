## Why

Liveries are built from custom shapes: stripes, swooshes, flames, silhouettes, chevrons. Today TruckPaint can only draw rectangles and ellipses, and four tools (Polygon, Pen, Line, Direct Selection) are still marked "not available yet". Without freeform vector shapes users have to make their artwork in another program and import it as an SVG, and then they can't edit it in TruckPaint. Vector paths are the biggest missing piece of the editor. They don't depend on anything external, so they come next.

## What Changes

- **Path objects**: a new kind of vector object made of one or more subpaths of anchor points with optional Bézier handles. Each subpath is open or closed.
  - Closed paths are filled and stroked.
  - Open paths are lines: they're stroked only and never filled. If no stroke is set when an open path is created, it gets a default stroke in the current fill color so it is visible.
  - Paths stay fully vector. They behave like every other object: moving, resizing, rotating, flipping with handles, grouping, opacity, visibility, lock, undo, layers, export and hit testing on the actual shape.
- **Pen tool (P)**:
  - Click to add corner points, or drag to add smooth points with symmetric handles.
  - Click the first point to close the path; press Enter or Escape (or switch tool) to finish an open path.
  - Backspace removes the last point while drawing; removing every point cancels the path. Shift constrains segments and handles to 45° steps.
  - A live preview shows the next segment under the pointer.
- **Line tool (\\)**: drag to draw a straight line, an open path with two points. Shift constrains it to 45° steps and Alt/Option draws from the center.
- **Polygon tool (Y)**:
  - Drag to draw a regular polygon (6 sides by default). Shift keeps it regular and Alt/Option draws from the center.
  - Polygons stay editable shapes: the Properties panel shows the sides (3–12), a Star toggle and the star's inner radius. New polygons use the last values set.
- **Direct Selection tool (A)**: shows the anchor points and handles of the selected paths.
  - Click or Shift+click selects points; dragging from empty space draws a point marquee.
  - Dragging moves points and handles. Arrow keys nudge the selected points.
  - Double-clicking a point toggles it between corner and smooth. Alt-dragging a handle breaks the symmetry.
  - Double-clicking a segment inserts a point, and Delete removes the selected points.
  - Every edit is one undo step.
- **Object › Convert to Path**: turns rectangles, ellipses and polygons (also inside groups) into editable paths that look identical, keeping fill, stroke, opacity, name and identity.
- **Canvas and export rendering of concave shapes**: paths, polygons and stars render correctly on the canvas and in exported textures, including concave shapes, multiple subpaths and holes (non-zero winding).
- **Project file format version 2**: adds polygons and paths. Version 1 files still open (with no changes needed); like any older format, they open marked as having unsaved changes. A version 1 TruckPaint refuses version 2 files with the "newer version" message.

Not in this change:

- grid, guides and snapping (the `precision-aids` change);
- boolean operations (union, subtract);
- text to outlines;
- a pencil or freehand brush;
- path offset and outline stroke;
- editing points across several objects at once with transforms, such as scaling the selected points;
- dash patterns, caps and joins in the Stroke panel.

## Capabilities

### New Capabilities

- `path-tools`: the Pen, Line and Polygon tools. Covers creating paths point by point, finishing, closing and cancelling, modifier constraints, the defaults for new paths and polygons, and the polygon and star settings in the Properties panel.
- `path-editing`: the Direct Selection tool and Convert to Path. Covers point and handle selection, moving and nudging, corner/smooth conversion, inserting and deleting points, and undo.

### Modified Capabilities

- `document-model`:
  - adds path and polygon objects (geometry, open versus closed subpaths, default names "Path", "Line" and "Polygon");
  - hit testing for paths: the filled area for closed paths, the stroke with a tolerance for open paths;
  - non-zero fill rule.
- `shape-tools`: the "Tool feedback" requirement (unavailable-tool hint) is replaced by "Tool cursors": every tool works, with a crosshair for Polygon and Line, a pen cursor for Pen and the default arrow for Direct Selection.
- `project-files`: a version-2 format containing paths and polygons. Opening a version-1 file migrates it.
- `texture-export`: exported textures render paths, open strokes and polygons exactly like the canvas.
- `undo-history`: path point edits, polygon settings and Convert to Path are undoable, and undo restores the selected points.

## Impact

- **`tp-core`**:
  - `ShapeKind::Polygon { sides, star }` and `ShapeKind::Path`;
  - path geometry (`PathData` with subpaths and nodes) held by `Object`, stored in local coordinates so resizes, flips and rotations stay exact;
  - updated `path()`, `bounding_box`, `contains` and marquee intersection for concave and open shapes;
  - resize applies the exact transform to path nodes;
  - conversion of shapes to paths, and node-editing operations (move, insert at a segment parameter, delete, toggle smooth).
- **`tp-app`**:
  - Pen, Line, Polygon and Direct Selection gestures and their previews;
  - the point overlay;
  - the Convert to Path command;
  - the Polygon section in Properties;
  - canvas filling through the lyon tessellation (already used for texts) instead of convex polygons, with geometry cached per object and zoom.
- **`tp-render`**: draws paths and polygons with tiny-skia, filling closed subpaths only and stroking with round joins and caps on open ends.
- **`tp-file`**: new `v2` module and `FORMAT_VERSION = 2`; v1 is frozen and migrated by `v1 → v2`; a committed `v2.truckpaint` fixture.
- **Dependencies**: none new. kurbo and lyon are already in the workspace.
