
## Panel titles and empty states

panel-properties = Properties
panel-layers = Layers
panel-colors = Colors
panel-stroke = Stroke
panel-transform = Transform
panel-assets = Assets
panel-vehicle = Vehicles
empty-properties = Nothing selected
empty-properties-hint = Select an object on the canvas to edit its properties.
empty-layers = No layers yet
empty-layers-hint = Shapes, text and images you add will appear here.
empty-colors = No color selected
empty-colors-hint = Pick a fill or stroke to edit its color.
empty-stroke = No stroke
empty-stroke-hint = Select an object to edit its outline.
empty-transform = Nothing to transform
empty-transform-hint = Select an object to edit position, size and rotation.
empty-assets = No assets
empty-assets-hint = Imported logos and images will be listed here.
empty-vehicle = No vehicle
empty-vehicle-hint = Vehicle templates will be available in a future update.

## 3D preview

preview-hide = Hide 3D Preview
preview-soon = 3D preview coming soon
preview-soon-hint = Your livery will be shown on the vehicle model here once vehicle models are available.

## Panels and Layers

panels-all-closed = All panels are closed
panels-all-closed-hint = Reopen panels from the View menu, or reset the workspace.
panel-expand = Expand
panel-collapse = Collapse
panel-close = Close Panel
layers-empty-hint = Shapes you draw and layers you add appear here.
layers-collapse = Collapse { $name }
layers-expand = Expand { $name }
layers-hide = Hide { $name }
layers-show = Show { $name }
layers-lock = Lock { $name }
layers-unlock = Unlock { $name }
layers-name = Layer name

## Assets

assets-uses = { $count ->
    [one] { $count } use
   *[other] { $count } uses
}
assets-used-by = { $count ->
    [one] Used by { $count } object
   *[other] Used by { $count } objects
}
assets-vector = Vector
assets-item = Asset { $name }
assets-place = Place Asset
assets-name = Asset name

## Stroke panel

stroke-enabled = Stroke enabled
stroke-width = Stroke width
stroke-center = Center
stroke-center-name = Center stroke
stroke-inside = Inside
stroke-inside-name = Inside stroke
stroke-outside = Outside
stroke-outside-name = Outside stroke
undo-change-stroke-width = Change Stroke Width
undo-change-stroke-alignment = Change Stroke Alignment
undo-add-stroke = Add Stroke
undo-remove-stroke = Remove Stroke

## Transform panel

field-width = Width
field-height = Height
field-w = W
field-h = H
field-r = R
field-s = S
transform-x = X position
transform-y = Y position
transform-rotation = Rotation
transform-scale = Scale
transform-unlock-proportions = Unlock proportions
transform-lock-proportions = Lock proportions
transform-align-to = Align to

## Properties panel

undo-change-opacity = Change Opacity
undo-change-corner-radius = Change Corner Radius
undo-change-sides = Change Sides
undo-change-inner-radius = Change Inner Radius
undo-change-line-width = Change Line Width
undo-change-star = Change Star
props-sides = Sides
props-star = Star
props-inner = Inner
props-inner-radius = Inner radius
props-surface-summary = { $size } × { $size } px · Select an object to edit it
props-objects = { $count } objects
props-multiple = Multiple selection
props-source-svg = SVG · Vector
props-source-image = Image · { $width } × { $height } px
props-source = Source: { $name } · { $source }
props-opacity = Opacity
props-opacity-slider = Opacity slider
props-radius = Radius
props-corner-radius = Corner radius
props-line-width = Line width
props-fill = Fill
props-fill-color = Fill color
props-stroke-color = Stroke color

## Dashes, caps and joins

line-preset-dashed = Dashed 20/10
line-preset-dotted = Dotted 0/12
line-preset-long-dash = Long dash 60/20
undo-change-stroke-caps = Change Stroke Caps
undo-change-stroke-joins = Change Stroke Joins
undo-change-stroke-dashes = Change Stroke Dashes
undo-change-line-caps = Change Line Caps
undo-change-line-joins = Change Line Joins
undo-change-line-dashes = Change Line Dashes
line-dashed = Dashed
line-presets = Presets
line-dash-presets = Dash presets
line-dash = Dash
line-dash-length = Dash length
line-gap = Gap
line-gap-length = Gap length
line-cap = Cap
line-butt = Butt
line-butt-cap = Butt cap
line-round = Round
line-round-cap = Round cap
line-square = Square
line-square-cap = Square cap
line-join = Join
line-miter = Miter
line-miter-join = Miter join
line-round-join = Round join
line-bevel = Bevel
line-bevel-join = Bevel join
line-limit = Limit
line-miter-limit = Miter limit

## Colors panel

colors-no-stroke = No stroke
colors-solid = Solid
colors-solid-paint = Solid paint
colors-linear = Linear
colors-linear-gradient = Linear gradient
colors-radial = Radial
colors-radial-gradient = Radial gradient
colors-location = Location
colors-stop-location = Stop location
colors-angle = Angle
colors-gradient-angle = Gradient angle
colors-reverse = Reverse gradient
colors-red = Red
colors-green = Green
colors-blue = Blue
colors-hue = Hue
colors-saturation = Saturation
colors-value = Value
colors-lightness = Lightness
colors-alpha = Alpha
colors-hex = Hex
colors-hex-color = Hex color
colors-hex-invalid = Invalid hex color. Use #RGB, #RRGGBB or #RRGGBBAA.
colors-recent = Recent
colors-recent-item = Recent color { $color }
colors-palette = Palette
colors-palette-item = Palette color { $color }
colors-add-to-palette = Add to Palette
colors-differ = Colors differ in the selection.
colors-palette-empty = Save colors you reuse with +.
colors-remove-from-palette = Remove from Palette
colors-aspect = Aspect
colors-aspect-ratio = Gradient aspect ratio

## Character panel

weight-thin = Thin
weight-extra-light = Extra Light
weight-light = Light
weight-regular = Regular
weight-medium = Medium
weight-semi-bold = Semi Bold
weight-bold = Bold
weight-extra-bold = Extra Bold
weight-black = Black
char-title = Character
char-font-weight = Font weight
char-italic = Italic
char-size = Size
char-font-size = Font size
char-align-left = Align left
char-align-center = Align center
char-align-right = Align right
char-tracking = Tracking
char-letter-spacing = Letter spacing
char-line = Line
char-line-height = Line height
char-family-missing-name = Font family: { $family } (Font not found: { $family })
char-family-name = Font family: { $family }
char-font-not-found = Font not found: { $family }
char-font-family = Font family
char-search-fonts = Search fonts
char-no-fonts = No matching fonts
undo-change-font-weight = Change Font Weight
undo-change-italic = Change Italic
undo-change-text-size = Change Text Size
undo-change-alignment = Change Alignment
undo-change-letter-spacing = Change Letter Spacing
undo-change-line-height = Change Line Height
undo-change-font = Change Font

## Vehicle panel

vehicle-panel-package = Package { $version } · game versions { $games }
vehicle-panel-package-missing = Package { $version } (not installed)
vehicle-panel-update = Version { $version } is available
vehicle-panel-texture = Texture { $name }
vehicle-panel-layout-changed = Layout changed
vehicle-panel-dismiss = Dismiss
vehicle-panel-dismiss-named = Dismiss layout change of { $name }
vehicle-panel-removed = Not in this version
vehicle-panel-opacity = Template
vehicle-panel-opacity-name = Template opacity

## Fleet

vehicle-panel-update-named = Update the template of { $name }
vehicles-variants = Variants…
vehicles-remove-from-project = Remove from Project

## Vehicles sidebar


## Sidebar

sidebar-project = Project
sidebar-show = Show Sidebar
sidebar-hide = Hide Sidebar
sidebar-vehicles-game = Vehicles · { $game }
project-name = Name
project-version = Version
project-game-versions = Game versions
vehicle-actions-named = Actions for { $name }
vehicle-panel-needs-check = A texture needs checking after an update
texture-layout-changed = The layout of this texture changed in version { $version }: check your artwork.
texture-not-in-version = This texture is not in version { $version } of the vehicle: it has no template.
