## MODIFIED Requirements

### Requirement: Inspector sections
The Properties panel SHALL become the **inspector**, on the right of the canvas in the Workshop space (see the workspace-layout capability). The inspector SHALL show only the settings that apply to what is selected, never an empty state such as "Nothing to transform". With a selection, it SHALL show these sections, in this order, each one only when it applies:

1. **Header:** the selection summary (see Selection summary) and its symbol actions (see Symbol actions in the header).
2. **Layout:** position, size, rotation, align, distribute, combine and flip (see the transform-panel capability).
3. **Text:** the character settings, only when the selection contains texts (see Character settings section).
4. **Appearance:** opacity, the **Fill** row, the **Stroke** row (see the color-panel and stroke-panel capabilities), the **Shadow** row or + Add a shadow (see the drop-shadow capability), the line settings for open paths (see Line settings) and the corner radius for rectangles. Fill and Stroke are left out when they don't apply (images; instances, whose look is edited in the symbol). The Shadow row is left out for groups and instances.
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
- **THEN** the inspector shows the header, Layout, Appearance with opacity and + Add a shadow but without Fill and Stroke rows, and Image

#### Scenario: Never empty
- **WHEN** nothing is selected
- **THEN** the inspector shows the texture's properties, then On this texture, and no section reads "Nothing to transform" or another empty state
