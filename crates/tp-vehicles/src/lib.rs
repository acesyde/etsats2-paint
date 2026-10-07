//! Vehicle packages (`.tpv`): a ZIP holding a `vehicle.json` manifest and
//! the template images of one truck or trailer. Packages are data only;
//! this crate reads and validates them from bytes (no filesystem access).

use std::collections::HashSet;
use std::io::{Cursor, Read};

use serde::Deserialize;

pub mod sample;

/// Highest package format this version understands.
pub const FORMAT: u32 = 1;
/// File extension of packages, without the dot.
pub const EXTENSION: &str = "tpv";
/// Name of the manifest inside a package.
pub const MANIFEST: &str = "vehicle.json";
/// Allowed texture sizes (pixels).
pub const SIZES: [u32; 6] = [256, 512, 1024, 2048, 4096, 8192];
/// Largest total uncompressed size of a package.
pub const MAX_UNCOMPRESSED: u64 = 512 * 1024 * 1024;
/// Largest side of a template image.
pub const MAX_IMAGE_SIDE: u32 = 16_384;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Truck,
    Trailer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Game {
    Ets2,
    Ats,
}

impl Game {
    pub fn code(self) -> &'static str {
        match self {
            Game::Ets2 => "ets2",
            Game::Ats => "ats",
        }
    }
}

impl Kind {
    pub fn code(self) -> &'static str {
        match self {
            Kind::Truck => "truck",
            Kind::Trailer => "trailer",
        }
    }
}

/// A mod the vehicle depends on (community vehicles).
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Requirement {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
}

/// What a part's template looks like on the texture.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Texture {
    /// Square side in pixels (one of [`SIZES`]).
    pub size: u32,
    /// Path of the template image inside the package.
    pub template: String,
    pub layout_version: u32,
}

/// A paintable part of the vehicle: one texture, and what it covers in the
/// game (cabin internal names for a main texture, accessory ids for an
/// accessory).
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Part {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub game_ids: Vec<String>,
    pub texture: Texture,
}

/// Whether a part is a main texture or an accessory.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    Main,
    Accessory,
}

/// The vehicle's paint job: main textures (one per cabin layout, or one for
/// the whole vehicle) and accessory textures shared by the whole vehicle.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct PaintJob {
    #[serde(default)]
    pub main: Vec<Part>,
    #[serde(default)]
    pub accessories: Vec<Part>,
}

impl PaintJob {
    /// Every part in package order: main textures, then accessories.
    pub fn parts(&self) -> impl Iterator<Item = (Role, &Part)> {
        self.main
            .iter()
            .map(|p| (Role::Main, p))
            .chain(self.accessories.iter().map(|p| (Role::Accessory, p)))
    }

    /// The part `id` and its role.
    pub fn part(&self, id: &str) -> Option<(Role, &Part)> {
        self.parts().find(|(_, p)| p.id == id)
    }

    /// Position of part `id` in package order.
    pub fn position(&self, id: &str) -> Option<usize> {
        self.parts().position(|(_, p)| p.id == id)
    }
}

/// What the package targets in the game.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct GameTarget {
    pub id: Game,
    /// The game versions the package is made for.
    pub versions: semver::VersionReq,
    /// The vehicle's path in the game's definitions (`scania.r_2016`).
    pub path: String,
    /// The paint job uses the vehicle's alternate UV set.
    #[serde(default)]
    pub alt_uv: bool,
    /// The paint job lets the player pick a base color.
    #[serde(default)]
    pub colour_picker: bool,
    #[serde(default)]
    pub requires: Vec<Requirement>,
}

/// The `vehicle.json` manifest (unknown fields are ignored).
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Manifest {
    pub format: u32,
    pub id: String,
    pub version: semver::Version,
    pub name: String,
    pub brand: String,
    pub kind: Kind,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub preview: Option<String>,
    pub game: GameTarget,
    pub paint_job: PaintJob,
}

/// Why a package is refused. Messages are built by the application.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageError {
    NotAZip,
    NoManifest,
    /// Invalid JSON or a missing / mistyped field (serde message).
    BadManifest(String),
    NewerFormat(u32),
    BadId(String),
    BadGamePath(String),
    NoMainTexture,
    /// Two parts share an id.
    DuplicatePart(String),
    /// An accessory, or one of several main textures, has no game id
    /// (the part's name).
    MissingGameIds(String),
    BadGameId(String),
    /// A game id appears twice among the main textures or among the
    /// accessories.
    DuplicateGameId(String),
    BadSize {
        texture: String,
        size: u32,
    },
    UnsafePath(String),
    TooLarge,
    MissingTemplate {
        texture: String,
        path: String,
    },
    BadTemplate {
        texture: String,
        path: String,
    },
    TemplateTooLarge {
        texture: String,
    },
}

impl std::fmt::Display for PackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for PackageError {}

/// Kind of template image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageKind {
    Png,
    Svg,
}

/// A template image read from a package.
#[derive(Clone, Debug, PartialEq)]
pub struct TemplateImage {
    pub bytes: Vec<u8>,
    pub kind: ImageKind,
    /// Natural size in pixels.
    pub width: f64,
    pub height: f64,
}

/// A validated package: its manifest and every template, in package order.
#[derive(Clone, Debug)]
pub struct Package {
    pub manifest: Manifest,
    /// `(part id, image)`.
    pub templates: Vec<(String, TemplateImage)>,
}

impl Package {
    /// The template of part `id`.
    pub fn template(&self, id: &str) -> Option<&TemplateImage> {
        self.templates.iter().find(|(p, _)| p == id).map(|(_, i)| i)
    }

    /// Reads and fully validates a package.
    pub fn read(bytes: &[u8]) -> Result<Package, PackageError> {
        let mut zip = open(bytes)?;
        check_entries(&mut zip)?;
        let manifest = parse_manifest(&mut zip)?;
        validate(&manifest)?;
        let mut templates = Vec::new();
        for (_, part) in manifest.paint_job.parts() {
            let image = read_template(&mut zip, part)?;
            templates.push((part.id.clone(), image));
        }
        Ok(Package {
            manifest,
            templates,
        })
    }

    /// Reads only the manifest (listing an installed library).
    pub fn read_manifest(bytes: &[u8]) -> Result<Manifest, PackageError> {
        let mut zip = open(bytes)?;
        let manifest = parse_manifest(&mut zip)?;
        validate(&manifest)?;
        Ok(manifest)
    }
}

type Zip<'a> = zip::ZipArchive<Cursor<&'a [u8]>>;

fn open(bytes: &[u8]) -> Result<Zip<'_>, PackageError> {
    zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| PackageError::NotAZip)
}

/// Every entry path is relative without `..`; the total size is bounded.
fn check_entries(zip: &mut Zip<'_>) -> Result<(), PackageError> {
    let mut total: u64 = 0;
    for i in 0..zip.len() {
        let entry = zip.by_index_raw(i).map_err(|_| PackageError::NotAZip)?;
        let name = entry.name().to_owned();
        if !is_safe_path(&name) {
            return Err(PackageError::UnsafePath(name));
        }
        total = total.saturating_add(entry.size());
    }
    if total > MAX_UNCOMPRESSED {
        return Err(PackageError::TooLarge);
    }
    Ok(())
}

/// Relative, `/`-separated, no `..`, no drive or root.
pub fn is_safe_path(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('/')
        && !name.starts_with('\\')
        && !name.contains(':')
        && name.split(['/', '\\']).all(|part| part != "..")
}

fn read_entry(zip: &mut Zip<'_>, name: &str) -> Option<Vec<u8>> {
    let mut entry = zip.by_name(name).ok()?;
    if entry.size() > MAX_UNCOMPRESSED {
        return None;
    }
    let mut out = Vec::with_capacity(entry.size() as usize);
    entry
        .by_ref()
        .take(MAX_UNCOMPRESSED)
        .read_to_end(&mut out)
        .ok()?;
    Some(out)
}

fn parse_manifest(zip: &mut Zip<'_>) -> Result<Manifest, PackageError> {
    let bytes = read_entry(zip, MANIFEST).ok_or(PackageError::NoManifest)?;
    // The format number first, so a newer package is reported as such
    // rather than as an unreadable manifest.
    #[derive(Deserialize)]
    struct Head {
        format: u32,
    }
    if let Ok(head) = serde_json::from_slice::<Head>(&bytes)
        && head.format > FORMAT
    {
        return Err(PackageError::NewerFormat(head.format));
    }
    serde_json::from_slice(&bytes).map_err(|e| PackageError::BadManifest(e.to_string()))
}

/// Lowercase words separated by dots, at least two.
pub fn is_valid_id(id: &str) -> bool {
    let words: Vec<&str> = id.split('.').collect();
    words.len() >= 2
        && words.iter().all(|w| {
            !w.is_empty()
                && w.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
        })
}

/// Words of a–z, 0–9 and `_` separated by dots (the game's unit names;
/// no length limit, real accessory ids exceed the 12 characters of a token).
pub fn is_valid_game_name(name: &str) -> bool {
    name.split('.').all(|w| {
        !w.is_empty()
            && w.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    })
}

fn validate(m: &Manifest) -> Result<(), PackageError> {
    if m.format > FORMAT {
        return Err(PackageError::NewerFormat(m.format));
    }
    if !is_valid_id(&m.id) {
        return Err(PackageError::BadId(m.id.clone()));
    }
    if !is_valid_game_name(&m.game.path) {
        return Err(PackageError::BadGamePath(m.game.path.clone()));
    }
    let job = &m.paint_job;
    if job.main.is_empty() {
        return Err(PackageError::NoMainTexture);
    }
    let mut ids = HashSet::new();
    for (_, part) in job.parts() {
        if !ids.insert(&part.id) {
            return Err(PackageError::DuplicatePart(part.id.clone()));
        }
        validate_part(part)?;
    }
    // Several main textures each say which cabins they are for; an
    // accessory always says which accessories it covers.
    validate_game_ids(&job.main, job.main.len() > 1)?;
    validate_game_ids(&job.accessories, true)
}

fn validate_part(part: &Part) -> Result<(), PackageError> {
    if !SIZES.contains(&part.texture.size) {
        return Err(PackageError::BadSize {
            texture: part.name.clone(),
            size: part.texture.size,
        });
    }
    if !is_safe_path(&part.texture.template) {
        return Err(PackageError::UnsafePath(part.texture.template.clone()));
    }
    Ok(())
}

/// Game ids of a list of parts: well formed, unique in the list, and
/// present on each part when `required`.
fn validate_game_ids(parts: &[Part], required: bool) -> Result<(), PackageError> {
    let mut seen = HashSet::new();
    for part in parts {
        if required && part.game_ids.is_empty() {
            return Err(PackageError::MissingGameIds(part.name.clone()));
        }
        for id in &part.game_ids {
            if !is_valid_game_name(id) {
                return Err(PackageError::BadGameId(id.clone()));
            }
            if !seen.insert(id) {
                return Err(PackageError::DuplicateGameId(id.clone()));
            }
        }
    }
    Ok(())
}

/// Options to check an SVG and read its size: texts are left out without
/// looking for fonts (the default resolver would warn for every `<text>`;
/// the application renders templates with its own fonts).
fn size_only_options() -> resvg::usvg::Options<'static> {
    resvg::usvg::Options {
        font_resolver: resvg::usvg::FontResolver {
            select_font: Box::new(|_, _| None),
            select_fallback: Box::new(|_, _, _| None),
        },
        ..resvg::usvg::Options::default()
    }
}

fn read_template(zip: &mut Zip<'_>, part: &Part) -> Result<TemplateImage, PackageError> {
    let t = &part.texture;
    let bytes = read_entry(zip, &t.template).ok_or_else(|| PackageError::MissingTemplate {
        texture: part.name.clone(),
        path: t.template.clone(),
    })?;
    let bad = || PackageError::BadTemplate {
        texture: part.name.clone(),
        path: t.template.clone(),
    };
    let lower = t.template.to_ascii_lowercase();
    let (kind, width, height) = if lower.ends_with(".png") {
        let reader = image::ImageReader::with_format(Cursor::new(&bytes), image::ImageFormat::Png);
        let (w, h) = reader.into_dimensions().map_err(|_| bad())?;
        if w > MAX_IMAGE_SIDE || h > MAX_IMAGE_SIDE {
            return Err(PackageError::TemplateTooLarge {
                texture: part.name.clone(),
            });
        }
        (ImageKind::Png, f64::from(w), f64::from(h))
    } else if lower.ends_with(".svg") {
        let tree = resvg::usvg::Tree::from_data(&bytes, &size_only_options()).map_err(|_| bad())?;
        let s = tree.size();
        (ImageKind::Svg, f64::from(s.width()), f64::from(s.height()))
    } else {
        return Err(bad());
    };
    Ok(TemplateImage {
        bytes,
        kind,
        width,
        height,
    })
}

#[cfg(test)]
mod tests;
