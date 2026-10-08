//! The personal library file.

use std::sync::Arc;

use tp_core::document::{CharStyle, Frame, Object, ObjectId, Paint, Rgba, ShapeKind, TextBlock};
use tp_core::kurbo::{Point, Size};
use tp_core::{AssetKind, LibraryKey, Project};
use tp_file::{Error, library};

const PNG: &[u8] = b"\x89PNG logo";
const VERT: Rgba = Rgba::rgb(0x1E, 0x8C, 0x3A);

fn shape(kind: ShapeKind, x: f64) -> Object {
    Object::new(
        ObjectId(0),
        kind,
        Frame::new(Point::new(x, 100.0), Size::new(100.0, 50.0), 0.0),
    )
}

/// A library holding the symbol "Logo Ardent" (an image and a text), its
/// image, the swatch "Vert Ardent", a graphic style and a text style, each
/// with its key.
fn ardent_library() -> Project {
    let mut l = library::empty();
    let (vert, _) = l.add_swatch(VERT, "Color");
    l.rename_swatch(vert, "Vert Ardent");
    let (image, _) = l.add_asset(
        "logo",
        AssetKind::Raster,
        Arc::from(PNG),
        Size::new(8.0, 8.0),
    );
    let picture = l.add(shape(ShapeKind::Image { asset: image }, 0.0));
    let mut stripe = shape(ShapeKind::rectangle(), 200.0);
    stripe.fill = Paint::Solid(VERT);
    stripe.fill_swatch = Some(vert);
    let stripe = l.add(stripe);
    let graphic = l.new_graphic_style(stripe, "Bande").unwrap();
    let mut lettering = shape(ShapeKind::Text, 400.0);
    lettering.text = Some(TextBlock::new("ARDENT", CharStyle::default()));
    let lettering = l.add(lettering);
    let text = l.new_text_style(lettering, "Titre").unwrap();
    let (logo, _) = l
        .convert_to_symbol(&[picture, stripe, lettering], "Logo Ardent")
        .unwrap();
    // The placeholder surface holds nothing: its instance goes.
    l.surface_mut().objects.clear();
    let key = |n: u8| LibraryKey(format!("{n:032x}"));
    l.record_origins(&[
        (vert.0, key(1)),
        (graphic.0, key(2)),
        (text.0, key(3)),
        (logo.0, key(4)),
    ]);
    l
}

#[test]
fn a_library_round_trips() {
    let l = ardent_library();
    let back = library::from_bytes(&library::to_bytes(&l).unwrap()).unwrap();
    assert_eq!(back.palette, l.palette);
    assert_eq!(back.graphic_styles, l.graphic_styles);
    assert_eq!(back.text_styles, l.text_styles);
    assert_eq!(back.symbols, l.symbols);
    assert_eq!(back.assets, l.assets);
    assert!(back.vehicles.is_empty());
    assert_eq!(
        back.symbols[0].origin,
        Some(LibraryKey(format!("{:032x}", 4)))
    );
}

#[test]
fn a_library_file_is_written_atomically() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(library::FILE_NAME);
    library::write(&ardent_library(), &path).unwrap();
    assert_eq!(library::read(&path).unwrap().symbols.len(), 1);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn unused_assets_are_dropped_on_save() {
    let mut l = ardent_library();
    l.add_asset(
        "old",
        AssetKind::Svg,
        Arc::from(&b"<svg/>"[..]),
        Size::new(1.0, 1.0),
    );
    assert_eq!(l.assets.len(), 2);
    let back = library::from_bytes(&library::to_bytes(&l).unwrap()).unwrap();
    assert_eq!(back.assets.len(), 1);
    assert_eq!(&*back.assets.values().next().unwrap().bytes, PNG);
}

#[test]
fn projects_and_libraries_are_not_mistaken_for_each_other() {
    let project = tp_file::read(&fixture(1)).unwrap().project;
    let as_library = library::from_bytes(&tp_file::to_bytes(&project).unwrap());
    assert!(
        matches!(as_library, Err(Error::NotAProject)),
        "{as_library:?}"
    );
    let as_project = tp_file::from_bytes(&library::to_bytes(&ardent_library()).unwrap());
    assert!(
        matches!(as_project, Err(Error::NotAProject)),
        "{as_project:?}"
    );
    assert!(matches!(
        library::from_bytes(b"junk"),
        Err(Error::NotAProject)
    ));
}

fn fixture(version: u32) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(format!("v{version}.truckpaint"))
}
