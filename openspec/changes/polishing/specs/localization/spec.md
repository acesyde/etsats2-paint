## MODIFIED Requirements

### Requirement: Layouts fit every language
No translated text SHALL be placed in a box whose width is fixed for the English text. In particular:
- buttons, tabs and segmented-control options SHALL size to their label in the current language;
- field labels SHALL be placed above their field, not beside it in a fixed-width column;
- tools in the tool rail and other icon-only controls SHALL show no text, their translated name and shortcut being given by their tooltip.

Every screen and dialog (home screen, New Project, Project, Workshop, Brand, Export Mod, Vehicle Library, Preferences and the other dialogs) SHALL show every label whole, without clipping, overlap or truncation, in German, and with every interface text 40% longer than its English text, at the default window size and at the minimum window size (960 × 600), at a UI scale of 100%. A text that can't be given its whole width, such as a user's name in a list, SHALL end with "…" and show in full in a tooltip.

Game version ranges SHALL be shown in words in the current language (">=1.56" reads "1.56 or later"), never with their operators; the operators stay in the fields where a range is typed.

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

#### Scenario: German at the minimum width
- **WHEN** the language is German, the window is 960 px wide and the Project space shows a vehicle with an update
- **THEN** the All / To do / To check filter doesn't overlap the subtitle, and the vehicle card's header shows each block whole or ending with "…"

#### Scenario: Version range in words
- **WHEN** every vehicle of a project supports ">=1.56" and the language is English
- **THEN** the Project space reads "Supported by every vehicle: 1.56 or later"
