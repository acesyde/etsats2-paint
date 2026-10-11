## Context

**New Project** (`ui/new_project.rs`):
- `NewProjectDraft` (`state.rs`) holds `vehicle: Option<VehicleChoice>`, where `VehicleChoice { id, version, textures }` is an installed vehicle and its checked textures.
- `list()` paints each row by hand with a radio mark (`vehicle_row`). A click sets the choice.
- `fleet()` draws one vehicle: chips for the main textures, the mode, accessory checkboxes, the count, then Textures created.
- Create Project calls `vehicle_project::fleet_project(name, &package, &textures)`, which builds a `Project` for one vehicle and picks the project's texture resolution from its largest texture.

**Add Vehicle…** (`ui/vehicle_dialogs.rs`):
- `AddVehicleDialog { filter, choice: Option<VehicleChoice>, messages, installing }`.
- `vehicle_list` draws selectable rows, and the selected row shows its texture checkboxes under it.
- Add calls `Workspace::add_vehicle(&package, &textures, now)`: one `edit("undo-add-vehicle")` that checks the game and duplicates, appends the surfaces and the vehicle, and activates the first new surface.

**Coming back to the dialog:**
- `ui/origin.rs` (`Origin::NewProject` / `AddVehicle` / `Library`) brings the dialog back after Custom vehicle or the template pack.
- `choose` / `created` *replace* the choice with the new vehicle. Official installs finish through the same path.

**Mockup** (TruckPaint 5, "02 Nouveau projet"):
- rows with a 16 px square checkbox (4 px corners, light fill and dark ✓ when checked);
- "Votre flotte 3 véhicules": the first card open (cabins, main texture mode, count), the others on one line with their count, each card with "Retirer";
- "Textures créées" at the bottom.

## Goals / Non-Goals

**Goals:**
- Several vehicles in both dialogs, with the single-vehicle path unchanged in behavior: one checked vehicle gives exactly today's project.
- One undo step for Add of several vehicles.
- No new place to learn: same rows, same fleet panel, same texture checkboxes.

**Non-Goals:**
- Reordering the fleet by hand.
- The mockup's "Installed only" filter.
- Changes to the Project space or Textures tab, which already show fleets.

## Decisions

### 1. State: an ordered list of choices plus the open card

- **New Project:** `NewProjectDraft.vehicles: Vec<VehicleChoice>` (fleet order = check order) and `open: Option<String>` (the package id of the open card).
  - Checking appends and opens the new vehicle's card.
  - Unchecking, or Remove, removes the vehicle; if its card was open, the last remaining one opens.
  - Changing the game keeps only the vehicles of the new game.
- **Add Vehicle…:** `AddVehicleDialog.choices: Vec<VehicleChoice>`, with no open card: each checked row shows its textures under it.
- The helpers on the choice list (`toggle`, `remove`, `push`, `retain_game`) live with `VehicleChoice` in `vehicle_dialogs.rs`, shared by both dialogs.
- **Origins:** `Origin::choose` / `created` *add* the vehicle (and open its card) instead of replacing the choice. If the vehicle is already in the list, it stays where it is and its card opens. This covers Custom vehicle and official installs.

*Alternative:* a set without order. Rejected: the fleet order is visible (cards, Textures created, then the project's surfaces), so it must follow what the player did.

### 2. Building a project from several vehicles

- `vehicle_project::fleet_project(name, &[(Package, Vec<String>)])` takes the whole fleet:
  - the resolution comes from the largest texture of every vehicle;
  - the first vehicle's surfaces come first, and its first texture is active.
- The per-vehicle part of `add_vehicle` (resolve textures, build surfaces, push the vehicle) moves into a function on `Project` used by both, so the two paths can't drift.
- Callers with one vehicle pass a one-element slice. The existing tests change only in their call.
- **Duplicates and games:** the dialog can't produce them; `fleet_project` still refuses them (`FleetError::AlreadyThere`, `OtherGame`) as `add_vehicle` does.

### 3. Adding several vehicles in one undo step

- `Workspace::add_vehicles(&[(&Package, &[String])], now)` first checks every vehicle (game, not already in the project, valid textures), so a refusal changes nothing.
- It then does one `edit("undo-add-vehicles")` appending each vehicle in order, and activates the first new surface.
- `add_vehicle` becomes a call with one vehicle, keeping its undo label for that case.

### 4. Rows: checkboxes, as in the mockup

- The radio mark of `vehicle_row` becomes a square checkbox (16 px, `radius::SM`): outlined when unchecked; filled with the primary color and a ✓ in `TEXT_ON_PRIMARY` when checked.
- The row's accessibility role becomes a checkbox with its checked state, so kittest finds rows by role and state.
- A checked row keeps the grey row fill of today's selected row (never a left bar).
- Add Vehicle's `vehicle_list` uses the same row painter as New Project, with the texture checkboxes under each checked row.

### 5. Your fleet: cards

- **Title:** "Your fleet" and the count ("1 vehicle" / "3 vehicles", plural via Fluent).
- **Card header:** the vehicle's name, a muted "{brand} · {kind}" line, and a **Remove** text button on the right.
- **Open card:** today's content of `fleet()` (chips, mode, accessories, count), unchanged.
- **Folded card:** one line, "{checked main texture names, comma-separated} · {count}". The whole card is a button that opens it.
- **Below the cards:** Textures created for every vehicle, in fleet order, then the total. The panel scrolls when the cards don't fit; the Textures created block and Create Project stay visible.

### 6. Strings

New keys in en/fr/de/es:
- the vehicle count (plural);
- Remove;
- the empty fleet ("Check vehicles in the list");
- the disabled Create Project tooltip ("Check at least one vehicle");
- Add's disabled tooltip.

Existing keys are kept for the card content.

## Risks / Trade-offs

- **Texture resolution with mixed fleets.** The project resolution follows the largest texture of all vehicles. That is today's rule applied to the whole fleet, and Add Vehicle already lives with it.
- **Long fleets.** With many vehicles, Textures created grows. It scrolls with the cards, and the total stays visible.
- **Official installs while several are checked.** Checking an uninstalled official vehicle starts its install. The list is disabled during the install (as today), so only one installs at a time, and the others stay checked.
