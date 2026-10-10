use std::sync::Arc;

use kurbo::{Affine, Point, Size};

use super::*;
use crate::document::{CharStyle, Frame, ObjectId, Rgba, TextBlock};
use crate::project::{AssetKind, TextureResolution};

const VERT: Rgba = Rgba::rgb(0x1E, 0x8C, 0x3A);
const PNG: &[u8] = b"\x89PNG logo";

fn rect(x: f64) -> Object {
    Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(x, 100.0), Size::new(100.0, 50.0), 0.0),
    )
}

fn text(content: &str) -> Object {
    let mut o = rect(300.0);
    o.kind = ShapeKind::Text;
    o.text = Some(TextBlock::new(content, CharStyle::default()));
    o
}

/// The project's element ids.
struct Ardent {
    logo: SymbolId,
    vert: SwatchId,
    titre: StyleId,
    image: AssetId,
}

/// A project with the symbol "Logo Ardent": an image, the text "ARDENT"
/// following the text style "Titre", and a rectangle linked to the swatch
/// "Vert Ardent".
fn ardent() -> (Project, Ardent) {
    let mut p = Project::new("A", TextureResolution::R2048);
    let (vert, _) = p.add_swatch(VERT, "Color");
    p.rename_swatch(vert, "Vert Ardent");
    let (image, _) = p.add_asset(
        "logo",
        AssetKind::Raster,
        Arc::from(PNG),
        Size::new(8.0, 8.0),
    );
    let picture = p.add(Object::new(
        ObjectId(0),
        ShapeKind::Image { asset: image },
        Frame::new(Point::new(0.0, 0.0), Size::new(80.0, 40.0), 0.0),
    ));
    let lettering = p.add(text("ARDENT"));
    let titre = p.new_text_style(lettering, "Text style").unwrap();
    p.rename_style(titre, "Titre");
    let mut stripe = rect(500.0);
    stripe.fill = Paint::Solid(VERT);
    stripe.fill_swatch = Some(vert);
    let stripe = p.add(stripe);
    let (logo, _) = p
        .convert_to_symbol(&[picture, lettering, stripe], "Symbol")
        .unwrap();
    p.rename_symbol(logo, "Logo Ardent");
    let ids = Ardent {
        logo,
        vert,
        titre,
        image,
    };
    (p, ids)
}

fn symbol_picks(id: SymbolId) -> Picks {
    Picks {
        symbols: vec![id],
        ..Picks::default()
    }
}

fn empty() -> Project {
    Project::new("B", TextureResolution::R2048)
}

/// Every object of `list` and their descendants.
fn all(list: &[Arc<Object>]) -> Vec<Object> {
    let mut out = Vec::new();
    for o in list {
        out.push((**o).clone());
        out.extend(all(&o.children));
    }
    out
}

/// A library made by publishing `picks` of `from`: keys "k1", "k2"…
fn library_of(from: &mut Project, picks: &Picks) -> Project {
    let mut library = Project::new("Library", TextureResolution::R2048);
    let mut n = 0;
    let mut new_key = || {
        n += 1;
        LibraryKey(format!("k{n}"))
    };
    let done = import(from, picks, &mut library, ImportMode::Publish(&mut new_key));
    from.record_origins(&done.origins);
    library
}

// ── Closure ─────────────────────────────────────────────────────────────

#[test]
fn a_symbol_brings_its_image_text_style_and_swatch() {
    let (p, ids) = ardent();
    let c = closure(&p, &symbol_picks(ids.logo));
    assert_eq!(
        c,
        Closure {
            symbols: vec![ids.logo],
            swatches: vec![ids.vert],
            graphic_styles: vec![],
            text_styles: vec![ids.titre],
            assets: vec![ids.image],
        }
    );
}

#[test]
fn a_graphic_style_brings_the_swatch_its_look_links_to() {
    let mut p = empty();
    let (vert, _) = p.add_swatch(VERT, "Color");
    let mut o = rect(0.0);
    o.fill = Paint::Solid(VERT);
    o.fill_swatch = Some(vert);
    let o = p.add(o);
    let style = p.new_graphic_style(o, "Style").unwrap();
    let c = closure(
        &p,
        &Picks {
            graphic_styles: vec![style],
            ..Picks::default()
        },
    );
    assert_eq!(c.swatches, vec![vert]);
    assert_eq!(c.graphic_styles, vec![style]);
    assert!(c.symbols.is_empty() && c.assets.is_empty());
}

// ── Import ──────────────────────────────────────────────────────────────

#[test]
fn a_symbol_and_its_dependencies_into_an_empty_project() {
    let (a, ids) = ardent();
    let mut b = empty();
    let done = import(&a, &symbol_picks(ids.logo), &mut b, ImportMode::Import);
    assert_eq!(done.added, 3);
    assert_eq!(b.symbols.len(), 1);
    assert_eq!(b.symbols[0].name, "Logo Ardent");
    assert_eq!(b.palette.len(), 1);
    assert_eq!(
        (b.palette[0].name.as_str(), b.palette[0].color),
        ("Vert Ardent", VERT)
    );
    assert_eq!(b.text_styles[0].name, "Titre");
    assert_eq!(b.assets.len(), 1);
    // Every link points to the new project's elements, and holds.
    let content = all(&b.symbols[0].surface.objects);
    let asset = *b.assets.keys().next().unwrap();
    assert!(content.iter().any(|o| o.kind == ShapeKind::Image { asset }));
    assert!(
        content
            .iter()
            .any(|o| o.fill_swatch == Some(b.palette[0].id))
    );
    let titre = b.text_styles[0].id;
    assert!(
        content
            .iter()
            .any(|o| o.text.as_ref().is_some_and(|t| t.style_id == Some(titre)))
    );
    // New ids: none of the source's elements or objects.
    assert_ne!(b.symbols[0].id, ids.logo);
    let source_ids: Vec<ObjectId> = all(&a.symbols[0].surface.objects)
        .iter()
        .map(|o| o.id)
        .collect();
    assert!(content.iter().all(|o| !source_ids.contains(&o.id)));
}

#[test]
fn an_identical_swatch_is_reused() {
    let (a, ids) = ardent();
    let mut b = empty();
    let (own, _) = b.add_swatch(VERT, "Color");
    b.rename_swatch(own, "Vert Ardent");
    import(&a, &symbol_picks(ids.logo), &mut b, ImportMode::Import);
    assert_eq!(b.palette.len(), 1, "no second Vert Ardent");
    let content = all(&b.symbols[0].surface.objects);
    assert!(content.iter().any(|o| o.fill_swatch == Some(own)));
}

#[test]
fn a_taken_name_gets_a_number() {
    let (a, ids) = ardent();
    let mut b = empty();
    let r = b.add(rect(0.0));
    let (own, _) = b.convert_to_symbol(&[r], "Symbol").unwrap();
    b.rename_symbol(own, "Logo Ardent");
    let (green, _) = b.add_swatch(Rgba::rgb(0, 255, 0), "Color");
    b.rename_swatch(green, "Vert Ardent");
    import(&a, &symbol_picks(ids.logo), &mut b, ImportMode::Import);
    let names: Vec<&str> = b.symbols.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["Logo Ardent", "Logo Ardent 2"]);
    // A different color under the same name is another swatch.
    let names: Vec<&str> = b.palette.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["Vert Ardent", "Vert Ardent 2"]);
}

#[test]
fn an_element_of_the_same_library_entry_is_reused_unchanged() {
    let (mut a, ids) = ardent();
    let library = library_of(&mut a, &symbol_picks(ids.logo));
    let key = library.palette[0].origin.clone().unwrap();
    // The project has its own copy of "Vert Ardent", renamed and recolored.
    let mut b = empty();
    let (own, _) = b.add_swatch(Rgba::rgb(0, 90, 0), "Color");
    b.rename_swatch(own, "Green");
    b.record_origins(&[(own.0, key)]);
    import(
        &library,
        &symbol_picks(library.symbols[0].id),
        &mut b,
        ImportMode::Import,
    );
    assert_eq!(b.palette.len(), 1);
    assert_eq!(
        (b.palette[0].name.as_str(), b.palette[0].color),
        ("Green", Rgba::rgb(0, 90, 0))
    );
    // The imported stripe follows the project's swatch.
    let content = all(&b.symbols[0].surface.objects);
    let stripe = content.iter().find(|o| o.fill_swatch == Some(own)).unwrap();
    assert_eq!(stripe.fill, Paint::Solid(Rgba::rgb(0, 90, 0)));
}

#[test]
fn pasted_objects_are_remapped_to_the_target() {
    let (mut a, ids) = ardent();
    let (image, _) = a.add_asset(
        "logo",
        AssetKind::Raster,
        Arc::from(PNG),
        Size::new(8.0, 8.0),
    );
    assert_eq!(image, ids.image);
    let picture = a.add(Object::new(
        ObjectId(0),
        ShapeKind::Image { asset: image },
        Frame::new(Point::new(0.0, 0.0), Size::new(80.0, 40.0), 0.0),
    ));
    let mut lettering = text("ARDENT");
    lettering.fill = Paint::Solid(VERT);
    lettering.fill_swatch = Some(ids.vert);
    let lettering = a.add(lettering);
    let instance = a.new_instance(ids.logo, Affine::IDENTITY).unwrap();
    let instance = a.add(instance);
    a.refresh_instances();
    let objects = a.selected_objects(&[picture, lettering, instance]);
    // The target already has other elements: ids differ between projects.
    let mut b = empty();
    b.add_swatch(Rgba::rgb(1, 2, 3), "Color");
    b.add_asset(
        "other",
        AssetKind::Svg,
        Arc::from(&b"<svg/>"[..]),
        Size::new(1.0, 1.0),
    );
    let done = import(
        &a,
        &Picks {
            objects,
            ..Picks::default()
        },
        &mut b,
        ImportMode::Import,
    );
    let vert = b
        .palette
        .iter()
        .find(|s| s.name == "Vert Ardent")
        .unwrap()
        .id;
    let asset = b.assets.values().find(|a| &*a.bytes == PNG).unwrap().id;
    let logo = b.symbols[0].id;
    let [picture, lettering, instance] = done.objects.as_slice() else {
        panic!("three objects");
    };
    assert_eq!(picture.kind, ShapeKind::Image { asset });
    assert_eq!(lettering.fill_swatch, Some(vert));
    assert!(matches!(instance.kind, ShapeKind::Instance { symbol, .. } if symbol == logo));
}

#[test]
fn publishing_updates_the_entry_it_came_from() {
    let (mut a, ids) = ardent();
    let mut library = library_of(&mut a, &symbol_picks(ids.logo));
    // Every element brought is linked to its entry, in both projects.
    assert!(library.palette.iter().all(|s| s.origin.is_some()));
    assert!(library.text_styles.iter().all(|s| s.origin.is_some()));
    let key = library.symbols[0].origin.clone().unwrap();
    assert_eq!(a.symbol(ids.logo).unwrap().origin, Some(key.clone()));
    assert_eq!(
        a.swatch(ids.vert).unwrap().origin,
        library.palette[0].origin
    );
    let entry = library.symbols[0].id;
    // Edit the symbol and the swatch in the project, then publish again.
    a.rename_symbol(ids.logo, "Logo Ardent 2026");
    a.set_swatch_color(ids.vert, Rgba::rgb(0, 200, 0));
    let mut n = 100;
    let mut new_key = || {
        n += 1;
        LibraryKey(format!("k{n}"))
    };
    let done = import(
        &a,
        &symbol_picks(ids.logo),
        &mut library,
        ImportMode::Publish(&mut new_key),
    );
    assert_eq!(done.added, 0);
    assert_eq!(library.symbols.len(), 1);
    assert_eq!(library.symbols[0].id, entry, "same entry");
    assert_eq!(library.symbols[0].name, "Logo Ardent 2026");
    assert_eq!(library.symbols[0].origin, Some(key.clone()));
    assert_eq!(library.palette[0].color, Rgba::rgb(0, 200, 0));
    // The library's content follows the new color.
    let content = all(&library.symbols[0].surface.objects);
    let id = library.palette[0].id;
    let stripe = content.iter().find(|o| o.fill_swatch == Some(id)).unwrap();
    assert_eq!(stripe.fill, Paint::Solid(Rgba::rgb(0, 200, 0)));
    assert!(done.origins.contains(&(ids.logo.0, key)));
}

#[test]
fn importing_twice_adds_nothing_the_second_time() {
    let (mut a, ids) = ardent();
    let library = library_of(&mut a, &symbol_picks(ids.logo));
    let mut b = empty();
    let picks = symbol_picks(library.symbols[0].id);
    assert_eq!(
        import(&library, &picks, &mut b, ImportMode::Import).added,
        3
    );
    let before = b.clone();
    let done = import(&library, &picks, &mut b, ImportMode::Import);
    assert_eq!(done.added, 0);
    assert_eq!(b.symbols, before.symbols);
    assert_eq!(b.palette, before.palette);
    assert_eq!(b.text_styles, before.text_styles);
    assert_eq!(b.assets, before.assets);
    // The imported elements keep their library entries.
    let key = library.symbols[0].origin.clone().unwrap();
    assert!(b.has_origin(&key));
}

#[test]
fn unique_names_take_the_first_free_number() {
    let taken = ["Logo", "Logo 2", "Logo 4"];
    assert_eq!(unique_name("Logo", |n| taken.contains(&n)), "Logo 3");
    assert_eq!(unique_name("Badge", |n| taken.contains(&n)), "Badge");
}

// ── Shadows ─────────────────────────────────────────────────────────────

const NIGHT: Rgba = Rgba::rgb(10, 10, 30);

/// Adds the swatch "Night".
fn add_night(p: &mut Project) -> SwatchId {
    let (id, _) = p.add_swatch(NIGHT, "Color");
    p.rename_swatch(id, "Night");
    id
}

fn night_shadow(swatch: SwatchId) -> Option<Shadow> {
    Some(Shadow {
        color: NIGHT,
        swatch: Some(swatch),
        ..Shadow::DEFAULT
    })
}

#[test]
fn a_shadows_swatch_comes_along() {
    let mut p = empty();
    let night = add_night(&mut p);
    let mut t = text("TITRE");
    t.shadow = night_shadow(night);
    let t = p.add(t);
    let titre = p.new_text_style(t, "Titre").unwrap();
    let picks = Picks {
        text_styles: vec![titre],
        ..Picks::default()
    };
    assert_eq!(closure(&p, &picks).swatches, vec![night]);
    let library = library_of(&mut p, &picks);
    assert_eq!(library.palette.len(), 1);
    assert_eq!(library.palette[0].name, "Night");
    let shadow = library.text_styles[0].look.shadow.unwrap();
    assert_eq!(shadow.swatch, Some(library.palette[0].id));
}

#[test]
fn a_symbols_shadow_link_is_remapped_to_an_equal_swatch() {
    let mut a = empty();
    let night = add_night(&mut a);
    let mut t = text("ACE");
    t.shadow = night_shadow(night);
    let t = a.add(t);
    let (logo, _) = a.convert_to_symbol(&[t], "Symbol").unwrap();
    let mut b = empty();
    b.add_swatch(Rgba::rgb(1, 2, 3), "Other");
    let own = add_night(&mut b);
    assert_ne!(own, night);
    import(&a, &symbol_picks(logo), &mut b, ImportMode::Import);
    assert_eq!(b.palette.len(), 2, "the equal swatch is reused");
    let content = all(&b.symbols[0].surface.objects);
    let shadow = content.iter().find_map(|o| o.shadow).unwrap();
    assert_eq!((shadow.color, shadow.swatch), (NIGHT, Some(own)));
}

#[test]
fn a_pasted_shadow_without_its_swatch_is_unlinked() {
    let mut a = empty();
    let night = add_night(&mut a);
    let mut t = text("ACE");
    t.shadow = night_shadow(night);
    let mut b = empty();
    let done = import(
        &a,
        &Picks {
            objects: vec![t],
            ..Picks::default()
        },
        &mut b,
        ImportMode::Import,
    );
    // The swatch comes along with the pasted object.
    let shadow = done.objects[0].shadow.unwrap();
    assert_eq!(shadow.swatch, Some(b.palette[0].id));
    a.palette.clear();
    let mut c = empty();
    let mut orphan = text("B");
    orphan.shadow = night_shadow(night);
    let done = import(
        &a,
        &Picks {
            objects: vec![orphan],
            ..Picks::default()
        },
        &mut c,
        ImportMode::Import,
    );
    assert_eq!(done.objects[0].shadow.unwrap().swatch, None);
}
