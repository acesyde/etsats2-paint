## MODIFIED Requirements

### Requirement: Breadcrumb
The Workshop SHALL tell which texture is being painted with a breadcrumb `<vehicle> › Main textures|Accessories › <texture>`, followed by the texture's size (`1024²`, or `8192 × 1024` when its sides differ), grouped as in the Textures tab. It SHALL be inlaid in a corner of the canvas area, where it stays visible whatever left tab is shown and when the panels are hidden. It SHALL follow the active texture as soon as it changes. The inlaid breadcrumb SHALL NOT take pointer input from the canvas.

While a symbol is being edited, the breadcrumb SHALL read `Symbol › <name>` instead.

When the breadcrumb does not fit, it SHALL be shortened with an ellipsis and shown in full on hover.

#### Scenario: Accessory texture
- **WHEN** the sample truck's Side skirts (1024 px) is the active texture
- **THEN** the canvas shows "TruckPaint Sample Truck › Accessories › Side skirts 1024²"

#### Scenario: Following the active texture
- **WHEN** the user presses Cmd/Ctrl+] and the next texture is the sample truck's Chassis (4096 px)
- **THEN** the breadcrumb reads "TruckPaint Sample Truck › Accessories › Chassis 4096²"

#### Scenario: Main texture
- **WHEN** the sample truck's High roof is active
- **THEN** the breadcrumb reads "TruckPaint Sample Truck › Main textures › High roof 4096²"

#### Scenario: Editing a symbol
- **WHEN** the user edits the symbol "Logo"
- **THEN** the breadcrumb reads "Symbol › Logo"
