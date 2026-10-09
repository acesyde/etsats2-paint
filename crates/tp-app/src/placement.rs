//! Placing imported files and assets as image objects.

use tp_core::document::{AssetId, Frame, Object, ObjectId, ShapeKind};
use tp_core::kurbo::{Point, Size};
use tp_i18n::tr;

use crate::import::{ImportError, ImportedFile};
use crate::workspace::Workspace;

/// Offset between images placed together, in texture pixels.
pub const PLACE_OFFSET: f64 = 40.0;

/// Natural size scaled down to fit within half of the surface.
pub fn placement_size(natural: Size, surface: f64) -> Size {
    let limit = surface / 2.0;
    let scale = (limit / natural.width).min(limit / natural.height).min(1.0);
    Size::new(natural.width * scale, natural.height * scale)
}

impl Workspace {
    /// Center of the canvas view in document space (or of the surface).
    pub fn view_center(&self) -> Point {
        self.viewport.map_or_else(
            || {
                let s = self.project.surface().size / 2.0;
                Point::new(s, s)
            },
            |v| v.center,
        )
    }

    fn image_object(&self, asset: AssetId, center: Point) -> Option<Object> {
        let a = self.project.assets.get(&asset)?;
        let size = placement_size(a.size, self.project.surface().size);
        let mut object = Object::new(
            ObjectId(0),
            ShapeKind::Image { asset },
            Frame::new(center, size, 0.0),
        );
        object.name = a.name.clone();
        Some(object)
    }

    /// Imports files as assets and places one image object per file around
    /// `at` (or the view center), selected, as one "Place" step. Returns the
    /// errors of files that could not be imported.
    pub fn place_files(
        &mut self,
        files: Vec<Result<ImportedFile, ImportError>>,
        at: Option<Point>,
        now: f64,
    ) -> Vec<ImportError> {
        self.end_text_session(now);
        self.commit_pending(now);
        let (ok, errors): (Vec<_>, Vec<_>) = files.into_iter().partition(Result::is_ok);
        let errors: Vec<ImportError> = errors.into_iter().filter_map(Result::err).collect();
        let files: Vec<ImportedFile> = ok.into_iter().filter_map(Result::ok).collect();
        if !files.is_empty() {
            let center = at.unwrap_or_else(|| self.view_center());
            let layer = self.active_layer();
            let before = self.snapshot();
            let mut placed = Vec::new();
            for (i, file) in files.into_iter().enumerate() {
                let (asset, _) = self
                    .project
                    .add_asset(&file.name, file.kind, file.bytes, file.size);
                let offset = PLACE_OFFSET * i as f64;
                if let Some(object) =
                    self.image_object(asset, center + tp_core::kurbo::Vec2::new(offset, offset))
                {
                    placed.push(self.project.add_to(layer, object));
                }
            }
            self.selection = placed;
            self.record("undo-place", before, now, false);
            // Imported from the Brand space: shown placed and selected.
            if self.space == crate::layout::Space::Brand {
                self.space = crate::layout::Space::Workshop;
            }
        }
        if let Some(first) = errors.first() {
            let more = errors.len() - 1;
            let text = if more == 0 {
                first.to_string()
            } else {
                tr!("hint-import-errors", first = first.to_string(), more = more)
            };
            self.show_hint(text, now);
        }
        errors
    }

    /// Places an existing asset again at `at` (or the view center).
    pub fn place_asset(&mut self, asset: AssetId, at: Option<Point>, now: f64) {
        self.end_text_session(now);
        let center = at.unwrap_or_else(|| self.view_center());
        let Some(object) = self.image_object(asset, center) else {
            return;
        };
        let layer = self.active_layer();
        self.edit("undo-place", now, false, |project, selection| {
            *selection = vec![project.add_to(layer, object)];
        });
    }

    pub fn rename_asset(&mut self, asset: AssetId, name: &str, now: f64) {
        let name = name.trim().to_owned();
        if name.is_empty() {
            return;
        }
        self.edit("undo-rename-asset", now, false, |project, _| {
            project.rename_asset(asset, &name);
        });
    }

    pub fn remove_asset(&mut self, asset: AssetId, now: f64) {
        self.edit("undo-remove-asset", now, false, |project, _| {
            project.remove_asset(asset);
        });
    }

    /// Restores the placement size of the selected images, keeping their
    /// center and rotation.
    pub fn reset_image_size(&mut self, now: f64) {
        let surface = self.project.surface().size;
        let mut objects = self.selected_objects();
        for o in &mut objects {
            if let ShapeKind::Image { asset } = o.kind
                && let Some(a) = self.project.assets.get(&asset)
            {
                o.frame.size = placement_size(a.size, surface);
            }
        }
        self.edit("undo-reset-size", now, false, |project, _| {
            project.surface_mut().replace(&objects);
        });
    }
}

#[cfg(test)]
mod tests {
    use tp_core::{Project, TextureResolution};

    use super::*;
    use crate::import::{read_bytes, solid_png};

    fn ws(resolution: TextureResolution) -> Workspace {
        Workspace::new(Project::new("t", resolution))
    }

    #[test]
    fn large_images_are_scaled_to_half_the_surface() {
        assert_eq!(
            placement_size(Size::new(6000.0, 3000.0), 4096.0),
            Size::new(2048.0, 1024.0)
        );
        assert_eq!(
            placement_size(Size::new(800.0, 400.0), 4096.0),
            Size::new(800.0, 400.0)
        );
    }

    #[test]
    fn place_dedup_and_undo() {
        let mut ws = ws(TextureResolution::R4096);
        let png = solid_png(80, 40, [0, 0, 255, 255]);
        let files = vec![
            read_bytes("logo.png", png.clone()),
            read_bytes("anim.gif", b"GIF89a".to_vec()),
        ];
        let errors = ws.place_files(files, Some(Point::new(1000.0, 1000.0)), 0.0);
        assert_eq!(errors.len(), 1);
        assert_eq!(
            ws.hint.as_ref().unwrap().text,
            "anim.gif: unsupported format"
        );
        let image = &ws.selected_objects()[0];
        assert_eq!(image.name, "logo");
        assert_eq!(image.frame.size, Size::new(80.0, 40.0));
        assert_eq!(ws.history.undo_label(), Some("undo-place"));

        ws.place_files(vec![read_bytes("logo.png", png)], None, 1.0);
        assert_eq!(ws.project.assets.len(), 1);
        let asset = *ws.project.assets.keys().next().unwrap();
        assert_eq!(ws.project.asset_usage(asset), 2);

        ws.undo();
        ws.undo();
        assert!(ws.project.assets.is_empty());
        assert!(ws.project.surface().objects.is_empty());
    }

    #[test]
    fn reset_size_keeps_center() {
        let mut ws = ws(TextureResolution::R4096);
        let png = solid_png(800, 400, [255, 0, 0, 255]);
        ws.place_files(
            vec![read_bytes("logo.png", png)],
            Some(Point::new(1000.0, 900.0)),
            0.0,
        );
        let mut o = ws.selected_objects()[0].clone();
        o.frame.size = Size::new(800.0, 800.0);
        o.frame.rotation_deg = 15.0;
        ws.project.surface_mut().replace(&[o]);
        ws.reset_image_size(1.0);
        let o = &ws.selected_objects()[0];
        assert_eq!(o.frame.size, Size::new(800.0, 400.0));
        assert_eq!(o.frame.center, Point::new(1000.0, 900.0));
        assert_eq!(o.frame.rotation_deg, 15.0);
    }
}
