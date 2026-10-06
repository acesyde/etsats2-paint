## Why

The application shell exists, but the canvas is a static placeholder: nothing can be drawn, selected or navigated. The canvas is the heart of a livery editor, so the next step is a real document model and a canvas that feels like Illustrator / Figma: smooth navigation, vector shapes, a clear selection system with handles, and reliable undo. Everything that follows (text, images, layers, vehicle templates, export) builds on this foundation.

## What Changes

- Introduce a vector **document model** in `tp-core`: a project holds one or more texture surfaces (artboards in texture-pixel coordinates); each surface holds an ordered list of objects. This change adds rectangle (with optional corner radius) and ellipse objects, each with a frame (center, size, rotation), a solid fill, an optional stroke and an opacity. The model is resolution-independent and contains no UI types.
- Replace the static canvas with an **interactive canvas**: zoom toward the pointer (2%–6400%), pan, Fit to Screen, Actual Size (100%), Zoom In/Out commands, Zoom and Hand tools, and the hybrid wheel behavior (mouse wheel zooms, trackpad two-finger scroll pans, pinch zooms, Cmd/Ctrl+wheel always zooms, Shift+wheel pans horizontally).
- Add **Rectangle and Ellipse tools**: drag to create, Shift constrains to square/circle, Alt/Option draws from the center, Escape cancels an in-progress drag.
- Add a **selection and transform system**: click, Shift+click, marquee selection, Select All, Deselect; bounding box with eight resize handles, rotation zone and center mark; move by drag or arrow keys (Shift = 10 px); resize with Shift (keep proportions) and Alt (from center); rotate with Shift snapping to 15°; works on multiple objects at once.
- Add **object commands**: Delete, Duplicate, Cut / Copy / Paste within the application, Bring Forward / Send Backward.
- Add **undo / redo**: every document change (create, delete, move, resize, rotate, paste, reorder) is undoable; one drag gesture is one undo step; history is bounded.
- Enable the corresponding menu commands and shortcuts that were disabled in `app-shell` (Undo, Redo, Cut, Copy, Paste, Duplicate, Delete, Select All, Bring Forward, Send Backward, Zoom In/Out, Fit to Screen, Actual Size).
- The status bar shows the live zoom level and the document's save state reacts to edits (still "Unsaved changes" until persistence exists).

Not in this change: colors panel and editing fills/strokes (`editing-panels`), layers panel and groups, text, images/SVG, pen tool and paths, snapping/grid/guides, saving/opening projects, vehicle templates, export, 3D preview. Tool buttons for those remain present and their tools show a "not available yet" hint on the canvas.

## Capabilities

### New Capabilities

- `document-model`: surfaces in texture-pixel coordinates, ordered vector objects (rectangle, ellipse) with frame, fill, stroke and opacity; object identity and z-order.
- `canvas-navigation`: zoom and pan behavior, zoom limits, fit / actual size, wheel and trackpad mapping, Zoom and Hand tools, navigation that keeps the view stable when the window resizes.
- `shape-tools`: creating rectangles and ellipses by dragging, with modifier constraints and cancellation.
- `selection-transform`: selecting objects, the selection overlay (bounding box, handles, rotation, center), moving, resizing, rotating single and multiple objects, nudging, and object commands (delete, duplicate, clipboard, z-order).
- `undo-history`: undo/redo semantics, gesture coalescing, history bounds and interaction with the save state.

### Modified Capabilities

- `workspace-layout`: the "Canvas area" requirement changes — the artboard is fitted when a project opens but the view is then user-controlled (zoom/pan), and resizing the window keeps the view centered instead of refitting.

## Impact

- `tp-core`: new `document` module (surfaces, objects, frames, colors), geometry via `kurbo`, hit testing, undo history. New dependency: `kurbo`.
- `tp-app`: canvas rewritten as an interactive widget (viewport state, tool state machines, overlay drawing); `Workspace` owns the document, selection, viewport and history; command availability for editing commands now depends on state (e.g. selection non-empty).
- `tp-ui`: tokens for selection overlay (handle size, selection color) and canvas cursors.
- egui configuration: built-in UI zoom on Cmd/Ctrl +/- is disabled so those shortcuts drive the canvas zoom.
- Tests: unit tests for geometry, hit testing, transforms and history in `tp-core`; headless UI tests (egui_kittest) for tools, selection and navigation.
