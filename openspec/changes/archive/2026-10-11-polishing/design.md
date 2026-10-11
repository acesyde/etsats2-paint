## Context

The fixes come from a screenshot audit (`cargo test -p tp-app --release --test screenshots -- --ignored`), so most of them are local to one screen. A few share a cause and get one shared fix:

- **Default names.** `tp_core::document::Object::new` names objects with the English `ShapeKind::name()`, while the app names new objects with `crate::workspace::object_name(kind)`, the kind in the current language. The inspector's `display_name` (`ui/workspace/inspector.rs:374`) and the Layers panel (`panels/layers.rs:287`) compare the name with `object_name(kind)` only. A document made in English and opened in French, or the long-text mode, misses the match. The localization spec keeps stored names unchanged when the language changes, so the fix must be in how names are read, not in renaming.
- **Text cut without "…".** Several places paint text in a clip rect (`layers.rs:313`), pass a fixed max width larger than the room left (`spaces/project.rs:272-285`), or use `.wrap()` where one line is wanted (`panels/vehicle.rs:426`, `panels/assets.rs:240`).
- **Checkboxes.** `library_dialog.rs:317` and `export_dialog.rs:297` use egui's `Checkbox`, which the theme's corner radius turns into a circle. New Project and Add Vehicle already draw their own square check.
- **Target mark.** `paint_rows` (`inspector.rs:511`) marks the row of `ws.panels.color_target` whatever popover is open.

## Goals / Non-Goals

**Goals:**
- Every item listed in the proposal fixed, with screenshots showing it.
- Shared causes fixed once, in a helper reused by every place, so the next screens don't repeat them.

**Non-Goals:**
- No new feature, no change to project files, packages or the mod export.
- No general review of English casing beyond the pairs listed; the rule is written down for later changes.
- The items listed as left out in the proposal.

## Decisions

**1. A default name is the kind's name in any language.** tp-i18n gets `tr_all(id) -> Vec<String>`: the message in each supported language, plus the current one as shown (lengthened in the long-text mode). `workspace::is_default_name(kind, name)` is true when `name` is `kind.name()` (the core's English name) or one of `tr_all` of the kind's key. `display_name` and the Layers row use it. A text whose name is default shows its first non-empty line, and its kind is shown in small type. A shape whose name is default shows its name once, with no kind label. Alternative rejected: renaming defaults on load or on language change. The localization spec forbids it, and it would dirty the document.

**2. One truncated label helper.** `tp_ui::widgets::truncated_label(ui, text, width) -> Response` lays the text out on one line with `egui::Label::truncate()`, inside a width given by the caller and capped at `ui.available_width()`. A truncated text gets a tooltip with the full text (egui's truncate already does this; check and keep it). The Layers row stops painting the name in a clip rect and lays it out truncated to `name_rect` instead, with a tooltip when it was shortened. The Project space's vehicle card blocks cap their max width at the space left, and skip a block when less than 40 px is left.

**3. One checkbox widget.** The square check of New Project and Add Vehicle moves to `tp_ui::widgets` (if it isn't already there) and replaces egui's `Checkbox` in Import from Library and Export Texture. A test grep or a kittest test makes sure `egui::Checkbox` is no longer used in `tp-app`.

**4. Target mark only while no other popover is open.** `paint_rows` passes `.target(target == X && !other_popover_open)`, where `other_popover_open` is whether the stroke, line or shadow popover is open (`Popover::is_open`). The color target itself doesn't change, so swatches still apply to it when the popovers close.

**5. Symbol header.** `nothing_selected` in `inspector.rs` checks `ws.is_editing_symbol()` first. It shows the symbol's name, the kind label "Symbol" (a new key, or the existing symbol kind key if there is one) and the symbol's size with the same `size_label` as textures.

**6. Version ranges in words.** A function in tp-app formats a `semver::VersionReq` for display. `*` gives "any version"; a single `>=X` gives "X or later"; `<=X` "X or earlier"; `=X` "X"; `>=X, <Y` "X to before Y"; anything else falls back to the raw text. Each form is an i18n message with the version as argument, and versions are shown as `major.minor` when the patch is 0. Used by the Project space (`panels/vehicle.rs:252`) and the Vehicle Library details.

**7. Custom Vehicle rows.** Each texture row is laid out as a fixed grid: file name, role combo (fixed width), width and height combos (fixed width), Replace… and remove. The short labels "Role", "Width" and "Height" go above the combos, following the localization spec ("labels above their field"). The long "… of texture N" strings stay as the widgets' accessible names, so the existing tests keep finding them. The dialog keeps `set_width(680)`, and a test checks that its rect fits in the window.

**8. German filter at 960 px.** In the Project space header, the subtitle and the All / To do / To check filter share one row. When both don't fit, the subtitle is truncated (decision 2) before the filter, which keeps its whole width.

**9. Screenshots.** `status_*_before_exporting` scrolls the Mod information column to Before exporting before capturing. `palette_*_narrow` uses a 960 px window. Each fix that changes a screen is checked on its screenshot; new screenshots are added only where no existing one shows the fix (the Custom Vehicle dialog with long texts, the Layers panel with text layers).

## Risks / Trade-offs

- [Many small edits across the UI can conflict with each other] → Group the tasks by file so one agent owns a file at a time; the i18n files are edited in one task per group, not concurrently.
- [`tr_all` builds every language's bundle on first use] → It is cached like `tr`, and four small bundles cost little; it is only called for names equal to some default, after a cheap check against the current language first.
- [Changing English labels breaks kittest tests that look widgets up by label] → Each wording task updates its tests in the same task; accessible names of the Custom Vehicle rows stay unchanged on purpose.
- [A checkbox restyle changes many screenshots] → Only two places use egui's checkbox; the others already use the square check.
