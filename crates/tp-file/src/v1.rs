//! Format version 1. Frozen: these structures never change. Read only:
//! documents are migrated to version 2 (`v2::from_v1`).

use serde::{Deserialize, Serialize};

/// The document (`project.ron`). Asset bytes live in separate ZIP entries.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileProject {
    pub format: u32,
    pub name: String,
    /// Texture side in pixels (2048, 4096 or 8192).
    pub resolution: u32,
    pub active_surface: usize,
    #[serde(default)]
    pub palette: Vec<[u8; 4]>,
    pub surfaces: Vec<FileSurface>,
    #[serde(default)]
    pub assets: Vec<FileAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileSurface {
    pub name: String,
    pub size: f64,
    #[serde(default)]
    pub objects: Vec<FileObject>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FileKind {
    Rectangle { corner_radius: f64 },
    Ellipse,
    Group,
    Text,
    Image { asset: u64 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileStroke {
    pub color: [u8; 4],
    pub width: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileText {
    pub content: String,
    pub family: String,
    pub weight: u16,
    pub italic: bool,
    pub size: f64,
    pub align: FileAlign,
    pub letter_spacing: f64,
    pub line_height: f64,
    pub layout_size: [f64; 2],
    pub scale: [f64; 2],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileObject {
    pub id: u64,
    pub name: String,
    pub kind: FileKind,
    pub center: [f64; 2],
    pub size: [f64; 2],
    pub rotation: f64,
    pub fill: [u8; 4],
    #[serde(default)]
    pub stroke: Option<FileStroke>,
    pub opacity: f32,
    pub visible: bool,
    pub locked: bool,
    #[serde(default)]
    pub children: Vec<FileObject>,
    #[serde(default)]
    pub text: Option<FileText>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileAssetKind {
    Raster,
    Svg,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileAsset {
    pub id: u64,
    pub name: String,
    pub kind: FileAssetKind,
    /// ZIP entry holding the original bytes.
    pub entry: String,
    pub size: [f64; 2],
}
