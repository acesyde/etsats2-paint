
## Export sizes and errors

size-mb = { $value } MB
size-kb = { $value } KB
size-about = about { $value }
export-stopped = the export stopped unexpectedly

## Saving and opening projects

file-save-failed-title = Could not save
file-save-failed = { $file } could not be saved.
file-save-failed-reason = { $file } could not be saved ({ $reason }). Your changes are still open.
file-open-failed-title = Cannot open project
file-restore-failed-title = Cannot restore project
file-recovered-copy = The recovered copy
file-not-a-project = { $file } is not a TruckPaint project.
file-damaged = { $file } is damaged and cannot be opened.
file-newer-version = { $file } was created with a newer version of TruckPaint.
file-io = { $file } could not be read or written ({ $reason }).

## File dialogs

dialog-open-project = Open Project
dialog-save-project = Save Project
dialog-export-texture = Export Texture
filter-project = TruckPaint project
filter-images = Images

## Home screen

home-recovered = Recovered projects
home-recovered-hint = TruckPaint closed unexpectedly. These projects had unsaved changes.
home-never-saved = Never saved
home-recovery-damaged = Damaged recovery copy
home-recovery-damaged-hint = This copy cannot be restored.
home-recovered-item = Recovered { $name }
home-restore = Restore
home-discard = Discard
app-tagline = Livery editor for Euro Truck Simulator 2 and American Truck Simulator
home-new-project = New Project
home-new-project-tip = Create a new livery project ({ $shortcut })
home-preferences = Preferences
home-recent = Recent projects
home-recent-empty = No recent projects
home-recent-empty-hint = Projects you open will appear here. Start by creating a new project.
home-recent-missing-item = { $name } (file not found)
home-remove-recent = Remove from list
home-file-not-found = File not found
home-open-recent = Open { $path }
home-file-moved = This project's file was moved or deleted.

## Dialogs

unsaved-title = Save changes to “{ $name }” before closing?
unsaved-hint = Your changes will be lost if you don't save them.
button-dont-save = Don't Save
button-cancel = Cancel
button-ok = OK
button-create = Create
button-done = Done
button-close = Close
new-project-step-name = Name
new-project-name = Project name
prefs-interface = Interface
prefs-ui-scale = UI scale
prefs-text-size = Text size
prefs-scale-hint = Changes apply immediately. 100% follows your display's scale factor.
prefs-canvas = Canvas
prefs-grid-spacing = Grid spacing
prefs-reset = Reset to defaults
shortcuts-temporary-hand = Temporary Hand tool
shortcuts-hold-space = Hold Space
about-version = Version { $version }
about-tagline = Livery editor for Euro Truck Simulator 2 and American Truck Simulator.

## Export Texture dialog

export-done = Exported { $file }
export-failed-title = Could not export
export-failed = { $file } could not be written ({ $reason }).
export-preview = Export preview
export-info = { $size } × { $size } px · { $format } · { $bytes }
export-progress = Exporting { $file }…
export-start = Export…
export-format = Format
export-dds-bc3 = BC3 / DXT5 (recommended)
export-dds-rgba = Uncompressed RGBA
export-dds-encoding = DDS encoding
export-size = Size
export-background = Background
export-transparent = Transparent
export-background-color = Background color
prefs-language = Language
file-dev-format = { $file } uses a development format that this version cannot open.

## Vehicle packages

pkg-install-failed = { $file } could not be installed: { $reason }
pkg-no-library = there is no vehicle library folder.
pkg-io = it could not be read or written ({ $reason }).
pkg-not-a-zip = it is not a vehicle package.
pkg-no-manifest = it has no vehicle.json manifest.
pkg-bad-manifest = its manifest is invalid ({ $reason }).
pkg-newer-format = it was made for a newer version of TruckPaint.
pkg-bad-id = its id “{ $id }” is invalid.
pkg-no-main-texture = it has no main texture.
pkg-bad-game-path = its game path “{ $path }” is not valid (words of a–z, 0–9 and _ separated by dots).
pkg-bad-game-id = the game id “{ $id }” is not valid (words of a–z, 0–9 and _ separated by dots).
pkg-missing-game-ids = “{ $texture }” has no game id.
pkg-duplicate-part = two textures have the id “{ $id }”.
pkg-duplicate-game-id = the game id “{ $id }” is listed twice.
pkg-bad-size = the texture “{ $texture }” has an invalid size ({ $size }).
pkg-unsafe-path = it contains an unsafe path ({ $path }).
pkg-too-large = it is too large.
pkg-missing-template = the template of “{ $texture }” is missing.
pkg-bad-template = the template of “{ $texture }” is not a readable PNG or SVG image.
pkg-template-too-large = the template of “{ $texture }” is too large.

## Vehicle library and updates

filter-packages = Vehicle packages
vehicles-search = Search vehicles
vehicles-all-games = All games
vehicles-game-filter = Game
vehicles-all-kinds = Trucks and trailers
vehicles-kind-filter = Kind
vehicles-truck = Truck
vehicles-trailer = Trailer
vehicles-installed = Installed { $name } { $version }
vehicles-empty = No vehicle installed
vehicles-empty-hint = Vehicle packages (.tpv) hold the templates of a truck or trailer. Install one to start a livery on it.
vehicles-details = Game versions { $versions } · { $main } · Accessories: { $accessories }
vehicles-version = Version { $version }
vehicles-remove-version = Remove { $name } { $version }
vehicles-remove-confirm = Remove { $name } from the library? Projects keep their templates.
vehicles-remove = Remove
vehicles-install = Install…
vehicles-install-sample = Install the sample vehicles
update-versions = { $name }: version { $from } → { $to }
update-replaced = { $name }: template replaced
update-layout-changed = { $name }: layout changed, check the artwork
update-resized = { $name }: size { $old } → { $new } px, artwork scaled
update-new-textures = New in this version:
update-removed = { $name }: no longer in this version, artwork kept
update-artwork-kept = Your artwork is kept. You can undo the update.
update-apply = Update

## New Project wizard

new-project-step-vehicle = Vehicle
new-project-no-vehicles-hint = No vehicle yet? Try the sample truck, or install a package.
new-project-textures = Textures
button-next = Next
button-back = Back

## Fleet

add-vehicle-none = No other vehicle of this game is installed. Install a package to add one.
add-vehicle-add = Add
fleet-error-other-game = This vehicle is for another game than the project.
fleet-error-already-there = This vehicle is already in the project.
fleet-error-textures = Choose at least one main texture of this vehicle.
fleet-error-version = The installed version differs from the one the project uses.
fleet-error-unknown = This vehicle is not in the project.
textures-title = Textures of { $name }
textures-missing-version = Version { $version } of this vehicle is not installed. Update its template to change its textures.
textures-remove-confirm = The artwork of { $textures } will be removed with them.
textures-remove = Remove
textures-apply = Apply
remove-vehicle-confirm = Remove { $name } and its artwork from the project?
