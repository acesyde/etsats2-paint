## 1. Several vehicles in a project

- [x] 1.1 Make `fleet_project` take the whole fleet and share the per-vehicle code with `add_vehicle` (design Decision 2); verify with unit tests: a truck and a trailer give the truck's surfaces then the trailer's with the first one active, the resolution follows the largest texture of all vehicles, a duplicate or another game is refused, and the existing single-vehicle tests pass with a one-element fleet.
- [x] 1.2 Add `Workspace::add_vehicles` in one undo step, checking every vehicle before changing anything (design Decision 3); verify with unit tests: two vehicles added in order with the first new surface active, one Undo removes both, and one refused vehicle leaves the project unchanged.

## 2. Choosing several vehicles

- [x] 2.1 Replace the single choice with an ordered list in `NewProjectDraft` (plus the open card) and `AddVehicleDialog`, with the shared helpers, and make the origins (Custom vehicle, official installs) add to it (design Decision 1); verify with unit tests on the helpers (toggle, remove, open card, game change) and the existing kittest of Custom vehicle and official vehicles passing.
- [x] 2.2 Draw checkbox rows in both lists (design Decision 4) and the texture checkboxes under each checked row in Add Vehicle…; verify with kittest: start-screen "Unchecking in the list" and "Changing the game", vehicle-projects "Several vehicles in one step", and the existing "Custom trailer added to a fleet".
- [x] 2.3 Draw Your fleet as cards: count, open and folded cards, Remove, Textures created for every vehicle, empty state, Create Project enabled with at least one vehicle (design Decisions 5, 6); verify with kittest: start-screen "A truck and a trailer at once", "One card open at a time", "Remove from the fleet", "Custom vehicle added to the fleet", and every existing New Project scenario.
- [x] 2.4 Strings in en/fr/de/es; verify with the i18n key tests and screenshots of New Project with three vehicles checked (one card open) and Add Vehicle… with two checked, in EN and DE at 100 %, with no clipped text.

## 3. Checks

- [x] 3.1 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --release --locked` and the screenshot suite; verify all pass.
