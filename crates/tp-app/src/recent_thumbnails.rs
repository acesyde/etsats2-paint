//! Thumbnails of the recent projects on the home screen: the artwork of a
//! project's first main texture drawn over its template, as the Project
//! space shows it.
//!
//! The files are read and rendered in a background thread, never at
//! startup on the UI thread: a card shows its placeholder until its
//! thumbnail is ready, and keeps it when the file can't be read. A
//! thumbnail is kept for the session with the file's modification time,
//! and made again when the file changed since (checked each time the home
//! screen shows again).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::SystemTime;

use egui::{Color32, ColorImage, TextureHandle, TextureOptions};
use tp_core::document::Rgba;
use tp_core::{Project, TexturePart};
use tp_render::RenderOptions;
use tp_text::FontLibrary;

/// Side of a thumbnail, in pixels: sharp for a card's 120-point picture
/// up to 2×.
pub const SIDE: u32 = 256;

/// Delay before the UI picks up finished thumbnails.
const REPAINT_DELAY: std::time::Duration = std::time::Duration::from_millis(50);

/// A project's thumbnail and what it was made from.
struct Entry {
    modified: Option<SystemTime>,
    /// `None`: the file can't be read.
    texture: Option<TextureHandle>,
    image: Option<ColorImage>,
}

/// What the worker found for one file.
enum Read {
    /// Same modification time as the thumbnail shown.
    Unchanged,
    Done {
        path: PathBuf,
        modified: Option<SystemTime>,
        image: Option<ColorImage>,
    },
}

/// Thumbnails of the recent projects, by file.
#[derive(Default)]
pub struct RecentThumbnails {
    shown: HashMap<PathBuf, Entry>,
    job: Option<Receiver<Vec<Read>>>,
    /// The files shown were checked since the home screen last showed.
    checked: bool,
}

impl RecentThumbnails {
    /// Picks up finished thumbnails, then reads the files of `paths` that
    /// have none yet, or that may have changed since the home screen last
    /// showed.
    pub fn update(&mut self, ctx: &egui::Context, paths: &[&Path]) {
        if let Some(job) = &self.job {
            match job.try_recv() {
                Ok(reads) => {
                    for read in reads {
                        if let Read::Done {
                            path,
                            modified,
                            image,
                        } = read
                        {
                            let texture = image.clone().map(|image| {
                                ctx.load_texture(
                                    format!("recent_thumbnail_{}", path.display()),
                                    image,
                                    TextureOptions::LINEAR,
                                )
                            });
                            self.shown.insert(
                                path,
                                Entry {
                                    modified,
                                    texture,
                                    image,
                                },
                            );
                        }
                    }
                    self.job = None;
                }
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => self.job = None,
            }
        }
        let todo: Vec<(PathBuf, Option<SystemTime>)> = paths
            .iter()
            .filter(|p| !self.checked || !self.shown.contains_key(**p))
            .map(|p| {
                let modified = self.shown.get(*p).and_then(|e| e.modified);
                (p.to_path_buf(), modified)
            })
            .collect();
        self.checked = true;
        if !todo.is_empty() {
            self.job = start(ctx, todo);
        }
    }

    /// Checks the files again the next time they are shown (the home screen
    /// shows again: a project may have been saved meanwhile).
    pub fn invalidate(&mut self) {
        self.checked = false;
    }

    /// The thumbnail of `path`, once read (`None` too when it can't be).
    pub fn texture(&self, path: &Path) -> Option<&TextureHandle> {
        self.shown.get(path)?.texture.as_ref()
    }

    /// Whether files are being read.
    pub fn is_reading(&self) -> bool {
        self.job.is_some()
    }

    /// Color of the thumbnail of `path` at `uv` (0..1), for tests.
    pub fn color_at(&self, path: &Path, uv: [f32; 2]) -> Option<Rgba> {
        let image = self.shown.get(path)?.image.as_ref()?;
        let [w, h] = image.size;
        let x = ((uv[0] * w as f32) as usize).min(w.saturating_sub(1));
        let y = ((uv[1] * h as f32) as usize).min(h.saturating_sub(1));
        let [r, g, b, a] = image.pixels.get(y * w + x)?.to_srgba_unmultiplied();
        Some(Rgba::with_alpha(r, g, b, a))
    }
}

/// Reads and renders `files` on a worker thread; `None` when it can't
/// start.
fn start(
    ctx: &egui::Context,
    files: Vec<(PathBuf, Option<SystemTime>)>,
) -> Option<Receiver<Vec<Read>>> {
    let (tx, rx) = mpsc::channel();
    let ctx = ctx.clone();
    let spawned = std::thread::Builder::new()
        .name("recent-thumbnails".into())
        .spawn(move || {
            let mut fonts = FontLibrary::bundled();
            let reads = files
                .into_iter()
                .map(|(path, shown)| {
                    let modified = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
                    if modified.is_some() && modified == shown {
                        return Read::Unchanged;
                    }
                    let image = tp_file::read(&path)
                        .ok()
                        .and_then(|opened| thumbnail(&opened.project, &mut fonts));
                    Read::Done {
                        path,
                        modified,
                        image,
                    }
                })
                .collect();
            let _ = tx.send(reads);
            ctx.request_repaint_after(REPAINT_DELAY);
        });
    match spawned {
        Ok(_) => Some(rx),
        Err(err) => {
            tracing::error!(%err, "cannot read the recent projects' thumbnails");
            None
        }
    }
}

/// The artwork of `project`'s first main texture (its first texture when
/// none is a main texture) over its template, on the artboard color.
pub fn thumbnail(project: &Project, fonts: &mut FontLibrary) -> Option<ColorImage> {
    let surface = project
        .surfaces
        .iter()
        .position(|s| {
            s.template
                .as_ref()
                .is_some_and(|t| t.part == TexturePart::Main)
        })
        .or((!project.surfaces.is_empty()).then_some(0))?;
    let side = SIDE as usize;
    let mut pixels = vec![tp_ui::tokens::canvas::ARTBOARD; side * side];
    let template = project.surfaces[surface]
        .template
        .as_ref()
        .and_then(|t| project.assets.get(&t.asset))
        .and_then(|asset| crate::image_cache::asset_image(asset, SIDE));
    if let Some(template) = template {
        let [w, h] = template.size;
        for (i, pixel) in pixels.iter_mut().enumerate() {
            // Stretched to the square texture, as the editor shows it.
            let x = (i % side) * w / side;
            let y = (i / side) * h / side;
            *pixel = over(template.pixels[y * w + x], *pixel);
        }
    }
    let options = RenderOptions {
        size: SIDE,
        background: None,
    };
    let artwork = tp_render::render(project, surface, options, fonts, &mut |_, _| true).ok()?;
    let (chunks, _) = artwork.data().as_chunks::<4>();
    for (pixel, src) in pixels.iter_mut().zip(chunks) {
        let src = Color32::from_rgba_premultiplied(src[0], src[1], src[2], src[3]);
        *pixel = over(src, *pixel);
    }
    Some(ColorImage::new([side, side], pixels))
}

/// `src` drawn over `dst` (both premultiplied).
fn over(src: Color32, dst: Color32) -> Color32 {
    let keep = 255 - u32::from(src.a());
    let mix = |s: u8, d: u8| (u32::from(s) + (u32::from(d) * keep + 127) / 255).min(255) as u8;
    Color32::from_rgba_premultiplied(
        mix(src.r(), dst.r()),
        mix(src.g(), dst.g()),
        mix(src.b(), dst.b()),
        mix(src.a(), dst.a()),
    )
}

#[cfg(test)]
mod tests {
    use tp_core::document::{Frame, Object, ObjectId, Paint, ShapeKind};
    use tp_core::kurbo::{Point, Size};

    use super::*;

    #[test]
    fn the_artwork_is_drawn_over_the_artboard() {
        let mut p = Project::new("Test", tp_core::TextureResolution::default());
        let side = p.surface().size;
        let mut o = Object::new(
            ObjectId(0),
            ShapeKind::rectangle(),
            Frame::new(
                Point::new(side / 4.0, side / 2.0),
                Size::new(side / 2.0, side),
                0.0,
            ),
        );
        o.fill = Paint::Solid(Rgba::rgb(255, 0, 0));
        p.add(o);
        let image = thumbnail(&p, &mut FontLibrary::bundled()).unwrap();
        let at = |x: usize| image.pixels[(SIDE as usize / 2) * SIDE as usize + x];
        assert_eq!(at(10), Color32::from_rgb(255, 0, 0));
        assert_eq!(at(SIDE as usize - 10), tp_ui::tokens::canvas::ARTBOARD);
    }
}
