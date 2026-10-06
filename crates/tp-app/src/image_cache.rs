//! Display textures of image assets: raster images decoded once (downscaled
//! for display) and SVGs rendered in a background thread at the size the
//! current zoom needs.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use egui::{ColorImage, TextureHandle, TextureOptions};
use resvg::{tiny_skia, usvg};
use tp_core::document::{AssetId, Rgba};
use tp_core::{Asset, AssetKind};

use crate::import::svg_options;

/// Longest side of raster display copies.
pub const MAX_RASTER_SIDE: u32 = 2048;
/// Smallest and largest SVG render sizes (longest side).
const MIN_SVG_SIDE: u32 = 64;
const MAX_SVG_SIDE: u32 = 4096;
/// SVGs are first rendered at this size synchronously so they never show
/// up empty; sharper versions follow from the worker.
const FIRST_SVG_SIDE: u32 = 512;

fn texture_options() -> TextureOptions {
    TextureOptions {
        magnification: egui::TextureFilter::Linear,
        minification: egui::TextureFilter::Linear,
        wrap_mode: egui::TextureWrapMode::ClampToEdge,
        mipmap_mode: Some(egui::TextureFilter::Linear),
    }
}

/// SVG render size for an on-screen size in physical pixels: the next power
/// of two, clamped.
pub fn svg_bucket(screen_px: f32) -> u32 {
    (screen_px.max(1.0).ceil() as u32)
        .next_power_of_two()
        .clamp(MIN_SVG_SIDE, MAX_SVG_SIDE)
}

struct Raster {
    side: u32,
    texture: TextureHandle,
    pixels: Arc<ColorImage>,
}

#[derive(Default)]
struct Entry {
    raster: Option<Raster>,
    /// Size being rendered by the worker.
    pending: Option<u32>,
    svg: Option<Arc<usvg::Tree>>,
    failed: bool,
    used: bool,
}

struct Job {
    asset: AssetId,
    tree: Arc<usvg::Tree>,
    side: u32,
}

struct Worker {
    jobs: Sender<Job>,
    results: Receiver<(AssetId, u32, ColorImage)>,
}

/// Textures of assets, keyed by asset.
#[derive(Default)]
pub struct ImageCache {
    entries: HashMap<AssetId, Entry>,
    worker: Option<Worker>,
}

fn render_svg(tree: &usvg::Tree, side: u32) -> Option<ColorImage> {
    let size = tree.size();
    let scale = side as f32 / size.width().max(size.height());
    let w = (size.width() * scale).round().max(1.0) as u32;
    let h = (size.height() * scale).round().max(1.0) as u32;
    let mut pixmap = tiny_skia::Pixmap::new(w, h)?;
    resvg::render(
        tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    Some(ColorImage::from_rgba_premultiplied(
        [w as usize, h as usize],
        pixmap.data(),
    ))
}

fn decode_raster(bytes: &[u8], max_side: u32) -> Option<ColorImage> {
    let image = image::load_from_memory(bytes).ok()?;
    let limit = MAX_RASTER_SIDE.min(max_side);
    let image = if image.width().max(image.height()) > limit {
        image.resize(limit, limit, image::imageops::FilterType::Triangle)
    } else {
        image
    };
    let rgba = image.to_rgba8();
    Some(ColorImage::from_rgba_unmultiplied(
        [rgba.width() as usize, rgba.height() as usize],
        rgba.as_raw(),
    ))
}

impl ImageCache {
    fn worker(&mut self, ctx: &egui::Context) -> &Worker {
        self.worker.get_or_insert_with(|| {
            let (jobs, job_rx) = mpsc::channel::<Job>();
            let (result_tx, results) = mpsc::channel();
            let ctx = ctx.clone();
            let spawned = thread::Builder::new()
                .name("svg-render".into())
                .spawn(move || {
                    while let Ok(job) = job_rx.recv() {
                        if let Some(image) = render_svg(&job.tree, job.side)
                            && result_tx.send((job.asset, job.side, image)).is_err()
                        {
                            break;
                        }
                        ctx.request_repaint();
                    }
                });
            if let Err(err) = spawned {
                tracing::error!(%err, "cannot start the SVG renderer");
            }
            Worker { jobs, results }
        })
    }

    /// Uploads SVG renders finished by the worker.
    pub fn poll(&mut self, ctx: &egui::Context) {
        let Some(worker) = &self.worker else {
            return;
        };
        let done: Vec<_> = worker.results.try_iter().collect();
        for (asset, side, image) in done {
            let Some(entry) = self.entries.get_mut(&asset) else {
                continue;
            };
            if entry.pending == Some(side) {
                entry.pending = None;
            }
            let texture = ctx.load_texture(
                format!("asset-{}-{side}", asset.0),
                image.clone(),
                texture_options(),
            );
            entry.raster = Some(Raster {
                side,
                texture,
                pixels: Arc::new(image),
            });
        }
    }

    /// The texture to draw `asset` at `screen_px` (longest on-screen side in
    /// physical pixels). `None` while nothing can be shown yet, or when the
    /// asset cannot be decoded (see [`Self::failed`]).
    pub fn texture(
        &mut self,
        ctx: &egui::Context,
        asset: &Asset,
        screen_px: f32,
    ) -> Option<TextureHandle> {
        // The GPU's limit, rounded down to a power of two.
        let max_side = ctx.input(|i| i.max_texture_side).max(1) as u32;
        let max_side = 1 << (31 - max_side.leading_zeros());
        let entry = self.entries.entry(asset.id).or_default();
        entry.used = true;
        if entry.failed {
            return None;
        }
        match asset.kind {
            AssetKind::Raster => {
                if entry.raster.is_none() {
                    match decode_raster(&asset.bytes, max_side) {
                        Some(image) => {
                            let side = image.width().max(image.height()) as u32;
                            let texture = ctx.load_texture(
                                format!("asset-{}", asset.id.0),
                                image.clone(),
                                texture_options(),
                            );
                            entry.raster = Some(Raster {
                                side,
                                texture,
                                pixels: Arc::new(image),
                            });
                        }
                        None => {
                            entry.failed = true;
                            return None;
                        }
                    }
                }
            }
            AssetKind::Svg => {
                if entry.svg.is_none() {
                    match usvg::Tree::from_data(&asset.bytes, svg_options()) {
                        Ok(tree) => entry.svg = Some(Arc::new(tree)),
                        Err(_) => {
                            entry.failed = true;
                            return None;
                        }
                    }
                }
                let tree = entry.svg.clone().expect("parsed above");
                let wanted = svg_bucket(screen_px).min(max_side);
                if entry.raster.is_none() {
                    let side = wanted.min(FIRST_SVG_SIDE);
                    if let Some(image) = render_svg(&tree, side) {
                        let texture = ctx.load_texture(
                            format!("asset-{}-{side}", asset.id.0),
                            image.clone(),
                            texture_options(),
                        );
                        entry.raster = Some(Raster {
                            side,
                            texture,
                            pixels: Arc::new(image),
                        });
                    }
                }
                let current = entry.raster.as_ref().map_or(0, |r| r.side);
                if wanted > current && entry.pending.is_none_or(|p| p < wanted) {
                    entry.pending = Some(wanted);
                    let id = asset.id;
                    let _ = self.worker(ctx).jobs.send(Job {
                        asset: id,
                        tree,
                        side: wanted,
                    });
                }
            }
        }
        let entry = self.entries.get(&asset.id)?;
        entry.raster.as_ref().map(|r| r.texture.clone())
    }

    pub fn failed(&self, asset: AssetId) -> bool {
        self.entries.get(&asset).is_some_and(|e| e.failed)
    }

    /// Whether an SVG render is still running.
    pub fn is_rendering(&self) -> bool {
        self.entries.values().any(|e| e.pending.is_some())
    }

    /// Color at `uv` (0..1) of the displayed copy of `asset`.
    pub fn sample(&self, asset: AssetId, uv: [f64; 2]) -> Option<Rgba> {
        let raster = self.entries.get(&asset)?.raster.as_ref()?;
        let [w, h] = raster.pixels.size;
        let x = ((uv[0] * w as f64).floor() as usize).min(w.saturating_sub(1));
        let y = ((uv[1] * h as f64).floor() as usize).min(h.saturating_sub(1));
        let c = raster.pixels.pixels.get(y * w + x)?.to_srgba_unmultiplied();
        Some(Rgba {
            r: c[0],
            g: c[1],
            b: c[2],
            a: c[3],
        })
    }

    /// Drops textures of assets not drawn since the previous call.
    pub fn prune(&mut self) {
        self.entries.retain(|_, e| std::mem::take(&mut e.used));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_buckets_are_powers_of_two() {
        assert_eq!(svg_bucket(10.0), 64);
        assert_eq!(svg_bucket(300.0), 512);
        assert_eq!(svg_bucket(512.0), 512);
        assert_eq!(svg_bucket(100_000.0), 4096);
    }

    #[test]
    fn svg_renders_premultiplied_pixels() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10"><rect width="20" height="10" fill="#0000ff"/></svg>"##;
        let tree = usvg::Tree::from_data(svg, svg_options()).unwrap();
        let image = render_svg(&tree, 64).unwrap();
        assert_eq!(image.size, [64, 32]);
        assert_eq!(image.pixels[0].to_srgba_unmultiplied(), [0, 0, 255, 255]);
    }
}
