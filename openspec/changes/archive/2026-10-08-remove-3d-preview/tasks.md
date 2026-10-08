## 1. Workspace and commands (tp-app)

- [x] 1.1 Make the central area the canvas only:
  - `ui/workspace/mod.rs` calls `canvas_area` directly and no longer imports `ViewMode` and `SPLIT_FRACTION_RANGE`;
  - delete `ui/workspace/preview.rs` and its `mod` line;
  - `status_bar::show` loses its `view_mode` parameter and always shows zoom and coordinates.

  Verify with `cargo build -p tp-app`; the status bar tests still pass.
- [x] 1.2 Remove the commands:
  - `CommandId::SetViewMode`, `TogglePreview`, their `meta()` entries and their place in `all()`;
  - the now unused `CMD_ALT` constant;
  - their dispatch in `state.rs`, the `SetViewMode(_) | TogglePreview` arm of the availability match, and the "a project opens in 2D" rule in `open_project`.

  Verify with `cargo build -p tp-app` and the command tests (`all()` and the shortcut conflict check).
- [x] 1.3 In `ui/menu_bar.rs`, remove the segmented control on the right of the menu bar and the four View menu items with the separator they leave doubled. The other View items stay. Verify with a UI test: the View menu lists Zoom In… Actual Size, then Show Grid, with no "3D" or "Split" item, and the menu bar has no "Split" button.
- [x] 1.4 Remove `ViewMode`, `WorkspaceLayout.view_mode`, `split_fraction` and `SPLIT_FRACTION_DEFAULT`/`SPLIT_FRACTION_RANGE` from `layout.rs`, with their sanitizing and test lines. Verify with a prefs test: a preferences file written by the previous build, with `view_mode: ThreeD` and `split_fraction: 0.42` in its layout and UI scale 1.25, loads with `issue` empty, UI scale 1.25 and the other layout values kept. Update the existing round-trip test, which sets `ViewMode::Split`.

## 2. Icons, gallery and strings

- [x] 2.1 Delete `PREVIEW_3D`, `VIEW_2D`, `VIEW_3D` and `VIEW_SPLIT` from `tp-ui/src/icons.rs`. In `ui/gallery.rs`, demo `SegmentedControl` with the rectangle, ellipse and polygon tool icons. Verify with `cargo build --workspace` and a grep that finds none of the four icon names.
- [x] 2.2 Remove the strings `view-2d`, `view-3d`, `view-split`, `cmd-2d-canvas`, `cmd-3d-preview`, `cmd-split-view`, `cmd-show-3d-preview`, `preview-hide`, `preview-soon` and `preview-soon-hint`, and the `## 3D preview` section header, from the en/fr/es/de files. Verify with the i18n tests (same keys in every language, no unused key if such a check exists) and the localization screen test.

## 3. Tests

- [x] 3.1 Delete `view_modes_show_expected_regions` (`tests/ui.rs`) and `projects_open_showing_the_canvas` (`tests/vehicles.rs`). Remove the Split screenshot from `tests/screenshots.rs`, keeping the 2D captures. Verify that `cargo test --workspace` passes, and that `mise run screenshots` still renders the workspace screens.

## 4. Specs and docs

- [x] 4.1 Update the Purpose of `openspec/specs/workspace-layout/spec.md` to drop "3D preview area, view modes" (a delta can't change a Purpose). Verify with `openspec validate --specs --strict`.
- [x] 4.2 In `docs/roadmap.md`, drop "The 3D preview will make placement easier later." and record the decision (the 3D preview is dropped). In `docs/roadmap_light.md`, mark the item as done. Verify by reading the diff.
- [x] 4.3 Run `mise run ci`. Verify that format, lint, tests and builds pass, and that a final `grep -rni "3d\\|ViewMode\\|split_fraction" crates` shows no leftover outside archived history.
