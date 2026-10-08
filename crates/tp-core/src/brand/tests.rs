use kurbo::{Point, Size, Vec2};

use super::*;
use crate::document::{
    CharStyle, ColorStop, Frame, Gradient, GradientKind, Object, ObjectId, Paint, Rgba, ShapeKind,
    StrokeStyle, TextBlock,
};
use crate::project::{Project, Surface, TextureResolution};

const RED: Rgba = Rgba::rgb(200, 0, 0);
const DARK_RED: Rgba = Rgba::rgb(0x8B, 0, 0);
const BLUE: Rgba = Rgba::rgb(0, 0, 255);

fn project() -> Project {
    let mut p = Project::new("p", TextureResolution::R2048);
    p.surfaces.push(Surface::new("Second", 2048.0));
    p
}

fn rect(x: f64) -> Object {
    Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(x, 100.0), Size::new(100.0, 50.0), 0.0),
    )
}

fn text(content: &str) -> Object {
    let mut o = rect(0.0);
    o.kind = ShapeKind::Text;
    o.text = Some(TextBlock::new(content, CharStyle::default()));
    o
}

fn get(p: &Project, surface: usize, id: ObjectId) -> Object {
    (**tree::get(&p.surfaces[surface].objects, id).expect("object")).clone()
}

/// Adds `o` to surface `surface` and returns its id.
fn add_on(p: &mut Project, surface: usize, o: Object) -> ObjectId {
    let active = p.active_surface;
    p.active_surface = surface;
    let id = p.add(o);
    p.active_surface = active;
    id
}

/// A rectangle filled with `color`, linked to swatch `id`.
fn linked_rect(color: Rgba, id: SwatchId) -> Object {
    let mut o = rect(0.0);
    o.fill = Paint::Solid(color);
    o.fill_swatch = Some(id);
    o
}

fn gradient(stops: &[ColorStop]) -> Paint {
    Paint::Gradient(Gradient::new(GradientKind::Linear, stops))
}

// ── Swatches ────────────────────────────────────────────────────────────

#[test]
fn swatches_are_named_and_unique() {
    let mut p = project();
    let (a, new_a) = p.add_swatch(RED, "Color");
    let (b, _) = p.add_swatch(BLUE, "Color");
    let (again, new_again) = p.add_swatch(RED, "Color");
    assert!(new_a && !new_again);
    assert_eq!(again, a, "no duplicate color");
    assert_eq!(p.swatch(a).unwrap().name, "Color 1");
    assert_eq!(p.swatch(b).unwrap().name, "Color 2");
    p.delete_swatch(b);
    let (c, _) = p.add_swatch(DARK_RED, "Color");
    assert_eq!(p.swatch(c).unwrap().name, "Color 2", "first unused number");
    assert!(!p.rename_swatch(a, "  "));
    assert!(p.rename_swatch(a, "Company red"));
    assert_eq!(p.swatch(a).unwrap().name, "Company red");
}

#[test]
fn undo_restores_a_deleted_swatch() {
    let mut p = project();
    let (a, _) = p.add_swatch(RED, "Color");
    let r = add_on(&mut p, 0, linked_rect(RED, a));
    let snap = p.snapshot(&[]);
    p.delete_swatch(a);
    assert!(p.palette.is_empty());
    assert_eq!(get(&p, 0, r).fill_swatch, None, "the link is dropped");
    assert_eq!(get(&p, 0, r).fill, Paint::Solid(RED), "the look is kept");
    p.restore(&snap);
    assert_eq!(p.swatch(a).unwrap().color, RED);
    assert_eq!(get(&p, 0, r).fill_swatch, Some(a));
}

#[test]
fn a_swatch_recolors_the_fleet() {
    let mut p = project();
    let (red, _) = p.add_swatch(RED, "Color");
    let a = add_on(&mut p, 0, linked_rect(RED, red));
    let b = add_on(&mut p, 1, linked_rect(RED, red));
    // A linked stroke, a gradient with one linked stop, inside a group.
    let mut stroked = rect(10.0);
    stroked.stroke = Some(StrokeStyle {
        paint: Paint::Solid(RED),
        swatch: Some(red),
        ..StrokeStyle::default()
    });
    let mut linked_stop = ColorStop::new(0.0, RED);
    linked_stop.swatch = Some(red);
    stroked.fill = gradient(&[linked_stop, ColorStop::new(1.0, RED)]);
    let s = add_on(&mut p, 1, stroked);
    p.active_surface = 1;
    let group = p.group(&[s]).unwrap();
    p.active_surface = 0;
    // The same red, not linked.
    let mut plain = rect(20.0);
    plain.fill = Paint::Solid(RED);
    let plain = add_on(&mut p, 0, plain);
    let plain_arc = tree::get(&p.surfaces[0].objects, plain).unwrap().clone();

    p.set_swatch_color(red, DARK_RED);
    assert_eq!(get(&p, 0, a).fill, Paint::Solid(DARK_RED));
    assert_eq!(get(&p, 1, b).fill, Paint::Solid(DARK_RED));
    let inner = get(&p, 1, s);
    assert_eq!(inner.stroke.unwrap().paint, Paint::Solid(DARK_RED));
    let stops = inner.fill.gradient().unwrap().stops().to_vec();
    assert_eq!(stops[0].color, DARK_RED, "the linked stop");
    assert_eq!(stops[1].color, RED, "the other stop");
    assert!(tree::get(&p.surfaces[1].objects, group).is_some());
    assert_eq!(get(&p, 0, plain).fill, Paint::Solid(RED));
    assert!(
        Arc::ptr_eq(
            &plain_arc,
            tree::get(&p.surfaces[0].objects, plain).unwrap()
        ),
        "unchanged objects keep their Arc"
    );
}

#[test]
fn relink_drops_links_that_no_longer_match() {
    let mut p = project();
    let (red, _) = p.add_swatch(RED, "Color");
    let recolored = add_on(&mut p, 0, linked_rect(RED, red));
    let to_gradient = add_on(&mut p, 0, linked_rect(RED, red));
    let mut pasted = linked_rect(RED, red);
    pasted.fill_swatch = Some(SwatchId(999));
    let pasted = add_on(&mut p, 0, pasted);
    let kept = add_on(&mut p, 0, linked_rect(RED, red));

    let edit = |p: &mut Project, id: ObjectId, f: &dyn Fn(&mut Object)| {
        let mut o = get(p, 0, id);
        f(&mut o);
        p.surface_mut().replace(&[o]);
    };
    edit(&mut p, recolored, &|o| o.fill = Paint::Solid(BLUE));
    edit(&mut p, to_gradient, &|o| {
        o.fill = Paint::Gradient(Gradient::from_color(GradientKind::Linear, RED));
    });
    p.relink();
    assert_eq!(get(&p, 0, recolored).fill_swatch, None);
    assert_eq!(get(&p, 0, to_gradient).fill_swatch, None);
    assert_eq!(get(&p, 0, pasted).fill_swatch, None, "unknown swatch");
    assert_eq!(get(&p, 0, kept).fill_swatch, Some(red));
}

#[test]
fn links_survive_transforms_and_structure_changes() {
    let mut p = project();
    let (red, _) = p.add_swatch(RED, "Color");
    let a = p.add(linked_rect(RED, red));
    let moved = crate::document::translate(&[get(&p, 0, a)], Vec2::new(30.0, 0.0), false);
    p.surface_mut().replace(&moved);
    let turned = crate::document::rotate(&[get(&p, 0, a)], Point::ORIGIN, 30.0, false);
    p.surface_mut().replace(&turned);
    let copies = p.duplicate(&[a], Vec2::ZERO);
    let group = p.group(&[a]).unwrap();
    let mut path = get(&p, 0, a);
    assert!(path.convert_to_path());
    p.surface_mut().replace(&[path]);
    p.relink();
    assert_eq!(get(&p, 0, a).fill_swatch, Some(red));
    assert_eq!(get(&p, 0, copies[0]).fill_swatch, Some(red));
    assert!(tree::get(&p.surfaces[0].objects, group).is_some());
}

#[test]
fn set_stops_keeps_links() {
    let mut stop = ColorStop::new(0.3, RED);
    stop.swatch = Some(SwatchId(7));
    let mut g = Gradient::new(GradientKind::Linear, &[ColorStop::new(0.0, BLUE), stop]);
    assert_eq!(g.stops()[1].swatch, Some(SwatchId(7)));
    g.set_stops(&[stop]);
    assert!(g.stops().iter().all(|s| s.swatch == Some(SwatchId(7))));
}

// ── Graphic styles ──────────────────────────────────────────────────────

fn styled(fill: Paint, opacity: f32) -> Object {
    let mut o = rect(0.0);
    o.fill = fill;
    o.opacity = opacity;
    o.stroke = Some(StrokeStyle {
        paint: Paint::Solid(Rgba::rgb(0, 0, 0)),
        width: 4.0,
        ..StrokeStyle::default()
    });
    o
}

#[test]
fn matching_ignores_gradient_position_only() {
    let g = gradient(&[ColorStop::new(0.0, RED), ColorStop::new(1.0, BLUE)]);
    let source = styled(g, 0.8);
    let style = GraphicStyle::from_object(StyleId(1), "S".into(), &source);
    assert!(style.matches(&source));
    let mut moved = source.clone();
    moved.fill.gradient_mut().unwrap().start = Point::new(0.2, 0.9);
    assert!(
        style.matches(&moved),
        "gradient points are the object's own"
    );
    let mut faded = source.clone();
    faded.opacity = 0.5;
    assert!(!style.matches(&faded));
    let mut recolored = source.clone();
    recolored.fill = gradient(&[ColorStop::new(0.0, BLUE), ColorStop::new(1.0, BLUE)]);
    assert!(!style.matches(&recolored));
}

#[test]
fn new_apply_and_group() {
    let mut p = project();
    let source = p.add(styled(Paint::Solid(RED), 0.8));
    let id = p.new_graphic_style(source, "Style").unwrap();
    assert_eq!(p.graphic_style(id).unwrap().name, "Style 1");
    assert_eq!(get(&p, 0, source).style, Some(id), "the source follows it");
    let a = p.add(rect(200.0));
    let b = p.add(rect(400.0));
    let group = p.group(&[a, b]).unwrap();
    p.apply_graphic_style(id, &[group]);
    for o in [a, b] {
        let o = get(&p, 0, o);
        assert_eq!(o.style, Some(id));
        assert_eq!(o.fill, Paint::Solid(RED));
        assert_eq!(o.opacity, 0.8);
        assert_eq!(o.stroke.unwrap().width, 4.0);
    }
    assert_eq!(get(&p, 0, group).style, None, "a group follows no style");
    assert_eq!(p.styles_of(&[group]), vec![id]);
    assert_eq!(p.style_users(id, 0).len(), 3);
}

#[test]
fn applying_keeps_the_objects_gradient_position() {
    let mut p = project();
    let source = p.add(styled(
        gradient(&[ColorStop::new(0.0, RED), ColorStop::new(1.0, BLUE)]),
        1.0,
    ));
    let id = p.new_graphic_style(source, "Style").unwrap();
    let mut own = Gradient::from_color(GradientKind::Linear, BLUE);
    own.start = Point::new(0.5, 0.0);
    own.end = Point::new(0.5, 1.0);
    let mut target = rect(300.0);
    target.fill = Paint::Gradient(own);
    let target = p.add(target);
    p.apply_graphic_style(id, &[target]);
    let g = *get(&p, 0, target).fill.gradient().unwrap();
    assert_eq!((g.start, g.end), (own.start, own.end));
    assert_eq!(g.stops()[0].color, RED);
}

#[test]
fn redefine_changes_users_on_every_surface() {
    let mut p = project();
    let source = p.add(styled(Paint::Solid(RED), 1.0));
    let id = p.new_graphic_style(source, "Style").unwrap();
    let other = add_on(&mut p, 1, rect(0.0));
    p.active_surface = 1;
    p.apply_graphic_style(id, &[other]);
    p.active_surface = 0;
    let mut changed = get(&p, 0, source);
    changed.fill = Paint::Solid(BLUE);
    p.surface_mut().replace(&[changed]);
    assert!(p.redefine_graphic_style(id, source));
    assert_eq!(p.graphic_style(id).unwrap().look.fill, Paint::Solid(BLUE));
    assert_eq!(get(&p, 1, other).fill, Paint::Solid(BLUE));
    p.relink();
    assert_eq!(get(&p, 1, other).style, Some(id));
    assert_eq!(get(&p, 0, source).style, Some(id));
}

#[test]
fn deleting_a_style_keeps_looks() {
    let mut p = project();
    let source = p.add(styled(Paint::Solid(RED), 0.5));
    let id = p.new_graphic_style(source, "Style").unwrap();
    p.delete_style(id);
    let o = get(&p, 0, source);
    assert_eq!(o.style, None);
    assert_eq!((o.fill, o.opacity), (Paint::Solid(RED), 0.5));
}

#[test]
fn a_style_follows_its_swatch() {
    let mut p = project();
    let (red, _) = p.add_swatch(RED, "Color");
    let source = p.add(linked_rect(RED, red));
    let id = p.new_graphic_style(source, "Style").unwrap();
    let user = add_on(&mut p, 1, rect(0.0));
    p.active_surface = 1;
    p.apply_graphic_style(id, &[user]);
    p.set_swatch_color(red, DARK_RED);
    p.relink();
    assert_eq!(
        p.graphic_style(id).unwrap().look.fill,
        Paint::Solid(DARK_RED)
    );
    let u = get(&p, 1, user);
    assert_eq!((u.fill, u.style), (Paint::Solid(DARK_RED), Some(id)));
}

#[test]
fn own_changes_detach_transforms_dont() {
    let mut p = project();
    let source = p.add(styled(
        gradient(&[ColorStop::new(0.0, RED), ColorStop::new(1.0, BLUE)]),
        1.0,
    ));
    let id = p.new_graphic_style(source, "Style").unwrap();
    let turned = crate::document::rotate(&[get(&p, 0, source)], Point::ORIGIN, 30.0, false);
    p.surface_mut().replace(&turned);
    p.relink();
    assert_eq!(
        get(&p, 0, source).style,
        Some(id),
        "rotation keeps the link"
    );
    let mut faded = get(&p, 0, source);
    faded.opacity = 0.4;
    p.surface_mut().replace(&[faded]);
    p.relink();
    assert_eq!(get(&p, 0, source).style, None, "opacity detaches");
}

// ── Text styles ─────────────────────────────────────────────────────────

#[test]
fn text_styles() {
    let mut p = project();
    let t = p.add(text("ACE"));
    let id = p.new_text_style(t, "Text style").unwrap();
    assert_eq!(p.text_style(id).unwrap().name, "Text style 1");
    let shape = p.add(rect(0.0));
    assert_eq!(p.new_text_style(shape, "Text style"), None);
    let other = add_on(&mut p, 1, text("LOGISTICS"));
    p.active_surface = 1;
    p.apply_text_style(id, &[other]);
    p.active_surface = 0;
    // Redefine from a changed text: both follow.
    let mut bigger = get(&p, 0, t);
    bigger.text.as_mut().unwrap().style.size = 300.0;
    p.surface_mut().replace(&[bigger]);
    assert!(p.redefine_text_style(id, t));
    p.relink();
    let o = get(&p, 1, other);
    assert_eq!(o.text.as_ref().unwrap().style.size, 300.0);
    assert_eq!(o.text.as_ref().unwrap().style_id, Some(id));
    // An own change detaches.
    let mut own = get(&p, 1, other);
    own.text.as_mut().unwrap().style.size = 120.0;
    p.surfaces[1].replace(&[own]);
    p.relink();
    assert_eq!(get(&p, 1, other).text.unwrap().style_id, None);
    assert_eq!(get(&p, 0, t).text.unwrap().style_id, Some(id));
}

#[test]
fn style_names() {
    let mut p = project();
    let (r0, r1, t0) = (p.add(rect(0.0)), p.add(rect(10.0)), p.add(text("A")));
    let a = p.new_graphic_style(r0, "Style").unwrap();
    let b = p.new_graphic_style(r1, "Style").unwrap();
    let t = p.new_text_style(t0, "Style").unwrap();
    assert_eq!(p.graphic_style(b).unwrap().name, "Style 2");
    assert!(!p.rename_style(b, "Style 1"), "same kind, same name");
    assert!(!p.rename_style(b, " "));
    assert!(p.rename_style(b, "Stripe"));
    assert!(
        p.rename_style(t, "Style 1"),
        "another kind may share a name"
    );
    assert_eq!(p.graphic_style(a).unwrap().name, "Style 1");
}

/// A blue text with a 6 px red stroke at 70% opacity, Inter Black 400 px.
fn lettering() -> Object {
    let mut o = text("ACE");
    o.fill = Paint::Solid(BLUE);
    o.stroke = Some(StrokeStyle {
        paint: Paint::Solid(Rgba::rgb(255, 0, 0)),
        width: 6.0,
        ..StrokeStyle::default()
    });
    o.opacity = 0.7;
    let style = &mut o.text.as_mut().unwrap().style;
    style.weight = 900;
    style.size = 400.0;
    o
}

#[test]
fn a_text_style_carries_the_whole_lettering() {
    let mut p = project();
    let source = p.add(lettering());
    let id = p.new_text_style(source, "Text style").unwrap();
    let other = p.add(text("LOGISTICS"));
    p.apply_text_style(id, &[other]);
    p.relink();
    let o = get(&p, 0, other);
    assert_eq!(o.fill, Paint::Solid(BLUE));
    let stroke = o.stroke.unwrap();
    assert_eq!(
        (stroke.paint, stroke.width),
        (Paint::Solid(Rgba::rgb(255, 0, 0)), 6.0)
    );
    assert_eq!(o.opacity, 0.7);
    let t = o.text.unwrap();
    assert_eq!((t.style.weight, t.style.size), (900, 400.0));
    assert_eq!(t.style_id, Some(id));
    assert_eq!(get(&p, 0, source).text.unwrap().style_id, Some(id));
}

#[test]
fn recoloring_a_lettering_detaches_it() {
    let mut p = project();
    let source = p.add(lettering());
    let id = p.new_text_style(source, "Text style").unwrap();
    let mut green = get(&p, 0, source);
    green.fill = Paint::Solid(Rgba::rgb(0, 200, 0));
    p.surface_mut().replace(&[green]);
    p.relink();
    assert_eq!(get(&p, 0, source).text.unwrap().style_id, None);
    assert!(p.text_style(id).is_some());
}

#[test]
fn a_text_style_follows_its_swatch_and_replaces_a_graphic_style() {
    let mut p = project();
    let (red, _) = p.add_swatch(RED, "Color");
    let mut source = lettering();
    source.fill = Paint::Solid(RED);
    source.fill_swatch = Some(red);
    let source = p.add(source);
    let id = p.new_text_style(source, "Text style").unwrap();
    // A text following a graphic style switches to the text style.
    let other = p.add(text("B"));
    let shape_style = p.new_graphic_style(other, "Style").unwrap();
    assert_eq!(get(&p, 0, other).style, Some(shape_style));
    p.apply_text_style(id, &[other]);
    p.relink();
    let o = get(&p, 0, other);
    assert_eq!(
        (o.style, o.text.as_ref().unwrap().style_id),
        (None, Some(id))
    );
    p.set_swatch_color(red, DARK_RED);
    p.relink();
    let o = get(&p, 0, other);
    assert_eq!(o.fill, Paint::Solid(DARK_RED));
    assert_eq!(o.text.unwrap().style_id, Some(id), "still follows");
}
