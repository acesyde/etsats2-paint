# localization Specification

## Purpose

Makes the interface available in English, French, Spanish and German: how the language is chosen and switched, what is translated, and how numbers, counts, shortcuts and new object names follow the language.

## Requirements

### Requirement: Supported languages
The interface SHALL be available in English (international), French, Spanish and German, each identified in the interface by its own name: "English", "Français", "Español", "Deutsch". English is the source language. When a text has no translation in the current language, the English text SHALL be shown, so the interface never shows a message key or an empty label.

#### Scenario: Interface in German
- **WHEN** the language is German
- **THEN** the menus are "Datei", "Bearbeiten", "Objekt", "Ebene", "Ansicht", "Fahrzeug" and "Hilfe"

#### Scenario: Missing translation falls back to English
- **WHEN** a text has no French translation and the language is French
- **THEN** that text is shown in English and every other text in French

### Requirement: Initial language from the system
When the user has never chosen a language, the interface language SHALL follow the operating system's preferred language:
- any French variant (fr-FR, fr-CA, fr-BE, …) gives French;
- any Spanish variant gives Spanish;
- any German variant gives German;
- any other language, or one that cannot be read, gives English.

The detected language SHALL NOT be saved as the user's choice, so a later change of the system language is followed.

#### Scenario: Canadian French system
- **WHEN** the application starts for the first time on a system set to fr-CA
- **THEN** the interface is in French

#### Scenario: Unsupported system language
- **WHEN** the application starts for the first time on a system set to Italian
- **THEN** the interface is in English

### Requirement: Switching language
Changing the language in Preferences SHALL apply immediately to the whole interface, without a restart: menus, panels, dialogs (including the open Preferences dialog), tooltips, the status bar, canvas hints, accessibility names and undo/redo labels. Switching language SHALL NOT change the document, the selection, the undo history or unsaved changes.

#### Scenario: Live switch
- **WHEN** a project is open with unsaved changes and the user picks "Español" in Preferences
- **THEN** the Preferences dialog, the menu bar and the panels are shown in Spanish at once and the project still has its unsaved changes

#### Scenario: Undo label follows the language
- **WHEN** the user changes a fill color in English, then switches to French and opens the Edit menu
- **THEN** the undo item reads "Annuler Modifier le fond"

### Requirement: What is translated
Every text the interface shows SHALL be translated, including:
- menu items, command labels, tooltips and the explanations of disabled commands;
- panel titles, field labels, buttons and empty states;
- dialogs, the home screen, the status bar, canvas hints and error messages;
- undo/redo labels, accessibility names and file dialog filter names.

User content SHALL NOT be translated: project, object, layer and asset names, text objects, font family names and file paths. File contents SHALL stay in English whatever the language: project files, exported textures and mods, preferences and logs.

#### Scenario: User names stay as typed
- **WHEN** a layer named "Stripes" is shown with the language set to German
- **THEN** its row reads "Stripes" while the Layers panel title reads "Ebenen"

### Requirement: Localized numbers
Numbers shown in fields and labels SHALL use the language's decimal separator: a point in English, and a comma in French, Spanish and German. Numeric fields SHALL accept both a point and a comma as the decimal separator in every language. Units and symbols (px, %, °) SHALL be unchanged. Hexadecimal colors SHALL be unchanged.

#### Scenario: Decimal comma
- **WHEN** the language is French and a stroke is 12.5 px wide
- **THEN** the Width field shows "12,5"

#### Scenario: Either separator accepted
- **WHEN** the language is German and the user types "12.5" in the Width field and presses Enter
- **THEN** the width becomes 12.5 px and the field shows "12,5"

### Requirement: Counts and relative times
Texts containing a count SHALL use the plural forms of the current language. This covers the relative times of recent projects ("2 days ago") and counts of objects.

#### Scenario: Relative time in Spanish
- **WHEN** the language is Spanish and a recent project was opened 2 days ago
- **THEN** its entry reads "hace 2 días"

#### Scenario: Singular in French
- **WHEN** the language is French and a recent project was opened 1 hour ago
- **THEN** its entry reads "il y a 1 heure"

### Requirement: Localized shortcut notation
On Windows and Linux, shortcuts SHALL be shown with modifier names of the current language:

| Language | Ctrl | Shift | Alt |
|---|---|---|---|
| English | Ctrl | Shift | Alt |
| French | Ctrl | Maj | Alt |
| Spanish | Ctrl | Mayús | Alt |
| German | Strg | Umschalt | Alt |

On macOS, the symbols (⌃⌥⇧⌘) SHALL be unchanged. The keys that trigger a shortcut SHALL NOT change with the language.

#### Scenario: German shortcut on Windows
- **WHEN** the language is German on Windows
- **THEN** the Redo menu item shows "Strg+Umschalt+Z"

### Requirement: Names of new objects
Default names given at creation SHALL use the current language. That covers new shapes ("Rectangle" → "Rechteck"), layers ("Layer 1" → "Calque 1"), groups, texts and new projects ("Untitled"). Names already in a document SHALL NOT change when the language changes.

#### Scenario: New layer in French
- **WHEN** the language is French and the user creates a layer in a new project
- **THEN** the layer is named "Calque 1"

#### Scenario: Existing names kept
- **WHEN** a rectangle was created in English (named "Rectangle") and the language is switched to German
- **THEN** it is still named "Rectangle"

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
