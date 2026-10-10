## MODIFIED Requirements

### Requirement: Brand palette
The **Palette** section SHALL list the project's swatches, each with its color, its name, its hex value and its usage (see the brand-impact capability). It SHALL offer:
- **New Color** (+) in the section header: adds the current target color to the palette, with the rules of the color-panel's Add to palette (no duplicates; the current target becomes linked to the new swatch);
- in each swatch's context menu: **Edit Swatch…**, which opens the swatch's before/after editor as a panel on the right of the Brand space (it is also how a swatch is renamed), **Add to Library** or **Update in Library**, and **Delete Swatch**.

Each change SHALL be one undo step, as in the color-panel capability.

The editor panel SHALL keep the sections visible on its left, with the edited element marked. Opening another element's editor SHALL replace the open one, leaving the project untouched, as Cancel does. Switching to another space SHALL cancel the open editor. With no swatch, the section SHALL show a short explanation of what the palette is for.

#### Scenario: Recolor from Brand
- **WHEN** objects on two textures are linked to the swatch "Company red" and the user chooses Edit Swatch… on it in Brand, enters `#8B0000` and clicks Apply to Fleet
- **THEN** those objects are dark red on both textures, and one Undo restores the previous red everywhere

#### Scenario: Rename a swatch
- **WHEN** the user opens Edit Swatch… on "Company red" in Brand, types "Ardent red" as its name and clicks Apply to Fleet
- **THEN** the palette lists "Ardent red" with the same color

#### Scenario: Usage on a swatch card
- **WHEN** "Cream" is used by 24 objects on 11 textures and "Black" by nothing
- **THEN** the "Cream" card reads "11 textures · 24 objects" and the "Black" card reads "Unused"

#### Scenario: Switching space cancels the editor
- **WHEN** the editor of "Company red" is open with `#8B0000` entered and the user presses `Cmd/Ctrl+2`
- **THEN** the Workshop is shown, "Company red" keeps its previous color, and nothing is added to the undo history

### Requirement: Brand styles
The **Graphic styles** and **Text styles** sections SHALL list the project's styles as the shared-styles capability describes them: a graphic style with a preview of its fill and stroke and its name; a text style with a preview of its fill and stroke, its name in its own font, weight and italic, and its size. Each style SHALL show its usage (see the brand-impact capability). The styles followed by the selection SHALL be marked, without relying on color alone.

Each section header SHALL offer **New Style from Selection** (+), with the same rules and disabled tooltips as in the shared-styles capability. Each style's context menu SHALL offer **Rename** (inline), **Edit Style…** (graphic styles only; it opens the style's before/after editor as a panel on the right, like Edit Swatch…), **Redefine from Selection**, **Select Users on This Texture**, **Add to Library** or **Update in Library**, and **Delete**, with the same effects. Select Users on This Texture SHALL also show the Workshop, so that the selection is visible.

Every change SHALL be one undo step. With no style, each section SHALL show the explanation of what styles are for.

#### Scenario: Redefine from Brand
- **WHEN** a text follows the text style "Lettering" and is selected with its font changed to Roboto Condensed, and the user chooses Redefine from Selection on "Lettering" in Brand
- **THEN** every text following "Lettering", on every texture, uses Roboto Condensed

#### Scenario: Select users from Brand
- **WHEN** three rectangles of the active texture follow "Stripe" and the user chooses Select Users on This Texture on "Stripe" in Brand
- **THEN** the Workshop is shown with the three rectangles selected

#### Scenario: Edit Style from Brand
- **WHEN** the user chooses Edit Style… on "Stripe" in Brand
- **THEN** the before/after editor opens on the right with "Stripe"'s fill, stroke and opacity, its usage as the impact, and the thumbnails of the textures holding objects that follow it

### Requirement: Brand symbols
The **Symbols** section SHALL list the project's symbols, each with a symbol icon, its name, its number of instances in the project and the number of textures holding them ("14 instances · 9 textures", see the brand-impact capability). Its header SHALL offer **Create from Selection**, which runs Object › Convert to Symbol and is enabled under the same rule.

Each symbol SHALL offer **Place**, **Edit**, **Rename** (inline), **Duplicate**, **Add to Library** or **Update in Library**, and **Delete**, with the effects and the delete confirmation of the symbols capability. In Brand:
- **Edit** SHALL open the symbol's edit view in the Workshop;
- **Place** SHALL add the instance to the active texture, select it, and show the Workshop. It is disabled while a symbol is being edited, with a tooltip saying why.

Each change SHALL be one undo step. With no symbol, the section SHALL show the explanation of what symbols are and how to make one.

#### Scenario: Edit a symbol from Brand
- **WHEN** the user clicks Edit on "Logo" in Brand
- **THEN** the Workshop is shown with "Logo" being edited, and Done returns to the texture that was active

#### Scenario: Place from Brand
- **WHEN** the Chassis is the active texture and the user clicks Place on "Logo" in Brand
- **THEN** the Workshop shows the Chassis with a new instance of "Logo" at 100% in the middle of the view, selected

#### Scenario: Create a symbol from Brand
- **WHEN** a circle and a text are selected and the user clicks Create from Selection in Brand
- **THEN** the symbol "Symbol 1" is created as by Convert to Symbol, and the Symbols section lists it with "1 instance · 1 texture"
