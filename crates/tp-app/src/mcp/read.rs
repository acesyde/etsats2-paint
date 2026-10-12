//! The read and see tools: the project, a texture's objects, one object,
//! and PNG renders of a template or of a texture's artwork.

use std::sync::Arc;

use serde_json::{Value, json};
use tp_core::document::{GradientKind, Object, ObjectId, Paint, ShapeKind, TextAlign};
use tp_core::{Project, Surface, TexturePart};
use tp_mcp::{Outcome, Reply};

use crate::workspace::Workspace;

/// The id of surface `index`: `"<package>/<texture>"`, or its index for a
/// surface without a vehicle texture.
pub(super) fn texture_id(project: &Project, index: usize) -> String {
    match &project.surfaces[index].template {
        Some(t) => format!("{}/{}", t.package_id, t.texture_id),
        None => index.to_string(),
    }
}

/// The surface whose id is `id`.
pub(super) fn texture_index(project: &Project, id: &str) -> Result<usize, String> {
    let id = id.trim();
    (0..project.surfaces.len())
        .find(|i| texture_id(project, *i) == id)
        .ok_or_else(|| format!("unknown texture `{id}`; get_project lists the texture ids"))
}

fn round(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn kind_name(o: &Object) -> &'static str {
    match o.kind {
        ShapeKind::Rectangle { .. } => "rectangle",
        ShapeKind::Ellipse => "ellipse",
        ShapeKind::Polygon { star: None, .. } => "polygon",
        ShapeKind::Polygon { star: Some(_), .. } => "star",
        ShapeKind::Path => "path",
        ShapeKind::Group => "group",
        ShapeKind::Text => "text",
        ShapeKind::Image { .. } => "image",
        ShapeKind::Instance { .. } => "symbol_instance",
    }
}

/// The object's unrotated box and rotation.
fn box_json(o: &Object) -> Value {
    let f = o.frame;
    json!({
        "x": round(f.center.x - f.size.width / 2.0),
        "y": round(f.center.y - f.size.height / 2.0),
        "width": round(f.size.width),
        "height": round(f.size.height),
        "rotation": round(f.rotation_deg),
    })
}

fn swatch_name(project: &Project, id: Option<tp_core::document::SwatchId>) -> Option<String> {
    id.and_then(|id| project.swatch(id)).map(|s| s.name.clone())
}

fn paint_json(
    project: &Project,
    paint: &Paint,
    swatch: Option<tp_core::document::SwatchId>,
    o: &Object,
) -> Value {
    match paint {
        Paint::Solid(c) if c.a == 0 => json!("none"),
        Paint::Solid(c) => match swatch_name(project, swatch) {
            Some(name) => json!({"swatch": name, "color": c.to_hex()}),
            None => json!(c.to_hex()),
        },
        Paint::Gradient(g) => json!({
            "gradient": match g.kind { GradientKind::Linear => "linear", GradientKind::Radial => "radial" },
            "angle": round(g.angle(&o.frame)),
            "stops": g.stops().iter().map(|s| {
                let color = match swatch_name(project, s.swatch) {
                    Some(name) => json!({"swatch": name, "color": s.color.to_hex()}),
                    None => json!(s.color.to_hex()),
                };
                json!({"offset": s.offset, "color": color})
            }).collect::<Vec<_>>(),
        }),
    }
}

/// The open project: game, vehicles and their textures, palette, styles,
/// symbols.
pub(super) fn project(ws: &Workspace) -> Reply {
    let p = &ws.project;
    let texture = |i: usize| {
        let s = &p.surfaces[i];
        let part = s.template.as_ref().map(|t| match t.part {
            TexturePart::Main => "main",
            TexturePart::Accessory => "accessory",
        });
        json!({
            "id": texture_id(p, i),
            "name": s.name,
            "part": part,
            "width": s.size.width,
            "height": s.size.height,
            "objects": s.objects.len(),
            "has_template": s.template.as_ref().is_some_and(|t| t.status != tp_core::TemplateStatus::Removed),
        })
    };
    let vehicles: Vec<Value> = p
        .vehicles
        .iter()
        .map(|v| {
            json!({
                "id": v.package_id,
                "name": v.name,
                "brand": v.brand,
                "kind": v.kind,
                "textures": p.vehicle_range(&v.package_id).map(texture).collect::<Vec<_>>(),
            })
        })
        .collect();
    let mut result = json!({
        "name": p.name,
        "game": p.game(),
        "vehicles": vehicles,
        "palette": p.palette.iter().map(|s| json!({"id": s.id.0, "name": s.name, "color": s.color.to_hex()})).collect::<Vec<_>>(),
        "graphic_styles": p.graphic_styles.iter().map(|s| json!({"id": s.id.0, "name": s.name})).collect::<Vec<_>>(),
        "text_styles": p.text_styles.iter().map(|s| json!({"id": s.id.0, "name": s.name})).collect::<Vec<_>>(),
        "symbols": p.symbols.iter().map(|s| json!({"id": s.id.0, "name": s.name, "width": s.surface.size.width, "height": s.surface.size.height})).collect::<Vec<_>>(),
        "texture_shown": texture_id(p, p.active_surface),
        "editing_symbol": p.edited_symbol().map(|s| s.name.clone()),
    });
    let loose: Vec<Value> = (0..p.surfaces.len())
        .filter(|i| p.surfaces[*i].template.is_none())
        .map(texture)
        .collect();
    if !loose.is_empty() {
        result["other_textures"] = Value::Array(loose);
    }
    Ok(Outcome::Json(result))
}

fn tree_json(list: &[Arc<Object>]) -> Vec<Value> {
    list.iter()
        .rev()
        .map(|o| {
            let mut v = json!({
                "id": o.id.0,
                "kind": kind_name(o),
                "name": o.name,
                "visible": o.visible,
                "locked": o.locked,
            });
            if let (Value::Object(v), Value::Object(b)) = (&mut v, box_json(o)) {
                v.extend(b);
            }
            if o.is_group() {
                v["children"] = Value::Array(tree_json(&o.children));
            }
            v
        })
        .collect()
}

/// A texture's objects, top to bottom.
pub(super) fn list_objects(ws: &Workspace, texture: &str) -> Reply {
    let index = texture_index(&ws.project, texture)?;
    let s = &ws.project.surfaces[index];
    Ok(Outcome::Json(json!({
        "texture": texture_id(&ws.project, index),
        "width": s.size.width,
        "height": s.size.height,
        "objects": tree_json(&s.objects),
    })))
}

/// One object's properties.
pub(super) fn object(ws: &Workspace, texture: &str, id: ObjectId) -> Reply {
    let p = &ws.project;
    let index = texture_index(p, texture)?;
    let o = p.surfaces[index]
        .get(id)
        .ok_or_else(|| format!("no object {} on texture `{texture}`", id.0))?;
    let mut v = json!({
        "id": o.id.0,
        "kind": kind_name(o),
        "name": o.name,
        "visible": o.visible,
        "locked": o.locked,
        "opacity": o.opacity,
    });
    if let (Value::Object(v), Value::Object(b)) = (&mut v, box_json(o)) {
        v.extend(b);
    }
    if !o.is_group() && !o.is_instance() && !matches!(o.kind, ShapeKind::Image { .. }) {
        v["fill"] = paint_json(p, &o.fill, o.fill_swatch, o);
        v["stroke"] = o.stroke.map_or(Value::Null, |s| {
            json!({
                "color": paint_json(p, &s.paint, s.swatch, o),
                "width": s.width,
                "align": format!("{:?}", s.align).to_lowercase(),
            })
        });
    }
    if let Some(s) = o.shadow {
        v["shadow"] = json!({
            "color": match swatch_name(p, s.swatch) {
                Some(name) => json!({"swatch": name, "color": s.color.to_hex()}),
                None => json!(s.color.to_hex()),
            },
            "opacity": s.opacity,
            "offset_x": s.offset.x,
            "offset_y": s.offset.y,
            "blur": s.blur,
        });
    }
    if let Some(style) = o.style {
        let name = p
            .graphic_style(style)
            .map(|s| s.name.clone())
            .or_else(|| p.text_style(style).map(|s| s.name.clone()));
        v["style"] = json!(name);
    }
    match o.kind {
        ShapeKind::Rectangle { corner_radius } => v["corner_radius"] = json!(corner_radius),
        ShapeKind::Polygon { sides, star } => {
            v["sides"] = json!(sides);
            if let Some(ratio) = star {
                v["inner_ratio"] = json!(ratio);
            }
        }
        ShapeKind::Path => v["d"] = json!(o.path().to_svg()),
        ShapeKind::Image { asset } => {
            v["image"] = json!(p.assets.get(&asset).map(|a| a.name.clone()));
        }
        ShapeKind::Instance { symbol, .. } => {
            v["symbol"] = json!(p.symbol(symbol).map(|s| s.name.clone()));
        }
        ShapeKind::Group => v["children"] = Value::Array(tree_json(&o.children)),
        _ => {}
    }
    if let Some(t) = &o.text {
        v["text"] = json!(t.content);
        v["font"] = json!(t.style.family);
        v["size"] = json!(t.style.size);
        v["weight"] = json!(t.style.weight);
        v["italic"] = json!(t.style.italic);
        v["align"] = json!(match t.style.align {
            TextAlign::Left => "left",
            TextAlign::Center => "center",
            TextAlign::Right => "right",
        });
    }
    Ok(Outcome::Json(v))
}

/// Pixel size of a render of `surface` with its longest side at
/// `max_size`, never larger than the texture.
fn render_size(surface: &Surface, max_size: u32) -> (u32, u32) {
    let side = (max_size as f64).min(surface.max_side()).max(1.0) as u32;
    crate::surface_thumbnails::thumbnail_size(surface.size, side)
}

fn scale_text(what: &str, surface: &Surface, (w, h): (u32, u32)) -> String {
    // Not localized: read by the model.
    let k = round(surface.size.width / f64::from(w));
    format!(
        "{what}, {w} × {h} px for a {} × {} texture: 1 image px = {k} texture px (multiply image coordinates by {k} to get texture pixels).",
        surface.size.width, surface.size.height
    )
}

fn encode_png(image: &image::RgbaImage) -> Result<Vec<u8>, String> {
    let mut out = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut out, image::ImageFormat::Png)
        .map_err(|e| format!("cannot encode the image: {e}"))?;
    Ok(out.into_inner())
}

/// The template's pixels at `size` (stretched over the texture, as on the
/// canvas).
fn template_image(asset: &tp_core::Asset, (w, h): (u32, u32)) -> Option<image::RgbaImage> {
    let decoded = crate::image_cache::image_of(asset.kind, &asset.bytes, w.max(h).max(1024))?;
    let [iw, ih] = decoded.size;
    let rgba: Vec<u8> = decoded
        .pixels
        .iter()
        .flat_map(|c| c.to_srgba_unmultiplied())
        .collect();
    let img = image::RgbaImage::from_raw(iw as u32, ih as u32, rgba)?;
    Some(image::imageops::resize(
        &img,
        w,
        h,
        image::imageops::FilterType::Triangle,
    ))
}

/// The template of a texture as a PNG.
pub(super) fn template(ws: &Workspace, texture: &str, max_size: u32) -> Reply {
    let index = texture_index(&ws.project, texture)?;
    let surface = &ws.project.surfaces[index];
    let asset = surface
        .template
        .as_ref()
        .filter(|t| t.status != tp_core::TemplateStatus::Removed)
        .and_then(|t| ws.project.assets.get(&t.asset).cloned())
        .ok_or_else(|| format!("texture `{texture}` has no template"))?;
    let size = render_size(surface, max_size);
    let text = scale_text("Template", surface, size);
    Ok(Outcome::Deferred(Box::new(move || {
        let image = template_image(&asset, size).ok_or("the template image cannot be read")?;
        Ok(Outcome::Image {
            png: encode_png(&image)?,
            text,
        })
    })))
}

/// The artwork of a texture as a PNG, on the artboard color, with the
/// template over it as the canvas shows it.
pub(super) fn render(ws: &Workspace, texture: &str, max_size: u32, with_template: bool) -> Reply {
    let index = texture_index(&ws.project, texture)?;
    let surface = &ws.project.surfaces[index];
    let size = render_size(surface, max_size);
    let template = surface
        .template
        .as_ref()
        .filter(|t| with_template && t.status != tp_core::TemplateStatus::Removed)
        .and_then(|t| Some((ws.project.assets.get(&t.asset).cloned()?, t.opacity)));
    let what = if template.is_some() {
        "Artwork with the template over it"
    } else {
        "Artwork"
    };
    let text = scale_text(what, surface, size);
    let project = ws.project.clone();
    let mut fonts = ws.text.fonts.fork();
    Ok(Outcome::Deferred(Box::new(move || {
        let artboard = tp_ui::tokens::canvas::ARTBOARD;
        let options = tp_render::RenderOptions {
            width: size.0,
            height: size.1,
            background: Some(tp_core::document::Rgba::rgb(
                artboard.r(),
                artboard.g(),
                artboard.b(),
            )),
        };
        let pixmap = tp_render::render(&project, index, options, &mut fonts, &mut |_, _| true)
            .map_err(|_| "the render was cancelled")?;
        let mut image = tp_render::to_rgba(&pixmap);
        if let Some((asset, opacity)) = template
            && let Some(over) = template_image(&asset, size)
        {
            blend(&mut image, &over, opacity);
        }
        Ok(Outcome::Image {
            png: encode_png(&image)?,
            text,
        })
    })))
}

/// Draws `over` on `under` at `opacity`.
fn blend(under: &mut image::RgbaImage, over: &image::RgbaImage, opacity: f32) {
    for (dst, src) in under.pixels_mut().zip(over.pixels()) {
        let a = f32::from(src[3]) / 255.0 * opacity.clamp(0.0, 1.0);
        for c in 0..3 {
            let v = f32::from(dst[c]) * (1.0 - a) + f32::from(src[c]) * a;
            dst[c] = v.round().clamp(0.0, 255.0) as u8;
        }
    }
}
