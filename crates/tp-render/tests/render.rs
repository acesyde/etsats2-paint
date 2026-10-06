//! Pixel tests of the export renderer.

use std::sync::Arc;

use tp_core::document::{
    CharStyle, Frame, Node, Object, ObjectId, PathData, Rgba, ShapeKind, StrokeStyle, Subpath,
    TextBlock,
};
use tp_core::kurbo::{Point, Shape, Size};
use tp_core::{AssetKind, Project, TextureResolution};
use tp_render::{Cancelled, RenderOptions, render, to_rgba};
use tp_text::FontLibrary;

const WHITE: Rgba = Rgba::rgb(255, 255, 255);
const RED: Rgba = Rgba::rgb(255, 0, 0);
const BLUE: Rgba = Rgba::rgb(0, 0, 255);

fn project(res: TextureResolution) -> Project {
    Project::new("t", res)
}

fn shape(kind: ShapeKind, c: (f64, f64), s: (f64, f64), rot: f64, fill: Rgba) -> Object {
    let mut o = Object::new(
        ObjectId(0),
        kind,
        Frame::new(Point::new(c.0, c.1), Size::new(s.0, s.1), rot),
    );
    o.fill = fill;
    o
}

fn draw(p: &Project, size: u32, background: Option<Rgba>) -> image::RgbaImage {
    let pixmap = render(
        p,
        0,
        RenderOptions { size, background },
        &mut FontLibrary::bundled(),
        &mut |_, _| true,
    )
    .unwrap();
    to_rgba(&pixmap)
}

fn px(img: &image::RgbaImage, x: u32, y: u32) -> [u8; 4] {
    img.get_pixel(x, y).0
}

fn close(a: [u8; 4], b: [u8; 4], tol: u8) -> bool {
    a.iter().zip(b).all(|(x, y)| x.abs_diff(y) <= tol)
}

#[test]
fn background_or_transparent() {
    let p = project(TextureResolution::R2048);
    assert_eq!(px(&draw(&p, 64, Some(WHITE)), 3, 3), [255; 4]);
    assert_eq!(px(&draw(&p, 64, None), 3, 3)[3], 0);
}

#[test]
fn rectangle_edges_at_full_and_half_size() {
    let mut p = project(TextureResolution::R2048);
    p.add(shape(
        ShapeKind::rectangle(),
        (100.0, 100.0),
        (100.0, 50.0),
        0.0,
        RED,
    ));
    let full = draw(&p, 2048, Some(WHITE));
    assert_eq!(px(&full, 50, 100), [255, 0, 0, 255]);
    assert_eq!(px(&full, 149, 124), [255, 0, 0, 255]);
    assert_eq!(px(&full, 49, 100), [255; 4]);
    assert_eq!(px(&full, 150, 100), [255; 4]);
    assert_eq!(px(&full, 100, 74), [255; 4]);
    let half = draw(&p, 1024, Some(WHITE));
    assert_eq!(px(&half, 25, 50), [255, 0, 0, 255]);
    assert_eq!(px(&half, 24, 50), [255; 4]);
}

#[test]
fn rotated_rounded_rectangle() {
    let mut p = project(TextureResolution::R2048);
    p.add(shape(
        ShapeKind::Rectangle {
            corner_radius: 20.0,
        },
        (500.0, 500.0),
        (200.0, 200.0),
        45.0,
        BLUE,
    ));
    let img = draw(&p, 2048, Some(WHITE));
    assert_eq!(px(&img, 500, 500), [0, 0, 255, 255]);
    // The diamond tip (141 px away) is cut by the rounding.
    assert_eq!(px(&img, 628, 500), [0, 0, 255, 255]);
    assert_eq!(px(&img, 638, 500), [255; 4]);
    // Outside the diamond's edges.
    assert_eq!(px(&img, 580, 580), [255; 4]);
}

#[test]
fn half_transparent_red_over_blue() {
    let mut p = project(TextureResolution::R2048);
    p.add(shape(
        ShapeKind::Ellipse,
        (300.0, 300.0),
        (200.0, 200.0),
        0.0,
        BLUE,
    ));
    let mut red = shape(
        ShapeKind::rectangle(),
        (300.0, 300.0),
        (100.0, 100.0),
        0.0,
        RED,
    );
    red.opacity = 0.5;
    p.add(red);
    let img = draw(&p, 2048, Some(WHITE));
    assert!(
        close(px(&img, 300, 300), [128, 0, 127, 255], 2),
        "{:?}",
        px(&img, 300, 300)
    );
}

#[test]
fn hidden_groups_skipped_locked_objects_drawn_and_clipping() {
    let mut p = project(TextureResolution::R2048);
    let a = p.add(shape(
        ShapeKind::rectangle(),
        (100.0, 100.0),
        (50.0, 50.0),
        0.0,
        RED,
    ));
    let g = p.group(&[a]).unwrap();
    let mut group = (**p.surface().get(g).unwrap()).clone();
    group.visible = false;
    p.surface_mut().replace(&[group]);
    let mut locked = shape(
        ShapeKind::rectangle(),
        (400.0, 100.0),
        (50.0, 50.0),
        0.0,
        BLUE,
    );
    locked.locked = true;
    p.add(locked);
    // Half outside the surface.
    p.add(shape(
        ShapeKind::rectangle(),
        (2048.0, 1000.0),
        (200.0, 200.0),
        0.0,
        RED,
    ));
    let img = draw(&p, 2048, Some(WHITE));
    assert_eq!(px(&img, 100, 100), [255; 4]);
    assert_eq!(px(&img, 400, 100), [0, 0, 255, 255]);
    assert_eq!(px(&img, 2047, 1000), [255, 0, 0, 255]);
}

#[test]
fn text_counter_is_empty_and_stroke_covers_outline() {
    let mut fonts = FontLibrary::bundled();
    let mut block = TextBlock::new("O", CharStyle::default());
    let layout = tp_text::layout(&mut fonts, "O", &block.style);
    block.layout_size = layout.size;
    let mut text = Object::text(ObjectId(0), block, Point::new(1000.0, 1000.0));
    text.fill = Rgba::rgb(0, 0, 0);
    let mut p = project(TextureResolution::R2048);
    let id = p.add(text.clone());
    let placed = (**p.surface().get(id).unwrap()).clone();
    let outline = tp_text::layout_to_doc(&placed)
        * tp_text::GlyphCache::default().outline(&mut fonts, &layout);
    let b = outline.bounding_box();
    let (cx, cy) = (b.center().x as u32, b.center().y as u32);
    let img = draw(&p, 2048, Some(WHITE));
    assert_eq!(px(&img, cx, cy), [255; 4], "counter of the O");
    let ring = (b.x0 + 0.06 * b.width()) as u32;
    assert_eq!(px(&img, ring, cy), [0, 0, 0, 255], "ring of the O");

    let mut stroked = placed;
    stroked.stroke = Some(StrokeStyle {
        color: RED,
        width: 10.0,
    });
    p.surface_mut().replace(&[stroked]);
    let img = draw(&p, 2048, Some(WHITE));
    assert_eq!(px(&img, b.x0.round() as u32, cy), [255, 0, 0, 255]);
}

fn png(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
    let mut out = Vec::new();
    image::RgbaImage::from_pixel(w, h, image::Rgba(rgba))
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

#[test]
fn raster_image_and_its_opacity() {
    let mut p = project(TextureResolution::R2048);
    let (asset, _) = p.add_asset(
        "logo",
        AssetKind::Raster,
        Arc::from(png(40, 20, [0, 0, 255, 255])),
        Size::new(40.0, 20.0),
    );
    let mut image = shape(
        ShapeKind::Image { asset },
        (1000.0, 1000.0),
        (400.0, 200.0),
        0.0,
        WHITE,
    );
    p.add(image.clone());
    let img = draw(&p, 2048, Some(WHITE));
    assert_eq!(px(&img, 1000, 1000), [0, 0, 255, 255]);
    assert_eq!(px(&img, 1195, 1095), [0, 0, 255, 255]);
    assert_eq!(px(&img, 1205, 1000), [255; 4]);
    image.opacity = 0.5;
    image.id = p.surface().objects[0].id;
    p.surface_mut().replace(&[image]);
    let img = draw(&p, 2048, Some(WHITE));
    assert!(
        close(px(&img, 1000, 1000), [128, 128, 255, 255], 2),
        "{:?}",
        px(&img, 1000, 1000)
    );
}

#[test]
fn svg_stays_sharp_at_8k() {
    let mut p = project(TextureResolution::R8192);
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50"><rect width="100" height="50" fill="red"/></svg>"#;
    let (asset, _) = p.add_asset(
        "badge",
        AssetKind::Svg,
        Arc::from(&svg[..]),
        Size::new(100.0, 50.0),
    );
    p.add(shape(
        ShapeKind::Image { asset },
        (4096.0, 4096.0),
        (512.0, 256.0),
        0.0,
        WHITE,
    ));
    let img = draw(&p, 8192, Some(WHITE));
    // Left edge at x = 3840: one pixel outside is white, one inside is red.
    assert_eq!(px(&img, 3839, 4096), [255; 4]);
    assert_eq!(px(&img, 3840, 4096), [255, 0, 0, 255]);
    assert_eq!(px(&img, 4351, 4096), [255, 0, 0, 255]);
    assert_eq!(px(&img, 4352, 4096), [255; 4]);
}

#[test]
fn progress_can_cancel() {
    let mut p = project(TextureResolution::R2048);
    for i in 0..3 {
        p.add(shape(
            ShapeKind::Ellipse,
            (100.0 * f64::from(i), 100.0),
            (50.0, 50.0),
            0.0,
            RED,
        ));
    }
    let mut seen = Vec::new();
    let result = render(
        &p,
        0,
        RenderOptions {
            size: 64,
            background: None,
        },
        &mut FontLibrary::bundled(),
        &mut |done, total| {
            seen.push((done, total));
            done < 2
        },
    );
    assert_eq!(result.unwrap_err(), Cancelled);
    assert_eq!(seen, vec![(1, 3), (2, 3)]);
}

fn polyline(points: &[(f64, f64)], closed: bool) -> Subpath {
    Subpath::new(
        points
            .iter()
            .map(|p| Node::corner(Point::new(p.0, p.1)))
            .collect(),
        closed,
    )
}

const BLACK: Rgba = Rgba::rgb(0, 0, 0);

fn red_v(line_width: f64) -> Object {
    let mut v = Object::from_path(
        ObjectId(0),
        PathData::new(vec![polyline(
            &[(100.0, 100.0), (200.0, 300.0), (300.0, 100.0)],
            false,
        )]),
    );
    v.fill = RED;
    v.edit_path(|p| p.line_width = line_width);
    v
}

#[test]
fn open_path_is_a_line_in_the_fill_color() {
    let mut p = project(TextureResolution::R2048);
    p.add(red_v(20.0));
    let img = draw(&p, 2048, Some(WHITE));
    assert_eq!(px(&img, 200, 150), [255; 4], "nothing between the arms");
    assert_eq!(
        px(&img, 200, 295),
        [255, 0, 0, 255],
        "line at the bottom of the V"
    );
    assert_eq!(px(&img, 150, 200), [255, 0, 0, 255], "line along an arm");
    // 20 px wide: about 10 px on each side of the center line.
    assert_eq!(px(&img, 150 + 13, 200), [255; 4]);
    // Round cap past the first point.
    assert_eq!(px(&img, 100, 93), [255, 0, 0, 255]);
}

#[test]
fn outlined_line() {
    let mut p = project(TextureResolution::R2048);
    let mut v = red_v(40.0);
    v.stroke = Some(StrokeStyle {
        color: BLACK,
        width: 8.0,
    });
    p.add(v);
    let img = draw(&p, 2048, Some(WHITE));
    // Along the left arm, the center line goes through (150, 200); the
    // normal direction is about (0.894, -0.447).
    let at = |d: f64| {
        let (x, y) = (150.0 + 0.894 * d, 200.0 - 0.447 * d);
        px(&img, x.round() as u32, y.round() as u32)
    };
    assert_eq!(at(0.0), [255, 0, 0, 255], "red in the middle");
    assert_eq!(at(20.0), [0, 0, 0, 255], "black on the edge");
    assert_eq!(at(-20.0), [0, 0, 0, 255], "black on the other edge");
    assert_eq!(at(30.0), [255; 4], "nothing past the outline");
    // The outline also goes around the rounded end.
    assert_eq!(px(&img, 100, 100 - 20), [0, 0, 0, 255]);
}

#[test]
fn hole_shows_the_background() {
    let mut p = project(TextureResolution::R2048);
    let mut o = Object::from_path(
        ObjectId(0),
        PathData::new(vec![
            polyline(
                &[
                    (100.0, 100.0),
                    (400.0, 100.0),
                    (400.0, 400.0),
                    (100.0, 400.0),
                ],
                true,
            ),
            polyline(
                &[
                    (200.0, 200.0),
                    (200.0, 300.0),
                    (300.0, 300.0),
                    (300.0, 200.0),
                ],
                true,
            ),
        ]),
    );
    o.fill = BLUE;
    p.add(o);
    let img = draw(&p, 2048, None);
    assert_eq!(px(&img, 150, 150), [0, 0, 255, 255]);
    assert_eq!(px(&img, 250, 250)[3], 0, "the hole is transparent");
}

#[test]
fn star_notch_is_transparent() {
    let mut p = project(TextureResolution::R2048);
    p.add(shape(
        ShapeKind::Polygon {
            sides: 5,
            star: Some(0.4),
        },
        (500.0, 500.0),
        (400.0, 400.0),
        0.0,
        RED,
    ));
    let img = draw(&p, 2048, None);
    assert_eq!(px(&img, 500, 520), [255, 0, 0, 255], "center");
    assert_eq!(px(&img, 500, 330), [255, 0, 0, 255], "top point");
    assert_eq!(px(&img, 340, 340)[3], 0, "notch between two points");
}

#[test]
fn mirrored_path_renders_mirrored() {
    use tp_core::document::{Handle, ResizeOptions, resize};
    // A right triangle with its right angle at the bottom-left.
    let tri = Object::from_path(
        ObjectId(0),
        PathData::new(vec![polyline(
            &[(100.0, 100.0), (100.0, 300.0), (300.0, 300.0)],
            true,
        )]),
    );
    let mut flipped = resize(
        std::slice::from_ref(&tri),
        tri.frame,
        Handle { x: 1, y: 0 },
        Point::new(-100.0, 200.0),
        ResizeOptions::default(),
    )
    .remove(0);
    flipped.fill = RED;
    // Now spans x -100..100 with the right angle at the bottom-right: move it
    // back on the surface.
    flipped.translate_deep(tp_core::kurbo::Vec2::new(300.0, 0.0));
    let mut p = project(TextureResolution::R2048);
    p.add(flipped);
    let img = draw(&p, 2048, Some(WHITE));
    assert_eq!(
        px(&img, 390, 290),
        [255, 0, 0, 255],
        "bottom-right is filled"
    );
    assert_eq!(px(&img, 210, 120), [255; 4], "top-left is empty");
}
