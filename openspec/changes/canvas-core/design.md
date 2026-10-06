## Context

`app-shell` delivered the workspace with a placeholder canvas (`crates/tp-app/src/ui/workspace/canvas.rs`) that draws a fitted artboard and maps the pointer to texture pixels. `Workspace` (`crates/tp-app/src/state.rs`) holds a `ProjectStub` (name, resolution), the active tool, and the last artboard rectangle. Commands live in a registry (`commands.rs`) whose availability is static (`Always` / `NeedsProject` / `NotYet`) and evaluated by `state::is_enabled(id, has_project)`. Space already switches temporarily to the Hand tool. `tp-core` has no geometry yet.

See proposal.md for motivation and specs/ for required behavior. The canvas wheel mapping (mouse wheel zooms, trackpad scroll pans) was chosen by the user.

## Goals / Non-Goals

**Goals:**
- A document model in `tp-core` that later changes (text, images, paths, groups, persistence, export) extend without reshaping it.
- One geometry source (`kurbo` paths) used for drawing, hit testing and, later, export rasterization.
- Interaction code structured as explicit gesture state machines, so tools stay testable and predictable.
- Navigation and selection overlays that stay smooth with 1000 objects.

**Non-Goals:**
- Concave or self-intersecting geometry (needed by the future pen tool): only convex shapes are drawn here.
- System clipboard interop; groups; editing colors; snapping, grid, guides; persistence; GPU or CPU rasterization for export.

## Decisions

### D1. Document types in `tp-core::document`

```
Project { name, resolution, surfaces: Vec<Surface>, active_surface: usize, next_id: u64 }
Surface { id: SurfaceId, name, size: f64, objects: Vec<Arc<Object>> }
Object  { id: ObjectId, kind: ShapeKind, frame: Frame, fill: Rgba, stroke: Option<StrokeStyle>, opacity: f32 }
ShapeKind = Rectangle { corner_radius: f64 } | Ellipse
Frame   { center: Point, size: Size, rotation_deg: f64 }        // texture pixels
```
`ProjectStub` is replaced by `Project` (`ProjectStub::new` logic — trimmed name, "Untitled" default — moves to `Project::new`). IDs come from a per-project counter so they stay stable and serializable later. Objects are `Arc`-shared so snapshots of a surface are cheap (see D6). A `Frame` with rotation (instead of a full affine) keeps X/Y/W/H/rotation directly editable by the future Properties panel; skew is deliberately not representable.
*Alternatives:* full `Affine` per object (allows skew, but every property edit needs decomposition); an ECS (overkill for a document of this size).

### D2. Geometry with `kurbo`
Each kind builds its outline in its local unit space (`kurbo::Rect` / `RoundedRect` / `Ellipse` centered at the origin with the frame size), placed by `frame.to_affine()` (translate · rotate). Hit testing transforms the point into local space with the inverse affine and uses `kurbo::Shape::contains`, with a tolerance expressed in screen pixels converted to texture pixels by the caller (so thin or tiny shapes stay clickable). Bounds come from `bounding_box()` of the transformed path.
*Alternatives:* `lyon` geometry (heavier; useful later for tessellating concave paths, can be added then); hand-written math (error-prone for rounded rectangles and rotation).

### D3. Viewport
`Viewport { center: Point /* texture px at the canvas center */, zoom: f64 /* physical screen px per texture px; 1.0 = 100% */, fitted: bool }` lives in `Workspace`, outside the document and the history. Screen mapping: `screen = canvas_center + (doc - center) * zoom / pixels_per_point`. Zooming at a pointer recomputes `center` so the document point under the pointer is invariant. `fitted` is true after open / Fit to Screen and cleared by any user zoom or pan; while true, a resized canvas refits (spec: Canvas area). Zoom is clamped to 0.02..=64.0; presets for Zoom In/Out: 2, 3, 4, 6, 8, 12, 16, 25, 33, 50, 67, 100, 150, 200, 300, 400, 600, 800, 1200, 1600, 3200, 6400 %.

### D4. Rendering on the canvas
All shapes in this change are convex (rectangle, rounded rectangle, ellipse), so each object is drawn as one `epaint` convex polygon (fill) plus a closed line (stroke), with opacity multiplied into the colors. Outlines are flattened (`kurbo::flatten`) in texture space and cached per object, keyed by the `Arc` pointer plus a zoom bucket (power of two) that sets the flattening tolerance; panning and zooming within a bucket only re-map cached points with one affine (spec: no geometry recompute on view changes). The cache is pruned of entries not used during the frame. Drawing order: pasteboard → artboard fill → objects (clipped to the canvas rect, not to the artboard, so off-artboard objects stay visible) → artboard border → hover outline → selection overlay → gesture feedback (marquee, preview, labels).
*Alternatives:* `lyon` tessellation now (needed only for concave paths — postponed to the pen tool change); CPU raster preview with tiny-skia (too slow for interactive feedback; kept for export).

### D5. Gestures as state machines
The canvas widget owns a `Gesture` value in `Workspace`:

```
Idle --press(shape tool)--------------> Drawing { kind, start }
Idle --press on handle----------------> Resizing { handle, start_frames, start_bounds }
Idle --press near corner (outside)----> Rotating { pivot, start_angle, start_frames }
Idle --press on object----------------> Moving { start, start_frames }
Idle --press on empty (select tool)---> Marquee { start, additive }
Idle --press (zoom tool)--------------> ZoomRect { start }
Idle --press (hand / Space / middle)--> Panning { last }
any  --release------------------------> commit (one history entry) --> Idle
any  --Escape-------------------------> restore start state, no entry --> Idle
```
During Moving / Resizing / Rotating the gesture applies transforms to the live document from the recorded start frames each frame (no accumulated drift); modifier keys are re-read every frame so Shift/Alt apply live. A press is classified by a hit-test priority: rotation zone > resize handles > objects > empty canvas. A drag threshold of 3 screen pixels separates clicks from drags.

### D6. Undo history by snapshots
`History { undo: Vec<Entry>, redo: Vec<Entry>, max: 200 }`, `Entry { label: &'static str, before: Snapshot, after: Snapshot, at: Instant }`, `Snapshot { surfaces' object lists (Vec<Arc<Object>>), active_surface, selection }`. Snapshots share unchanged objects through `Arc`, so cost is proportional to the object count, not their content. `Workspace::commit(label)` records the change; nudges with the same label within 1 s replace the previous entry's `after` (coalescing). Undo/Redo restore the document and selection, mark the project unsaved, and never touch the viewport.
*Alternatives:* command pattern with inverse operations — more memory-efficient but every new operation needs a correct inverse; rejected as less reliable (user priority: stability).

### D7. Selection and multi-object transforms
Selection is an ordered `Vec<ObjectId>` in `Workspace` (part of snapshots). Single selection uses the object's oriented frame for bounds and handles. Multi-selection uses the axis-aligned union of object bounds: move translates centers; rotate rotates centers around the bounds center and adds the angle to each rotation; resize scales centers relative to the anchor and scales each object's size by the scale factors projected onto its own axes (exact for unrotated objects and uniform scaling, a close approximation otherwise — skew is not representable by D1). Dragging a handle past the anchor flips the scale sign; the result is stored as a positive size with the frame mirrored by adjusting rotation by 180° where needed.

### D8. State-dependent command availability
`Availability` gains a `WhenAvailable(fn(&EditContext) -> bool)` variant, and `is_enabled(id, &EditContext)` replaces `is_enabled(id, has_project)`. `EditContext { has_project, has_selection, can_undo, can_redo, has_clipboard }` is built once per frame from `AppState`. Undo/Redo menu labels become dynamic ("Undo Move") through an optional label override in `CommandUi`. Backspace is added as an alternative shortcut for Delete, arrow keys (with and without Shift) become canvas-scoped commands, and Escape clears the selection when no gesture is active. The application clipboard is a `Vec<Object>` in `AppState` (not the OS clipboard).

### D9. Input mapping and egui configuration
`egui::Options::zoom_with_keyboard` is set to `false` so Cmd/Ctrl +/-/0 reach the canvas commands instead of scaling the interface (UI scale stays in Preferences). The canvas reads raw `Event::MouseWheel { unit, delta, modifiers }` and `Event::Zoom` while hovered: `Line`/`Page` units → zoom (Shift → horizontal pan), `Point` units → pan, any unit with Cmd/Ctrl → zoom, `Zoom(factor)` → zoom at the pointer. Wheel zoom uses `factor = 1.0015^(-delta_points)` (line deltas converted at 40 points per line) for smooth steps.

### D10. Cursors and overlay styling
egui cursors cover crosshair, grab/grabbing, move, zoom in/out and the four resize directions; the resize cursor is chosen from the handle direction rotated by the object rotation, rounded to the nearest of the four. There is no rotation cursor, so the canvas hides the system cursor in the rotation zone and draws a rotate glyph at the pointer. Overlay tokens are added to `tp-ui`: selection color (accent), a 1 px contrasting halo under every overlay line, 8 px square handles with white fill, a 6 px center mark, and a 28 px rotation zone around corners. Default fill for new shapes: `#5B8DEF` (overlay-distinct, readable on the light artboard), no stroke.

### D11. Testing
- `tp-core`: unit tests for frames, affine round-trips, hit testing (rotation, rounded corners, ellipse corners), bounds, multi-object transforms, history (coalescing, redo clearing, bound).
- `tp-app`: viewport math tests (zoom invariance at pointer, fit, clamp) and kittest scenarios driven with `drag_at`/`drop_at`/`hover_at`/raw wheel events: draw with modifiers, select/marquee, move/resize/rotate, undo/redo, commands' enabled state.
- A `#[ignore]` performance check builds 1000 shapes and asserts a frame with pan stays under a time budget in release mode.

## Risks / Trade-offs

- [Mouse vs. trackpad detection relies on line vs. pixel scroll units; some devices (Magic Mouse, some Windows precision touchpads) report the "other" unit] → Cmd/Ctrl+scroll always zooms and Space/middle-drag always pan, so every device has a reliable path; a preference to choose the wheel behavior can be added later.
- [Approximate multi-object resize of rotated objects (no skew)] → documented in D7; matches what users see in practice for typical livery work (mostly unrotated stripes and logos).
- [Snapshot history memory grows with object count × 200 steps] → objects are `Arc`-shared, so a step costs one pointer per object; fine for thousands of objects.
- [Geometry cache keyed by `Arc` pointer can be invalidated by pointer reuse] → entries are pruned every frame and compared together with the object id and frame, so a reused address with different content misses the cache.
- [kittest gesture simulation might not cover every modifier timing] → transform math is unit-tested in `tp-core`; UI tests cover the wiring.

## Migration Plan

Not applicable: no persisted documents exist yet. `ProjectStub` is removed and its uses in `tp-app` are migrated to `Project` in the same change.

## Open Questions

- Default fill color for new shapes is a placeholder until the Colors panel exists (`editing-panels`); changing it later is cosmetic.
