## MODIFIED Requirements

### Requirement: Export Mod dialog
File › Export Mod… (Cmd/Ctrl+E) SHALL be available when a project is open, except while a symbol is being edited. It SHALL also be reached from the **Export…** button of the top bar, shown in every space (see the workspace-spaces capability). They SHALL be enabled and disabled together, with the same reason as a tooltip.

The dialog SHALL edit nothing: the mod settings and pictures are edited in the Project space (see the project-screen capability). It SHALL be titled "Export Mod" with the project's name and game below the title, and SHALL hold, from top to bottom:
- **a summary of the mod:** each vehicle of the project, with the main textures and accessories it paints. For a truck with several main textures, each main texture lists its cabins, and the cabins of textures that aren't painted are listed as "not painted";
- **the destination** (see Destination), under its label, with **Change…**;
- **the problems that block the export,** if any (see Checks before export), listed just above the buttons, each with an icon and its text, and a **Show** action;
- **the warnings about the textures,** if any (see Checks before export), after the problems, each with its own icon and its text;
- **Cancel** and **Export**, the primary action.

Show on a problem SHALL close the dialog, show the Project space and take the player to where the problem is fixed, as the project-screen capability describes (Problems before exporting): the setting's field, or the vehicle's card.

The dialog SHALL fit German labels without truncation: buttons have free widths and labels sit above their values.

Export SHALL be disabled while a problem remains; warnings SHALL NOT disable it. Cancel (or Escape) SHALL close the dialog. Opening the dialog, closing it, following a problem and exporting SHALL NOT change the project, its undo history or its save state.

#### Scenario: Opening the dialog
- **WHEN** a project named "ACE Logistics" holds the sample truck painting Standard cab, Chassis and Cab accessories, and the user presses Cmd/Ctrl+E
- **THEN** the Export Mod dialog opens with "ACE Logistics" and the game below its title, its summary shows the TruckPaint Sample Truck with Standard cab (cabin "standard"), High roof not painted, and the accessories Chassis and Cab accessories, and it shows the destination "ACE Logistics.scs"

#### Scenario: Export… in the top bar
- **WHEN** the Brand space is shown and the user clicks Export… in the top bar
- **THEN** the Export Mod dialog opens

#### Scenario: From the Project space
- **WHEN** the Project space is shown and the user clicks Export… in the top bar
- **THEN** the Export Mod dialog opens; the Mod information column has no button of its own for it

#### Scenario: Nothing to edit in the dialog
- **WHEN** the Export Mod dialog is open
- **THEN** it shows no field for the Name, Version, Author, Description, Price, Unlock level or internal name, and no picture

#### Scenario: Advanced is collapsed
- **WHEN** the user opens Export Mod… on a project whose internal name has no problem
- **THEN** the dialog has no Advanced section: the internal name is shown only under the collapsed Advanced section of the Project space's Mod information column

#### Scenario: A problem leads to its field
- **WHEN** the project's Price is 0, the user opens Export Mod… and clicks Show on the problem saying the price must be more than 0
- **THEN** the dialog closes, the Project space is shown and its Price field has keyboard focus

#### Scenario: Advanced opens on an internal name problem
- **WHEN** a project's internal name was set to "ace_logistic" (12 characters) while it held only the sample trailer, the sample truck (whose package has two main textures) was then added, and the user opens Export Mod…
- **THEN** the problem saying the internal name can have at most 10 characters is listed above the buttons and Export is disabled; clicking its Show closes the dialog and gives the internal name field, under the opened Advanced section of the Project space, keyboard focus

#### Scenario: Cancel keeps the settings
- **WHEN** the user opens Export Mod… on a saved project and presses Escape
- **THEN** the dialog closes, the status bar still shows "Saved" and the undo history has no new step

#### Scenario: Warnings don't block
- **WHEN** a project with no problem has an Empty texture and a texture To check, and the user opens Export Mod…
- **THEN** the dialog lists both warnings, and Export is enabled
