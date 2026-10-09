//! Previews of the mod's pictures, the shop icon and the Mod Manager image,
//! as the mod export makes them (see [`crate::mod_export::picture`]): for
//! the Project space, rendered in a background thread and kept until the
//! first texture's artwork, the fonts or the chosen pictures change; and
//! the rendering shared with the Export Mod dialog's previews.
//!
//! Staleness is told as for the surface thumbnails: the previews remember
//! the first texture's top-level objects, and any edit replaces at least one
//! of their `Arc`s.

use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};

use egui::{ColorImage, TextureHandle, TextureOptions};
use tp_core::Project;
use tp_core::document::{AssetId, Object};
use tp_text::FontLibrary;

use crate::mod_export::{ICON_SIZE, IMAGE_SIZE, Picture};

/// Delay before the UI picks up finished previews.
const REPAINT_DELAY: std::time::Duration = std::time::Duration::from_millis(50);

/// Renders the shop icon `icon` and the Mod Manager image `image` of
/// `project` (a blank picture when one can't be made).
pub fn render(
    project: &Project,
    icon: &Picture,
    image: &Picture,
    fonts: &mut FontLibrary,
) -> [ColorImage; 2] {
    let mut render = |picture: &Picture, size: (u32, u32)| {
        let pixmap = crate::mod_export::picture(project, picture.bytes(project), size, fonts)
            .unwrap_or_else(|_| tp_render::Pixmap::new(size.0, size.1).expect("size"));
        let rgba = tp_render::to_rgba(&pixmap);
        ColorImage::from_rgba_unmultiplied([size.0 as usize, size.1 as usize], rgba.as_raw())
    };
    [render(icon, ICON_SIZE), render(image, IMAGE_SIZE)]
}

/// What the previews were rendered from.
#[derive(Clone)]
struct Key {
    /// The first texture's top-level objects and size (generated pictures).
    objects: Vec<Arc<Object>>,
    size: f64,
    fonts: u64,
    /// The chosen pictures.
    icon: Option<AssetId>,
    image: Option<AssetId>,
}

impl Key {
    fn of(project: &Project, fonts: u64) -> Self {
        let (objects, size) = project
            .surfaces
            .first()
            .map_or((Vec::new(), 0.0), |s| (s.objects.clone(), s.size));
        Self {
            objects,
            size,
            fonts,
            icon: project.mod_settings.icon,
            image: project.mod_settings.image,
        }
    }

    fn same(&self, other: &Self) -> bool {
        self.size == other.size
            && self.fonts == other.fonts
            && self.icon == other.icon
            && self.image == other.image
            && self.objects.len() == other.objects.len()
            && self
                .objects
                .iter()
                .zip(&other.objects)
                .all(|(a, b)| Arc::ptr_eq(a, b))
    }
}

/// The mod's pictures of an open project.
#[derive(Default)]
pub struct ModPreviews {
    shown: Option<(Key, [TextureHandle; 2])>,
    job: Option<(Key, Receiver<[ColorImage; 2]>)>,
    /// The renderer stopped (it could not start, or panicked): no retry.
    broken: bool,
    /// Number of renders, for tests and diagnostics.
    pub renders: u64,
}

impl ModPreviews {
    /// Picks up a finished render, then starts one when the shown previews
    /// are missing or stale. Nothing starts while a pointer button is down.
    pub fn update(&mut self, ctx: &egui::Context, project: &Project, fonts: &FontLibrary) {
        if let Some((key, job)) = &self.job {
            match job.try_recv() {
                Ok([icon, image]) => {
                    let options = TextureOptions::LINEAR;
                    let textures = [
                        ctx.load_texture("project_mod_icon", icon, options),
                        ctx.load_texture("project_mod_image", image, options),
                    ];
                    self.shown = Some((key.clone(), textures));
                    self.job = None;
                }
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => {
                    self.job = None;
                    self.broken = true;
                }
            }
        }
        if self.broken || ctx.input(|i| i.pointer.any_down()) {
            return;
        }
        let key = Key::of(project, fonts.generation());
        if self.shown.as_ref().is_some_and(|(k, _)| k.same(&key)) {
            return;
        }
        self.renders += 1;
        let (tx, done) = mpsc::channel();
        let project = project.clone();
        let mut fonts = fonts.fork();
        let repaint = ctx.clone();
        let spawned = std::thread::Builder::new()
            .name("project-mod-pictures".into())
            .spawn(move || {
                let icon = Picture::of(project.mod_settings.icon);
                let image = Picture::of(project.mod_settings.image);
                let _ = tx.send(render(&project, &icon, &image, &mut fonts));
                // Shown at the next frame, without waking the UI at once,
                // as the surface thumbnails.
                repaint.request_repaint_after(REPAINT_DELAY);
            });
        match spawned {
            Ok(_) => self.job = Some((key, done)),
            Err(err) => {
                tracing::error!(%err, "cannot render the mod's pictures");
                self.broken = true;
            }
        }
    }

    /// The shop icon and the Mod Manager image, once rendered (the previous
    /// ones stay shown while they are rendered again).
    pub fn textures(&self) -> Option<&[TextureHandle; 2]> {
        self.shown.as_ref().map(|(_, t)| t)
    }

    /// Whether the previews are being rendered.
    pub fn is_rendering(&self) -> bool {
        self.job.is_some()
    }
}
