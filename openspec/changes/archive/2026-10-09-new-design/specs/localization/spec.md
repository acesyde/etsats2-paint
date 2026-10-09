## ADDED Requirements

### Requirement: Layouts fit every language
No translated text SHALL be placed in a box whose width is fixed for the English text. In particular:
- buttons, tabs and segmented-control options SHALL size to their label in the current language;
- field labels SHALL be placed above their field, not beside it in a fixed-width column;
- tools in the tool rail and other icon-only controls SHALL show no text, their translated name and shortcut being given by their tooltip.

Every screen and dialog (home screen, New Project, Project, Workshop, Brand, Export Mod, Vehicle Library, Preferences and the other dialogs) SHALL show every label whole, without clipping, overlap or truncation, in German, and with every interface text 40% longer than its English text, at the default window size and UI scale of 100%.

#### Scenario: German button sized to its label
- **WHEN** the language is German and the top bar is shown
- **THEN** the Export… button shows its whole German label and is wider than the same button in English

#### Scenario: Field label above its field
- **WHEN** the Export Mod dialog is open in any language
- **THEN** each field's label is drawn above the field, and the field starts at the same left edge as its label

#### Scenario: Icon-only tools
- **WHEN** the language is German and the pointer rests on the Rectangle tool in the tool rail
- **THEN** the rail shows only the tool's icon and the tooltip reads the German tool name with its shortcut

#### Scenario: Texts 40% longer
- **WHEN** each screen and dialog is rendered with every interface text lengthened by 40%
- **THEN** no label is clipped, overlaps another control or is cut with an ellipsis

### Requirement: Every interface text in all four languages
Every text added or changed in the interface SHALL have a translation in English, French, Spanish and German. An automated test SHALL fail when a message key present in English is missing in French, Spanish or German.

#### Scenario: New text without a German translation
- **WHEN** a new message is added to the English texts only and the test suite runs
- **THEN** the localization test fails and names the missing key and language

#### Scenario: Space names translated
- **WHEN** the language is French
- **THEN** the space switcher, the left panel tabs and the View menu items for them are shown in French
