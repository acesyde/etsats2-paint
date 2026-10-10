use kurbo::{Affine, Point, Size};

use super::*;
use crate::document::{
    CharStyle, ColorStop, Frame, Gradient, GradientKind, ObjectId, Rgba, StrokeStyle, TextBlock,
};
use crate::project::{Surface, TextureResolution};
use crate::symbols::Symbol;

const RED: Rgba = Rgba::rgb(200, 0, 0);

/// "Cab", "Chassis" and "Trailer", and the swatch "Company red".
fn fleet() -> (Project, SwatchId) {
    let mut p = Project::new("p", TextureResolution::R2048);
    p.surfaces[0].name = "Cab".into();
    p.surfaces.push(Surface::new("Chassis", 2048.0));
    p.surfaces.push(Surface::new("Trailer", 2048.0));
    let (red, _) = p.add_swatch(RED, "Color");
    p.rename_swatch(red, "Company red");
    (p, red)
}

fn rect() -> Object {
    Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(100.0, 100.0), Size::new(100.0, 50.0), 0.0),
    )
}

fn filled(id: SwatchId) -> Object {
    let mut o = rect();
    o.fill = Paint::Solid(RED);
    o.fill_swatch = Some(id);
    o
}

fn stroked(id: SwatchId) -> Object {
    let mut o = rect();
    o.stroke = Some(StrokeStyle {
        paint: Paint::Solid(RED),
        swatch: Some(id),
        ..StrokeStyle::default()
    });
    o
}

fn add_on(p: &mut Project, surface: usize, o: Object) -> ObjectId {
    let active = p.active_surface;
    p.active_surface = surface;
    let id = p.add(o);
    p.active_surface = active;
    id
}

/// The symbol "Logo" holding `content`.
fn logo(p: &mut Project, content: Vec<Object>) -> SymbolId {
    let id = SymbolId(p.fresh_id());
    p.symbols.push(Symbol {
        id,
        name: "Logo".into(),
        surface: Surface::new("Logo", 500.0),
        origin: None,
    });
    p.editing_symbol = Some(id);
    for o in content {
        p.add(o);
    }
    p.editing_symbol = None;
    id
}

fn place(p: &mut Project, symbol: SymbolId, surface: usize) -> ObjectId {
    let o = p.new_instance(symbol, Affine::IDENTITY).unwrap();
    let id = add_on(p, surface, o);
    p.refresh_instances();
    id
}

#[test]
fn a_swatch_used_on_two_textures() {
    let (mut p, red) = fleet();
    for _ in 0..3 {
        add_on(&mut p, 0, filled(red));
    }
    add_on(&mut p, 1, stroked(red));
    let count = p.usage().swatch(red).clone();
    assert_eq!(count.objects, 4);
    assert_eq!(count.textures(), 2);
    assert_eq!(count.surfaces, [0, 1]);
}

#[test]
fn through_a_symbol() {
    let (mut p, red) = fleet();
    let logo = logo(&mut p, vec![filled(red), rect()]);
    for surface in [0, 0, 1, 2, 2] {
        place(&mut p, logo, surface);
    }
    let usage = p.usage();
    assert_eq!(usage.swatch(red).objects, 5, "an instance counts once");
    assert_eq!(usage.swatch(red).textures(), 3);
}

#[test]
fn an_object_linked_twice_counts_once() {
    let (mut p, red) = fleet();
    let mut o = filled(red);
    o.stroke = stroked(red).stroke;
    add_on(&mut p, 0, o);
    assert_eq!(p.usage().swatch(red).objects, 1);
}

#[test]
fn unused_swatch() {
    let (mut p, _) = fleet();
    let (cream, _) = p.add_swatch(Rgba::rgb(250, 240, 220), "Color");
    add_on(&mut p, 0, rect());
    let usage = p.usage();
    assert!(usage.swatch(cream).is_unused());
    assert_eq!(usage.swatch(cream).textures(), 0);
}

#[test]
fn symbol_usage() {
    let (mut p, _) = fleet();
    let logo = logo(&mut p, vec![rect()]);
    for surface in [0, 0, 0, 0, 1, 1] {
        place(&mut p, logo, surface);
    }
    let count = p.usage().symbol(logo).clone();
    assert_eq!((count.objects, count.textures()), (6, 2));
}

#[test]
fn a_gradient_stop_links_its_object() {
    let (mut p, red) = fleet();
    let mut stop = ColorStop::new(0.0, RED);
    stop.swatch = Some(red);
    let mut o = rect();
    o.fill = Paint::Gradient(Gradient::new(
        GradientKind::Linear,
        &[stop, stop, ColorStop::new(1.0, Rgba::rgb(0, 0, 0))],
    ));
    add_on(&mut p, 2, o);
    let count = p.usage().swatch(red).clone();
    assert_eq!((count.objects, count.surfaces), (1, vec![2]));
}

#[test]
fn groups_count_their_objects_one_by_one() {
    let (mut p, red) = fleet();
    let a = add_on(&mut p, 0, filled(red));
    let b = add_on(&mut p, 0, filled(red));
    p.group(&[a, b]).unwrap();
    assert_eq!(p.usage().swatch(red).objects, 2);
}

#[test]
fn texts_following_a_text_style() {
    let (mut p, _) = fleet();
    let mut t = rect();
    t.kind = ShapeKind::Text;
    t.text = Some(TextBlock::new("ACE", CharStyle::default()));
    let first = add_on(&mut p, 0, t.clone());
    let style = p.new_text_style(first, "Text style").unwrap();
    for surface in [1, 2] {
        p.active_surface = surface;
        let other = p.add(t.clone());
        p.apply_text_style(style, &[other]);
    }
    p.active_surface = 0;
    let count = p.usage().style(style).clone();
    assert_eq!((count.objects, count.textures()), (3, 3));
}

#[test]
fn graphic_style_followers_and_instances() {
    let (mut p, _) = fleet();
    let a = add_on(&mut p, 0, rect());
    let style = p.new_graphic_style(a, "Style").unwrap();
    let b = add_on(&mut p, 1, rect());
    p.active_surface = 1;
    p.apply_graphic_style(style, &[b]);
    // A symbol whose content follows the style: each instance counts.
    let mut inside = rect();
    inside.style = Some(style);
    p.active_surface = 0;
    let logo = logo(&mut p, vec![inside]);
    place(&mut p, logo, 2);
    let count = p.usage().style(style).clone();
    assert_eq!((count.objects, count.surfaces), (3, vec![0, 1, 2]));
}

#[test]
fn symbol_counts_match_instance_count() {
    let (mut p, red) = fleet();
    let logo = logo(&mut p, vec![filled(red)]);
    let badge = logo_named(&mut p, "Badge");
    let mut placed = Vec::new();
    for (i, surface) in [0, 1, 1, 2, 2, 2].into_iter().enumerate() {
        let symbol = if i % 2 == 0 { logo } else { badge };
        placed.push((surface, place(&mut p, symbol, surface)));
    }
    // One instance inside a group.
    p.active_surface = 2;
    let extra = p.add(rect());
    let (_, last) = placed[placed.len() - 1];
    p.group(&[extra, last]).unwrap();
    p.active_surface = 0;
    let usage = p.usage();
    for id in [logo, badge] {
        assert_eq!(usage.symbol(id).objects, p.instance_count(id));
    }
    assert_eq!(usage.symbol(logo).objects, 3);
}

fn logo_named(p: &mut Project, name: &str) -> SymbolId {
    let id = logo(p, vec![rect()]);
    p.rename_symbol(id, name);
    id
}

#[test]
fn through_a_shadow() {
    let (mut p, _) = fleet();
    let (night, _) = p.add_swatch(Rgba::rgb(10, 10, 30), "Night");
    let mut t = rect();
    t.kind = ShapeKind::Text;
    t.text = Some(TextBlock::new("ACE", CharStyle::default()));
    t.shadow = Some(crate::document::Shadow {
        color: Rgba::rgb(10, 10, 30),
        swatch: Some(night),
        ..crate::document::Shadow::DEFAULT
    });
    add_on(&mut p, 0, t.clone());
    let count = p.usage().swatch(night).clone();
    assert_eq!((count.objects, count.textures()), (1, 1));
    // Inside a symbol's content, each instance counts.
    let logo = logo(&mut p, vec![t]);
    place(&mut p, logo, 1);
    assert_eq!(p.usage().swatch(night).objects, 2);
}
