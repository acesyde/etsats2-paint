# shared-styles Specification

## Purpose

Lets a project share named looks across every texture of its fleet: graphic styles (fill, stroke and opacity) and text styles (the whole lettering: character settings, fill, stroke and opacity), which objects follow until their own look is changed.

## Requirements

### Requirement: Graphic and text styles
A project SHALL hold two ordered lists of named styles, both starting empty:
- **graphic styles:** a fill, a stroke (or none) and an opacity;
- **text styles:** the whole look of a lettering. That is a character style (font family, weight, italic, size, alignment, letter spacing and line height), plus a fill, a stroke (or none) and an opacity.

A style's colors MAY be linked to palette swatches. Editing such a swatch SHALL update the style, and every object following the style.

Style names SHALL NOT be empty. Two styles of the same kind SHALL NOT have the same name.

#### Scenario: A new project has no style
- **WHEN** a project is created
- **THEN** it has no graphic style and no text style

#### Scenario: A style follows its swatch
- **WHEN** the graphic style "Stripe" fills with the swatch "Company red", and the user edits "Company red" to dark red
- **THEN** "Stripe" fills with dark red, and so does every object following "Stripe"

### Requirement: Following a style
An object SHALL follow at most one style. A shape follows graphic styles. A text follows either a graphic style or a text style, not both: applying one replaces the other.

While it follows a style:
- **graphic style:** the object's fill, stroke and opacity are the style's;
- **gradients:** the style gives the gradient's kind and stops, but the gradient's position on the object (its start, end and radii) stays the object's own;
- **text style:** a text's character settings, fill, stroke and opacity are the style's, with the same rule for gradients.

The object SHALL stop following the style when its own look stops being equal to the style's. This happens when:
- the user changes its fill, stroke or opacity, or, for a text style, its character settings;
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

### Requirement: Styles panel
The workspace SHALL offer a **Styles** panel in the right panel stack. It SHALL have a **Graphic styles** section and a **Text styles** section.

**The lists:**
- each graphic style is shown with a preview of its fill and stroke and its name;
- each text style is shown with a preview of its fill and stroke, its name in its own font, weight and italic, and its size;
- the styles followed by the selection are marked, without relying on color alone;
- with no style, each section shows a short explanation of what styles are for.

**Creating and applying:**
- **New Style from Selection** (+) is offered in each section. It creates a style from the look of the selected object, named "Style 1", "Style 2"… or "Text style 1"…, and the selected objects that have that look then follow it.
  - A graphic style needs one selected shape or text.
  - A text style needs one selected text.
- Clicking a style applies it to the selection. Each selected shape or text, including those inside selected groups, takes the style's look and follows it. A text style applies only to texts.

**Each style's context menu offers:**
- **Rename**, inline;
- **Redefine from Selection**: the style takes the look of the single selected object, and every object following it changes on every texture;
- **Select Users on This Texture**: selects the objects of the active texture that follow the style;
- **Delete**: every object keeps its look and stops following the style.

Every change to styles SHALL be one undo step, and Undo SHALL restore the styles, the looks and the links. When a text is being typed on the canvas, the typing SHALL be ended first, as its own undo step, before the style change.

#### Scenario: Restyle the fleet's lettering
- **WHEN** texts on three textures follow the text style "Lettering" (Inter, 200 px), the user selects one of them, sets its font to Roboto Condensed, and chooses Redefine from Selection on "Lettering"
- **THEN** the texts on the three textures use Roboto Condensed, are laid out again, and follow "Lettering", and one Undo restores Inter on all three

#### Scenario: Apply by clicking
- **WHEN** two rectangles are selected and the user clicks the graphic style "Stripe"
- **THEN** both get "Stripe"'s fill, stroke and opacity, and the Styles panel marks "Stripe"

#### Scenario: New style from a selected object
- **WHEN** one red rectangle with a 4 px black stroke is selected and the user clicks New Style from Selection in Graphic styles
- **THEN** the style "Style 1" is created with a red fill, a 4 px black stroke and the rectangle's opacity, and the rectangle follows it

#### Scenario: Delete keeps the looks
- **WHEN** the user deletes the graphic style "Stripe", which five objects follow
- **THEN** the five objects look the same and follow no style

#### Scenario: Applying a style while typing
- **WHEN** the user creates a text, types "Left" and, still typing, clicks the text style "Lettering"
- **THEN** the text "Left" follows "Lettering", and one Undo restores its previous look with the text still there

#### Scenario: Text style needs a text
- **WHEN** only a rectangle is selected
- **THEN** New Style from Selection is disabled in Text styles, with a tooltip saying a text must be selected
