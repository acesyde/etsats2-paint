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

### Requirement: Styles lists
The project's styles SHALL be listed in two places: the **Styles** section of the Workshop's **Resources** tab, and the **Graphic styles** and **Text styles** sections of the Brand space (see the brand-space capability). Each place SHALL show **Graphic styles** and **Text styles** separately.

**The lists:**
- each graphic style is shown with a preview of its fill and stroke and its name;
- each text style is shown with a preview of its fill and stroke, its name in its own font, weight and italic, and its size;
- the styles followed by the selection are marked in the link color and with a link icon, so the mark does not rely on color alone;
- in the Brand space, each style also shows its usage (see the brand-impact capability), as "9 textures · 24 objects" or "Unused";
- with no style, each list shows a short explanation of what styles are for.

**Creating and applying:**
- **New Style from Selection** (+) is offered for each kind. It creates a style from the look of the selected object, named "Style 1", "Style 2"… or "Text style 1"…, and the selected objects that have that look then follow it.
  - A graphic style needs one selected shape or text.
  - A text style needs one selected text.
- Clicking a style in the Resources tab, or choosing it in the inspector's Style row, applies it to the selection. Each selected shape or text, including those inside selected groups, takes the style's look and follows it. A text style applies only to texts.

**Each style's context menu offers:**
- **Rename**, inline;
- **Edit Style…**, for graphic styles only: opens the style's before/after editor (see the brand-impact capability) with its name, fill, stroke and opacity. Nothing changes before Apply to Fleet, which records the new look and name as one undo step; every object following the style changes on every texture and keeps following it. Text styles have no editor;
- **Redefine from Selection**: the style takes the look of the single selected object, and every object following it changes on every texture;
- **Select Users on This Texture**: selects the objects of the active texture that follow the style;
- **Delete**: every object keeps its look and stops following the style.

Every change to styles SHALL be one undo step, and Undo SHALL restore the styles, the looks and the links. When a text is being typed on the canvas, the typing SHALL be ended first, as its own undo step, before the style change.

#### Scenario: Restyle the fleet's lettering
- **WHEN** texts on three textures follow the text style "Lettering" (Inter, 200 px), the user selects one of them, sets its font to Roboto Condensed, and chooses Redefine from Selection on "Lettering"
- **THEN** the texts on the three textures use Roboto Condensed, are laid out again, and follow "Lettering", and one Undo restores Inter on all three

#### Scenario: Apply by clicking
- **WHEN** two rectangles are selected and the user clicks the graphic style "Stripe" in the Resources tab
- **THEN** both get "Stripe"'s fill, stroke and opacity, and the Styles section marks "Stripe"

#### Scenario: New style from a selected object
- **WHEN** one red rectangle with a 4 px black stroke is selected and the user clicks New Style from Selection for graphic styles
- **THEN** the style "Style 1" is created with a red fill, a 4 px black stroke and the rectangle's opacity, and the rectangle follows it

#### Scenario: Delete keeps the looks
- **WHEN** the user deletes the graphic style "Stripe", which five objects follow
- **THEN** the five objects look the same and follow no style

#### Scenario: Applying a style while typing
- **WHEN** the user creates a text, types "Left" and, still typing, clicks the text style "Lettering"
- **THEN** the text "Left" follows "Lettering", and one Undo restores its previous look with the text still there

#### Scenario: Text style needs a text
- **WHEN** only a rectangle is selected
- **THEN** New Style from Selection is disabled for text styles, with a tooltip saying a text must be selected

#### Scenario: Edit a graphic style
- **WHEN** rectangles on three textures follow "Stripe" and the user chooses Edit Style… on it, sets its opacity to 50% and clicks Apply to Fleet
- **THEN** the rectangles on the three textures are at 50% opacity and still follow "Stripe", and one Undo restores their opacity

#### Scenario: No editor for text styles
- **WHEN** the user opens the context menu of the text style "Lettering"
- **THEN** it offers no Edit Style… item, and Redefine from Selection is still offered

### Requirement: Style row and links in the inspector
With a selection of shapes or texts, the inspector SHALL show a **Style** row:
- it names the style the selection follows, in the link color, or reads "None" when it follows none, or "Mixed" when the selected objects follow different styles;
- clicking it lists the project's styles that can apply to the selection (graphic styles, and text styles when the selection holds a text); choosing one applies it as clicking it in the Resources tab does.

The inspector's Fill and Stroke rows SHALL show a color linked to a palette swatch with the swatch's name in the link color. An object following a style and a color linked to a swatch SHALL also be named, so the link does not rely on color alone.

#### Scenario: Follow a style from the inspector
- **WHEN** one rectangle is selected and the user chooses "Stripe" in the inspector's Style row
- **THEN** the rectangle follows "Stripe", and the Style row reads "Stripe" in the link color

#### Scenario: Linked fill
- **WHEN** the selected rectangle's fill is linked to the swatch "Company red"
- **THEN** the Fill row shows "Company red" in the link color

#### Scenario: Own change unlinks
- **WHEN** a rectangle follows "Stripe" and the user changes its opacity
- **THEN** the Style row reads "None"
