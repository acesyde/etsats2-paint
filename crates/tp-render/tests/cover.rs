//! Pictures covering a fixed size: the mod's icon and Mod Manager image.

use tp_core::document::{Frame, Object, ObjectId, Rgba, ShapeKind};
use tp_core::kurbo::{Point, Size};
use tp_core::{Project, TextureResolution};
use tp_render::{cover_image, encode_jpeg, render_cover, to_rgba};
use tp_text::FontLibrary;

const WHITE: Rgba = Rgba::rgb(255, 255, 255);

fn rect(center: (f64, f64), size: (f64, f64), fill: Rgba) -> Object {
    let mut o = Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(
            Point::new(center.0, center.1),
            Size::new(size.0, size.1),
            0.0,
        ),
    );
    o.fill = fill.into();
    o
}

#[test]
fn a_surface_covered_keeps_its_center() {
    let mut p = Project::new("t", TextureResolution::R4096);
    // A red square in the middle, and a blue band at the top that the
    // crop removes (276 × 162 covers rows 846 to 3250 of 4096).
    p.add(rect(
        (2048.0, 2048.0),
        (1024.0, 1024.0),
        Rgba::rgb(255, 0, 0),
    ));
    p.add(rect((2048.0, 400.0), (4096.0, 800.0), Rgba::rgb(0, 0, 255)));
    let pixmap = render_cover(&p, 0, 276, 162, Some(WHITE), &mut FontLibrary::bundled());
    let img = to_rgba(&pixmap);
    assert_eq!(img.dimensions(), (276, 162));
    assert_eq!(img.get_pixel(138, 81).0, [255, 0, 0, 255]);
    assert_eq!(img.get_pixel(138, 0).0, [255, 255, 255, 255]);
    assert_eq!(img.get_pixel(0, 81).0, [255, 255, 255, 255]);
}

#[test]
fn an_image_covered_is_resized_then_cropped() {
    // 1280 × 720: resized to 288 × 162, then 6 columns cropped each side.
    // A red band 20 px wide at the left becomes 4.5 px: cropped away.
    let image = image::RgbaImage::from_fn(1280, 720, |x, _| {
        if x < 20 {
            image::Rgba([255, 0, 0, 255])
        } else {
            image::Rgba([0, 0, 255, 255])
        }
    });
    let img = to_rgba(&cover_image(&image, 276, 162));
    assert_eq!(img.dimensions(), (276, 162));
    let [r, _, b, a] = img.get_pixel(0, 81).0;
    assert!(b > 200 && r < 40 && a == 255, "{r} {b} {a}");
}

#[test]
fn jpeg_of_a_transparent_picture_is_white() {
    let pixmap = render_cover(
        &Project::new("t", TextureResolution::R2048),
        0,
        276,
        162,
        None,
        &mut FontLibrary::bundled(),
    );
    let bytes = encode_jpeg(&pixmap, 90).unwrap();
    let back = image::load_from_memory(&bytes).unwrap().to_rgb8();
    assert_eq!(back.dimensions(), (276, 162));
    assert!(back.get_pixel(10, 10).0.iter().all(|c| *c > 250));
}
