## Why

Real liveries often put a drop shadow under their lettering and logos, so that a company name stays readable on a busy or light background. Today a player fakes it: they duplicate the text, recolor it, move it a few pixels and keep both copies in sync by hand on every texture. The redesign's inspector (`new-design`) has a slot for it: Appearance shows Fill, Stroke and "+ Add a shadow" as one compact row each.

**This is a new feature, not polish.** It is attached to roadmap #2 (`polishing`) only because it comes from the same mockup, and it is **optional and low priority**: it can be dropped or postponed without affecting the other changes. It must also be weighed against "TruckPaint is not a general vector editor". The case for it: a shadow is a livery effect (lettering, logos, stripes) that the game's textures need and players already fake. It does not open the door to general effects: no inner shadow, glow, multiple shadows or filter stack.

## What Changes

Depends on `new-design` (the inspector's Appearance section).

- **A minimal shadow** on shapes, paths, texts and images. It has:
  - a color, which may link to a palette swatch like a fill;
  - an opacity (0–100%);
  - an offset x / y in texture pixels;
  - a blur radius in texture pixels (0 = a hard shadow).
  The shadow is the object's silhouette (fill and stroke) drawn under it in the shadow color, then moved and blurred. The object's opacity applies to it too. One shadow at most per object. Groups and symbol instances have no shadow of their own; a symbol's content can.
- **Inspector:** "+ Add a shadow" in Appearance adds a default shadow (black, 50%, offset 8 / 8 px, blur 8 px). The shadow then shows as one compact row, like Fill and Stroke. Clicking the row opens a popover with the four settings, closed with Escape, and the row has a button to remove the shadow. Each change is one undo step, and a drag of a value is one step.
- **Linked like other colors:** a shadow color picked from a swatch follows Edit Swatch…. Picking another color unlinks it.
- **Styles carry the shadow.** Graphic and text styles describe the whole look, so they include the shadow (or none). Changing an object's shadow detaches it from its style, like changing its fill. Redefine from Selection and New Style from Selection take the shadow.
- **Rendered everywhere:** on the canvas, and identically in Export Texture (PNG, DDS) and in Export Mod's textures. A shadow that extends past the texture is clipped like any object. Hit testing and selection bounds ignore the shadow.
- **Saved in project files** as an optional field of objects and styles in format 1. Files without it open with no shadows.
  - Older builds ignore the field: they open a file with shadows, without them, and a save from such a build drops them. No TruckPaint release exists yet (distribution is roadmap #4), so the format stays 1 (decided).
- **Library and paste:** a shadow's swatch link is a dependency like a fill's, so Add to Library, Import from Library and paste across projects bring it.

Non-goals: inner shadows, glows, several shadows, spread, blend modes, shadows on groups or instances.

## Capabilities

### New Capabilities
- `drop-shadow`: the shadow property:
  - what it holds (color, swatch link, opacity, offset, blur) and the objects that can have one;
  - how it is drawn (silhouette, order, object opacity, clipping, no effect on hit testing or bounds);
  - the Appearance row and popover, and undo.

### Modified Capabilities
- `shared-styles`: graphic and text styles include the shadow; changing an object's shadow detaches it from its style.
- `texture-export`: Rendering fidelity lists shadows, identical to the canvas. Export Mod uses the same renderer, so it follows.
- `project-files`: objects and styles save their shadow and its swatch link. Files without shadows open unchanged.
- `shared-library`: a shadow's swatch link counts among the swatches an element uses, for Add to Library and import.
- `properties-panel`: Appearance lists the Shadow row (or + Add a shadow), left out for groups and instances.
- `brand-impact`: a swatch's usage counts objects whose shadow links to it.

## Impact

- **tp-core:**
  - a `Shadow { color, swatch, opacity, offset, blur }` as `Option<Shadow>` on `Object` and in `Look` (`brand.rs`), so style matching and detaching cover it;
  - swatch relinking and the library import closure follow its link.
- **tp-render:** draws the shadow under each object. tiny-skia has no blur, so a separable box blur (three passes, close to a Gaussian) on an offscreen pixmap of the silhouette, bounded to the blurred area. No new dependency expected.
- **tp-app:**
  - the canvas (`ui/workspace/canvas/paint.rs`) draws with egui meshes and can't blur, so shadows are rasterized per object with tp-render and cached as textures, invalidated with the geometry cache;
  - the Appearance row and popover;
  - strings in en/fr/es/de.
- **tp-file:** optional `shadow` on `FileObject`, `FileGraphicStyle` and `FileTextStyle` in format 1 (see above).
- **Performance:** blur cost grows with radius and object size on 4096² textures. The blur is capped at 200 px, and the canvas cache renders shadows on a worker at a zoom-tied resolution (see design).
