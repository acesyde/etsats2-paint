//! Format version 2: adds polygons, paths and stroke options. Frozen: these
//! structures never change. Read only: documents are migrated to version 3
//! (`v3::from_v2`).

use serde::{Deserialize, Serialize};
use tp_core::document::DEFAULT_MITER_LIMIT;

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
    /// Added before format 2 was released: optional, empty when absent.
    #[serde(default)]
    pub guides: Vec<FileGuide>,
}

/// A guide: vertical at x = `position`, or horizontal at y = `position`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileGuide {
    pub vertical: bool,
    pub position: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FileKind {
    Rectangle {
        corner_radius: f64,
    },
    Ellipse,
    Group,
    Text,
    Image {
        asset: u64,
    },
    /// `star`: inner radius ratio of a star.
    Polygon {
        sides: u8,
        star: Option<f64>,
    },
    Path,
}

/// An anchor point (frame-local coordinates).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileNode {
    pub p: [f64; 2],
    #[serde(default)]
    pub hin: Option<[f64; 2]>,
    #[serde(default)]
    pub hout: Option<[f64; 2]>,
    #[serde(default)]
    pub smooth: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileSubpath {
    pub closed: bool,
    pub nodes: Vec<FileNode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileStroke {
    pub color: [u8; 4],
    pub width: f64,
    #[serde(default)]
    pub align: FileStrokeAlign,
    /// Dash pattern, caps and joins of the outline.
    #[serde(default)]
    pub line: FileLineStyle,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileStrokeAlign {
    #[default]
    Center,
    Inside,
    Outside,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileCap {
    Butt,
    #[default]
    Round,
    Square,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

fn default_miter_limit() -> f64 {
    DEFAULT_MITER_LIMIT
}

/// Dash pattern (`[dash, gap]`), caps, joins and miter limit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileLineStyle {
    #[serde(default)]
    pub dash: Option<[f64; 2]>,
    #[serde(default)]
    pub cap: FileCap,
    #[serde(default)]
    pub join: FileJoin,
    #[serde(default = "default_miter_limit")]
    pub miter_limit: f64,
}

impl Default for FileLineStyle {
    fn default() -> Self {
        Self {
            dash: None,
            cap: FileCap::Round,
            join: FileJoin::Miter,
            miter_limit: DEFAULT_MITER_LIMIT,
        }
    }
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
    #[serde(default)]
    pub path: Option<Vec<FileSubpath>>,
    /// Width of the lines of a path's open subpaths.
    #[serde(default)]
    pub line_width: Option<f64>,
    /// Dash pattern, caps and joins of those lines.
    #[serde(default)]
    pub line_style: Option<FileLineStyle>,
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

/// Converts a version 1 document (field for field: version 2 only adds).
pub fn from_v1(old: super::v1::FileProject) -> FileProject {
    use super::v1;
    fn object(o: v1::FileObject) -> FileObject {
        FileObject {
            id: o.id,
            name: o.name,
            kind: match o.kind {
                v1::FileKind::Rectangle { corner_radius } => FileKind::Rectangle { corner_radius },
                v1::FileKind::Ellipse => FileKind::Ellipse,
                v1::FileKind::Group => FileKind::Group,
                v1::FileKind::Text => FileKind::Text,
                v1::FileKind::Image { asset } => FileKind::Image { asset },
            },
            center: o.center,
            size: o.size,
            rotation: o.rotation,
            fill: o.fill,
            stroke: o.stroke.map(|s| FileStroke {
                color: s.color,
                width: s.width,
                align: FileStrokeAlign::Center,
                line: FileLineStyle::default(),
            }),
            opacity: o.opacity,
            visible: o.visible,
            locked: o.locked,
            children: o.children.into_iter().map(object).collect(),
            text: o.text.map(|t| FileText {
                content: t.content,
                family: t.family,
                weight: t.weight,
                italic: t.italic,
                size: t.size,
                align: match t.align {
                    v1::FileAlign::Left => FileAlign::Left,
                    v1::FileAlign::Center => FileAlign::Center,
                    v1::FileAlign::Right => FileAlign::Right,
                },
                letter_spacing: t.letter_spacing,
                line_height: t.line_height,
                layout_size: t.layout_size,
                scale: t.scale,
            }),
            path: None,
            line_width: None,
            line_style: None,
        }
    }
    FileProject {
        format: 2,
        name: old.name,
        resolution: old.resolution,
        active_surface: old.active_surface,
        palette: old.palette,
        surfaces: old
            .surfaces
            .into_iter()
            .map(|s| FileSurface {
                name: s.name,
                size: s.size,
                objects: s.objects.into_iter().map(object).collect(),
                guides: Vec::new(),
            })
            .collect(),
        assets: old
            .assets
            .into_iter()
            .map(|a| FileAsset {
                id: a.id,
                name: a.name,
                kind: match a.kind {
                    v1::FileAssetKind::Raster => FileAssetKind::Raster,
                    v1::FileAssetKind::Svg => FileAssetKind::Svg,
                },
                entry: a.entry,
                size: a.size,
            })
            .collect(),
    }
}
