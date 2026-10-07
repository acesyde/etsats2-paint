## MODIFIED Requirements

### Requirement: Selection summary
The Properties panel SHALL show what is selected: the object's name and kind icon for a single object or group, or "N objects" for several. With nothing selected, it SHALL show the active surface name and size, a hint to select an object, and the active surface's **Template** settings (see the vehicle-projects capability): what an update flagged, Show Template and the template opacity.

#### Scenario: Multiple selection summary
- **WHEN** three objects are selected
- **THEN** the panel header area reads "3 objects"

#### Scenario: Nothing selected
- **WHEN** nothing is selected and the active texture is the 4096 px Cabin
- **THEN** the panel shows "Cabin", "4096 × 4096 px", the hint, and the Template settings with Show Template and the opacity slider
