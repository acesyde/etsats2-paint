## ADDED Requirements

### Requirement: Paste into another project
Objects copied or cut in one project and pasted into another SHALL keep their look: their images, the swatches and styles they are linked to, and the symbols of their instances SHALL be imported into the target project with the library's import rules (see shared-library), as part of the Paste undo step. Pasting into the project the objects were copied from SHALL behave as before. The clipboard SHALL keep the objects' dependencies as they were when copied, so they can be pasted after the source project is closed.

#### Scenario: Pasting a logo into another project
- **WHEN** the user copies an image and a text linked to the swatch "Vert Ardent" in project A, opens project B and pastes
- **THEN** project B shows the same image and text, has a swatch "Vert Ardent" with the same color, and the text is linked to it

#### Scenario: Pasting an instance into another project
- **WHEN** the user copies an instance of the symbol "Logo Ardent" in project A, closes A, opens project B and pastes
- **THEN** project B has the symbol "Logo Ardent" and the pasted object is an instance of it

#### Scenario: Undo a paste into another project
- **WHEN** the user presses Undo right after such a paste
- **THEN** the pasted objects and the elements imported with them are removed
