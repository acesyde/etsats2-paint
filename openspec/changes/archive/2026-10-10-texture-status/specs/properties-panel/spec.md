## ADDED Requirements

### Requirement: On this texture summary
With nothing selected, the inspector SHALL show, under the texture's properties, a section **On this texture** with three counts of the active texture:
- **Objects:** its top-level objects, as counted in the Layers tab's header;
- **Symbol instances:** the symbol instances it holds, inside groups too;
- **Off-palette colors:** its off-palette colors (see the texture-status capability).

The counts SHALL follow every edit. When the off-palette count is above zero, it SHALL be shown in the signal color and be a button, named for assistive technologies "Select objects with off-palette colors"; clicking it SHALL select the objects using them that are unlocked (objects inside groups are selected themselves, as in the Layers tab), and the inspector then shows that selection. When some of those objects are locked, the count's tooltip SHALL say how many are left out of the selection. When every one of them is locked, or the count is zero, the count is plain text.

While a symbol is being edited, the section SHALL read **In this symbol** and count the symbol's objects and off-palette colors, with no Symbol instances line (symbols don't nest).

#### Scenario: Summary of a texture
- **WHEN** nothing is selected and the active texture holds four top-level objects, none of them an instance, using two unlinked colors
- **THEN** the inspector shows On this texture with Objects 4, Symbol instances 0 and Off-palette colors 2, the 2 in the signal color

#### Scenario: Selecting the off-palette objects
- **WHEN** the active texture holds a group "Stripes" whose two rectangles are filled with unlinked colors, and a text filled with a linked swatch, and the user clicks the Off-palette colors count
- **THEN** the two rectangles are selected, not the group nor the text, and the inspector shows "2 objects"

#### Scenario: Instances counted, their colors not
- **WHEN** the active texture holds two instances of "Logo", one of them inside a group, and nothing else
- **THEN** On this texture shows Objects 2, Symbol instances 2 and Off-palette colors 0

#### Scenario: Editing a symbol
- **WHEN** the user edits the symbol "Logo", whose content is a circle filled with an unlinked color, and nothing is selected
- **THEN** the inspector shows In this symbol with Objects 1 and Off-palette colors 1, and no Symbol instances line

## MODIFIED Requirements

### Requirement: Selection summary
The inspector header SHALL show what is selected: the object's name and kind (name and kind icon) for a single object or group, or "N objects" for several. With nothing selected, the inspector SHALL show the active texture's properties instead:
- its name, its kind (main texture or accessory texture) and its size in pixels;
- when a package update flagged it, that it is **To check** (see the texture-status capability) and why: its layout changed in the vehicle's version, with a **Mark as Checked** action, or it is not in this version and is left out of the mod, with no action;
- a hint to select an object to set its layout, fill and stroke.

The template's Show Template and opacity settings SHALL NOT be in the inspector: they are view settings in the status bar (see the workspace-layout capability).

#### Scenario: Multiple selection summary
- **WHEN** three objects are selected
- **THEN** the inspector header reads "3 objects"

#### Scenario: Nothing selected
- **WHEN** nothing is selected and the active texture is the 4096 px main texture "Cabin"
- **THEN** the inspector shows "Cabin", that it is a main texture, "4096 × 4096 px" and the hint, and no template show or opacity setting

#### Scenario: Layout changed notice
- **WHEN** nothing is selected and an update to version 1.3.0 flagged the active texture's layout as changed
- **THEN** the inspector says the texture is To check because its layout changed in 1.3.0, with a Mark as Checked action, and clicking Mark as Checked removes the notice

#### Scenario: Not in this version notice
- **WHEN** nothing is selected and the active texture is marked "Not in this version"
- **THEN** the inspector says the texture is To check because it is not in this version and is left out of the mod, with no Mark as Checked action

### Requirement: Inspector sections
The Properties panel SHALL become the **inspector**, on the right of the canvas in the Workshop space (see the workspace-layout capability). The inspector SHALL show only the settings that apply to what is selected, never an empty state such as "Nothing to transform". With a selection, it SHALL show these sections, in this order, each one only when it applies:

1. **Header:** the selection summary (see Selection summary) and its symbol actions (see Symbol actions in the header).
2. **Layout:** position, size, rotation, align, distribute, combine and flip (see the transform-panel capability).
3. **Text:** the character settings, only when the selection contains texts (see Character settings section).
4. **Appearance:** opacity, the **Fill** row, the **Stroke** row (see the color-panel and stroke-panel capabilities), the line settings for open paths (see Line settings) and the corner radius for rectangles. Fill and Stroke are left out when they don't apply (images; instances, whose look is edited in the symbol).
5. **Polygon:** the polygon settings (Sides, Star and, for stars, Inner radius), only when every selected object is a polygon (see Polygon section).
6. **Style:** the style the selection follows (see Style row), for shapes and texts.
7. **Image:** the image information, for a single image (see Image information).

With nothing selected, the inspector SHALL show the texture's properties (see Selection summary), then On this texture (see On this texture summary), then the look used for new objects (see Fill and stroke swatches) and, when the Text tool is active, the Text section for new texts.

The polygon settings for new polygons are in the Polygon tool's options bar (see the workspace-layout capability); the inspector's Polygon section edits the selected polygons.

#### Scenario: Sections of a text
- **WHEN** a single text is selected
- **THEN** the inspector shows, in order, the header, Layout, Text, Appearance and Style, and no Image section

#### Scenario: Sections of a rectangle
- **WHEN** a single rectangle is selected
- **THEN** the inspector shows the header, Layout, Appearance with the corner radius, and Style, and no Text section

#### Scenario: Sections of a polygon
- **WHEN** a single polygon is selected
- **THEN** the inspector shows, in order, the header, Layout, Appearance, Polygon and Style, and no Text or Image section

#### Scenario: Sections of an image
- **WHEN** a single image is selected
- **THEN** the inspector shows the header, Layout, Appearance with opacity and without Fill and Stroke rows, and Image

#### Scenario: Never empty
- **WHEN** nothing is selected
- **THEN** the inspector shows the texture's properties, then On this texture, and no section reads "Nothing to transform" or another empty state
