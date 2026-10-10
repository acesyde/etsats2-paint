## MODIFIED Requirements

### Requirement: Add to Library
The context menu of a symbol, of a swatch and of a style SHALL offer **Add to Library** wherever the element is listed: symbols and styles in the Workshop's Resources tab and in the Brand space; swatches in the palette of the color popover, in the Palette section of the Resources tab, and in the Brand space. It SHALL copy the element into the library with everything it uses:
- a symbol, with the images its objects show, the swatches their fills, strokes, gradient stops and shadows link to, and the styles they follow;
- a style, with the swatches its look links to, its shadow's included;
- a swatch, alone.

The element and each dependency SHALL be linked to their library entry. When the element is already linked to an entry of the library, the item SHALL read **Update in Library** and SHALL replace that entry's content and name with the project's. Dependencies are added the same way: linked entries are updated, and the others are added. Adding to the library SHALL NOT change the project's content and SHALL NOT add an undo step. The library origins it records are saved with the project (the project shows unsaved changes) and are not part of the undo history: Undo and Redo keep them.

#### Scenario: Adding a symbol brings its dependencies
- **WHEN** the symbol "Logo Ardent" holds an image and a text whose fill is linked to the swatch "Vert Ardent" and that follows the text style "Titre", and the user chooses Add to Library on it
- **THEN** the library holds "Logo Ardent", its image, "Vert Ardent" and "Titre"

#### Scenario: Updating a library symbol
- **WHEN** "Logo Ardent" was added to the library, the user edits it in the project and opens its context menu
- **THEN** the menu offers Update in Library, and choosing it replaces the library's "Logo Ardent" with the edited one

#### Scenario: Same item in every list
- **WHEN** the swatch "Vert Ardent" was added to the library and the user opens its context menu in the Brand space, then in the color popover's palette
- **THEN** both menus offer Update in Library

#### Scenario: A shadow's swatch comes along
- **WHEN** the text style "Titre" has a shadow linked to the swatch "Night" and the user chooses Add to Library on "Titre"
- **THEN** the library holds "Titre" and "Night"
