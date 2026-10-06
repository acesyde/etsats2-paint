## Context

Greenfield repository: only `mise.toml` (Rust 1.98.1), OpenSpec and git tooling exist. See proposal.md for motivation and scope. Requirements are in `specs/`.

Constraints that shape this design:

- Desktop only, three first-class platforms: Linux (X11 and Wayland), Windows, macOS. WebAssembly is postponed, not abandoned.
- The user's primary criterion is perceived product quality and UX; when in doubt prefer simple and stable over elegant and complex.
- Later changes (canvas, vehicles, export, 3D) must slot in without reworking the shell. Domain knowledge gathered from Paintjob Packer (MIT; JSON vehicle DB `format_version: 1`; one main texture per cabin plus one texture per accessory group; deterministic SII/SUI/TOBJ/MAT generation; DDS BC3 output) informs the data model introduced in later changes; this change only reserves the right places for it.

## Goals / Non-Goals

**Goals:**
- A workspace + crate layout that every later change extends rather than restructures.
- A design system good enough that later features look finished by default when built with its widgets.
- One action pipeline (commands) shared by menus, tool bar, context menus and keyboard.
- CI that proves, on every push, that the app builds and its tests pass on all three OSes.

**Non-Goals:**
- Any document editing behavior (canvas navigation, shapes, selection, undo) — `canvas-core`.
- Project file format and autosave — `persistence`.
- Vehicle catalog and templates — `vehicle-library`.
- 3D rendering — `export-and-preview`.
- Installers, signing, notarization, file associations — `distribution`.
- Native macOS menu bar; detachable/floating panels.

## Decisions

### D1. eframe with the wgpu backend
`eframe` (egui) with `wgpu` renders through Vulkan / DX12 / Metal, with GL fallback on Linux. The 3D preview will later be a wgpu paint callback inside an egui panel, so choosing wgpu now avoids a backend switch.
*Alternatives:* `glow` backend (simpler, but a second GPU abstraction for 3D later); Bevy/other engines (heavy, poor fit for a document UI); Slint/iced (weaker ecosystem for custom canvas + immediate interaction).

### D2. Crate layout

```
crates/
  tp-core   domain types, no UI, no fs, no threads (document stub, resolution, ids)
  tp-ui     design system: tokens, theme install, fonts/icons, widgets
  tp-app    eframe binary: screens, commands, layout, preferences, platform I/O
```
Further crates (`tp-render`, `tp-vehicles`, `tp-modgen`) arrive with their changes. Rule: only `tp-app` touches the filesystem, OS dirs, threads or dialogs; `tp-core`/`tp-ui` take bytes/readers. This keeps them testable and keeps a future WASM target cheap.
*Alternative:* single crate — faster at first, but the boundary rule would erode immediately.

### D3. Design tokens as code, theme installed into egui
`tp-ui::tokens` defines colors (surface 0–4 elevations, text primary/secondary/disabled, accent, semantic), spacing scale (2/4/8/12/16/24), radii, stroke widths and type scale. `tp-ui::theme::install(ctx, &Prefs)` maps tokens onto egui `Style`/`Visuals`/`Spacing` and registers fonts. Widgets that egui styles poorly (tool buttons, panel headers, segmented control, menu rows with shortcuts, numeric scrub fields, empty states) are custom widgets in `tp-ui::widgets` built from tokens.
A unit test computes WCAG contrast ratios for every declared text/background pair (spec: Text contrast).
*Alternative:* only tweak egui `Visuals` — insufficient for the target look (no indicator bars, weak focus/active cues).

### D4. Embedded fonts and icons
UI typeface: Inter (SIL OFL), embedded via `include_bytes!`. Icons: Phosphor icon font via `egui-phosphor` (MIT), merged into the proportional family so icons can be inlined in labels. Only the regular weight of each is embedded initially to keep binary size reasonable.
*Alternative:* SVG icons rasterized at runtime — sharper control but more code; can be revisited if Phosphor lacks a needed glyph (custom glyphs can be added as SVG later).

### D5. Command registry
`CommandId` is an enum; a static table describes each command (label, icon, default `KeyboardShortcut`, context: global / canvas / text-input-safe). Each frame, UI code calls `commands.button(ui, id)` / `commands.menu_item(ui, id)`; triggered commands are pushed to a queue and executed once at the end of the frame by `App::dispatch`. Enabled state is computed by a single `fn is_enabled(id, &AppState) -> bool`. Shortcut matching uses egui's `Modifiers::COMMAND` (Cmd on macOS, Ctrl elsewhere) and `ModifierNames` formatting for labels. A test asserts no two default bindings collide in the same context.
Queued dispatch avoids borrowing conflicts in immediate mode and makes every action loggable/testable.
*Alternative:* closures stored per widget — harder to test, duplicate logic between menu and shortcut.

### D6. Application state machine

```
App
 +-- prefs: Prefs                    (persisted)
 +-- screen: Home | Workspace(WorkspaceState)
 +-- modal: Option<Modal>            (NewProject wizard, Preferences)
 +-- commands: CommandQueue
WorkspaceState
 +-- project: tp_core::ProjectStub { name, resolution }
 +-- active_tool: Tool
 +-- view_mode: TwoD | ThreeD | Split
 +-- layout: PanelLayout             (persisted part lives in prefs)
 +-- save_state: Saved | Unsaved
```
The New Project modal is a step-based wizard (`Vec<WizardStep>`) with only the "Name & resolution" step for now; `vehicle-library` will insert Game / Manufacturer / Vehicle / Cabin steps before it.

### D7. Custom right panel stack instead of a docking library
A resizable `SidePanel` hosts an ordered list of `PanelKind` entries, each with `open` / `collapsed` flags, drawn with the `PanelHeader` widget; the body scrolls when the stack overflows. The 3D preview is a separate resizable region whose placement depends on view mode (Split: right of canvas with a draggable divider). This gives full control over the look and is enough for open/close/collapse/resize.
*Alternatives:* `egui_dock` / `egui_tiles` — powerful (tabs, floating, drag docking) but their styling fights the target look and adds state complexity; revisit when detachable panels are requested.

### D8. Canvas area in this change
The canvas area draws the pasteboard and a fitted artboard rectangle (with drop shadow) using the egui painter, and reports pointer position in texture pixels. Zoom shows the fit percentage. No navigation yet — `canvas-core` replaces this widget with the real canvas.

### D9. Preferences storage
`directories::ProjectDirs` provides the per-OS config dir (`~/.config/...`, `%APPDATA%\...`, `~/Library/Application Support/...`). Preferences are a versioned `serde` struct saved as RON (`prefs.ron`), written atomically (write temp file in same dir, fsync, rename). Invalid/unknown files are renamed to `prefs.ron.bak-<timestamp>` and defaults are used. Window geometry uses eframe's persistence hooks combined with a visibility clamp against available monitors.
Saving is debounced (on change + on exit) to avoid disk churn while dragging dividers.

### D10. Testing strategy
- Unit tests in `tp-core` / `tp-ui` (contrast, token sanity, shortcut formatting).
- Headless UI tests with `egui_kittest` in `tp-app` for spec scenarios (tool switching, panel collapse/close/reopen, modal keyboard flow, disabled commands). Image snapshot tests are deferred until rendering is stable across CI GPUs.

### D11. CI
`mise.toml` is the single source of truth for runtimes (Rust 1.98.1 with rustfmt and clippy) and for build commands, exposed as mise tasks: `fmt`, `fmt:check`, `lint`, `test`, `test:release`, `build`, `run`, `screenshots`, and `ci` (all checks in order). GitHub Actions installs Rust through `jdx/mise-action` (mise version pinned) and calls the same mise tasks as local runs, in two stages: a `checks` job on Linux runs the OS-independent `fmt:check` and `lint` once and fails fast; then a `build` matrix (`ubuntu-latest`, `windows-latest`, `macos-latest`) runs `test:release` and `build`, since tests exercise platform behavior (file system, OS directories). Tests run in release mode so the release build reuses their artifacts instead of compiling everything twice; `rust-cache` also saves after failed runs. Each job writes a GitHub step summary (runner, CPU count, cache hit or miss, test counts, step durations, binary size, and the relevant error excerpt on failure). Third-party actions use their latest release, pinned by commit SHA; the CI mise version matches the one used locally (the 30-day `minimum_release_age` in `mise.toml` applies only to tools installed by mise). The Linux job installs the winit/wgpu system packages (libxkbcommon, wayland, x11, xcb). Cargo cache via `Swatinem/rust-cache`.

*Alternatives:* `rust-toolchain.toml` plus raw `cargo` commands in the workflow — rejected, as it duplicates the version pin and lets CI drift from local commands.

### D12. Logging
`tracing` + `tracing-subscriber` with env-filter; logs to stderr and, on desktop, a rolling file in the data dir (useful for crash reports from non-technical users).

## Risks / Trade-offs

- [egui's default widgets leak the "egui look" into new features] → Every later feature must use `tp-ui` widgets; code review checklist item; theme sets egui visuals so even stray default widgets look close.
- [Inter + Phosphor do not cover all needs (e.g., some tool icons like Direct Selection)] → Allow custom SVG-sourced icons in `tp-ui::icons` behind the same API.
- [wgpu driver issues on some Linux setups / VMs] → Allow `WGPU_BACKEND=gl` override and document it; fall back gracefully with a clear error dialog if no adapter is found.
- [Wayland differences (fractional scaling, client-side decorations)] → CI builds on Linux but cannot test Wayland visually; manual check on X11 and Wayland before archiving the change.
- [Custom panel stack lacks docking] → Accepted for MVP; D7 records the escape hatch.
- [Immediate-mode UI cost grows with panels] → Panels draw only when open and expanded; `canvas-core` will introduce caching for heavy content.

## Migration Plan

Not applicable (first change). Rollback = revert the change.

## Open Questions

- **3D vehicle models source — to be determined.** Options: (A) provided glTF/OBJ exports with game UVs, (B) reading the installed game's archives (HashFS + PMG), (C) generic proxy for the MVP. Only affects `export-and-preview`; this change reserves the 3D panel and view modes regardless of the answer.
- Format of the templates the user will provide (Paintjob Packer `templates.zip` layout or other) — affects `vehicle-library` only.
- Apple Developer account for signing/notarization — affects `distribution` only.
- Final product name / app icon — a placeholder name ("TruckPaint") and icon are used until decided; changing them later is cosmetic.
