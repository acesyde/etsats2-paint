//! Round trips, versions and errors of the `.truckpaint` format.

use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use tp_core::document::{
    Cap, CharStyle, ColorStop, Dash, Frame, Gradient, GradientKind, Join, LineStyle, Node, Object,
    ObjectId, Paint, PathData, Rgba, ShapeKind, StrokeAlign, StrokeStyle, Subpath, TextAlign,
    TextBlock,
};
use tp_core::kurbo::{Point, Size, Vec2};
use tp_core::{AssetKind, Project, TextureResolution};
use tp_file::{Error, FORMAT_VERSION};

const PNG: &[u8] = &[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n', 1, 2, 3];
const SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="300" height="100"/>"#;

/// A project using every kind of content.
fn rich_project() -> Project {
    let mut p = Project::new("ACE Logistics", TextureResolution::R4096);
    let shape = |kind, x: f64| {
        Object::new(
            ObjectId(0),
            kind,
            Frame::new(Point::new(x, 500.0), Size::new(300.0, 200.0), 12.5),
        )
    };
    let mut base = shape(
        ShapeKind::Rectangle {
            corner_radius: 24.0,
        },
        1000.0,
    );
    base.name = "Burgundy base".into();
    base.fill = Paint::Solid(Rgba::rgb(0x7A, 0x1F, 0x2B));
    base.stroke = Some(StrokeStyle {
        paint: Paint::Solid(Rgba::rgb(0, 0, 0)),
        width: 6.0,
        align: StrokeAlign::Outside,
        line: LineStyle {
            dash: Some(Dash {
                dash: 12.0,
                gap: 6.0,
            }),
            cap: Cap::Butt,
            join: Join::Round,
            miter_limit: 4.0,
        },
        swatch: None,
    });
    let base = p.add(base);
    let mut stripe = shape(ShapeKind::Ellipse, 1500.0);
    stripe.opacity = 0.5;
    stripe.visible = false;
    let stripe = p.add(stripe);
    let group = p.group(&[base, stripe]).unwrap();
    let mut g = (**p.surface().get(group).unwrap()).clone();
    g.name = "Graphics".into();
    g.locked = true;
    p.surface_mut().replace(&[g]);

    let mut block = TextBlock::new(
        "ACE\nLOGISTICS",
        CharStyle {
            family: "Barlow Condensed".into(),
            weight: 600,
            italic: true,
            size: 320.0,
            align: TextAlign::Center,
            letter_spacing: 40.0,
            line_height: 95.0,
        },
    );
    block.layout_size = Size::new(900.0, 600.0);
    block.scale = Vec2::new(1.5, 1.0);
    let mut text = Object::text(ObjectId(0), block, Point::new(2000.0, 1500.0));
    text.frame.rotation_deg = -10.0;
    p.add(text);

    let (png, _) = p.add_asset(
        "logo",
        AssetKind::Raster,
        Arc::from(PNG),
        Size::new(800.0, 400.0),
    );
    let (svg, _) = p.add_asset(
        "badge",
        AssetKind::Svg,
        Arc::from(SVG),
        Size::new(300.0, 100.0),
    );
    for (asset, name) in [(png, "logo"), (svg, "badge")] {
        let mut image = shape(ShapeKind::Image { asset }, 3000.0);
        image.name = name.into();
        p.add(image);
    }
    p.add_to_palette(Rgba::rgb(0xF0, 0xB4, 0x4C));
    for o in vector_objects().into_iter().chain(gradient_objects()) {
        p.add(o);
    }
    p.add_guide(tp_core::Guide::new(tp_core::Axis::Vertical, 2048.0));
    p.add_guide(tp_core::Guide::new(tp_core::Axis::Horizontal, 1200.5));
    add_brand(&mut p);
    add_vehicle(&mut p);
    add_symbol(&mut p);
    p
}

/// The symbol "Logo" (a circle and a text), shown by its first instance on
/// the main surface and by a rotated, mirrored instance on the second one.
fn add_symbol(p: &mut Project) {
    let mut circle = Object::new(
        ObjectId(0),
        ShapeKind::Ellipse,
        Frame::new(Point::new(3000.0, 3000.0), Size::new(200.0, 200.0), 0.0),
    );
    circle.fill = Paint::Solid(Rgba::rgb(0x20, 0x40, 0x80));
    let mut text = Object::new(
        ObjectId(0),
        ShapeKind::Text,
        Frame::new(Point::new(3000.0, 3200.0), Size::new(300.0, 80.0), 0.0),
    );
    text.text = Some(TextBlock::new("ACE", CharStyle::default()));
    let (a, b) = (p.add(circle), p.add(text));
    let (logo, _) = p.convert_to_symbol(&[a, b], "Symbol").unwrap();
    p.rename_symbol(logo, "Logo");
    let placement = tp_core::kurbo::Affine::translate((800.0, 600.0))
        * tp_core::kurbo::Affine::rotate(0.6)
        * tp_core::kurbo::Affine::scale_non_uniform(-0.5, 0.5);
    let second = p.new_instance(logo, placement).unwrap();
    p.active_surface = 1;
    p.add(second);
    p.active_surface = 0;
    p.refresh_instances();
}

/// The surfaces with every instance's content ids cleared (they are new
/// after opening a file).
fn without_content_ids(p: &Project) -> Vec<tp_core::Surface> {
    fn clear(o: &mut Object, inside: bool) {
        if inside {
            o.id = ObjectId(0);
        }
        let inside = inside || o.is_instance();
        for c in &mut o.children {
            clear(Arc::make_mut(c), inside);
        }
    }
    p.surfaces
        .iter()
        .cloned()
        .map(|mut s| {
            for o in &mut s.objects {
                clear(Arc::make_mut(o), false);
            }
            s
        })
        .collect()
}

const COMPANY_RED: Rgba = Rgba::rgb(0xC0, 0x10, 0x20);
const COMPANY_GREY: Rgba = Rgba::rgb(0x50, 0x55, 0x5A);

/// The swatches "Company red" and "Company grey"; the graphic style
/// "Style 1", a gradient whose first stop is linked to "Company red", and
/// an ellipse "Styled" following it; the text style "Text style 1" and the
/// text "Brand lettering" following it, its fill linked to "Company grey".
fn add_brand(p: &mut Project) {
    let (red, _) = p.add_swatch(COMPANY_RED, "Color");
    let (grey, _) = p.add_swatch(COMPANY_GREY, "Color");
    p.rename_swatch(red, "Company red");
    p.rename_swatch(grey, "Company grey");
    let mut stop = ColorStop::new(0.0, COMPANY_RED);
    stop.swatch = Some(red);
    let mut source = Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(500.0, 3500.0), Size::new(400.0, 80.0), 0.0),
    );
    source.name = "Stripe".into();
    source.fill = Paint::Gradient(Gradient::new(
        GradientKind::Linear,
        &[stop, ColorStop::new(1.0, COMPANY_GREY)],
    ));
    source.opacity = 0.9;
    let source = p.add(source);
    let style = p.new_graphic_style(source, "Style").unwrap();
    let mut styled = Object::new(
        ObjectId(0),
        ShapeKind::Ellipse,
        Frame::new(Point::new(900.0, 3500.0), Size::new(80.0, 80.0), 0.0),
    );
    styled.name = "Styled".into();
    let styled = p.add(styled);
    p.apply_graphic_style(style, &[styled]);
    let mut lettering = Object::new(
        ObjectId(0),
        ShapeKind::Text,
        Frame::new(Point::new(1500.0, 3500.0), Size::new(600.0, 120.0), 0.0),
    );
    lettering.name = "Brand lettering".into();
    lettering.fill = Paint::Solid(COMPANY_GREY);
    lettering.fill_swatch = Some(grey);
    lettering.text = Some(TextBlock::new("ACE", CharStyle::default()));
    let lettering = p.add(lettering);
    p.new_text_style(lettering, "Text style").unwrap();
}

/// Makes `p` a fleet: the Volvo FH16 2012 with its main textures
/// "Globetrotter XL" (the main surface) and "Globetrotter" and its
/// accessories "Chassis" and "Mirrors", and the Krone Cool Liner trailer
/// with its main texture "Base".
fn add_vehicle(p: &mut Project) {
    use tp_core::{ProjectVehicle, Surface, SurfaceTemplate, TemplateStatus, TexturePart};
    let template =
        |p: &mut Project, key: (&str, &str, TexturePart), layout, opacity, visible, status| {
            let (asset, _) = p.add_asset(
                &format!("{} template", key.1),
                AssetKind::Raster,
                Arc::from(PNG),
                Size::new(64.0, 64.0),
            );
            SurfaceTemplate {
                package_id: key.0.into(),
                texture_id: key.1.into(),
                part: key.2,
                asset,
                layout_version: layout,
                opacity,
                visible,
                status,
            }
        };
    const VOLVO: &str = "scs.volvo.fh16_2012";
    const KRONE: &str = "scs.krone.cool_liner";
    use TexturePart::{Accessory, Main};
    p.surfaces[0].name = "Globetrotter XL".into();
    p.surfaces[0].template = Some(template(
        p,
        (VOLVO, "globetrotter_xl", Main),
        2,
        0.6,
        true,
        TemplateStatus::Current,
    ));
    let add = |p: &mut Project, name: &str, size, t| {
        let mut s = Surface::new(name, size);
        s.template = Some(t);
        p.surfaces.push(s);
    };
    let t = template(
        p,
        (VOLVO, "globetrotter", Main),
        2,
        0.8,
        true,
        TemplateStatus::Current,
    );
    add(p, "Globetrotter", 4096.0, t);
    let t = template(
        p,
        (VOLVO, "chassis", Accessory),
        1,
        0.35,
        false,
        TemplateStatus::Current,
    );
    add(p, "Chassis", 2048.0, t);
    let t = template(
        p,
        (VOLVO, "mirrors", Accessory),
        3,
        0.6,
        true,
        TemplateStatus::LayoutChanged,
    );
    add(p, "Mirrors", 1024.0, t);
    let t = template(
        p,
        (KRONE, "base", Main),
        1,
        0.6,
        true,
        TemplateStatus::Removed,
    );
    add(p, "Base", 4096.0, t);
    p.vehicles = vec![
        ProjectVehicle {
            package_id: VOLVO.into(),
            version: "1.3.0".into(),
            name: "Volvo FH16 2012".into(),
            brand: "Volvo".into(),
            kind: "truck".into(),
            game: "ets2".into(),
        },
        ProjectVehicle {
            package_id: KRONE.into(),
            version: "2.0.0".into(),
            name: "Krone Cool Liner".into(),
            brand: "Krone".into(),
            kind: "trailer".into(),
            game: "ets2".into(),
        },
    ];
}

/// A project with only `objects` on a fleet's first surface.
fn plain_project() -> Project {
    let mut p = Project::new("x", TextureResolution::R2048);
    add_vehicle(&mut p);
    p
}

/// A star, a curved path with a hole and an open line (format 2).
fn vector_objects() -> Vec<Object> {
    let mut star = Object::new(
        ObjectId(0),
        ShapeKind::Polygon {
            sides: 5,
            star: Some(0.45),
        },
        Frame::new(Point::new(600.0, 2600.0), Size::new(400.0, 380.0), 15.0),
    );
    star.name = "Star".into();
    let corner = |x: f64, y: f64| Node::corner(Point::new(x, y));
    let outer = Subpath::new(
        vec![
            Node::smooth(Point::new(1000.0, 3000.0), Point::new(1200.0, 2900.0)),
            corner(1600.0, 3000.0),
            Node {
                handle_in: Some(Point::new(1700.0, 3300.0)),
                ..corner(1600.0, 3400.0)
            },
            corner(1000.0, 3400.0),
        ],
        true,
    );
    let hole = Subpath::new(
        vec![
            corner(1200.0, 3100.0),
            corner(1200.0, 3300.0),
            corner(1400.0, 3300.0),
            corner(1400.0, 3100.0),
        ],
        true,
    );
    let mut swoosh = Object::from_path(ObjectId(0), PathData::new(vec![outer, hole]));
    swoosh.frame.rotation_deg = -20.0;
    swoosh.fill = Paint::Solid(Rgba::rgb(0x10, 0x60, 0xA0));
    let mut line = Object::from_path(
        ObjectId(0),
        PathData::new(vec![Subpath::new(
            vec![corner(2000.0, 3000.0), corner(2800.0, 3200.0)],
            false,
        )]),
    );
    line.name = "Line".into();
    line.fill = Paint::Solid(Rgba::rgb(255, 255, 255));
    line.edit_path(|p| {
        p.line_width = 30.0;
        p.line_style.dash = Some(Dash {
            dash: 0.0,
            gap: 45.0,
        });
    });
    vec![star, swoosh, line]
}

fn stops3() -> Vec<ColorStop> {
    vec![
        ColorStop::new(0.0, Rgba::rgb(0xF0, 0xB4, 0x4C)),
        ColorStop::new(0.4, Rgba::with_alpha(0x7A, 0x1F, 0x2B, 200)),
        ColorStop::new(1.0, Rgba::with_alpha(0x7A, 0x1F, 0x2B, 0)),
    ]
}

/// A rectangle with a linear fill, an ellipse with a skewed radial fill and
/// a line with a gradient outline (format 3).
fn gradient_objects() -> Vec<Object> {
    let frame = |x: f64| Frame::new(Point::new(x, 3800.0), Size::new(600.0, 200.0), 8.0);
    let mut fade = Object::new(ObjectId(0), ShapeKind::rectangle(), frame(500.0));
    fade.name = "Fade".into();
    let mut linear = Gradient::new(GradientKind::Linear, &stops3());
    linear.start = Point::new(0.1, 0.2);
    linear.end = Point::new(0.9, 0.7);
    fade.fill = Paint::Gradient(linear);
    let mut glow = Object::new(ObjectId(0), ShapeKind::Ellipse, frame(1500.0));
    glow.name = "Glow".into();
    let mut radial = Gradient::new(GradientKind::Radial, &stops3());
    radial.minor = Point::new(0.6, 0.9);
    glow.fill = Paint::Gradient(radial);
    let mut line = Object::from_path(
        ObjectId(0),
        PathData::new(vec![Subpath::new(
            vec![
                Node::corner(Point::new(2000.0, 3800.0)),
                Node::corner(Point::new(2600.0, 3900.0)),
            ],
            false,
        )]),
    );
    line.name = "Gradient line".into();
    line.stroke = Some(StrokeStyle {
        paint: Paint::Gradient(Gradient::new(GradientKind::Linear, &stops3())),
        width: 4.0,
        ..StrokeStyle::default()
    });
    vec![fade, glow, line]
}

fn assert_same_document(a: &Project, b: &Project) {
    assert_eq!(a.name, b.name);
    assert_eq!(a.resolution, b.resolution);
    assert_eq!(a.active_surface, b.active_surface);
    assert_eq!(a.palette, b.palette);
    assert_eq!(a.graphic_styles, b.graphic_styles);
    assert_eq!(a.text_styles, b.text_styles);
    assert_eq!(a.symbols, b.symbols);
    // Instance content is rebuilt on opening with new ids: compare it
    // without them.
    assert_eq!(without_content_ids(a), without_content_ids(b));
    assert_eq!(a.assets, b.assets);
    assert_eq!(a.vehicles, b.vehicles);
}

#[test]
fn document_conversion_round_trip_keeps_ids() {
    let p = rich_project();
    let file = tp_file::current::from_project(&p);
    assert_eq!(file.format, FORMAT_VERSION);
    let bytes: std::collections::HashMap<String, Vec<u8>> = p
        .assets
        .values()
        .map(|a| (tp_file::current::asset_entry(a), a.bytes.to_vec()))
        .collect();
    let mut back = tp_file::current::into_project(&file, |e| bytes.get(e).cloned()).unwrap();
    assert_same_document(&p, &back);
    let largest = p.assets.keys().map(|a| a.0).max().unwrap();
    assert!(back.next_object_id().0 > largest);
}

#[test]
fn file_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ace.truckpaint");
    let p = rich_project();
    tp_file::write(&p, &path).unwrap();
    let opened = tp_file::read(&path).unwrap();
    assert!(!opened.migrated);
    assert_same_document(&p, &opened.project);
    // No temporary file left behind.
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn writing_records_the_current_version() {
    let bytes = tp_file::to_bytes(&rich_project()).unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    assert_eq!(zip.by_index(0).unwrap().name(), "mimetype");
    let mut text = String::new();
    std::io::Read::read_to_string(&mut zip.by_name("project.ron").unwrap(), &mut text).unwrap();
    assert!(
        text.contains(&format!("format: {FORMAT_VERSION}")),
        "{text}"
    );
}

#[cfg(unix)]
#[test]
fn failed_write_keeps_the_previous_file() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ace.truckpaint");
    tp_file::write(&plain_project(), &path).unwrap();
    let before = std::fs::read(&path).unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
    let result = tp_file::write(&rich_project(), &path);
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(matches!(result, Err(Error::Io(_))));
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

/// ZIP bytes with the given document text and no assets.
fn zip_with(document: &str) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let o = zip::write::SimpleFileOptions::default();
    zip.start_file("mimetype", o).unwrap();
    zip.write_all(b"application/x-truckpaint").unwrap();
    zip.start_file("project.ron", o).unwrap();
    zip.write_all(document.as_bytes()).unwrap();
    zip.finish().unwrap().into_inner()
}

#[test]
fn newer_version_is_detected_from_the_header_alone() {
    // A future document whose body does not match today's structures.
    let doc = r#"(format: 7, name: "x", surfaces: {"main": [1, 2, 3]}, new_thing: Some(true))"#;
    let err = tp_file::from_bytes(&zip_with(doc)).unwrap_err();
    assert!(
        matches!(
            err,
            Error::NewerVersion {
                found: 7,
                supported: FORMAT_VERSION
            }
        ),
        "{err:?}"
    );
    assert_eq!(
        err.message("ace.truckpaint"),
        "ace.truckpaint was created with a newer version of TruckPaint."
    );
}

#[test]
fn not_a_project_and_damaged_files() {
    assert!(matches!(
        tp_file::from_bytes(b"hello"),
        Err(Error::NotAProject)
    ));
    let mut truncated = tp_file::to_bytes(&rich_project()).unwrap();
    truncated.truncate(truncated.len() / 2);
    let err = tp_file::from_bytes(&truncated).unwrap_err();
    assert!(matches!(err, Error::Damaged(_)), "{err:?}");
    assert_eq!(
        err.message("a.truckpaint"),
        "a.truckpaint is damaged and cannot be opened."
    );
    assert!(matches!(
        tp_file::from_bytes(&zip_with("(format: 1, name: ")),
        Err(Error::Damaged(_))
    ));
}

#[test]
fn image_with_missing_asset_is_damaged() {
    let p = rich_project();
    let mut file = tp_file::current::from_project(&p);
    file.assets.clear();
    let text = ron::ser::to_string(&file).unwrap();
    let err = tp_file::from_bytes(&zip_with(&text)).unwrap_err();
    assert!(matches!(err, Error::Damaged(_)), "{err:?}");
}

fn fixture(version: u32) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("tests/fixtures/v{version}.truckpaint"))
}

/// Writes the fixture of the current version. Run once when a new format
/// version is introduced; fixtures of released versions never change.
#[test]
#[ignore = "regenerates a committed fixture"]
fn write_current_fixture() {
    tp_file::write(&rich_project(), &fixture(FORMAT_VERSION)).unwrap();
}

#[test]
fn every_stroke_option_round_trips() {
    let mut p = plain_project();
    let caps = [Cap::Butt, Cap::Round, Cap::Square];
    let joins = [Join::Miter, Join::Round, Join::Bevel];
    let aligns = [
        StrokeAlign::Center,
        StrokeAlign::Inside,
        StrokeAlign::Outside,
    ];
    for i in 0..3 {
        let line = LineStyle {
            dash: (i > 0).then_some(Dash {
                dash: 4.0 * i as f64,
                gap: 2.5,
            }),
            cap: caps[i],
            join: joins[i],
            miter_limit: 2.0 + i as f64,
        };
        let mut o = Object::new(
            ObjectId(0),
            ShapeKind::rectangle(),
            Frame::new(Point::new(100.0, 100.0), Size::new(50.0, 50.0), 0.0),
        );
        o.stroke = Some(StrokeStyle {
            paint: Paint::Solid(Rgba::rgb(1, 2, 3)),
            width: 5.0,
            align: aligns[i],
            line,
            swatch: None,
        });
        p.add(o);
        let mut l = Object::from_path(
            ObjectId(0),
            PathData::new(vec![Subpath::new(
                vec![
                    Node::corner(Point::new(0.0, 0.0)),
                    Node::corner(Point::new(10.0, 0.0)),
                ],
                false,
            )]),
        );
        l.edit_path(|d| d.line_style = line);
        p.add(l);
    }
    let opened = tp_file::from_bytes(&tp_file::to_bytes(&p).unwrap()).unwrap();
    assert_eq!(opened.project.surfaces, p.surfaces);
}

#[test]
fn newer_and_development_formats() {
    let err = tp_file::from_bytes(&zip_with("(format: 4, name: \"x\")")).unwrap_err();
    assert!(
        matches!(
            err,
            Error::NewerVersion {
                found: 4,
                supported: 1
            }
        ),
        "{err:?}"
    );
    for dev in [2, 3] {
        let doc = format!("(format: {dev}, name: \"x\")");
        let err = tp_file::from_bytes(&zip_with(&doc)).unwrap_err();
        assert!(
            matches!(err, Error::Unsupported { found } if found == dev),
            "{err:?}"
        );
        assert!(err.message("a.truckpaint").contains("development format"));
    }
}

#[test]
fn v1_fixture_opens() {
    let opened = tp_file::read(&fixture(1)).unwrap();
    let p = &opened.project;
    let names: Vec<&str> = p.surfaces.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "Globetrotter XL",
            "Globetrotter",
            "Chassis",
            "Mirrors",
            "Base"
        ]
    );
    assert_eq!(p.surfaces[3].size, 1024.0);
    let chassis = p.surfaces[2].template.as_ref().unwrap();
    assert_eq!((chassis.opacity, chassis.visible), (0.35, false));
    let parts: Vec<tp_core::TexturePart> = p
        .surfaces
        .iter()
        .map(|s| s.template.as_ref().unwrap().part)
        .collect();
    use tp_core::TexturePart::{Accessory, Main};
    assert_eq!(parts, [Main, Main, Accessory, Accessory, Main]);
    assert_eq!(
        p.surfaces[3].template.as_ref().unwrap().status,
        tp_core::TemplateStatus::LayoutChanged
    );
    assert_eq!(p.vehicles.len(), 2);
    assert_eq!(p.vehicles[0].version, "1.3.0");
    assert_eq!(p.vehicle_range("scs.volvo.fh16_2012"), 0..4);
    assert_eq!(p.vehicle_range("scs.krone.cool_liner"), 4..5);
    assert!(!opened.migrated);
    assert_same_document(&opened.project, &rich_project());
    let objects = &opened.project.surface().objects;
    // The vector and gradient objects, the brand kit's three, then the
    // instance of "Logo".
    assert_eq!(objects.len(), 14);
    assert!(objects[13].is_instance());
    assert_eq!(opened.project.symbols[0].name, "Logo");
    let fade = objects[7].fill.gradient().unwrap();
    assert_eq!(fade.kind, GradientKind::Linear);
    assert_eq!(fade.stops(), stops3().as_slice());
    assert_eq!(fade.end, Point::new(0.9, 0.7));
    assert_eq!(
        objects[8].fill.gradient().unwrap().minor,
        Point::new(0.6, 0.9)
    );
    assert!(objects[9].stroke.unwrap().paint.gradient().is_some());
}

#[test]
fn gradients_round_trip() {
    let mut p = plain_project();
    for o in gradient_objects() {
        p.add(o);
    }
    let opened = tp_file::from_bytes(&tp_file::to_bytes(&p).unwrap()).unwrap();
    assert_eq!(opened.project.surfaces, p.surfaces);
}

#[test]
fn gradient_with_one_stop_is_damaged() {
    let doc = r#"(format: 1, name: "x", resolution: 2048, active_surface: 0,
        surfaces: [(name: "Main texture", size: 2048.0, objects: [
            (id: 1, name: "Box", kind: Rectangle(corner_radius: 0.0), center: (100.0, 100.0),
             size: (50.0, 50.0), rotation: 0.0,
             fill: Linear(start: (0.0, 0.5), end: (1.0, 0.5),
                          stops: [(offset: 0.0, color: (255, 0, 0, 255))]),
             opacity: 1.0, visible: true, locked: false),
        ])],
        vehicles: [(package_id: "a.b", version: "1.0.0", name: "A", brand: "B",
                    kind: "truck", game: "ets2")])"#;
    let err = tp_file::from_bytes(&zip_with(doc)).unwrap_err();
    assert!(matches!(err, Error::Damaged(_)), "{err:?}");
}

/// Files of development builds that recorded variants: a single vehicle
/// and variant, then fleets with variants.
#[test]
fn variant_files_of_development_builds_are_refused() {
    for name in ["dev-single-vehicle", "dev-variants"] {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/fixtures/{name}.truckpaint"));
        let err = tp_file::read(&path).unwrap_err();
        assert!(
            matches!(err, Error::Unsupported { found: 1 }),
            "{name}: {err:?}"
        );
        assert!(err.message("a.truckpaint").contains("development format"));
    }
}

#[test]
fn blank_texture_files_are_a_development_format() {
    let doc = r#"(format: 1, name: "x", resolution: 2048, active_surface: 0,
        surfaces: [(name: "Main texture", size: 2048.0)])"#;
    let err = tp_file::from_bytes(&zip_with(doc)).unwrap_err();
    assert!(matches!(err, Error::Unsupported { found: 1 }), "{err:?}");
    assert!(err.message("a.truckpaint").contains("development format"));
}

#[test]
fn inconsistent_fleets_are_damaged() {
    let p = rich_project();
    let damaged = |edit: &dyn Fn(&mut tp_file::current::FileProject)| {
        let mut file = tp_file::current::from_project(&p);
        edit(&mut file);
        let text = ron::ser::to_string(&file).unwrap();
        // Asset entries are missing too, but the fleet is checked on the
        // document: build a full ZIP from the real project instead.
        let bytes = tp_file::to_bytes(&p).unwrap();
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        let mut out = Vec::new();
        {
            let mut w = zip::ZipWriter::new(std::io::Cursor::new(&mut out));
            for i in 0..zip.len() {
                let mut entry = zip.by_index(i).unwrap();
                let name = entry.name().to_owned();
                let mut data = Vec::new();
                std::io::Read::read_to_end(&mut entry, &mut data).unwrap();
                if name == "project.ron" {
                    data = text.clone().into_bytes();
                }
                w.start_file(name, zip::write::SimpleFileOptions::default())
                    .unwrap();
                w.write_all(&data).unwrap();
            }
            w.finish().unwrap();
        }
        tp_file::from_bytes(&out).unwrap_err()
    };
    let mixed = damaged(&|f| f.vehicles[1].game = "ats".into());
    assert!(
        matches!(mixed, Error::Damaged(ref why) if why.contains("different games")),
        "{mixed:?}"
    );
    let unknown = damaged(&|f| {
        f.surfaces[1].template.as_mut().unwrap().package_id = "x.y".into();
    });
    assert!(matches!(unknown, Error::Damaged(_)), "{unknown:?}");
    let painted_twice = damaged(&|f| {
        f.surfaces[1].template.as_mut().unwrap().texture_id = "globetrotter_xl".into();
    });
    assert!(
        matches!(painted_twice, Error::Damaged(ref why) if why.contains("painted twice")),
        "{painted_twice:?}"
    );
    let nothing_painted = damaged(&|f| {
        f.surfaces[4].template.as_mut().unwrap().package_id = "scs.volvo.fh16_2012".into();
        f.surfaces[4].template.as_mut().unwrap().texture_id = "base".into();
    });
    assert!(
        matches!(nothing_painted, Error::Damaged(ref why) if why.contains("paints no texture")),
        "{nothing_painted:?}"
    );
    let twice = damaged(&|f| {
        let v = f.vehicles[0].clone();
        f.vehicles.push(v);
    });
    assert!(matches!(twice, Error::Damaged(_)), "{twice:?}");
}

/// The object named `name` on the main surface.
fn named(p: &Project, name: &str) -> Object {
    fn find(list: &[Arc<Object>], name: &str) -> Option<Object> {
        list.iter().find_map(|o| {
            if o.name == name {
                Some((**o).clone())
            } else {
                find(&o.children, name)
            }
        })
    }
    find(&p.surfaces[0].objects, name).expect("object")
}

#[test]
fn brand_kit_round_trip() {
    let p = rich_project();
    let mut opened = tp_file::from_bytes(&tp_file::to_bytes(&p).unwrap())
        .unwrap()
        .project;
    assert_same_document(&p, &opened);
    let names: Vec<&str> = opened.palette.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["Color 1", "Company red", "Company grey"]);
    let style = opened.graphic_styles[0].id;
    assert_eq!(named(&opened, "Styled").style, Some(style));
    let lettering = named(&opened, "Brand lettering");
    assert!(lettering.fill_swatch.is_some());
    assert_eq!(
        lettering.text.unwrap().style_id,
        Some(opened.text_styles[0].id)
    );
    // Editing "Company red" still recolors the style and its users.
    let red = opened.palette[1].id;
    let dark = Rgba::rgb(0x8B, 0, 0);
    opened.set_swatch_color(red, dark);
    opened.relink();
    let stop = |paint: &Paint| paint.gradient().unwrap().stops()[0].color;
    assert_eq!(stop(&opened.graphic_styles[0].look.fill), dark);
    let styled = named(&opened, "Styled");
    assert_eq!(stop(&styled.fill), dark);
    assert_eq!(styled.style, Some(style));
}

#[test]
fn inconsistent_links_are_dropped_when_opening() {
    let p = rich_project();
    let mut file = tp_file::current::from_project(&p);
    // A hand-edited swatch color no longer matches the linked colors.
    let grey = file
        .swatches
        .iter_mut()
        .find(|s| s.name == "Company grey")
        .unwrap();
    grey.color = [1, 2, 3, 255];
    let bytes: std::collections::HashMap<String, Vec<u8>> = p
        .assets
        .values()
        .map(|a| (tp_file::current::asset_entry(a), a.bytes.to_vec()))
        .collect();
    let back = tp_file::current::into_project(&file, |e| bytes.get(e).cloned()).unwrap();
    let lettering = named(&back, "Brand lettering");
    assert_eq!(lettering.fill_swatch, None);
    assert_eq!(
        lettering.fill,
        Paint::Solid(COMPANY_GREY),
        "the look is kept"
    );
}

#[test]
fn palette_of_an_earlier_file() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/v1-palette-colors.truckpaint");
    let p = tp_file::read(&path).unwrap().project;
    let swatches: Vec<(&str, Rgba)> = p
        .palette
        .iter()
        .map(|s| (s.name.as_str(), s.color))
        .collect();
    assert_eq!(swatches, [("Color 1", Rgba::rgb(0xF0, 0xB4, 0x4C))]);
    assert!(p.graphic_styles.is_empty() && p.text_styles.is_empty());
}

#[test]
fn text_style_without_its_look_opens_with_the_default_look() {
    let p = rich_project();
    let mut file = tp_file::current::from_project(&p);
    let style = &mut file.text_styles[0];
    style.fill = None;
    style.fill_swatch = None;
    style.stroke = None;
    style.opacity = None;
    let bytes: std::collections::HashMap<String, Vec<u8>> = p
        .assets
        .values()
        .map(|a| (tp_file::current::asset_entry(a), a.bytes.to_vec()))
        .collect();
    let back = tp_file::current::into_project(&file, |e| bytes.get(e).cloned()).unwrap();
    let look = back.text_styles[0].look;
    assert_eq!(look.fill, Paint::Solid(tp_core::document::DEFAULT_FILL));
    assert_eq!((look.stroke, look.opacity), (None, 1.0));
    // The grey lettering no longer matches it.
    assert_eq!(named(&back, "Brand lettering").text.unwrap().style_id, None);
}

#[test]
fn symbols_round_trip() {
    let p = rich_project();
    let mut opened = tp_file::from_bytes(&tp_file::to_bytes(&p).unwrap())
        .unwrap()
        .project;
    assert_same_document(&p, &opened);
    let logo = opened.symbols[0].id;
    assert_eq!(opened.symbols[0].name, "Logo");
    assert_eq!(opened.instance_count(logo), 2);
    // Both instances follow an edit of "Logo".
    opened.editing_symbol = Some(logo);
    let mut circle = (*opened.surface().objects[0]).clone();
    circle.fill = Paint::Solid(Rgba::rgb(9, 9, 9));
    opened.surface_mut().replace(&[circle]);
    opened.editing_symbol = None;
    opened.refresh_instances();
    let filled = |s: &tp_core::Surface| {
        s.objects
            .iter()
            .filter(|o| o.is_instance())
            .all(|o| o.children[0].fill == Paint::Solid(Rgba::rgb(9, 9, 9)))
    };
    assert!(filled(&opened.surfaces[0]) && filled(&opened.surfaces[1]));
    let mirrored = opened.surfaces[1]
        .objects
        .iter()
        .find_map(|o| match o.kind {
            ShapeKind::Instance { placement, .. } => Some(placement),
            _ => None,
        })
        .unwrap();
    assert!(mirrored.determinant() < 0.0);
}

#[test]
fn files_without_symbols_have_no_symbols_field() {
    let bytes = tp_file::to_bytes(&plain_project()).unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut doc = String::new();
    std::io::Read::read_to_string(&mut zip.by_name("project.ron").unwrap(), &mut doc).unwrap();
    assert!(!doc.contains("symbols"), "{doc}");
}
