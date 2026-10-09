## Context

See proposal.md for the motivation and the scope (constant functionality).

Today the workspace is built in `tp-app/src/ui/workspace/mod.rs` as egui panels: a menu bar on top, the status bar at the bottom, the tool bar and the Vehicles sidebar on the left, the canvas in the centre, and a resizable right column that stacks the eight panels of `layout::PanelKind`. Each panel body is a function in `ui/workspace/panels/*.rs`. The layout (`layout::WorkspaceLayout`: panel slots, column width, sidebar open and width) is saved in the preferences with `#[serde(default)]`, so unknown and missing fields are tolerated. Visual tokens live in `tp-ui/src/tokens.rs` and `theme.rs`, fonts in `tp-ui/src/fonts.rs` (Inter + Phosphor icons), and every command, with its label, icon, shortcuts and enabled rule, is declared in `tp-app/src/commands.rs`. Tests drive the UI with `egui_kittest` (`crates/tp-app/tests/*.rs`), and `tests/screenshots.rs` renders key screens for visual QA (ignored by default, needs a GPU).

The reference is the design spec "TruckPaint 5" (claude.ai design project *TruckPaint éditeur de livrées*, file `TruckPaint 5.dc.html`): nine artboards (first launch, New Project, Project, Workshop with nothing selected, Workshop with a text selected, Resources tab, Brand, Export, Vehicle Library). The first launch artboard belongs to `first-run` and is not built here.

## Goals / Non-Goals

**Goals:**
- One layout model for the three spaces, simple enough to test with `egui_kittest` and to persist.
- Reuse the existing panel bodies as sections (layers list, color picker, stroke options, transform fields, styles, symbols, assets, vehicle tree, properties) instead of rewriting their logic.
- Every command keeps a menu entry and, when it had one, a shortcut; only the keys listed below change.

**Non-Goals:**
- No new document data, file format or package change.
- No custom title bar or native menu bar (`title-bar-menus`), no command palette (`command-palette`).
- No new state shown on textures (`texture-status`), no inline editing of mod settings (`mod-settings-in-project`).
- No browser for the personal library (see Decision 9).

## Decisions

### 1. Spaces are a field of the open project, not separate screens
`Screen::Workspace` keeps one `Workspace` and gains `space: Space { Project, Workshop, Brand }`. The top bar and the status bar are shared; the central area is drawn by one function per space. Switching spaces keeps the selection, the undo history, the zoom and the active texture.
- *Alternative:* three `Screen` variants. Rejected: the project, its caches and its history would have to move between variants, and dialogs opened from one space (Export…, Library) must work in all three.
- Opening or creating a project shows **Project**; clicking a texture there sets it active and shows **Workshop**. Recovered projects open in Project too. The space is not persisted per project.

### 2. Frame of the window
```
+---------------------------------------------------------------------+
| menu bar (unchanged, in the window)                                 |
+---------------------------------------------------------------------+
| top bar 44: name + game badge | breadcrumb | spaces | Export…       |
+---------------------------------------------------------------------+
| tool options bar (Workshop only)                                    |
+----+------------+-------------------------------------+-------------+
|rail| left panel |  canvas (breadcrumb inlaid)         | inspector   |
| 48 | tabs 1/2/3 |                                     | (selection) |
+----+------------+-------------------------------------+-------------+
| status bar 28: zoom, x/y | template, opacity, G | snap grid guides | save |
+---------------------------------------------------------------------+
```
- Project and Brand use the full width under the top bar; the status bar shows only the save state there.
- Left panel and inspector are resizable within bounds (`tokens::size`), and **Tab** hides both (focus mode). The rail and the bars stay.
- The breadcrumb reads `Vehicle › Main textures|Accessories › Texture` with its size (`1024²`), as the Textures tab groups them; while a symbol is edited it reads `Symbol › <name>`. It is shown in the Workshop only.

### 3. Panel bodies become sections
The functions in `ui/workspace/panels/*.rs` are kept and called from new containers:
| Today | Container after the change |
|---|---|
| `vehicle.rs` (sidebar tree) | Textures tab, and the vehicle cards of the Project screen |
| `layers.rs` | Layers tab |
| palette list of `colors.rs`, `styles.rs`, `symbols.rs`, `assets.rs` | Resources tab (compact: Palette, Symbols, Styles, Images) and Brand space (full) |
| `properties.rs` | Inspector: texture properties with no selection; selection summary, opacity, corner radius, image information with a selection |
| `transform.rs` | Inspector › Layout (only with a selection) |
| `character.rs` | Inspector › Text (only when the selection holds text, or the Text tool is editing) |
| `colors.rs`, `stroke.rs`, `line_style.rs` | Inspector › Appearance: a Fill row and a Stroke row; each opens a popover |
| Polygon settings in Properties | Tool options bar of the Polygon tool (settings for new polygons), and an inspector Polygon section when every selected object is a polygon |
| Line width, dashes, caps and joins of open paths in Properties | Inspector › Appearance (Width field; the rest in the stroke-style popover) |

`PanelKind`, `PanelSlot` and the right column go away. The empty-state widgets stay for lists that can be empty (Layers of an empty texture, Resources with no symbol) but no container is ever an empty panel.

### 4. Popovers for colors and strokes
A small `tp-ui` popover (an `egui::Area` anchored under its row, closed by Escape, by a click outside, or by opening another one) hosts the existing color picker and stroke options. In the color popover: Solid / Linear / Radial as a segmented control, then the picker, then the **brand palette**, then the recent colors. Only one popover is open at a time; edits inside it are applied live and recorded as they are today (one undo step per committed edit).
- *Alternative:* keep an always-open Colors section. Rejected by the design ("plus de sélecteur de couleur ouvert en permanence").

### 5. Tokens and theme
`tokens.rs` is rewritten around meanings, keeping the existing elevation levels so most call sites only change names:
| Token | Value | Use |
|---|---|---|
| `SURFACE_0` canvas | `#121214` | pasteboard, canvas area |
| `SURFACE_1` panel | `#161618` | bars, panels, dialogs |
| `SURFACE_2` raised | `#232326` | fields, rows, popovers |
| `TEXT_PRIMARY` ink | `#EDEDED` | text |
| `ACCENT_PRIMARY` | `#ECECEC` | primary button, active segment, active tool |
| `SIGNAL` | `#F05252` | alerts, update available, selection on the canvas |
| `LINK` | `#50B9DF` (oklch 0.74 0.11 225) | linked swatch, styled object, symbol instance |

Success / warning / error keep their own tokens. The segmented control gets a white pill with dark text for the active option (a fill change, so it stays visible in grayscale). The existing contrast test (`tp-ui/src/contrast.rs`) is extended to the new pairs, including dark text on the white pill and `LINK` / `SIGNAL` on every surface.

### 6. Fonts
Geist (Regular, Medium, SemiBold) and JetBrains Mono (Regular, Medium) are added to `assets/fonts/` with their OFL licenses. Geist becomes the Proportional family, JetBrains Mono the Monospace family (numeric fields, paths, pixel values, hex codes). Inter stays in `assets/fonts/` as a document font. Type scale: title 22/600, heading 15/600, body and label 13/400, mono 12, caption 11.
- *Alternative:* subset the fonts to cut binary size. Deferred: about 1 MB, `distribution` can subset if needed.

### 7. Keyboard
| Command | Before | After |
|---|---|---|
| Project / Workshop / Brand | — | `Cmd/Ctrl+1` / `+2` / `+3` |
| Textures / Layers / Resources tab | F7 (Layers panel) | `1` / `2` / `3` |
| Hide Panels (focus mode) | — | `Tab` |
| Actual Size | `Cmd/Ctrl+1` | `Cmd/Ctrl+0` |
| Fit to Screen | `Cmd/Ctrl+0` | `Shift+Cmd/Ctrl+0` |
| Bring Forward / Send Backward | `Cmd/Ctrl+]` / `[` | `Alt+Cmd/Ctrl+]` / `[` |
| Next / Previous Texture | `Cmd/Ctrl+PageDown` / `PageUp` | `Cmd/Ctrl+]` / `[`, PageDown / PageUp kept |
| Show Template | `Shift+T` | `G` |
| Gradient tool | `G` | `Shift+G` |
| Export… (Export Mod) | `Shift+Cmd/Ctrl+E` | `Cmd/Ctrl+E` |
| Export Texture… | `Cmd/Ctrl+E` | `Shift+Cmd/Ctrl+E` |
| Sidebar | F5 | removed (Project space) |
| Colors / Properties panels | F6 / F8 | removed (Fill row, inspector) |

The Gradient tool takes `Shift+G` and not `D` as first discussed: `D` is already Default Colors. The single keys `Tab`, `1`, `2`, `3` and `G` only act when no text field has keyboard focus, no text is being edited on the canvas, and no modal dialog is open; otherwise egui's focus navigation gets Tab as today. `Tab` and `G` act in the Workshop only. `1`, `2`, `3` (and the View menu's tab items) from Project or Brand show the Workshop with that tab, and show the panels again when they are hidden. `no_duplicate_default_shortcuts` keeps guarding the table.

### 8. Commands and View menu
New commands: `ShowSpace(Space)`, `ShowLeftTab(LeftTab)`, `TogglePanels`. Removed: `ToggleVehicles`, `TogglePanel(PanelKind)`, `VehicleInfo`. The View menu lists the spaces, the three tabs, Hide Panels, then Template, Grid, Guides, Snapping and the zoom commands. Reset Workspace shows both panels at their default widths with the Textures tab. The menu bar keeps File, Edit, Object, Layer, View, Vehicle, Export, Help.

### 9. Brand space and the personal library
The Brand space shows the **project's** palette, graphic styles, text styles, symbols and images, with the actions their panels have today. The personal library stays reachable through **Import from Library…** (a button in the Brand header and in the Resources tab, and the Object menu) and the **Add to Library / Update in Library** context items. No library browser is added: that would be a new function. This settles the roadmap's open question for this change; a library view can be its own change later.

### 10. Preferences
`WorkspaceLayout` becomes `{ left_tab, left_width, inspector_width, panels_hidden, generation }`. With `#[serde(default)]`, preferences from earlier builds load: their `panels`, `column_width`, `vehicles_open` and `vehicles_width` are ignored and the defaults apply. Nothing in project files changes.

### 11. Screens
- **Home:** recent projects as cards with their thumbnail, New Project and Open… as primary and secondary buttons; recovered projects as today.
- **New Project:** still one vehicle, as today (more are added with Add Vehicle…). Left, the vehicle list (search, All / Trucks / Trailers, Custom vehicle…; no "Installed only" filter since every listed vehicle is installed); right, "Your fleet": the chosen vehicle, its main textures (cabins) to tick, the main texture mode as read-only text, and the textures that will be created ("2 main + 3 accessories"), with the total. The project name and the game are on top; Create Project is the primary action.
- **Project:** vehicle cards (thumbnail, package version, cabins, main texture mode, Update available with Update Template…, Textures…, ⋯ for Remove from Project), each listing its textures with their real thumbnail; right column "Mod information" read only (with **Edit in Export Mod…**) and the editable Game versions field with its supported-versions hint.
- **Export Mod dialog** and **Vehicle Library:** same fields and actions, laid out as in the mockup (fields labelled above, problems listed before the button, Advanced collapsed).

### 12. Tests
Kittest tests that click panel headers, the sidebar or F-keys are rewritten against the spaces, tabs and inspector. `screenshots.rs` renders each space and dialog at 100 % and 200 %, in English and German, for visual QA. New unit tests cover the layout model (defaults, old preferences), the shortcut table and the Tab/1/2/3 focus rule.

## Risks / Trade-offs

- [`SIGNAL` on `SURFACE_2` is 4.50:1, at the limit of the 4.5:1 rule] → The contrast test fails on any drift; red text is kept off raised surfaces where possible.
- [Muscle memory: eight shortcuts move] → The Keyboard Shortcuts window (`Cmd/Ctrl+/`) lists them. The change is noted in the release notes when `distribution` ships.
- [Popovers hide the canvas while editing a color] → The popover opens beside the inspector, never over the selection's bounds when there is room.
- [German strings overflow buttons and tabs] → Free-width buttons and labels above fields; `screenshots.rs` renders German at 200 % for review, and the localization test checks that no new string is missing in any language.
- [Large UI diff, many tests to move] → Implementation goes in the order of `tasks.md`: tokens and fonts first (no layout change), then the frame and spaces, then sections one by one, keeping the app usable after each group.
- [Binary size grows by about 1 MB with the fonts] → Accepted; subsetting is left to `distribution`.

## Migration Plan

No data migration: project files, packages and the library are unchanged. Preferences from earlier builds load with the new layout defaults (Decision 10). No rollback beyond reverting the change.

## Open Questions

- Exact icon choices for the tool rail and the inspector rows (Phosphor has equivalents for all of them); settled while implementing, without changing behavior.
