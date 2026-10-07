//! Pixel tests of the export renderer.

use std::sync::Arc;

use tp_core::document::{
    CharStyle, ColorStop, Frame, Gradient, GradientKind, Node, Object, ObjectId, Paint, PathData,
    Rgba, ShapeKind, StrokeAlign, StrokeStyle, Subpath, TextBlock,
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
    o.fill = fill.into();
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
    text.fill = Rgba::rgb(0, 0, 0).into();
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
        paint: RED.into(),
        width: 10.0,
        ..Default::default()
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
    v.fill = RED.into();
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
        paint: BLACK.into(),
        width: 8.0,
        ..Default::default()
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
    o.fill = BLUE.into();
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
    flipped.fill = RED.into();
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

// Stroke options.

use tp_core::document::{Cap, Dash, LineStyle};

fn black_stroke(width: f64, align: StrokeAlign) -> Option<StrokeStyle> {
    Some(StrokeStyle {
        paint: BLACK.into(),
        width,
        align,
        ..Default::default()
    })
}

#[test]
fn inside_border_stays_within_the_shape() {
    let mut p = project(TextureResolution::R2048);
    let mut r = shape(
        ShapeKind::rectangle(),
        (500.0, 500.0),
        (400.0, 400.0),
        0.0,
        RED,
    );
    r.stroke = black_stroke(30.0, StrokeAlign::Inside);
    p.add(r);
    let img = draw(&p, 2048, Some(WHITE));
    assert_eq!(px(&img, 300 + 2, 500), [0, 0, 0, 255], "at the edge");
    assert_eq!(px(&img, 300 + 28, 500), [0, 0, 0, 255], "30 px deep");
    assert_eq!(px(&img, 300 + 32, 500), [255, 0, 0, 255], "then the fill");
    assert_eq!(px(&img, 300 - 2, 500), [255; 4], "nothing outside");
    assert_eq!(px(&img, 302, 302), [0, 0, 0, 255], "sharp inner corner");
}

#[test]
fn outside_outline_never_covers_the_letters() {
    let mut fonts = FontLibrary::bundled();
    let mut block = TextBlock::new("OA", CharStyle::default());
    let layout = tp_text::layout(&mut fonts, "OA", &block.style);
    block.layout_size = layout.size;
    let mut text = Object::text(ObjectId(0), block, Point::new(1000.0, 1000.0));
    text.fill = RED.into();
    let mut p = project(TextureResolution::R2048);
    let id = p.add(text);
    let plain = draw(&p, 2048, Some(WHITE));
    let mut stroked = (**p.surface().get(id).unwrap()).clone();
    stroked.stroke = black_stroke(12.0, StrokeAlign::Outside);
    p.surface_mut().replace(&[stroked]);
    let outlined = draw(&p, 2048, Some(WHITE));
    let red = |x: u32, y: u32| px(&plain, x, y) == [255, 0, 0, 255];
    let (mut letters, mut black) = (0, 0);
    for (x, y, b) in outlined.enumerate_pixels() {
        // Inside the letters: red with red neighbours (edge pixels mix
        // the fill and the outline by anti-aliasing).
        let interior = (1..2047).contains(&x)
            && (1..2047).contains(&y)
            && red(x, y)
            && red(x - 1, y)
            && red(x + 1, y)
            && red(x, y - 1)
            && red(x, y + 1);
        if interior {
            letters += 1;
            assert_eq!(b.0, [255, 0, 0, 255], "letter pixel ({x}, {y})");
        }
        if b.0 == [0, 0, 0, 255] {
            black += 1;
        }
    }
    assert!(letters > 1000 && black > 1000, "{letters} {black}");
}

fn horizontal_line(width: f64, line_style: LineStyle) -> Object {
    let mut l = Object::from_path(
        ObjectId(0),
        PathData::new(vec![polyline(&[(100.0, 500.0), (300.0, 500.0)], false)]),
    );
    l.fill = RED.into();
    l.edit_path(|p| {
        p.line_width = width;
        p.line_style = line_style;
    });
    l
}

#[test]
fn dashed_pinstripe_without_stroke() {
    let mut p = project(TextureResolution::R2048);
    p.add(horizontal_line(
        10.0,
        LineStyle {
            dash: Some(Dash {
                dash: 20.0,
                gap: 20.0,
            }),
            cap: Cap::Butt,
            ..Default::default()
        },
    ));
    let img = draw(&p, 2048, Some(WHITE));
    assert_eq!(px(&img, 110, 500), [255, 0, 0, 255], "first dash");
    assert_eq!(px(&img, 130, 500), [255; 4], "first gap");
    assert_eq!(px(&img, 150, 500), [255, 0, 0, 255], "second dash");
    assert_eq!(px(&img, 110, 507), [255; 4], "10 px wide");
}

#[test]
fn dotted_line_gives_round_dots() {
    let mut p = project(TextureResolution::R2048);
    p.add(horizontal_line(
        10.0,
        LineStyle {
            dash: Some(Dash {
                dash: 0.0,
                gap: 20.0,
            }),
            cap: Cap::Round,
            ..Default::default()
        },
    ));
    let img = draw(&p, 2048, Some(WHITE));
    for x in [100, 120, 140, 280] {
        assert_eq!(px(&img, x, 500), [255, 0, 0, 255], "dot at {x}");
    }
    assert_eq!(px(&img, 110, 500), [255; 4], "gap between dots");
    assert_eq!(px(&img, 124, 504), [255; 4], "round, not square");
}

#[test]
fn square_caps_extend_the_line() {
    let draw_cap = |cap| {
        let mut p = project(TextureResolution::R2048);
        p.add(horizontal_line(
            20.0,
            LineStyle {
                cap,
                ..Default::default()
            },
        ));
        draw(&p, 2048, Some(WHITE))
    };
    let square = draw_cap(Cap::Square);
    assert_eq!(px(&square, 308, 508), [255, 0, 0, 255], "square corner");
    let butt = draw_cap(Cap::Butt);
    assert_eq!(px(&butt, 302, 500), [255; 4], "butt stops at the end");
    assert_eq!(px(&butt, 298, 500), [255, 0, 0, 255]);
}

#[test]
fn line_outline_alignment_sets_the_widths() {
    // A 20 px line with a 4 px outline, sampled across its width.
    let column = |align| {
        let mut p = project(TextureResolution::R2048);
        let mut l = horizontal_line(20.0, LineStyle::default());
        l.stroke = black_stroke(4.0, align);
        p.add(l);
        let img = draw(&p, 2048, Some(WHITE));
        (0..16)
            .map(move |d| px(&img, 200, 500 + d))
            .collect::<Vec<_>>()
    };
    let black = [0, 0, 0, 255];
    let red = [255, 0, 0, 255];
    // Center: outline 8..12 around the 10 px edge.
    let c = column(StrokeAlign::Center);
    assert_eq!((c[7], c[9], c[11], c[13]), (red, black, black, [255; 4]));
    // Outside: the line keeps its 10 px half-width, outline 10..14.
    let o = column(StrokeAlign::Outside);
    assert_eq!((o[9], o[11], o[13], o[15]), (red, black, black, [255; 4]));
    // Inside: outline 6..10, nothing past 10.
    let i = column(StrokeAlign::Inside);
    assert_eq!((i[5], i[7], i[9], i[11]), (red, black, black, [255; 4]));
}

fn two_stops(kind: GradientKind, a: Rgba, b: Rgba) -> Gradient {
    Gradient::new(kind, &[ColorStop::new(0.0, a), ColorStop::new(1.0, b)])
}

#[test]
fn linear_gradient_across_the_surface() {
    let mut p = project(TextureResolution::R2048);
    let black = Rgba::rgb(0, 0, 0);
    let mut o = shape(
        ShapeKind::rectangle(),
        (1024.0, 1024.0),
        (2048.0, 2048.0),
        0.0,
        WHITE,
    );
    o.fill = Paint::Gradient(two_stops(GradientKind::Linear, black, WHITE));
    p.add(o);
    let img = draw(&p, 1024, None);
    let mut prev = 0;
    for x in 0..1024 {
        let c = px(&img, x, 512);
        assert_eq!((c[0], c[1], c[3]), (c[2], c[2], 255), "gray at {x}");
        assert!(c[0] >= prev, "non-decreasing at {x}");
        prev = c[0];
    }
    assert!(px(&img, 0, 512)[0] <= 1 && px(&img, 1023, 512)[0] >= 254);
    assert!(px(&img, 512, 512)[0].abs_diff(128) <= 1);
}

#[test]
fn radial_gradient_center_and_edge() {
    let mut p = project(TextureResolution::R2048);
    let mut o = shape(
        ShapeKind::rectangle(),
        (1024.0, 1024.0),
        (1024.0, 1024.0),
        0.0,
        WHITE,
    );
    o.fill = Paint::Gradient(two_stops(GradientKind::Radial, RED, BLUE));
    p.add(o);
    let img = draw(&p, 2048, Some(WHITE));
    assert!(
        close(px(&img, 1024, 1024), [255, 0, 0, 255], 2),
        "red center"
    );
    // Halfway along the radius (256 px): the 50% mix.
    assert!(close(px(&img, 1280, 1024), [128, 0, 128, 255], 2));
    assert!(close(px(&img, 1024, 768), [128, 0, 128, 255], 2));
    // Beyond the radius (corners of the square): the last stop.
    assert!(
        close(px(&img, 530, 530), [0, 0, 255, 255], 1),
        "blue corner"
    );
}

#[test]
fn gradient_follows_rotation() {
    let mut p = project(TextureResolution::R2048);
    let mut o = shape(
        ShapeKind::rectangle(),
        (1024.0, 1024.0),
        (1000.0, 400.0),
        90.0,
        WHITE,
    );
    o.fill = Paint::Gradient(two_stops(GradientKind::Linear, RED, BLUE));
    p.add(o);
    let img = draw(&p, 2048, Some(WHITE));
    // Turned 90° clockwise: the left edge (red) is now at the top.
    assert!(close(px(&img, 1024, 530), [255, 0, 0, 255], 4), "red top");
    assert!(
        close(px(&img, 1024, 1518), [0, 0, 255, 255], 4),
        "blue bottom"
    );
}

#[test]
fn outside_stroke_with_a_gradient() {
    let mut p = project(TextureResolution::R2048);
    let mut o = shape(
        ShapeKind::rectangle(),
        (1024.0, 1024.0),
        (1000.0, 1000.0),
        0.0,
        WHITE,
    );
    o.stroke = Some(StrokeStyle {
        paint: Paint::Gradient(two_stops(GradientKind::Linear, RED, BLUE)),
        width: 40.0,
        align: StrokeAlign::Outside,
        ..StrokeStyle::default()
    });
    p.add(o);
    let img = draw(&p, 2048, Some(Rgba::rgb(0, 255, 0)));
    // Left of the shape: before the start, the first stop.
    assert!(close(px(&img, 504, 1024), [255, 0, 0, 255], 2), "red left");
    assert!(
        close(px(&img, 1544, 1024), [0, 0, 255, 255], 2),
        "blue right"
    );
    // On the top band, at the middle: the 50% mix.
    assert!(close(px(&img, 1024, 504), [128, 0, 128, 255], 2));
    // Inside the shape: the white fill, not the stroke.
    assert_eq!(px(&img, 1024, 1024), [255; 4]);
}

#[test]
fn degenerate_gradient_paints_the_last_stop() {
    let mut p = project(TextureResolution::R2048);
    let mut o = shape(
        ShapeKind::rectangle(),
        (1024.0, 1024.0),
        (400.0, 400.0),
        0.0,
        WHITE,
    );
    let mut g = two_stops(GradientKind::Linear, RED, BLUE);
    g.end = g.start;
    o.fill = Paint::Gradient(g);
    p.add(o);
    let img = draw(&p, 2048, Some(WHITE));
    assert_eq!(px(&img, 1024, 1024), [0, 0, 255, 255]);
}

#[test]
fn gradient_opacity_and_transparent_stops() {
    let mut p = project(TextureResolution::R2048);
    let mut o = shape(
        ShapeKind::rectangle(),
        (1024.0, 1024.0),
        (2048.0, 2048.0),
        0.0,
        WHITE,
    );
    o.fill = Paint::Gradient(Gradient::from_color(GradientKind::Linear, RED));
    o.opacity = 0.5;
    p.add(o);
    let img = draw(&p, 1024, None);
    let left = px(&img, 0, 512);
    assert_eq!((left[0], left[1], left[2]), (255, 0, 0), "never darkened");
    assert!(left[3].abs_diff(127) <= 2, "half opacity: {left:?}");
    let mid = px(&img, 512, 512);
    assert_eq!((mid[0], mid[1], mid[2]), (255, 0, 0), "never darkened");
    assert!(mid[3].abs_diff(64) <= 2, "{mid:?}");
}
