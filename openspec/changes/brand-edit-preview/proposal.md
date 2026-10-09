## Why

Linked swatches, styles and symbols are what make a fleet consistent: one edit changes every texture. That is also what makes an edit risky. Today nothing says how far an edit reaches. Edit Swatch… recolors the fleet live, but only the active texture is visible, so a player can't see the 10 other textures it changes. The Symbols panel gives an instance count, but nothing says which textures hold them. The redesign's Brand space (`new-design`) promises that "every change announces its impact on the project before it is applied". This change delivers that promise.

## What Changes

Depends on `new-design`, which creates the Brand space and moves the palette, styles and symbols into it. Part of roadmap #2 (`polishing`).

- **Usage on every element.** In the Brand space:
  - each swatch and each style shows where it is used, as "11 textures · 38 objects", or "Unused";
  - each symbol shows "14 instances · 9 textures".
  A swatch counts every object it changes: objects whose fill, stroke or gradient stop links to it, objects that follow a style linked to it, and the instances of symbols whose content links to it. The counts update as the project changes.
- **Edit Swatch… becomes a before/after editor.** Today Edit Swatch… is a popup that recolors every linked color live in the document, with OK as one undo step and Cancel restoring. It is replaced by an editor that:
  - shows the swatch's name, the "Before" color and the "After" color, with the color picker and hex entry;
  - states the impact, as "Impact: 11 textures, 38 objects";
  - shows live thumbnails of the affected textures with the new color.
  Nothing changes in the project before **Apply to Fleet**, which records the new color (and name) as one undo step. **Cancel** or Escape leaves the project untouched. Opening Edit Swatch… from the Workshop's fill and stroke picker opens the same editor.
- **Edit Style… for graphic styles (feasible, in scope).** A graphic style is a fill, a stroke and an opacity, so the same editor can offer those three controls (the Appearance rows of the inspector), with the same impact line, thumbnails and Apply to Fleet / Cancel. Today a style can only be changed with Redefine from Selection, which stays.
- **Text styles keep Redefine from Selection only.** Their before/after would need the whole character section and a re-layout of every text for each preview. Text styles get usage counts but no editor in this change.
- **Unchanged:** the "Linked, not copied" rules (a link holds while the value equals its source), Delete Swatch, the Styles and Symbols actions, and the symbol view (Edit Symbol), which is already a full editor.

Open question, not resolved here: **how does the Brand space relate to the personal library?** The library (`shared-library`) holds copies of swatches, styles and symbols shared by every project. The Brand space shows the project's own. It is not decided whether the Brand space should list library entries, show which elements came from the library, or offer Update in Library after an edit applied to the fleet. This change keeps Add / Update in Library and Import from Library… as they are.

## Capabilities

### New Capabilities
- `brand-impact`: usage counts of swatches, styles and symbols (what is counted, "Unused"), and the before/after editor shared by swatches and graphic styles: impact line, live thumbnails of affected textures, nothing applied before Apply to Fleet (one undo step), Cancel/Escape leaves the project unchanged.

### Modified Capabilities
- `color-panel`: Edit Swatch… opens the before/after editor instead of recoloring the document live. The "Edit the company red" and "Cancel an edit" scenarios change accordingly.
- `shared-styles`: graphic styles gain Edit Style… through the before/after editor, and styles show their usage.
- `symbols`: the symbol list shows instances and textures ("14 instances · 9 textures").

## Impact

- **tp-core:**
  - usage queries over the whole project, across surfaces and symbol content, through style links: per swatch, per style, and per symbol (with textures);
  - `instance_count` exists, the others are new.
  - Pure and unit-tested.
- **tp-render:** small thumbnails of a surface rendered from a modified copy of the project (`Project` is cheap to clone), on a worker thread. The thumbnails of `new-design`'s Project screen are reused if they exist.
- **tp-app:**
  - `brand_ops.rs`: `SwatchEdit` no longer edits live. A pending value is kept and applied by one command;
  - a new style edit on the same model;
  - the editor UI in the Brand space, and `ui/workspace/panels/colors.rs` opening it;
  - strings in en/fr/es/de.
- **tp-file:** none. Nothing new is saved.
- **Performance:** counts are recomputed on project change, not every frame. Thumbnails are throttled while the picker is dragged, for fleets of 30–40 textures at 4096².
