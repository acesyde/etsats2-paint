//! What fills and strokes are painted with: a solid color or a linear or
//! radial gradient.
//!
//! A gradient's points are stored in frame units: (0, 0) is the frame's
//! local top-left corner and (1, 1) its bottom-right, before rotation. The
//! gradient therefore follows its object through every frame change.

use kurbo::{Affine, Point, Vec2};

use super::color::Rgba;
use super::object::Frame;

/// Largest number of stops a gradient may have.
pub const MAX_STOPS: usize = 16;
/// Smallest number of stops a gradient may have.
pub const MIN_STOPS: usize = 2;

/// Fill or stroke paint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Paint {
    Solid(Rgba),
    Gradient(Gradient),
}

impl From<Rgba> for Paint {
    fn from(c: Rgba) -> Self {
        Paint::Solid(c)
    }
}

impl Paint {
    /// The color of a solid paint.
    pub fn solid_color(&self) -> Option<Rgba> {
        match self {
            Paint::Solid(c) => Some(*c),
            Paint::Gradient(_) => None,
        }
    }

    pub fn gradient(&self) -> Option<&Gradient> {
        match self {
            Paint::Solid(_) => None,
            Paint::Gradient(g) => Some(g),
        }
    }

    pub fn gradient_mut(&mut self) -> Option<&mut Gradient> {
        match self {
            Paint::Solid(_) => None,
            Paint::Gradient(g) => Some(g),
        }
    }

    /// Whether anything of it can be seen (some alpha above zero).
    pub fn is_visible(&self) -> bool {
        match self {
            Paint::Solid(c) => c.a > 0,
            Paint::Gradient(g) => g.stops().iter().any(|s| s.color.a > 0),
        }
    }

    /// A representative color: the solid color, or the first stop's.
    pub fn first_color(&self) -> Rgba {
        match self {
            Paint::Solid(c) => *c,
            Paint::Gradient(g) => g.stops()[0].color,
        }
    }

    /// The kind of paint.
    pub fn kind(&self) -> PaintKind {
        match self {
            Paint::Solid(_) => PaintKind::Solid,
            Paint::Gradient(g) => match g.kind {
                GradientKind::Linear => PaintKind::Linear,
                GradientKind::Radial => PaintKind::Radial,
            },
        }
    }

    /// The same paint placed so that it stays where it is in the document
    /// when its object goes from frame `from` to frame `to`, after the
    /// document transform `transform` (identity when only the frame
    /// changes).
    pub fn remapped(&self, from: &Frame, transform: Affine, to: &Frame) -> Paint {
        match self {
            Paint::Solid(_) => *self,
            Paint::Gradient(g) => Paint::Gradient(g.remapped(from, transform, to)),
        }
    }
}

/// Kind of paint, as chosen in the Colors panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PaintKind {
    Solid,
    Linear,
    Radial,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GradientKind {
    /// Colors vary from `start` to `end`, constant across that direction.
    Linear,
    /// Colors form concentric ellipses around `start`.
    Radial,
}

/// A color at a location along a gradient.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorStop {
    /// 0.0 (start or center) ..= 1.0 (end or radius).
    pub offset: f32,
    pub color: Rgba,
}

impl ColorStop {
    pub fn new(offset: f32, color: Rgba) -> Self {
        Self { offset, color }
    }
}

/// A linear or radial gradient (see the module documentation for the
/// coordinates).
#[derive(Clone, Copy, Debug)]
pub struct Gradient {
    pub kind: GradientKind,
    /// Linear start, or radial center.
    pub start: Point,
    /// Linear end, or the end of the radial's main radius.
    pub end: Point,
    /// Radial only: the end of the second radius (perpendicular to the main
    /// one on the document until a transform skews it).
    pub minor: Point,
    stops: [ColorStop; MAX_STOPS],
    len: u8,
}

impl PartialEq for Gradient {
    /// Unused stop slots are ignored.
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.start == other.start
            && self.end == other.end
            && self.minor == other.minor
            && self.stops() == other.stops()
    }
}

/// Maps frame units to the document.
pub fn unit_to_document(frame: &Frame) -> Affine {
    let (w, h) = (frame.size.width, frame.size.height);
    frame.affine() * Affine::translate((-w / 2.0, -h / 2.0)) * Affine::scale_non_uniform(w, h)
}

fn lerp(a: u8, b: u8, t: f32) -> f32 {
    f32::from(a) + (f32::from(b) - f32::from(a)) * t
}

impl Gradient {
    /// A gradient of `kind` with its default placement (linear across the
    /// frame from left to right; radial centered, its ellipse fitting the
    /// frame) and the given stops (see [`Gradient::set_stops`]).
    pub fn new(kind: GradientKind, stops: &[ColorStop]) -> Self {
        let mut g = Self {
            kind,
            start: Point::new(0.0, 0.5),
            end: Point::new(1.0, 0.5),
            minor: Point::new(0.5, 1.0),
            stops: [ColorStop::new(0.0, Rgba::rgb(0, 0, 0)); MAX_STOPS],
            len: 0,
        };
        g.set_kind(kind);
        g.set_stops(stops);
        g
    }

    /// The default gradient made from a solid color: the color, fading to
    /// the same color fully transparent.
    pub fn from_color(kind: GradientKind, c: Rgba) -> Self {
        Self::new(
            kind,
            &[
                ColorStop::new(0.0, c),
                ColorStop::new(1.0, Rgba { a: 0, ..c }),
            ],
        )
    }

    /// Changes the kind, placing the gradient at the default position of
    /// that kind when it changes.
    pub fn set_kind(&mut self, kind: GradientKind) {
        if self.kind == kind && self.len > 0 {
            return;
        }
        self.kind = kind;
        match kind {
            GradientKind::Linear => {
                self.start = Point::new(0.0, 0.5);
                self.end = Point::new(1.0, 0.5);
                self.minor = Point::new(0.5, 1.0);
            }
            GradientKind::Radial => {
                self.start = Point::new(0.5, 0.5);
                self.end = Point::new(1.0, 0.5);
                self.minor = Point::new(0.5, 1.0);
            }
        }
    }

    /// The stops, sorted by offset.
    pub fn stops(&self) -> &[ColorStop] {
        &self.stops[..usize::from(self.len)]
    }

    /// Replaces the stops: offsets are clamped to 0..=1, the stops sorted
    /// (equal offsets keep their order), extra stops beyond [`MAX_STOPS`]
    /// dropped, and a single stop doubled. An empty slice is ignored.
    pub fn set_stops(&mut self, stops: &[ColorStop]) {
        if stops.is_empty() {
            return;
        }
        let mut list: Vec<ColorStop> = stops
            .iter()
            .take(MAX_STOPS)
            .map(|s| ColorStop::new(s.offset.clamp(0.0, 1.0), s.color))
            .collect();
        if list.len() < MIN_STOPS {
            let only = list[0];
            list = vec![
                ColorStop::new(0.0, only.color),
                ColorStop::new(1.0, only.color),
            ];
        }
        list.sort_by(|a, b| a.offset.total_cmp(&b.offset));
        self.len = list.len() as u8;
        self.stops[..list.len()].copy_from_slice(&list);
    }

    /// Color at `t` (0 at the first location, 1 at the last), interpolated
    /// channel by channel in sRGB with straight alpha; outside the stops the
    /// end stop's color.
    pub fn color_at(&self, t: f32) -> Rgba {
        let stops = self.stops();
        let first = stops[0];
        if t <= first.offset || t.is_nan() {
            return first.color;
        }
        for pair in stops.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if t < b.offset {
                let span = b.offset - a.offset;
                let f = if span <= f32::EPSILON {
                    1.0
                } else {
                    (t - a.offset) / span
                };
                let q = |v: f32| v.round().clamp(0.0, 255.0) as u8;
                return Rgba::with_alpha(
                    q(lerp(a.color.r, b.color.r, f)),
                    q(lerp(a.color.g, b.color.g, f)),
                    q(lerp(a.color.b, b.color.b, f)),
                    q(lerp(a.color.a, b.color.a, f)),
                );
            }
        }
        stops[stops.len() - 1].color
    }

    /// The color of the last stop (used where the gradient is degenerate).
    pub fn last_color(&self) -> Rgba {
        self.stops()[self.stops().len() - 1].color
    }

    /// Maps gradient space to the document: for linear gradients (0, 0) is
    /// the start and (1, 0) the end, with y perpendicular on the document;
    /// for radial gradients the unit circle is the gradient's outer
    /// ellipse. `None` when degenerate (no length, or a flat ellipse).
    pub fn to_document(&self, frame: &Frame) -> Option<Affine> {
        let d = unit_to_document(frame);
        let (s, e) = (d * self.start, d * self.end);
        let x = e - s;
        if x.hypot() < 1e-9 {
            return None;
        }
        let y = match self.kind {
            GradientKind::Linear => Vec2::new(-x.y, x.x),
            GradientKind::Radial => d * self.minor - s,
        };
        let det = x.cross(y);
        if det.abs() < 1e-12 * x.hypot2().max(1.0) {
            return None;
        }
        Some(Affine::new([x.x, x.y, y.x, y.y, s.x, s.y]))
    }

    /// Direction from start to end on the document, in degrees clockwise
    /// from the x axis (y points down).
    pub fn angle(&self, frame: &Frame) -> f64 {
        let d = unit_to_document(frame);
        let v = d * self.end - d * self.start;
        if v.hypot() < 1e-12 {
            0.0
        } else {
            super::object::normalize_degrees(v.y.atan2(v.x).to_degrees())
        }
    }

    /// Radial only: length of the second radius relative to the main one
    /// (1.0 is a circle), on the document.
    pub fn aspect(&self, frame: &Frame) -> f64 {
        let d = unit_to_document(frame);
        let main = (d * self.end - d * self.start).hypot();
        if main < 1e-12 {
            1.0
        } else {
            (d * self.minor - d * self.start).hypot() / main
        }
    }

    /// Sets start, end (document points) and the aspect ratio of the second
    /// radius, rebuilding it perpendicular on the document.
    fn place(&mut self, frame: &Frame, start: Point, end: Point, aspect: f64) {
        let d = unit_to_document(frame);
        let Some(inv) = invert(d) else {
            return;
        };
        let v = end - start;
        let minor = start + Vec2::new(-v.y, v.x) * aspect;
        self.start = inv * start;
        self.end = inv * end;
        self.minor = inv * minor;
    }

    /// Sets the document points of the start (center) and end (radius
    /// point), keeping the aspect ratio.
    pub fn set_points(&mut self, frame: &Frame, start: Point, end: Point) {
        let aspect = self.aspect(frame);
        self.place(frame, start, end, aspect);
    }

    /// Document points of the start (center), end and second radius end.
    pub fn document_points(&self, frame: &Frame) -> (Point, Point, Point) {
        let d = unit_to_document(frame);
        (d * self.start, d * self.end, d * self.minor)
    }

    /// Rotates the gradient to `deg` on the document around its midpoint
    /// (linear) or center (radial), keeping its length and aspect.
    pub fn set_angle(&mut self, frame: &Frame, deg: f64) {
        let (s, e, _) = self.document_points(frame);
        let aspect = self.aspect(frame);
        let len = (e - s).hypot();
        let dir = Vec2::from_angle(deg.to_radians()) * len;
        let (start, end) = match self.kind {
            GradientKind::Linear => {
                let mid = s.midpoint(e);
                (mid - dir / 2.0, mid + dir / 2.0)
            }
            GradientKind::Radial => (s, s + dir),
        };
        self.place(frame, start, end, aspect);
    }

    /// Radial only: sets the second radius to `aspect` × the main one,
    /// perpendicular on the document.
    pub fn set_aspect(&mut self, frame: &Frame, aspect: f64) {
        let (s, e, _) = self.document_points(frame);
        self.place(frame, s, e, aspect.clamp(0.01, 10.0));
    }

    /// Mirrors every stop's location.
    pub fn reverse(&mut self) {
        let mut stops: Vec<ColorStop> = self
            .stops()
            .iter()
            .rev()
            .map(|s| ColorStop::new(1.0 - s.offset, s.color))
            .collect();
        // Keep the order of stops sharing a location after mirroring.
        stops.sort_by(|a, b| a.offset.total_cmp(&b.offset));
        self.set_stops(&stops);
    }

    /// See [`Paint::remapped`].
    pub fn remapped(&self, from: &Frame, transform: Affine, to: &Frame) -> Gradient {
        let Some(inv) = invert(unit_to_document(to)) else {
            return *self;
        };
        let m = inv * transform * unit_to_document(from);
        let mut g = *self;
        g.start = m * self.start;
        g.end = m * self.end;
        g.minor = m * self.minor;
        if g.kind == GradientKind::Linear {
            // Linear gradients only use their start and end; keep the
            // unused point meaningful for a later switch to radial.
            let aspect = 1.0;
            let (s, e) = (unit_to_document(to) * g.start, unit_to_document(to) * g.end);
            g.place(to, s, e, aspect);
        }
        g
    }
}

fn invert(a: Affine) -> Option<Affine> {
    (a.determinant().abs() > 1e-18).then(|| a.inverse())
}

#[cfg(test)]
mod tests {
    use kurbo::Size;

    use super::*;

    const RED: Rgba = Rgba::rgb(255, 0, 0);
    const BLUE: Rgba = Rgba::rgb(0, 0, 255);

    fn red_blue(kind: GradientKind) -> Gradient {
        Gradient::new(kind, &[ColorStop::new(0.0, RED), ColorStop::new(1.0, BLUE)])
    }

    fn close(a: Point, b: Point) -> bool {
        (a - b).hypot() < 1e-6
    }

    #[test]
    fn stops_are_sorted_clamped_and_bounded() {
        let mut g = red_blue(GradientKind::Linear);
        g.set_stops(&[
            ColorStop::new(0.8, BLUE),
            ColorStop::new(-1.0, RED),
            ColorStop::new(0.5, Rgba::rgb(1, 1, 1)),
            ColorStop::new(0.5, Rgba::rgb(2, 2, 2)),
        ]);
        let offsets: Vec<f32> = g.stops().iter().map(|s| s.offset).collect();
        assert_eq!(offsets, [0.0, 0.5, 0.5, 0.8]);
        // Equal offsets keep their order.
        assert_eq!(g.stops()[1].color, Rgba::rgb(1, 1, 1));
        let many: Vec<ColorStop> = (0..20)
            .map(|i| ColorStop::new(i as f32 / 20.0, RED))
            .collect();
        g.set_stops(&many);
        assert_eq!(g.stops().len(), MAX_STOPS);
        g.set_stops(&[ColorStop::new(0.3, RED)]);
        assert_eq!(g.stops().len(), MIN_STOPS);
        g.set_stops(&[]);
        assert_eq!(g.stops().len(), MIN_STOPS);
    }

    #[test]
    fn interpolation_in_straight_srgb() {
        let g = red_blue(GradientKind::Linear);
        assert_eq!(g.color_at(0.5), Rgba::rgb(128, 0, 128));
        assert_eq!(g.color_at(-1.0), RED);
        assert_eq!(g.color_at(2.0), BLUE);
        let fade = Gradient::from_color(GradientKind::Linear, Rgba::rgb(255, 255, 255));
        let mid = fade.color_at(0.5);
        assert_eq!((mid.r, mid.g, mid.b), (255, 255, 255), "no darkening");
        assert_eq!(mid.a, 128);
    }

    #[test]
    fn hard_edge_at_equal_offsets() {
        let white = Rgba::rgb(255, 255, 255);
        let g = Gradient::new(
            GradientKind::Linear,
            &[
                ColorStop::new(0.0, white),
                ColorStop::new(0.5, white),
                ColorStop::new(0.5, RED),
                ColorStop::new(1.0, RED),
            ],
        );
        assert_eq!(g.color_at(0.499), white);
        assert_eq!(g.color_at(0.5), RED);
    }

    #[test]
    fn linear_follows_rotation_and_resize() {
        let g = red_blue(GradientKind::Linear);
        let frame = Frame::new(Point::new(200.0, 50.0), Size::new(400.0, 100.0), 0.0);
        let a = g.to_document(&frame).unwrap();
        assert!(close(a * Point::ORIGIN, Point::new(0.0, 50.0)));
        assert!(close(a * Point::new(1.0, 0.0), Point::new(400.0, 50.0)));
        // Rotated 90° and twice as wide: still spans the object top to bottom.
        let turned = Frame::new(Point::new(0.0, 0.0), Size::new(800.0, 100.0), 90.0);
        let a = g.to_document(&turned).unwrap();
        assert!(close(a * Point::ORIGIN, Point::new(0.0, -400.0)));
        assert!(close(a * Point::new(1.0, 0.0), Point::new(0.0, 400.0)));
        assert!((g.angle(&turned) - 90.0).abs() < 1e-9);
    }

    #[test]
    fn radial_default_fits_the_frame_and_aspect() {
        let g = red_blue(GradientKind::Radial);
        let frame = Frame::new(Point::new(0.0, 0.0), Size::new(400.0, 100.0), 0.0);
        let a = g.to_document(&frame).unwrap();
        assert!(close(a * Point::new(1.0, 0.0), Point::new(200.0, 0.0)));
        assert!(close(a * Point::new(0.0, 1.0), Point::new(0.0, 50.0)));
        // Elliptical glow: radius point 100 px right, aspect 50%.
        let square = Frame::new(Point::new(0.0, 0.0), Size::new(400.0, 400.0), 0.0);
        let mut g = red_blue(GradientKind::Radial);
        g.set_points(&square, Point::ORIGIN, Point::new(100.0, 0.0));
        g.set_aspect(&square, 0.5);
        let a = g.to_document(&square).unwrap();
        assert!(close(a * Point::new(1.0, 0.0), Point::new(100.0, 0.0)));
        assert!(close(a * Point::new(0.0, 1.0), Point::new(0.0, 50.0)));
        assert!((g.aspect(&square) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn degenerate_gradient_has_no_mapping() {
        let frame = Frame::new(Point::ORIGIN, Size::new(100.0, 100.0), 0.0);
        let mut g = red_blue(GradientKind::Linear);
        g.end = g.start;
        assert!(g.to_document(&frame).is_none());
        let mut r = red_blue(GradientKind::Radial);
        r.minor = r.start;
        assert!(r.to_document(&frame).is_none());
    }

    #[test]
    fn angle_rotates_around_the_midpoint() {
        let frame = Frame::new(Point::ORIGIN, Size::new(200.0, 200.0), 0.0);
        let mut g = red_blue(GradientKind::Linear);
        g.set_angle(&frame, 90.0);
        let (s, e, _) = g.document_points(&frame);
        assert!(close(s, Point::new(0.0, -100.0)) && close(e, Point::new(0.0, 100.0)));
    }

    #[test]
    fn remap_keeps_document_placement() {
        let g = red_blue(GradientKind::Radial);
        let from = Frame::new(Point::new(10.0, 10.0), Size::new(100.0, 50.0), 30.0);
        let to = Frame::new(Point::new(40.0, 0.0), Size::new(300.0, 80.0), 0.0);
        let r = g.remapped(&from, Affine::IDENTITY, &to);
        let (a, b) = (g.document_points(&from), r.document_points(&to));
        assert!(close(a.0, b.0) && close(a.1, b.1) && close(a.2, b.2));
        // With a transform: a mirror carries the gradient along.
        let mirror = Affine::scale_non_uniform(-1.0, 1.0);
        let r = g.remapped(&from, mirror, &to);
        assert!(close(r.document_points(&to).1, mirror * a.1));
    }

    #[test]
    fn reverse_mirrors_locations() {
        let mut g = Gradient::new(
            GradientKind::Linear,
            &[
                ColorStop::new(0.0, RED),
                ColorStop::new(0.25, BLUE),
                ColorStop::new(1.0, BLUE),
            ],
        );
        g.reverse();
        let offsets: Vec<f32> = g.stops().iter().map(|s| s.offset).collect();
        assert_eq!(offsets, [0.0, 0.75, 1.0]);
        assert_eq!(g.stops()[2].color, RED);
    }
}
