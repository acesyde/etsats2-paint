//! Design tokens: the single source of truth for colors, spacing, sizes and
//! typography. Widgets and the egui theme are derived from these values.

/// Color tokens (dark theme), named by meaning.
///
/// The theme is low in saturation so that livery colors read true. Surfaces
/// go up in elevation from the canvas to the raised level; three accents
/// each carry one meaning and are used for nothing else:
/// - [`color::ACCENT_PRIMARY`] (white): the primary action of a screen or
///   dialog, the active option of a segmented control, the active tool;
/// - [`color::SIGNAL`] (red): alerts, "update available", the selection on
///   the canvas;
/// - [`color::LINK`] (blue): everything linked to the brand (a fill linked to
///   a palette swatch, an object following a shared style, a symbol
///   instance).
pub mod color {
    use egui::Color32;

    /// Canvas surface: pasteboard and canvas area (lowest elevation).
    pub const SURFACE_0: Color32 = Color32::from_rgb(0x12, 0x12, 0x14);
    /// Panel surface: bars, panels, dialogs.
    pub const SURFACE_1: Color32 = Color32::from_rgb(0x16, 0x16, 0x18);
    /// Raised surface: fields, rows, popovers, menus.
    pub const SURFACE_2: Color32 = Color32::from_rgb(0x23, 0x23, 0x26);
    /// Hovered controls.
    pub const SURFACE_3: Color32 = Color32::from_rgb(0x2C, 0x2C, 0x30);
    /// Pressed controls.
    pub const SURFACE_4: Color32 = Color32::from_rgb(0x36, 0x36, 0x3A);

    /// The canvas surface, by meaning.
    pub const CANVAS: Color32 = SURFACE_0;
    /// The panel surface, by meaning.
    pub const PANEL: Color32 = SURFACE_1;
    /// The raised surface, by meaning.
    pub const RAISED: Color32 = SURFACE_2;

    /// Background of the selected row of a list (with an indicator bar, so
    /// the selection does not rely on color alone).
    pub const SELECTED: Color32 = Color32::from_rgb(0x34, 0x34, 0x3A);

    /// Hairline separators between regions.
    pub const BORDER: Color32 = Color32::from_rgb(0x2A, 0x2A, 0x2E);
    /// Control outlines (inputs, popups).
    pub const BORDER_STRONG: Color32 = Color32::from_rgb(0x3A, 0x3A, 0x3F);

    /// Ink: primary text.
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xED, 0xED, 0xED);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0xA9, 0xA9, 0xB0);
    pub const TEXT_DISABLED: Color32 = Color32::from_rgb(0x76, 0x76, 0x7D);

    /// Primary accent (white): primary button, active segment, active tool.
    pub const ACCENT_PRIMARY: Color32 = Color32::from_rgb(0xEC, 0xEC, 0xEC);
    /// Dark text drawn on [`ACCENT_PRIMARY`] (primary button, active segment).
    pub const TEXT_ON_PRIMARY: Color32 = SURFACE_0;
    /// Signal accent (red): alerts, update available, canvas selection.
    pub const SIGNAL: Color32 = Color32::from_rgb(0xF0, 0x52, 0x52);
    /// Link accent (blue, oklch 0.74 0.11 225): linked to the brand.
    pub const LINK: Color32 = Color32::from_rgb(0x50, 0xB9, 0xDF);

    /// Bars and markers of an active or selected item (selected list row,
    /// active tool, active swatch, drop position): the primary accent.
    pub const INDICATOR: Color32 = ACCENT_PRIMARY;
    /// Keyboard focus ring: ink, so it reads on every surface without taking
    /// one of the accents' meanings.
    pub const FOCUS: Color32 = TEXT_PRIMARY;

    pub const SUCCESS: Color32 = Color32::from_rgb(0x6F, 0xC8, 0x84);
    pub const WARNING: Color32 = Color32::from_rgb(0xF0, 0xB4, 0x4C);
    pub const ERROR: Color32 = Color32::from_rgb(0xFF, 0x7A, 0x72);

    /// Modal backdrop.
    pub const BACKDROP: Color32 = Color32::from_black_alpha(150);
    /// Drop shadows.
    pub const SHADOW: Color32 = Color32::from_black_alpha(110);
}

/// Spacing scale in logical points.
pub mod space {
    pub const XXS: f32 = 2.0;
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 32.0;
}

/// Corner radii.
pub mod radius {
    pub const SM: u8 = 4;
    pub const MD: u8 = 6;
    pub const LG: u8 = 8;
}

/// Stroke widths.
pub mod stroke {
    pub const HAIRLINE: f32 = 1.0;
    pub const FOCUS: f32 = 1.5;
    /// Width of the active indicator bar on tool buttons and list rows.
    pub const INDICATOR: f32 = 3.0;
}

/// Fixed component sizes.
pub mod size {
    /// Minimum interactive area of any clickable control.
    pub const HIT_MIN: f32 = 24.0;
    pub const ICON: f32 = 16.0;
    pub const ICON_LG: f32 = 18.0;
    pub const TOOL_BUTTON: f32 = 32.0;
    /// Width of the Workshop's tool rail.
    pub const TOOL_RAIL_WIDTH: f32 = 48.0;
    pub const MENU_BAR_HEIGHT: f32 = 30.0;
    /// Height of the top bar of an open project (name, spaces, Export…).
    pub const TOP_BAR_HEIGHT: f32 = 44.0;
    /// Height of the Workshop's tool options bar.
    pub const TOOL_OPTIONS_HEIGHT: f32 = 36.0;
    pub const STATUS_BAR_HEIGHT: f32 = 28.0;
    pub const PANEL_HEADER_HEIGHT: f32 = 28.0;
    /// Width of the Workshop's left panel (Textures, Layers, Resources).
    pub const LEFT_PANEL_DEFAULT: f32 = 260.0;
    pub const LEFT_PANEL_MIN: f32 = 200.0;
    pub const LEFT_PANEL_MAX: f32 = 420.0;
    /// Width of the Workshop's inspector.
    pub const INSPECTOR_DEFAULT: f32 = 280.0;
    pub const INSPECTOR_MIN: f32 = 220.0;
    pub const INSPECTOR_MAX: f32 = 480.0;
}

/// Typographic scale (points at 100% text size). Weights: title and heading
/// 600 (Geist SemiBold), the others 400; [`MONO`] is JetBrains Mono, for
/// values (numbers, sizes, positions, hex colors, paths).
pub mod typography {
    pub const CAPTION: f32 = 11.0;
    pub const BODY: f32 = 13.0;
    pub const LABEL: f32 = 13.0;
    pub const HEADING: f32 = 15.0;
    pub const TITLE: f32 = 22.0;
    /// Home screen hero text.
    pub const DISPLAY: f32 = 30.0;
    pub const MONO: f32 = 12.0;
}

/// Canvas overlay styling (selection, handles, guides drawn over artwork).
pub mod canvas {
    use egui::Color32;

    /// Selection outlines and handle borders: the signal accent, saturated so
    /// they stand out on light and dark artwork alike (always drawn over a
    /// halo).
    pub const SELECTION: Color32 = super::color::SIGNAL;
    /// Contrasting halo drawn under every overlay line.
    pub const HALO: Color32 = Color32::from_rgba_premultiplied(230, 230, 230, 230);
    pub const HANDLE_FILL: Color32 = Color32::WHITE;
    /// Highlight behind selected characters while editing a text.
    pub const TEXT_SELECTION: Color32 = Color32::from_rgba_premultiplied(84, 29, 29, 90);
    /// Caret of the text being edited.
    pub const CARET: Color32 = SELECTION;
    /// Placeholder of an image still loading.
    pub const IMAGE_PLACEHOLDER: Color32 = Color32::from_rgb(0xC8, 0xCA, 0xCF);
    pub const MARQUEE_FILL: Color32 = Color32::from_rgba_premultiplied(38, 13, 13, 40);
    /// Pasteboard around the artboard.
    pub const PASTEBOARD: Color32 = super::color::CANVAS;
    /// Light and dark cells of the checkerboard behind transparent colors
    /// and previews.
    pub const CHECKER_LIGHT: Color32 = Color32::from_gray(200);
    pub const CHECKER_DARK: Color32 = Color32::from_gray(150);
    /// Empty artboard (texture background).
    pub const ARTBOARD: Color32 = Color32::from_rgb(0xE6, 0xE7, 0xEA);

    /// Side of a resize handle square, in points.
    pub const HANDLE_SIZE: f32 = 8.0;
    /// Distance at which a press grabs a handle, in points.
    pub const HANDLE_HIT_RADIUS: f32 = 7.0;
    /// Distance from a corner within which a press outside the bounds rotates.
    pub const ROTATE_ZONE: f32 = 28.0;
    pub const CENTER_MARK: f32 = 6.0;
    pub const LINE: f32 = 1.0;
    /// Hit tolerance for clicking objects, in points.
    pub const HIT_TOLERANCE: f32 = 4.0;
    /// Pointer travel before a press becomes a drag, in points.
    pub const DRAG_THRESHOLD: f32 = 3.0;

    /// Side of a path anchor point square, in points.
    pub const POINT_SIZE: f32 = 7.0;
    /// Diameter of a path handle end, in points.
    pub const HANDLE_DOT: f32 = 5.0;
    /// Distance at which a press grabs a path point or handle, in points.
    pub const POINT_HIT_RADIUS: f32 = 6.0;

    /// Thickness of the rulers along the canvas, in points.
    pub const RULER_SIZE: f32 = 20.0;
    pub const RULER_BG: Color32 = super::color::PANEL;
    pub const RULER_TICK: Color32 = Color32::from_rgb(0x5A, 0x5A, 0x60);
    pub const RULER_TEXT: Color32 = super::color::TEXT_SECONDARY;
    /// Pointer position marker on the rulers.
    pub const RULER_MARKER: Color32 = SELECTION;
    /// Minimum distance between labelled ruler graduations, in points.
    pub const RULER_LABEL_GAP: f32 = 60.0;
    /// Guide lines.
    pub const GUIDE: Color32 = Color32::from_rgb(0x00, 0xC8, 0xE8);
    /// Distance at which a press grabs a guide, in points.
    pub const GUIDE_HIT: f32 = 4.0;
    pub const GRID_MINOR: Color32 = Color32::from_rgba_premultiplied(40, 40, 48, 40);
    pub const GRID_MAJOR: Color32 = Color32::from_rgba_premultiplied(30, 30, 40, 90);
    /// Minimum distance between drawn grid lines, in points.
    pub const GRID_MIN_GAP: f32 = 8.0;
    /// Alignment lines and marks of snapping.
    pub const SNAP: Color32 = Color32::from_rgb(0xFF, 0x2D, 0xA6);
    /// Distance within which positions snap, in points.
    pub const SNAP_DISTANCE: f32 = 6.0;
    /// Outline of the key object (Align to: Key object), in points.
    pub const KEY_OBJECT_LINE: f32 = 2.5;
}
