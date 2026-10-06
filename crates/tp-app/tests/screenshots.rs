//! Renders key screens to PNG for visual QA.
//!
//! Ignored by default (needs a GPU). Run with:
//! `cargo test -p tp-app --test screenshots -- --ignored`
//! Images are written to `target/screenshots/`.

mod common;

use std::path::PathBuf;

use egui::Vec2;
use egui_kittest::kittest::Queryable;
use tp_app::layout::ViewMode;
use tp_app::prefs::{Prefs, RecentProject};

fn out_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/screenshots");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn save(harness: &mut egui_kittest::Harness<'static, tp_app::AppState>, name: &str) {
    harness.run();
    let image = harness.render().expect("render");
    image.save(out_dir().join(format!("{name}.png"))).unwrap();
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_screens() {
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let mut prefs = Prefs::default();
        prefs.ui_scale = scale;
        prefs.recent = vec![
            RecentProject {
                name: "ACE Logistics".into(),
                path: "/home/user/Liveries/ace.truckpaint".into(),
                last_opened: tp_app::prefs::now_unix() - 7_200,
            },
            RecentProject {
                name: "Old fleet".into(),
                path: "/missing/old-fleet.truckpaint".into(),
                last_opened: 0,
            },
        ];
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs.clone(), size);
        save(&mut h, &format!("home_{suffix}"));

        h.get_by_label("New Project").click();
        h.run();
        save(&mut h, &format!("new_project_{suffix}"));

        h.get_by_label("Create").click();
        save(&mut h, &format!("workspace_{suffix}"));

        h.state_mut().prefs.layout.view_mode = ViewMode::Split;
        save(&mut h, &format!("workspace_split_{suffix}"));
    }

    let mut h = common::wgpu_harness_with(Prefs::default(), Vec2::new(1440.0, 900.0));
    common::create_project(&mut h);
    h.get_by_label("Edit").click();
    h.run();
    save(&mut h, "menu_edit");
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_gallery_in_color_and_grayscale() {
    let mut h = common::wgpu_harness_with(Prefs::default(), Vec2::new(900.0, 900.0));
    h.run();
    h.state_mut().show_gallery = true;
    h.run();
    let mut image = h.render().expect("render");
    image.save(out_dir().join("gallery.png")).unwrap();
    for pixel in image.pixels_mut() {
        let [r, g, b, a] = pixel.0;
        let luma = (0.2126 * f32::from(r) + 0.7152 * f32::from(g) + 0.0722 * f32::from(b)) as u8;
        pixel.0 = [luma, luma, luma, a];
    }
    image.save(out_dir().join("gallery_grayscale.png")).unwrap();
}

/// Adds a varied scene to the open project and returns the ids.
fn demo_scene(
    h: &mut egui_kittest::Harness<'static, tp_app::AppState>,
) -> Vec<tp_core::document::ObjectId> {
    use tp_core::document::{Frame, Object, ObjectId, Rgba, ShapeKind, StrokeStyle};
    use tp_core::kurbo::{Point, Size};
    let ws = h.state_mut().workspace_mut().expect("project open");
    let mut ids = Vec::new();
    let mut add = |kind,
                   center: (f64, f64),
                   size: (f64, f64),
                   rot,
                   fill: Rgba,
                   stroke: Option<StrokeStyle>,
                   opacity| {
        let mut o = Object::new(
            ObjectId(0),
            kind,
            Frame::new(
                Point::new(center.0, center.1),
                Size::new(size.0, size.1),
                rot,
            ),
        );
        o.fill = fill;
        o.stroke = stroke;
        o.opacity = opacity;
        ids.push(ws.project.add(o));
    };
    add(
        ShapeKind::rectangle(),
        (2048.0, 900.0),
        (3800.0, 900.0),
        0.0,
        Rgba::rgb(0x7A, 0x1F, 0x2B),
        None,
        1.0,
    );
    add(
        ShapeKind::Rectangle {
            corner_radius: 120.0,
        },
        (1300.0, 2300.0),
        (1600.0, 900.0),
        -12.0,
        Rgba::rgb(0xF2, 0xF2, 0xF2),
        Some(StrokeStyle {
            color: Rgba::rgb(0x20, 0x20, 0x20),
            width: 24.0,
        }),
        1.0,
    );
    add(
        ShapeKind::Ellipse,
        (2900.0, 2600.0),
        (1400.0, 1400.0),
        0.0,
        Rgba::rgb(0xF0, 0xB4, 0x4C),
        None,
        0.7,
    );
    add(
        ShapeKind::rectangle(),
        (2900.0, 3500.0),
        (1800.0, 160.0),
        25.0,
        Rgba::rgb(0x30, 0x30, 0x36),
        None,
        1.0,
    );
    ids
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_canvas_scene() {
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        common::create_project(&mut h);
        let ids = demo_scene(&mut h);
        h.state_mut().workspace_mut().unwrap().selection = vec![ids[1]];
        save(&mut h, &format!("canvas_single_{suffix}"));
        h.state_mut().workspace_mut().unwrap().selection = vec![ids[2], ids[3]];
        save(&mut h, &format!("canvas_multi_{suffix}"));
    }
    // Zoomed in: overlay keeps its size.
    let mut h = common::wgpu_harness_with(Prefs::default(), Vec2::new(1440.0, 900.0));
    common::create_project(&mut h);
    let ids = demo_scene(&mut h);
    {
        let ws = h.state_mut().workspace_mut().unwrap();
        ws.selection = vec![ids[1]];
        let view = ws.viewport.as_mut().unwrap();
        view.center = tp_core::kurbo::Point::new(1300.0, 2300.0);
        view.set_zoom_centered(0.25);
    }
    save(&mut h, "canvas_zoom_25");
    {
        let view = h
            .state_mut()
            .workspace_mut()
            .unwrap()
            .viewport
            .as_mut()
            .unwrap();
        view.set_zoom_centered(4.0);
        view.center = tp_core::kurbo::Point::new(540.0, 2300.0);
    }
    save(&mut h, "canvas_zoom_400");
}

/// A small livery tree: Background, Graphics (3 stripes), Branding (logo).
fn livery_tree(
    h: &mut egui_kittest::Harness<'static, tp_app::AppState>,
) -> Vec<tp_core::document::ObjectId> {
    use tp_core::document::{Frame, Object, ObjectId, Rgba, ShapeKind, StrokeStyle};
    use tp_core::kurbo::{Point, Size};
    let ws = h.state_mut().workspace_mut().expect("project open");
    let shape = |kind, name: &str, c: (f64, f64), s: (f64, f64), fill: Rgba| {
        let mut o = Object::new(
            ObjectId(0),
            kind,
            Frame::new(Point::new(c.0, c.1), Size::new(s.0, s.1), 0.0),
        );
        o.name = name.into();
        o.fill = fill;
        o
    };
    let bg = ws.project.add(shape(
        ShapeKind::rectangle(),
        "Background",
        (2048.0, 2048.0),
        (4096.0, 4096.0),
        Rgba::rgb(0xF2, 0xF2, 0xF2),
    ));
    let base = ws.project.add(shape(
        ShapeKind::rectangle(),
        "Burgundy base",
        (2048.0, 1500.0),
        (4096.0, 1200.0),
        Rgba::rgb(0x7A, 0x1F, 0x2B),
    ));
    let white = ws.project.add(shape(
        ShapeKind::rectangle(),
        "White stripe",
        (2048.0, 2200.0),
        (4096.0, 160.0),
        Rgba::rgb(0xFF, 0xFF, 0xFF),
    ));
    let grey = ws.project.add(shape(
        ShapeKind::rectangle(),
        "Grey stripe",
        (2048.0, 2400.0),
        (4096.0, 120.0),
        Rgba::rgb(0x70, 0x70, 0x78),
    ));
    let graphics = ws.project.group(&[base, white, grey]).unwrap();
    let mut logo = shape(
        ShapeKind::Ellipse,
        "Logo",
        (1200.0, 3200.0),
        (900.0, 900.0),
        Rgba::rgb(0xF0, 0xB4, 0x4C),
    );
    logo.stroke = Some(StrokeStyle {
        color: Rgba::rgb(0x20, 0x20, 0x20),
        width: 24.0,
    });
    let logo = ws.project.add(logo);
    let mut name = shape(
        ShapeKind::Rectangle {
            corner_radius: 40.0,
        },
        "Company name",
        (2700.0, 3200.0),
        (1600.0, 300.0),
        Rgba::rgb(0x30, 0x30, 0x36),
    );
    name.opacity = 0.8;
    let name = ws.project.add(name);
    let branding = ws.project.group(&[logo, name]).unwrap();
    for (id, label) in [(graphics, "Graphics"), (branding, "Branding")] {
        let mut g = (**ws.project.surface().get(id).unwrap()).clone();
        g.name = label.into();
        ws.project.surface_mut().replace(&[g]);
    }
    ws.panels.expanded.extend([graphics, branding]);
    vec![bg, graphics, branding, logo]
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_editing_panels() {
    use tp_app::layout::PanelKind;
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let mut prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        for slot in &mut prefs.layout.panels {
            slot.collapsed = false;
            slot.open = !matches!(
                slot.kind,
                PanelKind::Assets | PanelKind::Vehicle | PanelKind::Colors | PanelKind::Stroke
            );
        }
        prefs.layout.column_width = 300.0;
        prefs.recent_colors = vec![
            [0x7A, 0x1F, 0x2B, 255],
            [255, 255, 255, 255],
            [0xF0, 0xB4, 0x4C, 255],
        ];
        let size = Vec2::new(1440.0, 1000.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs.clone(), size);
        common::create_project(&mut h);
        let ids = livery_tree(&mut h);
        h.state_mut().workspace_mut().unwrap().selection = vec![ids[3]];
        save(&mut h, &format!("panels_layers_transform_{suffix}"));

        for slot in &mut h.state_mut().prefs.layout.panels {
            slot.open = matches!(slot.kind, PanelKind::Colors | PanelKind::Stroke);
        }
        save(&mut h, &format!("panels_colors_stroke_{suffix}"));
    }
}

/// A text object laid out with the workspace's text engine.
fn add_text(
    ws: &mut tp_app::workspace::Workspace,
    content: &str,
    style: tp_core::document::CharStyle,
    anchor: (f64, f64),
    fill: tp_core::document::Rgba,
    stroke: Option<tp_core::document::StrokeStyle>,
) -> tp_core::document::ObjectId {
    use tp_core::document::{Object, ObjectId, TextBlock};
    let anchor = tp_core::kurbo::Point::new(anchor.0, anchor.1);
    let mut o = Object::text(ObjectId(0), TextBlock::new(content, style), anchor);
    o.fill = fill;
    o.stroke = stroke;
    ws.text.place_at(&mut o, anchor);
    ws.project.add(o)
}

/// A livery with lettering, a PNG logo and an SVG badge.
fn lettering_scene(
    h: &mut egui_kittest::Harness<'static, tp_app::AppState>,
) -> Vec<tp_core::document::ObjectId> {
    use tp_core::document::TextAlign;
    use tp_core::document::{CharStyle, Frame, Object, ObjectId, Rgba, ShapeKind, StrokeStyle};
    use tp_core::kurbo::{Point, Size};
    let ws = h.state_mut().workspace_mut().expect("project open");
    let mut base = Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(2048.0, 1700.0), Size::new(4096.0, 1800.0), 0.0),
    );
    base.fill = Rgba::rgb(0x7A, 0x1F, 0x2B);
    ws.project.add(base);
    let white = Rgba::rgb(255, 255, 255);
    let black = Rgba::rgb(0x15, 0x15, 0x18);
    let title = add_text(
        ws,
        "ACE LOGISTICS",
        CharStyle {
            family: "Barlow Condensed".into(),
            weight: 700,
            italic: true,
            size: 420.0,
            ..CharStyle::default()
        },
        (300.0, 1500.0),
        white,
        Some(StrokeStyle {
            color: black,
            width: 14.0,
        }),
    );
    let phone = add_text(
        ws,
        "+33 1 23 45 67 89",
        CharStyle {
            family: "Oswald".into(),
            weight: 500,
            size: 160.0,
            letter_spacing: 80.0,
            ..CharStyle::default()
        },
        (320.0, 1850.0),
        Rgba::rgb(0xF0, 0xB4, 0x4C),
        None,
    );
    let slogan = add_text(
        ws,
        "EUROPE\nWIDE",
        CharStyle {
            family: "Bebas Neue".into(),
            weight: 400,
            size: 300.0,
            align: TextAlign::Center,
            line_height: 90.0,
            ..CharStyle::default()
        },
        (3300.0, 3000.0),
        Rgba::rgb(0x30, 0x30, 0x36),
        None,
    );
    let rotated = add_text(
        ws,
        "Montserrat",
        CharStyle {
            family: "Montserrat".into(),
            weight: 800,
            size: 220.0,
            ..CharStyle::default()
        },
        (500.0, 3300.0),
        Rgba::rgb(0x1F, 0x6F, 0xFF),
        None,
    );
    {
        let mut o = (**ws.project.surface().get(rotated).unwrap()).clone();
        o.frame.rotation_deg = -15.0;
        o.frame.size.width *= 1.4;
        o.sync_text_scale();
        ws.project.surface_mut().replace(&[o]);
    }
    // A PNG logo (radial gradient) and an SVG badge.
    let png = {
        let img = image::RgbaImage::from_fn(512, 512, |x, y| {
            let (dx, dy) = (x as f32 - 256.0, y as f32 - 256.0);
            let d = (dx * dx + dy * dy).sqrt() / 256.0;
            if d > 1.0 {
                image::Rgba([0, 0, 0, 0])
            } else {
                image::Rgba([20, (60.0 + 150.0 * d) as u8, 220, 255])
            }
        });
        let mut out = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    };
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="600" height="300" viewBox="0 0 600 300">
  <rect x="10" y="10" width="580" height="280" rx="60" fill="#F0B44C" stroke="#151518" stroke-width="16"/>
  <path d="M80 220 L180 80 L280 220 Z" fill="#7A1F2B"/>
  <circle cx="420" cy="150" r="90" fill="none" stroke="#151518" stroke-width="24"/>
</svg>"##;
    let files = vec![
        tp_app::import::read_bytes("logo.png", png),
        tp_app::import::read_bytes("badge.svg", svg.to_vec()),
    ];
    ws.place_files(files, Some(Point::new(3000.0, 1300.0)), 0.0);
    let images = ws.selection.clone();
    {
        let mut badge = ws.selected_objects()[1].clone();
        badge.frame.center = Point::new(1900.0, 3000.0);
        badge.frame.size = Size::new(1200.0, 600.0);
        badge.frame.rotation_deg = 8.0;
        ws.project.surface_mut().replace(&[badge]);
    }
    ws.selection.clear();
    let mut ids = vec![title, phone, slogan, rotated];
    ids.extend(images);
    ids
}

/// Steps until SVG renders from the background thread are uploaded.
fn wait_for_images(h: &mut egui_kittest::Harness<'static, tp_app::AppState>) {
    for _ in 0..200 {
        h.step();
        if !h.state().workspace().unwrap().images.is_rendering() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    h.step();
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_text_and_images() {
    use tp_app::layout::PanelKind;
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let mut prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        for slot in &mut prefs.layout.panels {
            slot.collapsed = false;
            slot.open = matches!(
                slot.kind,
                PanelKind::Properties | PanelKind::Layers | PanelKind::Assets
            );
        }
        prefs.layout.column_width = 300.0;
        let size = Vec2::new(1440.0, 1000.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        common::create_project(&mut h);
        let ids = lettering_scene(&mut h);
        wait_for_images(&mut h);
        h.state_mut().workspace_mut().unwrap().selection = vec![ids[0]];
        save(&mut h, &format!("text_livery_{suffix}"));

        // Editing on the canvas, with part of the text selected.
        {
            let ws = h.state_mut().workspace_mut().unwrap();
            ws.start_editing(ids[0], None, 0.0);
            ws.text_move(tp_text::Motion::WordLeft, true, 0.0);
        }
        save(&mut h, &format!("text_editing_{suffix}"));
        h.state_mut().workspace_mut().unwrap().end_text_session(0.0);

        // Font picker.
        h.get_by_label_contains("Font family").click();
        h.run();
        save(&mut h, &format!("font_picker_{suffix}"));
        h.key_press(egui::Key::Escape);
        h.run();

        // Image selected: image information, no fill/stroke.
        h.state_mut().workspace_mut().unwrap().selection = vec![ids[5]];
        save(&mut h, &format!("image_selected_{suffix}"));
    }
    // Zoomed out and in on the images.
    let mut h = common::wgpu_harness_with(Prefs::default(), Vec2::new(1440.0, 900.0));
    common::create_project(&mut h);
    lettering_scene(&mut h);
    for (zoom, center, name) in [
        (0.25, (2048.0, 2048.0), "images_zoom_25"),
        (8.0, (1700.0, 2900.0), "images_zoom_800_svg"),
        (8.0, (3000.0, 1300.0), "images_zoom_800_png"),
        (8.0, (700.0, 1450.0), "text_zoom_800"),
    ] {
        {
            let view = h
                .state_mut()
                .workspace_mut()
                .unwrap()
                .viewport
                .as_mut()
                .unwrap();
            view.center = tp_core::kurbo::Point::new(center.0, center.1);
            view.set_zoom_centered(zoom);
        }
        h.step();
        wait_for_images(&mut h);
        save(&mut h, name);
    }
}
