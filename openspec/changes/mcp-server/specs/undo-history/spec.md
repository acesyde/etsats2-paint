## ADDED Requirements

### Requirement: Changes made through the MCP server
Each write call applied through the MCP server (see the mcp-server capability) SHALL be exactly one undo step, never merged with another call or with a change made by the user, whatever the time between them. Its label SHALL start with "AI:" followed by the action ("AI: Add Path", "AI: Import Image", "AI: Delete"…), in the interface language. A call that fails SHALL NOT create a step. Undo and redo of these steps SHALL behave as for any other change, including restoring the selection.

#### Scenario: Undoing the assistant's change
- **WHEN** a client adds a path and the user opens the Edit menu
- **THEN** it offers "Undo AI: Add Path", and choosing it removes the path

#### Scenario: Two quick calls
- **WHEN** a client adds two shapes within half a second
- **THEN** the history has two steps, and one Undo removes only the second shape

#### Scenario: Interleaved with the user
- **WHEN** the user moves a rectangle, then a client adds an ellipse, and the user presses Cmd/Ctrl+Z twice
- **THEN** the first Undo removes the ellipse and the second returns the rectangle to its previous position
