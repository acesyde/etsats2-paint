//! Fleet projects: vehicles with the textures they paint (main textures
//! and accessories), one surface per texture with its template; adding and
//! removing vehicles and textures; moving a vehicle to a newer version of
//! its package.

use std::sync::Arc;

use tp_core::document::{AssetId, Object, apply_affine};
use tp_core::kurbo::Size;
use tp_core::{
    AssetKind, GameData, Guide, Project, ProjectVehicle, RequiredMod, Surface, SurfaceTemplate,
    TemplateStatus, TexturePart, TextureResolution,
};
use tp_vehicles::{ImageKind, Manifest, Package, Part, Role};

use crate::workspace::Workspace;

/// The recorded vehicle of `m`.
pub fn project_vehicle(m: &Manifest) -> ProjectVehicle {
    ProjectVehicle {
        package_id: m.id.clone(),
        version: m.version.to_string(),
        name: m.name.clone(),
        brand: m.brand.clone(),
        kind: m.kind.code().to_owned(),
        game: m.game.id.code().to_owned(),
        game_data: Some(game_data(m)),
    }
}

/// What the mod export needs to know about the vehicle of `m`.
pub fn game_data(m: &Manifest) -> GameData {
    GameData {
        path: m.game.path.clone(),
        versions: m.game.versions.to_string(),
        alt_uv: m.game.alt_uv,
        colour_picker: m.game.colour_picker,
        requires: m
            .game
            .requires
            .iter()
            .map(|r| RequiredMod {
                name: r.name.clone(),
                version: r.version.clone(),
            })
            .collect(),
        main_count: m.paint_job.main.len(),
    }
}

/// A main texture's position among the main textures of `m`.
fn main_index(m: &Manifest, role: Role, id: &str) -> Option<usize> {
    (role == Role::Main)
        .then(|| m.paint_job.main.iter().position(|p| p.id == id))
        .flatten()
}

/// Records the game data of `m` in its vehicle of `project` and in the
/// templates of the textures `m` has, when the vehicle has none (files
/// written before it was recorded). Returns whether anything was filled.
pub fn fill_game_data(project: &mut Project, m: &Manifest) -> bool {
    let Some(vehicle) = project
        .vehicles
        .iter_mut()
        .find(|v| v.package_id == m.id && v.game_data.is_none())
    else {
        return false;
    };
    vehicle.game_data = Some(game_data(m));
    for surface in &mut project.surfaces {
        if let Some(t) = &mut surface.template
            && t.package_id == m.id
            && let Some((role, part)) = m.paint_job.part(&t.texture_id)
        {
            t.game_ids = part.game_ids.clone();
            t.main_index = main_index(m, role, &part.id);
        }
    }
    true
}

/// The project's part kind of a package role.
pub fn texture_part(role: Role) -> TexturePart {
    match role {
        Role::Main => TexturePart::Main,
        Role::Accessory => TexturePart::Accessory,
    }
}

/// The textures painted by default: the first main texture and every
/// accessory.
pub fn default_textures(m: &Manifest) -> Vec<String> {
    let job = &m.paint_job;
    job.main
        .iter()
        .take(1)
        .chain(&job.accessories)
        .map(|p| p.id.clone())
        .collect()
}

/// Whether part `id` is always painted: the single main texture.
pub fn is_always_painted(m: &Manifest, id: &str) -> bool {
    let main = &m.paint_job.main;
    main.len() == 1 && main[0].id == id
}

/// The textures to paint for `chosen`, in package order: known ids only,
/// a single main texture always, and at least one main texture.
fn resolve(m: &Manifest, chosen: &[String]) -> Result<Vec<String>, FleetError> {
    if chosen.iter().any(|id| m.paint_job.part(id).is_none()) {
        return Err(FleetError::BadTextures);
    }
    let ids: Vec<String> = m
        .paint_job
        .parts()
        .filter(|(_, p)| chosen.contains(&p.id) || is_always_painted(m, &p.id))
        .map(|(_, p)| p.id.clone())
        .collect();
    if !m.paint_job.main.iter().any(|p| ids.contains(&p.id)) {
        return Err(FleetError::BadTextures);
    }
    Ok(ids)
}

/// Adds the template image of `part` to the project's assets.
fn template_for(
    project: &mut Project,
    package: &Package,
    role: Role,
    part: &Part,
    opacity: f32,
    visible: bool,
) -> Option<SurfaceTemplate> {
    let image = package.template(&part.id)?;
    let kind = match image.kind {
        ImageKind::Png => AssetKind::Raster,
        ImageKind::Svg => AssetKind::Svg,
    };
    let (asset, _) = project.add_asset(
        &part.name,
        kind,
        Arc::from(image.bytes.as_slice()),
        Size::new(image.width, image.height),
    );
    Some(SurfaceTemplate {
        package_id: package.manifest.id.clone(),
        texture_id: part.id.clone(),
        part: texture_part(role),
        asset,
        layout_version: part.texture.layout_version,
        game_ids: part.game_ids.clone(),
        main_index: main_index(&package.manifest, role, &part.id),
        opacity,
        visible,
        status: TemplateStatus::Current,
    })
}

/// A new empty surface for part `id` of `package`, its template added to
/// `project`'s assets.
fn part_surface(project: &mut Project, package: &Package, id: &str) -> Option<Surface> {
    let (role, part) = package.manifest.paint_job.part(id)?;
    let mut surface = Surface::new(part.name.clone(), f64::from(part.texture.size));
    surface.template = Some(template_for(
        project,
        package,
        role,
        part,
        SurfaceTemplate::DEFAULT_OPACITY,
        true,
    )?);
    Some(surface)
}

/// Where a new surface for part `id` goes: before the first surface of the
/// vehicle whose texture comes after it in package order (surfaces whose
/// texture the package no longer has keep their place), else at the end of
/// the vehicle's surfaces.
fn insertion_index(project: &Project, m: &Manifest, id: &str) -> usize {
    let range = project.vehicle_range(&m.id);
    let Some(position) = m.paint_job.position(id) else {
        return range.end;
    };
    range
        .clone()
        .find(|&i| {
            project.surfaces[i]
                .template
                .as_ref()
                .and_then(|t| m.paint_job.position(&t.texture_id))
                .is_some_and(|p| p > position)
        })
        .unwrap_or(range.end)
}

/// Inserts `surface` at `at`, keeping the same surface active.
fn insert_surface(project: &mut Project, at: usize, surface: Surface) {
    if project.active_surface >= at {
        project.active_surface += 1;
    }
    project.surfaces.insert(at, surface);
}

/// Whether the vehicle `package` has a surface for texture `id`.
fn paints(project: &Project, package: &str, id: &str) -> bool {
    project
        .surfaces
        .iter()
        .any(|s| s.template.as_ref().is_some_and(|t| t.is_of(package, id)))
}

/// A new project for `chosen` textures of `package` (see [`resolve`]): one
/// surface per texture in package order, each with its template; the first
/// one active.
pub fn fleet_project(
    name: &str,
    package: &Package,
    chosen: &[String],
) -> Result<Project, FleetError> {
    let ids = resolve(&package.manifest, chosen)?;
    let largest = ids
        .iter()
        .filter_map(|id| package.manifest.paint_job.part(id))
        .map(|(_, p)| p.texture.size)
        .max()
        .unwrap_or(4096);
    let resolution = TextureResolution::ALL
        .into_iter()
        .find(|r| r.side() >= largest)
        .unwrap_or(TextureResolution::R8192);
    let mut project = Project::new(name, resolution);
    let mut surfaces = Vec::new();
    for id in &ids {
        surfaces.push(part_surface(&mut project, package, id).ok_or(FleetError::BadTextures)?);
    }
    project.surfaces = surfaces;
    project.active_surface = 0;
    project.vehicles = vec![project_vehicle(&package.manifest)];
    Ok(project)
}

/// Why a fleet change is refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FleetError {
    /// The vehicle is for another game than the project.
    OtherGame,
    /// The vehicle is already in the project.
    AlreadyThere,
    /// No main texture chosen, or a texture the package does not have.
    BadTextures,
    /// The package is not the version the project records.
    WrongVersion,
    /// The project's last vehicle cannot be removed.
    LastVehicle,
    /// Not a vehicle of the project.
    Unknown,
}

/// What an update does to one texture the vehicle paints.
#[derive(Clone, Debug, PartialEq)]
pub enum TextureChange {
    /// The template is replaced (the artwork is kept).
    Replaced {
        name: String,
        layout_changed: bool,
        /// Old and new sizes when the texture's size changed.
        resized: Option<(f64, f64)>,
    },
    /// No longer in the package: the surface is kept, without template.
    Removed { name: String },
}

/// A texture of the new version that the vehicle doesn't paint yet.
#[derive(Clone, Debug, PartialEq)]
pub struct NewTexture {
    pub id: String,
    pub name: String,
    pub part: TexturePart,
    /// The single main texture: always added.
    pub always: bool,
}

impl NewTexture {
    /// Offered checked: accessories, and the texture always added.
    pub fn checked_by_default(&self) -> bool {
        self.always || self.part == TexturePart::Accessory
    }
}

/// Moving a vehicle of a project to another version of its package.
#[derive(Clone, Debug, PartialEq)]
pub struct UpdatePlan {
    pub vehicle: String,
    pub from: String,
    pub to: String,
    pub changes: Vec<TextureChange>,
    pub new: Vec<NewTexture>,
}

/// The plan to move the project's vehicle of `package` to that package
/// version: every surface of the vehicle, and the textures it could add.
pub fn plan(project: &Project, package: &Package) -> Option<UpdatePlan> {
    let m = &package.manifest;
    let vehicle = project.vehicle(&m.id)?;
    let mut changes = Vec::new();
    for surface in &project.surfaces[project.vehicle_range(&m.id)] {
        let Some(t) = &surface.template else {
            continue;
        };
        let name = surface.name.clone();
        match m.paint_job.part(&t.texture_id) {
            Some((_, part)) => {
                let size = f64::from(part.texture.size);
                changes.push(TextureChange::Replaced {
                    name,
                    layout_changed: part.texture.layout_version != t.layout_version,
                    resized: (size != surface.size).then_some((surface.size, size)),
                });
            }
            None => changes.push(TextureChange::Removed { name }),
        }
    }
    let new = m
        .paint_job
        .parts()
        .filter(|(_, p)| !paints(project, &m.id, &p.id))
        .map(|(role, p)| NewTexture {
            id: p.id.clone(),
            name: p.name.clone(),
            part: texture_part(role),
            always: is_always_painted(m, &p.id),
        })
        .collect();
    Some(UpdatePlan {
        vehicle: vehicle.name.clone(),
        from: vehicle.version.clone(),
        to: m.version.to_string(),
        changes,
        new,
    })
}

/// `objects` of a texture of `old` pixels, scaled for a texture of `new`
/// pixels (anchored at the top-left corner).
pub fn scale_objects(objects: &[Object], old: f64, new: f64) -> Vec<Object> {
    if objects.is_empty() || old == new {
        return objects.to_vec();
    }
    apply_affine(objects, tp_core::kurbo::Affine::scale(new / old))
}

/// The other main textures of the vehicle surface `index` is a main
/// texture of: the cabins Copy From Cabin can copy from.
pub fn cabin_sources(project: &Project, index: usize) -> Vec<usize> {
    let main_of = |i: usize| {
        project.surfaces[i]
            .template
            .as_ref()
            .filter(|t| t.part == TexturePart::Main)
            .map(|t| t.package_id.as_str())
    };
    let Some(package) = main_of(index) else {
        return Vec::new();
    };
    (0..project.surfaces.len())
        .filter(|&i| i != index && main_of(i) == Some(package))
        .collect()
}

/// Scales a surface's artwork and guides from `old` to `new` texture pixels
/// (anchored at the top-left corner).
fn scale_surface(surface: &mut Surface, old: f64, new: f64) {
    let factor = new / old;
    let objects: Vec<Object> = surface.objects.iter().map(|o| (**o).clone()).collect();
    if !objects.is_empty() {
        let scaled = scale_objects(&objects, old, new);
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
    /// Adds `package` as a new vehicle of the project painting `chosen`
    /// textures (see [`resolve`]), after its surfaces; the first new surface
    /// becomes active. One undo step.
    pub fn add_vehicle(
        &mut self,
        package: &Package,
        chosen: &[String],
        now: f64,
    ) -> Result<(), FleetError> {
        let m = &package.manifest;
        if self.project.game().is_some_and(|g| g != m.game.id.code()) {
            return Err(FleetError::OtherGame);
        }
        if self.project.vehicle(&m.id).is_some() {
            return Err(FleetError::AlreadyThere);
        }
        let ids = resolve(m, chosen)?;
        let mut result = Ok(());
        self.edit("undo-add-vehicle", now, false, |project, selection| {
            let mut surfaces = Vec::new();
            for id in &ids {
                match part_surface(project, package, id) {
                    Some(s) => surfaces.push(s),
                    None => {
                        result = Err(FleetError::BadTextures);
                        return;
                    }
                }
            }
            selection.clear();
            let first = project.surfaces.len();
            project.surfaces.extend(surfaces);
            project.vehicles.push(project_vehicle(m));
            project.active_surface = first;
        });
        result
    }

    /// Sets the textures the project's vehicle `package` paints (the version
    /// the project records; see [`resolve`]): unchecked ones lose their
    /// surface, new ones get one in package order. Surfaces of textures the
    /// version doesn't have are kept. One undo step.
    pub fn set_textures(
        &mut self,
        package: &Package,
        chosen: &[String],
        now: f64,
    ) -> Result<(), FleetError> {
        let m = &package.manifest;
        let Some(vehicle) = self.project.vehicle(&m.id) else {
            return Err(FleetError::Unknown);
        };
        if vehicle.version != m.version.to_string() {
            return Err(FleetError::WrongVersion);
        }
        let ids = resolve(m, chosen)?;
        self.edit("undo-change-textures", now, false, |project, selection| {
            selection.clear();
            for i in project.vehicle_range(&m.id).rev() {
                let unchecked = project.surfaces[i].template.as_ref().is_some_and(|t| {
                    m.paint_job.part(&t.texture_id).is_some() && !ids.contains(&t.texture_id)
                });
                if unchecked {
                    remove_surfaces(project, i..i + 1);
                }
            }
            for id in &ids {
                if paints(project, &m.id, id) {
                    continue;
                }
                let at = insertion_index(project, m, id);
                if let Some(surface) = part_surface(project, package, id) {
                    insert_surface(project, at, surface);
                }
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
    /// one undo step, adding the new textures `added` and a new single main
    /// texture. Returns false when the package does not apply.
    pub fn apply_update(&mut self, package: &Package, added: &[String], now: f64) -> bool {
        let m = &package.manifest;
        if self.project.vehicle(&m.id).is_none() {
            return false;
        }
        self.edit("undo-update-template", now, false, |project, selection| {
            selection.clear();
            let mut old_assets = Vec::new();
            for i in project.vehicle_range(&m.id) {
                let Some(t) = project.surfaces[i].template.clone() else {
                    continue;
                };
                let Some((role, part)) = m.paint_job.part(&t.texture_id) else {
                    project.surfaces[i]
                        .template
                        .as_mut()
                        .expect("template")
                        .status = TemplateStatus::Removed;
                    continue;
                };
                let Some(mut replaced) =
                    template_for(project, package, role, part, t.opacity, t.visible)
                else {
                    continue;
                };
                // Flagged when the layout changed, and still flagged
                // until dismissed.
                replaced.status = if part.texture.layout_version != t.layout_version
                    || t.status == TemplateStatus::LayoutChanged
                {
                    TemplateStatus::LayoutChanged
                } else {
                    TemplateStatus::Current
                };
                old_assets.push(t.asset);
                let surface = &mut project.surfaces[i];
                let size = f64::from(part.texture.size);
                if size != surface.size {
                    let old = surface.size;
                    scale_surface(surface, old, size);
                }
                surface.template = Some(replaced);
            }
            for (_, part) in m.paint_job.parts() {
                let wanted = added.contains(&part.id) || is_always_painted(m, &part.id);
                if !wanted || paints(project, &m.id, &part.id) {
                    continue;
                }
                let at = insertion_index(project, m, &part.id);
                if let Some(surface) = part_surface(project, package, &part.id) {
                    insert_surface(project, at, surface);
                }
            }
            for asset in old_assets {
                project.remove_asset(asset);
            }
            if let Some(v) = project.vehicles.iter_mut().find(|v| v.package_id == m.id) {
                *v = project_vehicle(m);
            }
        });
        true
    }

    /// Clears the "Layout changed" flag of surface `index`: Mark as
    /// Checked (one undo step).
    pub fn dismiss_layout_change(&mut self, index: usize, now: f64) {
        self.edit("undo-mark-checked", now, false, |project, _| {
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
    let package = Package::read(&tp_vehicles::sample::package(
        "scs.sample.truck",
        "Sample Truck",
        "1.0.0",
        &tp_vehicles::sample::truck_textures(),
    ))
    .expect("sample package");
    fleet_project(name, &package, &default_textures(&package.manifest)).expect("sample textures")
}

#[cfg(test)]
mod tests {
    use tp_core::document::{Frame, ObjectId, ShapeKind};
    use tp_core::kurbo::{Point, Rect};
    use tp_vehicles::sample::{self, SampleTexture};

    use super::*;

    /// A truck whose first texture is its main texture, the others
    /// accessories.
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

    /// A version of a committed sample package.
    fn example(name: &str, version: &str) -> Package {
        let path = format!(
            "{}/../../examples/vehicles/community.truckpaint.{name}-{version}.tpv",
            env!("CARGO_MANIFEST_DIR")
        );
        Package::read(&std::fs::read(path).unwrap()).unwrap()
    }

    fn truck(version: &str) -> Package {
        example("sample_truck", version)
    }

    fn trailer() -> Package {
        example("sample_trailer", "1.0.0")
    }

    const TRUCK: &str = "community.truckpaint.sample_truck";
    const TRAILER: &str = "community.truckpaint.sample_trailer";

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    /// A trailer of `game` with id `id` and a single main texture "Body".
    fn other_vehicle(id: &str, game: &str) -> Package {
        let mut m = sample::manifest_with(
            id,
            "Other",
            "1.0.0",
            "trailer",
            &[tex("body", "Body", 1024, 1)],
            &[],
        );
        m["game"]["id"] = game.into();
        Package::read(&sample::zip(
            &m,
            &[("templates/body.png".into(), sample::png(16))],
        ))
        .unwrap()
    }

    /// `(package, texture)` of every surface.
    fn keys(project: &Project) -> Vec<(String, String)> {
        project
            .surfaces
            .iter()
            .map(|s| {
                let t = s.template.as_ref().unwrap();
                (t.package_id.clone(), t.texture_id.clone())
            })
            .collect()
    }

    fn textures_of(project: &Project, package: &str) -> Vec<String> {
        keys(project)
            .into_iter()
            .filter(|(p, _)| p == package)
            .map(|(_, t)| t)
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
    fn project_from_a_truck_with_two_main_textures() {
        let p = truck("1.1.0");
        let chosen = ids(&[
            "standard",
            "high_roof",
            "chassis",
            "cab_accessories",
            "side_skirts",
        ]);
        let project = fleet_project("Fleet", &p, &chosen).unwrap();
        let names: Vec<(&str, f64)> = project
            .surfaces
            .iter()
            .map(|s| (s.name.as_str(), s.size))
            .collect();
        assert_eq!(
            names,
            [
                ("Standard cab", 4096.0),
                ("High roof", 4096.0),
                ("Chassis", 4096.0),
                ("Cab accessories", 1024.0),
                ("Side skirts", 1024.0)
            ]
        );
        let parts: Vec<TexturePart> = project
            .surfaces
            .iter()
            .map(|s| s.template.as_ref().unwrap().part)
            .collect();
        use TexturePart::{Accessory, Main};
        assert_eq!(parts, [Main, Main, Accessory, Accessory, Accessory]);
        assert_eq!(project.active_surface, 0);
        let v = &project.vehicles[0];
        assert_eq!(
            (v.version.as_str(), v.name.as_str()),
            ("1.1.0", "TruckPaint Sample Truck")
        );
        assert_eq!(project.game(), Some("ets2"));
        assert_eq!(project.resolution, TextureResolution::R4096);
        // The order chosen doesn't matter: package order.
        let reversed: Vec<String> = chosen.iter().rev().cloned().collect();
        assert_eq!(
            keys(&fleet_project("F", &p, &reversed).unwrap()),
            keys(&project)
        );
    }

    #[test]
    fn defaults_left_out_accessories_and_refusals() {
        let p = truck("1.1.0");
        assert_eq!(
            default_textures(&p.manifest),
            ids(&["standard", "chassis", "cab_accessories", "side_skirts"])
        );
        let project = fleet_project("F", &p, &ids(&["standard", "chassis"])).unwrap();
        assert_eq!(textures_of(&project, TRUCK), ids(&["standard", "chassis"]));
        // No main texture, or an unknown one.
        assert_eq!(
            fleet_project("F", &p, &ids(&["chassis"])).unwrap_err(),
            FleetError::BadTextures
        );
        assert_eq!(
            fleet_project("F", &p, &ids(&["standard", "nope"])).unwrap_err(),
            FleetError::BadTextures
        );
    }

    #[test]
    fn a_trailer_always_paints_its_single_main_texture() {
        let p = trailer();
        assert!(is_always_painted(&p.manifest, "base"));
        let project =
            fleet_project("T", &p, &ids(&["body_13_6", "body_10_5", "mudflaps"])).unwrap();
        let names: Vec<&str> = project.surfaces.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "Base",
                "Curtain body 13.6 m",
                "Curtain body 10.5 m",
                "Mudflaps"
            ]
        );
        // Nothing chosen: the Base alone.
        let alone = fleet_project("T", &p, &[]).unwrap();
        assert_eq!(textures_of(&alone, TRAILER), ids(&["base"]));
    }

    #[test]
    fn add_and_remove_vehicles() {
        let mut ws =
            Workspace::new(fleet_project("F", &truck("1.0.0"), &ids(&["standard"])).unwrap());
        let assets = ws.project.assets.len();
        ws.add_vehicle(&trailer(), &ids(&["mudflaps"]), 1.0)
            .unwrap();
        assert_eq!(ws.project.vehicles.len(), 2);
        assert_eq!(
            textures_of(&ws.project, TRAILER),
            ids(&["base", "mudflaps"])
        );
        assert_eq!(ws.project.active_surface, 1, "the new vehicle is shown");
        assert_eq!(ws.history.undo_label(), Some("undo-add-vehicle"));
        // Refusals.
        assert_eq!(
            ws.add_vehicle(&trailer(), &[], 2.0),
            Err(FleetError::AlreadyThere)
        );
        let ats = other_vehicle("scs.peterbilt.579", "ats");
        assert_eq!(ws.add_vehicle(&ats, &[], 2.0), Err(FleetError::OtherGame));
        let third = other_vehicle("scs.schmitz.box", "ets2");
        assert_eq!(
            ws.add_vehicle(&third, &ids(&["nope"]), 2.0),
            Err(FleetError::BadTextures)
        );
        // Remove the truck: only the trailer remains, its surface active.
        ws.remove_vehicle(TRUCK, 3.0).unwrap();
        assert_eq!(ws.project.surfaces.len(), 2);
        assert_eq!(ws.project.active_surface, 0);
        assert!(
            ws.project
                .assets
                .keys()
                .all(|a| ws.project.asset_usage(*a) > 0),
            "unused templates dropped"
        );
        assert_eq!(
            ws.remove_vehicle(TRAILER, 4.0),
            Err(FleetError::LastVehicle)
        );
        assert_eq!(ws.remove_vehicle("x.y", 4.0), Err(FleetError::Unknown));
        ws.undo();
        assert_eq!(ws.project.vehicles.len(), 2);
        ws.undo();
        assert_eq!(ws.project.vehicles.len(), 1);
        assert_eq!(
            ws.project.assets.len(),
            assets,
            "templates of the trailer gone"
        );
    }

    #[test]
    fn set_textures_inserts_in_package_order_and_undoes() {
        let v = truck("1.1.0");
        let mut ws = Workspace::new(
            fleet_project("F", &v, &ids(&["standard", "chassis", "side_skirts"])).unwrap(),
        );
        ws.add_vehicle(&trailer(), &[], 1.0).unwrap();
        // The trailer is active; an accessory added between two others keeps
        // it active.
        assert_eq!(ws.project.active_surface, 3);
        ws.set_textures(
            &v,
            &ids(&["standard", "chassis", "cab_accessories", "side_skirts"]),
            2.0,
        )
        .unwrap();
        assert_eq!(
            textures_of(&ws.project, TRUCK),
            ids(&["standard", "chassis", "cab_accessories", "side_skirts"])
        );
        assert_eq!(ws.project.active_surface, 4);
        assert_eq!(ws.history.undo_label(), Some("undo-change-textures"));
        // A main texture added before the others: after Standard cab.
        ws.set_textures(
            &v,
            &ids(&[
                "standard",
                "high_roof",
                "chassis",
                "cab_accessories",
                "side_skirts",
            ]),
            3.0,
        )
        .unwrap();
        assert_eq!(
            textures_of(&ws.project, TRUCK)[..2],
            ids(&["standard", "high_roof"])
        );
        // Removing the Standard cab with artwork, then undoing.
        ws.set_active_surface(0);
        let id = rect(&mut ws);
        ws.set_textures(&v, &ids(&["high_roof", "chassis"]), 4.0)
            .unwrap();
        assert_eq!(
            textures_of(&ws.project, TRUCK),
            ids(&["high_roof", "chassis"])
        );
        assert_eq!(ws.project.active_surface, 0);
        assert!(
            ws.project
                .assets
                .keys()
                .all(|a| ws.project.asset_usage(*a) > 0),
            "unused templates dropped"
        );
        ws.undo();
        assert_eq!(textures_of(&ws.project, TRUCK).len(), 5);
        assert!(ws.project.surfaces[0].get(id).is_some(), "artwork back");
        // Refusals: no main texture, another version.
        assert_eq!(
            ws.set_textures(&v, &ids(&["chassis"]), 5.0),
            Err(FleetError::BadTextures)
        );
        assert_eq!(
            ws.set_textures(&truck("1.0.0"), &ids(&["standard"]), 5.0),
            Err(FleetError::WrongVersion)
        );
    }

    #[test]
    fn set_textures_keeps_textures_missing_from_the_version_in_place() {
        // A vehicle updated to a version without "old": its surface stays,
        // marked, and insertion goes around it.
        let v1 = package(
            "1.2.0",
            &[
                tex("cabin", "Cabin", 1024, 1),
                tex("old", "Old", 512, 1),
                tex("z", "Z", 512, 1),
            ],
        );
        let mut ws = Workspace::new(fleet_project("T", &v1, &ids(&["cabin", "old", "z"])).unwrap());
        let v2 = package(
            "1.3.0",
            &[
                tex("cabin", "Cabin", 1024, 1),
                tex("new", "New", 512, 1),
                tex("z", "Z", 512, 1),
            ],
        );
        assert!(ws.apply_update(&v2, &[], 1.0));
        ws.set_textures(&v2, &ids(&["cabin", "new", "z"]), 2.0)
            .unwrap();
        assert_eq!(
            textures_of(&ws.project, "scs.sample.truck"),
            ids(&["cabin", "old", "new", "z"])
        );
        assert_eq!(
            ws.project.surfaces[1].template.as_ref().unwrap().status,
            TemplateStatus::Removed
        );
    }

    #[test]
    fn game_data_is_recorded_on_creation() {
        let project =
            fleet_project("F", &truck("1.1.0"), &ids(&["standard", "side_skirts"])).unwrap();
        let data = project.vehicles[0].game_data.as_ref().unwrap();
        assert_eq!(data.path, "truckpaint.sample");
        assert_eq!(data.main_count, 2);
        assert!(!data.alt_uv && !data.colour_picker);
        let standard = project.surfaces[0].template.as_ref().unwrap();
        assert_eq!(standard.game_ids, ids(&["standard"]));
        assert_eq!(standard.main_index, Some(0));
        let skirts = project.surfaces[1].template.as_ref().unwrap();
        assert_eq!(skirts.game_ids, ids(&["sideskirt.sample"]));
        assert_eq!(skirts.main_index, None);
    }

    /// Version `version` of the generated truck, whose "chassis" accessory
    /// covers `chassis_ids`.
    fn truck_with_chassis_ids(
        version: &str,
        textures: &[SampleTexture],
        chassis_ids: &[&str],
    ) -> Package {
        let mut m = sample::manifest("scs.sample.truck", "Sample Truck", version, textures);
        for part in m["paint_job"]["accessories"].as_array_mut().unwrap() {
            if part["id"] == "chassis" {
                part["game_ids"] = chassis_ids.to_vec().into();
            }
        }
        Package::read(&sample::zip(&m, &sample::templates(textures))).unwrap()
    }

    #[test]
    fn update_template_replaces_the_game_data() {
        let textures = [
            tex("cabin", "Cabin", 1024, 1),
            tex("chassis", "Chassis", 512, 1),
            tex("old", "Old", 512, 1),
        ];
        let v1 = truck_with_chassis_ids("1.2.0", &textures, &["chassis.a"]);
        let mut ws =
            Workspace::new(fleet_project("T", &v1, &ids(&["cabin", "chassis", "old"])).unwrap());
        let v2 = truck_with_chassis_ids("1.3.0", &textures[..2], &["chassis.a", "chassis.b"]);
        assert!(ws.apply_update(&v2, &[], 1.0));
        let game_ids = |ws: &Workspace, i: usize| {
            ws.project.surfaces[i]
                .template
                .as_ref()
                .unwrap()
                .game_ids
                .clone()
        };
        assert_eq!(game_ids(&ws, 1), ids(&["chassis.a", "chassis.b"]));
        // "old" is not in 1.3.0: it keeps the ids it had.
        assert_eq!(game_ids(&ws, 2), ids(&["old.sample"]));
        ws.undo();
        assert_eq!(game_ids(&ws, 1), ids(&["chassis.a"]));
    }

    #[test]
    fn missing_game_data_is_filled_from_the_package() {
        let v = truck("1.1.0");
        let mut project = fleet_project("F", &v, &ids(&["high_roof", "chassis"])).unwrap();
        let recorded = project.clone();
        project.vehicles[0].game_data = None;
        for s in &mut project.surfaces {
            let t = s.template.as_mut().unwrap();
            t.game_ids.clear();
            t.main_index = None;
        }
        assert!(fill_game_data(&mut project, &v.manifest));
        assert_eq!(project, recorded);
        assert!(!fill_game_data(&mut project, &v.manifest), "already filled");
    }

    #[test]
    fn views_reset_when_the_surfaces_change() {
        let v = truck("1.0.0");
        let mut ws =
            Workspace::new(fleet_project("F", &v, &ids(&["standard", "chassis"])).unwrap());
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
        ws.set_textures(&v, &ids(&["standard", "high_roof", "chassis"]), 2.0)
            .unwrap();
        assert!(ws.viewport.is_none() && ws.viewports.is_empty());
        ws.viewport = Some(view);
        ws.undo();
        assert!(ws.viewport.is_none(), "undoing the change resets too");
        ws.set_active_surface(1);
        ws.redo();
        assert_eq!(ws.project.surfaces.len(), 3);
    }

    #[test]
    fn update_replaces_flags_resizes_and_removes() {
        let v1 = package(
            "1.2.0",
            &[
                tex("cabin", "Cabin", 4096, 1),
                tex("chassis", "Chassis", 2048, 1),
                tex("old", "Old", 512, 1),
            ],
        );
        let mut ws = Workspace::new(fleet_project("T", &v1, &ids(&["chassis", "old"])).unwrap());
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
            ]
        );
        assert_eq!(
            p.new,
            [NewTexture {
                id: "new".into(),
                name: "New".into(),
                part: TexturePart::Accessory,
                always: false
            }]
        );
        assert!(p.new[0].checked_by_default());
        let before_assets = ws.project.assets.len();
        assert!(ws.apply_update(&v2, &ids(&["new"]), 2.0));
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
    }

    #[test]
    fn sample_truck_update_flags_scales_and_offers_side_skirts() {
        let (v1, v2) = (truck("1.0.0"), truck("1.1.0"));
        let fleet = || {
            let mut ws =
                Workspace::new(fleet_project("F", &v1, &default_textures(&v1.manifest)).unwrap());
            ws.add_vehicle(&trailer(), &[], 1.0).unwrap();
            ws
        };
        let mut ws = fleet();
        let plan = plan(&ws.project, &v2).unwrap();
        assert_eq!(plan.vehicle, "TruckPaint Sample Truck");
        assert_eq!(
            plan.changes,
            [
                TextureChange::Replaced {
                    name: "Standard cab".into(),
                    layout_changed: true,
                    resized: None
                },
                TextureChange::Replaced {
                    name: "Chassis".into(),
                    layout_changed: false,
                    resized: Some((2048.0, 4096.0))
                },
                TextureChange::Replaced {
                    name: "Cab accessories".into(),
                    layout_changed: false,
                    resized: None
                },
            ]
        );
        // The High roof (a main texture not painted) and Side skirts are
        // offered; only the accessory is checked.
        let offered: Vec<(&str, bool)> = plan
            .new
            .iter()
            .map(|n| (n.id.as_str(), n.checked_by_default()))
            .collect();
        assert_eq!(offered, [("high_roof", false), ("side_skirts", true)]);
        // Unchecked: no Side skirts.
        let mut skipped = fleet();
        assert!(skipped.apply_update(&v2, &[], 2.0));
        assert!(!textures_of(&skipped.project, TRUCK).contains(&"side_skirts".to_owned()));
        // Checked: after the other accessories, before the trailer.
        assert!(ws.apply_update(&v2, &ids(&["side_skirts"]), 2.0));
        assert_eq!(
            keys(&ws.project)
                .iter()
                .map(|(_, t)| t.as_str())
                .collect::<Vec<_>>(),
            [
                "standard",
                "chassis",
                "cab_accessories",
                "side_skirts",
                "base"
            ]
        );
        assert_eq!(ws.project.vehicles[0].version, "1.1.0");
        assert_eq!(ws.project.vehicles[1].version, "1.0.0", "trailer untouched");
        assert_eq!(
            ws.project.surfaces[0].template.as_ref().unwrap().status,
            TemplateStatus::LayoutChanged
        );
        ws.undo();
        assert_eq!(ws.project.surfaces.len(), 4);
        assert_eq!(ws.project.vehicles[0].version, "1.0.0");
    }

    #[test]
    fn a_new_single_main_texture_is_always_added() {
        // The main texture changed id: the old one is marked, the new one
        // is added without being chosen.
        let v1 = package("1.2.0", &[tex("cabin", "Cabin", 1024, 1)]);
        let mut ws = Workspace::new(fleet_project("T", &v1, &[]).unwrap());
        let v2 = package("1.3.0", &[tex("cabin_2", "Cabin", 1024, 1)]);
        let p = plan(&ws.project, &v2).unwrap();
        assert!(p.new[0].always && p.new[0].checked_by_default());
        assert!(ws.apply_update(&v2, &[], 1.0));
        assert_eq!(
            textures_of(&ws.project, "scs.sample.truck"),
            ids(&["cabin", "cabin_2"])
        );
        assert_eq!(
            ws.project.surfaces[0].template.as_ref().unwrap().status,
            TemplateStatus::Removed
        );
    }

    #[test]
    fn mark_a_layout_change_as_checked() {
        let v1 = package("1.2.0", &[tex("cabin", "Cabin", 1024, 1)]);
        let mut ws = Workspace::new(fleet_project("T", &v1, &[]).unwrap());
        let v2 = package("1.3.0", &[tex("cabin", "Cabin", 1024, 2)]);
        ws.apply_update(&v2, &[], 1.0);
        let to_check = tp_core::TextureState::ToCheck(tp_core::CheckReason::LayoutChanged);
        assert_eq!(ws.project.surfaces[0].state(), to_check);
        ws.dismiss_layout_change(0, 2.0);
        assert_eq!(
            ws.project.surfaces[0].template.as_ref().unwrap().status,
            TemplateStatus::Current
        );
        assert_eq!(ws.history.undo_label(), Some("undo-mark-checked"));
        assert_eq!(ws.project.surfaces[0].state(), tp_core::TextureState::Empty);
        ws.undo();
        assert_eq!(
            ws.project.surfaces[0].template.as_ref().unwrap().status,
            TemplateStatus::LayoutChanged
        );
        assert_eq!(ws.project.surfaces[0].state(), to_check);
    }

    #[test]
    fn paint_on_the_chassis_and_undo_across_textures() {
        let p = package("1.2.0", &sample::truck_textures());
        let mut ws =
            Workspace::new(fleet_project("T", &p, &default_textures(&p.manifest)).unwrap());
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

    #[test]
    fn copy_from_cabin_scales_keeps_links_and_undoes() {
        use tp_core::document::{Frame, Object, SwatchId};
        let mut project = tp_core::Project::new("p", TextureResolution::R2048);
        project.surfaces[0].size = 2048.0;
        project.surfaces.push(Surface::new("High roof", 4096.0));
        let mut r = Object::new(
            ObjectId(0),
            ShapeKind::rectangle(),
            Frame::from_rect(Rect::new(0.0, 75.0, 200.0, 125.0)),
        );
        r.fill_swatch = Some(SwatchId(99));
        let source = project.add(r);
        let mut ws = Workspace::new(project);
        ws.set_active_surface(1);
        ws.copy_from_texture(0, 1.0);
        let copies = &ws.project.surfaces[1].objects;
        assert_eq!(copies.len(), 1);
        let copy = &copies[0];
        assert_ne!(copy.id, source, "fresh id");
        assert_eq!(copy.frame.center, tp_core::kurbo::Point::new(200.0, 200.0));
        assert_eq!(copy.frame.size, Size::new(400.0, 100.0));
        assert_eq!(ws.selection, vec![copy.id]);
        assert_eq!(ws.project.surfaces[0].objects.len(), 1, "source unchanged");
        ws.undo();
        assert!(ws.project.surfaces[1].objects.is_empty());
    }

    #[test]
    fn cabin_sources_are_main_textures_of_the_same_vehicle() {
        let mut textures = default_textures(&truck("1.1.0").manifest);
        textures.push("high_roof".into());
        let mut project = fleet_project("p", &truck("1.1.0"), &textures).unwrap();
        let names: Vec<&str> = project.surfaces.iter().map(|s| s.name.as_str()).collect();
        let high_roof = names.iter().position(|n| *n == "High roof").unwrap();
        let chassis = names.iter().position(|n| *n == "Chassis").unwrap();
        assert_eq!(cabin_sources(&project, 0), vec![high_roof]);
        assert_eq!(cabin_sources(&project, high_roof), vec![0]);
        assert!(cabin_sources(&project, chassis).is_empty(), "an accessory");
        let mut ws = Workspace::new(project);
        ws.add_vehicle(&trailer(), &default_textures(&trailer().manifest), 1.0)
            .unwrap();
        project = ws.project;
        let base = project
            .surfaces
            .iter()
            .position(|s| s.name == "Base")
            .unwrap();
        assert!(cabin_sources(&project, base).is_empty(), "a trailer");
    }
}
