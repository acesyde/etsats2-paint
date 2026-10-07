//! The `.truckpaint` project file: a ZIP archive holding
//! - `mimetype`: `application/x-truckpaint` (identifies the file),
//! - `project.ron`: the document, starting with its `format` version,
//! - `assets/<id>.<ext>`: imported files, byte for byte.
//!
//! Every format version keeps a frozen module (`v1`, …). Reading parses a
//! file with the module of its own version and migrates it step by step to
//! the current one; writing always uses the current version.

pub mod v1;
pub mod v2;
pub mod v3;

use std::io::{Cursor, Read, Write};
use std::path::Path;

use serde::Deserialize;
use tp_core::Project;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// Current file format version.
pub const FORMAT_VERSION: u32 = 3;
/// File extension, without the dot.
pub const EXTENSION: &str = "truckpaint";
const MIMETYPE: &str = "application/x-truckpaint";
const DOCUMENT: &str = "project.ron";

/// The current format's module.
pub use v3 as current;

/// Why a file could not be read or written.
#[derive(Debug)]
pub enum Error {
    /// Not a TruckPaint project at all.
    NotAProject,
    /// A project, but incomplete or inconsistent.
    Damaged(String),
    /// Written by a newer TruckPaint.
    NewerVersion {
        found: u32,
        supported: u32,
    },
    Io(std::io::Error),
}

impl Error {
    /// Message for the user, about the file named `file`.
    pub fn message(&self, file: &str) -> String {
        match self {
            Self::NotAProject => format!("{file} is not a TruckPaint project."),
            Self::Damaged(_) => format!("{file} is damaged and cannot be opened."),
            Self::NewerVersion { .. } => {
                format!("{file} was created with a newer version of TruckPaint.")
            }
            Self::Io(err) => format!("{file} could not be read or written ({err})."),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAProject => write!(f, "not a TruckPaint project"),
            Self::Damaged(why) => write!(f, "damaged project: {why}"),
            Self::NewerVersion { found, supported } => {
                write!(f, "format {found} is newer than {supported}")
            }
            Self::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

/// A project read from a file.
#[derive(Debug)]
pub struct Opened {
    pub project: Project,
    /// The file used an older format and was converted.
    pub migrated: bool,
}

/// Only the version, read before anything else so it is known even when
/// the rest of the document does not match the current structures.
#[derive(Deserialize)]
struct Header {
    format: u32,
}

fn ron_options() -> ron::Options {
    ron::Options::default()
}

/// Serializes a project to `.truckpaint` bytes.
pub fn to_bytes(project: &Project) -> Result<Vec<u8>, Error> {
    let zip_err = |e: zip::result::ZipError| Error::Io(std::io::Error::other(e));
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    zip.start_file("mimetype", stored).map_err(zip_err)?;
    zip.write_all(MIMETYPE.as_bytes())?;

    let document = current::from_project(project);
    let text = ron_options()
        .to_string_pretty(&document, ron::ser::PrettyConfig::default())
        .map_err(|e| Error::Io(std::io::Error::other(e)))?;
    zip.start_file(DOCUMENT, deflated).map_err(zip_err)?;
    zip.write_all(text.as_bytes())?;

    for asset in project.assets.values() {
        let options = match asset.kind {
            tp_core::AssetKind::Svg => deflated,
            // Already compressed.
            tp_core::AssetKind::Raster => stored,
        };
        zip.start_file(current::asset_entry(asset), options)
            .map_err(zip_err)?;
        zip.write_all(&asset.bytes)?;
    }
    Ok(zip.finish().map_err(zip_err)?.into_inner())
}

/// Reads a project from `.truckpaint` bytes.
pub fn from_bytes(bytes: &[u8]) -> Result<Opened, Error> {
    let mut zip = ZipArchive::new(Cursor::new(bytes)).map_err(|e| {
        // A ZIP that cannot be read (e.g. truncated) is a damaged project;
        // anything else is not a project at all.
        if bytes.starts_with(b"PK\x03\x04") {
            Error::Damaged(e.to_string())
        } else {
            Error::NotAProject
        }
    })?;
    let mut mimetype = String::new();
    zip.by_name("mimetype")
        .map_err(|_| Error::NotAProject)?
        .read_to_string(&mut mimetype)
        .map_err(|_| Error::NotAProject)?;
    if mimetype.trim() != MIMETYPE {
        return Err(Error::NotAProject);
    }
    let mut text = String::new();
    zip.by_name(DOCUMENT)
        .map_err(|_| Error::Damaged("missing document".into()))?
        .read_to_string(&mut text)
        .map_err(|e| Error::Damaged(e.to_string()))?;
    let header: Header = ron_options()
        .from_str(&text)
        .map_err(|e| Error::Damaged(e.to_string()))?;
    let (document, migrated) = migrate(header.format, &text)?;
    let project = current::into_project(&document, |entry| {
        let mut out = Vec::new();
        zip.by_name(entry).ok()?.read_to_end(&mut out).ok()?;
        Some(out)
    })
    .map_err(Error::Damaged)?;
    Ok(Opened { project, migrated })
}

/// Parses the document with its own version's structures and converts it
/// to the current version. Returns whether a conversion happened.
fn migrate(format: u32, text: &str) -> Result<(current::FileProject, bool), Error> {
    let parse_err = |e: ron::error::SpannedError| Error::Damaged(e.to_string());
    match format {
        // Each older version is parsed with its own module, then converted
        // step by step (`vN → vN+1`).
        1 => {
            let v1: v1::FileProject = ron_options().from_str(text).map_err(parse_err)?;
            Ok((v3::from_v2(v2::from_v1(v1)), true))
        }
        2 => {
            let v2: v2::FileProject = ron_options().from_str(text).map_err(parse_err)?;
            Ok((v3::from_v2(v2), true))
        }
        3 => Ok((ron_options().from_str(text).map_err(parse_err)?, false)),
        0 => Err(Error::Damaged("invalid format version 0".into())),
        found => Err(Error::NewerVersion {
            found,
            supported: FORMAT_VERSION,
        }),
    }
}

/// Reads a project file.
pub fn read(path: &Path) -> Result<Opened, Error> {
    from_bytes(&std::fs::read(path)?)
}

/// Writes a project file atomically: the content goes to a temporary file
/// next to `path`, flushed to disk, then renamed over `path`, so an error
/// never leaves a partial file behind.
pub fn write(project: &Project, path: &Path) -> Result<(), Error> {
    let bytes = to_bytes(project)?;
    write_atomic(path, &bytes)
}

/// Replaces `path` with `bytes` atomically.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "project".into());
    let temp = path.with_file_name(format!(".{name}.tmp-{}", std::process::id()));
    let result = (|| {
        let mut file = std::fs::File::create(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    Ok(result?)
}
