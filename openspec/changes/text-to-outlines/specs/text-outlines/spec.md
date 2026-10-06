## Purpose

Lets users turn lettering into editable vector paths, to reshape letters and to stop depending on the fonts installed on the computer.

## ADDED Requirements

### Requirement: Create Outlines command
The Object menu SHALL provide "Create Outlines" (⌘⇧O on macOS, Ctrl+Shift+O elsewhere). It SHALL be enabled when the selection contains at least one text, directly or inside selected groups, and no canvas gesture is in progress. Running it while a text is being edited SHALL first end the editing session, keeping the typed text. The conversion of every selected text SHALL be one undo step.

#### Scenario: Undo restores the text
- **WHEN** the user outlines a text and presses Cmd/Ctrl+Z
- **THEN** the text is back, editable, with its content and style

#### Scenario: Disabled without texts
- **WHEN** only rectangles are selected
- **THEN** Create Outlines is disabled

### Requirement: Group of letter paths
Each outlined text SHALL be replaced, at the same place in the stacking order and in the same parent group, by a group:

- **The group:** keeps the text's identity, visibility, lock flag and opacity, and is named after the text's content (its first line, shortened to 32 characters).
- **The letters:** the group holds one path per visible letter, in reading order. Each letter is named after its character and contains all of its contours, so holes such as the counters of "A", "O" or "R" stay holes.
- **Empty texts:** characters without a visible shape (spaces) produce no path. A text with no visible letters SHALL be left unchanged.
- **Inside groups:** texts inside selected groups SHALL be outlined in place, and the other objects of those groups SHALL be left unchanged.

#### Scenario: Outline a word
- **WHEN** the text "ACE" is selected and the user chooses Create Outlines
- **THEN** the text is replaced by a group named "ACE" containing three paths named "A", "C" and "E", and the group is selected

#### Scenario: Counters stay holes
- **WHEN** the text "O" is outlined
- **THEN** its path has two closed subpaths and the area inside the inner one is not filled

#### Scenario: Only spaces
- **WHEN** a text containing only spaces is selected and the user chooses Create Outlines
- **THEN** the text is unchanged

### Requirement: Same appearance
Outlined letters SHALL look exactly like the text did:

- same glyph shapes (including the fallback font when the text's font was missing), positions, size, letter spacing, line height, alignment, rotation and scale;
- each letter SHALL have the text's fill and stroke.

The letters SHALL be ordinary paths: editable with the Direct Selection tool, saved with the project and exported like any path.

#### Scenario: Same pixels on export
- **WHEN** a rotated, stretched, outlined text with a stroke is exported
- **THEN** the image matches the export of the text before outlining, within anti-aliasing tolerance

#### Scenario: Reshape a letter
- **WHEN** the user selects the outlined letter "S" with the Direct Selection tool
- **THEN** its anchor points are shown and can be moved
