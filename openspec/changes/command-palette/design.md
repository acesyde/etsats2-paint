## Context

See proposal.md for the motivation and the specs for the behavior (`command-palette`, and the deltas of `command-system`, `workspace-layout`, `vehicle-projects`).

What exists today:
- **Registry** (`tp-app/src/commands.rs`): `CommandId` (about 100 values once parameterized variants are expanded by `CommandId::all()`), `meta()` → `CommandMeta { label, icon, shortcuts, scope, availability }`. `Scope::Global` fires even while typing (Save, Export…, New, Open, Close, Preferences, Quit), `App` on any screen when not typing, `Workspace` with a project when not typing. `take_triggered` matches modifiers exactly (`modifiers_match`), so `Cmd+K` and `Cmd+Shift+K` never collide. `no_duplicate_default_shortcuts` checks `(command, shift, alt, key)` across every command. No command uses `K` today; Convert to Symbol has no shortcut.
- **Enabled state and reasons** (`state.rs`): `is_enabled(id, &EditContext)` and `disabled_reason_for(id, &EditContext)`; `CommandUi::label` names Undo/Redo's operation. `ShortcutFormatter` formats a shortcut for the OS and language.
- **Keyboard** (`AppState::handle_keyboard`): `typing` is true when a text field has focus, had it last frame, a modal is open, or a gradient marker claims the keyboard; then only `Global` shortcuts fire. Commands go to `state.queue` and run at the end of the frame through `dispatch`, which ignores disabled commands, commits a pending panel edit, ends a text session or finishes a pen path unless `keeps_text_session(id)`, and refuses Export Mod over a modal.
- **Modals** (`state.rs::Modal`, `ui/dialogs.rs::show_modal`): one `Option<Modal>` at a time, drawn with `egui::Modal` after the screen and before the queue runs.
- **Menus** (`ui/menu_bar.rs`): the structure is code only (`menu_contents` matches on the title and calls `item`/`menu_toggle`; two submenus, Combine and Align; separators; View's toggles computed from `EditContext`, `WorkspaceLayout` and `ViewAids`; Design Gallery in debug builds only). Tools and the three color commands appear in no menu, nor does Done Editing Symbol.
- **Textures**: `Workspace::set_active_surface(i)` (clears the selection, keeps each surface's view, leaves a symbol edit); the Project space sets `ws.space = Space::Workshop` then calls it. `panels/vehicle.rs` has `part_of`, `texture_name`, `state_label`, `state_colors` and the marker drawing of `texture_row`; `Surface::state()` is in tp-core. The breadcrumb's `Crumbs::of` builds "vehicle › group › texture".
- **Look** (`tp-ui/src/tokens.rs`): `color::CONTROL` is the popover surface, `BORDER_STRONG`, `SELECTED`, `TEXT_MUTED`, `TEXT_DISABLED`, `typography::MONO` (JetBrains Mono), `radius::CARD`, `size::HIT_MIN`; `icons::SEARCH`, `icons::CHECK`, `icons::WARNING`.

## Goals / Non-Goals

**Goals:**
- One source for the menu structure, so the palette's paths can't drift from the menus.
- A pure, unit-tested matcher and ranking (no egui) in tp-app.
- The palette reuses every existing path: `queue` + `dispatch` for commands, `set_active_surface` for textures, `is_enabled`/`disabled_reason_for`/`CommandUi::label` for state and labels, the Textures tab's marker for states.
- No new dependency, no file format change.

**Non-Goals:**
- Caching the folded labels or the results across frames (the palette is only drawn while open; about 170 short strings).
- Persisting recent commands (see Decision 6).
- A shortcut for Convert to Symbol.

## Decisions

### 1. Command Palette is a registered command, global, never over another modal
`CommandId::CommandPalette`, label `cmd-command-palette` ("Command Palette…"), icon `icons::SEARCH`, shortcut `sc(CMD, Key::K)`, `Scope::Global`, availability `When(|c| !c.gesture_active, "")`. Global is what makes it work while a field has focus and on the home screen (spec: Opening and closing). In `dispatch`:
- palette open and `id == CommandPalette` → close it (toggle);
- another modal open → ignore (as Export Mod does);
- otherwise open `Modal::CommandPalette(Palette::new(false, ""))`.
It is added to `keeps_text_session`, so opening it neither ends a text edit nor finishes a pen path (spec: The palette leaves the work unchanged). It is added to the `default_workspace_shortcuts` test table; `no_duplicate_default_shortcuts` already covers clashes. The Keyboard Shortcuts window lists it with no change, since it iterates `CommandId::all()`.
- *Alternative:* `Scope::App`. Rejected: the user asked for it to work while a text field has focus.
- *Alternative:* opening it over a dialog. Rejected: dialogs are modal and own the keyboard; their commands are mostly unavailable anyway.

### 2. The palette is a `Modal` variant
`Modal::CommandPalette(Palette)` where
```rust
pub struct Palette { query: String, textures_only: bool, highlight: usize, scroll_to_highlight: bool }
```
Being a modal gives the behavior the spec wants for free: `handle_keyboard` treats it as typing, so single-key and `Workspace` shortcuts don't fire (letters type, arrows don't nudge, Escape doesn't deselect, Space doesn't take the Hand tool), and nothing else can open over it. Global shortcuts still reach `dispatch`; `dispatch` closes the palette before running any command other than `CommandPalette` (spec: Save while the palette is open).
It is drawn by `ui/palette.rs::show(ctx, state, &mut Palette) -> bool` from `show_modal`, with the same take/put-back pattern as Export and Export Mod. It uses `egui::Modal` with a transparent backdrop (the canvas stays readable; a press outside closes and is eaten by the backdrop, so it doesn't select or draw) and a custom `Area` anchored at the top center, `y = TOP_BAR_HEIGHT + MENU_BAR_HEIGHT + space::XL`, width `min(560, screen − 2 × space::LG)`.
- *Alternative:* a separate `AppState.palette: Option<Palette>` drawn as a free `Area`. Rejected: it would need its own rules in `handle_keyboard` for every key the modal path already blocks.

### 3. Keys are consumed before the field sees them
At the start of `palette::show`, before the `TextEdit`, the palette consumes with `input_mut(consume_key)`: ArrowDown/ArrowUp (±1, skipping headings, clamped), PageDown/PageUp (± visible rows), Enter (choose), Escape (close), Tab and Shift+Tab (nothing; focus stays), and Backspace when the query is empty and `textures_only` (lift the limit). A single-line `TextEdit` would otherwise move its caret on Up/Down and drop the focus on Enter/Escape. The field calls `request_focus()` every frame. Home/End stay with the field. `highlight` resets to the first selectable row whenever the query changes, and `scroll_to_highlight` makes the highlighted row call `scroll_to_me` once after a key move (not on hover, so the list doesn't jump under the mouse).

### 4. The menu structure becomes data (`tp-app/src/menus.rs`)
```rust
pub enum Entry { Item(CommandId), Toggle(CommandId), Separator, Submenu { title: &'static str, min_width: f32, entries: Vec<Entry> } }
pub struct Menu { pub title: &'static str, pub entries: Vec<Entry> }
pub fn menus() -> Vec<Menu>                       // MENUS order; Design Gallery only under cfg!(debug_assertions)
pub fn is_checked(id: CommandId, edit: &EditContext, layout: Option<&WorkspaceLayout>, aids: ViewAids) -> bool
pub fn catalog() -> Vec<(CommandId, Vec<&'static str>)> // every paletted command in menu order with its path keys
```
`menu_bar.rs` keeps drawing (`MenuBar`, the logo, `menu_button`, `set_min_width`) but walks `menus()`; `Toggle` rows call `is_checked`, which holds the logic now inlined in the View arm (space, left tab, panels hidden, template visible, grid, guides, snapping). View gets `Item(CommandPalette)` and a `Separator` first.
`catalog()` flattens the menus depth-first, the path being `[menu title key, submenu title key…]`, then appends the commands outside the menus with a group key: `palette-group-tools` for `SelectTool(t)` in `Tool::ALL` order, `palette-group-colors` for SwapColorTarget, SwapFillStroke, DefaultColors, `palette-group-symbol` for FinishSymbol. `CommandId::in_palette()` is false for `Nudge(..)` and `CommandPalette`. A command appearing twice in the menus (none today) keeps its first path.
Unit tests: every `CommandId::all()` value with `in_palette()` appears exactly once in `catalog()` (so a new command can't be forgotten); every command of `menus()` is in `CommandId::all()`; `Align(Edge::Left)` has path `["menu-object", "menu-align"]`; DesignGallery is in the catalog iff debug.
- *Alternative:* a `menu_path` field in `CommandMeta`. Rejected: it would repeat the menu layout in a second place and still let the two drift; the order, separators and submenus would remain code.
- *Alternative:* record the paths by running `menu_contents` once with a recording `Ui`. Rejected: needs an egui context and can't be unit-tested simply.

### 5. Home-made matcher, accent folding by table (`tp-app/src/palette.rs`)
Pure functions, no egui:
- `fold(text) -> Folded { chars: Vec<char>, origin: Vec<usize> }`: lowercase, then a small table for the letters of fr/de/es (à â ä á ã å → a, ç → c, é è ê ë → e, í ì î ï → i, ñ → n, ó ò ô ö õ → o, ú ù û ü → u, ý ÿ → y, ß → ss, æ → ae, œ → oe), keeping for each folded char the index of its source char (so `ß` → `ss` maps both to one char) for highlighting.
- `score(words, label, context) -> Option<Match { score: i32, label_hits: Vec<usize> }>`: each word (query split on whitespace, folded) must match as a subsequence of the label, else of the context, else the entry is out. For a word, every occurrence of its first letter is tried as a start, the rest is matched greedily left to right, and the best alignment kept. Per matched letter: +1; +8 when it starts a word (after start, space, `›`, `-`, `/`, `(`); +4 when it follows the previous matched letter; −1 per skipped letter between matches (capped at −6 per gap). A word matched in the label counts double. Then −label length / 8 so that shorter labels win ties. The exact weights are set by the ranking tests, not by the spec.
- `rank(entries, query) -> Vec<usize>`: sort by `(disabled, Reverse(score), list index)`; with an empty query, the list of Decision 6.
Unit tests (`palette/tests.rs`): "flp hor" → Flip Horizontal first among all English labels; "horizontal flip" matches; case and accents ("TOUT SELECTIONNER" ↔ "Tout sélectionner", "strasse" ↔ "Straße", "oeuvre" ↔ "Œuvre"); context-only matches ("accessories chassis"); word-start beats inside-word; contiguous beats scattered; label beats context; shorter label wins a tie; ties keep list order; disabled after enabled; empty query order; the highlighted indices of "flp" in "Flip Horizontal" are 0, 1, 3; fold of "ß" maps back to one source char.
- *Alternative:* `nucleo-matcher` (Helix's matcher). Better scoring on long lists and built-in Unicode normalization, but: a new dependency for about 170 short entries; MPL-2.0 where every dependency today is MIT/Apache-style; and its scores are tuned for file paths, so our rules (label over context, enabled first, shorter label) would wrap it anyway. Can replace `score` later behind the same function if search grows to objects and swatches.
- *Alternative:* `unicode-normalization` (NFD then strip combining marks). Rejected for now: one more crate for the four Latin-script languages we ship; the table is explicit and tested. It doesn't fold `ß`/`œ` anyway.

### 6. Entries, groups and recent commands
Each frame the palette builds its entries: for each `catalog()` command, `label = CommandUi::label(id)` (Undo names the operation), `context = path keys translated and joined with " › "`, enabled from `is_enabled`, reason from `disabled_reason_for`, check from `is_checked` for `Toggle` entries, shortcut from `ShortcutFormatter::command`; with a workspace, for each surface `i`: label `texture_name`, context `<vehicle> › <part group>` (the breadcrumb's parts, shared through a small helper extracted from `Crumbs::of`), size, `Surface::state()`, active flag. Without a project, no textures. With `textures_only`, commands are skipped.
Empty query: **Recent** (`AppState.palette_recent`, filtered by `in_palette`), **Textures**, **Commands** (catalog order, recents left out); headings are rows that can't be highlighted. With a query: one ranked list without headings.
`palette_recent: Vec<CommandId>` lives in `AppState`, at most 5, most recent first, deduplicated, pushed only when the palette runs a command. Session only: `CommandId` has no stable serialized name today (tools and parameterized variants), and adding one to `prefs.ron` for a convenience was judged not worth a format addition; it can come later with the label key as id.

### 7. Choosing
- Command, enabled: close the palette (`state.modal = None`), push the id to `palette_recent`, push it to `state.queue`; it runs in the same frame through `dispatch`, so undo labels, text session and pen rules, Export Mod's modal guard and every effect are the menu's.
- Command, disabled: nothing; the footer already shows the reason.
- Texture `i`: close, then `ws.space = Space::Workshop; ws.set_active_surface(i)` (what the Project space does).
- Click on a row = Enter on it. Hover sets `highlight`.

### 8. Search textures field (Textures tab)
A field-looking button at the top of `panels/vehicle.rs::show`, above the list heading: `FIELD` fill, `BORDER`, `radius::MD`, `icons::SEARCH` and "Search textures" in `TEXT_MUTED`, the Command Palette shortcut in the mono font at the right end; focusable, named "Search textures" for assistive technologies. Click, or Enter while focused, or a `Text` event while focused, sets `env.palette_request = Some(PaletteOpen { textures_only: true, query })` (new `PanelEnv` field, backed by `AppState.palette_request`, handled after the frame like `vehicle_request`). The tree is never filtered by it.
- *Alternative:* a real text field filtering the tree in place. Rejected: two searches with two behaviors; the palette already searches textures, ranked, with states.

### 9. Look
Frame: `color::CONTROL`, 1 px `BORDER_STRONG`, `radius::CARD`, the popup shadow, inner margin `space::SM`. Field: 32 pt high, `FIELD` fill, `icons::SEARCH` in `TEXT_MUTED`, hint "Search commands and textures" (or "Search textures") in `TEXT_MUTED`, body size. Rows: 28 pt (≥ `HIT_MIN`), icon column 16 pt, context in `TEXT_MUTED` then the label in `TEXT_PRIMARY` with matched letters in the strong (SemiBold) style, so matches show by weight and not by color alone; at the right, the shortcut in `typography::MONO` `TEXT_MUTED`, a `CHECK` icon for an active toggle, or for a texture the size (mono, muted) and the state marker drawn by the Textures tab's helper (extracted from `texture_row` as `state_marker(ui, rect, state)`). The highlighted row has the `SELECTED` fill (no indicator bar, as for the other lists since the redesign; the lighter fill also reads in grayscale); the active texture shows a small dot before its name. Disabled rows use `TEXT_DISABLED` everywhere. Headings: `section_heading` (small capitals, muted). List: `ScrollArea` with `max_height = 10 rows`. Footer: hairline, then caption-size hints "↑↓ Move · ↵ Run · Esc Close" in `TEXT_MUTED`; when the highlighted command is disabled, its reason instead, in `TEXT_SECONDARY` without an icon, so it doesn't read as an error. Context, label and shortcut are laid out so the label truncates last: the context gets at most 45 % of the row and is cut with an ellipsis first, the full row text being the hover text and accessible name. Accessibility: the field is labeled "Search commands and textures"; rows give `WidgetInfo::selected(SelectableLabel, enabled, highlighted, name)` with name = "context › label, shortcut" plus ", unavailable: reason" or the texture state.

### 10. Strings (en/fr/de/es)
`cmd-command-palette`, `palette-placeholder`, `palette-placeholder-textures`, `palette-no-match` (`{ $query }`), `palette-heading-recent`, `palette-heading-textures`, `palette-heading-commands`, `palette-group-tools`, `palette-group-colors`, `palette-group-symbol`, `palette-hint-move`, `palette-hint-run`, `palette-hint-close`, `palette-unavailable` (`{ $reason }`, accessible name suffix), `textures-search` ("Search textures"). The existing `tp-i18n` test fails on a missing key; `every_message_id_in_the_code_exists` covers the code side.

## Risks / Trade-offs

- [Menu refactor changes the menu bar] → `menu_bar.rs` only changes how it walks the structure; the existing kittest menu tests (`tests/ui.rs`, `tests/workspace_frame.rs`, `tests/precision.rs`, the menu-driven tests of `align.rs`, `combine.rs`, `flip.rs`, `outlines.rs`, `canvas.rs`, and `german_menu_bar` in `tests/localization.rs`) must pass unchanged, and the View menu test gains Command Palette….
- [Scoring weights feel wrong on real use] → the weights live in one function covered by ranking tests built from the real English, French and German labels; adjusting them doesn't touch the spec.
- [Folding table misses a letter] → it covers every accented letter of the four shipped catalogs; a unit test folds every command label and menu title of the four languages and asserts that no letter outside a–z remains.
- [Keys leaking to the canvas while open] → the modal path already marks typing; kittest checks arrows, Enter, Escape, letters and Space with a selection under the palette.
- [Cmd+K with the macOS menu bar later (`title-bar-menus`)] → the native menu will dispatch the same `CommandPalette` command; nothing palette-specific to change.
- [egui `Modal` eats the press outside] → intended (spec: the press doesn't select); a press on the menu bar while the palette is open only closes it.
- [Recent commands lost on restart] → accepted for this change; noted in the proposal's non-goals.

## Migration Plan

None: no file, preferences or format change. Rollback is reverting the change; the menu data module can stay.
