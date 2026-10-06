//! Format version 1. Frozen: once released, these structures never change;
//! a new format version gets its own module and a migration from this one.

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tp_core::document::{
    AssetId, CharStyle, Frame, Object, ObjectId, Rgba, ShapeKind, StrokeStyle, TextAlign, TextBlock,
};
use tp_core::kurbo::{Point, Size, Vec2};
use tp_core::{Asset, AssetKind, Project, Surface, TextureResolution};

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

fn color(c: Rgba) -> [u8; 4] {
    [c.r, c.g, c.b, c.a]
}

fn rgba([r, g, b, a]: [u8; 4]) -> Rgba {
    Rgba { r, g, b, a }
}

/// ZIP entry name of an asset: `assets/<id>.<png|jpg|svg>`.
pub fn asset_entry(asset: &Asset) -> String {
    let ext = match asset.kind {
        AssetKind::Svg => "svg",
        AssetKind::Raster if asset.bytes.starts_with(&[0xFF, 0xD8]) => "jpg",
        AssetKind::Raster => "png",
    };
    format!("assets/{}.{ext}", asset.id.0)
}

fn object_to_file(o: &Object) -> FileObject {
    FileObject {
        id: o.id.0,
        name: o.name.clone(),
        kind: match o.kind {
            ShapeKind::Rectangle { corner_radius } => FileKind::Rectangle { corner_radius },
            ShapeKind::Ellipse => FileKind::Ellipse,
            ShapeKind::Group => FileKind::Group,
            ShapeKind::Text => FileKind::Text,
            ShapeKind::Image { asset } => FileKind::Image { asset: asset.0 },
        },
        center: [o.frame.center.x, o.frame.center.y],
        size: [o.frame.size.width, o.frame.size.height],
        rotation: o.frame.rotation_deg,
        fill: color(o.fill),
        stroke: o.stroke.map(|s| FileStroke {
            color: color(s.color),
            width: s.width,
        }),
        opacity: o.opacity,
        visible: o.visible,
        locked: o.locked,
        children: o.children.iter().map(|c| object_to_file(c)).collect(),
        text: o.text.as_ref().map(|t| FileText {
            content: t.content.clone(),
            family: t.style.family.clone(),
            weight: t.style.weight,
            italic: t.style.italic,
            size: t.style.size,
            align: match t.style.align {
                TextAlign::Left => FileAlign::Left,
                TextAlign::Center => FileAlign::Center,
                TextAlign::Right => FileAlign::Right,
            },
            letter_spacing: t.style.letter_spacing,
            line_height: t.style.line_height,
            layout_size: [t.layout_size.width, t.layout_size.height],
            scale: [t.scale.x, t.scale.y],
        }),
    }
}

/// The document of `project` (asset bytes are written separately).
pub fn from_project(project: &Project) -> FileProject {
    FileProject {
        format: 1,
        name: project.name.clone(),
        resolution: project.resolution.side(),
        active_surface: project.active_surface,
        palette: project.palette.iter().map(|c| color(*c)).collect(),
        surfaces: project
            .surfaces
            .iter()
            .map(|s| FileSurface {
                name: s.name.clone(),
                size: s.size,
                objects: s.objects.iter().map(|o| object_to_file(o)).collect(),
            })
            .collect(),
        assets: project
            .assets
            .values()
            .map(|a| FileAsset {
                id: a.id.0,
                name: a.name.clone(),
                kind: match a.kind {
                    AssetKind::Raster => FileAssetKind::Raster,
                    AssetKind::Svg => FileAssetKind::Svg,
                },
                entry: asset_entry(a),
                size: [a.size.width, a.size.height],
            })
            .collect(),
    }
}

fn object_from_file(
    f: &FileObject,
    assets: &BTreeMap<AssetId, Arc<Asset>>,
) -> Result<Object, String> {
    let kind = match f.kind {
        FileKind::Rectangle { corner_radius } => ShapeKind::Rectangle { corner_radius },
        FileKind::Ellipse => ShapeKind::Ellipse,
        FileKind::Group => ShapeKind::Group,
        FileKind::Text => ShapeKind::Text,
        FileKind::Image { asset } => {
            if !assets.contains_key(&AssetId(asset)) {
                return Err(format!("object {} uses a missing asset", f.id));
            }
            ShapeKind::Image {
                asset: AssetId(asset),
            }
        }
    };
    let frame = Frame::new(
        Point::new(f.center[0], f.center[1]),
        Size::new(f.size[0], f.size[1]),
        f.rotation,
    );
    let mut o = Object::new(ObjectId(f.id), kind, frame);
    o.name.clone_from(&f.name);
    o.fill = rgba(f.fill);
    o.stroke = f.stroke.as_ref().map(|s| StrokeStyle {
        color: rgba(s.color),
        width: s.width,
    });
    o.opacity = f.opacity.clamp(0.0, 1.0);
    o.visible = f.visible;
    o.locked = f.locked;
    o.children = f
        .children
        .iter()
        .map(|c| object_from_file(c, assets).map(Arc::new))
        .collect::<Result<_, _>>()?;
    if kind == ShapeKind::Text && f.text.is_none() {
        return Err(format!("text {} has no content", f.id));
    }
    o.text = f.text.as_ref().map(|t| TextBlock {
        content: t.content.clone(),
        style: CharStyle {
            family: t.family.clone(),
            weight: t.weight,
            italic: t.italic,
            size: t.size,
            align: match t.align {
                FileAlign::Left => TextAlign::Left,
                FileAlign::Center => TextAlign::Center,
                FileAlign::Right => TextAlign::Right,
            },
            letter_spacing: t.letter_spacing,
            line_height: t.line_height,
        },
        layout_size: Size::new(t.layout_size[0], t.layout_size[1]),
        scale: Vec2::new(t.scale[0], t.scale[1]),
    });
    if o.is_group() {
        o.refresh_group_frame();
    }
    Ok(o)
}

/// Rebuilds the project; `bytes_of` returns the content of a ZIP entry.
pub fn into_project(
    file: &FileProject,
    mut bytes_of: impl FnMut(&str) -> Option<Vec<u8>>,
) -> Result<Project, String> {
    let resolution = TextureResolution::ALL
        .into_iter()
        .find(|r| r.side() == file.resolution)
        .ok_or_else(|| format!("unknown resolution {}", file.resolution))?;
    let mut assets = BTreeMap::new();
    for a in &file.assets {
        let bytes = bytes_of(&a.entry).ok_or_else(|| format!("missing {}", a.entry))?;
        let kind = match a.kind {
            FileAssetKind::Raster => AssetKind::Raster,
            FileAssetKind::Svg => AssetKind::Svg,
        };
        let asset = Asset::new(
            AssetId(a.id),
            &a.name,
            kind,
            bytes.into(),
            Size::new(a.size[0], a.size[1]),
        );
        assets.insert(asset.id, Arc::new(asset));
    }
    if file.surfaces.is_empty() {
        return Err("the project has no surface".to_owned());
    }
    let surfaces = file
        .surfaces
        .iter()
        .map(|s| {
            let mut surface = Surface::new(s.name.clone(), s.size);
            surface.objects = s
                .objects
                .iter()
                .map(|o| object_from_file(o, &assets).map(Arc::new))
                .collect::<Result<_, _>>()?;
            Ok(surface)
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(Project::from_parts(
        &file.name,
        resolution,
        surfaces,
        file.active_surface,
        file.palette.iter().map(|c| rgba(*c)).collect(),
        assets,
    ))
}
