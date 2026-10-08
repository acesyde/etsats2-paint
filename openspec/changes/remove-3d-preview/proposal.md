## Why

The workspace has had a 3D preview since the app shell, but it has only ever been a placeholder ("3D preview coming soon"), and the product decided to drop it (see `docs/roadmap_light.md`). The 3D and Split view modes show nothing useful and take space and shortcuts. The 2D / 3D / Split control in the menu bar suggests a feature that won't come.

## What Changes

- **BREAKING (UI):** remove the 3D preview panel and its placeholder. The central area always shows the canvas.
- **BREAKING (UI):** remove the view modes. That covers:
  - the 2D / 3D / Split control on the right of the menu bar;
  - the View menu items 2D Canvas, 3D Preview, Split View and Show 3D Preview;
  - their shortcuts Cmd/Ctrl+Alt+1, 2 and 3, which become free;
  - the draggable divider of the split view.
- Remove what only served them:
  - the view mode and split width stored in the preferences;
  - the rule that a project opens in 2D when the remembered mode was 3D;
  - the status bar's check that the canvas is shown;
  - the cube and view icons;
  - the strings in en/fr/es/de.
- **Older preference files keep working.** A file that stored a view mode, including 3D, or a split width still loads with all its other preferences. The old values are ignored.
- **Docs:**
  - `docs/roadmap.md` drops "The 3D preview will make placement easier later" and records the decision;
  - `docs/roadmap_light.md` marks the item as done.

Non-goals: any other View menu item (zoom, grid, guides, template, sidebar), the panel column, and the Vehicles sidebar stay as they are.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `workspace-layout`: the purpose no longer mentions a 3D preview area or view modes. The requirements "3D preview panel placeholder" and "View modes" are removed.
- `app-preferences`: the persisted preferences no longer include the 3D panel and the view mode, and preference files that still hold them load normally.
- `document-model`: the mirrored state no longer lists the 3D preview among the places an object is drawn.

## Impact

- **tp-app:**
  - `layout.rs`: `ViewMode`, `view_mode`, `split_fraction` and the `SPLIT_FRACTION_*` constants;
  - `commands.rs`: `SetViewMode` and `TogglePreview`, with their metadata and shortcuts;
  - `state.rs`: dispatch and the "open in 2D" rule;
  - `ui/menu_bar.rs`: the segmented control and the View menu items;
  - `ui/workspace/mod.rs`: the central area is the canvas only;
  - `ui/workspace/preview.rs` is deleted;
  - `ui/workspace/status_bar.rs` and the `ui/gallery.rs` demo of the segmented control.
- **tp-ui:** the icons `PREVIEW_3D`, `VIEW_2D`, `VIEW_3D` and `VIEW_SPLIT`. `SegmentedControl` stays: the Export Texture dialog uses it.
- **tp-i18n:** remove `view-2d`, `view-3d`, `view-split`, `cmd-2d-canvas`, `cmd-3d-preview`, `cmd-split-view`, `cmd-show-3d-preview`, `preview-hide`, `preview-soon` and `preview-soon-hint` in the four languages.
- **Tests:** remove the view mode UI tests (`tests/ui.rs`, `tests/vehicles.rs`) and the Split screenshot, update the prefs round trip, and add a test that an old preferences file loads.
- No file format change, no new dependency.
