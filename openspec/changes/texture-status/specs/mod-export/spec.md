## MODIFIED Requirements

### Requirement: Export Mod dialog
Export › Export Mod… (Cmd/Ctrl+E) SHALL be available when a project is open, except while a symbol is being edited. It SHALL also be reached from:
- the **Export…** button of the top bar, shown in every space (see the workspace-spaces capability);
- **Edit in Export Mod…** in the Mod information column of the Project space (see the project-screen capability), the place where the mod settings are shown when the dialog is closed.

They SHALL be enabled and disabled together, with the same reason as a tooltip.

The dialog SHALL be titled "Export Mod" with the project's name and game below the title, and SHALL hold, from top to bottom:
- **the mod settings** (see Mod settings), each with its label above its field: Name, Version, Author, Description, Price and Unlock level;
- **Advanced**, a collapsed section holding the internal name and its help. It SHALL open by itself while a problem concerns the internal name;
- **the images:** the shop icon and the Mod Manager image (see Mod images);
- **a summary of the mod:** each vehicle of the project, with the main textures and accessories it paints. For a truck with several main textures, each main texture lists its cabins, and the cabins of textures that aren't painted are listed as "not painted";
- **the problems that block the export,** if any (see Checks before export), listed just above the buttons, each with an icon and its text;
- **the warnings about the textures,** if any (see Checks before export), after the problems, each with its own icon and its text;
- **Cancel** and **Export…**, the primary action.

The dialog SHALL fit German labels without truncation: buttons have free widths and labels sit above their fields.

Export… SHALL be disabled while a problem remains; warnings SHALL NOT disable it. Cancel (or Escape) SHALL close the dialog and leave the project unchanged, mod settings included.

#### Scenario: Opening the dialog
- **WHEN** a project named "ACE Logistics" holds the sample truck painting Standard cab, Chassis and Cab accessories, and the user presses Cmd/Ctrl+E
- **THEN** the Export Mod dialog opens with the Name "ACE Logistics", and its summary shows the TruckPaint Sample Truck with Standard cab (cabin "standard"), High roof not painted, and the accessories Chassis and Cab accessories

#### Scenario: Export… in the top bar
- **WHEN** the Brand space is shown and the user clicks Export… in the top bar
- **THEN** the Export Mod dialog opens

#### Scenario: From the Project space
- **WHEN** the user clicks Edit in Export Mod… in the Project space's Mod information column
- **THEN** the Export Mod dialog opens with the project's mod settings

#### Scenario: Advanced is collapsed
- **WHEN** the user opens Export Mod… on a project whose internal name has no problem
- **THEN** the Advanced section is collapsed and the internal name field is hidden

#### Scenario: Advanced opens on an internal name problem
- **WHEN** a project's internal name was set to "ace_logistic" (12 characters) while it held only the sample trailer, the sample truck (whose package has two main textures) was then added, and the user opens Export Mod…
- **THEN** the Advanced section is open, the problem saying the internal name can have at most 10 characters is listed above the buttons, and Export… is disabled

#### Scenario: Cancel keeps the settings
- **WHEN** the user changes the Price to 9000 in the dialog and presses Escape
- **THEN** the dialog closes, the project's price is unchanged and the undo history has no new step

#### Scenario: Warnings don't block
- **WHEN** a project with no problem has an Empty texture and a texture To check, and the user opens Export Mod…
- **THEN** the dialog lists both warnings, and Export… is enabled

### Requirement: Checks before export
The dialog SHALL list every problem that blocks the export, each naming what to fix, and update the list as the settings change:
- the Name is empty, or contains `"`, `\` or a line break;
- the Version or the Author contains `"`, `\` or a line break;
- the internal name is empty, contains a character other than `a`–`z`, `0`–`9` and `_`, or is too long: at most 12 characters, or at most 10 when a vehicle of the project has several main textures in its package;
- the Price is 0;
- two vehicles of the project have the same game path;
- a vehicle's game data is missing (see project-files), naming the vehicle and the package version to install;
- one of the project's Game versions isn't written like the game's versions: two to four numbers separated by dots, the last one possibly replaced by `*` (`1.56`, `1.56.*`, `1.56.2`, `1.56.2.*`), naming it;
- one of the project's Game versions isn't supported by a vehicle's package, naming the version, the vehicle and its range;
- the vehicles have no game version in common, naming two vehicles whose ranges don't overlap and their ranges, and asking to update or remove one of them.

The dialog SHALL also list the project's warnings about its textures, the textures To check and the Empty textures, as the texture-status capability describes (Before exporting warnings). They don't block the export: the Empty textures are exported transparent, and the textures not in their vehicle's version are left out of the mod, as before.

#### Scenario: Empty name
- **WHEN** the user clears the Name
- **THEN** the dialog says the mod needs a name, and Export… is disabled

#### Scenario: Internal name too long for a truck with several cabins
- **WHEN** the project holds the sample truck, whose package has two main textures, and the internal name is "ace_logistic" (12 characters)
- **THEN** the dialog says the internal name can have at most 10 characters, and Export… is disabled

#### Scenario: Missing game data
- **WHEN** a project made by a build without game data is opened on a computer that doesn't have its vehicle's package version 1.1.0, and the user opens Export Mod…
- **THEN** the dialog says that TruckPaint Sample Truck 1.1.0 must be installed to export the mod, and Export… is disabled

#### Scenario: Badly written game version
- **WHEN** the project's Game versions are `1.56.x` and the user opens Export Mod…
- **THEN** the dialog says that 1.56.x isn't a game version and gives 1.56.* as an example, and Export… is disabled

#### Scenario: Game version a vehicle doesn't support
- **WHEN** the project holds the sample truck (`>=1.56`), its Game versions are `1.55.*, 1.56.*`, and the user opens Export Mod…
- **THEN** the dialog says that TruckPaint Sample Truck (>=1.56) doesn't support 1.55.*, and Export… is disabled

#### Scenario: No common game version
- **WHEN** the project holds the sample truck (`>=1.56`) and a custom vehicle "Old Hauler" supporting `<1.55`, and the user opens Export Mod…
- **THEN** the dialog says that TruckPaint Sample Truck (>=1.56) and Old Hauler (<1.55) have no game version in common and that one of them must be updated or removed, and Export… is disabled

#### Scenario: Empty textures warned
- **WHEN** a project's High roof texture is Empty and every other texture is Modified, and the user opens Export Mod…
- **THEN** the dialog warns that High roof is empty and exported with the game's color, and the exported mod still holds a transparent High roof texture

#### Scenario: Texture to check warned
- **WHEN** a project's Curtain body 13.6 m texture is To check because its layout changed, and the user opens Export Mod…
- **THEN** the dialog warns "Curtain body 13.6 m: layout changed", and Export… stays enabled when there is no problem
