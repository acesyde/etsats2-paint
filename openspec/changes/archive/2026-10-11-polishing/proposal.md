## Why

The first players will meet the app on the new design, and a screenshot audit of every screen (English, French, German, Spanish and the 40% longer texts) found layouts that break, labels cut mid-word, English names in other languages, controls that show the wrong state and wording that reads badly. None of it needs a new feature; together it makes the app look unfinished. This change fixes them before `official-vehicles` batches and `distribution` bring more players.

## What Changes

Fixes, ergonomics and wording only; no new feature. Each item names the screenshot that shows it.

**Layouts that break**
- The Custom Vehicle dialog grows wider than the window: the role combo of an accessory row takes all the remaining width, cutting the title, the left column and the Create button (`custom_vehicle_*`). Its texture rows get fixed-width combos, and the labels "Role / Width / Height of texture N" (cut to "Height of") are shown short, above their field, the long text staying the accessible name.
- Project space, vehicle card headers: blocks are cut mid-word with no ellipsis ("Kabinenlayou", "Standard c") when the space is narrow, as in German or at 960 px (`screen_de_project_update`, `title_bar_de_*_project_960`).
- Project space at 960 px in German: the All / To do / To check filter overlaps the subtitle "1 Fahrzeug · 4 Texturen · 1 geändert".
- Layers panel: long names are sliced mid-glyph by a clip ("Grey stripı") instead of ending with "…" (`panels_layers_transform`).
- Workshop left panel: vehicle names wrap on two lines ("TruckPaint Sample / Truck") and the version note is squeezed in German (`fleet_project_*`, `sample_*`).
- Resources panel: an asset's "Vektor · 0 Verwendungen" wraps mid-phrase (`screen_de_workshop_resources_content`).
- Preferences: the "100%" box of the UI and text scale sliders falls under its slider instead of beside it (`lang_*_preferences`).
- Inspector Stroke row: the swatch name is cut to "Comp…" while the row has room (`brand_palette_styles_*`).
- About: the license text runs under the scrollbar (`official_en_about`).
- Preferences, Template packs: each game's folder sits on one crowded row with text buttons Choose… and Clear. It becomes the game's name above a read-only field with two icon buttons; clicking the field shows the folder in Finder / Explorer / the file manager, its context menu copies the path, and a folder that no longer exists says so (`official_*_preferences`).
- Home screen: the empty Recent projects state sits centered in a 520 px box against the left edge, neither aligned nor centered. It is centered on the list's width, in a frame where the cards will be, with a Create Your First Project button (`lang_*_home_100`).

**Names and wording**
- The inspector header and the Layers panel tell a default name from a custom one by comparing it with the kind's name in the current language only. A text named "Text" in a French, Spanish or German session shows "Text" instead of its first line, and a "Rectangle" in German shows "Rectangle  Rechteck" (`lang_fr_workspace`, `screen_de_workshop_layers`, known issue in the long-text mode). A name that is the kind's default name in any language is now treated as a default name. The Layers panel also names text layers by their first line, as the inspector does and as the mockup shows, instead of four identical "Text" rows.
- While a symbol is edited with nothing selected, the inspector header names the parent texture ("Standard cab · Main texture") instead of the symbol (`symbols_edit_*`).
- Game version ranges are shown raw: "Supported by every vehicle: >=1.56". They read "1.56 or later" (and the translated equivalents) in the Project space and the Vehicle Library.
- Vehicle Library details: "Game versions any version · cabin · Accessories: 2" becomes "Any game version · Main texture: Cabin · 2 accessories", with the plural handled.
- Export summary: "Standard cab: cabins standard" becomes "Standard cab (cabin: standard)", and a trailer no longer reads "Base: every cabin".
- Template pack dialog: "you download it once for every vehicle" says the opposite of what is meant; it becomes "only once, for all vehicles" (checked in every language).
- The shadow row's summary "24 / 24 · 16" says which value is which, and in long texts the blur is no longer the part cut.
- Symbol bar: "Editing symbol ACE badge" quotes the name.
- English: one casing rule for commands ("Custom Vehicle…" everywhere, "Copy from Cabin" like "New Style from Selection") and American spelling ("Color picker").
- German: "In Pfade umwandeln" (Create Outlines) is nearly the same as "In Pfad umwandeln" (Convert to Path) and becomes "Konturen erstellen"; the texture kind "Haupt" becomes "Haupttextur"; the palette hint no longer leaves "+." alone on a line.
- French and Spanish: the generated shop icon hint is shortened to fit its 64 px box.

**Wrong state shown**
- The Fill row keeps its grey "color target" mark while the shadow, stroke options or line popover is open, so it looks selected (known issue; `shadow_*_popover`, `panels_stroke_popover`).
- The font picker highlights the first font of the list (Inter) instead of the current one (`font_picker`).
- While a file is dragged over a picture drop zone in Mod information, its text loses its contrast (`project_mod_en_drop`).
- While a symbol is edited, the template opacity reads "0 %" (`symbols_edit_*`).
- The Empty dot of a selected texture card nearly disappears on the grey fill (`project_mod_*`).
- To check is red everywhere except in the Export Mod dialog, where it is amber (`status_en_export_mod`).

**Controls that don't match the rest**
- Checkboxes in Import from Library and in Export Texture (Transparent) are drawn as circles, like radio buttons; they become the app's square checkbox (`import_from_library`, `export_png`).
- The Combine and Align submenu titles start 8 px left of the other menu items, with another height (`flip_menu`, `title_bar_*_workshop_object`).
- Mod information, Before exporting: the "N textures empty" row has two markers (dot and caret) and is indented differently; in Export Mod, the Show button is taller than its row and not centered.
- Template pack dialog: the refusal message starts left of the step text (`official_*_pack_dialog_refused`).
- Search fields are shorter than the combos beside them in Add Vehicle and the Vehicle Library.
- Section headers disappear when only one section is listed: "OFFICIAL" in Add Vehicle and New Project, "TEXTURES" in the command palette.
- Value fields don't line up under each other when their labels differ in width (Text section: Size, Tracking, Line; Brand style editor: Width, Opacity).
- In texture rows the status dot moves with the size text's width, and in the Vehicle Library the version moves with the badge's width.
- Paths: a recent project's path is monospace, a recovered project's is not; fleet card summaries mix both fonts.
- The error dialog has a divider above its button and the unsaved-changes dialog has none.
- Resources panel: Import from Library… is centered while the rest is left-aligned, and the drop zone text has a double space before Import….
- The German Images drop zone wraps left-aligned instead of centered.
- Very dark stroke and shadow swatches look like "no color": they get a light inner ring.

**Screenshots**
- The `status_*_before_exporting` screenshots show the column above Before exporting; they scroll to it. The narrow command palette screenshot uses the window's minimum width (960 px): at 480 px it showed the title bar overlapping the menus, which can't happen in the app.

**Left out, on purpose**
- Two opacity controls in the shadow popover (the color's alpha and Opacity): the drop-shadow spec asks for the same color picker as Fill, so this is a design question for later.
- German font weight names ("Bold 700"): they are usually left in English in German apps.
- A one-frame difference between the resize readout and the inspector size, a ruler label at the edge and the Add Vehicle list without column headers: not clearly defects.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `layers-panel`: text layers named after their kind show their first line; a name that is the kind's default in any language is a default name; long names end with "…" with the full name in a tooltip.
- `properties-panel`: the same default-name rule in the header; the header names the symbol while it is edited; the color target mark is not shown while another inspector popover is open.
- `ui-design-system`: one checkbox look everywhere; text that doesn't fit ends with "…" and shows the full text in a tooltip, never cut mid-glyph.
- `app-preferences`: the Template packs section's layout, showing the folder in the file manager and Copy Path, and a missing folder flagged.
- `start-screen`: the empty Recent projects state is centered on the list's width, framed, with a button that creates a project.
- `localization`: layouts fit every language at the minimum window size (960 × 600) too; version ranges are shown in words.

## Impact

- `crates/tp-app/src/ui/`: `custom_vehicle.rs`, `vehicle_dialogs.rs`, `new_project.rs`, `library_dialog.rs`, `export_dialog.rs`, `mod_export_dialog.rs`, `template_pack.rs`, `dialogs.rs`, `menu_bar.rs`, `home.rs`, `palette.rs`, and in `workspace/`: `inspector.rs`, `status_bar.rs`, `spaces/project.rs`, `panels/{layers,vehicle,assets,character,shadow,colors}.rs`, `brand_editor.rs`.
- `crates/tp-ui/src/widgets/`: `paint_row.rs` (dark swatch ring), a shared checkbox and a truncated label helper if none exists.
- `crates/tp-i18n/locales/{en,fr,de,es}/*.ftl`: the wording changes, with new keys for version ranges and counts.
- `crates/tp-app/tests/`: screenshot fixes, and tests for the behavior changes (default names in any language, symbol header, target mark, font picker highlight, version range wording, custom vehicle dialog width).
- New dependency: `opener` (without default features), to show a folder in the system's file manager.
- No change to project files, packages or the exported mod.
