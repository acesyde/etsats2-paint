//! Tool calls, parsed and checked, as the app runs them. Colors that name a
//! palette swatch and the ids of styles and symbols are resolved by the app,
//! which holds the project.

use tp_core::document::{BooleanOp, Edge, GradientKind, ObjectId, Rgba, StrokeAlign, TextAlign};
use tp_core::kurbo::BezPath;

/// A tool call. Few and short-lived, so its size doesn't matter.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum Call {
    GetProject,
    ListObjects {
        texture: String,
    },
    GetObject {
        texture: String,
        id: ObjectId,
    },
    GetTemplate {
        texture: String,
        max_size: u32,
    },
    RenderTexture {
        texture: String,
        max_size: u32,
        with_template: bool,
    },
    /// Adds a palette swatch (the palette is the project's, not a
    /// texture's).
    AddSwatch {
        name: String,
        color: Rgba,
    },
    /// A change to one texture.
    Write {
        texture: String,
        op: WriteOp,
    },
}

impl Call {
    /// Whether the call changes the document.
    pub fn is_write(&self) -> bool {
        matches!(self, Self::AddSwatch { .. } | Self::Write { .. })
    }
}

/// A color: hex, or a palette swatch by name or id (the color is linked to
/// it).
#[derive(Clone, Debug, PartialEq)]
pub enum ColorRef {
    Hex(Rgba),
    Swatch(String),
}

/// A fill.
#[derive(Clone, Debug, PartialEq)]
pub enum PaintSpec {
    None,
    Color(ColorRef),
    Gradient {
        kind: GradientKind,
        /// Degrees, 0 = left to right, clockwise (y down).
        angle: f64,
        /// Offsets 0..=1 and colors, two at least.
        stops: Vec<(f32, ColorRef)>,
    },
}

/// An outline stroke.
#[derive(Clone, Debug, PartialEq)]
pub struct StrokeSpec {
    pub color: ColorRef,
    /// Texture pixels.
    pub width: f64,
    pub align: StrokeAlign,
}

/// A drop shadow.
#[derive(Clone, Debug, PartialEq)]
pub struct ShadowSpec {
    pub color: ColorRef,
    pub opacity: f32,
    pub offset_x: f64,
    pub offset_y: f64,
    pub blur: f64,
}

/// What a call sets of an object's look; `None` leaves it as it is.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LookSpec {
    pub fill: Option<PaintSpec>,
    /// `Some(None)` removes the stroke.
    pub stroke: Option<Option<StrokeSpec>>,
    pub opacity: Option<f32>,
    /// `Some(None)` removes the shadow.
    pub shadow: Option<Option<ShadowSpec>>,
    pub name: Option<String>,
}

impl LookSpec {
    /// Whether it changes the fill, stroke, opacity or shadow (which
    /// detaches an object from its style).
    pub fn changes_look(&self) -> bool {
        self.fill.is_some()
            || self.stroke.is_some()
            || self.opacity.is_some()
            || self.shadow.is_some()
    }
}

/// An unrotated box in texture pixels and a rotation in degrees around its
/// center.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxSpec {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
}

/// What update_object changes of an object's box.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoxChange {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub rotation: Option<f64>,
}

impl BoxChange {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// The geometry of add_shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShapeSpec {
    Rectangle { corner_radius: f64 },
    Ellipse,
    Polygon { sides: u8 },
    Star { points: u8, inner_ratio: f64 },
}

/// An object's place in the layers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderMove {
    Forward,
    Backward,
    Front,
    Back,
}

/// What align lines objects up with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignTarget {
    /// The bounds of the objects together (the texture for one object).
    Selection,
    Texture,
}

/// An arrange action.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ArrangeAction {
    Order(OrderMove),
    Group,
    Ungroup,
    Align { edge: Edge, to: AlignTarget },
    Combine(BooleanOp),
}

/// A change to one texture.
#[derive(Clone, Debug, PartialEq)]
pub enum WriteOp {
    AddShape {
        shape: ShapeSpec,
        frame: BoxSpec,
        look: LookSpec,
        parent: Option<ObjectId>,
    },
    AddPath {
        /// Texture pixels.
        path: BezPath,
        /// Subpaths combine with the even-odd rule (else non-zero).
        even_odd: bool,
        look: LookSpec,
        parent: Option<ObjectId>,
    },
    AddText {
        content: String,
        family: Option<String>,
        weight: Option<u16>,
        italic: bool,
        size: f64,
        align: TextAlign,
        /// Top-left of the text's box.
        x: f64,
        y: f64,
        rotation: f64,
        look: LookSpec,
        parent: Option<ObjectId>,
    },
    ImportImage {
        /// The file name (tells the format of base64 data).
        name: String,
        bytes: Vec<u8>,
        frame: Option<BoxSpec>,
        parent: Option<ObjectId>,
    },
    UpdateObject {
        id: ObjectId,
        frame: BoxChange,
        look: LookSpec,
        visible: Option<bool>,
        locked: Option<bool>,
    },
    Arrange {
        ids: Vec<ObjectId>,
        action: ArrangeAction,
    },
    Delete {
        ids: Vec<ObjectId>,
    },
    ApplyStyle {
        /// Name or id.
        style: String,
        ids: Vec<ObjectId>,
    },
    PlaceSymbol {
        /// Name or id.
        symbol: String,
        frame: BoxSpec,
    },
}
