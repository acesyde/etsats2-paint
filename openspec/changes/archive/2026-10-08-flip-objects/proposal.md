## Why

A livery is often designed once and repeated facing the other way: a logo or a stripe drawn for the left side of the cab must point forward on the right side too. Today the only way to flip is to drag a resize handle past the opposite side. That is imprecise, because the size has to be matched by hand. It also does nothing to images and texts, which move but are never mirrored. Players need an exact, one-step flip that works on everything they place.

## What Changes

- New commands **Flip Horizontal** (Shift+H) and **Flip Vertical** (Shift+V) mirror the selection in place, about the center of the selection bounds. The objects keep their size and the selection keeps its bounds. With several objects, the whole selection is mirrored as one block, so the objects swap sides. Each flip is one undo step.
- The commands appear in the Object menu, in the canvas context menu and as two buttons in the Transform panel, next to the distribute buttons. Like other commands, they are disabled when nothing editable is selected.
- **Images and texts can be mirrored.** They get a mirrored state that both commands toggle, as does dragging a handle past the opposite side. A mirrored image shows its pixels reversed. A mirrored text reads backwards and stays editable as text. A vertical flip of an image or a text is drawn as a horizontal mirror turned by 180°.
- Mirrored images and texts stay mirrored wherever they are drawn: the canvas, the 3D preview, texture and mod export, and the content of mirrored symbol instances. The eyedropper samples the pixel shown. Create Outlines on a mirrored text gives mirrored letters.
- The mirrored state is saved in the `.truckpaint` file as an optional field. Files without it open with nothing mirrored, and the format version is unchanged.
- Shapes, paths, polygons, gradients, groups and instances already mirror correctly through their geometry. The commands reuse that behavior.
- **Remove the placeholder Object › Mirror to Other Side.** It has been disabled since the app shell, waiting for mirror data that vehicle packages don't have, and nothing on the roadmap plans that data. Next to the new Flip commands, a permanently disabled "Mirror" item would only confuse. Its command, menu entry and strings are removed. If the idea comes back, it will be a new change, together with the package data it needs.

Non-goals: mirroring across the vehicle, which would need left/right mapping data in packages; flipping about an arbitrary axis or a guide; a mirrored state on shapes, whose geometry already carries the mirror.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `selection-transform`: new Flip Horizontal and Flip Vertical object commands, and dragging a handle past the opposite side also mirrors images and texts.
- `document-model`: images and texts have a mirrored state, applied wherever they are drawn and when outlining.
- `transform-panel`: a row of Flip buttons.
- `project-files`: the mirrored state is saved and restored.

## Impact

- **tp-core:** an `Object.mirrored` flag, meaningful for images and texts; `transform::flip`, built on `resize_one`, which toggles the flag and fixes the rotation of images and texts when a transform mirrors them. This covers handle flips, flip commands and instance placements.
- **tp-text:** `layout_to_doc` applies the mirror. This covers text rendering, caret and selection hit testing and glyph outlines.
- **tp-render / tp-app canvas:** image placement and eyedropper UVs apply the mirror.
- **tp-file:** an optional `mirrored` field on `FileObject`. The `v1.truckpaint` fixture is unchanged, and a round-trip test is added.
- **tp-app:** `CommandId::Flip(FlipAxis)`, menus, context menu, Transform panel buttons, icons, undo labels, and locale strings in en/fr/de/es. `CommandId::MirrorToOtherSide` is removed, with its now unused `reason-soon-vehicles` reason and the `cmd-mirror-to-other-side` and `reason-soon-vehicles` strings in the four languages.
- No new dependency.
