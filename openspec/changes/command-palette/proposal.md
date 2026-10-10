## Why

TruckPaint has about a hundred commands spread over eight menus, two submenus, context menus and the panels, and a fleet can reach 30 to 40 textures. A player who knows what they want ("Flip Horizontal", "Chassis") still has to remember where it lives, or scroll the Textures tab. Since `new-design` moved every function into the Project, Workshop and Brand spaces, even regular players don't always know where things went. A search box that reaches every command and every texture from the keyboard fixes both, and is cheap because every action is already a registered command and every texture already has a state (`texture-status`) and a breadcrumb path.

## What Changes

- **Command palette.** Cmd+K (macOS) / Ctrl+K (Windows, Linux) opens a search field over the window, on the home screen and with a project open, in every space, even while a text field has focus. Pressing it again, Escape or a click outside closes it. It is not opened over a modal dialog. It is also in the View menu (Command Palette…).
- **Search textures field.** The Workshop's Textures tab gets the mockup's "Search textures ⌘K" field at its top (artboard 04 of the "TruckPaint 5" spec). Clicking it opens the palette limited to the project's textures; Backspace in its empty query widens it to everything.
- **Every command is listed** from the command registry: its translated label (Undo and Redo name the operation, as in the Edit menu), its icon, its shortcut in the platform's notation, its menu path as context ("Object › Align › Align Left"; commands outside the menus get a group: Tools, Colors, Symbol), and the check mark of a toggle. Disabled commands are listed after the enabled ones, in the disabled style, with the reason the registry gives, and can't be run. Commands that only make sense as a gesture (Nudge and its Shift variant; Space for the Hand tool is not a command) are left out, and so is the palette itself.
- **The project's textures are listed** as "Vehicle › Main textures|Accessories › Texture", like the breadcrumb, with their state marker (Empty, Modified, To check) as in the Textures tab. Choosing one makes it the active texture and shows the Workshop, from any space, as clicking it in the Project space does. Without a project, only commands are listed.
- **Fuzzy search** over the label and its context, in the interface language, ignoring case and accents ("flp hor", "chassis", "tout selectionner" finds "Tout sélectionner"). Words may come in any order. The best matches come first, the matched letters are highlighted. With an empty query the palette shows the commands recently run from it (this session), then the project's textures, then every command in menu order.
- **Keyboard only:** Up/Down (and Page Up/Down) move the highlight, Enter runs the command or opens the texture, Escape closes; the mouse works too. Running a command closes the palette first, then acts as if the command was chosen from its menu, including undo labels.
- The palette doesn't change the document by itself, and closing it leaves the selection, the active tool, the active texture, the space and a text being edited as they were.

Shortcut check: no command uses Cmd/Ctrl+K today, and Convert to Symbol still has no shortcut (the mockup suggests ⌘⇧K for it; that is left to a later change, and would not collide since the registry matches modifiers exactly). The `no_duplicate_default_shortcuts` test guards against a future clash. The palette's shortcut is global (it works while a text field has focus), like Save and Export….

Non-goals:
- searching objects, layers, symbols, swatches or styles (may come later, in the same field);
- recent commands kept across sessions;
- user-defined shortcuts or aliases;
- typed arguments ("zoom 200").

## Capabilities

### New Capabilities
- `command-palette`: opening and closing the palette (shortcut, View menu, Search textures field), what it lists (commands with shortcut, menu path, toggle state and disabled reason; the project's textures with their state), the search and its ordering, recent commands, running a command or opening a texture, and keyboard use.

### Modified Capabilities
- `command-system`: "Commands as the single action path" names the palette as an entry point with the same effect; "Disabled commands" covers the palette's rows; "Default workspace shortcuts" adds Command Palette `Cmd/Ctrl+K` (global), listed in the Keyboard Shortcuts window.
- `workspace-layout`: "Menu bar" adds Command Palette… as the first item of the View menu.
- `vehicle-projects`: "Switching textures" adds the palette as a way to switch textures, with the same effects (selection cleared, view kept per surface); "Textures tab" adds the Search textures field at the top of the tab.

## Impact

- **Builds on** `new-design` (spaces, Textures tab, tokens), `texture-status` (state markers) and the breadcrumb's "Vehicle › Main textures|Accessories › Texture" naming.
- **tp-app:**
  - `commands.rs`: a `CommandPalette` command (`Cmd/Ctrl+K`, `Scope::Global`) and `in_palette()`, which leaves out Nudge and the palette;
  - a new `menus.rs`: the menu structure as data (menus, submenus, separators, toggles), drawn by `ui/menu_bar.rs` and read by the palette for menu paths, so the two can't disagree;
  - a new `palette.rs` (pure: text folding, fuzzy matcher, ranking, unit-tested) and `ui/palette.rs` (the overlay, the result rows, the keys);
  - `state.rs`: a `Modal::CommandPalette` variant, the session's recent palette commands, and opening a chosen texture in the Workshop;
  - `ui/workspace/panels/vehicle.rs`: the Search textures field.
- **tp-i18n:** a dozen strings (command label, placeholders, empty result, footer hints, group names, accessibility names) in en/fr/es/de.
- **Dependencies:** none. A small home-made subsequence matcher and accent-folding table are enough for about 130 entries; `nucleo-matcher` is weighed and set aside in design.md.
- **Files:** no project or preferences format change.
