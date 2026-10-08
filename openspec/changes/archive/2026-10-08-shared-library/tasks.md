## 1. Origins (tp-core, tp-file)

- [x] 1.1 Add `LibraryKey` and `origin: Option<LibraryKey>` to `Swatch`, `GraphicStyle`, `TextStyle` and `Symbol` (None in every constructor).
  - `Project::restore` keeps the current origin of each element that still exists (by id);
  - `Snapshot::same_document` ignores origins for the history, while the saved-state comparison sees them (add a `same_saved` or equivalent, as design: Origins).

  Verify with unit tests:
  - restoring an older snapshot keeps an origin recorded after it;
  - an origin change alone is not an undo-recordable change but marks the document changed for saving.
- [x] 1.2 Write and read the optional `origin` fields in `tp-file` v1 (swatches, styles, symbols). Verify in `tests/format.rs`: a round trip keeps origins; the v1 fixture opens with none and writes no `origin` key.

## 2. Import (tp-core `import`)

- [x] 2.1 Implement `closure(from, picks)`: symbols, styles, swatches (also through style looks) and assets reached from the picked elements and objects. Verify with unit tests: a symbol holding an image, a text on a text style and a rectangle linked to a swatch gives exactly those four dependencies; a picked graphic style brings the swatch its look links to.
- [x] 2.2 Implement `import(from, picks, into, mode)` following the design order (assets, swatches, styles, symbols, objects), with reuse by origin or by name and value, unique numbered names, remapped links, fresh object ids, then `relink` and `refresh_instances`. Verify with unit tests for each spec scenario:
  - a symbol and its dependencies imported into an empty project;
  - no duplicate "Vert Ardent" when an identical swatch exists;
  - "Logo" becomes "Logo 2";
  - reuse by origin;
  - pasted objects (an image, a linked text, an instance) remapped to the target's assets, swatches and symbol;
  - `ImportMode::Publish` replaces a reused-by-origin element's content and returns the keys to record;
  - importing twice adds nothing the second time.

## 3. Library storage (tp-file, tp-app)

- [x] 3.1 Split `v1::into_project` into `into_document` (no fleet check) and `into_project`. Add the `tp_file::library` container (`mimetype` `application/x-truckpaint-library`, `library.ron`, assets) with `to_bytes`, `from_bytes`, `write` (atomic) and `read`; drop unused assets before writing. Verify with tests: a library with a symbol, its image, a swatch and two styles round-trips; a project file is refused as a library and a library file as a project.
- [x] 3.2 Add `AppDirs::library()` and `LibraryStore` in tp-app:
  - lazy load;
  - an unreadable file is backed up to `.bak-<timestamp>`, the library starts empty, and the issue is reported once;
  - `publish(project, picks) -> keys` and `remove(key)`;
  - saving after each change through the background saver;
  - key generation (design: Origins).

  Verify with unit tests: a missing file gives an empty library; a damaged one is backed up and reported; publish then reload shows the entries; keys are unique across 10 000 draws.

## 4. Add to Library (tp-app UI)

- [x] 4.1 Add Add to Library / Update in Library to the context menus of the Symbols, Colors and Styles panels:
  - it publishes the element, records the returned origins in the project outside the history, and shows the hint "Added to the library" (or "Library updated").

  Verify with UI tests:
  - adding a symbol puts it and its dependencies in the library;
  - the menu then reads Update in Library, and updating replaces the library's copy;
  - the project shows "Unsaved changes" and no undo step was added;
  - Undo of an earlier edit keeps the origin.

## 5. Import from Library (tp-app UI)

- [x] 5.1 Add `CommandId::ImportFromLibrary` (Object menu, after Convert to Symbol; disabled while editing a symbol) and `Workspace::import_from_library` (one undo step `undo-import-from-library`). Verify with a unit test: importing a symbol with its dependencies is one undo step that Undo fully reverts.
- [x] 5.2 Build the dialog (`Modal::ImportFromLibrary`):
  - four groups with checkboxes and previews, the swatch colors and symbol thumbnails rendered on a worker;
  - "In this project" rows that can't be checked;
  - Import disabled when nothing is checked;
  - Remove from Library with confirmation;
  - the empty-library explanation;
  - the one-time message for an unreadable library.

  Verify with UI tests for the spec scenarios: import, "In this project", remove, empty library, unreadable library.
- [x] 5.3 Offer "Import from Library…" in the empty states of the Symbols, Colors and Styles panels. Verify with a UI test: the button opens the dialog.

## 6. Paste across projects (tp-app)

- [x] 6.1 Give each workspace a session id. Change the clipboard to `Clipboard { objects, source }` (the source session and project snapshot), and make Paste from another session import dependencies within the same `cmd-paste` step. Verify with UI tests:
  - an image and a text linked to "Vert Ardent" pasted into another project keep their look and link;
  - an instance pasted after closing its project brings its symbol;
  - one Undo removes the pasted objects and what came with them;
  - pasting in the same project is unchanged (existing tests);
  - copy, close and reopen the same project, then paste, adds no duplicate swatch.

## 7. Strings, docs and checks

- [x] 7.1 Add the strings in en/fr/es/de:
  - the menu items (Add to Library, Update in Library, Remove from Library);
  - the command, the dialog title, groups, "In this project", Import, the empty-library text and the confirmation;
  - the hints, the unreadable-library message, and the undo label.

  Verify with the i18n tests and the localization screen test.
- [x] 7.2 Update the docs:
  - `docs/roadmap.md`: record the library decisions (personal library, copies with origins, reuse rules, paste across projects) and remove the "symbol library across projects" open question;
  - `docs/roadmap_light.md`: mark the library and the paste bug as done.

  Verify by reading the diff.
- [x] 7.3 Add a `mise run screenshots` capture of the Import from Library dialog with a few elements. Verify that it renders.
- [x] 7.4 Run `mise run ci`. Verify that format, lint, tests and builds pass.
