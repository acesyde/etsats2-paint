//! Design tokens: the single source of truth for colors, spacing, sizes and
//! typography. Widgets and the egui theme are derived from these values.

/// Color tokens (dark theme).
pub mod color {
    use egui::Color32;

    /// Pasteboard / application background (lowest elevation).
    pub const SURFACE_0: Color32 = Color32::from_rgb(0x18, 0x19, 0x1C);
    /// Panels, bars.
    pub const SURFACE_1: Color32 = Color32::from_rgb(0x22, 0x23, 0x27);
    /// Panel headers, inputs, popups, dialogs.
    pub const SURFACE_2: Color32 = Color32::from_rgb(0x2A, 0x2C, 0x31);
    /// Hovered controls.
    pub const SURFACE_3: Color32 = Color32::from_rgb(0x33, 0x35, 0x3B);
    /// Pressed controls.
    pub const SURFACE_4: Color32 = Color32::from_rgb(0x3C, 0x3F, 0x46);

    /// Hairline separators between regions.
    pub const BORDER: Color32 = Color32::from_rgb(0x34, 0x36, 0x3C);
    /// Control outlines (inputs, popups).
    pub const BORDER_STRONG: Color32 = Color32::from_rgb(0x48, 0x4B, 0x53);

    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xF2, 0xF3, 0xF5);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0xB3, 0xB7, 0xBF);
    pub const TEXT_DISABLED: Color32 = Color32::from_rgb(0x86, 0x8B, 0x94);

    /// Accent for indicators, focus rings and selection outlines (non-text).
    pub const ACCENT: Color32 = Color32::from_rgb(0x73, 0xAB, 0xFF);
    /// Accent fill for primary buttons; carries white text.
    pub const ACCENT_FILL: Color32 = Color32::from_rgb(0x1A, 0x4D, 0xB5);
    pub const ACCENT_FILL_HOVER: Color32 = Color32::from_rgb(0x1F, 0x56, 0xC3);
    /// Subtle accent background for active/selected items.
    pub const ACCENT_SUBTLE: Color32 = Color32::from_rgb(0x24, 0x36, 0x58);
    /// Text drawn on top of [`ACCENT_FILL`].
    pub const TEXT_ON_ACCENT: Color32 = Color32::WHITE;

    pub const SUCCESS: Color32 = Color32::from_rgb(0x46, 0xC2, 0x8A);
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
    pub const SM: u8 = 3;
    pub const MD: u8 = 5;
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
    pub const TOOL_BAR_WIDTH: f32 = 44.0;
    pub const MENU_BAR_HEIGHT: f32 = 30.0;
    pub const STATUS_BAR_HEIGHT: f32 = 24.0;
    pub const PANEL_HEADER_HEIGHT: f32 = 28.0;
    pub const PANEL_COLUMN_DEFAULT: f32 = 280.0;
    pub const PANEL_COLUMN_MIN: f32 = 220.0;
    pub const PANEL_COLUMN_MAX: f32 = 480.0;
}

/// Typographic scale (points at 100% text size).
pub mod typography {
    pub const CAPTION: f32 = 11.0;
    pub const BODY: f32 = 12.5;
    pub const LABEL: f32 = 12.0;
    pub const HEADING: f32 = 15.0;
    pub const TITLE: f32 = 22.0;
    pub const DISPLAY: f32 = 30.0;
    pub const MONO: f32 = 12.0;
}

/// Canvas overlay styling (selection, handles, guides drawn over artwork).
pub mod canvas {
    use egui::Color32;

    /// Selection outlines and handle borders: saturated so they stand out on
    /// light and dark artwork alike (always drawn over a halo).
    pub const SELECTION: Color32 = Color32::from_rgb(0x1F, 0x6F, 0xFF);
    /// Contrasting halo drawn under every overlay line.
    pub const HALO: Color32 = Color32::from_rgba_premultiplied(230, 230, 230, 230);
    pub const HANDLE_FILL: Color32 = Color32::WHITE;
    /// Highlight behind selected characters while editing a text.
    pub const TEXT_SELECTION: Color32 = Color32::from_rgba_premultiplied(31, 111, 255, 90);
    /// Caret of the text being edited.
    pub const CARET: Color32 = Color32::from_rgb(0x1F, 0x6F, 0xFF);
    /// Placeholder of an image still loading.
    pub const IMAGE_PLACEHOLDER: Color32 = Color32::from_rgb(0xC8, 0xCA, 0xCF);
    pub const MARQUEE_FILL: Color32 = Color32::from_rgba_premultiplied(15, 55, 128, 40);
    /// Pasteboard around the artboard.
    pub const PASTEBOARD: Color32 = Color32::from_rgb(0x18, 0x19, 0x1C);
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
}
