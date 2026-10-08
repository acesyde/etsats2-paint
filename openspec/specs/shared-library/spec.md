# shared-library Specification

## Purpose

Lets a player reuse their company's identity (symbols, swatches and styles) across all their projects, in both games, through one personal library that elements are added to and imported from with everything they depend on.

## Requirements

### Requirement: Personal library
The application SHALL keep one personal library per user, in its data folder, shared by every project of both games. It SHALL hold symbols, swatches, graphic styles and text styles, each with a name, and the images its symbols use. Fonts SHALL be referenced by family name only, as in projects.

The library SHALL be written when it changes, completely before it replaces the previous file. When it is missing, the library is empty. When it can't be read (damaged, or written by a newer TruckPaint), the application SHALL keep a backup copy of the file, start with an empty library, and say so once when the library is first used in the session.

#### Scenario: One library for every project
- **WHEN** the user adds the symbol "Logo Ardent" to the library from an ETS2 project, then opens an ATS project and chooses Import from Library…
- **THEN** the dialog lists "Logo Ardent"

#### Scenario: Library survives a restart
- **WHEN** the user adds the swatch "Vert Ardent" to the library, quits and restarts the application
- **THEN** Import from Library… still lists "Vert Ardent"

#### Scenario: Unreadable library
- **WHEN** the library file is damaged and the user opens Import from Library…
- **THEN** a message says the library could not be read and a backup was kept, and the dialog shows an empty library

### Requirement: Add to Library
The context menu of a symbol in the Symbols panel, of a swatch in the Colors panel and of a style in the Styles panel SHALL offer **Add to Library**. It SHALL copy the element into the library with everything it uses:
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

### Requirement: Import from Library
Object › **Import from Library…** SHALL be available when a project is open, except while a symbol is being edited. The Symbols, Colors and Styles panels SHALL offer it when they are empty. It SHALL open a dialog listing the library's symbols, swatches, graphic styles and text styles in four groups, each with its name and a preview (the color for a swatch, the symbol's content for a symbol). Each element has a checkbox:
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

### Requirement: Import rules
Importing elements into a project, from the library or by pasting from another project, SHALL bring each element with everything it uses (as for Add to Library), under new ids, with every link (images, swatches, styles, symbols) pointing to the project's elements. Each element, dependencies included, SHALL be:
- **reused** when the project already has an element linked to the same library entry, or a swatch or style with the same name and the same value. The project's element is kept unchanged, and the imported links point to it;
- **added** otherwise, linked to the same library entry as its source when it has one. When a different element of the project already has its name, it gets the first free numbered name ("Logo Ardent 2").

An image already in the project (same bytes) SHALL be reused. Text fonts SHALL be kept by family name; a family that isn't installed shows the usual "Font not found" warning.

Imported elements are copies: later edits of the library don't change the project, and edits in the project don't change the library until Update in Library.

#### Scenario: No duplicate swatch
- **WHEN** the project already has the swatch "Vert Ardent" with the same color, and the user imports a symbol from the library whose text uses "Vert Ardent"
- **THEN** the project still has one "Vert Ardent", and the imported text is linked to it

#### Scenario: Name taken by a different element
- **WHEN** the project has its own symbol "Logo", not from the library, and the user imports the library's "Logo"
- **THEN** the imported symbol is named "Logo 2"

#### Scenario: The library is not linked
- **WHEN** a project imported "Vert Ardent", and the user later updates "Vert Ardent" in the library from another project with another color
- **THEN** the first project's "Vert Ardent" keeps its color
