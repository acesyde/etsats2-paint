## MODIFIED Requirements

### Requirement: New Project dialog
Choosing New Project SHALL open a modal dialog on one page:

- **On top:** the **Project name** field and the **Game**, ETS2 or ATS, as a segmented control. The name defaults to the first checked vehicle's name when left empty. The game starts on the game of the installed vehicles, or ETS2 when vehicles of both games or none are installed. Changing the game unchecks the vehicles of the other game. A note says that vehicles can be added to the project at any time.
- **On the left, the vehicle list**, each row with a checkbox, under the column headers Vehicle, Type and Package:
  - first, under **Official**, the official vehicles of the chosen game from the catalog (see the official-vehicles capability), installed or not, one row each with the name, the kind and the brand, and "Official" in the Package column, followed by the newest installed version when it is installed;
  - then, under **Installed**, the other installed vehicles of the chosen game, one row each (name, kind, brand and newest version). An installed official vehicle is listed only under Official.
  - The section headers are shown only when both sections have rows.
  - The list is searchable by name and brand, and filterable by kind with a segmented control **All / Trucks / Trailers**. Both sections follow the search and the filter. Checked vehicles stay checked when the search or the filter hides them.
  - Under the list, a **Custom vehicle…** entry explains that a vehicle without a package can be created from its template files (DDS, PNG or SVG) and its game data, and opens the Custom Vehicle dialog (see the custom-vehicles capability). An **Install…** button installs packages (see the vehicle-packages capability).
  - With no vehicle installed and no official vehicle for the chosen game, the list offers Install…, Install the sample vehicles and Custom vehicle…. With no vehicle installed but official vehicles listed, Install the sample vehicles is offered under the list.
  - Clicking a row, or pressing Space on it, checks or unchecks the vehicle. Several vehicles, trucks and trailers, can be checked together. Checking an official vehicle installs it first when needed, asking for the template pack when its folder isn't known (see the official-vehicles capability), then checks it; until it is installed, the checked vehicles are unchanged.
- **On the right, "Your fleet",** with the number of checked vehicles ("3 vehicles"): one card per checked vehicle, in the order they were checked, each with the vehicle's name and **Remove**, which unchecks it. One card is open at a time: the vehicle checked last, or the card the user clicks. The open card shows:
  - under **Cabins**, one chip per main texture, named after it, that toggles it (filled when it is checked, outlined otherwise; the cabins it covers in its tooltip). The first one is checked when the vehicle is checked; the last checked main texture (and so a single main texture) can't be unchecked;
  - the **main texture mode**, read only, from the package: "One per cabin layout" when the package has several main textures, "One for every cabin" for a truck with a single main texture, "Single main texture" for a trailer. The user can't change it;
  - under **Accessories**, one checkbox per accessory, all checked when the vehicle is checked;
  - the count of the textures that will be created for it ("2 main + 3 accessories").

  A folded card shows, on one line, the names of its checked main textures and its count. Under the cards, **Textures created** lists the textures that will be created for every vehicle, each with its size, and their total. With no vehicle checked, Your fleet SHALL say to check vehicles in the list.
- **Create Project**, the primary action, SHALL be enabled only when at least one vehicle is checked (each keeps at least one main texture); its tooltip says why when it is disabled. **Cancel** sits beside it.

A vehicle created with Custom vehicle… SHALL be checked when the dialog is shown again, with its textures checked as for any checked vehicle; the vehicles checked before stay checked. The game, search and kind filter SHALL be changed if needed so it is listed; when the game changes, the vehicles of the other game are unchecked. Cancelling the Custom Vehicle dialog SHALL show the New Project dialog as it was.

Confirming SHALL open the editor workspace, in the Project space, with a new project for the checked vehicles, in fleet order, and their chosen textures (see the vehicle-projects capability); the first texture of the first vehicle is active. Cancelling SHALL return to the home screen without side effects. The dialog SHALL be fully operable by keyboard (Tab to move, Space to toggle a checkbox or a chip or check a vehicle, Enter to create when Create Project is enabled, Escape to cancel).

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
- **WHEN** the user opens New Project with no vehicle installed and the Game set to ATS, which has no official vehicle yet
- **THEN** Create Project is disabled, and Install…, Install the sample vehicles and Custom vehicle… are offered

#### Scenario: Project from a custom vehicle
- **WHEN** the user, with no vehicle installed, clicks Custom vehicle…, creates a truck from one template file, then clicks Create Project
- **THEN** the workspace opens with a project for that custom vehicle, with one surface showing the template

#### Scenario: Created vehicle hidden by a filter
- **WHEN** the New Project dialog's Game is ATS and the user creates an ETS2 custom vehicle
- **THEN** the dialog shows the Game ETS2 and the new vehicle chosen, with its textures checked

#### Scenario: Official vehicles first
- **WHEN** the sample truck is installed and the user opens New Project with the Game ETS2
- **THEN** the list shows, under Official, the DAF NGD (Truck, DAF, Official) and the Krone Cool Liner (Trailer, Krone, Official), then, under Installed, the TruckPaint Sample Truck

#### Scenario: Official vehicle filtered by kind
- **WHEN** the user sets the kind filter to Trailers
- **THEN** the Official section shows only the Krone Cool Liner

#### Scenario: Project from an official vehicle
- **WHEN** the ETS2 pack folder is known, the user picks the DAF NGD, waits for it to install, checks XF and clicks Create Project
- **THEN** the workspace opens with a project named "DAF NGD" on its XG+ and XF main textures and its four accessories, each showing its template


#### Scenario: A truck and a trailer at once
- **WHEN** the user checks the sample truck, then the sample trailer, and clicks Create Project with an empty name
- **THEN** the project, named "TruckPaint Sample Truck", holds both vehicles, the truck's textures then the trailer's, with the truck's Standard cab active

#### Scenario: One card open at a time
- **WHEN** the user checks the sample truck, then the sample trailer
- **THEN** Your fleet shows "2 vehicles", the trailer's card open with Base and its accessories, and the truck's card folded to "Standard cab · 1 main + 3 accessories"; clicking the truck's card opens it and folds the trailer's

#### Scenario: Remove from the fleet
- **WHEN** two vehicles are checked and the user clicks Remove on the first card
- **THEN** that vehicle is unchecked in the list, Your fleet shows "1 vehicle", and Textures created lists only the other vehicle's textures

#### Scenario: Unchecking in the list
- **WHEN** the sample truck is the only checked vehicle and the user clicks its row again
- **THEN** it is unchecked, Your fleet says to check vehicles in the list, and Create Project is disabled

#### Scenario: Changing the game
- **WHEN** an ETS2 truck and an ETS2 trailer are checked and the user sets the Game to ATS
- **THEN** no vehicle is checked

#### Scenario: Custom vehicle added to the fleet
- **WHEN** the sample truck is checked and the user creates a custom trailer with Custom vehicle…
- **THEN** the sample truck and the custom trailer are both checked, with the trailer's card open
