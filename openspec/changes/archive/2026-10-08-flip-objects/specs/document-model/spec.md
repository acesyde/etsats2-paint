## MODIFIED Requirements

### Requirement: Text and image objects
A text object SHALL hold its content (one or more lines) and its character style; its frame is derived from its laid-out bounds and scale. An image object SHALL reference a project asset and keep its own frame; it has no fill or stroke. Both kinds SHALL support opacity, visibility, lock, grouping and all transforms.

A text or an image SHALL also have a mirrored state, off by default. A mirrored object SHALL be drawn reversed across its own vertical axis, within the same frame. A vertical mirror is a mirrored object turned by a half-turn. The mirrored state SHALL apply wherever the object is drawn or read:

- on the canvas, in the 3D preview, in texture and mod export, and inside symbol instances;
- for an image, when the eyedropper samples it: it SHALL take the pixel shown under the pointer;
- for a text, while editing it: the caret and the selected characters SHALL be placed on the mirrored glyphs, and clicking places the caret under the pointer;
- for a text, when it is outlined: Create Outlines SHALL give mirrored letters.

The mirrored state SHALL NOT change the object's bounds, hit testing, size or its links to swatches and styles.

#### Scenario: Text bounds follow content
- **WHEN** characters are added to a text
- **THEN** its width grows and the selection bounds follow

#### Scenario: A mirrored image is exported mirrored
- **WHEN** a mirrored image of a horse facing left on the canvas is exported with Export Texture
- **THEN** the exported texture shows the horse facing left, at the same place

#### Scenario: Editing a mirrored text
- **WHEN** a mirrored text "TRANS" is double-clicked and the user types "PORT" at its end
- **THEN** the text reads "TRANSPORT", stays mirrored, and the new letters appear on the side where the text ends when mirrored

#### Scenario: Outlining a mirrored text
- **WHEN** a mirrored text is converted with Create Outlines
- **THEN** the letter paths are mirrored like the text was, at the same place

#### Scenario: Eyedropper on a mirrored image
- **WHEN** an image is red on its left half and blue on its right half, is mirrored, and the eyedropper is clicked on its left half
- **THEN** the picked color is blue
