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
    add_vehicle(&mut p);
    p
}

/// Makes `p` a three-texture vehicle project: Cabin (the main surface),
/// Chassis and Accessories, each with a template.
fn add_vehicle(p: &mut Project) {
    use tp_core::{Surface, SurfaceTemplate, TemplateStatus, VehicleRef};
    let template = |p: &mut Project, texture: &str, layout, opacity, visible, status| {
        let (asset, _) = p.add_asset(
            &format!("{texture} template"),
            AssetKind::Raster,
            Arc::from(PNG),
            Size::new(64.0, 64.0),
        );
        SurfaceTemplate {
            texture_id: texture.into(),
            asset,
            layout_version: layout,
            opacity,
            visible,
            status,
        }
    };
    p.surfaces[0].name = "Cabin".into();
    p.surfaces[0].template = Some(template(p, "cabin", 2, 0.6, true, TemplateStatus::Current));
    let mut chassis = Surface::new("Chassis", 2048.0);
    chassis.template = Some(template(
        p,
        "chassis",
        1,
        0.35,
        false,
        TemplateStatus::Current,
    ));
    let mut accessories = Surface::new("Accessories", 1024.0);
    accessories.template = Some(template(
        p,
        "accessories",
        3,
        0.6,
        true,
        TemplateStatus::LayoutChanged,
    ));
    p.surfaces.push(chassis);
    p.surfaces.push(accessories);
    p.vehicle = Some(VehicleRef {
        package_id: "scs.volvo.fh16_2012".into(),
        version: "1.3.0".into(),
        variant_id: "globetrotter_xl".into(),
        name: "Volvo FH16 2012".into(),
        brand: "Volvo".into(),
        kind: "truck".into(),
        game: "ets2".into(),
    });
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
    assert_eq!(a.surfaces, b.surfaces);
    assert_eq!(a.assets, b.assets);
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
    tp_file::write(&Project::new("old", TextureResolution::R2048), &path).unwrap();
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
    let mut p = Project::new("x", TextureResolution::R2048);
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
    assert_eq!(names, ["Cabin", "Chassis", "Accessories"]);
    assert_eq!(p.surfaces[2].size, 1024.0);
    let chassis = p.surfaces[1].template.as_ref().unwrap();
    assert_eq!((chassis.opacity, chassis.visible), (0.35, false));
    assert_eq!(
        p.surfaces[2].template.as_ref().unwrap().status,
        tp_core::TemplateStatus::LayoutChanged
    );
    assert_eq!(p.vehicle.as_ref().unwrap().version, "1.3.0");
    assert!(!opened.migrated);
    assert_same_document(&opened.project, &rich_project());
    let objects = &opened.project.surface().objects;
    assert_eq!(objects.len(), 10);
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
    let mut p = Project::new("x", TextureResolution::R2048);
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
        ])])"#;
    let err = tp_file::from_bytes(&zip_with(doc)).unwrap_err();
    assert!(matches!(err, Error::Damaged(_)), "{err:?}");
}
