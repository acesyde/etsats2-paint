## Why

`canvas-core` made shapes drawable and transformable on the canvas, but everything else about them is fixed: every shape is the same blue, strokes and opacity cannot be set, exact positions and sizes cannot be typed, and the document is a flat list with no names, grouping, hiding or locking. A livery is built from many precisely placed, colored and organized elements ("Background", "Graphics › White stripe", "Branding › Logo"), so the right-hand panels have to become real editing tools.

## What Changes

- **Document model**: objects gain a name, a visibility flag and a lock flag; a new **group** object holds an ordered list of children (groups can nest), so the document becomes a tree. A project gains a **palette** of saved colors. Hidden and locked objects (and the children of hidden/locked groups) are not hit by canvas clicks or marquees. The workspace keeps a **current fill and stroke** used for new shapes.
- **Transform panel**: X / Y (center), W / H, rotation and scale fields for the selection, with a proportions lock, scrubbable labels, typed values committed on Enter, Escape to revert, and "Mixed" display for differing values.
- **Properties panel**: context summary of the selection (name, kind or count), opacity, corner radius for rectangles, and fill / stroke swatches that target the Colors panel.
- **Colors panel**: fill / stroke target switch (X), swap (Shift+X) and default (D); color picker (saturation/value square, hue and alpha sliders); RGB / HSV / HSL sliders and fields; hex field; recent colors (persisted in preferences); project palette (add, apply, remove); "None" for stroke. With no selection, it edits the current style used for new shapes.
- **Stroke panel**: enable/disable the stroke and set its width in texture pixels.
- **Eyedropper tool (I)**: clicking an object picks its fill (or stroke, per the Colors target) and applies it to the selection, or to the current style when nothing is selected.
- **Layers panel**: tree of the active surface (topmost first) with expand/collapse, thumbnails-free rows showing kind icon and name, visibility and lock toggles, inline rename (double-click), selection sync with the canvas (click, Shift range, Cmd/Ctrl toggle), drag and drop to reorder and to move objects into or out of groups, and a context menu (Rename, Duplicate, Delete, Group, Ungroup).
- **Commands enabled**: Group (Cmd/Ctrl+G), Ungroup (Cmd/Ctrl+Shift+G), New Layer (Cmd/Ctrl+Shift+N, creates an empty top-level group), Duplicate Layer, Delete Layer.
- **Canvas selection with groups**: clicking an object inside a group selects the top-level group; Cmd/Ctrl+click selects the innermost object. Transforms apply to whole groups.
- **New shapes** use the current fill and stroke and are inserted at the top of the active layer (the group containing the current selection, or the top level).
- Every edit made through these panels is one undo step with a descriptive label.

Not in this change: gradients, text, images/SVG import and the Assets panel (`text-and-images`), blend modes, stroke alignment/joins/dashes, layer thumbnails, multiple surfaces UI, snapping/guides, persistence.

## Capabilities

### New Capabilities

- `transform-panel`: numeric position, size, rotation and scale editing of the selection, including mixed values, proportional lock and field interaction rules.
- `properties-panel`: contextual summary and appearance properties (opacity, corner radius, fill/stroke swatches) of the selection.
- `color-panel`: fill/stroke target, picker, color models, hex, recent colors, project palette, eyedropper tool, current style for new shapes.
- `stroke-panel`: stroke enable/disable and width.
- `layers-panel`: the object tree view with visibility, lock, rename, selection sync, drag-and-drop reordering and grouping, and the Group/Ungroup/New Layer commands.

### Modified Capabilities

- `document-model`: objects are organized as a tree with groups; objects have name, visibility and lock; the project has a palette; new shapes use the current style; hit testing ignores hidden and locked objects.
- `selection-transform`: selecting objects inside groups (top-level group by default, Cmd/Ctrl+click for the innermost object); hidden and locked objects cannot be selected on the canvas.
- `shape-tools`: drawn shapes use the current fill/stroke and go to the active layer; the Eyedropper is no longer listed as unavailable.
- `undo-history`: the list of undoable changes includes appearance, property, naming, visibility, lock, grouping and palette edits.

## Impact

- `tp-core`: `Object` gains `name`, `visible`, `locked`; `ShapeKind::Group { children }`; tree-aware operations (find by id, parent lookup, insert into group, move between parents, group/ungroup, flatten for drawing and hit testing); `Project.palette`; color conversions (RGB ↔ HSV/HSL, hex parsing/formatting).
- `tp-ui`: new widgets — numeric scrub field, color swatch, saturation/value square, hue/alpha sliders, tree row with drag handle, toggle icon buttons.
- `tp-app`: real implementations of the Properties, Transform, Colors, Stroke and Layers panels; Eyedropper tool; current style in `Workspace`; recent colors in `Prefs` (backward-compatible default); group-aware canvas selection; new commands.
- Tests: unit tests for tree operations and color conversions; kittest scenarios for every panel.
