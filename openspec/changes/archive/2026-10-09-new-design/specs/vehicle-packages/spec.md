## MODIFIED Requirements

### Requirement: Installing packages
The user SHALL be able to install a package:
- from Vehicle › Vehicle Library… › Install…, choosing one or more `.tpv` files;
- from the vehicle list of New Project › Install…;
- by dropping `.tpv` files on the home screen or the workspace.

Installed packages SHALL persist across sessions in the application's data folder.

Installing the same version again SHALL replace it. A different version of an installed vehicle SHALL be installed alongside the others. A confirmation SHALL name each vehicle and version installed.

#### Scenario: Install from the Vehicle Library
- **WHEN** the user installs `volvo_fh16-1.2.0.tpv` from the Vehicle Library and restarts the application
- **THEN** "Volvo FH16 2012" version 1.2.0 is listed in the library

#### Scenario: Two versions side by side
- **WHEN** versions 1.2.0 and 1.3.0 of the same vehicle are installed
- **THEN** the library lists the vehicle once, as version 1.3.0, and its details show 1.2.0 as an older installed version

### Requirement: Vehicle Library dialog
Vehicle › Vehicle Library… SHALL open a dialog with two tabs, a list and a detail pane:
- **Tabs:** **Installed**, every installed vehicle, and **My vehicles**, the custom vehicles (id starting with `custom.`). Each tab shows its count.
- **The list**, under a search field (name and brand) and the game and kind filters. Each row shows the vehicle's name, its brand, kind and game, its newest installed version, and a **status**, as text with an icon, never by color alone:
  - **Custom** for a custom vehicle, in the link color;
  - **Update** when the open project holds that vehicle at an older version than the newest installed one, in the signal color;
  - **Up to date** otherwise.
- **The detail pane** shows the vehicle chosen in the list (the first one when the dialog opens):
  - a preview of its template: the package's preview image when it has one, otherwise the template of its first main texture;
  - its name, brand, kind and game;
  - the newest installed version and its supported game versions, or "any version" when the package sets no limit;
  - its paint job: the names of its main textures and the number of accessories;
  - every installed version, each with **Export…** and **Remove**.

It SHALL offer:
- **Install…**;
- **Custom Vehicle…**, which opens the Custom Vehicle dialog (see the custom-vehicles capability);
- **New Version…** for a custom vehicle, in its detail pane, which opens the Custom Vehicle dialog filled in from its newest version;
- **Update Template…** in the detail pane of a vehicle whose status is Update. It closes the library and opens Update Template for that vehicle of the open project (see the vehicle-projects capability);
- **Export…** for a vehicle version. It saves that version's package, unchanged, as a `.tpv` file where the player chooses, proposing `<id>-<version>.tpv`. The save dialog SHALL remind the player that templates from the base games belong to SCS Software, and to check that they may share them;
- **Remove** for a vehicle version, after confirmation. Removing never changes projects, which embed their templates.

With no vehicle installed, the dialog SHALL show an empty state explaining what packages are and offering Install… and Custom Vehicle…. With no custom vehicle, the My vehicles tab SHALL explain that custom vehicles are made from template files and offer Custom Vehicle….

#### Scenario: Remove a version
- **WHEN** the user removes version 1.2.0 of a vehicle that also has 1.3.0 installed
- **THEN** only 1.3.0 remains in the library, and an open project made with 1.2.0 still shows its templates

#### Scenario: Paint job summary
- **WHEN** the sample truck 1.1.0 is installed, the Vehicle Library is open and the user chooses it in the list
- **THEN** its detail pane lists the main textures "Standard cab" and "High roof" and three accessories, with a preview of its template

#### Scenario: Export a version
- **WHEN** the user clicks Export… on version 1.0.0 of `custom.scania.r_2024` and chooses a folder
- **THEN** `custom.scania.r_2024-1.0.0.tpv` is written there with the same bytes as the installed package

#### Scenario: Any game version
- **WHEN** a custom vehicle was created with no game versions
- **THEN** its detail pane shows "any version" as its supported game versions

#### Scenario: Custom vehicle from the library
- **WHEN** the user clicks Custom Vehicle… in the Vehicle Library and creates a vehicle
- **THEN** the Vehicle Library is shown again and lists the new vehicle

#### Scenario: My vehicles
- **WHEN** the sample truck, the sample trailer and the custom vehicle `custom.scania.r_2024` are installed
- **THEN** the Installed tab shows 3 vehicles, the My vehicles tab shows 1, and the custom vehicle's status is Custom

#### Scenario: Update status
- **WHEN** a project made with the sample truck 1.0.0 is open and the sample truck 1.1.0 is installed
- **THEN** the sample truck's status in the Vehicle Library is Update, and its detail pane offers Update Template…, which opens Update Template for that vehicle

#### Scenario: Up to date without a project
- **WHEN** the Vehicle Library is opened from the home screen with the sample truck 1.0.0 and 1.1.0 installed
- **THEN** the sample truck's status is Up to date

### Requirement: Built-in sample vehicle
TruckPaint SHALL include the newest versions of the sample truck and the sample trailer (see the vehicle-authoring capability). When no vehicle is installed, an **Install the sample vehicles** button SHALL appear in two places:
- the empty state of the Vehicle Library;
- the vehicle list of New Project.

The button SHALL install both samples like any other package, with the same confirmation and the same errors. Once a vehicle is installed, the button SHALL no longer be shown. Installing the samples SHALL need no network access.

#### Scenario: First vehicle project
- **WHEN** a painter with no vehicle installed opens New Project and clicks Install the sample vehicles
- **THEN** "TruckPaint Sample Truck" and "TruckPaint Sample Trailer" appear in the list with their newest versions, a confirmation names them, and either can be picked to create a project

#### Scenario: Library with vehicles
- **WHEN** at least one vehicle is installed
- **THEN** neither the Vehicle Library nor New Project shows Install the sample vehicles
