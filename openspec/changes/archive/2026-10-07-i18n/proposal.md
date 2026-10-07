## Why

TruckPaint's community of ETS2/ATS livery makers is largely European, and every interface string is currently hard-coded in English. Translating the interface into French, Spanish and German makes the editor usable by most of that audience in their own language. Doing it now, while the UI is about 500 strings, is far cheaper than after vehicle templates, mod export and distribution add more.

## What Changes

- **Four interface languages.** English (international, the source and fallback) plus French, Spanish and German. This covers every user-facing text:
  - menus, command labels and tooltips, including why a command is disabled;
  - panels, dialogs, the home screen, the status bar and canvas hints;
  - error messages, undo/redo labels, accessibility names;
  - file dialog filter names.
- **Choosing the language.**
  - On first launch the language follows the operating system's (any French, Spanish or German variant maps to that language; anything else gives English).
  - A **Language** setting in Preferences overrides it, is saved with the preferences, and applies **immediately** without a restart.
- **Numbers follow the language.**
  - Numeric fields and value labels show a decimal comma in French, Spanish and German (`12,5`) and a decimal point in English.
  - Typing either separator is accepted in every language.
- **Words with grammar.** Plurals and counts ("2 days ago", "3 objects selected") are written per language, using Fluent translation files embedded in the application.
- **Keyboard shortcut notation.** On Windows and Linux, modifier names are localized (German `Strg+Umschalt+Z`, French `Ctrl+Maj+Z`, Spanish `Ctrl+Mayús+Z`). The macOS symbols (`⇧⌘Z`) are unchanged.
- **New object names.** Default names given to new objects, layers and projects ("Rectangle", "Layer 1", "Untitled") use the current language at creation. Names already in a document are user data and are never translated.

Not in this change: right-to-left languages, other languages, translating file contents (project files, exported mods, logs), and localized documentation.

## Capabilities

### New Capabilities
- `localization`: the supported languages, how the language is chosen and switched, the fallback to English, what is and is not translated, localized number display and input, plural forms, and the localized names of new objects and shortcuts.

### Modified Capabilities
- `app-preferences`:
  - the persisted preferences include the interface language;
  - the Preferences dialog gains the Language setting, which applies immediately.

## Impact

- **New crate `tp-i18n`:** `Language`, Fluent bundles (`fluent-bundle`, `unic-langid`) built from embedded `locales/{en,fr,es,de}/*.ftl`, the current language, `tr` lookups with arguments, and number formatting. It has no UI and no filesystem access, so tp-core stays untouched.
- **tp-ui:** its own few strings ("Mixed", gradient stop names, …) go through tp-i18n. `NumericField` formats with the current decimal separator, and `parse_number` already accepts both.
- **tp-app:**
  - every hard-coded string becomes a message key: `commands.rs` labels and reasons, menus, panels, dialogs, `home.rs` relative times, status bar, hints, export dialog;
  - undo labels stay stable keys stored in the history and are translated when shown;
  - `Prefs` gains `language: Option<Language>`, a field with a serde default, so older preference files still load;
  - `main.rs` detects the system locale (`sys-locale`).
- **Tests:** headless tests keep running in English whatever the machine's locale. New tests check:
  - every key exists in every language and every placeholder is used;
  - switching language live;
  - decimal-comma display and input.
- **Dependencies:** `fluent-bundle`, `unic-langid`, `sys-locale`. Inter already covers the accented letters these languages need.
