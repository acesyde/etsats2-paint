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

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Texture {
    pub id: String,
    pub name: String,
    pub size: u32,
    /// Path of the template image inside the package.
    pub template: String,
    pub layout_version: u32,
    /// Data for the mod export, kept as-is.
    #[serde(default)]
    pub export: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Variant {
    pub id: String,
    pub name: String,
    pub textures: Vec<Texture>,
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
    pub game: Game,
    pub game_versions: semver::VersionReq,
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
    #[serde(default)]
    pub requires: Vec<Requirement>,
    pub variants: Vec<Variant>,
}

impl Manifest {
    pub fn variant(&self, id: &str) -> Option<&Variant> {
        self.variants.iter().find(|v| v.id == id)
    }
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
    NoVariant,
    EmptyVariant(String),
    DuplicateTexture {
        variant: String,
        texture: String,
    },
    DuplicateVariant(String),
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

/// A validated package: its manifest and every template, in variant and
/// texture order.
#[derive(Clone, Debug)]
pub struct Package {
    pub manifest: Manifest,
    /// `(variant id, texture id, image)`.
    pub templates: Vec<(String, String, TemplateImage)>,
}

impl Package {
    /// The template of a texture of a variant.
    pub fn template(&self, variant: &str, texture: &str) -> Option<&TemplateImage> {
        self.templates
            .iter()
            .find(|(v, t, _)| v == variant && t == texture)
            .map(|(_, _, i)| i)
    }

    /// Reads and fully validates a package.
    pub fn read(bytes: &[u8]) -> Result<Package, PackageError> {
        let mut zip = open(bytes)?;
        check_entries(&mut zip)?;
        let manifest = parse_manifest(&mut zip)?;
        validate(&manifest)?;
        let mut templates = Vec::new();
        for variant in &manifest.variants {
            for texture in &variant.textures {
                let image = read_template(&mut zip, texture)?;
                templates.push((variant.id.clone(), texture.id.clone(), image));
            }
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

fn validate(m: &Manifest) -> Result<(), PackageError> {
    if m.format > FORMAT {
        return Err(PackageError::NewerFormat(m.format));
    }
    if !is_valid_id(&m.id) {
        return Err(PackageError::BadId(m.id.clone()));
    }
    if m.variants.is_empty() {
        return Err(PackageError::NoVariant);
    }
    let mut variant_ids = HashSet::new();
    for v in &m.variants {
        if !variant_ids.insert(&v.id) {
            return Err(PackageError::DuplicateVariant(v.id.clone()));
        }
        if v.textures.is_empty() {
            return Err(PackageError::EmptyVariant(v.name.clone()));
        }
        let mut ids = HashSet::new();
        for t in &v.textures {
            if !ids.insert(&t.id) {
                return Err(PackageError::DuplicateTexture {
                    variant: v.name.clone(),
                    texture: t.id.clone(),
                });
            }
            if !SIZES.contains(&t.size) {
                return Err(PackageError::BadSize {
                    texture: t.name.clone(),
                    size: t.size,
                });
            }
            if !is_safe_path(&t.template) {
                return Err(PackageError::UnsafePath(t.template.clone()));
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

fn read_template(zip: &mut Zip<'_>, t: &Texture) -> Result<TemplateImage, PackageError> {
    let bytes = read_entry(zip, &t.template).ok_or_else(|| PackageError::MissingTemplate {
        texture: t.name.clone(),
        path: t.template.clone(),
    })?;
    let bad = || PackageError::BadTemplate {
        texture: t.name.clone(),
        path: t.template.clone(),
    };
    let lower = t.template.to_ascii_lowercase();
    let (kind, width, height) = if lower.ends_with(".png") {
        let reader = image::ImageReader::with_format(Cursor::new(&bytes), image::ImageFormat::Png);
        let (w, h) = reader.into_dimensions().map_err(|_| bad())?;
        if w > MAX_IMAGE_SIDE || h > MAX_IMAGE_SIDE {
            return Err(PackageError::TemplateTooLarge {
                texture: t.name.clone(),
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
