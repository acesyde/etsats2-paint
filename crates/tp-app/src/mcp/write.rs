//! The write tools. Each call runs on a copy of the project and replaces
//! it only when every step succeeded, as one undo step labeled "AI: …".

use std::sync::Arc;

use serde_json::{Value, json};
use tp_core::document::{
    BooleanError, BooleanOp, ColorStop, Frame, Gradient, Object, ObjectId, OperandProblem, Paint,
    PathData, Rgba, Shadow, ShapeKind, StrokeStyle, StyleId, SwatchId, SymbolId, TextBlock, align,
    apply_affine, combine, operand_problem, orient_even_odd, tree,
};
use tp_core::kurbo::{Affine, Point, Rect, Size, Vec2};
use tp_core::{DEFAULT_SWATCH_PREFIX, Project};
use tp_mcp::call::{
    AlignTarget, ArrangeAction, BoxChange, BoxSpec, ColorRef, LookSpec, OrderMove, PaintSpec,
    ShapeSpec, WriteOp,
};
use tp_mcp::{Outcome, Reply};

use super::read::{texture_id, texture_index};
use crate::text_engine::TextEngine;
use crate::workspace::Workspace;

/// What write calls fail with while a symbol is edited.
pub const EDITING_SYMBOL: &str = "The user is editing a symbol in TruckPaint; textures can't be \
changed until they finish editing it (read and render tools still work).";

/// A change applied to the copy of the project.
struct Done {
    /// Undo label (a message id).
    label: &'static str,
    /// The objects created or changed.
    ids: Vec<ObjectId>,
    /// More to tell the client.
    extra: Value,
    /// Texts' character settings may have changed (lay them out again).
    relayout: bool,
}

impl Done {
    fn new(label: &'static str, ids: Vec<ObjectId>) -> Self {
        Self {
            label,
            ids,
            extra: Value::Null,
            relayout: false,
        }
    }
}

/// Runs a change to texture `texture`.
pub(super) fn run(ws: &mut Workspace, texture: &str, op: &WriteOp, now: f64) -> Reply {
    if ws.project.editing_symbol.is_some() {
        return Err(EDITING_SYMBOL.to_owned());
    }
    let index = texture_index(&ws.project, texture)?;
    let mut project = ws.project.clone();
    let shown = project.active_surface;
    // The project's operations act on the active surface.
    project.active_surface = index;
    let done = apply(&mut project, &mut ws.text, op);
    project.active_surface = shown;
    let done = done?;
    let before = ws.snapshot();
    ws.project = project;
    if done.relayout {
        ws.relayout_all_texts();
    }
    if index == shown {
        let surface = &ws.project.surfaces[index];
        let changed: Vec<ObjectId> = done
            .ids
            .iter()
            .copied()
            .filter(|id| surface.get(*id).is_some())
            .collect();
        ws.selection = if changed.is_empty() {
            ws.selection
                .iter()
                .copied()
                .filter(|id| surface.get(*id).is_some())
                .collect()
        } else {
            changed
        };
        ws.points.clear();
    }
    ws.record(done.label, before, now, false);
    let mut result = json!({
        "texture": texture_id(&ws.project, index),
        "ids": done.ids.iter().map(|id| id.0).collect::<Vec<_>>(),
    });
    if let (Value::Object(r), Value::Object(extra)) = (&mut result, done.extra) {
        r.extend(extra);
    }
    Ok(Outcome::Json(result))
}

/// Adds a named swatch to the palette (or tells the one that already has
/// the color).
pub(super) fn add_swatch(ws: &mut Workspace, name: &str, color: Rgba, now: f64) -> Reply {
    let p = &ws.project;
    if let Some(s) = p.palette.iter().find(|s| s.name.eq_ignore_ascii_case(name)) {
        return Err(format!(
            "a swatch named \"{}\" already exists ({}, id {})",
            s.name,
            s.color.to_hex(),
            s.id.0
        ));
    }
    if let Some(s) = p.palette.iter().find(|s| s.color == color) {
        return Ok(Outcome::Json(json!({
            "id": s.id.0,
            "name": s.name,
            "color": s.color.to_hex(),
            "note": "the palette already has this color; use this swatch",
        })));
    }
    let mut added = None;
    ws.edit("undo-ai-add-swatch", now, false, |project, _| {
        let (id, _) = project.add_swatch(color, DEFAULT_SWATCH_PREFIX);
        project.rename_swatch(id, name);
        added = Some(id);
    });
    let id = added.expect("added");
    Ok(Outcome::Json(
        json!({"id": id.0, "name": name, "color": color.to_hex()}),
    ))
}

fn apply(project: &mut Project, text: &mut TextEngine, op: &WriteOp) -> Result<Done, String> {
    match op {
        WriteOp::AddShape {
            shape,
            frame,
            look,
            parent,
        } => {
            let kind = match *shape {
                ShapeSpec::Rectangle { corner_radius } => ShapeKind::Rectangle { corner_radius },
                ShapeSpec::Ellipse => ShapeKind::Ellipse,
                ShapeSpec::Polygon { sides } => ShapeKind::Polygon { sides, star: None },
                ShapeSpec::Star {
                    points,
                    inner_ratio,
                } => ShapeKind::Polygon {
                    sides: points,
                    star: Some(inner_ratio),
                },
            };
            let mut o = Object::new(ObjectId(0), kind, frame_of(frame));
            apply_look(project, &mut o, look)?;
            let id = add(project, *parent, o)?;
            Ok(Done::new("undo-ai-add-shape", vec![id]))
        }
        WriteOp::AddPath {
            path,
            even_odd,
            look,
            parent,
        } => {
            let path = if *even_odd {
                orient_even_odd(path)
            } else {
                path.clone()
            };
            let data = PathData::from_bezpath(&path);
            if data.is_empty() {
                return Err("`d` draws nothing".to_owned());
            }
            let mut o = Object::from_path(ObjectId(0), data);
            apply_look(project, &mut o, look)?;
            let id = add(project, *parent, o)?;
            Ok(Done::new("undo-ai-add-path", vec![id]))
        }
        WriteOp::AddText {
            content,
            family,
            weight,
            italic,
            size,
            align,
            x,
            y,
            rotation,
            look,
            parent,
        } => {
            let mut style = tp_core::document::CharStyle::default();
            if let Some(family) = family {
                if !text.fonts.has_family(family) {
                    let some: Vec<&str> = text
                        .fonts
                        .families()
                        .iter()
                        .take(30)
                        .map(String::as_str)
                        .collect();
                    return Err(format!(
                        "the font \"{family}\" is not available; some that are: {}",
                        some.join(", ")
                    ));
                }
                let (found, _) = text.fonts.resolve_family(family);
                style.family = found.to_owned();
            }
            style.weight = weight.map_or(style.weight, |w| {
                text.fonts.nearest_weight(&style.family, w)
            });
            style.italic = *italic;
            style.size = *size;
            style.align = *align;
            let mut o = Object::text(
                ObjectId(0),
                TextBlock::new(content.clone(), style),
                Point::ORIGIN,
            );
            text.place_at(&mut o, Point::ORIGIN);
            let s = o.frame.size;
            o.frame.center = Point::new(x + s.width / 2.0, y + s.height / 2.0);
            o.frame.rotation_deg = *rotation;
            o.frame = o.frame.sanitized();
            o.name = content
                .lines()
                .next()
                .unwrap_or_default()
                .chars()
                .take(40)
                .collect();
            apply_look(project, &mut o, look)?;
            let id = add(project, *parent, o)?;
            Ok(Done {
                extra: json!({"width": s.width.round(), "height": s.height.round()}),
                ..Done::new("undo-ai-add-text", vec![id])
            })
        }
        WriteOp::ImportImage {
            name,
            bytes,
            frame,
            parent,
        } => {
            let file = crate::import::read_bytes(name, bytes.clone()).map_err(|err| {
                // Read by the model: in English.
                let language = tp_i18n::current();
                tp_i18n::set_language(tp_i18n::Language::English);
                let text = err.to_string();
                tp_i18n::set_language(language);
                text
            })?;
            let natural = file.size;
            let (asset, _) = project.add_asset(&file.name, file.kind, file.bytes, file.size);
            let surface = project.surface();
            let frame = frame.map_or_else(
                || {
                    let size = crate::placement::placement_size(natural, surface.size);
                    Frame::new(surface.bounds().center(), size, 0.0)
                },
                |b| frame_of(&b),
            );
            let mut o = Object::new(ObjectId(0), ShapeKind::Image { asset }, frame);
            o.name = file.name;
            let id = add(project, *parent, o)?;
            Ok(Done {
                extra: json!({"natural_width": natural.width, "natural_height": natural.height}),
                ..Done::new("undo-ai-import-image", vec![id])
            })
        }
        WriteOp::UpdateObject {
            id,
            frame,
            look,
            visible,
            locked,
        } => {
            let mut o = (**existing(project, &[*id])?.first().expect("one")).clone();
            if !frame.is_empty() {
                o = reframe(&o, frame);
            }
            apply_look(project, &mut o, look)?;
            if let Some(v) = visible {
                o.visible = *v;
            }
            if let Some(l) = locked {
                o.locked = *l;
            }
            project.surface_mut().replace(&[o]);
            Ok(Done::new("undo-ai-update-object", vec![*id]))
        }
        WriteOp::Arrange { ids, action } => {
            existing(project, ids)?;
            arrange(project, ids, *action)
        }
        WriteOp::Delete { ids } => {
            existing(project, ids)?;
            project.surface_mut().remove(ids);
            Ok(Done {
                extra: json!({"deleted": ids.iter().map(|i| i.0).collect::<Vec<_>>()}),
                ..Done::new("undo-ai-delete", Vec::new())
            })
        }
        WriteOp::ApplyStyle { style, ids } => {
            existing(project, ids)?;
            let (id, text_style) = find_style(project, style)?;
            if text_style {
                project.apply_text_style(id, ids);
            } else {
                project.apply_graphic_style(id, ids);
            }
            Ok(Done {
                relayout: text_style,
                ..Done::new("undo-ai-apply-style", ids.clone())
            })
        }
        WriteOp::PlaceSymbol { symbol, frame } => {
            let id = find_symbol(project, symbol)?;
            let size = project.symbol(id).expect("found").surface.size;
            let placement = Affine::translate(Vec2::new(
                frame.x + frame.width / 2.0,
                frame.y + frame.height / 2.0,
            )) * Affine::rotate(frame.rotation.to_radians())
                * Affine::scale_non_uniform(frame.width / size.width, frame.height / size.height)
                * Affine::translate(-size.to_vec2() / 2.0);
            let o = project
                .new_instance(id, placement)
                .ok_or("the symbol is gone")?;
            let id = project.add_to(None, o);
            // Its content is expanded from the symbol.
            project.refresh_instances();
            Ok(Done::new("undo-ai-place-symbol", vec![id]))
        }
    }
}

/// A frame from a box and a rotation.
fn frame_of(b: &BoxSpec) -> Frame {
    Frame::new(
        Point::new(b.x + b.width / 2.0, b.y + b.height / 2.0),
        Size::new(b.width, b.height),
        b.rotation,
    )
}

/// Adds `o` at the top of `parent` (a group of the surface) or of the
/// surface.
fn add(project: &mut Project, parent: Option<ObjectId>, o: Object) -> Result<ObjectId, String> {
    if let Some(p) = parent
        && !project.surface().get(p).is_some_and(|g| g.is_group())
    {
        return Err(format!("`parent` {} is not a group on this texture", p.0));
    }
    Ok(project.add_to(parent, o))
}

/// The objects `ids` of the surface, or an error naming the missing ones.
fn existing(project: &Project, ids: &[ObjectId]) -> Result<Vec<Arc<Object>>, String> {
    let surface = project.surface();
    let missing: Vec<String> = ids
        .iter()
        .filter(|id| surface.get(**id).is_none())
        .map(|id| id.0.to_string())
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "no object {} on this texture; nothing was changed (list_objects lists them)",
            missing.join(", ")
        ));
    }
    Ok(ids
        .iter()
        .filter_map(|id| surface.get(*id).cloned())
        .collect())
}

/// `o` moved and resized to the changed box: unrotated, fitted to the new
/// box, then turned to the new rotation (each step exact).
fn reframe(o: &Object, change: &BoxChange) -> Object {
    let f = o.frame;
    let old = Rect::from_center_size(f.center, f.size);
    let new = Rect::from_origin_size(
        (change.x.unwrap_or(old.x0), change.y.unwrap_or(old.y0)),
        (
            change.width.unwrap_or(old.width()),
            change.height.unwrap_or(old.height()),
        ),
    );
    let rotation = change.rotation.unwrap_or(f.rotation_deg);
    let about = |c: Point, deg: f64| {
        Affine::translate(c.to_vec2())
            * Affine::rotate(deg.to_radians())
            * Affine::translate(-c.to_vec2())
    };
    let one = |o: &Object, a: Affine| apply_affine(std::slice::from_ref(o), a).remove(0);
    let unrotated = one(o, about(f.center, -f.rotation_deg));
    let sx = new.width() / old.width().max(1e-9);
    let sy = new.height() / old.height().max(1e-9);
    let fitted = one(
        &unrotated,
        Affine::translate((new.x0, new.y0))
            * Affine::scale_non_uniform(sx, sy)
            * Affine::translate((-old.x0, -old.y0)),
    );
    one(&fitted, about(new.center(), rotation))
}

/// The color and the swatch it is linked to.
fn color(project: &Project, c: &ColorRef) -> Result<(Rgba, Option<SwatchId>), String> {
    match c {
        ColorRef::Hex(c) => Ok((*c, None)),
        ColorRef::Swatch(name) => {
            let found = project
                .palette
                .iter()
                .find(|s| s.name.eq_ignore_ascii_case(name))
                .or_else(|| {
                    let id = name.parse::<u64>().ok()?;
                    project.swatch(SwatchId(id))
                })
                .ok_or_else(|| {
                    let names: Vec<&str> =
                        project.palette.iter().map(|s| s.name.as_str()).collect();
                    format!(
                        "no swatch \"{name}\" in the palette (it has: {})",
                        if names.is_empty() {
                            "none".to_owned()
                        } else {
                            names.join(", ")
                        }
                    )
                })?;
            Ok((found.color, Some(found.id)))
        }
    }
}

/// Gives `o` the look `look` sets.
fn apply_look(project: &Project, o: &mut Object, look: &LookSpec) -> Result<(), String> {
    if let Some(name) = look.name.as_ref().filter(|n| !n.is_empty()) {
        o.name.clone_from(name);
    }
    let paints = !o.is_group() && !o.is_instance() && !matches!(o.kind, ShapeKind::Image { .. });
    if !paints && (look.fill.is_some() || look.stroke.is_some()) {
        return Err("groups, images and symbol instances have no fill or stroke".to_owned());
    }
    match &look.fill {
        None => {}
        Some(PaintSpec::None) => {
            o.fill = Paint::Solid(Rgba::with_alpha(0, 0, 0, 0));
            o.fill_swatch = None;
        }
        Some(PaintSpec::Color(c)) => {
            let (c, swatch) = color(project, c)?;
            o.fill = Paint::Solid(c);
            o.fill_swatch = swatch;
        }
        Some(PaintSpec::Gradient { kind, angle, stops }) => {
            let stops = stops
                .iter()
                .map(|(offset, c)| {
                    let (color, swatch) = color(project, c)?;
                    Ok(ColorStop {
                        offset: *offset,
                        color,
                        swatch,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            let mut g = Gradient::new(*kind, &stops);
            g.set_angle(&o.frame, *angle);
            o.fill = Paint::Gradient(g);
            o.fill_swatch = None;
        }
    }
    match &look.stroke {
        None => {}
        Some(None) => o.stroke = None,
        Some(Some(s)) => {
            let (c, swatch) = color(project, &s.color)?;
            let line = o.stroke.map(|old| old.line).unwrap_or_default();
            o.stroke = Some(StrokeStyle {
                paint: Paint::Solid(c),
                width: s.width,
                align: s.align,
                line,
                swatch,
            });
        }
    }
    if let Some(opacity) = look.opacity {
        o.opacity = opacity;
    }
    match &look.shadow {
        None => {}
        Some(None) => o.shadow = None,
        Some(Some(s)) => {
            if !o.takes_shadow() {
                return Err("groups and symbol instances have no shadow".to_owned());
            }
            let (c, swatch) = color(project, &s.color)?;
            o.shadow = Some(
                Shadow {
                    color: c,
                    swatch,
                    opacity: s.opacity,
                    offset: Vec2::new(s.offset_x, s.offset_y),
                    blur: s.blur,
                }
                .clamped(),
            );
        }
    }
    Ok(())
}

fn find_style(project: &Project, name: &str) -> Result<(StyleId, bool), String> {
    let id = name.parse::<u64>().ok().map(StyleId);
    let matches = |n: &str, i: StyleId| n.eq_ignore_ascii_case(name) || Some(i) == id;
    if let Some(s) = project
        .graphic_styles
        .iter()
        .find(|s| matches(&s.name, s.id))
    {
        return Ok((s.id, false));
    }
    if let Some(s) = project.text_styles.iter().find(|s| matches(&s.name, s.id)) {
        return Ok((s.id, true));
    }
    Err(format!(
        "no style \"{name}\" in the project (get_project lists them)"
    ))
}

fn find_symbol(project: &Project, name: &str) -> Result<SymbolId, String> {
    let id = name.parse::<u64>().ok().map(SymbolId);
    project
        .symbols
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case(name) || Some(s.id) == id)
        .map(|s| s.id)
        .ok_or_else(|| format!("no symbol \"{name}\" in the project (get_project lists them)"))
}

fn arrange(project: &mut Project, ids: &[ObjectId], action: ArrangeAction) -> Result<Done, String> {
    match action {
        ArrangeAction::Order(order) => {
            let surface = project.surface_mut();
            let steps = match order {
                OrderMove::Forward | OrderMove::Backward => 1,
                // Enough steps to pass every sibling.
                OrderMove::Front | OrderMove::Back => tree::paint_order(&surface.objects).len(),
            };
            for _ in 0..steps {
                match order {
                    OrderMove::Forward | OrderMove::Front => surface.bring_forward(ids),
                    OrderMove::Backward | OrderMove::Back => surface.send_backward(ids),
                }
            }
            Ok(Done::new("undo-ai-arrange", ids.to_vec()))
        }
        ArrangeAction::Group => {
            let id = project.group(ids).ok_or("nothing to group")?;
            Ok(Done::new("undo-ai-arrange", vec![id]))
        }
        ArrangeAction::Ungroup => {
            let released = project.ungroup(ids);
            if released.is_empty() {
                return Err("none of these objects is a group".to_owned());
            }
            Ok(Done::new("undo-ai-arrange", released))
        }
        ArrangeAction::Align { edge, to } => {
            let objects = project.selected_objects(ids);
            let target = match (to, objects.as_slice()) {
                (AlignTarget::Texture, _) | (AlignTarget::Selection, [_]) => {
                    project.surface().bounds()
                }
                (AlignTarget::Selection, many) => many
                    .iter()
                    .map(Object::bounding_box)
                    .reduce(|a, b| a.union(b))
                    .unwrap_or_default(),
            };
            let moved = align(&objects, edge, target);
            project.surface_mut().replace(&moved);
            Ok(Done::new("undo-ai-arrange", ids.to_vec()))
        }
        ArrangeAction::Combine(op) => combine_objects(project, ids, op),
    }
}

/// Combines closed shapes into one path, which takes the topmost one's
/// place, id and look (the bottom one's for subtract), as Object › Combine.
fn combine_objects(project: &mut Project, ids: &[ObjectId], op: BooleanOp) -> Result<Done, String> {
    let objects = project.selected_objects(ids);
    if objects.len() < 2 {
        return Err("combine needs two closed shapes or more".to_owned());
    }
    if let Some(problem) = objects.iter().find_map(operand_problem) {
        return Err(match problem {
            OperandProblem::OpenLine => "open lines have no area to combine",
            OperandProblem::Text => "texts can't be combined (they would need outlining first)",
            OperandProblem::Image => "images can't be combined",
            OperandProblem::Instance => "symbol instances can't be combined",
        }
        .to_owned());
    }
    let data = combine(&objects, op).map_err(|err| match err {
        BooleanError::Empty => {
            "the result would be empty (the shapes don't overlap, or nothing remains)"
        }
        BooleanError::Failed => "these shapes could not be combined reliably",
    })?;
    let top = objects.last().expect("two objects").clone();
    let style = if op == BooleanOp::MinusFront {
        &objects[0]
    } else {
        &top
    };
    let mut result = Object::from_path(top.id, data);
    result.name.clone_from(&style.name);
    result.fill = style.fill;
    result.fill_swatch = style.fill_swatch;
    result.stroke = style.stroke;
    result.remap_paints(&style.frame, Affine::IDENTITY);
    result.opacity = style.opacity;
    result.visible = style.visible;
    result.locked = style.locked;
    let others: Vec<ObjectId> = objects[..objects.len() - 1].iter().map(|o| o.id).collect();
    let surface = project.surface_mut();
    surface.replace(&[result]);
    surface.remove(&others);
    Ok(Done::new("undo-ai-arrange", vec![top.id]))
}
