## MODIFIED Requirements

### Requirement: Supported languages
The interface SHALL be available in English (international), French, Spanish and German, each identified in the interface by its own name: "English", "Français", "Español", "Deutsch". English is the source language. When a text has no translation in the current language, the English text SHALL be shown, so the interface never shows a message key or an empty label.

#### Scenario: Interface in German
- **WHEN** the language is German
- **THEN** the menus are "Datei", "Bearbeiten", "Objekt", "Ebene", "Ansicht", "Fahrzeug" and "Hilfe"

#### Scenario: Missing translation falls back to English
- **WHEN** a text has no French translation and the language is French
- **THEN** that text is shown in English and every other text in French
