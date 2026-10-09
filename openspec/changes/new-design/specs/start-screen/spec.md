## MODIFIED Requirements

### Requirement: Home screen on launch
When no project is open, the application SHALL show a home screen with:
- the application name and tagline;
- **New Project** as the primary button and **Open…** (Open Project) as a secondary button, each with its shortcut in its tooltip;
- the recovered projects, when there are any (see Recovered projects on the home screen);
- the **Recent projects** area (see Recent projects list);
- an entry to Preferences.

The buttons SHALL have free widths that fit their translated labels.

#### Scenario: First launch
- **WHEN** the application starts for the first time
- **THEN** the home screen is shown with New Project as the primary button, Open… as a secondary button, and a Recent projects area

#### Scenario: Returning to home
- **WHEN** the user closes the last open project
- **THEN** the home screen is shown again

#### Scenario: German labels
- **WHEN** the interface language is German
- **THEN** the New Project and Open… buttons show their whole German labels, without truncation

### Requirement: Recent projects list
The Recent projects area SHALL list previously opened projects, most recent first, as **cards**. Each card SHALL show a thumbnail of the project, its name, its file location and its last-opened date. The thumbnail is the artwork of the project's first main texture drawn over its template; while it is being read, and when the file can't be read, the card SHALL show a placeholder icon instead.

Clicking an available card SHALL open that project. When the list is empty, it SHALL show an explanatory empty state instead of a blank area. Entries whose file no longer exists SHALL be shown as unavailable, SHALL NOT open, and SHALL be removable from the list.

#### Scenario: Empty recent list
- **WHEN** no project has ever been opened
- **THEN** the Recent projects area shows an empty state message inviting the user to create a new project

#### Scenario: Missing recent file
- **WHEN** a recent project's file has been deleted or moved
- **THEN** its card is displayed as unavailable with a non-color indicator, shows the placeholder instead of a thumbnail, and offers a "Remove from list" action

#### Scenario: Open a recent project
- **WHEN** the user clicks an available recent card
- **THEN** the editor opens that project

#### Scenario: Thumbnail of a recent project
- **WHEN** a recent project's first main texture is covered by a red rectangle
- **THEN** its card's thumbnail is red

### Requirement: New Project dialog
Choosing New Project SHALL open a modal dialog on one page:

- **On top:** the **Project name** field and the **Game**, ETS2 or ATS, as a segmented control. The name defaults to the chosen vehicle's name when left empty. The game starts on the game of the installed vehicles, or ETS2 when vehicles of both games or none are installed. A note says that vehicles can be added to the project at any time.
- **On the left, the vehicle list:** the installed vehicles of the chosen game (name, brand, kind, newest version).
  - It is searchable by name and brand, and filterable by kind with a segmented control **All / Trucks / Trailers**.
  - Under the list, a **Custom vehicle…** entry explains that a vehicle without a package can be created from its template files (DDS, PNG or SVG) and its game data, and opens the Custom Vehicle dialog (see the custom-vehicles capability). An **Install…** button installs packages (see the vehicle-packages capability).
  - With no vehicle installed, the list offers Install…, Install the sample vehicles and Custom vehicle….
  - Clicking a vehicle chooses it; choosing another one replaces it.
- **On the right, "Your fleet":** the chosen vehicle, with:
  - under **Cabins**, one checkbox per main texture, named after it and followed by the cabins it covers. The first one is checked when the vehicle is chosen; the last checked main texture (and so a single main texture) can't be unchecked;
  - the **main texture mode**, read only, from the package: "One per cabin layout" when the package has several main textures, "One for every cabin" for a truck with a single main texture, "Single main texture" for a trailer. The user can't change it;
  - under **Accessories**, one checkbox per accessory, all checked when the vehicle is chosen;
  - the count of the textures that will be created ("2 main + 3 accessories");
  - **Textures created:** the list of the textures that will be created, each with its size, and their total.
  With no vehicle chosen, it SHALL say to pick a vehicle in the list.
- **Create Project**, the primary action, SHALL be enabled only when a vehicle is chosen and at least one main texture is checked; its tooltip says why when it is disabled. **Cancel** sits beside it.

A vehicle created with Custom vehicle… SHALL be chosen when the dialog is shown again, with its textures checked as for any chosen vehicle. The game, search and kind filter SHALL be changed if needed so it is listed. Cancelling the Custom Vehicle dialog SHALL show the New Project dialog as it was.

Confirming SHALL open the editor workspace, in the Project space, with a new project for that vehicle and its chosen textures (see the vehicle-projects capability). Cancelling SHALL return to the home screen without side effects. The dialog SHALL be fully operable by keyboard (Tab to move, Space to toggle a checkbox or choose a vehicle, Enter to create when Create Project is enabled, Escape to cancel).

#### Scenario: Creating a project
- **WHEN** the user opens New Project, picks the sample truck, keeps "Standard cab" and the accessories checked, and clicks Create Project
- **THEN** the editor workspace opens in the Project space with a project named "TruckPaint Sample Truck" on its Standard cab and accessory textures, and the window title shows the project name

#### Scenario: Cancelling
- **WHEN** the user presses Escape in the New Project dialog
- **THEN** the dialog closes and the home screen is unchanged

#### Scenario: Empty name
- **WHEN** the user leaves the name empty and confirms
- **THEN** the project is created with the vehicle's name

#### Scenario: Creating a project for a vehicle
- **WHEN** the user opens New Project, picks "Volvo FH16 2012", checks its "Globetrotter XL" main texture, unchecks every accessory, and clicks Create Project
- **THEN** the workspace opens with a project named "Volvo FH16 2012" with one surface, the Globetrotter XL main texture

#### Scenario: Creating a trailer project
- **WHEN** the user picks the sample trailer
- **THEN** Your fleet shows Base checked and impossible to uncheck, the main texture mode "Single main texture", the accessories checked, and Create Project is enabled

#### Scenario: The last main texture stays checked
- **WHEN** the user picks the sample truck, checks "High roof" and unchecks "Standard cab"
- **THEN** "High roof" can't be unchecked, and Create Project stays enabled

#### Scenario: Main texture mode is read only
- **WHEN** the user picks the sample truck, whose package has two main textures
- **THEN** Your fleet shows the main texture mode "One per cabin layout" as text, with no control to change it

#### Scenario: Textures that will be created
- **WHEN** the user picks the sample truck 1.1.0 and keeps Standard cab and every accessory checked
- **THEN** Your fleet shows "1 main + 3 accessories" and lists Standard cab (4096), Chassis (4096), Cab accessories (1024) and Side skirts (1024), 4 textures in total

#### Scenario: Only the chosen game is listed
- **WHEN** ETS2 and ATS vehicles are installed and the user sets the Game to ATS
- **THEN** the vehicle list shows only the ATS vehicles

#### Scenario: No vehicle installed
- **WHEN** the user opens New Project with no vehicle installed
- **THEN** Create Project is disabled, and Install…, Install the sample vehicles and Custom vehicle… are offered

#### Scenario: Project from a custom vehicle
- **WHEN** the user, with no vehicle installed, clicks Custom vehicle…, creates a truck from one template file, then clicks Create Project
- **THEN** the workspace opens with a project for that custom vehicle, with one surface showing the template

#### Scenario: Created vehicle hidden by a filter
- **WHEN** the New Project dialog's Game is ATS and the user creates an ETS2 custom vehicle
- **THEN** the dialog shows the Game ETS2 and the new vehicle chosen, with its textures checked

## REMOVED Requirements

### Requirement: Dialog structure ready for vehicle steps
**Reason**: New Project becomes a single page (design Decision 11): the name and game on top, the vehicle list on the left and "Your fleet" with the textures to create on the right, so there are no steps, no step indicator and no Next / Back buttons.
**Migration**: Choose the vehicle and its textures and type the name on the same page, then click Create Project (or press Enter). The checked textures stay as they are while the name is edited, so there is nothing to go back to.
