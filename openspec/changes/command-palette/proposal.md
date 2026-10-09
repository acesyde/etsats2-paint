## Why

TruckPaint has about a hundred commands spread over eight menus, context menus and the panels, and a fleet can reach 30 to 40 textures. A player who knows what they want ("Flip Horizontal", "Chassis") still has to remember where it lives, or scroll the sidebar's tree. The redesign (`new-design`) moves every function into the Project, Workshop and Brand spaces, so for a while even regular players won't know where things went. A search box that reaches every command and every texture from the keyboard fixes both, and is cheap because every action is already a registered command.

## What Changes

- **Command palette.** Cmd+K (macOS) / Ctrl+K (Windows, Linux) opens a search field over the window, on any screen. It is also in the View menu and in the Workshop's "Search a texture ⌘K" field from the mockup.
- **Every command is listed** from the command registry: its translated label, its icon, its shortcut in the platform's notation, and its menu as context ("Object › Flip Horizontal"). Disabled commands are listed but shown disabled with the reason the registry already gives, and can't be run. Commands that only make sense as a gesture (Nudge, Space for the Hand tool) are left out.
- **The project's textures are listed** as "Vehicle › Main textures › Standard cab" ("Vehicle › Cabin › Texture"). Choosing one makes it the active texture, as clicking it in the sidebar does. Without a project, only commands are listed.
- **Fuzzy search** over the label and its context, in the interface language, ignoring case and accents ("flp hor", "chassis", "retourner"). The best matches come first; with an empty query the palette shows recently run commands, then textures.
- **Keyboard only:** Up/Down to move, Enter to run or go, Escape to close; the mouse works too. Running a command closes the palette and acts as if it was chosen from its menu, including undo labels.
- The palette doesn't change the document by itself, and closing it leaves the selection, the active tool and the active texture as they were.

Shortcut check: no command uses Cmd/Ctrl+K today (Convert to Symbol has no shortcut). The mockup gives Convert to Symbol ⌘⇧K; the registry matches modifiers exactly, so ⌘K and ⌘⇧K don't collide, and the existing duplicate-shortcut test guards against a future clash. The palette's shortcut is global (it works while a text field has focus), like Save.

Non-goals:
- searching objects, layers, symbols or swatches (may come later, same field);
- user-defined shortcuts or aliases;
- typed arguments ("zoom 200").

## Capabilities

### New Capabilities
- `command-palette`: opening and closing the palette, what it lists (commands with shortcut, context and disabled state; the project's textures), the search and its ordering, recent commands, and keyboard use.

### Modified Capabilities
- `command-system`: the palette becomes one more entry point of "Commands as the single action path", with the same effect and the same disabled behavior; Cmd/Ctrl+K is a default, global shortcut.
- `vehicle-projects`: "Switching textures" names the sidebar's tree as the place to switch textures; the palette becomes a second way, with the same effects (selection cleared, view kept per surface).

## Impact

- **Depends on `new-design`:** the palette is drawn in the new design system and placed in its Workshop layout (the "Search a texture" field). Command labels and menu paths are read from the registry after the redesign has settled them.
- **tp-app:**
  - `commands.rs`: a `CommandPalette` command with its shortcut; a way to list paletted commands with their menu path (today the menu structure lives only in `ui/menu_bar.rs`, so it moves to data both use); a flag for gesture-only commands;
  - a new `ui/palette.rs`: the overlay, the fuzzy matcher (pure, unit-tested) and the result list;
  - `state.rs`: recent commands (session only) and dispatch of a chosen texture through the existing texture switch.
- **tp-i18n:** a few strings (placeholder, empty result, group names) in en/fr/es/de.
- **Dependencies:** none expected; a small subsequence matcher is enough. A crate such as `nucleo-matcher` is an option for design.md.
