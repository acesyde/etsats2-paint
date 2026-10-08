## Context

See proposal.md for the motivation. The 3D preview touches several modules, but holds no logic:
- `layout.rs`: `ViewMode { TwoD, ThreeD, Split }`, with `WorkspaceLayout.view_mode` and `split_fraction`. They are stored in the preferences, which are RON with `#[serde(default)]`.
- `commands.rs`: `SetViewMode(ViewMode)` (Cmd/Ctrl+Alt+1/2/3) and `TogglePreview` (no shortcut).
- `ui/menu_bar.rs`: a segmented control on the right of the menu bar, and four View menu items.
- `ui/workspace/mod.rs`: the central area chooses between canvas, preview and split. `ui/workspace/preview.rs` is the placeholder.
- `state.rs`: dispatch, and a rule that switches from 3D to 2D when a project opens.
- `ui/workspace/status_bar.rs`: hides the zoom and coordinates when the canvas isn't shown.
- `ui/gallery.rs`: demos `SegmentedControl` with the view icons.

## Goals / Non-Goals

**Goals:**
- Delete the feature and everything only it used, with no dead code left behind.
- Existing preference files load as before, minus the removed values.

**Non-Goals:**
- Reorganizing the View menu or the menu bar beyond removing these items.
- Removing `SegmentedControl` (Export Texture uses it).

## Decisions

### Delete the fields; serde already ignores them in old files
`view_mode` and `split_fraction` are removed from `WorkspaceLayout`, with no placeholder field. `Prefs` and `WorkspaceLayout` don't deny unknown fields, and RON skips an unknown field whatever its value. I checked this with ron 0.12.2: `(layout: (column_width: 300.0, view_mode: ThreeD, split_fraction: 0.42, vehicles_open: true))` loads with the other fields intact.

So a file from before this change loads without a reset or a backup, and the next save drops the two keys. `PREFS_VERSION` stays 1.

- *Alternative: keep a `view_mode` field that is read and never used.* Rejected: it is dead code, and serde doesn't need it.

### The central area is the canvas
`CentralPanel` calls `canvas_area` directly. The status bar always shows zoom and coordinates, so its `view_mode` parameter goes away.

### The menu bar's right side becomes empty
The segmented control was its only content. Nothing replaces it.

### The gallery demo uses other icons
`SegmentedControl` stays in the gallery with icons that remain in use (the rectangle, ellipse and polygon tool icons), so the four view icons can be deleted.

## Risks / Trade-offs

- **[Muscle memory for Cmd/Ctrl+Alt+1/2/3]** They do nothing now. The menu no longer lists them, and the shortcut conflict check ignores free keys.
- **[Archived changes still mention the 3D preview]** They are history and stay unchanged.
