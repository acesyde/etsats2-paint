## Purpose

Protects work against crashes and power loss by keeping recovery copies of unsaved projects that can be restored at the next launch, without ever overwriting the user's files.

## ADDED Requirements

### Requirement: Recovery copies
While the open project has unsaved changes, the application SHALL write a recovery copy at most every 2 minutes, in the application's per-user data folder, never to the project's own file. Writing a recovery copy SHALL NOT freeze the editor or change the save state. When the project is saved, closed (whatever the choice in the save prompt) or the application quits normally, its recovery copy SHALL be deleted.

#### Scenario: Recovery copy while editing
- **WHEN** the user edits a project for more than 2 minutes without saving
- **THEN** a recovery copy exists in the application data folder and the project file is unchanged

#### Scenario: Normal close removes the copy
- **WHEN** the user closes the project and chooses Don't Save
- **THEN** no recovery copy of it remains

### Requirement: Restore after a crash
At launch, when recovery copies remain from a previous session, the home screen SHALL list each recovered project with its name, original file (or "Never saved") and the time of the copy, offering Restore and Discard. Restore SHALL open the recovered project marked as having unsaved changes and associated with its original file, if any; Discard SHALL delete the recovery copy. Recovery copies that cannot be read SHALL be listed as damaged with only Discard.

#### Scenario: Restore unsaved work
- **WHEN** the application crashed while "ACE Logistics" had unsaved changes and is started again
- **THEN** the home screen offers to restore "ACE Logistics", and Restore opens it with those changes and "Unsaved changes" in the status bar

#### Scenario: Discard a recovered project
- **WHEN** the user chooses Discard on a recovered project
- **THEN** it disappears from the list and is not offered again
