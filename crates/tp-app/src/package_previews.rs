//! Previews of installed vehicle packages for the Vehicle Library's detail
//! pane: the package's preview image when it has one, otherwise the
//! template of its first main texture. A package is read in a background
//! thread the first time its preview is asked for, and kept for the
//! session (an installed version never changes).

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};

use egui::{ColorImage, TextureHandle, TextureOptions};
use tp_core::AssetKind;
use tp_vehicles::{ImageKind, Manifest, Package};

/// Longest side of a preview, in pixels.
pub const SIDE: u32 = 512;

/// Largest preview image read from a package, in bytes.
const MAX_PREVIEW_BYTES: u64 = 32 << 20;

/// Delay before the UI picks up a finished preview.
const REPAINT_DELAY: std::time::Duration = std::time::Duration::from_millis(50);

/// An installed version: its id and version.
type Key = (String, semver::Version);

/// What the dialog can show for a version.
pub enum Preview<'a> {
    Loading,
    Ready(&'a TextureHandle),
    /// Neither a preview image nor a template could be read.
    None,
}

/// Previews of installed versions.
#[derive(Default)]
pub struct PackagePreviews {
    shown: HashMap<Key, Option<TextureHandle>>,
    job: Option<(Key, Receiver<Option<ColorImage>>)>,
}

impl PackagePreviews {
    /// Picks up a finished preview, and starts reading `m`'s from the
    /// package file at `path` when it has none yet (one at a time).
    pub fn update(&mut self, ctx: &egui::Context, m: &Manifest, path: &Path) {
        if let Some((key, job)) = &self.job {
            match job.try_recv() {
                Ok(image) => {
                    let texture = image.map(|image| {
                        ctx.load_texture(
                            format!("package_preview_{}_{}", key.0, key.1),
                            image,
                            TextureOptions::LINEAR,
                        )
                    });
                    self.shown.insert(key.clone(), texture);
                    self.job = None;
                }
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => {
                    self.shown.insert(key.clone(), None);
                    self.job = None;
                }
            }
        }
        let key = (m.id.clone(), m.version.clone());
        if !self.shown.contains_key(&key) {
            self.job = start(ctx, path.to_path_buf()).map(|rx| (key, rx));
        }
    }

    /// The preview of version `version` of `id`.
    pub fn get(&self, id: &str, version: &semver::Version) -> Preview<'_> {
        match self.shown.get(&(id.to_owned(), version.clone())) {
            Some(Some(texture)) => Preview::Ready(texture),
            Some(None) => Preview::None,
            None => Preview::Loading,
        }
    }

    /// Whether a package is being read.
    pub fn is_reading(&self) -> bool {
        self.job.is_some()
    }
}

/// Reads the preview of the package at `path` on a worker thread.
fn start(ctx: &egui::Context, path: PathBuf) -> Option<Receiver<Option<ColorImage>>> {
    let (tx, rx) = mpsc::channel();
    let ctx = ctx.clone();
    let spawned = std::thread::Builder::new()
        .name("package-preview".into())
        .spawn(move || {
            let image = std::fs::read(&path).ok().and_then(|bytes| preview(&bytes));
            let _ = tx.send(image);
            ctx.request_repaint_after(REPAINT_DELAY);
        });
    match spawned {
        Ok(_) => Some(rx),
        Err(err) => {
            tracing::error!(%err, "cannot read the vehicle preview");
            None
        }
    }
}

/// The preview of a package file: its preview image, or else the template
/// of its first main texture.
pub fn preview(bytes: &[u8]) -> Option<ColorImage> {
    let package = Package::read(bytes).ok()?;
    let from_preview = package
        .manifest
        .preview
        .as_deref()
        .filter(|name| tp_vehicles::is_safe_path(name))
        .and_then(|name| {
            let image = entry(bytes, name)?;
            crate::image_cache::image_of(kind_of_name(name), &image, SIDE)
        });
    from_preview.or_else(|| {
        let main = package.manifest.paint_job.main.first()?;
        let template = package.template(&main.id)?;
        let kind = match template.kind {
            ImageKind::Png => AssetKind::Raster,
            ImageKind::Svg => AssetKind::Svg,
        };
        crate::image_cache::image_of(kind, &template.bytes, SIDE)
    })
}

/// An SVG preview by its extension, a raster image otherwise.
fn kind_of_name(name: &str) -> AssetKind {
    if name.to_ascii_lowercase().ends_with(".svg") {
        AssetKind::Svg
    } else {
        AssetKind::Raster
    }
}

/// The bytes of entry `name` of the package archive, if not too large.
fn entry(bytes: &[u8], name: &str) -> Option<Vec<u8>> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).ok()?;
    let file = zip.by_name(name).ok()?;
    if file.size() > MAX_PREVIEW_BYTES {
        return None;
    }
    let mut out = Vec::new();
    file.take(MAX_PREVIEW_BYTES).read_to_end(&mut out).ok()?;
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_main_template_without_a_preview_image() {
        let image = preview(crate::vehicles::SAMPLES[0].bytes).expect("template preview");
        assert!(image.size[0] > 0 && image.size[0].max(image.size[1]) <= SIDE as usize);
    }
}
