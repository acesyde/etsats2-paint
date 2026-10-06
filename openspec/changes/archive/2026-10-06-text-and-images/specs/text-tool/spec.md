## Purpose

Lets users write and edit lettering directly on the canvas, like in Illustrator or Figma: click to type, double-click to edit, with a visible caret and selection.

## ADDED Requirements

### Requirement: Create text by clicking
With the Text tool (T), clicking on the canvas SHALL create a text object at the click point (the left of its first line's baseline), using the current character style and current fill, insert it in the active layer, select it and enter edit mode with an empty content and a blinking caret. The Text tool SHALL remain active.

#### Scenario: Click and type
- **WHEN** the Text tool is active, the user clicks on the canvas and types "ACE Logistics"
- **THEN** a text object reading "ACE Logistics" appears at the click point and is selected

### Requirement: Enter and leave edit mode
Double-clicking a text with the Selection or Move tool, clicking an existing text with the Text tool, or pressing Enter when exactly one text is selected SHALL enter edit mode for that text with the caret at the clicked position (or at the end). Pressing Escape, clicking outside the text, or switching tools SHALL leave edit mode and keep the text selected. Leaving edit mode with an empty content SHALL delete the text. Locked or hidden texts SHALL NOT enter edit mode.

#### Scenario: Escape leaves edit mode
- **WHEN** the user is editing a text and presses Escape
- **THEN** edit mode ends, the caret disappears and the text remains selected with its handles

#### Scenario: Empty text is removed
- **WHEN** the user clicks with the Text tool and presses Escape without typing
- **THEN** no text object remains in the document

### Requirement: On-canvas caret and selection
While editing, the canvas SHALL draw a blinking caret at the insertion point and highlight selected characters, following the text's rotation and the current zoom. The selection overlay handles SHALL be hidden during editing, and canvas shortcuts (tools, nudges, Delete of objects) SHALL be suspended so keys go to the text.

#### Scenario: Typing a tool letter while editing
- **WHEN** the user types "V" while editing a text
- **THEN** a "V" is inserted and the active tool does not change

### Requirement: Keyboard and pointer editing
Edit mode SHALL support typing (including accented characters), Enter for a new line, Backspace and Delete, Left/Right/Up/Down arrows, Home/End, Shift to extend the selection with arrows and Home/End, Cmd/Ctrl+A to select all text, Cut/Copy/Paste of plain text, dragging to select, and double-clicking to select a word.

#### Scenario: Select all and replace
- **WHEN** the user edits "Old name", presses Cmd/Ctrl+A and types "New name"
- **THEN** the text reads "New name"

#### Scenario: New line
- **WHEN** the user types "Line 1", presses Enter and types "Line 2"
- **THEN** the text has two lines

### Requirement: Editing session undo
An editing session SHALL be one undo step labelled "Edit Text" (or "Create Text" for a new text), recorded when edit mode ends. Pressing Cmd/Ctrl+Z while editing SHALL undo text changes within the session (and outside a session, the last document step).

#### Scenario: Undo an editing session
- **WHEN** the user edits a text from "ACE" to "ACE Logistics", leaves edit mode and presses Cmd/Ctrl+Z
- **THEN** the text reads "ACE" again
