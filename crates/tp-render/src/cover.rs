//! Pictures that cover a fixed size (the mod's icon and Mod Manager
//! image): a surface or an image scaled to cover it, centered and cropped.

use resvg::tiny_skia::{IntRect, Pixmap};
use tp_core::Project;
use tp_core::document::Rgba;
use tp_text::FontLibrary;

use crate::render::{RenderOptions, pixmap_from_rgba, render};

/// The center `width` × `height` of `pixmap`.
fn crop_center(pixmap: &Pixmap, width: u32, height: u32) -> Pixmap {
    let x = (pixmap.width().saturating_sub(width) / 2) as i32;
    let y = (pixmap.height().saturating_sub(height) / 2) as i32;
    let rect = IntRect::from_xywh(x, y, width, height).expect("non-zero size");
    pixmap.clone_rect(rect).expect("inside the pixmap")
}

/// Surface `surface` of `project` (its artwork, never its template) scaled
/// to cover `width` × `height`, centered, over `background`.
pub fn render_cover(
    project: &Project,
    surface: usize,
    width: u32,
    height: u32,
    background: Option<Rgba>,
    fonts: &mut FontLibrary,
) -> Pixmap {
    let (width, height) = (width.max(1), height.max(1));
    let options = RenderOptions {
        size: width.max(height),
        background,
    };
    let full = render(project, surface, options, fonts, &mut |_, _| true).expect("never cancelled");
    crop_center(&full, width, height)
}

/// `image` scaled to cover `width` × `height` (Lanczos3), centered and
/// cropped.
pub fn cover_image(image: &image::RgbaImage, width: u32, height: u32) -> Pixmap {
    let (width, height) = (width.max(1), height.max(1));
    let (w, h) = (
        f64::from(image.width().max(1)),
        f64::from(image.height().max(1)),
    );
    let scale = (f64::from(width) / w).max(f64::from(height) / h);
    let rw = ((w * scale).round() as u32).max(width);
    let rh = ((h * scale).round() as u32).max(height);
    let resized = image::imageops::resize(image, rw, rh, image::imageops::FilterType::Lanczos3);
    let x = (rw - width) / 2;
    let y = (rh - height) / 2;
    let cropped = image::imageops::crop_imm(&resized, x, y, width, height).to_image();
    pixmap_from_rgba(&cropped).expect("non-zero size")
}
