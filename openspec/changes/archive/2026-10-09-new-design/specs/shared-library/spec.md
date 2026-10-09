## MODIFIED Requirements

### Requirement: Add to Library
The context menu of a symbol, of a swatch and of a style SHALL offer **Add to Library** wherever the element is listed: symbols and styles in the Workshop's Resources tab and in the Brand space; swatches in the palette of the color popover, in the Palette section of the Resources tab, and in the Brand space. It SHALL copy the element into the library with everything it uses:
- a symbol, with the images its objects show, the swatches their fills, strokes and gradient stops link to, and the styles they follow;
- a style, with the swatches its look links to;
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

### Requirement: Import from Library
**Import from Library…** SHALL be available when a project is open, except while a symbol is being edited, from three places: the Object menu, a button in the header of the Brand space, and a button in the Workshop's Resources tab. Each SHALL be disabled under the same rule, with a tooltip saying why. It SHALL open a dialog listing the library's symbols, swatches, graphic styles and text styles in four groups, each with its name and a preview (the color for a swatch, the symbol's content for a symbol). Each element has a checkbox:
- an element whose library entry the project already has SHALL be shown as "In this project", and can't be checked;
- **Import** SHALL import the checked elements following the import rules, as one undo step "Import from Library", and close the dialog. It is disabled while nothing is checked;
- **Remove from Library**, in an element's context menu, SHALL delete that library entry after confirmation. Projects keep their copies.

An empty library SHALL show an explanation of Add to Library instead of the groups.

#### Scenario: Importing a symbol
- **WHEN** the library holds "Logo Ardent" (with "Vert Ardent" and "Titre") and the user imports it into a project that has none of them
- **THEN** the project gets the symbol "Logo Ardent", the swatch "Vert Ardent" and the text style "Titre", and one Undo removes all three

#### Scenario: Already in the project
- **WHEN** a project imported "Logo Ardent" earlier and the user opens Import from Library… again
- **THEN** "Logo Ardent" is shown as "In this project" and can't be checked

#### Scenario: Removing from the library
- **WHEN** the user removes "Logo Ardent" from the library and confirms
- **THEN** the library no longer lists it, and projects that imported it keep their copy

#### Scenario: Import from the Resources tab
- **WHEN** the project has symbols, swatches and styles, and the user clicks Import from Library… in the Resources tab
- **THEN** the Import from Library dialog opens

#### Scenario: Disabled while editing a symbol
- **WHEN** a symbol is being edited
- **THEN** Import from Library… is disabled in the Object menu, the Brand header and the Resources tab, with a tooltip saying why
