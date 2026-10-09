## 1. Tokens, theme and fonts (no layout change yet)

- [x] 1.1 Rewrite `tp-ui/src/tokens.rs` around meanings (surfaces canvas/panel/raised, ink, `ACCENT_PRIMARY`, `SIGNAL`, `LINK`, success/warning/error, type scale) per design Decision 5, keeping the elevation names used by call sites; verify `cargo build --workspace` and that no screen uses a hard-coded color outside the tokens (grep `Color32::from_rgb` in `tp-app/src/ui`).
- [x] 1.2 Update `theme.rs` (widget visuals, selection, hover/pressed/focused/disabled states) to the new tokens; verify the existing UI tests in `crates/tp-app/tests/ui.rs` still pass.
- [x] 1.3 Extend `tp-ui/src/contrast.rs` to every pair of the ui-design-system delta ("Text contrast"), including dark text on the white pill and `SIGNAL` / `LINK` on each surface; verify the contrast test passes and fails when a token is darkened on purpose.
- [x] 1.4 Add Geist (Regular, Medium, SemiBold) and JetBrains Mono (Regular, Medium) with their OFL licenses to `assets/fonts/`, make them the Proportional and Monospace families in `fonts.rs`, keep Inter as a document font; verify with a kittest that a label renders with Geist and a numeric field with JetBrains Mono, and that text fonts of documents are unchanged.
- [x] 1.5 Add the segmented control (white pill, grayscale-visible active state) to `tp-ui/src/widgets/segmented.rs`; verify the "Segmented controls" scenarios of ui-design-system with a widget test, including the grayscale gallery render in `screenshots.rs`.

## 2. Commands and shortcuts

- [x] 2.1 Add `Space`, `LeftTab`, the commands `ShowSpace`, `ShowLeftTab`, `TogglePanels`, and remove `ToggleVehicles`, `TogglePanel`, `VehicleInfo` in `commands.rs`; verify `cargo test -p tp-app commands` and that every command still has a menu entry (existing registry test).
- [x] 2.2 Apply the shortcut table of design Decision 7 (spaces, tabs, Tab, Actual Size, Fit to Screen, Bring Forward / Send Backward, Next / Previous Texture with PageDown/Up kept, Show Template `G`, Gradient `Shift+G`, Export Mod `Cmd/Ctrl+E`, Export Texture `Shift+Cmd/Ctrl+E`, F5–F8 unbound); verify `no_duplicate_default_shortcuts` and a new test listing each key of the table against its command.
- [x] 2.3 Implement the focus rule for single keys (`Tab`, `1`, `2`, `3`, `G`: not while a text field has focus, while text is edited on the canvas, or with a modal open); verify the "Single-key workspace shortcuts respect focus" scenarios with kittest (typing "1" in a field types it; Tab in a dialog moves focus).
- [x] 2.4 Update the Keyboard Shortcuts window and the View / Vehicle menus (`ui/menu_bar.rs`) per workspace-layout "Menu bar"; verify the menu scenario (View lists spaces, tabs, Hide Panels, view aids, zoom, Reset Workspace; no Sidebar nor Vehicle Information).

## 3. Layout model and preferences

- [x] 3.1 Replace `layout::WorkspaceLayout` with `{ left_tab, left_width, inspector_width, panels_hidden, generation }` and drop `PanelKind`/`PanelSlot`; verify unit tests for defaults, clamped widths and Reset Workspace.
- [x] 3.2 Load preferences saved by earlier builds (panel stack, column width, sidebar fields) with the new defaults; verify with a fixture of today's preference file in `tests/persistence.rs` (app-preferences "Persisted preferences" scenarios).

## 4. Workspace frame and spaces

- [x] 4.1 Add `space` to the open workspace (`state.rs`): Project when a project is opened, created or recovered; switching keeps selection, history, zoom and active texture; verify the workspace-spaces scenarios "Space shown when a project opens" and "Switching spaces keeps the work" with kittest.
- [x] 4.2 Build the top bar (project name, game badge, breadcrumb in the Workshop, space switcher, Export… button) and the breadcrumb inlaid on the canvas, including `Symbol › <name>` while a symbol is edited; verify "Top bar", "Export button" and "Breadcrumb" scenarios.
- [x] 4.3 Turn the tool bar into the 48 px tool rail (icons only, tooltip with translated name and shortcut, active state not color-only); verify workspace-layout "Tool rail" scenarios.
- [x] 4.4 Add the tool options bar with the Polygon tool's Sides / Star / Inner radius (moved from Properties) and the tool name for other tools; verify workspace-layout "Tool options bar" and path-tools polygon scenarios.
- [x] 4.5 Rebuild the status bar: zoom, pointer position, template show / opacity / `G`, snapping, grid, guides, save state (text + icon); only the save state in Project and Brand; verify workspace-layout "Status bar" and vehicle-projects "Template overlay as a view setting".
- [x] 4.6 Lay out the Workshop: tool rail, left panel, canvas, inspector, both panels resizable within bounds, Focus mode with `Tab`; remove the right panel stack and the Vehicles sidebar; verify "Workspace frame", "Left panel", "Inspector", "Focus mode" and "Canvas area" scenarios.

## 5. Left panel tabs

- [x] 5.1 Textures tab from `panels/vehicle.rs`: vehicle → Main textures / Accessories → texture with real thumbnail, update button, ⋯ menu (Textures…, Update Template…, Remove from Project), Add Vehicle; verify vehicle-projects "Textures tab" and "Switching textures" scenarios.
- [x] 5.2 Layers tab from `panels/layers.rs` with the "<texture> · N layers" header and the add button; instances and styled objects in the link color with their icon; verify layers-panel and symbols "Instances in the editor" scenarios.
- [x] 5.3 Resources tab: Palette, Symbols, Styles and Images (compact lists from `colors.rs`, `symbols.rs`, `styles.rs`, `assets.rs`), drag to canvas, drop zone, Import from Library…; verify the "Left panel" Resources scenario, assets-panel "Asset list" / "Asset actions", and shared-library "Import from Library" entry points.

## 6. Inspector

- [x] 6.1 Add the popover widget to `tp-ui` (anchored under its row, Escape / click outside / opening another closes it, one at a time); verify with a widget test.
- [x] 6.2 Inspector with nothing selected: texture name, kind, size, "Layout changed" / "Not in this version" notices with their actions, hint, and the Fill / Stroke rows for new objects; verify properties-panel "Inspector sections" (nothing selected) scenarios.
- [x] 6.3 Inspector with a selection: header (name, kind, Convert to Symbol, Edit Symbol / Detach Instance for instances), Layout section from `transform.rs`, Text section from `character.rs`, Appearance (opacity, Fill row, Stroke row, corner radius, line settings of open paths), Polygon section, Style row, image information; verify the properties-panel, transform-panel, symbols "Detach Instance" and shared-styles "Style row and links in the inspector" scenarios.
- [x] 6.4 Color popover: Solid / Linear / Radial, gradient bar, picker and hex, brand palette before recent colors, swatch context menu (Edit Swatch…, Delete, Add to / Update in Library), X / Shift+X / D behavior; verify color-panel "Color popover", "Project palette", "Recent colors" and gradients scenarios.
- [x] 6.5 Stroke row and stroke popover (enable, width, alignment, dash, caps, joins, miter limit), summary "6 px · Outside"; verify stroke-panel scenarios.

## 7. Project space

- [x] 7.1 Project screen header and vehicle cards (thumbnail, kind and package version, cabins, read-only main texture mode, Update available with Update Template…, Textures…, ⋯ with Remove from Project) and texture rows (thumbnail, name, kind, size, notices); clicking a texture shows it in the Workshop; verify project-screen "Project space layout", "Vehicle cards", "Texture rows" and workspace-spaces "Opening a texture from the Project space".
- [x] 7.2 Mod information column (both pictures, Name, Author, Version, Description read only, Edit in Export Mod…) and the Game versions field moved from the sidebar with its hint; verify project-screen "Mod information column" and "Game versions field", reusing the scenarios of `tests/game_versions.rs`.

## 8. Brand space

- [x] 8.1 Brand space with its section index and the Palette, Graphic styles, Text styles, Symbols and Images sections, every action of today's panels, Import from Library… in the header; actions that place something show the Workshop; verify the brand-space scenarios and the symbols "Symbols lists" / shared-styles "Styles lists" scenarios.

## 9. Home, New Project and dialogs

- [x] 9.1 Home screen: recent projects as cards with a thumbnail of the first main texture (read in the background, placeholder until read or when unreadable), New Project primary, Open… secondary, recovered projects; verify start-screen "Home screen on launch" and "Recent projects list".
- [x] 9.2 New Project in one page: name and game on top, vehicle list (search, All / Trucks / Trailers, Custom vehicle…), "Your fleet" with main textures to tick, read-only main texture mode and the textures to be created; opens in the Project space; verify start-screen "New Project dialog" and custom-vehicles entry-point scenarios.
- [x] 9.3 Export Mod dialog: labels above fields, Advanced (internal name) collapsed and opened on its problem, problems before Cancel / Export…; opened by the top-bar button, `Cmd/Ctrl+E` and Edit in Export Mod…; verify mod-export "Export Mod dialog" and the existing `tests/mod_export.rs`.
- [x] 9.4 Vehicle Library: Installed / My vehicles tabs, rows with version and status, detail pane with template preview and the existing actions; verify vehicle-packages "Vehicle Library dialog" and `tests/vehicles.rs`.
- [x] 9.5 Restyle the remaining dialogs (Textures…, Custom vehicle…, Export Texture…, Library import, Preferences, crash recovery) with the new tokens and field layout; verify the existing tests of each dialog pass.

## 10. Languages

- [x] 10.1 Add and update every new or renamed string in `tp-i18n/locales/{en,fr,de,es}`; remove the strings of removed panels and commands; verify the localization test that fails on a missing key.
- [x] 10.2 Check that no label sits in a fixed-width box (buttons, tabs, segmented controls, inspector rows); verify `screenshots.rs` renders every space and dialog in German at 100 % and 200 % without clipped text (manual visual QA, noted in the PR).

## 11. Tests, QA and documentation

- [x] 11.1 Move the kittest tests that used panel headers, the sidebar, F5–F8 or old shortcuts (`tests/panels.rs`, `tests/vehicles.rs`, `tests/ui.rs`, others found by grep) to the spaces, tabs and inspector; verify `cargo test --workspace` passes.
- [x] 11.2 Update `tests/screenshots.rs` to render the home screen, New Project, the three spaces (Workshop with nothing selected and with a text selected), the Export Mod dialog and the Vehicle Library, in English and German; verify `cargo test -p tp-app --test screenshots -- --ignored` writes them and review them against the "TruckPaint 5" mockup.
- [x] 11.3 Run `cargo clippy --workspace --all-targets` and `cargo fmt --check`; verify both are clean.
- [x] 11.4 Update `docs/roadmap.md` when the change ships (redesign shipped, the Brand space / library question settled by design Decision 9, `new-design` moved to Shipped) and, at archive time, the Purpose lines of the gradients and layers-panel main specs that still name the Colors and Layers panels; verify `openspec validate new-design --strict` passes.
