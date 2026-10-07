//! Format version 3: fills and strokes are paints (solid colors or
//! gradients). Frozen once released: a new format version gets its own
//! module and a migration from this one.

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tp_core::document::{
    AssetId, Cap, CharStyle, ColorStop, DEFAULT_MITER_LIMIT, Dash, Frame, Gradient, GradientKind,
    Join, LineStyle, MIN_STOPS, Node, Object, ObjectId, Paint, PathData, Rgba, ShapeKind,
    StrokeAlign, StrokeStyle, Subpath, TextAlign, TextBlock,
};
use tp_core::kurbo::{Point, Size, Vec2};
use tp_core::{Asset, AssetKind, Axis, Guide, Project, Surface, TextureResolution};

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

/// A color at a location (0..=1) of a gradient.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileStop {
    pub offset: f32,
    pub color: [u8; 4],
}

/// A fill or stroke paint. Gradient points are in frame units: (0, 0) is
/// the frame's local top-left corner, (1, 1) its bottom-right.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FilePaint {
    Solid([u8; 4]),
    Linear {
        start: [f64; 2],
        end: [f64; 2],
        stops: Vec<FileStop>,
    },
    /// `minor`: end of the second radius.
    Radial {
        center: [f64; 2],
        end: [f64; 2],
        minor: [f64; 2],
        stops: Vec<FileStop>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileStroke {
    pub paint: FilePaint,
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
        line_style_to_file(&LineStyle::default())
    }
}

fn line_style_to_file(l: &LineStyle) -> FileLineStyle {
    FileLineStyle {
        dash: l.dash.map(|d| [d.dash, d.gap]),
        cap: match l.cap {
            Cap::Butt => FileCap::Butt,
            Cap::Round => FileCap::Round,
            Cap::Square => FileCap::Square,
        },
        join: match l.join {
            Join::Miter => FileJoin::Miter,
            Join::Round => FileJoin::Round,
            Join::Bevel => FileJoin::Bevel,
        },
        miter_limit: l.miter_limit,
    }
}

/// The line style of `f`; out-of-range values fall back to the defaults.
fn line_style_from_file(f: &FileLineStyle) -> LineStyle {
    let valid = |v: f64| v.is_finite() && v >= 0.0;
    LineStyle {
        dash: f
            .dash
            .filter(|[dash, gap]| valid(*dash) && valid(*gap) && *gap > 0.0)
            .map(|[dash, gap]| Dash { dash, gap }),
        cap: match f.cap {
            FileCap::Butt => Cap::Butt,
            FileCap::Round => Cap::Round,
            FileCap::Square => Cap::Square,
        },
        join: match f.join {
            FileJoin::Miter => Join::Miter,
            FileJoin::Round => Join::Round,
            FileJoin::Bevel => Join::Bevel,
        },
        miter_limit: if f.miter_limit.is_finite() && f.miter_limit >= 1.0 {
            f.miter_limit
        } else {
            DEFAULT_MITER_LIMIT
        },
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
    pub fill: FilePaint,
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

fn color(c: Rgba) -> [u8; 4] {
    [c.r, c.g, c.b, c.a]
}

fn rgba([r, g, b, a]: [u8; 4]) -> Rgba {
    Rgba { r, g, b, a }
}

fn paint_to_file(p: &Paint) -> FilePaint {
    let Paint::Gradient(g) = p else {
        return FilePaint::Solid(color(p.first_color()));
    };
    let xy = |p: Point| [p.x, p.y];
    let stops = g
        .stops()
        .iter()
        .map(|s| FileStop {
            offset: s.offset,
            color: color(s.color),
        })
        .collect();
    match g.kind {
        GradientKind::Linear => FilePaint::Linear {
            start: xy(g.start),
            end: xy(g.end),
            stops,
        },
        GradientKind::Radial => FilePaint::Radial {
            center: xy(g.start),
            end: xy(g.end),
            minor: xy(g.minor),
            stops,
        },
    }
}

/// The paint of `f`; a gradient needs at least two stops and finite points.
fn paint_from_file(f: &FilePaint, id: u64) -> Result<Paint, String> {
    let (kind, start, end, minor, stops) = match f {
        FilePaint::Solid(c) => return Ok(Paint::Solid(rgba(*c))),
        FilePaint::Linear { start, end, stops } => (GradientKind::Linear, start, end, None, stops),
        FilePaint::Radial {
            center,
            end,
            minor,
            stops,
        } => (GradientKind::Radial, center, end, Some(minor), stops),
    };
    let finite = |p: &[f64; 2]| p.iter().all(|v| v.is_finite());
    if stops.len() < MIN_STOPS
        || !finite(start)
        || !finite(end)
        || !minor.is_none_or(finite)
        || stops.iter().any(|s| !s.offset.is_finite())
    {
        return Err(format!("object {id} has an invalid gradient"));
    }
    let stops: Vec<ColorStop> = stops
        .iter()
        .map(|s| ColorStop::new(s.offset, rgba(s.color)))
        .collect();
    let mut g = Gradient::new(kind, &stops);
    let pt = |p: &[f64; 2]| Point::new(p[0], p[1]);
    g.start = pt(start);
    g.end = pt(end);
    if let Some(m) = minor {
        g.minor = pt(m);
    }
    Ok(Paint::Gradient(g))
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
            ShapeKind::Polygon { sides, star } => FileKind::Polygon { sides, star },
            ShapeKind::Path => FileKind::Path,
        },
        center: [o.frame.center.x, o.frame.center.y],
        size: [o.frame.size.width, o.frame.size.height],
        rotation: o.frame.rotation_deg,
        fill: paint_to_file(&o.fill),
        stroke: o.stroke.map(|s| FileStroke {
            paint: paint_to_file(&s.paint),
            width: s.width,
            align: match s.align {
                StrokeAlign::Center => FileStrokeAlign::Center,
                StrokeAlign::Inside => FileStrokeAlign::Inside,
                StrokeAlign::Outside => FileStrokeAlign::Outside,
            },
            line: line_style_to_file(&s.line),
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
        path: o.path_data().map(|p| {
            let xy = |p: tp_core::kurbo::Point| [p.x, p.y];
            p.subpaths
                .iter()
                .map(|s| FileSubpath {
                    closed: s.closed,
                    nodes: s
                        .nodes
                        .iter()
                        .map(|n| FileNode {
                            p: xy(n.point),
                            hin: n.handle_in.map(xy),
                            hout: n.handle_out.map(xy),
                            smooth: n.smooth,
                        })
                        .collect(),
                })
                .collect()
        }),
        line_width: o.path_data().map(|p| p.line_width),
        line_style: o.path_data().map(|p| line_style_to_file(&p.line_style)),
    }
}

/// The document of `project` (asset bytes are written separately).
pub fn from_project(project: &Project) -> FileProject {
    FileProject {
        format: 3,
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
                guides: s
                    .guides
                    .iter()
                    .map(|g| FileGuide {
                        vertical: g.axis == Axis::Vertical,
                        position: g.position,
                    })
                    .collect(),
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
        FileKind::Polygon { sides, star } => ShapeKind::Polygon {
            sides: sides.clamp(3, 12),
            star: star.map(|r| r.clamp(0.1, 0.9)),
        },
        FileKind::Path => ShapeKind::Path,
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
    o.fill = paint_from_file(&f.fill, f.id)?;
    o.stroke = match &f.stroke {
        Some(s) => Some(StrokeStyle {
            paint: paint_from_file(&s.paint, f.id)?,
            width: s.width,
            align: match s.align {
                FileStrokeAlign::Center => StrokeAlign::Center,
                FileStrokeAlign::Inside => StrokeAlign::Inside,
                FileStrokeAlign::Outside => StrokeAlign::Outside,
            },
            line: line_style_from_file(&s.line),
        }),
        None => None,
    };
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
    if kind == ShapeKind::Path {
        let pt = |p: [f64; 2]| Point::new(p[0], p[1]);
        let subpaths = f
            .path
            .as_ref()
            .ok_or_else(|| format!("path {} has no points", f.id))?
            .iter()
            .map(|s| Subpath {
                closed: s.closed,
                nodes: s
                    .nodes
                    .iter()
                    .map(|n| Node {
                        point: pt(n.p),
                        handle_in: n.hin.map(pt),
                        handle_out: n.hout.map(pt),
                        smooth: n.smooth,
                    })
                    .collect(),
            })
            .collect();
        // Stored as written: the frame was fitted to the points on save.
        let mut data = PathData::new(subpaths);
        if let Some(width) = f.line_width.filter(|w| w.is_finite() && *w > 0.0) {
            data.line_width = width;
        }
        if let Some(style) = &f.line_style {
            data.line_style = line_style_from_file(style);
        }
        o.path = Some(Arc::new(data));
    }
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
            surface.guides = s
                .guides
                .iter()
                .filter(|g| g.position.is_finite())
                .map(|g| {
                    let axis = if g.vertical {
                        Axis::Vertical
                    } else {
                        Axis::Horizontal
                    };
                    Guide::new(axis, g.position)
                })
                .collect();
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

/// Converts a version 2 document: colors become solid paints, the rest is
/// unchanged.
pub fn from_v2(old: super::v2::FileProject) -> FileProject {
    use super::v2;
    fn line(l: v2::FileLineStyle) -> FileLineStyle {
        FileLineStyle {
            dash: l.dash,
            cap: match l.cap {
                v2::FileCap::Butt => FileCap::Butt,
                v2::FileCap::Round => FileCap::Round,
                v2::FileCap::Square => FileCap::Square,
            },
            join: match l.join {
                v2::FileJoin::Miter => FileJoin::Miter,
                v2::FileJoin::Round => FileJoin::Round,
                v2::FileJoin::Bevel => FileJoin::Bevel,
            },
            miter_limit: l.miter_limit,
        }
    }
    fn object(o: v2::FileObject) -> FileObject {
        FileObject {
            id: o.id,
            name: o.name,
            kind: match o.kind {
                v2::FileKind::Rectangle { corner_radius } => FileKind::Rectangle { corner_radius },
                v2::FileKind::Ellipse => FileKind::Ellipse,
                v2::FileKind::Group => FileKind::Group,
                v2::FileKind::Text => FileKind::Text,
                v2::FileKind::Image { asset } => FileKind::Image { asset },
                v2::FileKind::Polygon { sides, star } => FileKind::Polygon { sides, star },
                v2::FileKind::Path => FileKind::Path,
            },
            center: o.center,
            size: o.size,
            rotation: o.rotation,
            fill: FilePaint::Solid(o.fill),
            stroke: o.stroke.map(|s| FileStroke {
                paint: FilePaint::Solid(s.color),
                width: s.width,
                align: match s.align {
                    v2::FileStrokeAlign::Center => FileStrokeAlign::Center,
                    v2::FileStrokeAlign::Inside => FileStrokeAlign::Inside,
                    v2::FileStrokeAlign::Outside => FileStrokeAlign::Outside,
                },
                line: line(s.line),
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
                    v2::FileAlign::Left => FileAlign::Left,
                    v2::FileAlign::Center => FileAlign::Center,
                    v2::FileAlign::Right => FileAlign::Right,
                },
                letter_spacing: t.letter_spacing,
                line_height: t.line_height,
                layout_size: t.layout_size,
                scale: t.scale,
            }),
            path: o.path.map(|subpaths| {
                subpaths
                    .into_iter()
                    .map(|s| FileSubpath {
                        closed: s.closed,
                        nodes: s
                            .nodes
                            .into_iter()
                            .map(|n| FileNode {
                                p: n.p,
                                hin: n.hin,
                                hout: n.hout,
                                smooth: n.smooth,
                            })
                            .collect(),
                    })
                    .collect()
            }),
            line_width: o.line_width,
            line_style: o.line_style.map(line),
        }
    }
    FileProject {
        format: 3,
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
                guides: s
                    .guides
                    .into_iter()
                    .map(|g| FileGuide {
                        vertical: g.vertical,
                        position: g.position,
                    })
                    .collect(),
            })
            .collect(),
        assets: old
            .assets
            .into_iter()
            .map(|a| FileAsset {
                id: a.id,
                name: a.name,
                kind: match a.kind {
                    v2::FileAssetKind::Raster => FileAssetKind::Raster,
                    v2::FileAssetKind::Svg => FileAssetKind::Svg,
                },
                entry: a.entry,
                size: a.size,
            })
            .collect(),
    }
}
