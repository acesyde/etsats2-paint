# assets-panel Specification

## Purpose

Shows the images and SVG files imported into the project and lets users reuse, rename or clean them up.

## Requirements

### Requirement: Asset list
The project's assets SHALL be listed in two places: the **Images** section of the Workshop's **Resources** tab, and the **Images** section of the Brand space. Both SHALL list every project asset with a thumbnail, its name, its pixel size or "Vector" for SVG, and the number of objects using it. Templates SHALL NOT be listed, unless an image uses the same file too.

With no assets, the Images section of the Resources tab SHALL show a drop zone reading "Drop a logo here or Import…":
- **Import…** SHALL run File › Place…;
- PNG, JPG/JPEG and SVG files dropped from the operating system onto the zone SHALL be imported as by File › Place…, in the middle of the visible part of the active texture.

With no assets, the Images section of Brand SHALL show a short explanation and its **Import…** button.

#### Scenario: Asset listed after import
- **WHEN** the user places "logo.svg"
- **THEN** the Images section of the Resources tab and the Images section of Brand list "logo" with "Vector" and "1 use"

#### Scenario: Import from the empty Images section
- **WHEN** the project has no asset and the user clicks Import… in the Images section of the Resources tab
- **THEN** the File › Place… dialog opens

#### Scenario: Drop a file on the drop zone
- **WHEN** the project has no asset and the user drops "logo.png" from the operating system onto the Images drop zone
- **THEN** "logo.png" is placed on the active texture and selected, and the Images section lists "logo"

### Requirement: Asset actions
Each asset SHALL offer: Place (adds a new image object of that asset centered in the view), Rename (inline; renames the asset, not existing objects) and Remove (only enabled when the asset is unused, with a tooltip explaining why otherwise). Dragging an asset row of the Resources tab onto the canvas SHALL place it at the drop point. Place from the Brand space SHALL add the image to the active texture and show the Workshop. Removing an asset SHALL be undoable.

#### Scenario: Remove an unused asset
- **WHEN** every object using "badge" has been deleted and the user removes the "badge" asset
- **THEN** it disappears from the list, and Undo brings it back

#### Scenario: Remove is disabled while used
- **WHEN** an asset is used by one object
- **THEN** its Remove action is disabled with the tooltip "Used by 1 object"

#### Scenario: Drag from the Resources tab
- **WHEN** the user drags the "logo" row of the Resources tab onto the canvas and releases it
- **THEN** an image of "logo" is placed at the drop point
