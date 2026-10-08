use kurbo::{Point, Size};

use super::*;
use crate::document::{Frame, ObjectId, ShapeKind};
use crate::project::TextureResolution;

fn rect(x: f64) -> Object {
    Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(x, 100.0), Size::new(100.0, 50.0), 0.0),
    )
}

/// A project with the empty symbol "Logo" (artboard 500).
fn project_with_symbol() -> (Project, SymbolId) {
    let mut p = Project::new("p", TextureResolution::R2048);
    let id = SymbolId(p.fresh_id());
    p.symbols.push(Symbol {
        id,
        name: "Logo".into(),
        surface: Surface::new("Logo", 500.0),
        origin: None,
    });
    (p, id)
}

#[test]
fn editing_a_symbol_redirects_the_active_surface() {
    let (mut p, id) = project_with_symbol();
    let on_texture = p.add(rect(0.0));
    p.editing_symbol = Some(id);
    let in_symbol = p.add(rect(10.0));
    assert_eq!(p.surface().size, 500.0);
    assert!(p.surface().get(in_symbol).is_some());
    assert!(p.surface().get(on_texture).is_none());
    assert_eq!(p.symbol(id).unwrap().surface.objects.len(), 1);
    p.editing_symbol = None;
    assert!(p.surface().get(on_texture).is_some());
    assert_eq!(p.surface().objects.len(), 1);
}

#[test]
fn snapshots_carry_symbols_and_the_editing_state() {
    let (mut p, id) = project_with_symbol();
    p.editing_symbol = Some(id);
    let before = p.snapshot(&[]);
    p.add(rect(0.0));
    p.editing_symbol = None;
    let after = p.snapshot(&[]);
    assert!(!before.same_document(&after));
    p.restore(&before);
    assert_eq!(p.editing_symbol, Some(id));
    assert!(p.surface().objects.is_empty());
    assert_eq!(before.editing_symbol(), Some(id));
}

#[test]
fn ids_continue_after_the_symbols() {
    let mut p = Project::new("p", TextureResolution::R2048);
    let mut content = Surface::new("Logo", 500.0);
    let mut r = rect(0.0);
    r.id = ObjectId(500);
    content.objects.push(Arc::new(r));
    p.set_symbols(vec![Symbol {
        id: SymbolId(400),
        name: "Logo".into(),
        surface: content,
        origin: None,
    }]);
    assert!(p.next_object_id().0 > 500);
}

/// An instance of `symbol` showing `content` through `placement`.
fn instance(symbol: SymbolId, content: &[Object], placement: kurbo::Affine) -> Object {
    let mut o = Object::new(
        ObjectId(900),
        ShapeKind::Instance { symbol, placement },
        Frame::new(Point::ORIGIN, Size::ZERO, 0.0),
    );
    o.children = crate::document::apply_affine(content, placement)
        .into_iter()
        .map(Arc::new)
        .collect();
    o.refresh_group_frame();
    o
}

fn placement_of(o: &Object) -> kurbo::Affine {
    match o.kind {
        ShapeKind::Instance { placement, .. } => placement,
        _ => panic!("an instance"),
    }
}

/// The instance's children, as re-expanding its content would give them.
fn assert_in_step(o: &Object, content: &[Object]) {
    let expected = crate::document::apply_affine(content, placement_of(o));
    for (child, want) in o.children.iter().zip(&expected) {
        assert!(
            child.frame.center.distance(want.frame.center) < 1e-6,
            "{:?} vs {:?}",
            child.frame,
            want.frame
        );
        assert!((child.frame.size.width - want.frame.size.width).abs() < 1e-6);
    }
}

#[test]
fn an_instance_is_drawn_and_hit_like_a_group_but_has_no_shapes() {
    use crate::document::tree;
    let content = vec![rect(100.0), rect(300.0)];
    let i = instance(
        SymbolId(1),
        &content,
        kurbo::Affine::translate((1000.0, 0.0)),
    );
    let list = vec![Arc::new(i.clone())];
    assert_eq!(tree::draw_list(&list).len(), 2, "its content is drawn");
    let hit = tree::hit_test(&list, Point::new(1100.0, 100.0), 0.0).unwrap();
    assert_eq!((hit.top, hit.inner), (i.id, i.id), "selected as a whole");
    assert!(tree::hit_test(&list, Point::new(1200.0, 100.0), 0.0).is_none());
    let b = i.bounding_box();
    assert_eq!((b.x0, b.x1), (1050.0, 1350.0));
    assert!(i.shapes().is_empty(), "its look is its symbol's");
    assert!(i.has_content() && !i.is_group());
}

#[test]
fn transforms_keep_the_placement_in_step() {
    use crate::document::{Handle, ResizeOptions, resize, rotate, translate};
    let content = vec![rect(100.0), rect(300.0)];
    let i = instance(
        SymbolId(1),
        &content,
        kurbo::Affine::translate((1000.0, 0.0)),
    );
    let moved = &translate(std::slice::from_ref(&i), kurbo::Vec2::new(5.0, 7.0), false)[0];
    assert_in_step(moved, &content);
    let turned = &rotate(
        std::slice::from_ref(moved),
        Point::new(1200.0, 100.0),
        30.0,
        false,
    )[0];
    assert_in_step(turned, &content);
    let bounds = turned.frame;
    let resized = &resize(
        std::slice::from_ref(turned),
        bounds,
        Handle { x: 1, y: 1 },
        bounds.affine() * Point::new(bounds.size.width, bounds.size.height),
        ResizeOptions::default(),
    )[0];
    assert_in_step(resized, &content);
    // A flip: a resize by −1 mirrors the placement.
    let flipped = &resize(
        std::slice::from_ref(resized),
        resized.frame,
        Handle { x: 1, y: 0 },
        resized.frame.affine() * Point::new(-resized.frame.size.width / 2.0, 0.0),
        ResizeOptions {
            proportional: false,
            from_center: true,
        },
    )[0];
    assert!(placement_of(flipped).determinant() < 0.0, "mirrored");
    assert_in_step(flipped, &content);
}

/// "Logo" holding two rectangles, and an instance of it on the texture
/// through `placement` (expanded by `refresh_instances`).
fn with_instance(placement: kurbo::Affine) -> (Project, SymbolId, ObjectId) {
    let (mut p, id) = project_with_symbol();
    p.editing_symbol = Some(id);
    p.add(rect(100.0));
    p.add(rect(300.0));
    p.editing_symbol = None;
    let mut i = Object::new(
        ObjectId(0),
        ShapeKind::Instance {
            symbol: id,
            placement,
        },
        Frame::new(Point::ORIGIN, Size::ZERO, 0.0),
    );
    i.name = "Logo".into();
    let instance = p.add(i);
    p.refresh_instances();
    (p, id, instance)
}

fn content(p: &Project, id: SymbolId) -> Vec<Object> {
    p.symbol(id)
        .unwrap()
        .surface
        .objects
        .iter()
        .map(|o| (**o).clone())
        .collect()
}

#[test]
fn expansion_at_100_percent_equals_the_content() {
    let (p, id, i) = with_instance(kurbo::Affine::IDENTITY);
    let o = p.surface().get(i).unwrap();
    let frames: Vec<Frame> = o.children.iter().map(|c| c.frame).collect();
    let want: Vec<Frame> = content(&p, id).iter().map(|c| c.frame).collect();
    assert_eq!(frames, want);
    let child_ids: Vec<ObjectId> = o.children.iter().map(|c| c.id).collect();
    assert!(
        child_ids
            .iter()
            .all(|c| content(&p, id).iter().all(|s| s.id != *c)),
        "own ids"
    );
}

#[test]
fn a_rotated_mirrored_instance_matches_apply_affine() {
    let placement = kurbo::Affine::translate((800.0, 300.0))
        * kurbo::Affine::rotate(0.5)
        * kurbo::Affine::scale_non_uniform(-1.5, 1.5);
    let (p, id, i) = with_instance(placement);
    let o = (**p.surface().get(i).unwrap()).clone();
    assert_in_step(&o, &content(&p, id));
}

#[test]
fn re_expansion_keeps_ids_and_unchanged_arcs() {
    let (mut p, id, i) = with_instance(kurbo::Affine::translate((1000.0, 0.0)));
    let before = p.surface().get(i).unwrap().clone();
    p.refresh_instances();
    assert!(
        Arc::ptr_eq(&before, p.surface().get(i).unwrap()),
        "unchanged"
    );
    // Recolor a shape in the symbol: the instance follows, ids kept.
    p.editing_symbol = Some(id);
    let mut first = (*p.surface().objects[0]).clone();
    first.fill = crate::document::Paint::Solid(crate::document::Rgba::rgb(1, 2, 3));
    p.surface_mut().replace(&[first]);
    p.editing_symbol = None;
    p.refresh_instances();
    let after = p.surface().get(i).unwrap();
    assert_eq!(
        after.children[0].fill,
        crate::document::Paint::Solid(crate::document::Rgba::rgb(1, 2, 3))
    );
    let ids = |o: &Object| o.children.iter().map(|c| c.id).collect::<Vec<_>>();
    assert_eq!(ids(after), ids(&before));
}

#[test]
fn an_instance_of_a_missing_symbol_becomes_a_group() {
    let (mut p, _, i) = with_instance(kurbo::Affine::IDENTITY);
    p.symbols.clear();
    p.refresh_instances();
    let o = p.surface().get(i).unwrap();
    assert!(o.is_group());
    assert_eq!(o.children.len(), 2);
}

fn circle(x: f64) -> Object {
    Object::new(
        ObjectId(0),
        ShapeKind::Ellipse,
        Frame::new(Point::new(x, 400.0), Size::new(80.0, 80.0), 15.0),
    )
}

/// The frames the objects `ids` of the active surface show, in paint order.
fn shown_frames(p: &Project, id: ObjectId) -> Vec<Frame> {
    p.surface()
        .get(id)
        .unwrap()
        .children
        .iter()
        .map(|c| c.frame)
        .collect()
}

fn close_frames(a: &[Frame], b: &[Frame]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b).all(|(x, y)| {
            x.center.distance(y.center) < 1e-6
                && (x.size.width - y.size.width).abs() < 1e-6
                && (x.rotation_deg - y.rotation_deg).abs() < 1e-6
        })
}

#[test]
fn convert_to_symbol_replaces_the_selection_where_it_was() {
    let mut p = Project::new("p", TextureResolution::R2048);
    let a = p.add(circle(300.0));
    let b = p.add(rect(500.0));
    let frames: Vec<Frame> = [a, b]
        .iter()
        .map(|id| p.surface().get(*id).unwrap().frame)
        .collect();
    let snap = p.snapshot(&[a, b]);
    let (symbol, instance) = p.convert_to_symbol(&[a, b], "Symbol").unwrap();
    let s = p.symbol(symbol).unwrap();
    assert_eq!(s.name, "Symbol 1");
    assert_eq!(s.surface.objects.len(), 2);
    // The artboard just encloses the content (the larger side).
    let content_bounds = s
        .surface
        .objects
        .iter()
        .map(|o| o.bounding_box())
        .reduce(|x, y| x.union(y))
        .unwrap();
    assert!(
        (s.surface.size - content_bounds.width().max(content_bounds.height()).ceil()).abs() < 1e-9
    );
    assert_eq!(p.surface().objects.len(), 1);
    assert!(p.surface().get(instance).unwrap().is_instance());
    assert!(
        close_frames(&shown_frames(&p, instance), &frames),
        "shown where they were"
    );
    assert_eq!(p.instance_count(symbol), 1);
    // Undo.
    p.restore(&snap);
    assert!(p.symbols.is_empty());
    assert!(p.surface().get(a).is_some() && p.surface().get(b).is_some());
}

#[test]
fn convert_inside_a_group_keeps_the_parent_and_position() {
    let mut p = Project::new("p", TextureResolution::R2048);
    let below = p.add(rect(0.0));
    let a = p.add(rect(100.0));
    let b = p.add(rect(200.0));
    let above = p.add(rect(300.0));
    let group = p.group(&[below, a, b, above]).unwrap();
    let (_, instance) = p.convert_to_symbol(&[a, b], "Symbol").unwrap();
    let g = p.surface().get(group).unwrap();
    let order: Vec<ObjectId> = g.children.iter().map(|c| c.id).collect();
    assert_eq!(order, vec![below, instance, above]);
}

#[test]
fn instances_in_the_selection_are_detached_first() {
    let mut p = Project::new("p", TextureResolution::R2048);
    let a = p.add(rect(100.0));
    let (first, inner) = p.convert_to_symbol(&[a], "Symbol").unwrap();
    let c = p.add(circle(600.0));
    let (second, _) = p.convert_to_symbol(&[inner, c], "Symbol").unwrap();
    let content = &p.symbol(second).unwrap().surface.objects;
    fn has_instance(list: &[Arc<Object>]) -> bool {
        list.iter()
            .any(|o| o.is_instance() || has_instance(&o.children))
    }
    assert!(!has_instance(content), "symbols don't nest");
    assert_eq!(p.instance_count(first), 0);
    assert_eq!(p.instance_count(second), 1);
}

#[test]
fn detach_duplicate_delete_and_rename() {
    let mut p = Project::new("p", TextureResolution::R2048);
    p.surfaces.push(Surface::new("Chassis", 2048.0));
    let a = p.add(rect(100.0));
    let (symbol, instance) = p.convert_to_symbol(&[a], "Symbol").unwrap();
    // A second instance on the other texture.
    let placement = p.placement_at(symbol, Point::new(1000.0, 1000.0)).unwrap();
    let other = p.new_instance(symbol, placement).unwrap();
    p.active_surface = 1;
    let other = p.add(other);
    p.refresh_instances();
    assert_eq!(p.instance_count(symbol), 2);
    let shown = p.surface().get(other).unwrap().bounding_box().center();
    assert!(
        shown.distance(Point::new(1000.0, 1000.0)) < 1e-6,
        "centered on the point"
    );
    // Detach the one on the chassis.
    assert_eq!(p.detach_instances(&[other]), vec![other]);
    let detached = p.surface().get(other).unwrap();
    assert!(detached.is_group() && detached.children.len() == 1);
    assert_eq!(p.instance_count(symbol), 1);
    p.active_surface = 0;
    // Duplicate: same content, no instance, fresh ids.
    let copy = p.duplicate_symbol(symbol, "copy").unwrap();
    assert_eq!(p.symbol(copy).unwrap().name, "Symbol 1 copy");
    assert_eq!(p.instance_count(copy), 0);
    assert_ne!(
        p.symbol(copy).unwrap().surface.objects[0].id,
        p.symbol(symbol).unwrap().surface.objects[0].id
    );
    let second = p.duplicate_symbol(symbol, "copy").unwrap();
    assert_eq!(p.symbol(second).unwrap().name, "Symbol 1 copy 2");
    // Names.
    assert!(!p.rename_symbol(symbol, " "));
    assert!(!p.rename_symbol(symbol, "Symbol 1 copy"));
    assert!(p.rename_symbol(symbol, "Logo"));
    // Delete: the instance becomes a group that looks the same.
    let frames = shown_frames(&p, instance);
    p.delete_symbol(symbol);
    let o = p.surface().get(instance).unwrap();
    assert!(o.is_group());
    assert!(close_frames(&shown_frames(&p, instance), &frames));
    assert!(p.symbol(symbol).is_none());
}

#[test]
fn recolor_a_swatch_used_in_a_symbol() {
    use crate::document::{Paint, Rgba};
    let red = Rgba::rgb(200, 0, 0);
    let dark = Rgba::rgb(0x8B, 0, 0);
    let mut p = Project::new("p", TextureResolution::R2048);
    let (swatch, _) = p.add_swatch(red, "Color");
    let mut shape = rect(100.0);
    shape.fill = Paint::Solid(red);
    shape.fill_swatch = Some(swatch);
    let a = p.add(shape);
    let (symbol, instance) = p.convert_to_symbol(&[a], "Symbol").unwrap();
    let placement = p.placement_at(symbol, Point::new(1500.0, 1500.0)).unwrap();
    let second = p.new_instance(symbol, placement).unwrap();
    let second = p.add(second);
    p.refresh_instances();
    p.set_swatch_color(swatch, dark);
    for i in [instance, second] {
        assert_eq!(
            p.surface().get(i).unwrap().children[0].fill,
            Paint::Solid(dark)
        );
    }
    assert_eq!(
        p.symbol(symbol).unwrap().surface.objects[0].fill,
        Paint::Solid(dark)
    );
    // Relinking keeps the link inside the symbol.
    p.relink();
    assert_eq!(
        p.symbol(symbol).unwrap().surface.objects[0].fill_swatch,
        Some(swatch)
    );
}

#[test]
fn a_style_redefined_on_a_texture_reaches_symbol_content() {
    use crate::document::{Paint, Rgba};
    let mut p = Project::new("p", TextureResolution::R2048);
    let source = p.add(rect(100.0));
    let style = p.new_graphic_style(source, "Style").unwrap();
    let inside = p.add(rect(400.0));
    p.apply_graphic_style(style, &[inside]);
    let (symbol, instance) = p.convert_to_symbol(&[inside], "Symbol").unwrap();
    let mut blue = (**p.surface().get(source).unwrap()).clone();
    blue.fill = Paint::Solid(Rgba::rgb(0, 0, 255));
    p.surface_mut().replace(&[blue]);
    assert!(p.redefine_graphic_style(style, source));
    let content = &p.symbol(symbol).unwrap().surface.objects[0];
    assert_eq!(content.fill, Paint::Solid(Rgba::rgb(0, 0, 255)));
    assert_eq!(content.style, Some(style));
    assert_eq!(
        p.surface().get(instance).unwrap().children[0].fill,
        Paint::Solid(Rgba::rgb(0, 0, 255))
    );
    // The Styles panel's users of the texture don't include the instance's
    // content.
    assert_eq!(p.style_users(style, 0), vec![source]);
}

#[test]
fn an_image_in_a_symbol_counts_once() {
    use crate::{AssetKind, document::AssetId};
    let mut p = Project::new("p", TextureResolution::R2048);
    let (asset, _) = p.add_asset(
        "logo",
        AssetKind::Svg,
        Arc::from(&b"<svg/>"[..]),
        Size::new(10.0, 10.0),
    );
    let image = Object::new(
        ObjectId(0),
        ShapeKind::Image { asset },
        Frame::new(Point::new(100.0, 100.0), Size::new(50.0, 50.0), 0.0),
    );
    let a = p.add(image);
    let (symbol, _) = p.convert_to_symbol(&[a], "Symbol").unwrap();
    for x in [500.0, 900.0] {
        let placement = p.placement_at(symbol, Point::new(x, x)).unwrap();
        let i = p.new_instance(symbol, placement).unwrap();
        p.add(i);
    }
    p.refresh_instances();
    assert_eq!(p.instance_count(symbol), 3);
    assert_eq!(p.asset_usage(asset), 1);
    let _: AssetId = asset;
}

#[test]
fn flipping_an_instance_mirrors_its_images_but_not_the_symbol() {
    use crate::document::{AssetId, FlipAxis, flip};
    let (mut p, id) = project_with_symbol();
    p.editing_symbol = Some(id);
    p.add(Object::new(
        ObjectId(0),
        ShapeKind::Image { asset: AssetId(7) },
        Frame::new(Point::new(100.0, 100.0), Size::new(80.0, 40.0), 0.0),
    ));
    p.editing_symbol = None;
    let mut ids = Vec::new();
    for x in [0.0, 1000.0] {
        let i = Object::new(
            ObjectId(0),
            ShapeKind::Instance {
                symbol: id,
                placement: kurbo::Affine::translate((x, 0.0)),
            },
            Frame::new(Point::ORIGIN, Size::ZERO, 0.0),
        );
        ids.push(p.add(i));
    }
    p.refresh_instances();
    let first = (**p.surface().get(ids[0]).unwrap()).clone();
    let flipped = flip(&[first], FlipAxis::Horizontal);
    p.surface_mut().replace(&flipped);
    p.refresh_instances();
    let shown = |p: &Project, i: ObjectId| p.surface().get(i).unwrap().children[0].mirrored;
    assert!(placement_of(p.surface().get(ids[0]).unwrap()).determinant() < 0.0);
    assert!(
        shown(&p, ids[0]),
        "the flipped instance shows the image mirrored"
    );
    assert!(!shown(&p, ids[1]), "the other instance does not");
    assert!(!content(&p, id)[0].mirrored, "the symbol is unchanged");
    // A mirrored image in the symbol shows unmirrored in a mirrored instance.
    p.editing_symbol = Some(id);
    let image = content(&p, id)[0].clone();
    let mirrored = flip(&[image], FlipAxis::Horizontal);
    p.surface_mut().replace(&mirrored);
    p.editing_symbol = None;
    p.refresh_instances();
    assert!(!shown(&p, ids[0]) && shown(&p, ids[1]));
}
