## Why

A TruckPaint project is a fleet: several trucks and trailers painted with one identity and exported as one mod. But New Project lets the player choose only **one** vehicle: clicking another one replaces it, and the rest of the fleet must be added afterwards, one by one, with Add Vehicle…. A player who wants "my DAF and my Krone" starts with half a fleet and has to know where Add Vehicle… is.

The mockup (TruckPaint 5, "02 Nouveau projet") already shows the intended design: a checkbox on each row, and "Your fleet" with one card per chosen vehicle (three in the example), each with **Remove**, and the total of the textures that will be created.

## What Changes

- **New Project: choose several vehicles.**
  - Each row of the vehicle list has a **checkbox**. Clicking a row checks or unchecks it; several trucks and trailers can be checked together, official or installed.
  - **"Your fleet"** shows one card per checked vehicle, in the order they were checked, with the count of vehicles in its title ("3 vehicles"):
    - the open card shows the vehicle's cabins as chips, its main texture mode and its accessories as checkboxes, as today;
    - the other cards are folded to one line: the checked main textures and the count ("1 main + 2 accessories"). Clicking a folded card opens it and folds the open one;
    - each card has **Remove**, which unchecks the vehicle.
  - **Textures created** lists the textures of every vehicle, and their total.
  - **Create Project** creates the project with every checked vehicle, in fleet order. It's enabled when at least one vehicle is checked; each vehicle keeps at least one main texture, as today.
  - With several vehicles and an empty name, the project takes the first vehicle's name, as with one.
- **Add Vehicle…: add several vehicles at once.** The list gets the same checkboxes. Each checked vehicle shows its texture checkboxes under its row. **Add** adds every checked vehicle to the project in one undo step; the first new texture becomes active.
- **Official vehicles and Custom vehicle** follow the same rules: checking an official vehicle installs it first (as today), then checks it; a vehicle made with Custom vehicle… is added to the checked vehicles instead of replacing the choice.
- **Changing the game** in New Project unchecks the vehicles of the other game, as it drops the choice today.

Non-goals: the "Installed only" filter of the mockup; reordering the fleet by hand; adding the same vehicle twice (a project holds each vehicle once).

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `start-screen`: New Project's vehicle list and "Your fleet" handle several vehicles.
- `vehicle-projects`: Add Vehicle… adds several vehicles in one step.

## Impact

- **tp-app:**
  - `NewProjectDraft` and `AddVehicleDialog` hold a list of choices instead of one;
  - the list rows (`ui/new_project.rs`, `ui/vehicle_dialogs.rs`) draw checkboxes; the fleet panel draws a card per vehicle;
  - creating a project from several vehicles; adding several vehicles in one undo step (`vehicle_project.rs`);
  - the custom and official vehicle origins add to the choices instead of replacing them;
  - strings in en/fr/de/es (vehicle count, Remove).
- **Tests:** kittest for both dialogs, the existing single-vehicle scenarios kept; a screenshot of New Project with three vehicles in EN and DE.
