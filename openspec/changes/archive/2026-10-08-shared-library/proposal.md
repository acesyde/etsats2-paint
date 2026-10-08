## Why

A company's identity (its logo, lettering, stripes, colors, styles) is the same on every fleet a player designs, and often in both games. Today it can't leave the project it was made in. A player with an ETS2 fleet and an ATS fleet redraws the logo and re-enters "Company red" by hand, and the two drift apart.

Copy/paste between projects looks like a way around this, but it is broken. The clipboard keeps objects whose images, swatch and style links, and symbol references are ids that only mean something in the project they came from. Pasted into another project, an image can be missing or show another asset, a fill can link to an unrelated swatch, and an instance can point to no symbol.

## What Changes

- **A personal library.** One library per user, stored in the application's data folder, shared by every project of both games. It holds symbols, swatches, graphic styles and text styles, with the images they use.
- **Add to Library:** a context-menu item on a symbol (Symbols panel), a swatch (Colors panel) and a style (Styles panel). The element goes into the library with everything it uses: a symbol brings its images, the swatches its fills and strokes link to, and the styles its objects follow. When the element is already in the library, the item reads **Update in Library** and replaces the library's copy.
- **Import from Library…:** a command in the Object menu, also offered in the empty Symbols, Colors and Styles panels. It opens a dialog that lists the library's symbols, swatches, graphic styles and text styles, to check and import as one undo step. Elements already in the project are shown as such and can't be checked. **Remove from Library** in the same dialog deletes a library entry.
- **Import brings dependencies, without duplicates.** Each element remembers which library entry it came from. Importing or pasting reuses what the project already has: the same library entry, or an identical swatch or style (same name and same value). It never adds "Company red 2" next to "Company red". A new element whose name is taken by a different one gets a numbered name ("Logo 2").
- **Copies, not links:** editing the library doesn't change projects, and editing a project doesn't change the library until **Update in Library**.
- **Paste into another project works.** Objects copied in one project and pasted into another bring their images, swatches, styles and symbols with them, following the same reuse rules. Pasting in the same project is unchanged.
- **Project files** record the library origin of symbols, swatches and styles, as an optional field (format 1). Older files open with none.
- **Docs:** `docs/roadmap.md` records the decision and moves the "symbol library across projects" open question to shipped. `docs/roadmap_light.md` marks the library and the paste bug as done.

Non-goals:
- live links, or updating a project from the library ("Update from Library"), which may come later;
- several libraries, and importing or exporting a library file to share it;
- the Brand tab of the planned redesign;
- usage counts.

## Capabilities

### New Capabilities
- `shared-library`: the personal library, where it is stored and how it loads, Add / Update in Library, the Import from Library dialog, Remove from Library, and the import rules (dependencies, reuse by origin or identity, unique names, one undo step).

### Modified Capabilities
- `selection-transform`: Paste into another project brings the objects' dependencies, following the library's import rules.
- `project-files`: symbols, swatches and styles record their library origin.

## Impact

- **tp-core:**
  - `LibraryKey` and an `origin: Option<LibraryKey>` on `Swatch`, `GraphicStyle`, `TextStyle` and `Symbol`;
  - a new `import` module: the closure of a set of objects or elements (symbols, swatches, styles, assets), and their copy into another project with new ids, remapped links, reuse and unique names. Pure and unit-tested.
- **tp-file:**
  - the optional `origin` fields in format 1;
  - a library container, `library.tplib`: a ZIP holding `library.ron` and its assets, written atomically.
- **tp-app:**
  - `library.rs`: load, save and the library's operations;
  - `AppDirs::library()`;
  - the context-menu items in the Symbols, Colors and Styles panels;
  - the `ImportFromLibrary` command and its dialog;
  - a clipboard that keeps its source project, and Paste importing dependencies across projects;
  - strings in en/fr/es/de.
- **Dependencies:** none (keys are random 128-bit hex strings; `blake3` and the existing ZIP code already cover hashing and storage).
