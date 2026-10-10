//! Drop shadows: an object's silhouette drawn offscreen, tinted and
//! blurred. tiny-skia has no blur, so a separable box blur run three times
//! stands in for a Gaussian.

use resvg::tiny_skia::{IntSize, Pixmap};
use tp_core::Asset;
use tp_core::document::{Object, Rgba};
use tp_core::kurbo::{Point, Rect};
use tp_text::FontLibrary;

use crate::render::{AssetCache, DrawCache, Geometry};

/// A rendered shadow, ready to draw just under its object.
pub struct ShadowLayer {
    /// The shadow's pixels, premultiplied, with the object's opacity in.
    pub pixmap: Pixmap,
    /// Document position of the pixmap's top-left corner.
    pub origin: Point,
    /// Pixels per document unit.
    pub scale: f64,
}

/// The shadow of `object` drawn at `opacity` (its own and its groups'),
/// `scale` pixels per document unit, or `None` when it has none or it
/// shows nothing. With `clip` (document space), the layer is cut to it, as
/// the texture's edges cut it; the blur still sees what lies beyond.
///
/// `asset` is the asset of an image object. The layer doesn't depend on
/// where the object is beyond its offset from `origin`, so a caller may
/// render the object moved to the origin and place the layer itself.
pub fn shadow_layer(
    object: &Object,
    opacity: f32,
    scale: f64,
    clip: Option<Rect>,
    asset: Option<&Asset>,
    fonts: &mut FontLibrary,
    cache: &mut DrawCache,
) -> Option<ShadowLayer> {
    object.shadow.filter(|_| object.takes_shadow())?;
    let geometry = Geometry::of(object, asset, fonts, &mut cache.glyphs)?;
    layer_of(&geometry, object, opacity, scale, clip, &mut cache.assets)
}

/// [`shadow_layer`] from the object's geometry.
pub(crate) fn layer_of(
    geometry: &Geometry,
    object: &Object,
    opacity: f32,
    scale: f64,
    clip: Option<Rect>,
    assets: &mut AssetCache,
) -> Option<ShadowLayer> {
    let shadow = object.shadow.filter(|_| object.takes_shadow())?;
    let strength = shadow.opacity * f32::from(shadow.color.a) / 255.0 * opacity;
    if strength <= 0.0 || !scale.is_finite() || scale <= 0.0 {
        return None;
    }
    let radii = box_radii(shadow.blur * scale / 2.0);
    let spread = radii.iter().sum::<usize>() as f64;
    let px = |r: Rect| Rect::new(r.x0 * scale, r.y0 * scale, r.x1 * scale, r.y1 * scale);
    // Pixels drawn: the silhouette grown by what the blur spreads, and no
    // more than the blur can bring into the clip.
    let silhouette = px(geometry.drawn_bounds(object) + shadow.offset).inflate(1.0, 1.0);
    let mut area = silhouette.inflate(spread, spread);
    let clip = clip.map(|c| px(c).expand());
    if let Some(c) = clip {
        area = area.intersect(c.inflate(spread, spread));
    }
    let area = area.expand();
    if area.width() < 1.0 || area.height() < 1.0 {
        return None;
    }
    let (w, h) = (area.width() as u32, area.height() as u32);
    let mut layer = Pixmap::new(w, h)?;
    // The object moved by the offset, with the area's corner at (0, 0).
    let shift = shadow.offset - area.origin().to_vec2() / scale;
    let mut moved = object.clone();
    moved.frame.center += shift;
    geometry
        .translated(shift)
        .draw(&mut layer, &moved, 1.0, scale, assets);
    let mut alpha: Vec<u16> = layer
        .pixels()
        .iter()
        .map(|p| u16::from(p.alpha()) * 257)
        .collect();
    drop(layer);
    blur(&mut alpha, w as usize, h as usize, radii);
    // Cut to the clip.
    let kept = match clip {
        Some(c) => area.intersect(c),
        None => area,
    };
    if kept.width() < 1.0 || kept.height() < 1.0 {
        return None;
    }
    let (x0, y0) = ((kept.x0 - area.x0) as usize, (kept.y0 - area.y0) as usize);
    let (kw, kh) = (kept.width() as usize, kept.height() as usize);
    let mut data = Vec::with_capacity(kw * kh * 4);
    let tint = Tint::new(shadow.color, strength);
    for row in alpha.chunks_exact(w as usize).skip(y0).take(kh) {
        for &a in &row[x0..x0 + kw] {
            data.extend_from_slice(&tint.pixel(a));
        }
    }
    let pixmap = Pixmap::from_vec(data, IntSize::from_wh(kw as u32, kh as u32)?)?;
    Some(ShadowLayer {
        pixmap,
        origin: Point::new(kept.x0 / scale, kept.y0 / scale),
        scale,
    })
}

/// The shadow color at a coverage, premultiplied.
struct Tint {
    color: Rgba,
    /// Alpha per unit of 16-bit coverage, in 1/65536 of a level.
    gain: u64,
}

impl Tint {
    fn new(color: Rgba, strength: f32) -> Self {
        let gain = (f64::from(strength.min(1.0)) * 255.0 / 65535.0 * 65536.0).round() as u64;
        Self { color, gain }
    }

    fn pixel(&self, coverage: u16) -> [u8; 4] {
        let a = ((u64::from(coverage) * self.gain + (1 << 15)) >> 16).min(255) as u16;
        let c = |v: u8| ((u16::from(v) * a + 127) / 255) as u8;
        [c(self.color.r), c(self.color.g), c(self.color.b), a as u8]
    }
}

/// Radii of the three box blurs that approximate a Gaussian of standard
/// deviation `sigma` (pixels).
pub(crate) fn box_radii(sigma: f64) -> [usize; 3] {
    let sigma = if sigma.is_finite() {
        sigma.max(0.0)
    } else {
        0.0
    };
    let n = 3.0;
    let ideal = (12.0 * sigma * sigma / n + 1.0).sqrt();
    let mut lower = ideal.floor() as i64;
    if lower % 2 == 0 {
        lower -= 1;
    }
    let lower = lower.max(1);
    let wl = lower as f64;
    // How many passes use the lower width so the variances add up.
    let m = ((12.0 * sigma * sigma - n * wl * wl - 4.0 * n * wl - 3.0 * n) / (-4.0 * wl - 4.0))
        .round() as i64;
    [0, 1, 2].map(|i| {
        let width = if i < m { lower } else { lower + 2 };
        (width as usize - 1) / 2
    })
}

/// Blurs a `w` × `h` coverage buffer in place: one box blur per radius,
/// horizontally then vertically. Outside the buffer is empty.
pub(crate) fn blur(data: &mut [u16], w: usize, h: usize, radii: [usize; 3]) {
    if radii.iter().all(|r| *r == 0) || w == 0 || h == 0 {
        return;
    }
    let mut tmp = vec![0u16; data.len()];
    for r in radii.into_iter().filter(|r| *r > 0) {
        box_rows(data, &mut tmp, w, r);
        data.copy_from_slice(&tmp);
    }
    for r in radii.into_iter().filter(|r| *r > 0) {
        box_columns(data, &mut tmp, w, h, r);
        data.copy_from_slice(&tmp);
    }
}

/// The mean of `2r + 1` values, rounded, from their sum.
struct Mean {
    mul: u64,
}

impl Mean {
    fn new(r: usize) -> Self {
        let d = (2 * r + 1) as u64;
        Self {
            mul: ((1u64 << 32) + d / 2) / d,
        }
    }

    fn of(&self, sum: u32) -> u16 {
        ((u64::from(sum) * self.mul + (1 << 31)) >> 32).min(65535) as u16
    }
}

/// Box blur of radius `r` along each row of `src` into `dst`.
fn box_rows(src: &[u16], dst: &mut [u16], w: usize, r: usize) {
    let mean = Mean::new(r);
    for (row, out) in src.chunks_exact(w).zip(dst.chunks_exact_mut(w)) {
        // Sum of row[x - r ..= x + r], starting at x = 0.
        let mut sum: u32 = row[..(r + 1).min(w)].iter().map(|v| u32::from(*v)).sum();
        for x in 0..w {
            out[x] = mean.of(sum);
            if x + r + 1 < w {
                sum += u32::from(row[x + r + 1]);
            }
            if x >= r {
                sum -= u32::from(row[x - r]);
            }
        }
    }
}

/// Box blur of radius `r` along each column of `src` into `dst`, a row at
/// a time so memory is read in order.
fn box_columns(src: &[u16], dst: &mut [u16], w: usize, h: usize, r: usize) {
    let mean = Mean::new(r);
    let row = |y: usize| &src[y * w..(y + 1) * w];
    let mut sums = vec![0u32; w];
    for y in 0..(r + 1).min(h) {
        for (s, v) in sums.iter_mut().zip(row(y)) {
            *s += u32::from(*v);
        }
    }
    for y in 0..h {
        for (o, s) in dst[y * w..(y + 1) * w].iter_mut().zip(&sums) {
            *o = mean.of(*s);
        }
        if y + r + 1 < h {
            for (s, v) in sums.iter_mut().zip(row(y + r + 1)) {
                *s += u32::from(*v);
            }
        }
        if y >= r {
            for (s, v) in sums.iter_mut().zip(row(y - r)) {
                *s -= u32::from(*v);
            }
        }
    }
}

#[cfg(test)]
mod tests;
