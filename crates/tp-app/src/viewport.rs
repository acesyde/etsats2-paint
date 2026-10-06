//! Canvas view: which part of the document is visible and at what zoom.

use egui::{Pos2, Rect, Vec2};
use tp_core::kurbo::{self, Point};

/// Smallest zoom (2%).
pub const MIN_ZOOM: f64 = 0.02;
/// Largest zoom (6400%).
pub const MAX_ZOOM: f64 = 64.0;
/// Space kept around the artboard when fitting, in points.
pub const FIT_MARGIN: f32 = 40.0;
/// Extra room above the artboard for its label, in points.
pub const LABEL_SPACE: f32 = 18.0;

/// Zoom In / Zoom Out steps.
pub const ZOOM_PRESETS: [f64; 22] = [
    0.02, 0.03, 0.04, 0.06, 0.08, 0.12, 0.16, 0.25, 0.333, 0.5, 0.667, 1.0, 1.5, 2.0, 3.0, 4.0,
    6.0, 8.0, 12.0, 16.0, 32.0, 64.0,
];

/// Visible region of the document.
///
/// `zoom` is in physical screen pixels per texture pixel, so 1.0 is 100%
/// whatever the display scale factor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    /// Document point shown at the center of the canvas area.
    pub center: Point,
    pub zoom: f64,
    /// True after opening or Fit to Screen, until the user navigates.
    pub fitted: bool,
}

/// Affine map from document to screen points (`screen = doc * scale + offset`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenMap {
    pub scale: f32,
    pub offset: Vec2,
}

impl ScreenMap {
    pub fn to_screen(&self, p: Point) -> Pos2 {
        Pos2::new(p.x as f32 * self.scale, p.y as f32 * self.scale) + self.offset
    }

    pub fn to_doc(&self, p: Pos2) -> Point {
        let v = (p - self.offset) / self.scale;
        Point::new(f64::from(v.x), f64::from(v.y))
    }

    /// Converts a length in screen points to texture pixels.
    pub fn doc_len(&self, points: f32) -> f64 {
        f64::from(points / self.scale)
    }
}

impl Viewport {
    /// View showing the whole surface of side `size` centered in `canvas`.
    pub fn fit(size: f64, canvas: Rect, pixels_per_point: f32) -> Self {
        let width = (canvas.width() - 2.0 * FIT_MARGIN).max(1.0);
        let height = (canvas.height() - 2.0 * FIT_MARGIN - LABEL_SPACE).max(1.0);
        let points_per_px = f64::from(width.min(height)) / size.max(1.0);
        let zoom = (points_per_px * f64::from(pixels_per_point)).clamp(MIN_ZOOM, MAX_ZOOM);
        // Shift down by half the label space so the label fits above.
        let shift = f64::from(LABEL_SPACE / 2.0) * f64::from(pixels_per_point) / zoom;
        Self {
            center: Point::new(size / 2.0, size / 2.0 - shift),
            zoom,
            fitted: true,
        }
    }

    /// Screen points per texture pixel.
    pub fn scale(&self, pixels_per_point: f32) -> f64 {
        self.zoom / f64::from(pixels_per_point)
    }

    pub fn map(&self, canvas: Rect, pixels_per_point: f32) -> ScreenMap {
        let scale = self.scale(pixels_per_point) as f32;
        let c = canvas.center();
        ScreenMap {
            scale,
            offset: Vec2::new(
                c.x - self.center.x as f32 * scale,
                c.y - self.center.y as f32 * scale,
            ),
        }
    }

    /// Zooms to `zoom`, keeping the document point under `anchor` fixed.
    pub fn zoom_at(&mut self, canvas: Rect, ppp: f32, anchor: Pos2, zoom: f64) {
        let doc = self.map(canvas, ppp).to_doc(anchor);
        self.zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        let scale = self.scale(ppp);
        let from_center = anchor - canvas.center();
        self.center = Point::new(
            doc.x - f64::from(from_center.x) / scale,
            doc.y - f64::from(from_center.y) / scale,
        );
        self.fitted = false;
    }

    /// Multiplies the zoom by `factor` around `anchor`.
    pub fn zoom_by(&mut self, canvas: Rect, ppp: f32, anchor: Pos2, factor: f64) {
        self.zoom_at(canvas, ppp, anchor, self.zoom * factor);
    }

    /// Moves the content by `delta` screen points.
    pub fn pan(&mut self, ppp: f32, delta: Vec2) {
        let scale = self.scale(ppp);
        self.center.x -= f64::from(delta.x) / scale;
        self.center.y -= f64::from(delta.y) / scale;
        self.fitted = false;
    }

    /// Next preset above (`direction` > 0) or below the current zoom.
    pub fn step_zoom(zoom: f64, direction: i32) -> f64 {
        if direction > 0 {
            ZOOM_PRESETS
                .iter()
                .copied()
                .find(|z| *z > zoom * 1.001)
                .unwrap_or(MAX_ZOOM)
        } else {
            ZOOM_PRESETS
                .iter()
                .rev()
                .copied()
                .find(|z| *z < zoom / 1.001)
                .unwrap_or(MIN_ZOOM)
        }
    }

    /// Zooms so that `rect` (document) fills the canvas, centered.
    pub fn zoom_to_rect(&mut self, canvas: Rect, ppp: f32, rect: kurbo::Rect) {
        let rect = rect.abs();
        if rect.width() < 1e-6 || rect.height() < 1e-6 {
            return;
        }
        let sx = f64::from(canvas.width()) / rect.width();
        let sy = f64::from(canvas.height()) / rect.height();
        self.zoom = (sx.min(sy) * f64::from(ppp)).clamp(MIN_ZOOM, MAX_ZOOM);
        self.center = rect.center();
        self.fitted = false;
    }

    /// Sets the zoom keeping the current view center.
    pub fn set_zoom_centered(&mut self, zoom: f64) {
        self.zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        self.fitted = false;
    }
}

#[cfg(test)]
mod tests {
    use egui::pos2;

    use super::*;

    fn canvas() -> Rect {
        Rect::from_min_size(pos2(100.0, 50.0), Vec2::new(1000.0, 800.0))
    }

    #[test]
    fn fit_shows_whole_artboard() {
        for ppp in [1.0, 2.0] {
            let v = Viewport::fit(4096.0, canvas(), ppp);
            let m = v.map(canvas(), ppp);
            let a = m.to_screen(Point::ORIGIN);
            let b = m.to_screen(Point::new(4096.0, 4096.0));
            assert!(canvas().contains(a) && canvas().contains(b), "{a:?} {b:?}");
            assert!((b.x - a.x - (b.y - a.y)).abs() < 0.01);
            assert!(((a.x + b.x) / 2.0 - canvas().center().x).abs() < 0.5);
        }
    }

    #[test]
    fn zoom_keeps_point_under_pointer() {
        let mut v = Viewport::fit(4096.0, canvas(), 1.0);
        let anchor = pos2(300.0, 200.0);
        let before = v.map(canvas(), 1.0).to_doc(anchor);
        v.zoom_by(canvas(), 1.0, anchor, 3.0);
        let after = v.map(canvas(), 1.0).to_doc(anchor);
        assert!(before.distance(after) < 1e-6);
        assert!(!v.fitted);
    }

    #[test]
    fn zoom_is_clamped() {
        let mut v = Viewport::fit(4096.0, canvas(), 1.0);
        v.zoom_by(canvas(), 1.0, canvas().center(), 1e9);
        assert_eq!(v.zoom, MAX_ZOOM);
        v.zoom_by(canvas(), 1.0, canvas().center(), 1e-9);
        assert_eq!(v.zoom, MIN_ZOOM);
    }

    #[test]
    fn presets_step_up_and_down() {
        assert_eq!(Viewport::step_zoom(1.0, 1), 1.5);
        assert_eq!(Viewport::step_zoom(1.0, -1), 0.667);
        assert_eq!(Viewport::step_zoom(0.9, 1), 1.0);
        assert_eq!(Viewport::step_zoom(64.0, 1), 64.0);
    }

    #[test]
    fn hundred_percent_is_one_physical_pixel() {
        let mut v = Viewport::fit(4096.0, canvas(), 2.0);
        v.set_zoom_centered(1.0);
        let m = v.map(canvas(), 2.0);
        let d = m.to_screen(Point::new(2.0, 0.0)).x - m.to_screen(Point::ORIGIN).x;
        // Two texture pixels = two physical pixels = one point at 2x.
        assert!((d - 1.0).abs() < 1e-5);
    }

    #[test]
    fn resize_after_navigation_keeps_center_and_zoom() {
        let mut v = Viewport::fit(4096.0, canvas(), 1.0);
        v.set_zoom_centered(2.0);
        let center_doc = v.map(canvas(), 1.0).to_doc(canvas().center());
        let bigger = Rect::from_min_size(pos2(0.0, 0.0), Vec2::new(1600.0, 900.0));
        let after = v.map(bigger, 1.0).to_doc(bigger.center());
        assert!(center_doc.distance(after) < 1e-6);
        assert_eq!(v.zoom, 2.0);
    }

    #[test]
    fn pan_moves_content_with_delta() {
        let mut v = Viewport::fit(4096.0, canvas(), 1.0);
        let p = Point::new(100.0, 100.0);
        let before = v.map(canvas(), 1.0).to_screen(p);
        v.pan(1.0, Vec2::new(30.0, -10.0));
        let after = v.map(canvas(), 1.0).to_screen(p);
        assert!((after - before - Vec2::new(30.0, -10.0)).length() < 1e-3);
    }

    #[test]
    fn zoom_to_rect_fits_rect() {
        let mut v = Viewport::fit(4096.0, canvas(), 1.0);
        v.zoom_to_rect(canvas(), 1.0, kurbo::Rect::new(0.0, 0.0, 500.0, 400.0));
        assert!((v.zoom - 2.0).abs() < 1e-9);
        assert_eq!(v.center, Point::new(250.0, 200.0));
    }
}
