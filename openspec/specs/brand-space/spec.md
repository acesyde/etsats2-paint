# brand-space Specification

## Purpose
Gathers in one space what a project reuses on every texture of its fleet: its palette, graphic styles, text styles, symbols and images, with the actions that create, edit, rename, delete and share them with the personal library.

## Requirements

### Requirement: Brand space
The **Brand** space (`Cmd/Ctrl+3`, or Brand in the space switcher) SHALL show the open project's reusable elements in five sections, in this order: **Palette**, **Graphic styles**, **Text styles**, **Symbols**, **Images**. It SHALL use the full width under the top bar.

On the left, a **section index** SHALL list the five sections. Each entry SHALL show the number of elements of its section ("Palette · 5"). Clicking an entry SHALL scroll its section into view and mark the entry as the current one.

The header SHALL offer **Import from Library…**, which runs the Object menu command of the same name and is enabled under the same rule.

The Brand space SHALL show the project's elements only. It SHALL NOT list the personal library's content: the library is reached through Import from Library… and the Add to Library / Update in Library items.

Switching to Brand and back SHALL keep the selection, the undo history, the zoom and the active texture.

#### Scenario: Open the Brand space
- **WHEN** a project with 5 swatches, 4 graphic styles, 3 text styles, 2 symbols and 1 image is open in the Workshop and the user presses `Cmd/Ctrl+3`
- **THEN** the Brand space shows the Palette, Graphic styles, Text styles, Symbols and Images sections, and the index reads "Palette · 5", "Graphic styles · 4", "Text styles · 3", "Symbols · 2" and "Images · 1"

#### Scenario: Jump to a section
- **WHEN** the user clicks "Symbols" in the section index
- **THEN** the Symbols section is scrolled into view and the "Symbols" entry is marked

#### Scenario: Import from the Brand header
- **WHEN** the user clicks Import from Library… in the Brand header
- **THEN** the Import from Library dialog opens, as from Object › Import from Library…

#### Scenario: No library listing
- **WHEN** the personal library holds the symbol "Logo Ardent" and the project has no symbol
- **THEN** the Symbols section of Brand does not list "Logo Ardent"

### Requirement: Brand palette
The **Palette** section SHALL list the project's swatches, each with its color, its name and its hex value. It SHALL offer:
- **New Color** (+) in the section header: adds the current target color to the palette, with the rules of the color-panel's Add to palette (no duplicates; the current target becomes linked to the new swatch);
- in each swatch's context menu: **Edit Swatch…** (the swatch's name and color, as in the color-panel capability, which is also how a swatch is renamed), **Add to Library** or **Update in Library**, and **Delete Swatch**.

Each change SHALL be one undo step, as in the color-panel capability. With no swatch, the section SHALL show a short explanation of what the palette is for.

#### Scenario: Recolor from Brand
- **WHEN** objects on two textures are linked to the swatch "Company red" and the user chooses Edit Swatch… on it in Brand, enters `#8B0000` and clicks OK
- **THEN** those objects are dark red on both textures, and one Undo restores the previous red everywhere

#### Scenario: Rename a swatch
- **WHEN** the user opens Edit Swatch… on "Company red" in Brand, types "Ardent red" as its name and clicks OK
- **THEN** the palette lists "Ardent red" with the same color

### Requirement: Brand styles
The **Graphic styles** and **Text styles** sections SHALL list the project's styles as the shared-styles capability describes them: a graphic style with a preview of its fill and stroke and its name; a text style with a preview of its fill and stroke, its name in its own font, weight and italic, and its size. The styles followed by the selection SHALL be marked, without relying on color alone.

Each section header SHALL offer **New Style from Selection** (+), with the same rules and disabled tooltips as in the shared-styles capability. Each style's context menu SHALL offer **Rename** (inline), **Redefine from Selection**, **Select Users on This Texture**, **Add to Library** or **Update in Library**, and **Delete**, with the same effects. Select Users on This Texture SHALL also show the Workshop, so that the selection is visible.

Every change SHALL be one undo step. With no style, each section SHALL show the explanation of what styles are for.

#### Scenario: Redefine from Brand
- **WHEN** a text follows the text style "Lettering" and is selected with its font changed to Roboto Condensed, and the user chooses Redefine from Selection on "Lettering" in Brand
- **THEN** every text following "Lettering", on every texture, uses Roboto Condensed

#### Scenario: Select users from Brand
- **WHEN** three rectangles of the active texture follow "Stripe" and the user chooses Select Users on This Texture on "Stripe" in Brand
- **THEN** the Workshop is shown with the three rectangles selected

### Requirement: Brand symbols
The **Symbols** section SHALL list the project's symbols, each with a symbol icon, its name and its number of instances in the project. Its header SHALL offer **Create from Selection**, which runs Object › Convert to Symbol and is enabled under the same rule.

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
- **THEN** the symbol "Symbol 1" is created as by Convert to Symbol, and the Symbols section lists it with 1 instance

### Requirement: Brand images
The **Images** section SHALL list the project's images and SVG files as the assets-panel capability describes them (thumbnail, name, pixel size or "Vector", number of uses), with **Place**, **Rename** and **Remove** under the same rules. Place SHALL add the image to the active texture and show the Workshop.

Its header SHALL offer **Import…**, which runs File › Place… and shows the Workshop with the imported images.

#### Scenario: Import an image from Brand
- **WHEN** the user clicks Import… in the Images section and chooses "logo.png"
- **THEN** the Workshop shows the active texture with "logo.png" placed and selected, and the section lists "logo"

#### Scenario: Remove an unused image
- **WHEN** the asset "badge" is used by no object and the user removes it in Brand
- **THEN** it disappears from the section, and Undo brings it back
