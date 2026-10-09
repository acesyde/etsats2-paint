## Why

The feature set is in place (fleet projects, paint jobs, brand kit, symbols, shared library, mod export), but the interface grew one panel at a time: a stack of eight right-hand panels, a sidebar mixing project settings and navigation, empty-state panels and a color picker that is always open. Before other players use the app (`polishing`, `distribution`), the interface should be organised around how a livery is made: choose the fleet, paint each texture, reuse the brand, export one mod. This change redesigns the interface from the "TruckPaint 5" design spec **without adding or removing a single function**: every current function is kept and moved to where it is looked for.

## What Changes

**Look (design tokens)**
- New dark theme, low in saturation so livery colors read true: surfaces canvas `#1C1C1F`, panel `#222225`, raised `#2E2E33`, ink `#ededed`.
- Three accents, each with one meaning: **white** for the primary action and the active option, **red** `#F56B6B` for signals (alert, update available, selection), **blue** `oklch(0.74 0.11 225)` for everything linked to the brand (a fill linked to a palette swatch, an object following a style, a symbol instance).
- Segmented controls with a white pill for the active option.
- **BREAKING (look):** fonts Geist (interface) and JetBrains Mono (values, paths, pixels) embedded under the OFL, replacing Inter as the UI typeface. Type scale: titles 22/600, interface 13/400, mono 12.

**Chrome**
- Top bar (44 px): project name with its game badge, the space switcher Project / Workshop / Brand, and an always-visible **Export…** button (Export Mod…).
- Tool rail (48 px), icons only, each with a tooltip giving its translated name and shortcut. A contextual **tool options bar** under the top bar shows the active tool's settings.
- Status bar (28 px): zoom, pointer position, save state, and the view settings: template (show, opacity, `G`), snapping, grid and guides.
- Breadcrumb Vehicle › Cabin › Texture, in the top bar and inlaid on the canvas.
- The in-window menu bar stays as it is (menus in the title bar are `title-bar-menus`).
- **Tab** hides the left panel and the inspector (focus mode), only when no text field has focus, and without breaking keyboard focus navigation inside dialogs.

**Three spaces** (`Cmd/Ctrl+1`, `+2`, `+3`)
- **Project:** the fleet (vehicles, their cabins and textures, package updates through Update Template) and the mod information (name, version, author, description, game versions, both mod pictures). It is the landing screen when a file is opened. Mod settings are shown here; editing them still goes through Export Mod…, as today. Game versions stay editable as they are in the sidebar today. Clicking a texture opens it in the Workshop.
- **Workshop:** paints one texture. Left panel with three tabs, Textures / Layers / Resources (keys `1`, `2`, `3`); canvas in the middle; on the right an **inspector** that shows only the settings of the selection.
- **Brand:** palette, graphic styles, text styles, symbols and images of the project, with the actions their panels have today (Add to Library, Import from Library…, Edit Swatch…, symbol Edit/Rename/Duplicate/Delete…).

**Where each current panel goes** — **BREAKING:** the right panel stack and the Vehicles sidebar are removed.
- Properties → inspector; with nothing selected it shows the texture's properties (including the template notices "Layout changed" / "Not in this version").
- Template settings → status bar, as a view setting.
- Layers → Layers tab.
- Colors → a **Fill** row in the inspector; the picker opens as a popover (Esc closes it), with the brand palette listed before the recent colors. Gradients are edited in the same popover.
- Stroke → a **Stroke** row under Fill.
- Transform (fields, align, distribute, combine, flip) → a **Layout** section, shown only with a selection.
- Styles, Symbols and Assets → Resources tab (dragging to the canvas works as today).
- Vehicles sidebar tree → Textures tab (and the Project screen); its Project section → Project screen.
- Inspector sections follow the selection: a text shows its font, an instance shows "Edit Symbol". No panel shows an empty state.

**Screens redone with existing functions only:** home screen, New Project (vehicle list on the left, "your fleet" on the right with the textures that will be created), Project, Workshop, Brand, Export Mod dialog, Vehicle Library.

**Keyboard**
- `Cmd/Ctrl+1/2/3` spaces, `Tab` hide panels, `1/2/3` left tabs, `Cmd/Ctrl+[` / `Cmd/Ctrl+]` previous / next texture, `G` show/hide template, `Cmd/Ctrl+E` Export… (Export Mod).
- **BREAKING:** F5 (View › Sidebar) and Vehicle › Vehicle Information are replaced by the spaces and Tab. The new keys displace current bindings, which move (new keys chosen in design): Actual Size (`Cmd/Ctrl+1`), Send Backward / Bring Forward (`Cmd/Ctrl+[` / `]`), Gradient tool (`G`), Export Texture… (`Cmd/Ctrl+E`), Show Template (`Shift+T`), Next / Previous Texture (`Cmd/Ctrl+Page Down/Up`, kept as alternates if free).

**Four languages:** no label in a fixed-width box (free-width buttons, field labels above the field, icon-only tools). Every screen fits German at +40% length.

**Deliberate deviations from the mockup**
- "One per cabin / One for all" is shown read-only in New Project: it comes from the package (`separate_paint_jobs`); the player only picks cabins.
- Export always writes the whole fleet as one mod: no "current texture / current vehicle / whole fleet" choice (Export Texture… stays its own command), and no "Enable in mod manager" checkbox.
- Custom vehicle wording: templates are DDS, PNG or SVG and the game data is typed in; no "PSD" nor "definitions folder".
- Both mod pictures are shown (shop icon 256×64 and Mod Manager image 276×162).
- No per-texture export resolution in the inspector (not a feature).
- Mockup elements that are new features belong to their own changes and are not built here: texture states Empty / Modified / To check, the "on this texture" summary with off-palette colors and the pre-export checklist (`texture-status`); editing mod info on the Project screen (`mod-settings-in-project`); the `Cmd/Ctrl+K` command palette (`command-palette`); usage counts, before/after preview and "Apply to fleet" in Brand (`brand-edit-preview`); menus in the title bar or the macOS system menu bar (`title-bar-menus`); drop shadow (`drop-shadow`); template update impact in the Vehicle Library (`template-update-impact`); the first-run wizard (`first-run`) and the package catalog (`marketplace`).

**Brand space and the personal library:** the Brand space shows the project's elements; the personal library stays reachable through Import from Library… and Add to / Update in Library, with no library browser (design.md, Decision 9).

## Capabilities

### New Capabilities
- `workspace-spaces`: the Project / Workshop / Brand spaces, the switcher in the top bar and its shortcuts, which space opens when a file is opened, the always-visible Export… button, and the breadcrumb.
- `project-screen`: the Project space: the fleet with its vehicles, cabins and textures, package updates, the read-only mod information column with the editable game versions, opening a texture in the Workshop.
- `brand-space`: the Brand space: the project's palette, graphic styles, text styles, symbols and images, with the actions their panels have today.

### Modified Capabilities
- `ui-design-system`: new surface and accent tokens and their meanings, segmented controls, Geist and JetBrains Mono, the new type scale.
- `workspace-layout`: top bar, tool rail with tooltips, tool options bar, status bar with view settings, left panel with tabs, selection-driven inspector, Tab focus mode; removes the right panel stack and the Vehicles sidebar.
- `start-screen`: the redesigned home screen and New Project (still one vehicle) (vehicle list, "your fleet", textures to be created, read-only main texture mode).
- `properties-panel`: becomes the inspector; texture properties with no selection; sections by selection kind; no empty state.
- `color-panel`: Fill row and picker popover (Esc closes), brand palette before recents.
- `gradients`: Solid / Linear / Radial and the gradient bar move into the picker popover.
- `stroke-panel`: Stroke row under Fill.
- `transform-panel`: Layout section shown only with a selection.
- `layers-panel`: lives in the Layers tab.
- `assets-panel`: lives in the Resources tab and the Brand space.
- `symbols`: Symbols list in the Resources tab and Brand space; instances shown in the link color; "Edit Symbol" in the inspector.
- `shared-styles`: Styles list in the Resources tab and Brand space; linked objects shown in the link color.
- `shared-library`: Add to Library and Import from Library… entry points move (no more empty panels).
- `vehicle-projects`: texture switching through the Textures tab and the Project screen; Next / Previous Texture and Show Template shortcuts; template settings in the status bar; the Vehicle panel replaced.
- `vehicle-packages`: redesigned Vehicle Library dialog.
- `mod-export`: Export… in the top bar with `Cmd/Ctrl+E`; redesigned dialog; mod info shown on the Project screen.
- `texture-export`: Export Texture… loses `Cmd/Ctrl+E`.
- `selection-transform`: Bring Forward / Send Backward lose `Cmd/Ctrl+]` / `[`.
- `canvas-navigation`: Actual Size loses `Cmd/Ctrl+1`.
- `command-system`: Gradient tool loses `G`; Tab and `1/2/3` don't fire while a text field has focus.
- `localization`: no fixed-width labels; German +40% budget.
- `app-preferences`: what is persisted changes (left tab, panel widths and hidden panels instead of the panel stack and sidebar); older preference files still load.
- `path-tools`: polygon settings in the Polygon tool's options bar and an inspector Polygon section; line width and line style of open paths in the inspector's Appearance section.
- `custom-vehicles`: Custom vehicle… is reached from New Project's vehicle list (no more Vehicle step).
- `image-import`, `document-model`, `text-style`, `path-booleans`: references to the Assets, Transform and Layers panels follow their new places (images lists, Layout section, Layers tab); no behavior change.

## Impact

- **tp-ui:** `tokens.rs` (surfaces, accents, type scale), `theme.rs`, `fonts.rs` (Geist and JetBrains Mono replace Inter as UI fonts; Inter stays available as a document font), `icons.rs`, and the widgets (segmented control, popover, inspector rows, breadcrumb, tool rail button). New font files and OFL licenses in `assets/fonts/`.
- **tp-app:** `ui/workspace` (panels, toolbar, status bar, a new layout with spaces, left tabs and inspector), `ui/home.rs`, `ui/dialogs.rs`, `ui/vehicle_dialogs.rs`, `ui/mod_export_dialog.rs`, `ui/library_dialog.rs`, `ui/menu_bar.rs`, `commands.rs` (new commands and the reassigned shortcuts), preferences.
- **tp-i18n:** new and changed strings in en, fr, de, es.
- **Documents and file formats:** none. Project files, packages and the library are unchanged.
- **docs/roadmap.md:** records the decisions and the open question when the change ships.
