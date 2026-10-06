//! Texture export: settings, background export job and preview renders.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use tp_core::Project;
use tp_core::document::Rgba;
use tp_render::{DdsEncoding, RenderOptions};
use tp_text::FontLibrary;

/// Output file format.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ExportFormat {
    #[default]
    Png,
    Dds,
}

impl ExportFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Dds => "dds",
        }
    }
}

/// Export choices, remembered for the session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExportSettings {
    pub format: ExportFormat,
    /// 1 (full size), 2 or 4.
    pub divisor: u32,
    /// `None`: transparent.
    pub background: Option<Rgba>,
    pub dds: DdsEncoding,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            format: ExportFormat::Png,
            divisor: 1,
            background: Some(Rgba::rgb(255, 255, 255)),
            dds: DdsEncoding::Bc3,
        }
    }
}

impl ExportSettings {
    /// Output side in pixels for a texture of side `side`.
    pub fn size(&self, side: u32) -> u32 {
        (side / self.divisor.max(1)).max(1)
    }

    /// Estimated file size in bytes, and whether it is exact.
    pub fn estimated_bytes(&self, side: u32) -> (u64, bool) {
        let size = self.size(side);
        match self.format {
            // PNG compresses livery artwork well; a rough average.
            ExportFormat::Png => (u64::from(size) * u64::from(size) * 2, false),
            ExportFormat::Dds => (tp_render::dds_size(size, self.dds), true),
        }
    }

    /// File name proposed for a project named `name`.
    pub fn suggested_name(&self, name: &str) -> String {
        let stem = crate::file_dialogs::suggested_file_name(name);
        let stem = stem.trim_end_matches(&format!(".{}", tp_file::EXTENSION));
        format!("{stem}.{}", self.format.extension())
    }
}

/// "12.4 MB", "about 32 MB".
pub fn size_text(bytes: u64, exact: bool) -> String {
    let mb = bytes as f64 / (1024.0 * 1024.0);
    let value = if mb >= 10.0 {
        format!("{mb:.0} MB")
    } else if mb >= 0.1 {
        format!("{mb:.1} MB")
    } else {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    };
    if exact {
        value
    } else {
        format!("about {value}")
    }
}

/// How an export ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportOutcome {
    Written(PathBuf),
    Cancelled,
    Failed { path: PathBuf, reason: String },
}

/// An export running on a worker thread.
pub struct ExportJob {
    pub path: PathBuf,
    progress: Arc<AtomicU32>,
    cancel: Arc<AtomicBool>,
    done: Receiver<ExportOutcome>,
}

/// Renders and encodes the texture (worker side).
fn run_export(
    project: &Project,
    settings: ExportSettings,
    mut fonts: FontLibrary,
    path: &std::path::Path,
    progress: &AtomicU32,
    cancel: &AtomicBool,
    notify: &(dyn Fn() + Send + Sync),
) -> ExportOutcome {
    let set = |permille: u32| {
        progress.store(permille, Ordering::Relaxed);
        notify();
    };
    let failed = |reason: String| ExportOutcome::Failed {
        path: path.to_path_buf(),
        reason,
    };
    let options = RenderOptions {
        size: settings.size(project.resolution.side()),
        background: settings.background,
    };
    let rendered = tp_render::render(
        project,
        project.active_surface,
        options,
        &mut fonts,
        &mut |done, total| {
            set((700 * done / total.max(1)) as u32);
            !cancel.load(Ordering::Relaxed)
        },
    );
    let Ok(pixmap) = rendered else {
        return ExportOutcome::Cancelled;
    };
    let bytes = match settings.format {
        ExportFormat::Png => match tp_render::encode_png(&pixmap) {
            Ok(bytes) => bytes,
            Err(reason) => return failed(reason),
        },
        ExportFormat::Dds => {
            let encoded = tp_render::encode_dds(&pixmap, settings.dds, &mut |level, levels| {
                set(700 + (250 * level / levels.max(1)) as u32);
                !cancel.load(Ordering::Relaxed)
            });
            match encoded {
                Ok(bytes) => bytes,
                Err(_) => return ExportOutcome::Cancelled,
            }
        }
    };
    drop(pixmap);
    if cancel.load(Ordering::Relaxed) {
        return ExportOutcome::Cancelled;
    }
    set(950);
    match tp_file::write_atomic(path, &bytes) {
        Ok(()) => {
            set(1000);
            ExportOutcome::Written(path.to_path_buf())
        }
        Err(err) => failed(match err {
            tp_file::Error::Io(e) => e.to_string(),
            other => other.to_string(),
        }),
    }
}

impl ExportJob {
    /// Starts exporting `project` (a snapshot: editing may continue).
    pub fn start(
        project: Project,
        settings: ExportSettings,
        fonts: FontLibrary,
        path: PathBuf,
        notify: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        let progress = Arc::new(AtomicU32::new(0));
        let cancel = Arc::new(AtomicBool::new(false));
        let (tx, done) = mpsc::channel();
        let (p, c, target) = (progress.clone(), cancel.clone(), path.clone());
        let spawned = thread::Builder::new()
            .name("texture-export".into())
            .spawn(move || {
                let outcome = run_export(&project, settings, fonts, &target, &p, &c, &notify);
                let _ = tx.send(outcome);
                notify();
            });
        if let Err(err) = spawned {
            tracing::error!(%err, "cannot start the export");
        }
        Self {
            path,
            progress,
            cancel,
            done,
        }
    }

    /// Progress from 0.0 to 1.0.
    pub fn progress(&self) -> f32 {
        self.progress.load(Ordering::Relaxed) as f32 / 1000.0
    }

    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    /// The outcome once finished.
    pub fn poll(&self) -> Option<ExportOutcome> {
        match self.done.try_recv() {
            Ok(outcome) => Some(outcome),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(ExportOutcome::Failed {
                path: self.path.clone(),
                reason: "the export stopped unexpectedly".into(),
            }),
        }
    }

    /// Blocks until the export ends (tests).
    pub fn wait(&self) -> ExportOutcome {
        self.done.recv().unwrap_or(ExportOutcome::Failed {
            path: self.path.clone(),
            reason: "the export stopped unexpectedly".into(),
        })
    }
}

/// Side of preview renders.
pub const PREVIEW_SIDE: u32 = 512;

/// A preview render on a worker thread.
pub struct PreviewJob {
    pub settings: ExportSettings,
    done: Receiver<egui::ColorImage>,
}

impl PreviewJob {
    pub fn start(
        project: Project,
        settings: ExportSettings,
        fonts: FontLibrary,
        notify: impl Fn() + Send + 'static,
    ) -> Self {
        let (tx, done) = mpsc::channel();
        let _ = thread::Builder::new()
            .name("export-preview".into())
            .spawn(move || {
                let mut fonts = fonts;
                let size = settings.size(project.resolution.side()).min(PREVIEW_SIDE);
                let options = RenderOptions {
                    size,
                    background: settings.background,
                };
                if let Ok(pixmap) = tp_render::render(
                    &project,
                    project.active_surface,
                    options,
                    &mut fonts,
                    &mut |_, _| true,
                ) {
                    let image = egui::ColorImage::from_rgba_premultiplied(
                        [pixmap.width() as usize, pixmap.height() as usize],
                        pixmap.data(),
                    );
                    let _ = tx.send(image);
                    notify();
                }
            });
        Self { settings, done }
    }

    pub fn poll(&self) -> Option<egui::ColorImage> {
        self.done.try_recv().ok()
    }
}

#[cfg(test)]
mod tests {
    use tp_core::TextureResolution;
    use tp_core::document::{Frame, Object, ObjectId, ShapeKind};
    use tp_core::kurbo::{Point, Size};

    use super::*;

    fn project() -> Project {
        let mut p = Project::new("ACE", TextureResolution::R2048);
        p.add(Object::new(
            ObjectId(0),
            ShapeKind::Ellipse,
            Frame::new(Point::new(1000.0, 1000.0), Size::new(800.0, 400.0), 0.0),
        ));
        p
    }

    #[test]
    fn settings_sizes_and_names() {
        let s = ExportSettings {
            divisor: 2,
            format: ExportFormat::Dds,
            ..ExportSettings::default()
        };
        assert_eq!(s.size(4096), 2048);
        assert_eq!(s.suggested_name("ACE Logistics"), "ACE Logistics.dds");
        let (bytes, exact) = s.estimated_bytes(4096);
        assert!(exact && bytes > 2048 * 2048);
        assert_eq!(size_text(5 * 1024 * 1024, true), "5.0 MB");
        assert_eq!(size_text(32 * 1024 * 1024, false), "about 32 MB");
    }

    #[test]
    fn completed_export_writes_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ace.png");
        let settings = ExportSettings {
            divisor: 4,
            ..ExportSettings::default()
        };
        let job = ExportJob::start(
            project(),
            settings,
            FontLibrary::bundled(),
            path.clone(),
            || {},
        );
        assert_eq!(job.wait(), ExportOutcome::Written(path.to_path_buf()));
        assert_eq!(image::image_dimensions(&path).unwrap(), (512, 512));
        assert!((job.progress() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cancelled_export_leaves_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ace.dds");
        let settings = ExportSettings {
            format: ExportFormat::Dds,
            ..ExportSettings::default()
        };
        let job = ExportJob::start(
            project(),
            settings,
            FontLibrary::bundled(),
            path.clone(),
            || {},
        );
        job.cancel();
        assert_eq!(job.wait(), ExportOutcome::Cancelled);
        assert!(std::fs::read_dir(dir.path()).unwrap().next().is_none());
    }

    #[test]
    fn failing_destination_is_reported() {
        let path = PathBuf::from("/nonexistent-dir/ace.png");
        let settings = ExportSettings {
            divisor: 4,
            ..ExportSettings::default()
        };
        let job = ExportJob::start(
            project(),
            settings,
            FontLibrary::bundled(),
            path.clone(),
            || {},
        );
        assert!(matches!(job.wait(), ExportOutcome::Failed { path: p, .. } if p == path));
    }
}
