## 1. tp-i18n crate

- [x] 1.1 Create `crates/tp-i18n` (workspace member; `fluent-bundle`, `unic-langid`) with `Language` (`ALL`, `native_name`, `code`, `from_locale`). Verify with unit tests: fr-CA → French, de-AT → German, es-MX → Spanish, it-IT and garbage → English.
- [x] 1.2 Add embedded bundles with isolation off, a thread-local `set_language`/`current`, `tr`, `tr_args` and `tr!`, English fallback, an id fallback with a one-time warning, and the cache for messages without arguments. Verify with unit tests: lookup, fallback to English, thread isolation (two threads, two languages), and no bidi marks in output.
- [x] 1.3 Add `decimal_separator` and `format_number`. Verify with unit tests: `12.5` gives "12.5" in English and "12,5" in French, German and Spanish, and trailing zeros are trimmed as `NumericField` does today.
- [x] 1.4 Add a completeness test over the `.ftl` files: same ids in every language, same variables per message, every file parses, and every message formats with sample arguments without errors. Verify with `cargo test -p tp-i18n`.

## 2. English source strings (no behavior change)

- [x] 2.1 Move every tp-ui widget string to ids (`common.ftl`), and make `NumericField` display numbers with `format_number`. Verify that `cargo test -p tp-ui` passes unchanged.
- [x] 2.2 Turn command labels, availability reasons and shortcut key names into ids (`commands.ftl`, `menus.ftl`), translated in `CommandUi`, menus, tool bar tooltips and the Keyboard Shortcuts dialog, with `format_shortcut` using `key-*` names. Verify that `mise run test` passes with the UI unchanged in English.
- [x] 2.3 Turn all undo labels into `undo-*` ids, translated in the Edit menu and the status of undo/redo, and update test assertions to the ids. Verify with `mise run test`.
- [x] 2.4 Move panels, dialogs, the home screen (with `relative_time` plurals), the status bar, canvas hints and errors, the export dialog and file dialog filters to ids (`panels.ftl`, `dialogs.ftl`, `canvas.ftl`). Verify that `mise run test` passes, and that the debug id registry test finds no id missing from English after rendering every screen, panel and dialog.
- [x] 2.5 Add `object_name` and translated default names for new shapes, lines, paths, polygons, stars, texts, images, groups, layers (with current-language numbering), new projects and outlines (`objects.ftl`). Verify with unit tests in `workspace.rs` for each kind and for layer numbering.

## 3. Language choice

- [x] 3.1 Add `Prefs.language: Option<Language>` (code string, serde default), plus `AppState.system_language` set from `sys-locale` in `main.rs` (English in `with_prefs`), and `set_language` at the start of each frame. Verify with unit tests: old preference files load with `None`, and a saved language round-trips.
- [x] 3.2 Add the Language combo box to Preferences ("System default" + native names), which applies immediately and is left alone by Reset to defaults. Verify with kittests:
  - "Live switch": the open dialog, menu bar and panels switch, and unsaved changes are kept;
  - "Undo label follows the language";
  - "Back to the system language";
  - "Language survives restart".

## 4. Translations

- [x] 4.1 Write the French `.ftl` files following the D9 glossary. Verify that the completeness test passes for French.
- [x] 4.2 Write the Spanish `.ftl` files. Verify that the completeness test passes for Spanish.
- [x] 4.3 Write the German `.ftl` files. Verify that the completeness test passes for German.
- [x] 4.4 Kittest the spec scenarios in a new `tests/localization.rs`:
  - German menu bar;
  - French fallback with a test-only missing id;
  - user names kept;
  - French decimal comma display and German point input;
  - Spanish and French relative times;
  - German shortcut notation (format function with `is_mac = false`);
  - French new layer name;
  - existing names kept after a switch.

  Verify with `cargo test -p tp-app --test localization`.

## 5. Layout and verification

- [x] 5.1 Add a `render_languages` screenshot test (home screen, workspace with all panels, Preferences, export dialog) in each language at 100% and 200%. Review the images, and fix overflowing widths with wrapping or min widths. Verify with `mise run screenshots` and a visual check of the images.
- [x] 5.2 Run `mise run fmt:check`, `mise run lint` and `mise run test`, plus the stress run (4 parallel × 5 rounds of panels, localization, gradients and vector_tools), and confirm everything passes. Measure the frame time while panning (`text_images` benchmark) in French against English and confirm no regression above 5%.
