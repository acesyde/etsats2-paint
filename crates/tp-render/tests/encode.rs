//! Mipmaps, PNG and DDS encoding.

use tp_core::document::{Frame, Object, ObjectId, Rgba, ShapeKind};
use tp_core::kurbo::{Point, Size};
use tp_core::{Project, TextureResolution};
use tp_render::{
    DdsEncoding, Pixmap, RenderOptions, dds_size, encode_dds, encode_png, mip_chain, render,
};

fn scene(size: u32) -> Pixmap {
    let mut p = Project::new("t", TextureResolution::R2048);
    let mut o = Object::new(
        ObjectId(0),
        ShapeKind::Ellipse,
        Frame::new(Point::new(1024.0, 1024.0), Size::new(1200.0, 800.0), 20.0),
    );
    o.fill = Rgba::rgb(200, 30, 40);
    p.add(o);
    render(
        &p,
        0,
        RenderOptions {
            size,
            background: Some(Rgba::rgb(240, 240, 240)),
        },
        &mut tp_text::FontLibrary::bundled(),
        &mut |_, _| true,
    )
    .unwrap()
}

#[test]
fn mip_chain_halves_down_to_one_pixel() {
    let levels = mip_chain(&scene(256));
    let sizes: Vec<u32> = levels.iter().map(Pixmap::width).collect();
    assert_eq!(sizes, vec![256, 128, 64, 32, 16, 8, 4, 2, 1]);
    // Center stays the ellipse color at every level above 8 px.
    let mid = &levels[3];
    let c = mid.pixel(16, 16).unwrap().demultiply();
    assert_eq!((c.red(), c.green(), c.blue()), (200, 30, 40));
}

#[test]
fn png_round_trip() {
    let pixmap = scene(256);
    let bytes = encode_png(&pixmap).unwrap();
    let back = image::load_from_memory(&bytes).unwrap().to_rgba8();
    assert_eq!(back, tp_render::to_rgba(&pixmap));
}

fn u32_at(b: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(b[offset..offset + 4].try_into().unwrap())
}

#[test]
fn dds_bc3_header_and_decoding() {
    let pixmap = scene(2048);
    let bytes = encode_dds(&pixmap, DdsEncoding::Bc3, &mut |_, _| true).unwrap();
    assert_eq!(&bytes[..4], b"DDS ");
    assert_eq!((u32_at(&bytes, 12), u32_at(&bytes, 16)), (2048, 2048));
    assert_eq!(u32_at(&bytes, 28), 12, "mip levels");
    assert_eq!(&bytes[84..88], b"DXT5");
    assert_eq!(bytes.len() as u64, dds_size(2048, DdsEncoding::Bc3));
    let decoded = image::load_from_memory_with_format(&bytes, image::ImageFormat::Dds)
        .unwrap()
        .to_rgba8();
    assert_eq!(decoded.dimensions(), (2048, 2048));
    let original = tp_render::to_rgba(&pixmap);
    for (x, y) in [(1024, 1024), (10, 10), (2000, 1500)] {
        let a = decoded.get_pixel(x, y).0;
        let b = original.get_pixel(x, y).0;
        assert!(
            a.iter().zip(b).all(|(p, q)| p.abs_diff(q) <= 8),
            "{a:?} {b:?}"
        );
    }
}

#[test]
fn dds_uncompressed_is_exact_bgra() {
    let pixmap = scene(64);
    let bytes = encode_dds(&pixmap, DdsEncoding::Rgba, &mut |_, _| true).unwrap();
    assert_eq!(u32_at(&bytes, 28), 7);
    assert_eq!(u32_at(&bytes, 88), 32, "bits per pixel");
    assert_eq!(bytes.len() as u64, dds_size(64, DdsEncoding::Rgba));
    let original = tp_render::to_rgba(&pixmap);
    let [r, g, b, a] = original.get_pixel(32, 32).0;
    let offset = 128 + (32 * 64 + 32) * 4;
    assert_eq!(&bytes[offset..offset + 4], &[b, g, r, a]);
}

#[test]
fn small_levels_are_padded_to_a_block() {
    let bytes = encode_dds(&scene(4), DdsEncoding::Bc3, &mut |_, _| true).unwrap();
    // 4×4, 2×2 and 1×1 levels: one 16-byte block each.
    assert_eq!(bytes.len(), 128 + 3 * 16);
}

/// Timing of full exports of a busy livery (release builds only).
#[test]
#[ignore = "performance check; run with --release -- --ignored --nocapture"]
fn export_timings() {
    use tp_core::document::{CharStyle, StrokeStyle, TextBlock};
    for res in [TextureResolution::R4096, TextureResolution::R8192] {
        let mut p = Project::new("perf", res);
        let side = f64::from(res.side());
        let mut fonts = tp_text::FontLibrary::bundled();
        for i in 0..200 {
            let f = f64::from(i);
            let mut o = Object::new(
                ObjectId(0),
                if i % 2 == 0 {
                    ShapeKind::Ellipse
                } else {
                    ShapeKind::Rectangle {
                        corner_radius: 30.0,
                    }
                },
                Frame::new(
                    Point::new((f * 97.0) % side, (f * 53.0) % side),
                    Size::new(side / 10.0, side / 20.0),
                    f,
                ),
            );
            o.fill = Rgba::rgb((i * 7 % 255) as u8, 80, 160);
            o.opacity = 0.8;
            p.add(o);
        }
        for i in 0..30 {
            let mut block = TextBlock::new(
                "ACE LOGISTICS 24",
                CharStyle {
                    size: side / 25.0,
                    ..CharStyle::default()
                },
            );
            block.layout_size = tp_text::layout(&mut fonts, &block.content, &block.style).size;
            let mut t = Object::text(
                ObjectId(0),
                block,
                Point::new(side / 2.0, f64::from(i) * side / 30.0 + 50.0),
            );
            t.stroke = Some(StrokeStyle {
                color: Rgba::rgb(0, 0, 0),
                width: 6.0,
            });
            p.add(t);
        }
        let start = std::time::Instant::now();
        let pixmap = render(
            &p,
            0,
            RenderOptions {
                size: res.side(),
                background: Some(Rgba::rgb(255, 255, 255)),
            },
            &mut fonts,
            &mut |_, _| true,
        )
        .unwrap();
        let rendered = start.elapsed();
        let png = encode_png(&pixmap).unwrap();
        let png_time = start.elapsed() - rendered;
        let t = std::time::Instant::now();
        let dds = encode_dds(&pixmap, DdsEncoding::Bc3, &mut |_, _| true).unwrap();
        println!(
            "{}: render {rendered:?}, png {png_time:?} ({} MB), dds {:?} ({} MB)",
            res.side(),
            png.len() / 1_000_000,
            t.elapsed(),
            dds.len() / 1_000_000
        );
    }
}
