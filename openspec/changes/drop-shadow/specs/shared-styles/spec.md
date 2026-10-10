## MODIFIED Requirements

### Requirement: Graphic and text styles
A project SHALL hold two ordered lists of named styles, both starting empty:
- **graphic styles:** a fill, a stroke (or none), an opacity and a shadow (or none);
- **text styles:** the whole look of a lettering. That is a character style (font family, weight, italic, size, alignment, letter spacing and line height), plus a fill, a stroke (or none), an opacity and a shadow (or none).

A style's colors MAY be linked to palette swatches. Editing such a swatch SHALL update the style, and every object following the style.

Style names SHALL NOT be empty. Two styles of the same kind SHALL NOT have the same name.

#### Scenario: A new project has no style
- **WHEN** a project is created
- **THEN** it has no graphic style and no text style

#### Scenario: A style follows its swatch
- **WHEN** the graphic style "Stripe" fills with the swatch "Company red", and the user edits "Company red" to dark red
- **THEN** "Stripe" fills with dark red, and so does every object following "Stripe"

#### Scenario: A style carries the shadow
- **WHEN** a text with a black shadow, offset 8 / 8, blur 8 is saved with New Style from Selection as the text style "Lettering", and the user applies "Lettering" to a text without shadow
- **THEN** that text gets the same shadow and follows "Lettering"

### Requirement: Following a style
An object SHALL follow at most one style. A shape follows graphic styles. A text follows either a graphic style or a text style, not both: applying one replaces the other.

While it follows a style:
- **graphic style:** the object's fill, stroke, opacity and shadow are the style's;
- **gradients:** the style gives the gradient's kind and stops, but the gradient's position on the object (its start, end and radii) stays the object's own;
- **text style:** a text's character settings, fill, stroke, opacity and shadow are the style's, with the same rule for gradients.

The object SHALL stop following the style when its own look stops being equal to the style's. This happens when:
- the user changes its fill, stroke, opacity or shadow (adding, editing or removing it), or, for a text style, its character settings;
- the user picks another swatch, another style, or the eyedropper's color for it.

Moving, resizing, rotating, flipping, duplicating, grouping or copying an object SHALL NOT make it stop following its style. Neither SHALL moving it to another texture, or converting a shape to a path.

A group SHALL NOT follow a style. Applying a style to a group applies it to every shape and text inside. Applying a graphic style to an image does nothing.

#### Scenario: Moving keeps the link
- **WHEN** a rectangle with a linear gradient follows the graphic style "Fade", and the user rotates it by 30°
- **THEN** it still follows "Fade", and its gradient turns with it

#### Scenario: Own change detaches
- **WHEN** a text follows the text style "Lettering" and the user sets its size to 300
- **THEN** it no longer follows "Lettering", and editing "Lettering" afterwards doesn't change it

#### Scenario: A text style carries the whole lettering
- **WHEN** a text in blue with a 6 px red stroke, at 70% opacity, in Inter Black 400 px is saved as the text style "Lettering", and the user applies "Lettering" to another text
- **THEN** the other text is blue with a 6 px red stroke, at 70% opacity, in Inter Black 400 px, and follows "Lettering"

#### Scenario: Recoloring a lettering detaches it
- **WHEN** a text follows the text style "Lettering" and the user gives it a green fill
- **THEN** it no longer follows "Lettering"

#### Scenario: Group
- **WHEN** a group of two rectangles is selected and the user applies the graphic style "Stripe"
- **THEN** both rectangles follow "Stripe", and the group follows no style

#### Scenario: Changing the shadow detaches
- **WHEN** a text follows "Lettering", which has a shadow, and the user removes the text's shadow
- **THEN** the text no longer follows "Lettering"
