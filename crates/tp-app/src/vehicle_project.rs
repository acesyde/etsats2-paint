//! Fleet projects: vehicles with their chosen variants, one surface per
//! texture with its template; adding and removing vehicles and variants;
//! moving a vehicle to a newer version of its package.

use std::sync::Arc;

use tp_core::document::{AssetId, Frame, Handle, Object, ResizeOptions, resize};
use tp_core::kurbo::{Point, Rect, Size};
use tp_core::{
    AssetKind, Guide, Project, ProjectVehicle, Surface, SurfaceTemplate, TemplateStatus,
    TextureResolution, VariantRef,
};
use tp_vehicles::{ImageKind, Manifest, Package, TemplateImage, Texture};

use crate::workspace::Workspace;

/// The recorded vehicle of `m` with the variants `variant_ids` (in that
/// order; unknown ids are skipped).
pub fn project_vehicle(m: &Manifest, variant_ids: &[String]) -> ProjectVehicle {
    ProjectVehicle {
        package_id: m.id.clone(),
        version: m.version.to_string(),
        variants: variant_ids
            .iter()
            .filter_map(|id| m.variant(id))
            .map(|v| VariantRef {
                id: v.id.clone(),
                name: v.name.clone(),
            })
            .collect(),
        name: m.name.clone(),
        brand: m.brand.clone(),
        kind: m.kind.code().to_owned(),
        game: m.game.code().to_owned(),
    }
}

/// Adds the template image of `texture` of `variant` to the project's
/// assets.
fn template_for(
    project: &mut Project,
    package: &Manifest,
    variant: &str,
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
        package_id: package.id.clone(),
        variant_id: variant.to_owned(),
        texture_id: texture.id.clone(),
        asset,
        layout_version: texture.layout_version,
        opacity,
        visible,
        status: TemplateStatus::Current,
    }
}

/// The surfaces of `variant` of `package`, in texture order, their
/// templates added to `project`'s assets. `None` when a variant or
/// template is missing.
fn variant_surfaces(
    project: &mut Project,
    package: &Package,
    variant: &str,
) -> Option<Vec<Surface>> {
    let v = package.manifest.variant(variant)?;
    let mut surfaces = Vec::new();
    for texture in &v.textures {
        let image = package.template(variant, &texture.id)?;
        let mut surface = Surface::new(texture.name.clone(), f64::from(texture.size));
        surface.template = Some(template_for(
            project,
            &package.manifest,
            variant,
            texture,
            image,
            SurfaceTemplate::DEFAULT_OPACITY,
            true,
        ));
        surfaces.push(surface);
    }
    Some(surfaces)
}

/// A new project for `variants` of `package`: one surface per texture of
/// each variant, in order, each with its template; the first one active.
/// `None` without variants or when one is unknown.
pub fn fleet_project(name: &str, package: &Package, variants: &[String]) -> Option<Project> {
    if variants.is_empty() {
        return None;
    }
    let largest = variants
        .iter()
        .filter_map(|v| package.manifest.variant(v))
        .flat_map(|v| v.textures.iter().map(|t| t.size))
        .max()
        .unwrap_or(4096);
    let resolution = TextureResolution::ALL
        .into_iter()
        .find(|r| r.side() >= largest)
        .unwrap_or(TextureResolution::R8192);
    let mut project = Project::new(name, resolution);
    let mut surfaces = Vec::new();
    for variant in variants {
        surfaces.extend(variant_surfaces(&mut project, package, variant)?);
    }
    project.surfaces = surfaces;
    project.active_surface = 0;
    project.vehicles = vec![project_vehicle(&package.manifest, variants)];
    Some(project)
}

/// A new project for one variant of `package`.
pub fn vehicle_project(name: &str, package: &Package, variant_id: &str) -> Option<Project> {
    fleet_project(name, package, &[variant_id.to_owned()])
}

/// Why a fleet change is refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FleetError {
    /// The vehicle is for another game than the project.
    OtherGame,
    /// The vehicle is already in the project.
    AlreadyThere,
    /// No variant chosen, or one the package does not have.
    BadVariants,
    /// The package is not the version the project records.
    WrongVersion,
    /// The project's last vehicle cannot be removed.
    LastVehicle,
    /// Not a vehicle of the project.
    Unknown,
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

/// Moving a vehicle of a project to another version of its package.
#[derive(Clone, Debug, PartialEq)]
pub struct UpdatePlan {
    pub vehicle: String,
    pub from: String,
    pub to: String,
    pub changes: Vec<TextureChange>,
}

/// "Variant › Texture" when the vehicle has several variants, else the
/// texture name.
fn change_name(vehicle: &ProjectVehicle, variant: &str, texture: &str) -> String {
    if vehicle.variants.len() > 1 {
        let v = vehicle
            .variant(variant)
            .map_or(variant, |v| v.name.as_str());
        format!("{v} › {texture}")
    } else {
        texture.to_owned()
    }
}

/// The plan to move the project's vehicle of `package` to that package
/// version: every surface of the vehicle's chosen variants.
pub fn plan(project: &Project, package: &Package) -> Option<UpdatePlan> {
    let m = &package.manifest;
    let vehicle = project.vehicle(&m.id)?;
    let mut changes = Vec::new();
    for variant in &vehicle.variants {
        let new = m.variant(&variant.id);
        let range = project.variant_range(&m.id, &variant.id);
        for surface in &project.surfaces[range] {
            let Some(t) = &surface.template else {
                continue;
            };
            let name = change_name(vehicle, &variant.id, &surface.name);
            match new.and_then(|v| v.textures.iter().find(|x| x.id == t.texture_id)) {
                Some(texture) => changes.push(TextureChange::Replaced {
                    name,
                    layout_changed: texture.layout_version != t.layout_version,
                    resized: (f64::from(texture.size) != surface.size)
                        .then(|| (surface.size, f64::from(texture.size))),
                }),
                None => changes.push(TextureChange::Removed { name }),
            }
        }
        for texture in new.map(|v| v.textures.as_slice()).unwrap_or_default() {
            let known = project.surfaces.iter().any(|s| {
                s.template
                    .as_ref()
                    .is_some_and(|t| t.is_of(&m.id, &variant.id) && t.texture_id == texture.id)
            });
            if !known {
                changes.push(TextureChange::Added {
                    name: change_name(vehicle, &variant.id, &texture.name),
                });
            }
        }
    }
    Some(UpdatePlan {
        vehicle: vehicle.name.clone(),
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

/// Template assets of `surfaces`.
fn assets_of(surfaces: &[Surface]) -> Vec<AssetId> {
    surfaces
        .iter()
        .filter_map(|s| s.template.as_ref().map(|t| t.asset))
        .collect()
}

/// Removes the surfaces in `range`, then the template assets nothing uses
/// any more, and keeps a valid active surface.
fn remove_surfaces(project: &mut Project, range: std::ops::Range<usize>) {
    let active_removed = range.contains(&project.active_surface);
    let removed: Vec<Surface> = project.surfaces.drain(range.clone()).collect();
    for asset in assets_of(&removed) {
        project.remove_asset(asset);
    }
    if active_removed {
        project.active_surface = 0;
    } else if project.active_surface >= range.end {
        project.active_surface -= range.len();
    }
}

impl Workspace {
    /// Adds `variants` of `package` as a new vehicle of the project, after
    /// its surfaces; the first new surface becomes active. One undo step.
    pub fn add_vehicle(
        &mut self,
        package: &Package,
        variants: &[String],
        now: f64,
    ) -> Result<(), FleetError> {
        let m = &package.manifest;
        if self.project.game().is_some_and(|g| g != m.game.code()) {
            return Err(FleetError::OtherGame);
        }
        if self.project.vehicle(&m.id).is_some() {
            return Err(FleetError::AlreadyThere);
        }
        if variants.is_empty() || variants.iter().any(|v| m.variant(v).is_none()) {
            return Err(FleetError::BadVariants);
        }
        let mut result = Ok(());
        self.edit("undo-add-vehicle", now, false, |project, selection| {
            let mut surfaces = Vec::new();
            for variant in variants {
                match variant_surfaces(project, package, variant) {
                    Some(s) => surfaces.extend(s),
                    None => {
                        result = Err(FleetError::BadVariants);
                        return;
                    }
                }
            }
            selection.clear();
            let first = project.surfaces.len();
            project.surfaces.extend(surfaces);
            project.vehicles.push(project_vehicle(m, variants));
            project.active_surface = first;
        });
        result
    }

    /// Sets the chosen variants of the project's vehicle `package` (the
    /// version the project records): missing variants get their surfaces
    /// after the vehicle's, unchecked ones lose theirs. One undo step.
    pub fn set_variants(
        &mut self,
        package: &Package,
        variants: &[String],
        now: f64,
    ) -> Result<(), FleetError> {
        let m = &package.manifest;
        let Some(vehicle) = self.project.vehicle(&m.id).cloned() else {
            return Err(FleetError::Unknown);
        };
        if vehicle.version != m.version.to_string() {
            return Err(FleetError::WrongVersion);
        }
        if variants.is_empty() || variants.iter().any(|v| m.variant(v).is_none()) {
            return Err(FleetError::BadVariants);
        }
        self.edit("undo-change-variants", now, false, |project, selection| {
            selection.clear();
            for old in &vehicle.variants {
                if !variants.contains(&old.id) {
                    let range = project.variant_range(&m.id, &old.id);
                    remove_surfaces(project, range);
                }
            }
            let mut order: Vec<String> = vehicle
                .variants
                .iter()
                .map(|v| v.id.clone())
                .filter(|v| variants.contains(v))
                .collect();
            for variant in variants {
                if order.contains(variant) {
                    continue;
                }
                let at = project.vehicle_range(&m.id).end;
                if let Some(new) = variant_surfaces(project, package, variant) {
                    if project.active_surface >= at {
                        project.active_surface += new.len();
                    }
                    project.surfaces.splice(at..at, new);
                    order.push(variant.clone());
                }
            }
            if let Some(v) = project.vehicles.iter_mut().find(|v| v.package_id == m.id) {
                v.variants = project_vehicle(m, &order).variants;
            }
        });
        Ok(())
    }

    /// Removes the project's vehicle `package_id` and its surfaces. One
    /// undo step.
    pub fn remove_vehicle(&mut self, package_id: &str, now: f64) -> Result<(), FleetError> {
        if self.project.vehicle(package_id).is_none() {
            return Err(FleetError::Unknown);
        }
        if self.project.vehicles.len() <= 1 {
            return Err(FleetError::LastVehicle);
        }
        self.edit("undo-remove-vehicle", now, false, |project, selection| {
            selection.clear();
            let range = project.vehicle_range(package_id);
            remove_surfaces(project, range);
            project.vehicles.retain(|v| v.package_id != package_id);
        });
        Ok(())
    }

    /// Moves the project's vehicle of `package` to that package version, as
    /// one undo step. Returns false when the package does not apply.
    pub fn apply_update(&mut self, package: &Package, now: f64) -> bool {
        let m = &package.manifest;
        let Some(vehicle) = self.project.vehicle(&m.id).cloned() else {
            return false;
        };
        self.edit("undo-update-template", now, false, |project, selection| {
            selection.clear();
            let mut old_assets = Vec::new();
            for variant in &vehicle.variants {
                let new = m.variant(&variant.id);
                for i in project.variant_range(&m.id, &variant.id) {
                    let Some(t) = project.surfaces[i].template.clone() else {
                        continue;
                    };
                    let found = new.and_then(|v| {
                        let texture = v.textures.iter().find(|x| x.id == t.texture_id)?;
                        Some((texture, package.template(&v.id, &texture.id)?))
                    });
                    let Some((texture, image)) = found else {
                        project.surfaces[i]
                            .template
                            .as_mut()
                            .expect("template")
                            .status = TemplateStatus::Removed;
                        continue;
                    };
                    let mut replaced = template_for(
                        project,
                        m,
                        &variant.id,
                        texture,
                        image,
                        t.opacity,
                        t.visible,
                    );
                    // Flagged when the layout changed, and still flagged
                    // until dismissed.
                    replaced.status = if texture.layout_version != t.layout_version
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
                    surface.template = Some(replaced);
                }
                // New textures go after their variant's surfaces.
                for texture in new.map(|v| v.textures.as_slice()).unwrap_or_default() {
                    let known = project.surfaces.iter().any(|s| {
                        s.template.as_ref().is_some_and(|t| {
                            t.is_of(&m.id, &variant.id) && t.texture_id == texture.id
                        })
                    });
                    if known {
                        continue;
                    }
                    let Some(image) = package.template(&variant.id, &texture.id) else {
                        continue;
                    };
                    let mut surface = Surface::new(texture.name.clone(), f64::from(texture.size));
                    surface.template = Some(template_for(
                        project,
                        m,
                        &variant.id,
                        texture,
                        image,
                        SurfaceTemplate::DEFAULT_OPACITY,
                        true,
                    ));
                    let at = project.variant_range(&m.id, &variant.id).end;
                    if project.active_surface >= at {
                        project.active_surface += 1;
                    }
                    project.surfaces.insert(at, surface);
                }
            }
            for asset in old_assets {
                project.remove_asset(asset);
            }
            if let Some(v) = project.vehicles.iter_mut().find(|v| v.package_id == m.id) {
                v.version = m.version.to_string();
                v.name = m.name.clone();
                v.brand = m.brand.clone();
                for variant in &mut v.variants {
                    if let Some(x) = m.variant(&variant.id) {
                        variant.name = x.name.clone();
                    }
                }
            }
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

/// A small one-vehicle project (tests that save and read projects).
#[cfg(test)]
pub(crate) fn test_project(name: &str) -> Project {
    let textures = tp_vehicles::sample::truck_textures();
    let package = Package::read(&tp_vehicles::sample::package(
        "scs.sample.truck",
        "Sample Truck",
        "1.0.0",
        &textures,
    ))
    .expect("sample package");
    vehicle_project(name, &package, "standard").expect("sample variant")
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
        let v = &project.vehicles[0];
        assert_eq!(
            (v.version.as_str(), v.variants[0].id.as_str()),
            ("1.2.0", "standard")
        );
        assert_eq!(v.variants[0].name, "Standard cabin");
        let t = project.surfaces[1].template.as_ref().unwrap();
        assert_eq!(
            (
                t.package_id.as_str(),
                t.variant_id.as_str(),
                t.texture_id.as_str()
            ),
            ("scs.sample.truck", "standard", "chassis")
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
        assert_eq!(ws.project.vehicles[0].version, "1.3.0");
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
        assert_eq!(ws.project.vehicles[0].version, "1.2.0");
        assert_eq!(
            ws.project.surfaces[0].template.as_ref().unwrap().status,
            TemplateStatus::Current
        );
        let _ = ObjectId(0);
    }

    /// A version of the committed sample vehicle (two variants).
    fn example(version: &str) -> Package {
        let path = format!(
            "{}/../../examples/vehicles/community.truckpaint.sample_truck-{version}.tpv",
            env!("CARGO_MANIFEST_DIR")
        );
        Package::read(&std::fs::read(path).unwrap()).unwrap()
    }

    const SAMPLE: &str = "community.truckpaint.sample_truck";

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    /// A one-variant truck of `game` with id `id`.
    fn other_vehicle(id: &str, game: &str) -> Package {
        let textures = [tex("body", "Body", 1024, 1)];
        let mut m = sample::manifest(id, "Other", "1.0.0", &textures);
        m["game"] = game.into();
        m["kind"] = "trailer".into();
        Package::read(&sample::zip(
            &m,
            &[("templates/body.png".into(), sample::png(16))],
        ))
        .unwrap()
    }

    fn keys(project: &Project) -> Vec<(String, String, String)> {
        project
            .surfaces
            .iter()
            .map(|s| {
                let t = s.template.as_ref().unwrap();
                (
                    t.package_id.clone(),
                    t.variant_id.clone(),
                    t.texture_id.clone(),
                )
            })
            .collect()
    }

    fn rect(ws: &mut Workspace) -> ObjectId {
        ws.create_shape(
            ShapeKind::rectangle(),
            Frame::new(Point::new(100.0, 100.0), Size::new(50.0, 50.0), 0.0),
            1.0,
        )
    }

    #[test]
    fn fleet_project_from_two_variants() {
        let p =
            fleet_project("Fleet", &example("1.0.0"), &ids(&["standard", "high_roof"])).unwrap();
        let names: Vec<(String, String)> = keys(&p).into_iter().map(|(_, v, t)| (v, t)).collect();
        assert_eq!(
            names,
            [
                ("standard", "cabin"),
                ("standard", "chassis"),
                ("standard", "accessories"),
                ("high_roof", "cabin"),
                ("high_roof", "chassis"),
                ("high_roof", "accessories"),
            ]
            .map(|(v, t)| (v.to_owned(), t.to_owned()))
        );
        assert_eq!(p.active_surface, 0);
        let v = &p.vehicles[0];
        assert_eq!(v.variants[1].name, "High roof");
        assert_eq!(v.name, "TruckPaint Sample Truck");
        assert_eq!(p.game(), Some("ets2"));
        assert!(fleet_project("x", &example("1.0.0"), &[]).is_none());
        assert!(fleet_project("x", &example("1.0.0"), &ids(&["nope"])).is_none());
    }

    #[test]
    fn add_and_remove_vehicles() {
        let mut ws = Workspace::new(vehicle_project("F", &example("1.0.0"), "standard").unwrap());
        let assets = ws.project.assets.len();
        let trailer = other_vehicle("scs.krone.cool_liner", "ets2");
        ws.add_vehicle(&trailer, &ids(&["standard"]), 1.0).unwrap();
        assert_eq!(ws.project.vehicles.len(), 2);
        assert_eq!(ws.project.surfaces.len(), 4);
        assert_eq!(ws.project.active_surface, 3, "the new vehicle is shown");
        assert_eq!(ws.history.undo_label(), Some("undo-add-vehicle"));
        // Refusals.
        assert_eq!(
            ws.add_vehicle(&trailer, &ids(&["standard"]), 2.0),
            Err(FleetError::AlreadyThere)
        );
        let ats = other_vehicle("scs.peterbilt.579", "ats");
        assert_eq!(
            ws.add_vehicle(&ats, &ids(&["standard"]), 2.0),
            Err(FleetError::OtherGame)
        );
        let third = other_vehicle("scs.schmitz.box", "ets2");
        assert_eq!(
            ws.add_vehicle(&third, &[], 2.0),
            Err(FleetError::BadVariants)
        );
        // Remove the truck: only the trailer remains, its surface active.
        ws.remove_vehicle(SAMPLE, 3.0).unwrap();
        assert_eq!(ws.project.surfaces.len(), 1);
        assert_eq!(ws.project.active_surface, 0);
        assert!(
            ws.project
                .assets
                .keys()
                .all(|a| ws.project.asset_usage(*a) > 0),
            "unused templates dropped"
        );
        assert_eq!(
            ws.remove_vehicle("scs.krone.cool_liner", 4.0),
            Err(FleetError::LastVehicle)
        );
        assert_eq!(ws.remove_vehicle("x.y", 4.0), Err(FleetError::Unknown));
        ws.undo();
        assert_eq!(ws.project.vehicles.len(), 2);
        assert_eq!(ws.project.surfaces.len(), 4);
        ws.undo();
        assert_eq!(ws.project.vehicles.len(), 1);
        assert_eq!(
            ws.project.assets.len(),
            assets,
            "templates of the trailer gone"
        );
    }

    #[test]
    fn change_variants_keeps_order_and_undoes() {
        let v1 = example("1.0.0");
        let mut ws = Workspace::new(vehicle_project("F", &v1, "standard").unwrap());
        let trailer = other_vehicle("scs.krone.cool_liner", "ets2");
        ws.add_vehicle(&trailer, &ids(&["standard"]), 1.0).unwrap();
        // The trailer is active; adding a variant to the truck inserts
        // after the truck's surfaces and keeps the trailer active.
        ws.set_variants(&v1, &ids(&["standard", "high_roof"]), 2.0)
            .unwrap();
        assert_eq!(ws.project.surfaces.len(), 7);
        assert_eq!(ws.project.variant_range(SAMPLE, "high_roof"), 3..6);
        assert_eq!(ws.project.active_surface, 6);
        assert_eq!(ws.history.undo_label(), Some("undo-change-variants"));
        // Remove the Standard cab with artwork on its Cabin.
        ws.set_active_surface(0);
        let id = rect(&mut ws);
        ws.set_variants(&v1, &ids(&["high_roof"]), 3.0).unwrap();
        assert_eq!(ws.project.vehicles[0].variants.len(), 1);
        assert_eq!(ws.project.variant_range(SAMPLE, "high_roof"), 0..3);
        assert_eq!(ws.project.active_surface, 0);
        ws.undo();
        assert_eq!(ws.project.surfaces.len(), 7);
        assert!(ws.project.surfaces[0].get(id).is_some(), "artwork back");
        // Refusals.
        assert_eq!(ws.set_variants(&v1, &[], 4.0), Err(FleetError::BadVariants));
        assert_eq!(
            ws.set_variants(&example("1.1.0"), &ids(&["standard"]), 4.0),
            Err(FleetError::WrongVersion)
        );
    }

    #[test]
    fn views_reset_when_the_surfaces_change() {
        let v1 = example("1.0.0");
        let mut ws = Workspace::new(vehicle_project("F", &v1, "standard").unwrap());
        let view = crate::viewport::Viewport {
            center: Point::new(10.0, 10.0),
            zoom: 2.0,
            fitted: false,
        };
        ws.viewport = Some(view);
        ws.set_active_surface(1);
        ws.viewport = Some(view);
        rect(&mut ws);
        assert!(ws.viewport.is_some(), "a plain edit keeps the view");
        ws.set_variants(&v1, &ids(&["standard", "high_roof"]), 2.0)
            .unwrap();
        assert!(ws.viewport.is_none() && ws.viewports.is_empty());
        ws.viewport = Some(view);
        ws.undo();
        assert!(ws.viewport.is_none(), "undoing the change resets too");
        ws.set_active_surface(2);
        ws.redo();
        assert_eq!(ws.project.surfaces.len(), 6);
    }

    #[test]
    fn update_one_vehicle_with_two_variants() {
        let (v1, v2) = (example("1.0.0"), example("1.1.0"));
        let mut ws =
            Workspace::new(fleet_project("F", &v1, &ids(&["standard", "high_roof"])).unwrap());
        let trailer = other_vehicle("scs.krone.cool_liner", "ets2");
        ws.add_vehicle(&trailer, &ids(&["standard"]), 1.0).unwrap();
        let plan = plan(&ws.project, &v2).unwrap();
        assert_eq!(plan.vehicle, "TruckPaint Sample Truck");
        assert!(plan.changes.contains(&TextureChange::Added {
            name: "High roof › Side skirts".into()
        }));
        assert!(ws.apply_update(&v2, 2.0));
        let order: Vec<(String, String)> = keys(&ws.project)
            .into_iter()
            .map(|(_, v, t)| (v, t))
            .collect();
        assert_eq!(
            order,
            [
                ("standard", "cabin"),
                ("standard", "chassis"),
                ("standard", "accessories"),
                ("standard", "side_skirts"),
                ("high_roof", "cabin"),
                ("high_roof", "chassis"),
                ("high_roof", "accessories"),
                ("high_roof", "side_skirts"),
                ("standard", "body"),
            ]
            .map(|(v, t)| (v.to_owned(), t.to_owned()))
        );
        assert_eq!(ws.project.vehicles[0].version, "1.1.0");
        assert_eq!(ws.project.vehicles[1].version, "1.0.0", "trailer untouched");
        assert_eq!(
            ws.project.surfaces[0].template.as_ref().unwrap().status,
            TemplateStatus::LayoutChanged
        );
        assert_eq!(
            ws.project.surfaces[4].template.as_ref().unwrap().status,
            TemplateStatus::Current,
            "the High roof cabin layout did not change"
        );
        ws.undo();
        assert_eq!(ws.project.surfaces.len(), 7);
        assert_eq!(ws.project.vehicles[0].version, "1.0.0");
    }

    #[test]
    fn a_variant_missing_from_the_new_version_is_removed() {
        let v1 = package("1.2.0", &sample::truck_textures());
        let mut ws = Workspace::new(vehicle_project("T", &v1, "standard").unwrap());
        // The same package, whose only variant now has another id.
        let mut m = sample::manifest(
            "scs.sample.truck",
            "Sample Truck",
            "1.3.0",
            &sample::truck_textures(),
        );
        m["variants"][0]["id"] = "renamed".into();
        let files: Vec<(String, Vec<u8>)> = sample::truck_textures()
            .iter()
            .map(|t| (format!("templates/{}.png", t.id), sample::png(16)))
            .collect();
        let v2 = Package::read(&sample::zip(&m, &files)).unwrap();
        let p = plan(&ws.project, &v2).unwrap();
        assert!(
            p.changes
                .iter()
                .all(|c| matches!(c, TextureChange::Removed { .. }))
        );
        assert!(ws.apply_update(&v2, 1.0));
        assert!(
            ws.project
                .surfaces
                .iter()
                .all(|s| { s.template.as_ref().unwrap().status == TemplateStatus::Removed })
        );
        assert_eq!(ws.project.surfaces.len(), 3);
    }

    #[test]
    fn sample_vehicle_update_exercises_every_change() {
        let (v1, v2) = (example("1.0.0"), example("1.1.0"));
        let project = vehicle_project("Sample", &v1, "standard").unwrap();
        let p = plan(&project, &v2).unwrap();
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
                TextureChange::Replaced {
                    name: "Accessories".into(),
                    layout_changed: false,
                    resized: None
                },
                TextureChange::Added {
                    name: "Side skirts".into()
                },
            ]
        );
        // The High roof cabin layout is unchanged.
        let high = vehicle_project("Sample", &v1, "high_roof").unwrap();
        assert!(matches!(
            plan(&high, &v2).unwrap().changes[0],
            TextureChange::Replaced {
                layout_changed: false,
                ..
            }
        ));
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
