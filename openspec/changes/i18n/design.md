## Context

**Strings.** Every user-facing string is a hard-coded English literal. There are about 500 distinct strings in tp-app (menus, `commands.rs`, panels, dialogs, home screen, status bar, canvas hints, export dialog) and a handful in tp-ui widgets ("Mixed", gradient stop names). Specifically:
- Command metadata (`CommandMeta.label`, `Availability::When/NotYet` reasons) and undo labels (`History` entries, `live_edit(label)`) are `&'static str`.
- Many headless tests find widgets by their English accessible name (`get_by_label("Linear gradient")`) and assert undo labels (`undo_label() == Some("Change Fill")`).

**Other relevant code:**
- `tp_ui::widgets::parse_number` already accepts a comma.
- `home.rs::relative_time` builds "N days ago" in English.
- `commands::format_shortcut` builds Windows/Linux and macOS notations.
- `Prefs` is `#[serde(default)]`, so new fields load from old files.
- The UI runs on the main thread. kittests run each test on its own thread, in parallel.

**User decisions:**
- system language by default, with a Preferences override;
- immediate switching;
- localized decimal comma, accepting both separators;
- Fluent files.

## Goals / Non-Goals

**Goals:**
- Every interface text available in en/fr/es/de, switchable live, with English as source and fallback.
- A translation format with real plural rules, embedded in the binary, and checked by tests for completeness.
- Tests stay deterministic, in English, whatever the machine's locale, and in parallel.

**Non-Goals:**
- Right-to-left scripts, CJK, other languages, or user-provided translation files.
- Translating file contents, logs or documentation.
- Thousands separators, localized dates beyond relative times, and localized units.

## Decisions

### D1. A `tp-i18n` crate on Fluent
New crate `crates/tp-i18n` with `fluent-bundle` and `unic-langid`.
- `locales/{en,fr,es,de}/{common,menus,commands,panels,dialogs,canvas,objects}.ftl` are embedded with `include_str!`.
- `pub enum Language { English, French, Spanish, German }` provides `ALL`, `native_name()` ("Français"…), `code()` and `from_locale(&str) -> Language`. `from_locale` reads the primary subtag (fr-CA → French) and falls back to English.
- One `FluentBundle` per language is built lazily, with `set_use_isolating(false)`. Fluent's bidi isolation marks would otherwise show as boxes in egui.
- **API:**
  - `tr(id) -> Cow<'static, str>`;
  - `tr_args(id, &FluentArgs) -> String`;
  - a `tr!(id, name = value, …)` macro.

  Each looks the message up in the current language, then in English. If both are missing (prevented by tests), it returns the id and logs a warning once.
- **Caching:** messages without arguments are cached per (language, id) as `Arc<str>`, so drawing a frame does not re-format the ~200 static labels.
- **Purity:** tp-i18n has no UI and no filesystem access. tp-core does not depend on it.

*Alternatives:*
- `rust-i18n` / YAML: simpler, but plurals are by hand.
- gettext with source-string keys: less churn in code, but `.po` tooling is heavier and plural handling is weaker for our case.

### D2. Current language: thread-local, set per frame
`tp_i18n::set_language(Language)` stores the language in a **thread-local**, and `current()` reads it.
- `AppState::show` sets it at the start of every frame from `prefs.language.unwrap_or(system_language)`.
- **Tests:** kittests run tests in parallel threads. A thread-local keeps a French test from leaking into an English one, and a thread defaults to English when nothing is set.
- **Background threads:** export and save workers don't format text. They return error values that the UI thread formats.

### D3. Message ids replace literals
- **Naming:** Fluent ids are kebab-case and grouped by area: `menu-file`, `cmd-undo`, `cmd-undo-reason-nothing`, `panel-colors-hex`, `undo-change-fill`, `hint-shapes-dont-overlap`, `object-rectangle`.
- **Command labels and reasons:** `CommandMeta.label`, `Availability` reasons and `NEEDS_SELECTION` and similar become ids. `CommandUi`, menus and tooltips translate at display time.
- **Undo labels:** everything stored in `History` (`edit`, `live_edit`, `record` labels) stays a `&'static str`, now an id such as `undo-change-fill`. The Edit menu shows `tr!("cmd-undo-named", action = tr(label))`. The history keeps ids, so switching language re-labels existing entries.
- **Accessibility names** come from the same ids, so in English they are unchanged. Tests that find widgets by English names keep working. Tests that assert undo labels compare ids.
- **tp-ui** widget strings use tp-i18n directly (tp-ui gains the dependency).

### D4. System language and the preference
- `Prefs.language: Option<Language>`, serialized as the language code. `None` means System default.
- `main.rs` reads `sys_locale::get_locale()`, maps it with `Language::from_locale`, and passes it to `AppState` as `system_language`.
- `AppState::with_prefs` (tests) uses English. A missing or unreadable locale also gives English.
- **Preferences dialog:** a Language combo box with "System default" (translated) then the four native names. Picking one sets `prefs.language`, marks the preferences changed, and takes effect on the next frame. egui redraws immediately, so in practice it is the same frame.
- **Reset to defaults** leaves `language` alone.

### D5. Numbers
- `tp_i18n::decimal_separator()` returns `.` or `,`. `tp_i18n::format_number(v, decimals)` formats with `{:.N}`, trims trailing zeros as `NumericField` does today, and swaps the separator.
- **Users:** `NumericField` display, and value labels such as the size pill ("400 × 100 px"), angle pills, the export dialog sizes, and Preferences percentages.
- **Unchanged:** `parse_number` already maps `,` to `.`. Integers, hex colors and status bar zoom are unaffected. The zoom is shown as a whole percentage, so it has no decimals.

### D6. Plurals and relative times
`relative_time` returns `tr!("time-ago-minutes", count = n)`. Plurals are handled with Fluent selectors and per-language CLDR plural rules. For example, French treats 0 and 1 as `one` ("il y a 0 minute" is never shown, since "just now" covers it).

Other counts ("{ $count } objects selected", "Delete { $count } guides") use the same pattern.

### D7. Shortcut notation
`format_shortcut` takes its modifier names from `key-ctrl`, `key-shift` and `key-alt` (and key names like `key-space`, `key-delete`, `key-backspace`). macOS symbols are untouched.

### D8. Default names of new objects
- tp-core keeps `ShapeKind::name()` in English for logs and tests.
- tp-app gets `object_name(kind)` from `object-rectangle`, `object-ellipse` and the like, and every place that creates a named object uses it: `create_shape`, `create_line`, paths, polygon/star, text, image placement, group, `new_layer` (`object-layer-n` with `$n`), new project ("Untitled"), and outlines.
- **New Layer numbering** counts existing layers whose names match the *current* language's pattern. Names from another language are left alone, so numbering restarts at 1 in a new language. That is acceptable, since names are user data.

### D9. Translations and their checks
- **Source and drafts:** English `.ftl` is the source. French, Spanish and German are drafted in this change for the user to review.
- **Glossary** (kept consistent across files):

  | English | French | Spanish | German |
  |---|---|---|---|
  | Fill | Fond | Relleno | Fläche |
  | Stroke | Contour | Trazo | Kontur |
  | Layer | Calque | Capa | Ebene |
  | Group | Groupe | Grupo | Gruppe |
  | Gradient | Dégradé | Degradado | Verlauf |
  | Artboard / Texture | Texture | Textura | Textur |
  | Guide | Repère | Guía | Hilfslinie |
  | Undo | Annuler | Deshacer | Rückgängig |
  | Redo | Rétablir | Rehacer | Wiederholen |

- **Unit tests in tp-i18n:**
  1. every id in `en` exists in `fr`, `es` and `de`, and no language has extra ids;
  2. each message uses the same set of variables in every language;
  3. every bundle parses without errors;
  4. formatting each message with sample arguments yields no Fluent errors.
- **A tp-app test** makes sure no id used in code is missing from English. Ids are collected by a small `tr!` registry in debug builds, exercised by rendering every panel and dialog.

### D10. Layout with longer text
German and French strings run about 30% longer. Fixed widths (segmented controls, field labels, menu widths, the Preferences grid) are checked by rendering screenshots of the main screens in all four languages at 100% and 200%. Overflowing places get wrapping or a min width rather than shortened translations.

### D11. Notes from implementation
- **Not translated:** the Design System Gallery window is a developer tool and stays in English, though its menu command is translated. Messages printed to the terminal before any window exists (graphics adapter failures in `main.rs`) also stay in English, like logs.
- **Raw-id check:** the debug id registry of D9 became `tests/localization.rs::no_screen_shows_a_raw_message_id`. It renders the home screen, the workspace with every panel, every menu and every dialog in each language, and fails if an accessible name or label looks like a message id. A missing English message shows its id, so this check catches it.
- **Undo labels:** `Workspace::record` and `live_edit` `debug_assert` that the label is an existing message id.
- **Combine problems:** tp-core's `operand_problem` returns an `OperandProblem` enum instead of English text, and tp-app maps it to message ids. tp-core stays free of i18n.
- **Error messages:** `ImportError` keeps a reason id plus an optional system detail, formatted when shown. tp-file's `Error::message` stays English for logs, and tp-app formats user messages with `project_io::file_error`.
- **Hints:** canvas hints are formatted when they are raised, so one that is visible while the language changes stays in the old language until it fades.
- **German undo items:** these read "Rückgängig: { action }", since German menus put the action after a colon.
- **Widget changes:** `NumericField` widens itself to fit the "Mixed" hint ("Gemischt"), and Preferences sliders format values with the current decimal separator.

## Risks / Trade-offs

- **Churn:** about 500 call sites and a few hundred test assertions on undo labels. The edits are mechanical, and English accessible names are unchanged, so most UI tests don't move.
- **Translation quality:** the drafts are machine-assisted, and the user reviews them, French first. The glossary keeps terms consistent, and the files are easy to edit later.
- **Thread-local language:** code running off the UI thread would see English. That is acceptable, since workers return values formatted on the UI thread, and a test covers an export error message in French.
- **Per-frame cost:** cached static messages are a hash lookup. Formatted messages with arguments are few per frame.
- **Layout regressions in long languages:** caught by the four-language screenshot review (D10).
