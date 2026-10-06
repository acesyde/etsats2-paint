# assets-panel Specification

## Purpose

Shows the images and SVG files imported into the project and lets users reuse, rename or clean them up.

## Requirements

### Requirement: Asset list
The Assets panel SHALL list every project asset with a thumbnail, its name, its pixel size or "Vector" for SVG, and the number of objects using it. With no assets, it SHALL show an empty state with a "Place…" button.

#### Scenario: Asset listed after import
- **WHEN** the user places "logo.svg"
- **THEN** the Assets panel lists "logo" with "Vector" and "1 use"

### Requirement: Asset actions
Each asset SHALL offer: Place (adds a new image object of that asset centered in the view), Rename (inline; renames the asset, not existing objects) and Remove (only enabled when the asset is unused, with a tooltip explaining why otherwise). Dragging an asset row onto the canvas SHALL place it at the drop point. Removing an asset SHALL be undoable.

#### Scenario: Remove an unused asset
- **WHEN** every object using "badge" has been deleted and the user removes the "badge" asset
- **THEN** it disappears from the list, and Undo brings it back

#### Scenario: Remove is disabled while used
- **WHEN** an asset is used by one object
- **THEN** its Remove action is disabled with the tooltip "Used by 1 object"
