## MODIFIED Requirements

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
