//! Icon glyphs used by the application, named by meaning rather than shape so
//! a glyph can be swapped without touching call sites.

use egui::{FontFamily, FontId, RichText};
use egui_phosphor::regular as ph;

/// Font family containing only the icon font (plus emoji fallbacks).
///
/// Icons must be drawn with this family: the UI typeface also defines glyphs
/// in the Private Use Area and would otherwise shadow the icon codepoints.
pub const FAMILY: &str = "icons";

pub fn family() -> FontFamily {
    FontFamily::Name(FAMILY.into())
}

/// Icon font at the given size.
pub fn font(size: f32) -> FontId {
    FontId::new(size, family())
}

/// Rich text for a single icon glyph.
pub fn rich(icon: &str) -> RichText {
    RichText::new(icon).family(family())
}

pub const SELECT: &str = ph::NAVIGATION_ARROW;
pub const DIRECT_SELECT: &str = ph::CURSOR;
pub const MOVE: &str = ph::ARROWS_OUT_CARDINAL;
pub const RECTANGLE: &str = ph::SQUARE;
pub const ELLIPSE: &str = ph::CIRCLE;
pub const POLYGON: &str = ph::POLYGON;
pub const PEN: &str = ph::PEN_NIB;
pub const LINE: &str = ph::LINE_SEGMENT;
pub const TEXT: &str = ph::TEXT_T;
pub const IMAGE: &str = ph::IMAGE;
pub const EYEDROPPER: &str = ph::EYEDROPPER;
pub const ZOOM: &str = ph::MAGNIFYING_GLASS;
pub const HAND: &str = ph::HAND;

pub const NEW_PROJECT: &str = ph::PLUS;
pub const OPEN_PROJECT: &str = ph::FOLDER_OPEN;
pub const SAVE: &str = ph::FLOPPY_DISK;
pub const EXPORT: &str = ph::EXPORT;
pub const RECENT: &str = ph::CLOCK_COUNTER_CLOCKWISE;
pub const FILE: &str = ph::FILE;
pub const FILE_MISSING: &str = ph::FILE_DASHED;
pub const CLOSE: &str = ph::X;
pub const SETTINGS: &str = ph::GEAR;
pub const RESET: &str = ph::ARROW_COUNTER_CLOCKWISE;
pub const ROTATE: &str = ph::ARROW_CLOCKWISE;
pub const REMOVE: &str = ph::TRASH;
pub const MORE: &str = ph::DOTS_THREE_VERTICAL;

pub const EXPANDED: &str = ph::CARET_DOWN;
pub const COLLAPSED: &str = ph::CARET_RIGHT;

pub const SAVED: &str = ph::CHECK_CIRCLE;
pub const CHECK: &str = ph::CHECK;
pub const UNSAVED: &str = ph::CIRCLE_DASHED;
pub const WARNING: &str = ph::WARNING;
pub const ALIGN_LEFT: &str = ph::TEXT_ALIGN_LEFT;
pub const ALIGN_CENTER: &str = ph::TEXT_ALIGN_CENTER;
pub const ALIGN_RIGHT: &str = ph::TEXT_ALIGN_RIGHT;
pub const ITALIC: &str = ph::TEXT_ITALIC;
pub const SEARCH: &str = ph::MAGNIFYING_GLASS;
pub const RENAME: &str = ph::PENCIL_SIMPLE;

pub const PROPERTIES: &str = ph::SLIDERS;
pub const LAYERS: &str = ph::STACK;
pub const COLORS: &str = ph::PALETTE;
pub const STROKE: &str = ph::CIRCLE_HALF;
pub const TRANSFORM: &str = ph::BOUNDING_BOX;
pub const ASSETS: &str = ph::IMAGES;
pub const VEHICLE: &str = ph::TRUCK;
pub const GROUP: &str = ph::FOLDER_SIMPLE;
pub const UNGROUP: &str = ph::FOLDER_SIMPLE_DASHED;
pub const NEW_LAYER: &str = ph::STACK_PLUS;
pub const VISIBLE: &str = ph::EYE;
pub const HIDDEN: &str = ph::EYE_SLASH;
pub const LOCKED: &str = ph::LOCK_SIMPLE;
pub const UNLOCKED: &str = ph::LOCK_SIMPLE_OPEN;
pub const LINKED: &str = ph::LINK_SIMPLE;
pub const UNLINKED: &str = ph::LINK_BREAK;
pub const ADD: &str = ph::PLUS;
pub const PREVIEW_3D: &str = ph::CUBE;

pub const VIEW_2D: &str = ph::SQUARE_HALF;
pub const VIEW_3D: &str = ph::CUBE;
pub const VIEW_SPLIT: &str = ph::COLUMNS;
