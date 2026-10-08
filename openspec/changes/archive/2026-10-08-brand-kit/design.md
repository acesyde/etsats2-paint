## Context

See proposal.md (Why) and the specs for the behavior. Today:
- `Project.palette` is a `Vec<Rgba>`. The Colors panel (`panels/colors.rs`, `palette()`) applies a swatch as a plain color. `Project::add_to_palette` refuses duplicates.
- Colors live in `Paint::Solid(Rgba)` for fills and strokes, and in `ColorStop { offset, color }` for gradients. `Paint` and `StrokeStyle` are `Copy` and are matched in about 13 files, including `tp-render`.
- A text's look is `TextBlock.style: CharStyle`. Its `layout_size` is computed by the text engine, and `Workspace::relayout_all_texts` redoes the layout.
- Every edit ends in `Workspace::record(label, before, now, coalesce)`, through `edit`, `live_edit` plus `commit_pending`, and the tools. Undo stores whole `Snapshot`s, in which objects are shared through `Arc`.
- `tp-file` v1 uses serde with `#[serde(default)]` on optional fields. Adding optional fields keeps old files readable.
- `vehicle_project::scale_surface` scales artwork from the top-left corner with `transform::resize`. Update Template uses it.
- Commands are declared in `commands.rs`, with an enablement rule and a reason, and listed in `menu_bar.rs`. `PanelLayout::sanitized` adds panels missing from saved preferences.

## Goals / Non-Goals

**Goals:**
- Links between colors and swatches, and between objects and styles, that existing tools don't need to know about. The picker, hex, eyedropper, gradient tool, stroke panel and Character settings keep working unchanged, and detaching happens by itself.
- Rendering and export unchanged. Objects keep holding their resolved colors and character settings.
- Project files stay at format version 1, and older files open.

**Non-Goals:**
- Symbols with instances: the next change, `symbols`. A symbol will be edited in its own view, as decided with the user.
- Style overrides, where an object follows a style except for one property. Any own change detaches it.
- Sharing swatches or styles across projects. The roadmap's open question "Symbol library across projects" also covers this.
- Applying a swatch from the Styles panel, or styles to images.
- A style picker in the Properties or Character panels. The Styles panel marks what the selection follows.

## Decisions

### 1. Links stored next to the resolved values
- **Ids:** `SwatchId(u64)` and `StyleId(u64)` come from the project's id counter, so they are never reused.
- **Swatch links:**
  - `Object.fill_swatch: Option<SwatchId>`, for a solid fill;
  - `StrokeStyle.swatch: Option<SwatchId>`, for a solid stroke;
  - `ColorStop.swatch: Option<SwatchId>`, for each stop.
- **Style links:** `Object.style: Option<StyleId>` for a graphic style, and `TextBlock.style_id: Option<StyleId>` for a text style.
- **Project:**
  - `palette: Vec<Swatch { id, name, color }>`;
  - `graphic_styles: Vec<GraphicStyle { id, name, fill, fill_swatch, stroke, opacity }>`;
  - `text_styles: Vec<TextStyle { id, name, style: CharStyle, look: Look }>`. A text style holds the whole lettering, as decided with the user after trying the first version: its `look` is a `Look { fill, fill_swatch, stroke, opacity }`, the same type a `GraphicStyle` holds.

  All three go into `Snapshot` and `same_document`.

The resolved `Rgba` and `CharStyle` stay where they are, and links only say where a value comes from. `tp-render`, export, hit testing and every `Paint::Solid(c)` match are untouched, and `Paint` stays `Copy`.

**Alternatives:**
- `Paint::Solid { color, swatch }`, or a `Paint::Swatch` variant. Rejected: every renderer and tool would need to resolve or ignore it, for no gain over a field next to the value.
- Resolving links at render time from ids. Rejected: it changes rendering and export, and makes a missing swatch a rendering concern.

### 2. One normalization pass when an edit is recorded
`Project::relink()` walks every object of every surface, and the graphic styles. It drops each link whose value no longer matches:
- a swatch link whose color differs from the swatch's, or whose fill or stroke paint is no longer solid;
- a swatch link to a swatch that doesn't exist, for example on an object pasted from another project;
- a graphic style link whose object's look differs from the style's (comparison below), or whose style doesn't exist;
- a text style link whose character settings, or whose look (compared as for graphic styles), differ from the style's.
- for a text that follows a text style, the graphic style link is cleared (a text follows one style).

`Workspace::record` calls `relink()` before taking the after-snapshot. That covers `edit`, `commit_pending`, and the tools that record directly. Every way of changing a look therefore detaches by itself, and no tool changes.

Walking the whole project per recorded edit is cheap. Unchanged objects are skipped, because `Arc::make_mut` only clones the ones whose links actually change.

**Alternative:** clear links in each editing path: picker, hex, eyedropper, gradient tool, stroke panel, Properties opacity, Character settings, paste. Rejected: about ten places to keep right, and any new tool would silently keep stale links.

### 3. Comparing a look with a graphic style
`GraphicStyle::matches(&Object)` compares:
- the fill: a solid color, or a gradient's kind and stops (offsets, colors and swatch links). It ignores the gradient's `start`, `end` and `minor`;
- the stroke, entirely, except a gradient stroke's points;
- the opacity, exactly.

Applying a style copies the fill and stroke paints, but **keeps the object's own gradient points** when the object already has a gradient of the same kind. Otherwise it uses the style's points, which are in frame units. Propagating a style edit works the same way.

Moving, rotating and resizing remap gradient points (`Paint::remapped`), so ignoring them keeps the link through every transform, as the spec requires.

### 4. Propagation as explicit project operations
These all live in `tp-core::project`:
- `set_swatch_color(id, color)`:
  1. updates the swatch;
  2. updates every color linked to it: object fills, strokes and stops, and the graphic styles' paints;
  3. updates every object following a graphic style that changed, which falls out of step 2 because those objects link the same swatch.
- `add_swatch(color) -> SwatchId`, `rename_swatch` and `delete_swatch`. Deleting leaves the colors and lets `relink()` drop the links.
- `new_graphic_style(from: ObjectId)` and `new_text_style(from: ObjectId)`. They name the style ("Style N" or "Text style N") and link every selected object whose look already matches.
- `apply_graphic_style(id, objects)` and `apply_text_style(id, objects)`. Groups are walked down to their shapes and texts. Images are skipped for graphic styles, and anything but texts for text styles.
- `redefine_graphic_style(id, from)`, `redefine_text_style(id, from)`, `rename_style` and `delete_style`.

The Edit Swatch popup uses `live_edit("colors-edit-swatch", …set_swatch_color…)` on each change, `commit_pending` on OK and `cancel_pending` on Cancel, as the picker does today. One undo step covers the whole edit.

After any text style change, the workspace calls `relayout_all_texts()`: new character settings change `layout_size`.

Style operations from the Styles panel first end a text being typed on the canvas (`end_text_session`), so the typing is its own undo step. Otherwise the session's step, recorded later from a snapshot taken before the text existed, would make a single Undo remove the text.

### 5. File format: optional fields in v1
New fields, all `#[serde(default)]`:
- `FileProject.swatches: Vec<FileSwatch { id, name, color }>`;
- `FileProject.graphic_styles` and `FileProject.text_styles`;
- `FileObject.fill_swatch` and `FileObject.style`;
- `FileStroke.swatch`, `FileStop.swatch` and `FileText.style_id`.

Writing fills `swatches` and leaves `palette` empty. Reading an empty `swatches` with a non-empty `palette` turns each legacy color into "Color N". After reading, `relink()` drops any inconsistent link, so a hand-edited file can't make a color disagree with its swatch.

The format version stays 1. The project is unreleased, and the fields are additive.

**Alternative:** format 2 with a migration. Rejected: no released file needs migrating, and the optional fields already read old files.

### 6. Copy from cabin
`Workspace::copy_from_texture(source, now)`:
1. clones the source surface's objects with fresh ids, recursively (`Project::duplicate` logic without the offset);
2. scales them by `target.size / source.size` from the top-left corner. This shares `scale_surface`'s `resize` call, extracted as `scale_objects(objects, old, new)`;
3. appends them to the active surface and selects them;
4. records one undo step (`"cmd-copy-from-cabin"`).

Enablement comes from a `CommandContext.cabin_sources` count: the other main textures of the active surface's vehicle. A `CopyFromCabinDialog` modal lists them with their object counts.

**Alternative:** paste into the target through the clipboard. Rejected: the clipboard is shared with the user's own copy and paste, and pasting offsets the objects.

### 7. Interface
- **Colors panel palette:**
  - swatches get their name as hover text and accessible name;
  - a ring and the name line mark the swatch linked to the current target or stop;
  - the context menu has Edit Swatch… and Delete Swatch;
  - clicking a swatch sets the color and the link in one edit.
- **Edit Swatch popup:** a small `egui::Modal` with the name field, the existing picker widgets (`sv_square`, `hue_slider`, `alpha_slider`) and the hex field, plus OK and Cancel.
- **Styles panel:** `PanelKind::Styles` sits after Colors in `PanelKind::ALL`. `sanitized()` inserts it, collapsed, into layouts saved without it. Each section shows:
  - a header with + (New Style from Selection, with its disabled reason);
  - rows for the styles: a fill and stroke chip for graphic styles, the name in its own font for text styles (from the font library, Inter as fallback);
  - a context menu per style: Rename (inline), Redefine from Selection, Select Users on This Texture, Delete.
- **Vehicle menu:** Copy From Cabin… after Next and Previous Texture.

## Risks / Trade-offs

- **[Exact float comparisons for opacity, gradient offsets and character settings]** Equality could drop a link after a round trip through a lossy edit. → Values are copied, never recomputed, when applied or propagated, so they stay bit-identical. File round trips keep `f32` and `f64` exactly.
- **[A color picked by hand that happens to equal a swatch isn't linked]** → As specified: linking is explicit (clicking the swatch). This keeps the rule simple, a link holding while the value equals.
- **[Live swatch edits rewrite many objects per frame]** → `Arc::make_mut` touches only the linked objects. A fleet of a few thousand objects stays well under a frame. Rendering caches are keyed by object content and refresh as usual.
- **[`relink()` in `record` hides a bug where an operation forgets to propagate]** The link would silently drop. → Unit tests assert that links survive every propagation operation and every transform, and the UI tests cover the spec scenarios.
- **[Text style changes need a relayout]** → `relayout_all_texts()` is called after every text style operation. A test checks a redefined size changes `layout_size`.

## Migration Plan

Additive. Files written by this version still open in the previous one, which ignores the unknown fields. They lose the palette, which is now written as `swatches`, the styles and the links, but every object keeps its look. That's acceptable before the first release. Rollback is reverting the change.
