## ADDED Requirements

### Requirement: Inspector sections
The Properties panel SHALL become the **inspector**, on the right of the canvas in the Workshop space (see the workspace-layout capability). The inspector SHALL show only the settings that apply to what is selected, never an empty state such as "Nothing to transform". With a selection, it SHALL show these sections, in this order, each one only when it applies:

1. **Header:** the selection summary (see Selection summary) and its symbol actions (see Symbol actions in the header).
2. **Layout:** position, size, rotation, align, distribute, combine and flip (see the transform-panel capability).
3. **Text:** the character settings, only when the selection contains texts (see Character settings section).
4. **Appearance:** opacity, the **Fill** row, the **Stroke** row (see the color-panel and stroke-panel capabilities), the line settings for open paths (see Line settings) and the corner radius for rectangles. Fill and Stroke are left out when they don't apply (images; instances, whose look is edited in the symbol).
5. **Polygon:** the polygon settings (Sides, Star and, for stars, Inner radius), only when every selected object is a polygon (see Polygon section).
6. **Style:** the style the selection follows (see Style row), for shapes and texts.
7. **Image:** the image information, for a single image (see Image information).

With nothing selected, the inspector SHALL show the texture's properties (see Selection summary) and, below them, the look used for new objects (see Fill and stroke swatches) and, when the Text tool is active, the Text section for new texts.

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
- **THEN** the inspector shows the texture's properties, and no section reads "Nothing to transform" or another empty state

### Requirement: Symbol actions in the header
The inspector header SHALL offer the symbol actions of the selection, running the same commands as the Object menu:
- **Convert to Symbol** when the selection holds at least one object that is not an instance (see the symbols capability);
- for a selection of instances only: the symbol's name (for a single symbol) marked in the link color (see the ui-design-system capability) and with a symbol icon, so the link does not rely on color alone; **Edit Symbol** (one symbol) and **Detach Instance**; and the note that the instance's look is edited in the symbol.

#### Scenario: Instance header
- **WHEN** a single instance of the symbol "Logo" is selected
- **THEN** the header shows "Logo" in the link color with the symbol icon, Edit Symbol and Detach Instance, and no Convert to Symbol

#### Scenario: Convert from the header
- **WHEN** a circle and a text are selected and the user clicks Convert to Symbol in the inspector header
- **THEN** they become one instance of a new symbol, as with Object › Convert to Symbol

### Requirement: Style row
When the selection contains shapes or texts, the inspector SHALL show a **Style** row with the name of the style the selection follows, or "None" when it follows none, and "Mixed" when the selected objects follow different styles. A followed style SHALL be shown in the link color with a style icon, so the link does not rely on color alone. The row SHALL open a menu that lists the project's graphic styles, and its text styles when the selection holds texts, and offers New Style from Selection. Choosing a style SHALL apply it exactly as clicking it in the Styles list does (see the shared-styles capability), as one undo step.

#### Scenario: Apply from the Style row
- **WHEN** two rectangles following no style are selected, and the user opens the Style row menu and chooses "Stripe"
- **THEN** both rectangles take the look of "Stripe" and follow it, the row reads "Stripe" in the link color, and one Undo restores both

#### Scenario: No style
- **WHEN** a rectangle following no style is selected
- **THEN** the Style row reads "None", not in the link color

### Requirement: Line settings
When every selected object is a path with open subpaths, the Appearance section SHALL show the line settings that the Properties panel shows today (see the path-tools capability): a Width field in the section, and the dashes, caps and joins in a popover opened from it, closed by Escape or a click outside. Their behavior SHALL be unchanged.

#### Scenario: Dashed line from the inspector
- **WHEN** a line is selected and the user opens its line settings and picks the Dashed 20/10 preset
- **THEN** the line becomes dashed, as it did from the Properties panel

### Requirement: Polygon section
When every selected object is a polygon, the inspector SHALL show a **Polygon** section, after Appearance, with a Sides field (3 to 12), a Star toggle and, when Star is on, an Inner radius field (10% to 90%), whatever the active tool. A change SHALL apply to every selected polygon as in the path-tools capability (Polygon settings): it keeps each polygon's bounds, center and rotation, is one undo step, and differing values show "Mixed". The values set SHALL also become the settings used for new polygons, as when they are set in the Polygon tool's options bar (see the workspace-layout capability). With a selection that holds anything other than polygons, the section SHALL NOT be shown.

#### Scenario: Star from the inspector
- **WHEN** two hexagons are selected with the Selection tool active and the user sets Sides to 5 and turns Star on in the Polygon section
- **THEN** both become five-point stars in their bounds, two undo steps were recorded (one per change), and the Polygon tool's options bar shows 5 sides and Star on

#### Scenario: Mixed values
- **WHEN** a hexagon and an octagon are selected
- **THEN** the Polygon section's Sides field shows "Mixed"

#### Scenario: Not for a mixed selection
- **WHEN** a polygon and a rectangle are selected
- **THEN** the inspector shows no Polygon section

## MODIFIED Requirements

### Requirement: Selection summary
The inspector header SHALL show what is selected: the object's name and kind (name and kind icon) for a single object or group, or "N objects" for several. With nothing selected, the inspector SHALL show the active texture's properties instead:
- its name, its kind (main texture or accessory texture) and its size in pixels;
- what a package update flagged for it, with its actions, as today (see the vehicle-projects capability): "Layout changed" with Dismiss, or "Not in this version";
- a hint to select an object to set its layout, fill and stroke.

The template's Show Template and opacity settings SHALL NOT be in the inspector: they are view settings in the status bar (see the workspace-layout capability).

#### Scenario: Multiple selection summary
- **WHEN** three objects are selected
- **THEN** the inspector header reads "3 objects"

#### Scenario: Nothing selected
- **WHEN** nothing is selected and the active texture is the 4096 px main texture "Cabin"
- **THEN** the inspector shows "Cabin", that it is a main texture, "4096 × 4096 px" and the hint, and no template show or opacity setting

#### Scenario: Layout changed notice
- **WHEN** nothing is selected and an update flagged the active texture's layout as changed
- **THEN** the inspector says "Layout changed" with a Dismiss action, and clicking Dismiss removes the notice

### Requirement: Opacity
The Appearance section of the inspector SHALL provide an opacity field and slider from 0% to 100% applying to every selected object. A slider drag SHALL be one undo step. Differing values SHALL show "Mixed".

#### Scenario: Set opacity
- **WHEN** an object is selected and the user sets opacity to 40% in the inspector
- **THEN** the object is drawn at 40% opacity and Undo restores 100%

### Requirement: Corner radius
When every selected object is a rectangle, the Appearance section of the inspector SHALL show a corner radius field (texture pixels, clamped to half of the smaller side). It SHALL be hidden otherwise.

#### Scenario: Rounded rectangle
- **WHEN** a 400×200 rectangle is selected and the user sets the corner radius to 300
- **THEN** the radius is clamped to 100

#### Scenario: No radius for an ellipse
- **WHEN** an ellipse is selected
- **THEN** the inspector shows no corner radius field

### Requirement: Fill and stroke swatches
The Appearance section SHALL show a **Fill** row and, under it, a **Stroke** row for the selection's shapes and texts (see the color-panel and stroke-panel capabilities). Each row's swatch SHALL show the paint (a "None" pattern for no stroke and a "Mixed" pattern for differing paints). Clicking a swatch SHALL make it the color target and open the color popover for it. With nothing selected, the inspector SHALL show the Fill and Stroke rows of the look used for new objects, under a heading saying so.

#### Scenario: Clicking the stroke swatch
- **WHEN** a stroked rectangle is selected and the user clicks the Stroke row's swatch
- **THEN** the stroke becomes the color target and the color popover opens beside the inspector, level with the Stroke row

#### Scenario: Look of new objects
- **WHEN** nothing is selected and the user sets the Fill row of new objects to red, then draws a rectangle
- **THEN** the new rectangle is red

### Requirement: Character settings section
When the selection contains texts (only texts, or texts within groups), the inspector SHALL show a **Text** section, after Layout, with the font family picker, weight, italic toggle, size, alignment (segmented: left, center, right), letter spacing and line height, applying to every selected text; differing values SHALL show "Mixed". With nothing selected and the Text tool active, the section edits the character style used for new texts.

#### Scenario: Change size of two texts
- **WHEN** two texts are selected and the user sets the size to 300
- **THEN** both texts use 300 px and one undo step restores both

#### Scenario: No Text section for shapes
- **WHEN** only a rectangle is selected
- **THEN** the inspector shows no Text section

### Requirement: Image information
When a single image is selected, the inspector SHALL show an **Image** section with its asset name, its kind and source size (pixels or "Vector"), and a "Reset Size" action that restores its placement size and aspect ratio while keeping its center and rotation.

#### Scenario: Reset a distorted image
- **WHEN** an 800×400 image was stretched to 800×800 and the user clicks Reset Size
- **THEN** it is 800×400 again, centered at the same point
