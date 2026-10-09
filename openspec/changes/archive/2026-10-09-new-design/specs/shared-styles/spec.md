## RENAMED Requirements

- FROM: `### Requirement: Styles panel`
- TO: `### Requirement: Styles lists`

## MODIFIED Requirements

### Requirement: Styles lists
The project's styles SHALL be listed in two places: the **Styles** section of the Workshop's **Resources** tab, and the **Graphic styles** and **Text styles** sections of the Brand space (see the brand-space capability). Each place SHALL show **Graphic styles** and **Text styles** separately.

**The lists:**
- each graphic style is shown with a preview of its fill and stroke and its name;
- each text style is shown with a preview of its fill and stroke, its name in its own font, weight and italic, and its size;
- the styles followed by the selection are marked in the link color and with a link icon, so the mark does not rely on color alone;
- with no style, each list shows a short explanation of what styles are for.

**Creating and applying:**
- **New Style from Selection** (+) is offered for each kind. It creates a style from the look of the selected object, named "Style 1", "Style 2"… or "Text style 1"…, and the selected objects that have that look then follow it.
  - A graphic style needs one selected shape or text.
  - A text style needs one selected text.
- Clicking a style in the Resources tab, or choosing it in the inspector's Style row, applies it to the selection. Each selected shape or text, including those inside selected groups, takes the style's look and follows it. A text style applies only to texts.

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

## ADDED Requirements

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
