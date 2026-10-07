//! Projects made for a vehicle: one surface per texture with its template,
//! and moving a project to a newer version of its package.

use std::sync::Arc;

use tp_core::document::{Frame, Handle, Object, ResizeOptions, resize};
use tp_core::kurbo::{Point, Rect, Size};
use tp_core::{
    AssetKind, Guide, Project, Surface, SurfaceTemplate, TemplateStatus, TextureResolution,
    VehicleRef,
};
use tp_vehicles::{ImageKind, Manifest, Package, TemplateImage, Texture};

use crate::workspace::Workspace;

/// The recorded vehicle of `variant_id` of `m`.
pub fn vehicle_ref(m: &Manifest, variant_id: &str) -> VehicleRef {
    VehicleRef {
        package_id: m.id.clone(),
        version: m.version.to_string(),
        variant_id: variant_id.to_owned(),
        name: m.name.clone(),
        brand: m.brand.clone(),
        kind: m.kind.code().to_owned(),
        game: m.game.code().to_owned(),
    }
}

/// Adds the template image of `texture` to the project's assets.
fn template_for(
    project: &mut Project,
    texture: &Texture,
    image: &TemplateImage,
    opacity: f32,
    visible: bool,
) -> SurfaceTemplate {
    let kind = match image.kind {
        ImageKind::Png => AssetKind::Raster,
        ImageKind::Svg => AssetKind::Svg,
    };
    let (asset, _) = project.add_asset(
        &texture.name,
        kind,
        Arc::from(image.bytes.as_slice()),
        Size::new(image.width, image.height),
    );
    SurfaceTemplate {
        texture_id: texture.id.clone(),
        asset,
        layout_version: texture.layout_version,
        opacity,
        visible,
        status: TemplateStatus::Current,
    }
}

/// A new project for `variant_id` of `package`: one surface per texture,
/// in order, each with its template; the first one active.
pub fn vehicle_project(name: &str, package: &Package, variant_id: &str) -> Option<Project> {
    let variant = package.manifest.variant(variant_id)?;
    let largest = variant
        .textures
        .iter()
        .map(|t| t.size)
        .max()
        .unwrap_or(4096);
    let resolution = TextureResolution::ALL
        .into_iter()
        .find(|r| r.side() >= largest)
        .unwrap_or(TextureResolution::R8192);
    let mut project = Project::new(name, resolution);
    let mut surfaces = Vec::new();
    for texture in &variant.textures {
        let image = package.template(variant_id, &texture.id)?;
        let mut surface = Surface::new(texture.name.clone(), f64::from(texture.size));
        surface.template = Some(template_for(
            &mut project,
            texture,
            image,
            SurfaceTemplate::DEFAULT_OPACITY,
            true,
        ));
        surfaces.push(surface);
    }
    project.surfaces = surfaces;
    project.active_surface = 0;
    project.vehicle = Some(vehicle_ref(&package.manifest, variant_id));
    Some(project)
}

/// What an update does to one texture.
#[derive(Clone, Debug, PartialEq)]
pub enum TextureChange {
    /// The template is replaced (the artwork is kept).
    Replaced {
        name: String,
        layout_changed: bool,
        /// Old and new sizes when the texture's size changed.
        resized: Option<(f64, f64)>,
    },
    /// A new texture: a new empty surface.
    Added { name: String },
    /// No longer in the package: the surface is kept, without template.
    Removed { name: String },
}

/// Moving a project to another version of its package.
#[derive(Clone, Debug, PartialEq)]
pub struct UpdatePlan {
    pub from: String,
    pub to: String,
    pub changes: Vec<TextureChange>,
}

/// The plan to move `project` to `package`, if it is a version of the
/// project's vehicle with its variant.
pub fn plan(project: &Project, package: &Package) -> Option<UpdatePlan> {
    let vehicle = project.vehicle.as_ref()?;
    let m = &package.manifest;
    if m.id != vehicle.package_id {
        return None;
    }
    let variant = m.variant(&vehicle.variant_id)?;
    let mut changes = Vec::new();
    for surface in &project.surfaces {
        let Some(t) = &surface.template else {
            continue;
        };
        match variant.textures.iter().find(|x| x.id == t.texture_id) {
            Some(texture) => changes.push(TextureChange::Replaced {
                name: surface.name.clone(),
                layout_changed: texture.layout_version != t.layout_version,
                resized: (f64::from(texture.size) != surface.size)
                    .then(|| (surface.size, f64::from(texture.size))),
            }),
            None => changes.push(TextureChange::Removed {
                name: surface.name.clone(),
            }),
        }
    }
    for texture in &variant.textures {
        let known = project.surfaces.iter().any(|s| {
            s.template
                .as_ref()
                .is_some_and(|t| t.texture_id == texture.id)
        });
        if !known {
            changes.push(TextureChange::Added {
                name: texture.name.clone(),
            });
        }
    }
    Some(UpdatePlan {
        from: vehicle.version.clone(),
        to: m.version.to_string(),
        changes,
    })
}

/// Scales a surface's artwork and guides from `old` to `new` texture pixels
/// (anchored at the top-left corner).
fn scale_surface(surface: &mut Surface, old: f64, new: f64) {
    let factor = new / old;
    let objects: Vec<Object> = surface.objects.iter().map(|o| (**o).clone()).collect();
    if !objects.is_empty() {
        let bounds = Frame::from_rect(Rect::new(0.0, 0.0, old, old));
        let scaled = resize(
            &objects,
            bounds,
            Handle { x: 1, y: 1 },
            Point::new(new, new),
            ResizeOptions::default(),
        );
        surface.objects = scaled.into_iter().map(Arc::new).collect();
    }
    for g in &mut surface.guides {
        *g = Guide::new(g.axis, g.position * factor);
    }
    surface.size = new;
}

impl Workspace {
    /// Moves the project to `package` (a newer version of its vehicle), as
    /// one undo step. Returns false when the package does not apply.
    pub fn apply_update(&mut self, package: &Package, now: f64) -> bool {
        let Some(vehicle) = self.project.vehicle.clone() else {
            return false;
        };
        let m = &package.manifest;
        let Some(variant) = m.variant(&vehicle.variant_id).cloned() else {
            return false;
        };
        if m.id != vehicle.package_id {
            return false;
        }
        let new_ref = vehicle_ref(m, &vehicle.variant_id);
        self.edit("undo-update-template", now, false, |project, selection| {
            selection.clear();
            let mut old_assets = Vec::new();
            for i in 0..project.surfaces.len() {
                let Some(t) = project.surfaces[i].template.clone() else {
                    continue;
                };
                let Some(texture) = variant.textures.iter().find(|x| x.id == t.texture_id) else {
                    project.surfaces[i]
                        .template
                        .as_mut()
                        .expect("template")
                        .status = TemplateStatus::Removed;
                    continue;
                };
                let Some(image) = package.template(&variant.id, &texture.id) else {
                    continue;
                };
                let mut new = template_for(project, texture, image, t.opacity, t.visible);
                // Flagged when the layout changed, and still flagged until
                // dismissed.
                new.status = if texture.layout_version != t.layout_version
                    || t.status == TemplateStatus::LayoutChanged
                {
                    TemplateStatus::LayoutChanged
                } else {
                    TemplateStatus::Current
                };
                old_assets.push(t.asset);
                let surface = &mut project.surfaces[i];
                let size = f64::from(texture.size);
                if size != surface.size {
                    let old = surface.size;
                    scale_surface(surface, old, size);
                }
                surface.template = Some(new);
            }
            for texture in &variant.textures {
                let known = project.surfaces.iter().any(|s| {
                    s.template
                        .as_ref()
                        .is_some_and(|t| t.texture_id == texture.id)
                });
                if known {
                    continue;
                }
                if let Some(image) = package.template(&variant.id, &texture.id) {
                    let mut surface = Surface::new(texture.name.clone(), f64::from(texture.size));
                    surface.template = Some(template_for(
                        project,
                        texture,
                        image,
                        SurfaceTemplate::DEFAULT_OPACITY,
                        true,
                    ));
                    project.surfaces.push(surface);
                }
            }
            for asset in old_assets {
                project.remove_asset(asset);
            }
            project.vehicle = Some(new_ref);
        });
        true
    }

    /// Clears the "Layout changed" flag of surface `index` (one undo step).
    pub fn dismiss_layout_change(&mut self, index: usize, now: f64) {
        self.edit("undo-dismiss-layout", now, false, |project, _| {
            if let Some(t) = project
                .surfaces
                .get_mut(index)
                .and_then(|s| s.template.as_mut())
                && t.status == TemplateStatus::LayoutChanged
            {
                t.status = TemplateStatus::Current;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use tp_core::document::{ObjectId, ShapeKind};
    use tp_vehicles::sample::{self, SampleTexture};

    use super::*;

    fn package(version: &str, textures: &[SampleTexture]) -> Package {
        Package::read(&sample::package(
            "scs.sample.truck",
            "Sample Truck",
            version,
            textures,
        ))
        .unwrap()
    }

    fn tex(id: &'static str, name: &'static str, size: u32, layout: u32) -> SampleTexture {
        SampleTexture {
            id,
            name,
            size,
            layout,
        }
    }

    #[test]
    fn project_from_a_vehicle() {
        let p = package("1.2.0", &sample::truck_textures());
        let project = vehicle_project("Sample Truck", &p, "standard").unwrap();
        let names: Vec<(&str, f64)> = project
            .surfaces
            .iter()
            .map(|s| (s.name.as_str(), s.size))
            .collect();
        assert_eq!(
            names,
            [
                ("Cabin", 4096.0),
                ("Chassis", 2048.0),
                ("Accessories", 1024.0)
            ]
        );
        assert_eq!(project.active_surface, 0);
        assert!(project.surfaces.iter().all(|s| s.template.is_some()));
        assert_eq!(project.template_assets().count(), 3);
        let v = project.vehicle.as_ref().unwrap();
        assert_eq!(
            (v.version.as_str(), v.variant_id.as_str()),
            ("1.2.0", "standard")
        );
        assert_eq!(project.resolution, TextureResolution::R4096);
        assert!(vehicle_project("x", &p, "nope").is_none());
    }

    #[test]
    fn update_replaces_flags_resizes_adds_and_removes() {
        let v1 = package(
            "1.2.0",
            &[
                tex("cabin", "Cabin", 4096, 1),
                tex("chassis", "Chassis", 2048, 1),
                tex("old", "Old", 512, 1),
            ],
        );
        let mut ws = Workspace::new(vehicle_project("T", &v1, "standard").unwrap());
        // Artwork on the chassis.
        ws.set_active_surface(1);
        let id = ws.create_shape(
            ShapeKind::rectangle(),
            Frame::new(Point::new(1000.0, 1000.0), Size::new(200.0, 100.0), 0.0),
            1.0,
        );
        ws.project.surfaces[0].template.as_mut().unwrap().opacity = 0.3;
        let v2 = package(
            "1.3.0",
            &[
                tex("cabin", "Cabin", 4096, 2),
                tex("chassis", "Chassis", 4096, 1),
                tex("new", "New", 1024, 1),
            ],
        );
        let p = plan(&ws.project, &v2).unwrap();
        assert_eq!((p.from.as_str(), p.to.as_str()), ("1.2.0", "1.3.0"));
        assert_eq!(
            p.changes,
            [
                TextureChange::Replaced {
                    name: "Cabin".into(),
                    layout_changed: true,
                    resized: None
                },
                TextureChange::Replaced {
                    name: "Chassis".into(),
                    layout_changed: false,
                    resized: Some((2048.0, 4096.0))
                },
                TextureChange::Removed { name: "Old".into() },
                TextureChange::Added { name: "New".into() },
            ]
        );
        let before_assets = ws.project.assets.len();
        assert!(ws.apply_update(&v2, 2.0));
        let s = &ws.project.surfaces;
        assert_eq!(s.len(), 4);
        let cabin = s[0].template.as_ref().unwrap();
        assert_eq!(
            (cabin.status, cabin.layout_version),
            (TemplateStatus::LayoutChanged, 2)
        );
        assert_eq!(cabin.opacity, 0.3, "settings kept");
        assert_eq!(
            s[1].template.as_ref().unwrap().status,
            TemplateStatus::Current
        );
        assert_eq!(s[1].size, 4096.0);
        // The chassis artwork doubled with its texture.
        let rect = s[1].objects.iter().find(|o| o.id == id).unwrap();
        assert!((rect.frame.center.x - 2000.0).abs() < 1e-6);
        assert!((rect.frame.size.width - 400.0).abs() < 1e-6);
        assert_eq!(
            s[2].template.as_ref().unwrap().status,
            TemplateStatus::Removed
        );
        assert_eq!(s[3].name, "New");
        assert_eq!(ws.project.vehicle.as_ref().unwrap().version, "1.3.0");
        // Replaced templates leave no unused asset behind (identical sample
        // images share one asset).
        assert!(ws.project.assets.len() <= before_assets + 1);
        assert!(
            ws.project
                .assets
                .keys()
                .all(|a| ws.project.asset_usage(*a) > 0),
            "no unused template asset"
        );
        assert_eq!(ws.history.undo_label(), Some("undo-update-template"));
        ws.undo();
        assert_eq!(ws.project.surfaces.len(), 3);
        assert_eq!(ws.project.vehicle.as_ref().unwrap().version, "1.2.0");
        assert_eq!(
            ws.project.surfaces[0].template.as_ref().unwrap().status,
            TemplateStatus::Current
        );
        let _ = ObjectId(0);
    }

    #[test]
    fn dismiss_a_layout_change() {
        let v1 = package("1.2.0", &[tex("cabin", "Cabin", 1024, 1)]);
        let mut ws = Workspace::new(vehicle_project("T", &v1, "standard").unwrap());
        let v2 = package("1.3.0", &[tex("cabin", "Cabin", 1024, 2)]);
        ws.apply_update(&v2, 1.0);
        ws.dismiss_layout_change(0, 2.0);
        assert_eq!(
            ws.project.surfaces[0].template.as_ref().unwrap().status,
            TemplateStatus::Current
        );
        ws.undo();
        assert_eq!(
            ws.project.surfaces[0].template.as_ref().unwrap().status,
            TemplateStatus::LayoutChanged
        );
    }

    #[test]
    fn paint_on_the_chassis_and_undo_across_textures() {
        let p = package("1.2.0", &sample::truck_textures());
        let mut ws = Workspace::new(vehicle_project("T", &p, "standard").unwrap());
        let canvas = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0));
        let cabin_view = crate::viewport::Viewport::fit(4096.0, canvas, 1.0);
        ws.viewport = Some(cabin_view);
        let frame = || Frame::new(Point::new(300.0, 300.0), Size::new(100.0, 50.0), 0.0);
        let on_cabin = ws.create_shape(ShapeKind::rectangle(), frame(), 1.0);
        ws.set_active_surface(1);
        assert!(ws.selection.is_empty());
        assert!(ws.viewport.is_none(), "the chassis fits on first display");
        let on_chassis = ws.create_shape(ShapeKind::rectangle(), frame(), 2.0);
        assert!(ws.project.surfaces[1].get(on_chassis).is_some());
        assert!(ws.project.surfaces[0].get(on_chassis).is_none());
        // Undo the chassis rectangle, then the cabin one: back on Cabin,
        // with its view.
        ws.undo();
        assert_eq!(ws.project.active_surface, 1);
        ws.undo();
        assert_eq!(ws.project.active_surface, 0);
        assert!(ws.project.surfaces[0].get(on_cabin).is_none());
        assert_eq!(ws.viewport, Some(cabin_view));
    }
}
