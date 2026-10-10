//! Drop shadows on the canvas: each shadowed object's shadow layer,
//! rendered by tp-render (the same rasterizer as the export) on a worker
//! thread and kept as a texture.
//!
//! A layer doesn't depend on where its object is: it is rendered with the
//! object moved to the document origin and drawn at the object's center,
//! so moving an object reuses its shadow. Changing what shapes the
//! silhouette, the shadow, the opacity, the zoom bucket or the fonts
//! renders it again; meanwhile the previous texture of the same object is
//! drawn where the object now is.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

use egui::{ColorImage, TextureHandle, TextureOptions};
use tp_core::Asset;
use tp_core::document::{Object, ObjectId, Rgba};
use tp_core::kurbo::{Point, Rect, Vec2};
use tp_render::DrawCache;
use tp_text::FontLibrary;

/// Longest side of a layer, in pixels: a blurred shadow needs no more.
pub const MAX_SIDE: u32 = 2048;

/// Entries unused this long (seconds) are dropped.
const KEEP_FOR: f64 = 2.0;

/// What a layer was rendered from.
#[derive(Clone)]
struct Key {
    /// The object moved to the document origin.
    object: Object,
    /// Its effective opacity (its own and its groups').
    opacity: f32,
    /// Layer pixels per texture pixel before the size cap: a power of two,
    /// at most 1 (the zoom bucket).
    bucket: i32,
    /// Fonts generation (texts are laid out again when fonts load).
    fonts: u64,
}

impl Key {
    fn matches(&self, object: &Object, opacity: f32, bucket: i32, fonts: u64) -> bool {
        self.opacity == opacity
            && self.bucket == bucket
            && self.fonts == fonts
            && same_silhouette(&self.object, object)
    }
}

/// Whether `moved` (at the origin) and `object` draw the same shadow once
/// placed: the same geometry, paints and shadow, wherever they are.
fn same_silhouette(moved: &Object, object: &Object) -> bool {
    let mut frame = object.frame;
    frame.center = moved.frame.center;
    moved.kind == object.kind
        && moved.frame == frame
        && moved.fill == object.fill
        && moved.stroke == object.stroke
        && moved.shadow == object.shadow
        && moved.text == object.text
        && moved.path == object.path
        && moved.mirrored == object.mirrored
}

/// `object` moved to the document origin.
fn at_origin(object: &Object) -> Object {
    let mut moved = object.clone();
    moved.frame.center = Point::ORIGIN;
    moved.children.clear();
    moved
}

/// A rendered layer, relative to its object's center.
struct Layer {
    texture: TextureHandle,
    /// Document offset of the layer's top-left corner from the object's
    /// center, and its size in texture pixels.
    rect: Rect,
    image: Arc<ColorImage>,
}

struct Entry {
    key: Key,
    /// `None`: the shadow shows nothing (fully transparent, say).
    layer: Option<Layer>,
    /// When the entry was last drawn (seconds).
    used: f64,
}

/// One layer for the worker to render.
struct Item {
    id: ObjectId,
    key: Key,
    scale: f64,
    asset: Option<Arc<Asset>>,
}

/// What the worker sends back for an item.
struct Done {
    id: ObjectId,
    key: Key,
    layer: Option<(ColorImage, Rect)>,
}

struct Job {
    items: Vec<Item>,
    /// New fonts, when they changed since the previous job.
    fonts: Option<FontLibrary>,
}

struct Worker {
    jobs: Sender<Job>,
    results: Receiver<Vec<Done>>,
}

/// Where to draw a shadow: its texture and its document rectangle.
pub struct Placed {
    pub texture: egui::TextureId,
    pub rect: Rect,
}

/// The canvas's shadow layers, by object.
#[derive(Default)]
pub struct ShadowCache {
    entries: HashMap<ObjectId, Entry>,
    worker: Option<Worker>,
    /// A job is being rendered.
    busy: bool,
    /// The renderer stopped (it could not start, or panicked): no retry.
    broken: bool,
    /// Layers missing this frame, rendered by the next job.
    missing: Vec<Item>,
    /// Fonts generation of the frame, and the one the worker has.
    fonts: u64,
    worker_fonts: Option<u64>,
    /// Layers rendered and jobs started, for tests and diagnostics.
    pub renders: u64,
    pub jobs: u64,
}

/// The layer resolution for a zoom (physical pixels per texture pixel):
/// the next power of two at or above it, at most 1, as an exponent.
pub fn bucket(zoom: f64) -> i32 {
    zoom.max(1e-6).log2().ceil().min(0.0) as i32
}

/// Layer pixels per texture pixel for `object` in `bucket`: the bucket's,
/// lowered so that the layer's longest side stays within [`MAX_SIDE`].
fn scale_for(object: &Object, bucket: i32) -> f64 {
    let base = 2f64.powi(bucket);
    let Some(shadow) = object.shadow else {
        return base;
    };
    let bounds = object.frame.bounding_box();
    let stroke = object.stroke.map_or(0.0, |s| s.width);
    let line = object.path.as_ref().map_or(0.0, |p| p.line_width);
    // The blur spreads about 1.5 × its radius on each side.
    let reach = 2.0 * (stroke + line + 1.5 * shadow.blur) + 4.0;
    let longest = bounds.width().max(bounds.height()) + reach;
    base.min(f64::from(MAX_SIDE) / longest.max(1.0))
}

/// Renders `item`'s layer; renders it again smaller if it came out past
/// [`MAX_SIDE`] (the estimate missed some of the drawn bounds).
fn render(
    item: &Item,
    fonts: &mut FontLibrary,
    cache: &mut DrawCache,
) -> Option<(ColorImage, Rect)> {
    let mut scale = item.scale;
    for attempt in 0..2 {
        let layer = tp_render::shadow_layer(
            &item.key.object,
            item.key.opacity,
            scale,
            None,
            item.asset.as_deref(),
            fonts,
            cache,
        )?;
        let (w, h) = (layer.pixmap.width(), layer.pixmap.height());
        let side = w.max(h);
        if side <= MAX_SIDE || attempt == 1 {
            let image =
                ColorImage::from_rgba_premultiplied([w as usize, h as usize], layer.pixmap.data());
            let size = Vec2::new(f64::from(w), f64::from(h)) / layer.scale;
            let rect = Rect::from_origin_size(layer.origin, size.to_size());
            return Some((image, rect));
        }
        // A few pixels short: the layer's own margins don't scale.
        scale *= f64::from(MAX_SIDE - 8) / f64::from(side);
    }
    None
}

impl ShadowCache {
    /// Picks up the finished job, if any; `fonts` is the fonts generation
    /// of this frame. Call once a frame before [`Self::layer`].
    pub fn begin_frame(&mut self, ctx: &egui::Context, fonts: u64) {
        self.fonts = fonts;
        self.missing.clear();
        let Some(worker) = &self.worker else {
            return;
        };
        if !self.busy {
            return;
        }
        match worker.results.try_recv() {
            Ok(done) => {
                self.busy = false;
                for d in done {
                    let layer = d.layer.map(|(image, rect)| {
                        let texture = ctx.load_texture(
                            format!("shadow_{}", d.id.0),
                            image.clone(),
                            TextureOptions::LINEAR,
                        );
                        Layer {
                            texture,
                            rect,
                            image: Arc::new(image),
                        }
                    });
                    let used = self.entries.get(&d.id).map_or(0.0, |e| e.used);
                    self.entries.insert(
                        d.id,
                        Entry {
                            key: d.key,
                            layer,
                            used,
                        },
                    );
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.busy = false;
                self.broken = true;
                self.worker = None;
            }
        }
    }

    /// The shadow of `object` (drawn at `opacity`, `zoom` physical pixels
    /// per texture pixel) to draw now: its layer, or while that is being
    /// rendered, the object's previous one moved to where it is. A missing
    /// layer is rendered by the next job. `asset` is an image's asset.
    pub fn layer(
        &mut self,
        object: &Object,
        opacity: f32,
        zoom: f64,
        asset: Option<&Arc<Asset>>,
        now: f64,
    ) -> Option<Placed> {
        if object.shadow.is_none() || !object.takes_shadow() {
            return None;
        }
        let bucket = bucket(zoom);
        let fonts = self.fonts;
        let entry = self.entries.get_mut(&object.id);
        let fresh = entry
            .as_ref()
            .is_some_and(|e| e.key.matches(object, opacity, bucket, fonts));
        if !fresh && !self.missing.iter().any(|i| i.id == object.id) {
            let key = Key {
                object: at_origin(object),
                opacity,
                bucket,
                fonts,
            };
            self.missing.push(Item {
                id: object.id,
                scale: scale_for(&key.object, bucket),
                key,
                asset: asset.cloned(),
            });
        }
        let entry = entry?;
        entry.used = now;
        let layer = entry.layer.as_ref()?;
        Some(Placed {
            texture: layer.texture.id(),
            rect: layer.rect + object.frame.center.to_vec2(),
        })
    }

    /// Starts rendering the layers found missing this frame, unless a job
    /// is running (they are asked for again next frame). Call once a frame
    /// after the last [`Self::layer`].
    pub fn end_frame(&mut self, ctx: &egui::Context, fonts: &FontLibrary) {
        if self.busy || self.broken || self.missing.is_empty() {
            return;
        }
        if self.worker.is_none() {
            self.worker = spawn(ctx);
            self.worker_fonts = None;
            if self.worker.is_none() {
                self.broken = true;
                return;
            }
        }
        let new_fonts = (self.worker_fonts != Some(self.fonts)).then(|| fonts.fork());
        let items = std::mem::take(&mut self.missing);
        let job = Job {
            items,
            fonts: new_fonts,
        };
        let count = job.items.len() as u64;
        let worker = self.worker.as_ref().expect("worker started");
        if worker.jobs.send(job).is_err() {
            self.broken = true;
            self.worker = None;
            return;
        }
        self.worker_fonts = Some(self.fonts);
        self.busy = true;
        self.jobs += 1;
        self.renders += count;
    }

    /// Drops the layers not drawn for a while.
    pub fn prune(&mut self, now: f64) {
        self.entries.retain(|_, e| now - e.used <= KEEP_FOR);
    }

    /// Whether layers are being rendered.
    pub fn is_rendering(&self) -> bool {
        self.busy
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Color of the shadow layer of `id` at document point `at`, as drawn
    /// for an object centered at `center`, for tests: `None` without one.
    pub fn color_at(&self, id: ObjectId, center: Point, at: Point) -> Option<Rgba> {
        let layer = self.entries.get(&id)?.layer.as_ref()?;
        let rect = layer.rect + center.to_vec2();
        let image = &layer.image;
        let [w, h] = image.size;
        let x = ((at.x - rect.x0) / rect.width() * w as f64).floor();
        let y = ((at.y - rect.y0) / rect.height() * h as f64).floor();
        if x < 0.0 || y < 0.0 || x >= w as f64 || y >= h as f64 {
            return Some(Rgba::with_alpha(0, 0, 0, 0));
        }
        let [r, g, b, a] = image.pixels[y as usize * w + x as usize].to_srgba_unmultiplied();
        Some(Rgba::with_alpha(r, g, b, a))
    }
}

/// The worker thread: renders each job's items with the fonts it was last
/// given, keeping glyph outlines and decoded images between jobs.
fn spawn(ctx: &egui::Context) -> Option<Worker> {
    let (jobs, job_rx) = mpsc::channel::<Job>();
    let (result_tx, results) = mpsc::channel();
    let ctx = ctx.clone();
    let spawned = std::thread::Builder::new()
        .name("shadow-layers".into())
        .spawn(move || {
            // The first job always brings fonts.
            let mut fonts: Option<FontLibrary> = None;
            let mut cache = DrawCache::default();
            while let Ok(job) = job_rx.recv() {
                if let Some(new) = job.fonts {
                    fonts = Some(new);
                    cache = DrawCache::default();
                }
                let Some(fonts) = fonts.as_mut() else {
                    break;
                };
                let done: Vec<Done> = job
                    .items
                    .into_iter()
                    .map(|item| Done {
                        id: item.id,
                        layer: render(&item, fonts, &mut cache),
                        key: item.key,
                    })
                    .collect();
                if result_tx.send(done).is_err() {
                    break;
                }
                ctx.request_repaint();
            }
        });
    match spawned {
        Ok(_) => Some(Worker { jobs, results }),
        Err(err) => {
            tracing::error!(%err, "cannot render the shadows");
            None
        }
    }
}

#[cfg(test)]
mod tests;
