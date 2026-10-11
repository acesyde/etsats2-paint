## MODIFIED Requirements

### Requirement: Templates
The player SHALL add templates by dropping files on the dialog or with **Add Templates…**, one or more at a time. Accepted files:
- PNG;
- SVG;
- DDS, in the formats `tpv` accepts: BC1, BC2 or BC3 (also with a DX10 header), and uncompressed 24 or 32-bit RGB(A).

Each file SHALL become one **texture row**, in the order added, with:
- **Name:** the file name without its extension, with `_` and `-` shown as spaces (`highline_8x4.dds` gives "highline 8x4"); editable;
- **Role:** Main texture or Accessory. A row is a main texture when the vehicle has no main texture yet, an accessory otherwise; the player can change it;
- **Game ids:** empty; editable as a list separated by commas or spaces;
- **Size:** a width and a height, each a power of two from 64 to 8192. They default to the image's width and height, each rounded to the nearest allowed side when it isn't one. The player can change either.

A row SHALL show its file name and the image's dimensions. It SHALL warn when the image's size differs from the texture's size, since the template is then scaled to the texture.

Each row SHALL offer:
- **Replace…**, which picks another file for that row; dropping a single file on a row does the same;
- **Remove**.

A file that can't be read SHALL not become a row. The dialog SHALL name the file and the reason, for example a DDS compression that isn't supported. Other files of the same drop SHALL still be added.

#### Scenario: Drop the game's templates
- **WHEN** the player drops `cabin.dds` (BC3, 4096×4096) and `mirrors.dds` (BC1, 1024×1024) on an empty dialog
- **THEN** the dialog lists "cabin" as a main texture of size 4096² and "mirrors" as an accessory of size 1024²

#### Scenario: Unsupported DDS
- **WHEN** the player drops `cabin.png` and `glass.dds`, a BC7 DDS file
- **THEN** "cabin" is added, and the dialog says that `glass.dds` can't be read because its compression is not supported

#### Scenario: Replace a template
- **WHEN** the player clicks Replace… on the "cabin" row and picks `cabin_v2.png`
- **THEN** the row keeps its name, role and game ids, and shows `cabin_v2.png`

#### Scenario: Not square
- **WHEN** the player adds a 4096×2048 PNG
- **THEN** its row has the size 4096 × 2048 and shows no warning: the texture keeps the image's shape

#### Scenario: Size that isn't a power of two
- **WHEN** the player adds a 4000×1000 PNG
- **THEN** its row has the size 4096 × 1024 and warns that the template will be scaled to that size
