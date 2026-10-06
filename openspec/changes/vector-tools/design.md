## Context

Today's object model:

- `Object` is a `Copy` `ShapeKind` plus a `Frame` (center, size, rotation). Kind-specific data lives in side fields (`children`, `text`).
- Shapes are described by `local_path()` in frame-local space (centered, unrotated) and mapped by `frame.affine()`.

Rendering, hit testing and selection rely on the shapes being convex:

- the canvas fills objects with `Shape::convex_polygon` from flattened outlines cached per zoom bucket in `GeometryCache`;
- hit testing uses `BezPath::contains`;
- marquee selection uses a convex SAT test (`convex_polygons_overlap`).

Texts already go through lyon tessellation (`tp_text::mesh::{fill, stroke}`, NonZero) and are drawn as egui meshes.

Transforms (`document/transform.rs`):

- they work on frames only;
- `resize_one` folds the new rotation into (−90°, 90°] because every current shape is symmetric, so a flip becomes a size change.

The file format is versioned: `tp-file` reads a header, `v1` is frozen and `migrate()` holds a chain that is still empty.

Snapshots (undo) hold the surfaces and the object selection.

## Goals / Non-Goals

**Goals:**

- Paths and polygons must be first-class objects: every existing feature (groups, layers, transforms, Transform panel, opacity, copy/paste, export, save, recovery) must work on them without special cases outside geometry.
- Transforms must be exact for paths (no drift, flips mirror).
- Rendering on the canvas and in exports must match for concave and multi-subpath shapes.

**Non-Goals:**

- A general node-transform tool (scaling or rotating a set of points).
- Joining or splitting subpaths.
- Continuing an existing open path with the Pen.
- Snapping. That belongs to the `precision-aids` change, which will hook into the same pointer pipeline.

## Decisions

### D1. Data model: marker kind plus side data, nodes in frame-local space

- `ShapeKind::Path` (a unit variant, keeping `ShapeKind: Copy`) with `Object.path: Option<Arc<PathData>>`, following the `text` pattern.
- `PathData { subpaths: Vec<Subpath> }`, where `Subpath { nodes: Vec<Node>, closed: bool }` and `Node { point, handle_in: Option<Point>, handle_out: Option<Point>, smooth: bool }`.
- Positions are absolute in frame-local coordinates (origin at the frame center, unrotated).
- `smooth` is stored rather than derived from handle alignment. Derived smoothness flips on its own when handles happen to line up, and the handle-drag behavior has to be predictable.

Alternatives considered:

- Document-space nodes: rejected, because the frame rotation, which the Transform panel and the rotated selection overlay need, would be lost.
- Nodes normalized to the frame (−0.5..0.5): rejected. A horizontal line has height 0, which is clamped to `MIN_SIZE`, so the normalization becomes degenerate. It also cannot express mirroring separately from rotation.

`ShapeKind::Polygon { sides: u8, star: Option<f64> }` is parametric: `star` holds the inner radius ratio 0.1–0.9. Like a rectangle, it is drawn from its frame.

### D2. Frame invariant for paths: `refit()`

After every edit of the path data:

1. compute the exact bounds of the local `BezPath` (kurbo bounding box, curve extrema included);
2. translate the nodes so that the bounds are centered on the origin;
3. move `frame.center` by the same offset, rotated by the frame rotation;
4. set `frame.size` to the bounds size, clamped to `MIN_SIZE`.

`frame.size` is therefore always derived and never scales the path implicitly. Unlike texts, there is no hidden scale factor: resizes are applied to the nodes themselves (see D3).

### D3. Exact transforms

- **Translate**: moves the center only.
- **Rotate**: changes the frame only; the nodes are unchanged.
- **Resize**: `resize_one` computes the full document transform `T` as it does today. For a path, the new frame (center and folded rotation) is chosen with the existing formula, exactly as for other shapes; the new nodes are then `new_frame.affine()⁻¹ · T · old_frame.affine()` applied to every point and handle, followed by `refit()`. The nodes absorb whatever the frame does not express (mirror, skew), so flips mirror while the rotation shown in the Transform panel stays the same as for a rectangle (0° after a horizontal flip). A multi-selection resize of rotated objects introduces skew: it is approximated for rectangles today, but it is exact for paths.
- **Polygons**: a polygon is mirror-symmetric about its local vertical axis. Its new rotation is derived from where `T` sends the local "up" axis, so a vertical flip turns it upside down; a horizontal flip changes nothing, by symmetry.

The Transform panel's W/H edits go through `resize`, so they get exactness for free.

### D4. Polygon geometry

- The unit vertices sit on a circle (alternating outer and inner radius for stars), starting at −90°.
- Their bounding box is mapped onto `frame.local_rect()`, so the polygon's bounds equal its frame. This keeps the selection handles hugging the shape and makes "bounds match the dragged box" true.
- Shift (regular) constrains the drawing box to the aspect ratio of the unit bounding box, rather than to a square.

### D5. Canvas rendering through lyon

The `GeometryCache` entry stores, per object and zoom bucket:

- a **fill mesh**: closed subpaths only, NonZero;
- a **stroke mesh**: all subpaths, miter join with limit 4 to match the export, round caps;
- the flattened outline, kept for hover and selection outlines.

Meshes are in document coordinates and mapped to the screen at paint time, as texts already are (`mesh_to_screen`).

Which kinds use which path:

- paths and polygons (stars are concave) always use the meshes;
- rectangles and ellipses keep the convex fast path, so this change does not regress their performance or screenshots.

The cache key adds a pointer check of `path`; edits replace the `Arc`.

Alternative considered: egui's `PathShape` with its own concave fill. Rejected because egui only fills convex polygons correctly.

### D6. Hit testing and marquee

`Object::contains(point, tol)`:

- **Closed paths and polygons**: kurbo `contains`, which uses non-zero winding. The point is also accepted when it lies within `stroke_width/2 + tol` of the outline (nearest point on each segment, via `ParamCurveNearest`).
- **Open paths**: only the distance test, with the tolerance at least the screen hit tolerance.

Marquee selection replaces the convex SAT test for these kinds with a general test on the flattened outline (closed subpaths filled):

1. any outline vertex is inside the rectangle;
2. any rectangle corner is inside the filled shape;
3. any outline edge crosses a rectangle edge.

`convex_polygons_overlap` stays for rectangles and ellipses.

### D7. Pen session outside `Gesture`

A pen path spans many presses, so it lives in `Workspace::pen: Option<PenSession>` (nodes so far, plus the drag state for the current handle), like `TextSession`.

- `Gesture::PenHandle` covers only the press-drag of a smooth point.
- The session ends through one `finish_pen()` called from the same hook points that end a text session: tool change, command dispatch, Enter or Escape, and a click on the first node. Undo and Backspace are intercepted while a session exists and pop the last node (spec).
- Finishing builds the object (`refit`), inserts it into the active layer with `ws.apply`, as the shape tools do, and creates one history step.

The Line tool reuses `Gesture::Drawing` with `ShapeKind::Path` and builds a two-node path. Polygon reuses `Gesture::Drawing` with `drawing_frame` (plus the D4 aspect constraint for Shift).

### D8. Point selection state

- `Workspace::points: BTreeSet<PointRef>`, where `PointRef { object: ObjectId, subpath: u32, node: u32 }`.
- It is pruned whenever the object selection changes, or when an edit renumbers nodes (insertion and deletion return the remapped set).
- `Snapshot` gains the point selection so that Undo and Redo restore it (spec); `same_document` ignores it, as it ignores the selection.

Direct Selection gestures:

- `Gesture::MovingPoints { before, originals, start }` and `Gesture::MovingHandle { node, which, break_symmetry }` both start from originals, so live updates never accumulate (the same pattern as `Transforming`).
- `Gesture::PointMarquee`.

Nudges reuse the existing nudge-coalescing into one step.

### D9. Path operations in `tp-core::document::path`

These are pure functions on `PathData` (in local space) plus a `refit` wrapper on `Object`:

- `move_nodes`
- `set_handle(aligned | broken)`
- `toggle_smooth`: new handles along `(next − prev)`, each one third of the adjacent segment length
- `insert_at(subpath, segment, t)`: de Casteljau via kurbo `CubicBez::subdivide`; a straight segment stays straight
- `delete_nodes`, which applies the spec's survival rules
- `nearest_segment(point)`

The canvas converts pointer positions to local space with `frame.affine().inverse()`.

### D10. Convert to Path

| Shape | Result |
|---|---|
| Rectangle | 4 corner nodes; with a radius, 8 nodes joined by quarter-circle cubics (κ = 0.5523) |
| Ellipse | 4 smooth nodes with κ handles (radial error below 0.03%) |
| Polygon | its vertices as corner nodes |

Kind, `path` and name handling are done in place, so `id`, style and stacking order are kept. The default name of the kind ("Rectangle"…) is kept as is: renaming would surprise users who named their layers.

kurbo's own `Ellipse::to_path` is not used, because it emits more segments than needed depending on the tolerance and would yield extra nodes.

### D11. File format v2

- New module `v2`: a copy of `v1` plus `FileKind::Polygon { sides, star }`, `FileKind::Path`, and `FileObject.path: Option<Vec<FileSubpath>>` (`#[serde(default)]`), where a `FileNode` has `p`, `hin`, `hout` and `smooth`.
- `FORMAT_VERSION = 2`; writing always uses v2.
- `v1` becomes read-only, frozen, and keeps its DTOs. `migrate(1)` parses v1, converts it with `From<v1::FileProject> for v2::FileProject` (field-for-field) and reports `migrated = true`.
- New fixture `tests/fixtures/v2.truckpaint`, holding a star, a curved path with a hole and an open line. The `v1` fixture test now also asserts `migrated`.

### D12. Export rendering

- `tp-render` builds two kurbo paths per path object: the closed subpaths (fill) and all subpaths (stroke).
- The stroke is drawn with `LineCap::Round`, which only affects open ends, and `MiterClip` with limit 4.
- Polygons go through the same `object.path()` as the other shapes.
- Pixel tests cover: an open "V" that isn't filled, a hole, a star notch, and a mirrored path.

### D13. UI details

- **Properties**: a Polygon section (Sides drag value 3–12, Star toggle, Inner radius %) built like the corner radius one, with "Mixed" for differing values.
- **New-polygon settings**: `Workspace::polygon_style`, not part of history, like the current style.
- **Convert to Path**: a new `Command::ConvertToPath` in the Object menu, enabled by a context flag `selection_has_convertible`.
- **Pen cursor**: egui has no pen cursor, so the canvas shows `CursorIcon::Crosshair` with the Pen icon painted at the bottom-right of the pointer. Direct Selection uses the default arrow.
- **Overlay**: square points, about 7 px; selected points filled with the accent color; handles as 1 px lines ending in 5 px dots; hover enlargement. Sizes go in `tp_ui::tokens::canvas`.
- **Hints**: the "not available yet" hint and its branches are removed. "Convert to Path to edit points" uses the existing `show_hint`.

## Risks / Trade-offs

- [Tessellation cost on the canvas for large paths and many stars] → Meshes are cached per object and zoom bucket and only rebuilt when the object's `Arc` changes. The existing release timing check (50 objects while panning) is repeated with 50 stars and 50 paths of 40 nodes each.
- [Polygons flipped vertically show 180° in the Transform panel] → Expected: it is their real orientation (pointing down). It is covered by a test.
- [A broad change to `resize_one` could regress existing shapes] → The path and polygon branches are separate; the existing transform tests must pass unchanged.
- [Opening v1 projects now marks them unsaved] → This is what the spec requires for older formats. The format bump is needed so that an older TruckPaint refuses newer files cleanly instead of failing on unknown variants.
- [Pointer precision while editing handles at low zoom] → Hit radii are in screen pixels; points take priority over handles, which take priority over segments.

## Migration Plan

Opening a v1 file migrates it in memory; nothing is rewritten until the user saves. Recovery copies use the current format. Rollback (an older build) refuses v2 files with the "newer version" message, as specified.
