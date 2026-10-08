//! Mod export: what blocks it, the mod's files (definitions, textures,
//! icon, manifest) planned from the project, and the background job that
//! renders and writes them as one `.scs` archive.

use std::collections::BTreeMap;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use tp_core::document::Rgba;
use tp_core::{ModSettings, Project, ProjectVehicle, TemplateStatus, TexturePart};
use tp_i18n::tr;
use tp_render::{DdsEncoding, Pixmap, RenderOptions};
use tp_text::FontLibrary;
use tp_vehicles::Manifest;

use crate::export::ExportOutcome;
use crate::workspace::Workspace;

/// Size of the shop icon, in pixels.
pub const ICON_SIZE: (u32, u32) = (256, 64);
/// Size of the Mod Manager image, in pixels.
pub const IMAGE_SIZE: (u32, u32) = (276, 162);
/// Extension of a mod archive.
pub const EXTENSION: &str = "scs";

/// The folder name of game `game` (`ets2` or `ats`).
fn game_folder(game: &str) -> Option<&'static str> {
    match game {
        "ets2" => Some("Euro Truck Simulator 2"),
        "ats" => Some("American Truck Simulator"),
        _ => None,
    }
}

/// Where game `game` reads mods on this platform: in the user's documents
/// folder on Windows, and in the user's data folder elsewhere
/// (`~/Library/Application Support` on macOS, `~/.local/share` on Linux).
pub fn mod_folder_in(game: &str, documents: Option<&Path>, data: Option<&Path>) -> Option<PathBuf> {
    let base = if cfg!(windows) { documents } else { data }?;
    Some(base.join(game_folder(game)?).join("mod"))
}

/// The mod folder of game `game` under `documents` or `data` (see
/// [`mod_folder_in`]), when it exists.
pub fn existing_mod_folder(
    game: &str,
    documents: Option<&Path>,
    data: Option<&Path>,
) -> Option<PathBuf> {
    mod_folder_in(game, documents, data).filter(|p| p.is_dir())
}

/// The mod folder of game `game`, when it exists on this computer.
pub fn game_mod_folder(game: &str) -> Option<PathBuf> {
    let user = directories::UserDirs::new();
    let base = directories::BaseDirs::new();
    existing_mod_folder(
        game,
        user.as_ref().and_then(|u| u.document_dir()),
        base.as_ref().map(|b| b.data_dir()),
    )
}

/// The file name proposed for a mod named `name`.
pub fn suggested_name(name: &str) -> String {
    let stem = crate::export::file_stem_safe(name);
    let stem = if stem.is_empty() { "mod" } else { &stem };
    format!("{stem}.{EXTENSION}")
}

/// `path` with the `.scs` extension, added when the user left it out.
pub fn with_extension(path: PathBuf) -> PathBuf {
    if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(EXTENSION))
    {
        return path;
    }
    let mut name = path.into_os_string();
    name.push(".");
    name.push(EXTENSION);
    PathBuf::from(name)
}

/// Something that blocks the export.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    NameEmpty,
    /// The Name contains `"`, `\` or a line break.
    NameInvalid,
    VersionInvalid,
    AuthorInvalid,
    InternalNameEmpty,
    /// A character other than `a`–`z`, `0`–`9` and `_`.
    InternalNameInvalid,
    InternalNameTooLong {
        max: usize,
    },
    PriceZero,
    /// Two vehicles share a game path.
    SamePath {
        path: String,
        first: String,
        second: String,
    },
    /// A vehicle's game data is missing: that package version must be
    /// installed.
    MissingGameData {
        vehicle: String,
        version: String,
    },
}

/// Whether `text` can be written inside a quoted SII string.
fn sii_safe(text: &str) -> bool {
    !text.contains(['"', '\\', '\n', '\r'])
}

/// Everything that blocks exporting `project`, in the dialog's order.
pub fn problems(project: &Project) -> Vec<Problem> {
    problems_with(project, &project.mod_settings)
}

/// Everything that blocks exporting `project` with mod settings `s`.
pub fn problems_with(project: &Project, s: &ModSettings) -> Vec<Problem> {
    let mut out = Vec::new();
    if s.name.trim().is_empty() {
        out.push(Problem::NameEmpty);
    } else if !sii_safe(&s.name) {
        out.push(Problem::NameInvalid);
    }
    if !sii_safe(&s.version) {
        out.push(Problem::VersionInvalid);
    }
    if !sii_safe(&s.author) {
        out.push(Problem::AuthorInvalid);
    }
    let max = project.internal_name_limit();
    let internal = s.internal_name(max);
    if internal.is_empty() {
        out.push(Problem::InternalNameEmpty);
    } else if !internal
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        out.push(Problem::InternalNameInvalid);
    } else if internal.chars().count() > max {
        out.push(Problem::InternalNameTooLong { max });
    }
    if s.price == 0 {
        out.push(Problem::PriceZero);
    }
    let mut paths: BTreeMap<&str, &str> = BTreeMap::new();
    for v in &project.vehicles {
        let Some(data) = &v.game_data else {
            out.push(Problem::MissingGameData {
                vehicle: v.name.clone(),
                version: v.version.clone(),
            });
            continue;
        };
        if let Some(first) = paths.insert(&data.path, &v.name) {
            out.push(Problem::SamePath {
                path: data.path.clone(),
                first: first.to_owned(),
                second: v.name.clone(),
            });
        }
    }
    out
}

/// What the mod holds for one vehicle (the dialog's summary).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VehicleSummary {
    pub name: String,
    /// Main textures in package order, painted or not.
    pub mains: Vec<MainSummary>,
    /// Names of the painted accessories.
    pub accessories: Vec<String>,
}

/// A main texture of a vehicle in the summary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MainSummary {
    pub name: String,
    /// The cabins it is painted on (none: every cabin).
    pub cabins: Vec<String>,
    pub painted: bool,
}

/// What the mod holds for each vehicle of `project`. `manifest` gives the
/// installed package version of a vehicle, to name the main textures it
/// doesn't paint (left out when it isn't installed).
pub fn summary(
    project: &Project,
    manifest: &dyn Fn(&ProjectVehicle) -> Option<Manifest>,
) -> Vec<VehicleSummary> {
    project
        .vehicles
        .iter()
        .map(|v| {
            let mut mains: Vec<(usize, MainSummary)> = Vec::new();
            let mut accessories = Vec::new();
            for i in project.vehicle_range(&v.package_id) {
                let surface = &project.surfaces[i];
                let Some(t) = surface
                    .template
                    .as_ref()
                    .filter(|t| t.status != TemplateStatus::Removed)
                else {
                    continue;
                };
                match t.part {
                    TexturePart::Main => mains.push((
                        t.main_index.unwrap_or(mains.len()),
                        MainSummary {
                            name: surface.name.clone(),
                            cabins: t.game_ids.clone(),
                            painted: true,
                        },
                    )),
                    TexturePart::Accessory => accessories.push(surface.name.clone()),
                }
            }
            if let Some(m) = manifest(v) {
                for (index, part) in m.paint_job.main.iter().enumerate() {
                    if !mains.iter().any(|(i, _)| *i == index) {
                        mains.push((
                            index,
                            MainSummary {
                                name: part.name.clone(),
                                cabins: part.game_ids.clone(),
                                painted: false,
                            },
                        ));
                    }
                }
            }
            mains.sort_by_key(|(i, _)| *i);
            VehicleSummary {
                name: v.name.clone(),
                mains: mains.into_iter().map(|(_, m)| m).collect(),
                accessories,
            }
        })
        .collect()
}

/// One file of the mod.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    /// A definition, material or text file.
    Text(String),
    /// A `.tobj`.
    Bytes(Vec<u8>),
    /// Surface `surface` rendered at `size` pixels, as DDS.
    Texture { surface: usize, size: u32 },
    /// The shop icon, as DDS.
    Icon,
    /// The Mod Manager image, as JPEG.
    Image,
}

/// The mod's files by path in the archive, sorted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModPlan {
    pub files: BTreeMap<String, Entry>,
}

impl ModPlan {
    /// The text of file `path`, if it is a text file.
    pub fn text(&self, path: &str) -> Option<&str> {
        match self.files.get(path)? {
            Entry::Text(t) => Some(t),
            _ => None,
        }
    }

    /// Total pixels to render: textures, icon and image.
    pub fn pixels(&self) -> u64 {
        self.files
            .values()
            .map(|e| match e {
                Entry::Texture { size, .. } => u64::from(*size) * u64::from(*size),
                Entry::Icon => u64::from(ICON_SIZE.0 * ICON_SIZE.1),
                Entry::Image => u64::from(IMAGE_SIZE.0 * IMAGE_SIZE.1),
                Entry::Text(_) | Entry::Bytes(_) => 0,
            })
            .sum()
    }
}

/// A binary `.tobj` pointing to the absolute `dds` path (`/vehicle/….dds`):
/// the header the games expect, then the path's length and the path.
pub fn tobj(dds: &str) -> Vec<u8> {
    const HEADER: [u8; 40] = [
        0x01, 0x0A, 0xB1, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x02, 0x00, 0x03, 0x03, 0x03, 0x00,
        0x02, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
    ];
    let mut out = HEADER.to_vec();
    out.extend_from_slice(&(dds.len() as u64).to_le_bytes());
    out.extend_from_slice(dds.as_bytes());
    out
}

/// The game folder of a vehicle kind.
fn game_type(vehicle: &ProjectVehicle) -> &'static str {
    if vehicle.kind == "trailer" {
        "trailer_owned"
    } else {
        "truck"
    }
}

/// `id` as a file name: `a`–`z`, `0`–`9`, `_` and `-`.
fn file_stem(id: &str) -> String {
    id.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// The name of the paint job of main texture `index` among `count`.
fn unit_name(internal: &str, count: usize, index: usize) -> String {
    if count <= 1 {
        return internal.to_owned();
    }
    match u8::try_from(index).ok().filter(|i| *i < 26) {
        Some(i) => format!("{internal}_{}", char::from(b'a' + i)),
        None => format!("{internal}_{index}"),
    }
}

/// A painted texture of a vehicle.
struct Painted<'a> {
    part: TexturePart,
    game_ids: &'a [String],
    main_index: Option<usize>,
    /// The texture's `.tobj`, absolute.
    tobj: String,
}

/// The mod's files for `project`, or what blocks the export.
pub fn plan(project: &Project) -> Result<ModPlan, Vec<Problem>> {
    let problems = problems(project);
    if !problems.is_empty() {
        return Err(problems);
    }
    let s = &project.mod_settings;
    let id = project.internal_name();
    let mut files = BTreeMap::new();
    files.insert(
        "manifest.sii".to_owned(),
        Entry::Text(format!(
            "SiiNunit\n{{\nmod_package : .package_name\n{{\n\
             \tpackage_version: \"{}\"\n\
             \tdisplay_name: \"{}\"\n\
             \tauthor: \"{}\"\n\
             \tcategory[]: \"paint_job\"\n\
             \tmp_mod_optional: true\n\
             \ticon: \"icon.jpg\"\n\
             \tdescription_file: \"description.txt\"\n\
             }}\n}}\n",
            s.version, s.name, s.author
        )),
    );
    files.insert(
        "description.txt".to_owned(),
        Entry::Text(description(project)),
    );
    files.insert(
        format!("material/ui/accessory/{id}_icon.mat"),
        Entry::Text(format!(
            "material: \"ui\"\n{{\n\ttexture: \"{id}_icon.tobj\"\n\ttexture_name: \"texture\"\n}}\n"
        )),
    );
    files.insert(
        format!("material/ui/accessory/{id}_icon.tobj"),
        Entry::Bytes(tobj(&format!("/material/ui/accessory/{id}_icon.dds"))),
    );
    files.insert(format!("material/ui/accessory/{id}_icon.dds"), Entry::Icon);
    files.insert("icon.jpg".into(), Entry::Image);

    for vehicle in &project.vehicles {
        let data = vehicle.game_data.as_ref().expect("checked by problems()");
        let kind = game_type(vehicle);
        let def = format!("def/vehicle/{kind}/{}/paint_job", data.path);
        let textures = format!("vehicle/{kind}/upgrade/paintjob/{id}/{}", data.path);
        let mut painted = Vec::new();
        let mut stems: Vec<String> = Vec::new();
        for i in project.vehicle_range(&vehicle.package_id) {
            let surface = &project.surfaces[i];
            let Some(t) = &surface.template else {
                continue;
            };
            // A texture the package version no longer has is left out.
            if t.status == TemplateStatus::Removed {
                continue;
            }
            let mut stem = file_stem(&t.texture_id);
            if stems.contains(&stem) {
                stem = format!("{stem}_{i}");
            }
            stems.push(stem.clone());
            let size = surface.size.round() as u32;
            files.insert(
                format!("{textures}/{stem}.dds"),
                Entry::Texture { surface: i, size },
            );
            let tobj_path = format!("{textures}/{stem}.tobj");
            files.insert(
                tobj_path.clone(),
                Entry::Bytes(tobj(&format!("/{textures}/{stem}.dds"))),
            );
            painted.push(Painted {
                part: t.part,
                game_ids: &t.game_ids,
                main_index: t.main_index,
                tobj: format!("/{tobj_path}"),
            });
        }
        let mut settings = format!(
            "\tname: \"{}\"\n\tprice: {}\n\tunlock: {}\n\ticon: \"{id}_icon\"\n\tairbrush: true\n",
            s.name, s.price, s.unlock_level
        );
        if data.alt_uv {
            settings.push_str("\talternate_uvset: true\n");
        }
        if data.colour_picker {
            settings.push_str("\tbase_color_locked: false\n");
        }
        files.insert(format!("{def}/{id}_settings.sui"), Entry::Text(settings));
        let accessories: Vec<&Painted> = painted
            .iter()
            .filter(|p| p.part == TexturePart::Accessory)
            .collect();
        let accessory_file = (!accessories.is_empty()).then(|| {
            let mut out = String::from("SiiNunit\n{\n");
            for (n, a) in accessories.iter().enumerate() {
                out.push_str(&format!(
                    "\nsimple_paint_job_data : .ovr{n}\n{{\n\tpaint_job_mask: \"{}\"\n",
                    a.tobj
                ));
                for game_id in a.game_ids {
                    out.push_str(&format!("\tacc_list[]: \"{game_id}\"\n"));
                }
                out.push_str("}\n");
            }
            out.push_str("}\n");
            out
        });
        for main in painted.iter().filter(|p| p.part == TexturePart::Main) {
            let unit = unit_name(&id, data.main_count, main.main_index.unwrap_or(0));
            let mut out = format!(
                "SiiNunit\n{{\naccessory_paint_job_data : {unit}.{path}.paint_job\n{{\n\
                 @include \"{id}_settings.sui\"\n",
                path = data.path
            );
            for cabin in main.game_ids {
                out.push_str(&format!(
                    "\tsuitable_for[]: \"{cabin}.{}.cabin\"\n",
                    data.path
                ));
            }
            out.push_str(&format!("\tpaint_job_mask: \"{}\"\n}}\n}}\n", main.tobj));
            files.insert(format!("{def}/{unit}.sii"), Entry::Text(out));
            if let Some(acc) = &accessory_file {
                files.insert(
                    format!("{def}/accessory/{unit}.sii"),
                    Entry::Text(acc.clone()),
                );
            }
        }
    }
    Ok(ModPlan { files })
}

/// `description.txt`: the player's description, the vehicles and the mods
/// they require.
fn description(project: &Project) -> String {
    let mut out = String::new();
    let own = project.mod_settings.description.trim_end();
    if !own.is_empty() {
        out.push_str(own);
        out.push_str("\n\n");
    }
    out.push_str("Vehicles supported:\n");
    for v in &project.vehicles {
        out.push_str(&format!("- {}\n", v.name));
    }
    let mut required: Vec<String> = Vec::new();
    for r in project
        .vehicles
        .iter()
        .filter_map(|v| v.game_data.as_ref())
        .flat_map(|g| &g.requires)
    {
        let line = match &r.version {
            Some(version) => format!("- {} {version}", r.name),
            None => format!("- {}", r.name),
        };
        if !required.contains(&line) {
            required.push(line);
        }
    }
    if !required.is_empty() {
        out.push_str("\nRequires:\n");
        for line in required {
            out.push_str(&line);
            out.push('\n');
        }
    }
    out
}

/// A picture chosen for the mod, not yet in the project.
#[derive(Clone, Debug, PartialEq)]
pub struct PictureFile {
    pub name: String,
    pub bytes: Arc<[u8]>,
    /// Pixel size.
    pub size: (u32, u32),
}

impl PictureFile {
    /// The picture of PNG or JPEG `bytes`, or why it can't be read.
    pub fn read(name: &str, bytes: Vec<u8>) -> Result<Self, String> {
        let image = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
        Ok(Self {
            name: name.to_owned(),
            bytes: Arc::from(bytes),
            size: (image.width(), image.height()),
        })
    }
}

/// Where a mod picture comes from while the dialog is open.
#[derive(Clone, Debug, PartialEq)]
pub enum Picture {
    /// Rendered from the first texture.
    Generated,
    /// A chosen picture already in the project.
    Asset(tp_core::document::AssetId),
    /// A newly chosen picture.
    File(PictureFile),
}

impl Picture {
    /// The picture of the chosen asset `asset`, if any.
    pub fn of(asset: Option<tp_core::document::AssetId>) -> Self {
        asset.map_or(Self::Generated, Self::Asset)
    }

    /// The image bytes, unless generated.
    pub fn bytes<'a>(&'a self, project: &'a Project) -> Option<&'a [u8]> {
        match self {
            Self::Generated => None,
            Self::Asset(a) => chosen_bytes(project, Some(*a)),
            Self::File(f) => Some(&f.bytes),
        }
    }

    /// Adds a newly chosen picture to the project; returns its asset.
    fn add_to(&self, project: &mut Project) -> Option<tp_core::document::AssetId> {
        match self {
            Self::Generated => None,
            Self::Asset(a) => Some(*a),
            Self::File(f) => Some(
                project
                    .add_asset(
                        &f.name,
                        tp_core::AssetKind::Raster,
                        f.bytes.clone(),
                        tp_core::kurbo::Size::new(f64::from(f.size.0), f64::from(f.size.1)),
                    )
                    .0,
            ),
        }
    }
}

impl Workspace {
    /// Sets the mod settings, with `icon` and `image` as their pictures, as
    /// one undo step. Newly chosen pictures become assets, and the pictures
    /// no longer used are removed. Records nothing when nothing changed.
    pub fn set_mod_settings(
        &mut self,
        mut settings: ModSettings,
        icon: &Picture,
        image: &Picture,
        now: f64,
    ) {
        self.edit("undo-edit-mod-settings", now, false, |project, _| {
            let previous = [project.mod_settings.icon, project.mod_settings.image];
            settings.icon = icon.add_to(project);
            settings.image = image.add_to(project);
            project.mod_settings = settings;
            for asset in previous.into_iter().flatten() {
                project.remove_asset(asset);
            }
        });
    }
}

/// Background of the generated pictures.
const PICTURE_BACKGROUND: Rgba = Rgba::rgb(255, 255, 255);
/// JPEG quality of the Mod Manager image.
const JPEG_QUALITY: u8 = 90;

/// The mod's picture of `size`: the image file `chosen` (PNG or JPEG)
/// scaled to cover it, or else the project's first texture.
pub fn picture(
    project: &Project,
    chosen: Option<&[u8]>,
    size: (u32, u32),
    fonts: &mut FontLibrary,
) -> Result<Pixmap, String> {
    match chosen {
        Some(bytes) => {
            let image = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
            Ok(tp_render::cover_image(&image.to_rgba8(), size.0, size.1))
        }
        None => Ok(tp_render::render_cover(
            project,
            0,
            size.0,
            size.1,
            Some(PICTURE_BACKGROUND),
            fonts,
        )),
    }
}

/// The bytes of the chosen mod image `asset`, if any.
fn chosen_bytes(project: &Project, asset: Option<tp_core::document::AssetId>) -> Option<&[u8]> {
    project.assets.get(&asset?).map(|a| &*a.bytes)
}

/// Renders, encodes and zips the mod, then writes it (worker side).
fn run_mod_export(
    project: &Project,
    plan: &ModPlan,
    mut fonts: FontLibrary,
    path: &Path,
    progress: &AtomicU32,
    cancel: &AtomicBool,
    notify: &(dyn Fn() + Send + Sync),
) -> ExportOutcome {
    let failed = |reason: String| ExportOutcome::Failed {
        path: path.to_path_buf(),
        reason,
    };
    let cancelled = || cancel.load(Ordering::Relaxed);
    // Progress up to 950 is shared by the pictures in proportion to their
    // pixels; writing the file takes the rest.
    let total = plan.pixels().max(1) as f64;
    let mut done = 0u64;
    let set = |done: u64, part: f64, weight: u64| {
        let value = (done as f64 + part * weight as f64) / total * 950.0;
        progress.store(value as u32, Ordering::Relaxed);
        notify();
    };
    let stored = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .last_modified_time(zip::DateTime::default())
        .unix_permissions(0o644);
    let deflated = stored.compression_method(zip::CompressionMethod::Deflated);
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, entry) in &plan.files {
        let (bytes, options) = match entry {
            Entry::Text(text) => (text.clone().into_bytes(), deflated),
            Entry::Bytes(bytes) => (bytes.clone(), deflated),
            Entry::Texture { surface, size } => {
                let weight = u64::from(*size) * u64::from(*size);
                let options = RenderOptions {
                    size: *size,
                    background: None,
                };
                let rendered =
                    tp_render::render(project, *surface, options, &mut fonts, &mut |i, n| {
                        set(done, 0.7 * i as f64 / n.max(1) as f64, weight);
                        !cancelled()
                    });
                let Ok(pixmap) = rendered else {
                    return ExportOutcome::Cancelled;
                };
                let encoded =
                    tp_render::encode_dds(&pixmap, DdsEncoding::Bc3, &mut |level, levels| {
                        set(
                            done,
                            0.7 + 0.3 * level as f64 / levels.max(1) as f64,
                            weight,
                        );
                        !cancelled()
                    });
                let Ok(bytes) = encoded else {
                    return ExportOutcome::Cancelled;
                };
                done += weight;
                (bytes, stored)
            }
            Entry::Icon => {
                let chosen = chosen_bytes(project, project.mod_settings.icon);
                let pixmap = match picture(project, chosen, ICON_SIZE, &mut fonts) {
                    Ok(p) => p,
                    Err(reason) => return failed(reason),
                };
                let Ok(bytes) =
                    tp_render::encode_dds(&pixmap, DdsEncoding::Bc3, &mut |_, _| !cancelled())
                else {
                    return ExportOutcome::Cancelled;
                };
                done += u64::from(ICON_SIZE.0 * ICON_SIZE.1);
                (bytes, stored)
            }
            Entry::Image => {
                let chosen = chosen_bytes(project, project.mod_settings.image);
                let encoded = picture(project, chosen, IMAGE_SIZE, &mut fonts)
                    .and_then(|p| tp_render::encode_jpeg(&p, JPEG_QUALITY));
                match encoded {
                    Ok(bytes) => {
                        done += u64::from(IMAGE_SIZE.0 * IMAGE_SIZE.1);
                        (bytes, stored)
                    }
                    Err(reason) => return failed(reason),
                }
            }
        };
        if cancelled() {
            return ExportOutcome::Cancelled;
        }
        set(done, 0.0, 0);
        // Writing into memory cannot fail.
        zip.start_file(name.as_str(), options).expect("zip");
        zip.write_all(&bytes).expect("zip");
    }
    let bytes = zip.finish().expect("zip").into_inner();
    if cancelled() {
        return ExportOutcome::Cancelled;
    }
    match tp_file::write_atomic(path, &bytes) {
        Ok(()) => {
            progress.store(1000, Ordering::Relaxed);
            notify();
            ExportOutcome::Written(path.to_path_buf())
        }
        Err(err) => failed(match err {
            tp_file::Error::Io(e) => e.to_string(),
            other => other.to_string(),
        }),
    }
}

/// A mod export running on a worker thread.
pub struct ModJob {
    pub path: PathBuf,
    progress: Arc<AtomicU32>,
    cancel: Arc<AtomicBool>,
    done: Receiver<ExportOutcome>,
}

impl ModJob {
    /// Starts writing the mod `plan` of `project` (a snapshot: editing may
    /// continue) to `path`.
    pub fn start(
        project: Project,
        plan: ModPlan,
        fonts: FontLibrary,
        path: PathBuf,
        notify: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        let progress = Arc::new(AtomicU32::new(0));
        let cancel = Arc::new(AtomicBool::new(false));
        let (tx, done) = mpsc::channel();
        let (p, c, target) = (progress.clone(), cancel.clone(), path.clone());
        let spawned = thread::Builder::new()
            .name("mod-export".into())
            .spawn(move || {
                let outcome = run_mod_export(&project, &plan, fonts, &target, &p, &c, &notify);
                let _ = tx.send(outcome);
                notify();
            });
        if let Err(err) = spawned {
            tracing::error!(%err, "cannot start the mod export");
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
                reason: tr("export-stopped"),
            }),
        }
    }

    /// Blocks until the export ends (tests).
    pub fn wait(&self) -> ExportOutcome {
        self.done.recv().unwrap_or(ExportOutcome::Failed {
            path: self.path.clone(),
            reason: tr("export-stopped"),
        })
    }
}

#[cfg(test)]
mod tests {
    use tp_core::RequiredMod;
    use tp_vehicles::Package;

    use super::*;
    use crate::vehicle_project::fleet_project;
    use crate::workspace::Workspace;

    /// A version of a committed sample package.
    fn example(name: &str, version: &str) -> Package {
        let path = format!(
            "{}/../../examples/vehicles/community.truckpaint.{name}-{version}.tpv",
            env!("CARGO_MANIFEST_DIR")
        );
        Package::read(&std::fs::read(path).unwrap()).unwrap()
    }

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    /// A project painting `textures` of the sample truck 1.1.0, with
    /// internal name "ace".
    fn truck(textures: &[&str]) -> Project {
        let mut p =
            fleet_project("ACE", &example("sample_truck", "1.1.0"), &ids(textures)).unwrap();
        p.mod_settings.internal_name = Some("ace".into());
        p
    }

    fn trailer(textures: &[&str]) -> Project {
        let mut p =
            fleet_project("ACE", &example("sample_trailer", "1.0.0"), &ids(textures)).unwrap();
        p.mod_settings.internal_name = Some("ace".into());
        p
    }

    /// A small truck (256 px textures: main "cabin", accessory "chassis")
    /// with internal name "ace" and a rectangle on its cabin.
    fn small_truck() -> Project {
        use tp_core::document::{Frame, Object, ObjectId, ShapeKind};
        use tp_core::kurbo::{Point, Size};
        let textures = [
            tp_vehicles::sample::SampleTexture {
                id: "cabin",
                name: "Cabin",
                size: 256,
                layout: 1,
            },
            tp_vehicles::sample::SampleTexture {
                id: "chassis",
                name: "Chassis",
                size: 256,
                layout: 1,
            },
        ];
        let package = Package::read(&tp_vehicles::sample::package(
            "scs.sample.truck",
            "Sample Truck",
            "1.0.0",
            &textures,
        ))
        .unwrap();
        let mut p = fleet_project("ACE", &package, &ids(&["cabin", "chassis"])).unwrap();
        p.mod_settings.internal_name = Some("ace".into());
        p.add(Object::new(
            ObjectId(0),
            ShapeKind::rectangle(),
            Frame::new(Point::new(128.0, 128.0), Size::new(100.0, 60.0), 0.0),
        ));
        p
    }

    /// Starts exporting `project` to `path` and waits for the outcome.
    fn export(project: &Project, path: &Path) -> ExportOutcome {
        let plan = plan(project).unwrap();
        ModJob::start(
            project.clone(),
            plan,
            FontLibrary::bundled(),
            path.to_path_buf(),
            || {},
        )
        .wait()
    }

    fn archive(path: &Path) -> zip::ZipArchive<std::fs::File> {
        zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap()
    }

    fn read_entry(zip: &mut zip::ZipArchive<std::fs::File>, name: &str) -> Vec<u8> {
        let mut out = Vec::new();
        std::io::Read::read_to_end(&mut zip.by_name(name).unwrap(), &mut out).unwrap();
        out
    }

    fn u32_at(bytes: &[u8], at: usize) -> u32 {
        u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
    }

    #[test]
    fn the_job_writes_the_planned_mod() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ACE.scs");
        let project = truck(&["standard", "side_skirts"]);
        assert_eq!(
            export(&project, &path),
            ExportOutcome::Written(path.clone())
        );
        let mut zip = archive(&path);
        let mut names: Vec<String> = zip.file_names().map(str::to_owned).collect();
        names.sort();
        let planned: Vec<String> = plan(&project).unwrap().files.into_keys().collect();
        assert_eq!(names, planned);
        let dds = read_entry(&mut zip, &format!("{TRUCK_TEX}/side_skirts.dds"));
        assert_eq!(&dds[84..88], b"DXT5");
        assert_eq!((u32_at(&dds, 12), u32_at(&dds, 16)), (1024, 1024));
        assert_eq!(u32_at(&dds, 28), 11, "mip levels");
        let icon = read_entry(&mut zip, "material/ui/accessory/ace_icon.dds");
        assert_eq!((u32_at(&icon, 16), u32_at(&icon, 12)), ICON_SIZE);
        let image = image::load_from_memory(&read_entry(&mut zip, "icon.jpg")).unwrap();
        assert_eq!((image.width(), image.height()), IMAGE_SIZE);
        // No temporary file left behind.
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn cancelling_leaves_no_file_and_keeps_an_earlier_one() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ACE.scs");
        std::fs::write(&path, b"earlier export").unwrap();
        // 4096 px textures: the job can't end before it is cancelled.
        let project = truck(&["standard"]);
        let job = ModJob::start(
            project.clone(),
            plan(&project).unwrap(),
            FontLibrary::bundled(),
            path.clone(),
            || {},
        );
        job.cancel();
        assert_eq!(job.wait(), ExportOutcome::Cancelled);
        assert_eq!(std::fs::read(&path).unwrap(), b"earlier export");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn an_unwritable_destination_is_reported() {
        let path = PathBuf::from("/nonexistent-dir/ACE.scs");
        assert!(matches!(
            export(&small_truck(), &path),
            ExportOutcome::Failed { path: p, .. } if p == path
        ));
    }

    #[test]
    fn exports_are_reproducible() {
        let dir = tempfile::tempdir().unwrap();
        let (a, b) = (dir.path().join("a.scs"), dir.path().join("b.scs"));
        let project = small_truck();
        export(&project, &a);
        export(&project, &b);
        assert_eq!(std::fs::read(&a).unwrap(), std::fs::read(&b).unwrap());
    }

    #[test]
    fn a_chosen_image_is_used() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ACE.scs");
        let mut project = small_truck();
        let red = image::RgbaImage::from_pixel(1280, 720, image::Rgba([255, 0, 0, 255]));
        let mut png = Vec::new();
        red.write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        let (asset, _) = project.add_asset(
            "Picture",
            tp_core::AssetKind::Raster,
            Arc::from(png.as_slice()),
            tp_core::kurbo::Size::new(1280.0, 720.0),
        );
        project.mod_settings.image = Some(asset);
        export(&project, &path);
        let jpeg = read_entry(&mut archive(&path), "icon.jpg");
        let image = image::load_from_memory(&jpeg).unwrap().to_rgb8();
        let [r, g, b] = image.get_pixel(138, 81).0;
        assert!(r > 240 && g < 20 && b < 20, "{r} {g} {b}");
    }

    #[test]
    fn mod_folders_per_platform() {
        let documents = Path::new("/home/jane/Documents");
        let data = Path::new("/home/jane/data");
        let base = if cfg!(windows) { documents } else { data };
        assert_eq!(
            mod_folder_in("ets2", Some(documents), Some(data)),
            Some(base.join("Euro Truck Simulator 2").join("mod"))
        );
        assert_eq!(
            mod_folder_in("ats", Some(documents), Some(data)),
            Some(base.join("American Truck Simulator").join("mod"))
        );
        assert_eq!(mod_folder_in("other", Some(documents), Some(data)), None);
        assert_eq!(mod_folder_in("ets2", None, None), None);
    }

    #[test]
    fn only_an_existing_mod_folder_is_used() {
        let dir = tempfile::tempdir().unwrap();
        let some = Some(dir.path());
        assert_eq!(existing_mod_folder("ets2", some, some), None);
        let folder = mod_folder_in("ets2", some, some).unwrap();
        std::fs::create_dir_all(&folder).unwrap();
        assert_eq!(existing_mod_folder("ets2", some, some), Some(folder));
        assert_eq!(existing_mod_folder("ats", some, some), None);
    }

    #[test]
    fn names_and_extension() {
        assert_eq!(suggested_name("ACE Logistics"), "ACE Logistics.scs");
        assert_eq!(suggested_name("A/B: C"), "A-B- C.scs");
        assert_eq!(suggested_name(".."), "mod.scs");
        assert_eq!(with_extension("/a/ace".into()), PathBuf::from("/a/ace.scs"));
        assert_eq!(
            with_extension("/a/ace.SCS".into()),
            PathBuf::from("/a/ace.SCS")
        );
    }

    const TRUCK_DEF: &str = "def/vehicle/truck/truckpaint.sample/paint_job";
    const TRUCK_TEX: &str = "vehicle/truck/upgrade/paintjob/ace/truckpaint.sample";

    #[test]
    fn a_valid_project_has_no_problem() {
        assert_eq!(problems(&truck(&["standard"])), []);
    }

    #[test]
    fn each_problem_is_reported() {
        let check = |edit: &dyn Fn(&mut Project), expected: Problem| {
            let mut p = truck(&["standard"]);
            edit(&mut p);
            assert_eq!(problems(&p), [expected]);
        };
        check(&|p| p.mod_settings.name = " ".into(), Problem::NameEmpty);
        check(
            &|p| p.mod_settings.name = "A \"B\"".into(),
            Problem::NameInvalid,
        );
        check(
            &|p| p.mod_settings.version = "1\\0".into(),
            Problem::VersionInvalid,
        );
        check(
            &|p| p.mod_settings.author = "A\nB".into(),
            Problem::AuthorInvalid,
        );
        check(
            &|p| p.mod_settings.internal_name = Some(String::new()),
            Problem::InternalNameEmpty,
        );
        check(
            &|p| p.mod_settings.internal_name = Some("Ace".into()),
            Problem::InternalNameInvalid,
        );
        // The sample truck has two main textures: 10 characters at most.
        check(
            &|p| p.mod_settings.internal_name = Some("ace_logistic".into()),
            Problem::InternalNameTooLong { max: 10 },
        );
        check(&|p| p.mod_settings.price = 0, Problem::PriceZero);
        check(
            &|p| p.vehicles[0].game_data = None,
            Problem::MissingGameData {
                vehicle: "TruckPaint Sample Truck".into(),
                version: "1.1.0".into(),
            },
        );
        let mut p = truck(&["standard"]);
        let mut twin = p.vehicles[0].clone();
        twin.name = "Twin".into();
        p.vehicles.push(twin);
        assert_eq!(
            problems(&p),
            [Problem::SamePath {
                path: "truckpaint.sample".into(),
                first: "TruckPaint Sample Truck".into(),
                second: "Twin".into(),
            }]
        );
        assert!(plan(&p).is_err());
    }

    #[test]
    fn summary_lists_painted_and_unpainted_cabins() {
        let package = example("sample_truck", "1.1.0");
        let p = truck(&["standard", "chassis", "cab_accessories"]);
        let manifest = package.manifest.clone();
        let list = summary(&p, &|_| Some(manifest.clone()));
        assert_eq!(
            list,
            [VehicleSummary {
                name: "TruckPaint Sample Truck".into(),
                mains: vec![
                    MainSummary {
                        name: "Standard cab".into(),
                        cabins: ids(&["standard"]),
                        painted: true,
                    },
                    MainSummary {
                        name: "High roof".into(),
                        cabins: ids(&["high_roof"]),
                        painted: false,
                    },
                ],
                accessories: ids(&["Chassis", "Cab accessories"]),
            }]
        );
        // Without the package, only what is painted.
        assert_eq!(summary(&p, &|_| None)[0].mains.len(), 1);
    }

    #[test]
    fn problems_of_edited_settings() {
        let p = truck(&["standard"]);
        let mut s = p.mod_settings.clone();
        s.price = 0;
        assert_eq!(problems_with(&p, &s), [Problem::PriceZero]);
        assert_eq!(problems(&p), []);
    }

    #[test]
    fn tobj_matches_paintjob_packer() {
        let reference = include_bytes!("../tests/fixtures/paintjob-packer.tobj");
        assert_eq!(
            tobj("/vehicle/truck/upgrade/paintjob/ace/truckpaint.sample/standard.dds"),
            reference
        );
    }

    #[test]
    fn truck_with_two_cabin_layouts() {
        let plan = plan(&truck(&["standard", "high_roof", "chassis", "side_skirts"])).unwrap();
        let paths: Vec<&str> = plan.files.keys().map(String::as_str).collect();
        let mut expected = vec![
            "description.txt".to_owned(),
            "icon.jpg".into(),
            "manifest.sii".into(),
            "material/ui/accessory/ace_icon.dds".into(),
            "material/ui/accessory/ace_icon.mat".into(),
            "material/ui/accessory/ace_icon.tobj".into(),
            format!("{TRUCK_DEF}/ace_a.sii"),
            format!("{TRUCK_DEF}/ace_b.sii"),
            format!("{TRUCK_DEF}/ace_settings.sui"),
            format!("{TRUCK_DEF}/accessory/ace_a.sii"),
            format!("{TRUCK_DEF}/accessory/ace_b.sii"),
        ];
        for t in ["chassis", "high_roof", "side_skirts", "standard"] {
            expected.push(format!("{TRUCK_TEX}/{t}.dds"));
            expected.push(format!("{TRUCK_TEX}/{t}.tobj"));
        }
        expected.sort();
        assert_eq!(paths, expected);
        assert_eq!(
            plan.text(&format!("{TRUCK_DEF}/ace_a.sii")).unwrap(),
            "SiiNunit\n{\naccessory_paint_job_data : ace_a.truckpaint.sample.paint_job\n{\n\
             @include \"ace_settings.sui\"\n\
             \tsuitable_for[]: \"standard.truckpaint.sample.cabin\"\n\
             \tpaint_job_mask: \"/vehicle/truck/upgrade/paintjob/ace/truckpaint.sample/standard.tobj\"\n\
             }\n}\n"
        );
        assert!(
            plan.text(&format!("{TRUCK_DEF}/ace_b.sii"))
                .unwrap()
                .contains("suitable_for[]: \"high_roof.truckpaint.sample.cabin\"")
        );
        let accessories = "SiiNunit\n{\n\
             \nsimple_paint_job_data : .ovr0\n{\n\
             \tpaint_job_mask: \"/vehicle/truck/upgrade/paintjob/ace/truckpaint.sample/chassis.tobj\"\n\
             \tacc_list[]: \"chassis.sample\"\n}\n\
             \nsimple_paint_job_data : .ovr1\n{\n\
             \tpaint_job_mask: \"/vehicle/truck/upgrade/paintjob/ace/truckpaint.sample/side_skirts.tobj\"\n\
             \tacc_list[]: \"sideskirt.sample\"\n}\n}\n";
        assert_eq!(
            plan.text(&format!("{TRUCK_DEF}/accessory/ace_a.sii")),
            Some(accessories)
        );
        assert_eq!(
            plan.text(&format!("{TRUCK_DEF}/accessory/ace_b.sii")),
            Some(accessories)
        );
        assert_eq!(
            plan.text(&format!("{TRUCK_DEF}/ace_settings.sui")).unwrap(),
            "\tname: \"ACE\"\n\tprice: 5000\n\tunlock: 0\n\ticon: \"ace_icon\"\n\tairbrush: true\n"
        );
        assert_eq!(
            plan.files[&format!("{TRUCK_TEX}/side_skirts.dds")],
            Entry::Texture {
                surface: 3,
                size: 1024
            }
        );
        assert_eq!(
            plan.files[&format!("{TRUCK_TEX}/standard.tobj")],
            Entry::Bytes(tobj(&format!("/{TRUCK_TEX}/standard.dds")))
        );
        assert_eq!(
            plan.text("manifest.sii").unwrap(),
            "SiiNunit\n{\nmod_package : .package_name\n{\n\
             \tpackage_version: \"1.0\"\n\tdisplay_name: \"ACE\"\n\tauthor: \"\"\n\
             \tcategory[]: \"paint_job\"\n\tmp_mod_optional: true\n\
             \ticon: \"icon.jpg\"\n\tdescription_file: \"description.txt\"\n}\n}\n"
        );
        assert_eq!(
            plan.text("material/ui/accessory/ace_icon.mat").unwrap(),
            "material: \"ui\"\n{\n\ttexture: \"ace_icon.tobj\"\n\ttexture_name: \"texture\"\n}\n"
        );
    }

    #[test]
    fn unpainted_cabin_gets_no_paint_job() {
        let plan = plan(&truck(&["high_roof", "chassis"])).unwrap();
        assert!(plan.files.contains_key(&format!("{TRUCK_DEF}/ace_b.sii")));
        assert!(!plan.files.contains_key(&format!("{TRUCK_DEF}/ace_a.sii")));
        assert!(
            !plan
                .files
                .contains_key(&format!("{TRUCK_TEX}/standard.dds"))
        );
    }

    #[test]
    fn trailer_paint_job() {
        let plan = plan(&trailer(&["base", "mudflaps"])).unwrap();
        let def = "def/vehicle/trailer_owned/truckpaint.sample_trailer/paint_job";
        let unit = plan.text(&format!("{def}/ace.sii")).unwrap();
        assert!(
            unit.contains("accessory_paint_job_data : ace.truckpaint.sample_trailer.paint_job")
        );
        assert!(!unit.contains("suitable_for"), "{unit}");
        assert!(unit.contains(
            "paint_job_mask: \"/vehicle/trailer_owned/upgrade/paintjob/ace/truckpaint.sample_trailer/base.tobj\""
        ));
        let acc = plan.text(&format!("{def}/accessory/ace.sii")).unwrap();
        assert!(acc.contains("acc_list[]: \"r_mudflap.sample\""), "{acc}");
        assert!(!acc.contains("body"), "{acc}");
    }

    #[test]
    fn fleet_in_one_mod() {
        let mut ws = Workspace::new(truck(&["standard"]));
        let t = example("sample_trailer", "1.0.0");
        ws.add_vehicle(&t, &ids(&["base"]), 1.0).unwrap();
        let plan = plan(&ws.project).unwrap();
        assert_eq!(
            plan.files
                .keys()
                .filter(|k| k.ends_with("manifest.sii"))
                .count(),
            1
        );
        assert!(
            plan.files.contains_key(
                "def/vehicle/trailer_owned/truckpaint.sample_trailer/paint_job/ace.sii"
            )
        );
        assert!(plan.files.contains_key(&format!("{TRUCK_DEF}/ace_a.sii")));
        assert_eq!(
            plan.text("description.txt").unwrap(),
            "Vehicles supported:\n- TruckPaint Sample Truck\n- TruckPaint Sample Trailer\n"
        );
    }

    #[test]
    fn alternate_uv_set_colour_picker_and_required_mods() {
        let mut p = trailer(&["base"]);
        p.mod_settings.description = "Company colors.\n".into();
        let data = p.vehicles[0].game_data.as_mut().unwrap();
        data.alt_uv = true;
        data.colour_picker = true;
        data.requires = vec![
            RequiredMod {
                name: "Trailer Pack".into(),
                version: Some(">=2.1".into()),
            },
            RequiredMod {
                name: "Wheels".into(),
                version: None,
            },
        ];
        let plan = plan(&p).unwrap();
        let settings = plan
            .text("def/vehicle/trailer_owned/truckpaint.sample_trailer/paint_job/ace_settings.sui")
            .unwrap();
        assert!(settings.ends_with("\talternate_uvset: true\n\tbase_color_locked: false\n"));
        assert_eq!(
            plan.text("description.txt").unwrap(),
            "Company colors.\n\nVehicles supported:\n- TruckPaint Sample Trailer\n\n\
             Requires:\n- Trailer Pack >=2.1\n- Wheels\n"
        );
        // No accessories painted: no accessory file.
        assert!(
            !plan
                .files
                .keys()
                .any(|k| k.contains("paint_job/accessory/"))
        );
    }
}
