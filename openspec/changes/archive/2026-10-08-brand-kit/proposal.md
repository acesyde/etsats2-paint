## Why

A project is a fleet: one company identity painted on several trucks and trailers (`docs/roadmap.md`, "What is shared: elements, not placement"). Today the identity can't be shared:
- the palette only stores colors to copy;
- a text's font settings and an object's fill and stroke are set one object at a time;
- the second cabin layout of a truck starts empty, though it often shares most of the first one's texture.

Changing the company red, or the lettering font, means finding and fixing every object on every texture. `brand-kit` makes the palette and the styles shared and linked, and lets a cabin start from another one.

Symbols with instances, the other shared element, get their own change, `symbols`, right after this one (decided with the user).

## What Changes

**Brand palette (linked colors):**
- Palette swatches get a name ("Company red"), which defaults to "Color 1", "Color 2"…
- Applying a swatch **links** the fill, the stroke or the gradient stop to it.
- **Edit Swatch…** changes a swatch's name and color. Every linked color in the project follows: all textures, every vehicle, and the shared styles.
- A color stays linked as long as it equals its swatch. Picking another color for it, with the picker, hex, eyedropper or a recent color, unlinks it.
- Deleting a swatch keeps every look as it is and drops the links.
- The Colors panel marks the swatch the current target is linked to.

**Shared styles (linked):**
- A **graphic style** is a named fill, stroke and opacity.
- A **text style** is the whole look of a lettering: a named font, weight, italic, size, alignment, letter spacing and line height, plus a fill, a stroke and an opacity. A text follows a graphic style or a text style, not both.
- A new **Styles** panel lists both kinds. It offers:
  - **New Style from Selection**;
  - applying a style to the selection by clicking it;
  - **Redefine from Selection**, **Rename** and **Delete**;
  - a mark on the styles the selection follows.
- Objects follow their style's edits on every texture.
- An object stays linked as long as its look equals the style. Changing its own fill, stroke, opacity or character settings detaches it.
- A gradient's colors and kind come from the style, but its position stays the object's own, so moving or rotating an object keeps it linked.
- Deleting a style keeps every look and drops the links.

**Copy from cabin:**
- **Vehicle › Copy From Cabin…** copies all the artwork of another main texture of the same truck onto the active main texture. It keeps the same coordinates, scaled when the sizes differ, adds the copies on top, selects them, and records one undo step.
- It is enabled only between main textures of one vehicle, since only cabin layouts of the same truck share coordinates.

**Project files:** the swatches with their names, the shared styles, and each color's and object's links are saved. Older files open, and their palette colors become named swatches. The format version stays 1: every new field is optional.

## Capabilities

### New Capabilities
- `shared-styles`: graphic styles and text styles (what they hold, how objects follow, detach and are redefined), and the Styles panel.

### Modified Capabilities
- `document-model`: the project palette becomes named swatches, with colors linked to them.
- `color-panel`: palette swatches have names, apply as links, are edited with Edit Swatch… (every linked color follows) and deleted without changing looks; the linked swatch is marked.
- `vehicle-projects`: new requirement, Copy from cabin.
- `project-files`: the file stores the swatches, the shared styles and the links. Older palettes open as named swatches.
- `workspace-layout`: the right panel stack includes the Styles panel.

## Impact

- **tp-core:**
  - `Swatch` and `SwatchId`;
  - link fields on `Object` (fill swatch, graphic style), `StrokeStyle` (swatch), `ColorStop` (swatch) and `TextBlock` (text style);
  - `GraphicStyle` and `TextStyle` in `Project`;
  - project operations: swatch edit and propagation, styles create, apply, redefine, rename and delete, link normalization;
  - the snapshot used by undo.
- **tp-file:** optional new fields in v1, reading of the legacy `palette`, round-trip tests and a fixture.
- **tp-app:**
  - link normalization when an edit is recorded;
  - text relayout after a text style changes;
  - the Colors panel palette and an Edit Swatch popup;
  - a Styles panel (`PanelKind::Styles`);
  - the Copy From Cabin command and dialog, built on the Update Template scaling;
  - UI tests and screenshots.
- **tp-i18n:** new messages in en, fr, de and es.
- **docs:** `roadmap.md`:
  - records the decisions: linked palette and styles, symbols as their own change, a symbol edited in its own view;
  - adds `symbols` to the next changes;
  - adds `brand-kit` to the Shipped list once it ships.
