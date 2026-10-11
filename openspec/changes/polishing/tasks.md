## 1. Shared helpers

- [x] 1.1 Add `tp_i18n::tr_all(id)` (the message in every supported language plus the current one as shown) and `workspace::is_default_name(kind, name)` (design 1); unit tests: "Text", "Texte", "Texto" and the lengthened "Text" are default for a text, "ACE" is not
- [x] 1.2 Add a truncated single-line label helper in `tp_ui::widgets` that ends with "…" and shows the full text in a tooltip (design 2); a tp-ui test checks a long text is shortened, stays on one line and has the full text as tooltip
- [x] 1.3 Move the square checkbox of New Project / Add Vehicle into `tp_ui::widgets` if it isn't there yet; a tp-ui test checks its checked and unchecked states and accessible role
- [x] 1.4 Add the version range formatter (design 6) with i18n messages in en/fr/de/es; unit tests for `*`, `>=1.56`, `<=1.50`, `=1.53`, `>=1.53, <1.56` and an odd range falling back to its raw text

## 2. Workshop: inspector and layers

- [x] 2.1 Use `is_default_name` in the inspector header and in the Layers rows; text layers named after their kind show their first line with the kind in small type; a kittest test in French shows "ACE" for a text named "Text", and a German one shows "Rectangle" without "Rechteck"
- [x] 2.2 Layers rows: lay the name out truncated with "…" and a tooltip instead of clipping it; screenshot `panels_layers_transform_100` shows "Grey strip…"
- [x] 2.3 Inspector with nothing selected while a symbol is edited: the symbol's name, "Symbol" and its size; kittest test on the symbol editing fixture
- [x] 2.4 Fill/Stroke target mark hidden while the stroke, line or shadow popover is open (design 4); kittest test: open the shadow popover, Fill row not marked, close it, marked again; screenshot `shadow_en_popover_100`
- [x] 2.5 Font picker: highlight the current family when it opens and when the query is cleared; keep the popup inside the panel; kittest test with Barlow Condensed
- [x] 2.6 Stroke row: give the swatch link name priority over the width/position summary so "Company grey" shows whole when it fits (`brand_palette_styles_en`)
- [x] 2.7 Shadow row summary readable ("x 24 · y 24 · blur 16" or the translated form), with the blur kept in long texts; update `inspector-shadow-summary` in the four languages
- [x] 2.8 Text section and Brand style editor: value fields aligned in one label column (`inspector_text_100`, `brand_editor_en_style_100`)
- [x] 2.9 Dark stroke and shadow swatches get a light inner ring in `paint_row.rs` (`canvas_single_100`)
- [x] 2.10 Status bar: template opacity keeps the project's value (or no value) while a symbol is edited instead of "0 %" (`symbols_edit_en`)
- [x] 2.11 Symbol bar wording quotes the symbol name (`symbol-bar-editing`, four languages)

## 3. Workshop: left panel, resources, menus

- [x] 3.1 Left panel vehicle names and version notes on one line, truncated with tooltip (`fleet_project_*`, `sample_*`)
- [x] 3.2 Texture rows: status dot at a fixed position, size right-aligned in a fixed slot (`rect_*_textures_tab`)
- [x] 3.3 Resources panel: asset meta on one line, truncated; Import from Library… left-aligned; no double space in the drop zone (`screen_de_workshop_resources_content_100`, `symbols_panel_*`)
- [x] 3.4 German Images drop zone text centered when it wraps (`brand_copy_from_cabin_de`)
- [x] 3.5 Combine and Align submenu titles with the same inset, height and font as the other menu items (`flip_menu`, `title_bar_en_drawn_workshop_object_100`)
- [x] 3.6 Texture names in the left panel's texture rows end with "…" and show in full on hover instead of being clipped (spec ui-design-system "Text that doesn't fit"); kittest test with a long texture name

## 4. Project space and mod export

- [x] 4.1 Vehicle card header blocks capped at the space left, skipped under 40 px (design 2); `screen_de_project_update_100` and `title_bar_de_drawn_project_960` show no word cut without "…"
- [x] 4.2 Header row at 960 px in German: the subtitle is truncated before the filter, which keeps its width (design 8); kittest test at 960 px in German checks the two rects don't overlap
- [x] 4.3 "Supported by every vehicle" uses the version range formatter; kittest test reads "1.56 or later"
- [x] 4.4 Before exporting: the empty-textures row has one marker, aligned with the others; in Export Mod, Show is centered and no taller than its row (`project_mod_en_column_100`, `mod_export_en_problems_100`)
- [x] 4.5 Export summary wording: "Standard cab (cabin: standard)", no cabin line for trailers; update `mod-summary-cabins` / `mod-summary-every-cabin` in four languages and the tests reading them
- [x] 4.6 To check shown in red in the Export Mod dialog, as elsewhere (`status_en_export_mod_100`)
- [x] 4.7 Picture drop zone keeps its text color while a file is dragged over it (`project_mod_en_drop_100`)
- [x] 4.8 Empty dot readable on a selected texture card (`project_mod_en_column_100`)
- [x] 4.9 FR/ES generated shop icon hint shortened to fit its box (`lang_fr_project_100`, `lang_es_project_100`)

## 5. Dialogs

- [x] 5.1 Custom Vehicle dialog texture rows as a fixed grid with short labels above the combos, long accessible names kept (design 7); kittest test: with a main and an accessory row in a 1440 × 1000 window the dialog's rect is inside the window; `custom_vehicle_en` and `_fr` screenshots
- [x] 5.2 Vehicle Library details line: "Any game version · Main texture: Cabin · 2 accessories" with the version formatter and a plural; four languages; `custom_vehicle_library_*`
- [x] 5.3 Library list: badge in a fixed-width slot so versions line up
- [x] 5.4 Search fields as tall as the combos beside them in Add Vehicle and the Vehicle Library
- [x] 5.5 Section headers kept with a single section: "OFFICIAL" in New Project and Add Vehicle, "TEXTURES" in the command palette; update their tests
- [x] 5.6 Import from Library, Export Texture and the accessory toggles of the fleet cards (`vehicle_dialogs.rs`, `textures_to_paint`) use the square checkbox (1.3); a test fails if `egui::Checkbox` is used in tp-app
- [x] 5.7 Template pack dialog: refusal aligned with the step text; `pack-explanation` says "only once, for all vehicles" in the four languages
- [x] 5.8 About: license text clear of the scrollbar (`official_en_about`)
- [x] 5.9 Preferences: scale value beside its slider; the template pack row's game label on the path's baseline (`lang_*_preferences_100`)
- [x] 5.10 Error and unsaved-changes dialogs share one footer style (`message_100`, `unsaved_prompt_100`)
- [x] 5.11 Paths in one font on the home screen (recent and recovered) and in fleet card summaries (`home_recovered_100`, `fleet_*`)

## 6. Wording

- [x] 6.1 English casing: "Custom Vehicle…" everywhere, "Copy from Cabin" (command and undo); write the rule (Title Case for commands and buttons, small words lowercase; sentence case for labels and hints) in `docs/`, next to the other UI conventions
- [x] 6.2 "Color picker" in English (`custom-colour-picker`)
- [x] 6.3 German: "Konturen erstellen" for Create Outlines (and `reason-combine-text`), "Haupttextur" for the main texture kind, palette hint without an orphan "+."

## 7. Screenshots and wrap-up

- [x] 7.1 `status_*_before_exporting` scrolled to Before exporting; `palette_*_narrow` at 960 px
- [x] 7.2 Run the screenshots and check every item of the proposal on its screenshot, in English and German, and in the long-text mode where one exists
- [x] 7.3 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` and `cargo test --workspace --release --locked` pass
- [x] 7.4 Update `docs/roadmap.md`: add `polishing` (this pass) to Shipped, keep the `polishing` row for later passes
- [x] 7.5 Home screen: the empty Recent projects state centered on the list's width, in a frame, with a Create Your First Project button (spec start-screen); test `ui::empty_recent_projects_centered_with_a_create_button`, screenshots `lang_*_home_100`
