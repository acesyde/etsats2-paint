use tp_core::document::{CharStyle, Frame, Paint, Shadow, ShapeKind, TextBlock};
use tp_core::kurbo::Size;

use super::*;

fn square(id: u64, at: Point) -> Object {
    let mut o = Object::new(
        ObjectId(id),
        ShapeKind::rectangle(),
        Frame::new(at, Size::new(100.0, 100.0), 0.0),
    );
    o.fill = Paint::Solid(Rgba::rgb(255, 255, 255));
    o.shadow = Some(Shadow {
        color: Rgba::rgb(0, 0, 0),
        opacity: 1.0,
        offset: Vec2::new(10.0, 10.0),
        blur: 0.0,
        swatch: None,
    });
    o
}

struct Canvas {
    ctx: egui::Context,
    fonts: FontLibrary,
    cache: ShadowCache,
}

impl Canvas {
    fn new() -> Self {
        Self {
            ctx: egui::Context::default(),
            fonts: FontLibrary::bundled(),
            cache: ShadowCache::default(),
        }
    }

    /// One frame drawing `objects` at `zoom`: where each shadow goes.
    fn frame(&mut self, objects: &[&Object], zoom: f64) -> Vec<Option<Rect>> {
        self.cache.begin_frame(&self.ctx, self.fonts.generation());
        let placed = self.draw(objects, zoom);
        self.cache.end_frame(&self.ctx, &self.fonts);
        placed
    }

    /// The shadows of `objects` without picking up the running job.
    fn draw(&mut self, objects: &[&Object], zoom: f64) -> Vec<Option<Rect>> {
        objects
            .iter()
            .map(|o| self.cache.layer(o, 1.0, zoom, None, 0.0).map(|p| p.rect))
            .collect()
    }

    /// Frames until no job runs.
    fn settle(&mut self, objects: &[&Object], zoom: f64) -> Vec<Option<Rect>> {
        for _ in 0..1000 {
            let placed = self.frame(objects, zoom);
            if !self.cache.is_rendering() {
                return placed;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        panic!("the shadows never finished");
    }
}

#[test]
fn moving_an_object_reuses_its_layer() {
    let mut canvas = Canvas::new();
    let mut o = square(1, Point::new(500.0, 500.0));
    let first = canvas.settle(&[&o], 1.0)[0].expect("a layer");
    assert_eq!(canvas.cache.renders, 1);
    // The hard shadow covers the square moved by 10 / 10.
    let black = Rgba::with_alpha(0, 0, 0, 255);
    let center = o.frame.center;
    assert_eq!(
        canvas
            .cache
            .color_at(o.id, center, Point::new(555.0, 555.0)),
        Some(black)
    );
    o.frame.center = Point::new(1200.0, 300.0);
    let moved = canvas.settle(&[&o], 1.0)[0].expect("a layer");
    assert_eq!(canvas.cache.renders, 1, "not rendered again");
    assert_eq!(moved, first + Vec2::new(700.0, -200.0));
}

#[test]
fn blur_and_zoom_bucket_render_again() {
    let mut canvas = Canvas::new();
    let mut o = square(1, Point::new(500.0, 500.0));
    canvas.settle(&[&o], 1.0);
    assert_eq!(canvas.cache.renders, 1);
    o.shadow.as_mut().unwrap().blur = 20.0;
    canvas.settle(&[&o], 1.0);
    assert_eq!(canvas.cache.renders, 2);
    // Zooming within a bucket keeps the layer, another bucket renders it.
    canvas.settle(&[&o], 0.8);
    assert_eq!(canvas.cache.renders, 2);
    canvas.settle(&[&o], 0.25);
    assert_eq!(canvas.cache.renders, 3);
    canvas.settle(&[&o], 0.1);
    assert_eq!(canvas.cache.renders, 4);
    // Above 100 %, layers stay at one pixel per texture pixel.
    canvas.settle(&[&o], 1.0);
    canvas.settle(&[&o], 4.0);
    assert_eq!(canvas.cache.renders, 5);
}

#[test]
fn at_most_one_job_at_a_time() {
    let mut canvas = Canvas::new();
    let a = square(1, Point::new(500.0, 500.0));
    let b = square(2, Point::new(900.0, 500.0));
    canvas.frame(&[&a], 1.0);
    assert_eq!(canvas.cache.jobs, 1);
    assert!(canvas.cache.is_rendering());
    // While it runs, a new shadow waits for the next job.
    canvas.draw(&[&a, &b], 1.0);
    canvas.cache.end_frame(&canvas.ctx, &canvas.fonts);
    assert_eq!(canvas.cache.jobs, 1);
    let placed = canvas.settle(&[&a, &b], 1.0);
    assert_eq!(canvas.cache.jobs, 2);
    assert!(placed.iter().all(Option::is_some));
    // Shadows missing together are rendered by one job.
    let mut fresh = Canvas::new();
    fresh.settle(&[&a, &b], 1.0);
    assert_eq!((fresh.cache.jobs, fresh.cache.renders), (1, 2));
}

#[test]
fn the_previous_layer_shows_while_rendering() {
    let mut canvas = Canvas::new();
    let mut o = square(1, Point::new(500.0, 500.0));
    let before = canvas.settle(&[&o], 1.0)[0].expect("a layer");
    o.shadow.as_mut().unwrap().blur = 40.0;
    o.frame.center.x += 100.0;
    let placed = canvas.frame(&[&o], 1.0)[0].expect("the previous layer");
    assert!(canvas.cache.is_rendering());
    assert_eq!(placed, before + Vec2::new(100.0, 0.0));
    let after = canvas.settle(&[&o], 1.0)[0].expect("a layer");
    assert!(after.width() > before.width() + 50.0, "blurred: wider");
}

#[test]
fn no_shadow_no_layer_and_unused_layers_are_pruned() {
    let mut canvas = Canvas::new();
    let mut o = square(1, Point::new(500.0, 500.0));
    canvas.settle(&[&o], 1.0);
    assert_eq!(canvas.cache.len(), 1);
    o.shadow = None;
    assert_eq!(canvas.settle(&[&o], 1.0), vec![None]);
    // Last drawn at 0 s: kept for 2 s.
    canvas.cache.prune(1.5);
    assert_eq!(canvas.cache.len(), 1);
    canvas.cache.prune(2.5);
    assert!(canvas.cache.is_empty());
}

#[test]
fn the_layer_is_capped() {
    let mut o = square(1, Point::new(2048.0, 2048.0));
    o.frame.size = Size::new(4096.0, 600.0);
    o.shadow.as_mut().unwrap().blur = 200.0;
    let mut canvas = Canvas::new();
    let rect = canvas.settle(&[&o], 1.0)[0].expect("a layer");
    let entry = canvas.cache.entries.get(&o.id).unwrap();
    let [w, h] = entry.layer.as_ref().unwrap().image.size;
    assert!(w.max(h) <= MAX_SIDE as usize, "{w} × {h}");
    // Still covers the whole blurred shadow.
    assert!(rect.width() > 4096.0 + 400.0, "{rect:?}");
}

/// A lettering across a 4096 px texture with a 200 px blur, at 100 % and
/// 25 %: each layer renders in under 30 ms in release. Timings are
/// printed (`--release -- --ignored --nocapture`).
#[test]
#[ignore = "performance measurement: run in release"]
fn worker_render_time() {
    use std::time::Instant;
    let mut fonts = FontLibrary::bundled();
    let mut cache = DrawCache::default();
    let mut o = Object::new(
        ObjectId(1),
        ShapeKind::Text,
        Frame::new(Point::ORIGIN, Size::new(4096.0, 728.0), 0.0),
    );
    o.text = Some(TextBlock::new(
        "ACE LOGISTICS",
        CharStyle {
            size: 560.0,
            ..CharStyle::default()
        },
    ));
    o.shadow = Some(Shadow {
        blur: 200.0,
        ..Shadow::DEFAULT
    });
    for zoom in [1.0, 0.25] {
        let bucket = bucket(zoom);
        let key = Key {
            object: at_origin(&o),
            opacity: 1.0,
            bucket,
            fonts: fonts.generation(),
        };
        let item = Item {
            id: o.id,
            scale: scale_for(&key.object, bucket),
            key,
            asset: None,
        };
        // Glyph outlines are kept between jobs: the first render warms up.
        render(&item, &mut fonts, &mut cache).expect("a layer");
        let mut times = Vec::new();
        let mut size = [0, 0];
        for _ in 0..5 {
            let start = Instant::now();
            let (image, _) = render(&item, &mut fonts, &mut cache).expect("a layer");
            times.push(start.elapsed());
            size = image.size;
        }
        let worst = times.iter().max().unwrap();
        println!(
            "zoom {zoom}: layer {} × {} at scale {:.3}, worst {worst:?} ({times:?})",
            size[0], size[1], item.scale
        );
        if !cfg!(debug_assertions) {
            assert!(worst.as_millis() < 30, "zoom {zoom}: {worst:?}");
        }
    }
}
