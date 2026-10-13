## MODIFIED Requirements

### Requirement: Content sections
The documentation SHALL be organized in these sections, in this order, in every language: Getting started (installation, first launch, first project); Vehicles (Vehicle Library, official package extraction and manual installation, custom vehicles); Project and fleet (trucks, variants, trailers); Workshop (vector tools, text, images, gradients, layers); Brand (palette, symbols, styles); Export (textures, mod); AI assistant (MCP). Its wording SHALL follow the product's rules and vocabulary (a project is a fleet of one game; sharing across vehicles is by elements, never by copied placement).

#### Scenario: Sidebar
- **WHEN** a reader opens any page
- **THEN** the sidebar lists the seven sections in the order above, titled in the page's language

The official vehicle documentation SHALL describe the maintained ETS2 extraction command, external ConverterPIX requirement, installed vehicle/tuning DLC coverage, explicit package version, unique batch outputs, technical report and unsupported-case failures, followed by manual package installation, project creation and mod export. It SHALL explain that old Carsmaniac packages are retired without migration and that reusing an installed id/version replaces that version under the existing installer behavior. It SHALL NOT instruct players to use the removed catalog or template pack settings. Newly generated vehicles SHALL be described as requiring in-game checks unless matching evidence exists.

#### Scenario: Current official vehicle workflow
- **WHEN** a reader follows the official vehicle documentation
- **THEN** it leads from extraction or an available .tpv through manual installation to a project and mod export, without a Carsmaniac download or catalog selection step
