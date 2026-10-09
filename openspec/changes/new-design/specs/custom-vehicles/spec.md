## MODIFIED Requirements

### Requirement: Custom Vehicle dialog
The **Custom Vehicle** dialog SHALL be reached from:
- **Custom vehicle…** in the vehicle list of New Project (see the start-screen capability);
- **Custom vehicle…** in Add Vehicle… (see the vehicle-projects capability);
- **Custom Vehicle…** in the Vehicle Library (see the vehicle-packages capability).

It SHALL ask for:
- **Name** and **Brand**, both required;
- **Kind:** truck or trailer;
- **Game:** ETS2 or ATS. Opened from Add Vehicle…, the game is the project's game and can't be changed;
- **Game path:** the vehicle's path in the game's definitions (`scania.r_2016`), required;
- **Game versions:** optional. Empty means any game version; otherwise a version range (`>=1.53`);
- **Alternate UV set** and **Colour picker:** unchecked by default;
- the **templates**, as described by the Templates requirement.

The dialog SHALL remind the player that templates from the base games belong to SCS Software and are for their own use.

Cancel or Escape SHALL close the dialog without installing anything and return to the dialog it was opened from, unchanged. The dialog SHALL be operable by keyboard.

#### Scenario: Opened from New Project
- **WHEN** the player clicks Custom vehicle… in the vehicle list of New Project
- **THEN** the Custom Vehicle dialog opens with an empty form, Kind set to Truck and Game set to ETS2

#### Scenario: Opened from Add Vehicle
- **WHEN** the player clicks Custom vehicle… in Add Vehicle… of an ATS project
- **THEN** the dialog's game is ATS and can't be changed

#### Scenario: Cancelling
- **WHEN** the player fills in part of the form and presses Escape
- **THEN** the dialog closes, nothing is installed, and the dialog it was opened from is shown as it was

### Requirement: Creating a custom vehicle
**Create** SHALL build a vehicle package as `tpv pack` does (see the vehicle-authoring capability):
- **id:** `custom.<brand>.<name>`. Each word is the brand or the name, lowercased, with characters outside `a`–`z`, `0`–`9`, `_` and `-` replaced by `_`, repeated `_` collapsed, and `_` trimmed at both ends. A word left empty becomes `vehicle`;
- **version:** 1.0.0;
- **every texture** at layout version 1. It gets a texture id made from its name the same way, unique within the vehicle, and its template;
- **DDS templates** converted to PNG of the same pixels.

The package SHALL pass the same validation as installing a package. It SHALL then be installed in the library, like an installed `.tpv`. While the package is built, the dialog SHALL show that it is working and SHALL stay responsive.

On success, the dialog SHALL close, and the dialog it was opened from SHALL show the new vehicle:
- in New Project and Add Vehicle…, it is selected, with its first main texture and every accessory checked;
- in the Vehicle Library, it is listed.

A confirmation SHALL name the vehicle and its version.

When a vehicle with the same id is already installed, Create SHALL be refused. The message SHALL say so and point to New Version… in the Vehicle Library. When building or validation fails, the dialog SHALL stay open with the reason. Nothing SHALL be installed then.

#### Scenario: Create from New Project
- **WHEN** the player creates the custom truck "R 2024" of brand "Scania", with one main texture "cabin" and one accessory, from the vehicle list of New Project
- **THEN** the package `custom.scania.r_2024` 1.0.0 is installed, and the vehicle list of New Project shows "R 2024" selected with "cabin" and the accessory checked

#### Scenario: DDS converted
- **WHEN** a custom vehicle is created from a BC3 DDS template of 4096×4096
- **THEN** the installed package holds a 4096×4096 PNG template of the same pixels

#### Scenario: Same id already installed
- **WHEN** the player creates the custom truck "R 2024" of brand "Scania" while `custom.scania.r_2024` is already installed
- **THEN** Create is refused with a message pointing to New Version…, and the library is unchanged

#### Scenario: Shared like any package
- **WHEN** a custom vehicle is exported from the Vehicle Library and installed on another computer
- **THEN** it is accepted and listed like any other vehicle
