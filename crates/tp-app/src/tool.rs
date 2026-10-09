//! Editing tools of the Workshop's tool rail.

use tp_ui::icons;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Tool {
    #[default]
    Select,
    DirectSelect,
    Move,
    Rectangle,
    Ellipse,
    Polygon,
    Pen,
    Line,
    Text,
    Image,
    Eyedropper,
    Gradient,
    Zoom,
    Hand,
}

impl Tool {
    /// Tool rail order.
    pub const ALL: [Self; 14] = [
        Self::Select,
        Self::DirectSelect,
        Self::Move,
        Self::Rectangle,
        Self::Ellipse,
        Self::Polygon,
        Self::Pen,
        Self::Line,
        Self::Text,
        Self::Image,
        Self::Eyedropper,
        Self::Gradient,
        Self::Zoom,
        Self::Hand,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Select => "tool-selection",
            Self::DirectSelect => "tool-direct-selection",
            Self::Move => "tool-move",
            Self::Rectangle => "tool-rectangle",
            Self::Ellipse => "tool-ellipse",
            Self::Polygon => "tool-polygon",
            Self::Pen => "tool-pen",
            Self::Line => "tool-line",
            Self::Text => "tool-text",
            Self::Image => "tool-image",
            Self::Eyedropper => "tool-eyedropper",
            Self::Gradient => "tool-gradient",
            Self::Zoom => "tool-zoom",
            Self::Hand => "tool-hand",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Select => icons::SELECT,
            Self::DirectSelect => icons::DIRECT_SELECT,
            Self::Move => icons::MOVE,
            Self::Rectangle => icons::RECTANGLE,
            Self::Ellipse => icons::ELLIPSE,
            Self::Polygon => icons::POLYGON,
            Self::Pen => icons::PEN,
            Self::Line => icons::LINE,
            Self::Text => icons::TEXT,
            Self::Image => icons::IMAGE,
            Self::Eyedropper => icons::EYEDROPPER,
            Self::Gradient => icons::GRADIENT_TOOL,
            Self::Zoom => icons::ZOOM,
            Self::Hand => icons::HAND,
        }
    }
}
