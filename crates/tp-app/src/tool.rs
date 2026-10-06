//! Editing tools of the left tool bar.

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
    Zoom,
    Hand,
}

impl Tool {
    /// Tool bar order.
    pub const ALL: [Self; 13] = [
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
        Self::Zoom,
        Self::Hand,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Select => "Selection",
            Self::DirectSelect => "Direct Selection",
            Self::Move => "Move",
            Self::Rectangle => "Rectangle",
            Self::Ellipse => "Ellipse",
            Self::Polygon => "Polygon",
            Self::Pen => "Pen",
            Self::Line => "Line",
            Self::Text => "Text",
            Self::Image => "Image",
            Self::Eyedropper => "Eyedropper",
            Self::Zoom => "Zoom",
            Self::Hand => "Hand",
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
            Self::Zoom => icons::ZOOM,
            Self::Hand => icons::HAND,
        }
    }
}
