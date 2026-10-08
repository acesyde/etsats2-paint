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
        h.get_by_role_and_label(egui::accesskit::Role::RadioButton, common::SAMPLE)
            .click();
        h.run();
        save(&mut h, &format!("new_project_{suffix}"));

        h.get_by_label("Next").click();
        h.run();
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
        o.fill = fill.into();
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
            paint: Rgba::rgb(0x20, 0x20, 0x20).into(),
            width: 24.0,
            ..Default::default()
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
        o.fill = fill.into();
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
        paint: Rgba::rgb(0x20, 0x20, 0x20).into(),
        width: 24.0,
        ..Default::default()
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
                PanelKind::Assets | PanelKind::Colors | PanelKind::Stroke
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
    o.fill = fill.into();
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
    base.fill = Rgba::rgb(0x7A, 0x1F, 0x2B).into();
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
            paint: black.into(),
            width: 14.0,
            ..Default::default()
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

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_persistence() {
    use tp_app::recovery::{RecoveryMeta, RecoveryStore};
    use tp_app::state::{Modal, PendingAction};
    let dir = tempfile::tempdir().unwrap();
    let recovery = dir.path().join("recovery");
    {
        // A copy left by a crashed session.
        let store = RecoveryStore::open(&recovery).unwrap();
        tp_file::write(
            &tp_core::Project::new("ACE Logistics", tp_core::TextureResolution::R4096),
            &store.copy_path(),
        )
        .unwrap();
        let (path, text) = store.meta_file(&RecoveryMeta {
            name: "ACE Logistics".into(),
            original: Some("/home/user/Liveries/ace.truckpaint".into()),
            saved_at: tp_app::prefs::now_unix() - 300,
        });
        std::fs::write(path, text).unwrap();
    }
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let mut prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        prefs.recent = vec![RecentProject {
            name: "Scania fleet".into(),
            path: "/home/user/Liveries/scania.truckpaint".into(),
            last_opened: tp_app::prefs::now_unix() - 86_400 * 3,
        }];
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        h.state_mut().enable_recovery(&recovery);
        save(&mut h, &format!("home_recovered_{suffix}"));

        common::create_project(&mut h);
        h.state_mut().modal = Some(Modal::UnsavedChanges(PendingAction::CloseProject));
        save(&mut h, &format!("unsaved_prompt_{suffix}"));
        h.state_mut().modal = Some(Modal::Message {
            title: "Cannot open project".into(),
            text: "ace.truckpaint was created with a newer version of TruckPaint.".into(),
        });
        save(&mut h, &format!("message_{suffix}"));
        h.state_mut().modal = None;
        let ws = h.state_mut().workspace_mut().unwrap();
        ws.saving = Some(tp_app::workspace::PendingSave {
            id: 0,
            snapshot: ws.snapshot(),
            path: "/tmp/a.truckpaint".into(),
        });
        save(&mut h, &format!("status_saving_{suffix}"));
    }
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_export() {
    use tp_app::state::Modal;
    use tp_app::ui::export_dialog::ExportDialog;
    let wait_preview = |h: &mut egui_kittest::Harness<'static, tp_app::AppState>| {
        for _ in 0..400 {
            h.step();
            let ready = matches!(&h.state().modal, Some(Modal::Export(d)) if d.preview_ready());
            if ready {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        h.step();
    };
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        common::create_project(&mut h);
        lettering_scene(&mut h);
        wait_for_images(&mut h);
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::E);
        wait_preview(&mut h);
        save(&mut h, &format!("export_png_{suffix}"));
        if let Some(Modal::Export(d)) = &mut h.state_mut().modal {
            d.settings.format = tp_app::export::ExportFormat::Dds;
            d.settings.background = None;
        }
        wait_preview(&mut h);
        save(&mut h, &format!("export_dds_transparent_{suffix}"));
        let _ = ExportDialog::preview_ready;
    }
    // The exported texture itself, to compare with the canvas.
    let mut h = common::wgpu_harness_with(Prefs::default(), Vec2::new(1440.0, 900.0));
    common::create_project(&mut h);
    lettering_scene(&mut h);
    let ws = h.state_mut().workspace_mut().unwrap();
    let pixmap = tp_render::render(
        &ws.project,
        0,
        tp_render::RenderOptions {
            size: 4096,
            background: Some(tp_core::document::Rgba::rgb(255, 255, 255)),
        },
        &mut ws.text.fonts,
        &mut |_, _| true,
    )
    .unwrap();
    std::fs::write(
        out_dir().join("export_4k.png"),
        tp_render::encode_png(&pixmap).unwrap(),
    )
    .unwrap();
}

/// Stars, a polygon, a swoosh with a hole and open lines.
fn vector_scene(
    h: &mut egui_kittest::Harness<'static, tp_app::AppState>,
) -> Vec<tp_core::document::ObjectId> {
    use tp_core::document::{
        Frame, Node, Object, ObjectId, PathData, Rgba, ShapeKind, StrokeStyle, Subpath,
    };
    use tp_core::kurbo::{Point, Size};
    let ws = h.state_mut().workspace_mut().unwrap();
    let mut ids = Vec::new();
    let mut add = |o: Object| ids.push(ws.project.add(o));
    let base = |kind, c: (f64, f64), s: (f64, f64), rot: f64, fill: Rgba| {
        let mut o = Object::new(ObjectId(0), kind, Frame::new(c.into(), s.into(), rot));
        o.fill = fill.into();
        o
    };
    add(base(
        ShapeKind::rectangle(),
        (2048.0, 2048.0),
        (4096.0, 4096.0),
        0.0,
        Rgba::rgb(0x1B, 0x2A, 0x41),
    ));
    let mut star = base(
        ShapeKind::Polygon {
            sides: 5,
            star: Some(0.42),
        },
        (800.0, 900.0),
        (900.0, 860.0),
        -8.0,
        Rgba::rgb(0xF0, 0xB4, 0x4C),
    );
    star.stroke = Some(StrokeStyle {
        paint: Rgba::rgb(255, 255, 255).into(),
        width: 24.0,
        ..Default::default()
    });
    add(star);
    add(base(
        ShapeKind::Polygon {
            sides: 6,
            star: None,
        },
        (2000.0, 900.0),
        (800.0, 700.0),
        0.0,
        Rgba::rgb(0xC0, 0x39, 0x2B),
    ));
    let corner = |x: f64, y: f64| Node::corner(Point::new(x, y));
    let swoosh = Subpath::new(
        vec![
            Node::smooth(Point::new(300.0, 2600.0), Point::new(1200.0, 1700.0)),
            Node::smooth(Point::new(3600.0, 2100.0), Point::new(3900.0, 2300.0)),
            Node::smooth(Point::new(3500.0, 2600.0), Point::new(2400.0, 2300.0)),
        ],
        true,
    );
    let hole = Subpath::new(
        vec![
            corner(1600.0, 2250.0),
            corner(1600.0, 2450.0),
            corner(1900.0, 2450.0),
            corner(1900.0, 2250.0),
        ],
        true,
    );
    let mut s = Object::from_path(ObjectId(0), PathData::new(vec![swoosh, hole]));
    s.fill = Rgba::rgb(0x2E, 0x86, 0xDE).into();
    s.name = "Swoosh".into();
    add(s);
    for (i, y) in [3200.0, 3400.0, 3600.0].into_iter().enumerate() {
        let mut line = Object::from_path(
            ObjectId(0),
            PathData::new(vec![Subpath::new(
                vec![
                    corner(400.0, y),
                    Node::smooth(Point::new(2000.0, y - 150.0), Point::new(2600.0, y - 150.0)),
                    corner(3700.0, y),
                ],
                false,
            )]),
        );
        line.fill = Rgba::rgb(255, 255, 255).into();
        line.edit_path(|p| p.line_width = 20.0 + 20.0 * i as f64);
        if i == 2 {
            // The thickest line outlined in black.
            line.stroke = Some(StrokeStyle {
                paint: Rgba::rgb(0, 0, 0).into(),
                width: 12.0,
                ..Default::default()
            });
        }
        line.name = "Line".into();
        add(line);
    }
    let _ = Size::ZERO;
    ids
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_vector_tools() {
    use tp_app::tool::Tool;
    use tp_core::document::{Node, NodeRef, PointRef};
    use tp_core::kurbo::Point;
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        common::create_project(&mut h);
        let ids = vector_scene(&mut h);
        save(&mut h, &format!("vector_canvas_{suffix}"));

        // Direct Selection on the swoosh, its second point selected.
        {
            let ws = h.state_mut().workspace_mut().unwrap();
            ws.tool = Tool::DirectSelect;
            ws.selection = vec![ids[3]];
            ws.points = [PointRef::new(ids[3], NodeRef::new(0, 1))].into();
        }
        save(&mut h, &format!("vector_direct_select_{suffix}"));

        // A pen path being drawn, the next segment following the pointer.
        {
            let ws = h.state_mut().workspace_mut().unwrap();
            ws.points.clear();
            ws.selection.clear();
            ws.tool = Tool::Pen;
            ws.pen = Some(tp_app::path_edit::PenSession {
                nodes: vec![
                    Node::corner(Point::new(600.0, 1500.0)),
                    Node::smooth(Point::new(1400.0, 1300.0), Point::new(1800.0, 1300.0)),
                ],
            });
        }
        let pointer = h
            .state()
            .workspace()
            .unwrap()
            .screen_map(1.0)
            .unwrap()
            .to_screen(Point::new(2400.0, 1700.0));
        h.event(egui::Event::PointerMoved(pointer));
        save(&mut h, &format!("vector_pen_{suffix}"));
    }
    // The exported texture, to compare with the canvas.
    let mut h = common::wgpu_harness_with(Prefs::default(), Vec2::new(1440.0, 900.0));
    common::create_project(&mut h);
    vector_scene(&mut h);
    let ws = h.state_mut().workspace_mut().unwrap();
    let pixmap = tp_render::render(
        &ws.project,
        0,
        tp_render::RenderOptions {
            size: 1024,
            background: None,
        },
        &mut ws.text.fonts,
        &mut |_, _| true,
    )
    .unwrap();
    std::fs::write(
        out_dir().join("vector_export_1k.png"),
        tp_render::encode_png(&pixmap).unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_precision_aids() {
    use tp_core::document::{Frame, Object, ObjectId, Rgba, ShapeKind};
    use tp_core::kurbo::{Point, Size};
    use tp_core::{Axis, Guide};
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let mut prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        prefs.view_aids.grid = true;
        prefs.view_aids.grid_spacing = 128.0;
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        common::create_project(&mut h);
        let moving = {
            let ws = h.state_mut().workspace_mut().unwrap();
            let mut a = Object::new(
                ObjectId(0),
                ShapeKind::rectangle(),
                Frame::new(Point::new(900.0, 1200.0), Size::new(800.0, 400.0), 0.0),
            );
            a.fill = Rgba::rgb(0xC0, 0x39, 0x2B).into();
            ws.project.add(a);
            let mut b = Object::new(
                ObjectId(0),
                ShapeKind::Ellipse,
                Frame::new(Point::new(2900.0, 2400.0), Size::new(600.0, 600.0), 0.0),
            );
            b.fill = Rgba::rgb(0x2E, 0x86, 0xDE).into();
            let b = ws.project.add(b);
            ws.project.add_guide(Guide::new(Axis::Horizontal, 3000.0));
            ws.project.add_guide(Guide::new(Axis::Vertical, 2048.0));
            ws.selection = vec![b];
            b
        };
        let _ = moving;
        h.run();
        save(&mut h, &format!("precision_aids_{suffix}"));

        // Mid-drag: the ellipse's top snaps to the rectangle's top edge.
        let map = |h: &egui_kittest::Harness<'static, tp_app::AppState>, x: f64, y: f64| {
            h.state()
                .workspace()
                .unwrap()
                .screen_map(1.0)
                .unwrap()
                .to_screen(Point::new(x, y))
        };
        let (from, to) = (map(&h, 2900.0, 2400.0), map(&h, 2900.0, 1310.0));
        h.event(egui::Event::PointerMoved(from));
        h.event(egui::Event::PointerButton {
            pos: from,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        });
        h.step();
        for i in 1..=8 {
            h.event(egui::Event::PointerMoved(
                from + (to - from) * (i as f32 / 8.0),
            ));
            h.step();
        }
        let image = h.render().expect("render");
        image
            .save(out_dir().join(format!("precision_snapping_{suffix}.png")))
            .unwrap();
    }
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_align() {
    use tp_app::layout::PanelKind;
    use tp_core::document::{Frame, Object, ObjectId, Rgba, ShapeKind};
    use tp_core::kurbo::{Point, Size};
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let mut prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        for slot in &mut prefs.layout.panels {
            match slot.kind {
                PanelKind::Transform => {
                    slot.open = true;
                    slot.collapsed = false;
                }
                // Room for the Transform panel's align rows.
                PanelKind::Colors | PanelKind::Layers => slot.collapsed = true,
                _ => {}
            }
        }
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        common::create_project(&mut h);
        {
            let ws = h.state_mut().workspace_mut().unwrap();
            let colors = [
                Rgba::rgb(0xC0, 0x39, 0x2B),
                Rgba::rgb(0x2E, 0x86, 0xDE),
                Rgba::rgb(0xF0, 0xB4, 0x4C),
            ];
            let mut ids = Vec::new();
            for (i, (x, y)) in [(700.0, 1200.0), (1700.0, 1500.0), (2800.0, 1000.0)]
                .into_iter()
                .enumerate()
            {
                let mut o = Object::new(
                    ObjectId(0),
                    if i == 1 {
                        ShapeKind::Ellipse
                    } else {
                        ShapeKind::rectangle()
                    },
                    Frame::new(
                        Point::new(x, y),
                        Size::new(500.0, 300.0 + 100.0 * i as f64),
                        0.0,
                    ),
                );
                o.fill = colors[i].into();
                ids.push(ws.project.add(o));
            }
            ws.selection = ids;
            ws.panels.align_to = tp_app::arrange::AlignTo::KeyObject;
        }
        save(&mut h, &format!("align_key_object_{suffix}"));
    }
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_combine() {
    use tp_app::layout::PanelKind;
    use tp_core::document::{BooleanOp, Frame, Object, ObjectId, Rgba, ShapeKind};
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let mut prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        for slot in &mut prefs.layout.panels {
            match slot.kind {
                PanelKind::Transform => {
                    slot.open = true;
                    slot.collapsed = false;
                }
                PanelKind::Colors | PanelKind::Layers => slot.collapsed = true,
                _ => {}
            }
        }
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        common::create_project(&mut h);
        {
            let ws = h.state_mut().workspace_mut().unwrap();
            let shape = |kind, c: (f64, f64), s: (f64, f64), fill| {
                let mut o = Object::new(ObjectId(0), kind, Frame::new(c.into(), s.into(), 0.0));
                o.fill = tp_core::document::Paint::Solid(fill);
                o
            };
            // A stripe with a round notch (Minus Front).
            let stripe = ws.project.add(shape(
                ShapeKind::rectangle(),
                (1500.0, 1200.0),
                (2600.0, 500.0),
                Rgba::rgb(0x2E, 0x86, 0xDE),
            ));
            let disk = ws.project.add(shape(
                ShapeKind::Ellipse,
                (2800.0, 1200.0),
                (700.0, 700.0),
                Rgba::rgb(0xC0, 0x39, 0x2B),
            ));
            ws.selection = vec![stripe, disk];
            ws.combine_selection(BooleanOp::MinusFront, 0.0);
            // Two overlapping shapes selected, ready to combine.
            let a = ws.project.add(shape(
                ShapeKind::Ellipse,
                (1200.0, 2700.0),
                (900.0, 900.0),
                Rgba::rgb(0xF0, 0xB4, 0x4C),
            ));
            let b = ws.project.add(shape(
                ShapeKind::rectangle(),
                (1800.0, 2700.0),
                (900.0, 700.0),
                Rgba::rgb(0x27, 0xAE, 0x60),
            ));
            ws.selection = vec![a, b];
        }
        save(&mut h, &format!("combine_{suffix}"));
    }
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_stroke_options() {
    use tp_app::layout::PanelKind;
    use tp_core::document::{
        Cap, CharStyle, Dash, Frame, Join, LineStyle, Node, Object, ObjectId, PathData, Rgba,
        ShapeKind, StrokeAlign, StrokeStyle, Subpath,
    };
    use tp_core::kurbo::Point;
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let mut prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        for slot in &mut prefs.layout.panels {
            match slot.kind {
                PanelKind::Stroke => {
                    slot.open = true;
                    slot.collapsed = false;
                }
                PanelKind::Colors | PanelKind::Layers | PanelKind::Transform => {
                    slot.collapsed = true;
                }
                _ => {}
            }
        }
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        common::create_project(&mut h);
        let (text, line) = {
            let ws = h.state_mut().workspace_mut().unwrap();
            // A burgundy base with an inside border.
            let mut base = Object::new(
                ObjectId(0),
                ShapeKind::rectangle(),
                Frame::new(Point::new(2048.0, 2048.0), (3400.0, 2200.0).into(), 0.0),
            );
            base.fill = Rgba::rgb(0x7A, 0x1F, 0x2B).into();
            base.stroke = Some(StrokeStyle {
                paint: Rgba::rgb(0xF0, 0xB4, 0x4C).into(),
                width: 40.0,
                align: StrokeAlign::Inside,
                line: LineStyle {
                    join: Join::Round,
                    ..LineStyle::default()
                },
                swatch: None,
            });
            ws.project.add(base);
            // Outside-outlined lettering.
            let text = add_text(
                ws,
                "ACE",
                CharStyle {
                    family: "Barlow Condensed".into(),
                    weight: 700,
                    size: 900.0,
                    ..CharStyle::default()
                },
                (900.0, 1300.0),
                Rgba::rgb(255, 255, 255),
                Some(StrokeStyle {
                    paint: Rgba::rgb(0x11, 0x11, 0x11).into(),
                    width: 30.0,
                    align: StrokeAlign::Outside,
                    line: LineStyle {
                        join: Join::Round,
                        ..LineStyle::default()
                    },
                    swatch: None,
                }),
            );
            // A dashed pinstripe and a dotted line under the lettering.
            let mut stripe = |y: f64, dash: Dash, cap: Cap, width: f64| {
                let mut data = PathData::new(vec![Subpath::new(
                    vec![
                        Node::corner(Point::new(700.0, y)),
                        Node::corner(Point::new(3400.0, y)),
                    ],
                    false,
                )]);
                data.line_width = width;
                data.line_style = LineStyle {
                    dash: Some(dash),
                    cap,
                    ..LineStyle::default()
                };
                let mut o = Object::from_path(ObjectId(0), data);
                o.name = "Pinstripe".into();
                o.fill = Rgba::rgb(0xF0, 0xB4, 0x4C).into();
                ws.project.add(o)
            };
            let line = stripe(
                2700.0,
                Dash {
                    dash: 120.0,
                    gap: 60.0,
                },
                Cap::Butt,
                36.0,
            );
            stripe(
                2900.0,
                Dash {
                    dash: 0.0,
                    gap: 60.0,
                },
                Cap::Round,
                30.0,
            );
            ws.selection = vec![text];
            (text, line)
        };
        let _ = text;
        save(&mut h, &format!("stroke_options_text_{suffix}"));
        h.state_mut().workspace_mut().unwrap().selection = vec![line];
        save(&mut h, &format!("stroke_options_line_{suffix}"));
    }
}

/// A gradient livery: a linear cab fade, a radial glow behind gradient
/// lettering with an Outside stroke, and a gradient dashed line. Returns the
/// lettering and the line.
fn gradient_scene(
    h: &mut egui_kittest::Harness<'static, tp_app::AppState>,
) -> (tp_core::document::ObjectId, tp_core::document::ObjectId) {
    use tp_core::document::{
        Cap, CharStyle, ColorStop, Dash, Frame, Gradient, GradientKind, Join, LineStyle, Node,
        Object, ObjectId, Paint, PathData, Rgba, ShapeKind, StrokeAlign, StrokeStyle, Subpath,
    };
    use tp_core::kurbo::Point;
    let ws = h.state_mut().workspace_mut().unwrap();
    let stops = |a: Rgba, b: Rgba| [ColorStop::new(0.0, a), ColorStop::new(1.0, b)];
    // Cab side fading from burgundy to near black, left to right.
    let mut base = Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(2048.0, 2048.0), (3600.0, 2400.0).into(), 0.0),
    );
    base.name = "Fade".into();
    base.fill = Paint::Gradient(Gradient::new(
        GradientKind::Linear,
        &stops(Rgba::rgb(0x9B, 0x22, 0x35), Rgba::rgb(0x1A, 0x08, 0x0C)),
    ));
    ws.project.add(base);
    // A gold glow behind the lettering.
    let mut glow = Object::new(
        ObjectId(0),
        ShapeKind::Ellipse,
        Frame::new(Point::new(2048.0, 1500.0), (2600.0, 1200.0).into(), 0.0),
    );
    glow.name = "Glow".into();
    glow.fill = Paint::Gradient(Gradient::new(
        GradientKind::Radial,
        &stops(
            Rgba::with_alpha(0xF0, 0xB4, 0x4C, 200),
            Rgba::with_alpha(0xF0, 0xB4, 0x4C, 0),
        ),
    ));
    ws.project.add(glow);
    // Gold-to-bronze lettering with a black Outside outline.
    let text = add_text(
        ws,
        "ACE",
        CharStyle {
            family: "Barlow Condensed".into(),
            weight: 700,
            size: 900.0,
            ..CharStyle::default()
        },
        (1300.0, 1050.0),
        Rgba::rgb(255, 255, 255),
        Some(StrokeStyle {
            paint: Rgba::rgb(0x11, 0x11, 0x11).into(),
            width: 30.0,
            align: StrokeAlign::Outside,
            line: LineStyle {
                join: Join::Round,
                ..LineStyle::default()
            },
            swatch: None,
        }),
    );
    let mut lettering = (**ws.project.surface().get(text).unwrap()).clone();
    let mut gold = Gradient::new(
        GradientKind::Linear,
        &stops(Rgba::rgb(0xFF, 0xE0, 0x8A), Rgba::rgb(0xA8, 0x6A, 0x1E)),
    );
    gold.set_angle(&lettering.frame, 90.0);
    lettering.fill = Paint::Gradient(gold);
    ws.project.surface_mut().replace(&[lettering]);
    // A dashed line going from yellow to red.
    let mut data = PathData::new(vec![Subpath::new(
        vec![
            Node::corner(Point::new(500.0, 2900.0)),
            Node::corner(Point::new(3600.0, 2900.0)),
        ],
        false,
    )]);
    data.line_width = 40.0;
    data.line_style = LineStyle {
        dash: Some(Dash {
            dash: 160.0,
            gap: 60.0,
        }),
        cap: Cap::Butt,
        ..LineStyle::default()
    };
    let mut line = Object::from_path(ObjectId(0), data);
    line.name = "Stripe".into();
    line.fill = Paint::Gradient(Gradient::new(
        GradientKind::Linear,
        &stops(Rgba::rgb(0xFF, 0xD0, 0x20), Rgba::rgb(0xE0, 0x20, 0x20)),
    ));
    let line = ws.project.add(line);
    (text, line)
}

#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_gradients() {
    use tp_app::layout::PanelKind;
    use tp_app::tool::Tool;
    use tp_app::workspace::ColorTarget;
    for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
        let mut prefs = Prefs {
            ui_scale: scale,
            ..Prefs::default()
        };
        for slot in &mut prefs.layout.panels {
            match slot.kind {
                PanelKind::Colors => {
                    slot.open = true;
                    slot.collapsed = false;
                }
                PanelKind::Stroke | PanelKind::Layers | PanelKind::Transform => {
                    slot.collapsed = true;
                }
                _ => {}
            }
        }
        let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
        let mut h = common::wgpu_harness_with(prefs, size);
        common::create_project(&mut h);
        let (text, line) = gradient_scene(&mut h);
        {
            let ws = h.state_mut().workspace_mut().unwrap();
            ws.selection = vec![text];
            ws.panels.color_target = ColorTarget::Fill;
        }
        save(&mut h, &format!("gradients_text_{suffix}"));
        {
            let ws = h.state_mut().workspace_mut().unwrap();
            ws.selection = vec![line];
            ws.tool = Tool::Gradient;
        }
        save(&mut h, &format!("gradients_tool_{suffix}"));
    }
}

/// The canvas and the export agree on gradient colors (interior pixels).
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn canvas_matches_export_for_gradients() {
    use tp_core::document::Rgba;
    use tp_core::kurbo::Point;
    let mut h = common::wgpu_harness_with(Prefs::default(), Vec2::new(1440.0, 900.0));
    common::create_project(&mut h);
    gradient_scene(&mut h);
    h.state_mut().workspace_mut().unwrap().selection.clear();
    h.run();
    let canvas = h.render().expect("render");
    let ws = h.state_mut().workspace_mut().unwrap();
    let map = ws.screen_map(1.0).expect("canvas");
    let side = 2048u32;
    let export = tp_render::to_rgba(
        &tp_render::render(
            &ws.project,
            0,
            tp_render::RenderOptions {
                size: side,
                background: Some(Rgba::rgb(255, 255, 255)),
            },
            &mut ws.text.fonts,
            &mut |_, _| true,
        )
        .unwrap(),
    );
    let surface = ws.project.surface().size;
    // Inside the cab fade (away from the glow, text and line) and on dashes.
    let samples = [
        Point::new(500.0, 2300.0),
        Point::new(1200.0, 2400.0),
        Point::new(2600.0, 2500.0),
        Point::new(3500.0, 2300.0),
        Point::new(700.0, 2900.0),
        Point::new(2050.0, 2900.0),
    ];
    let mut worst = 0u8;
    for p in samples {
        let s = map.to_screen(p);
        let c = canvas.get_pixel(s.x.round() as u32, s.y.round() as u32).0;
        let e = export
            .get_pixel(
                (p.x / surface * f64::from(side)) as u32,
                (p.y / surface * f64::from(side)) as u32,
            )
            .0;
        let d = (0..3).map(|i| c[i].abs_diff(e[i])).max().unwrap();
        worst = worst.max(d);
        eprintln!("{p:?}: canvas {c:?} export {e:?}");
    }
    assert!(worst <= 2, "canvas and export differ by {worst} levels");
}

/// The main screens in every language at 100% and 200% (layout check for
/// longer translations).
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_languages() {
    use tp_app::layout::PanelKind;
    use tp_app::state::Modal;
    use tp_i18n::Language;
    for language in Language::ALL {
        for (scale, suffix) in [(1.0, "100"), (2.0, "200")] {
            let mut prefs = Prefs {
                ui_scale: scale,
                ..Prefs::default()
            };
            prefs.set_language(Some(language));
            for slot in &mut prefs.layout.panels {
                slot.collapsed = !matches!(slot.kind, PanelKind::Properties | PanelKind::Colors);
            }
            let size = Vec2::new(1440.0, 900.0) * scale.max(1.0);
            let code = language.code();
            let mut h = common::wgpu_harness_with(prefs, size);
            save(&mut h, &format!("lang_{code}_home_{suffix}"));
            h.state_mut().modal = Some(Modal::NewProject(Default::default()));
            save(&mut h, &format!("lang_{code}_new_project_{suffix}"));
            h.state_mut().modal = None;
            h.run();
            // A project with a selected stroked text (Properties, Colors).
            h.get_by_label(&tp_i18n::tr("home-new-project")).click();
            h.run();
            h.get_by_label(&tp_i18n::tr("button-next")).click();
            h.run();
            h.get_by_label(&tp_i18n::tr("button-create")).click();
            h.run();
            let (text, _) = gradient_scene(&mut h);
            h.state_mut().workspace_mut().unwrap().selection = vec![text];
            save(&mut h, &format!("lang_{code}_workspace_{suffix}"));
            h.state_mut().modal = Some(Modal::Preferences);
            save(&mut h, &format!("lang_{code}_preferences_{suffix}"));
            h.state_mut().modal = None;
            h.run();
            h.state_mut().workspace_mut().unwrap().selection.clear();
            for slot in &mut h.state_mut().prefs.layout.panels {
                slot.collapsed = !matches!(slot.kind, PanelKind::Transform | PanelKind::Stroke);
            }
            let r = h
                .state_mut()
                .workspace_mut()
                .unwrap()
                .project
                .surface()
                .objects[0]
                .id;
            h.state_mut().workspace_mut().unwrap().selection = vec![r];
            save(&mut h, &format!("lang_{code}_transform_{suffix}"));
        }
    }
}

/// A vehicle project (template overlay, sidebar with an update) and the New
/// Project vehicle step, in English and German.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_vehicle_screens() {
    use tp_app::layout::PanelKind;
    use tp_app::state::Modal;
    use tp_vehicles::sample::{self, SampleTexture};
    for language in [tp_i18n::Language::English, tp_i18n::Language::German] {
        let dir = tempfile::tempdir().unwrap();
        let mut prefs = Prefs::default();
        prefs.set_language(Some(language));
        for slot in &mut prefs.layout.panels {
            slot.collapsed = !matches!(slot.kind, PanelKind::Layers);
        }
        let mut h = common::wgpu_harness_with(prefs, Vec2::new(1440.0, 900.0));
        h.state_mut().vehicles =
            tp_app::vehicles::VehicleLibrary::open(&dir.path().join("library"));
        let tex = |id, name, size, layout| SampleTexture {
            id,
            name,
            size,
            layout,
        };
        for (version, cabin_layout) in [("1.2.0", 1), ("1.3.0", 2)] {
            let bytes = sample::package(
                "scs.sample.truck",
                "Sample Truck",
                version,
                &[
                    tex("cabin", "Cabin", 4096, cabin_layout),
                    tex("chassis", "Chassis", 2048, 1),
                    tex("accessories", "Accessories", 1024, 1),
                ],
            );
            h.state_mut().vehicles.install_bytes(&bytes).unwrap();
        }
        let code = language.code();
        h.state_mut().modal = Some(Modal::NewProject(Default::default()));
        if let Some(Modal::NewProject(d)) = &mut h.state_mut().modal {
            d.vehicle = Some(tp_app::ui::vehicle_dialogs::VehicleChoice {
                id: "scs.sample.truck".into(),
                version: "1.3.0".parse().unwrap(),
                textures: vec!["chassis".into(), "accessories".into()],
            });
        }
        save(&mut h, &format!("vehicle_wizard_{code}"));
        h.state_mut().modal = None;
        h.run();
        let package = h
            .state()
            .vehicles
            .load("scs.sample.truck", &"1.2.0".parse().unwrap())
            .unwrap();
        let textures = tp_app::vehicle_project::default_textures(&package.manifest);
        let project =
            tp_app::vehicle_project::fleet_project("Sample Truck", &package, &textures).unwrap();
        h.state_mut().open_project(project);
        save(&mut h, &format!("vehicle_project_{code}"));
    }
}

/// The empty New Project vehicle step and Vehicle Library with the samples
/// button, then projects on the samples' templates, in English and French.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_sample_vehicle() {
    use tp_app::state::Modal;
    for language in [tp_i18n::Language::English, tp_i18n::Language::French] {
        let dir = tempfile::tempdir().unwrap();
        let mut prefs = Prefs::default();
        prefs.set_language(Some(language));
        let code = language.code();
        let mut h = common::wgpu_harness_with(prefs, Vec2::new(1440.0, 900.0));
        h.state_mut().vehicles =
            tp_app::vehicles::VehicleLibrary::open(&dir.path().join("library"));
        h.state_mut().modal = Some(Modal::NewProject(Default::default()));
        save(&mut h, &format!("sample_wizard_empty_{code}"));
        h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
        save(&mut h, &format!("sample_library_empty_{code}"));
        h.state_mut().modal = None;
        h.state_mut().vehicles.install_samples().unwrap();
        let truck = h
            .state()
            .vehicles
            .load(tp_app::vehicles::SAMPLE_ID, &"1.1.0".parse().unwrap())
            .unwrap();
        let trailer = h
            .state()
            .vehicles
            .load(
                tp_app::vehicles::SAMPLE_TRAILER_ID,
                &"1.0.0".parse().unwrap(),
            )
            .unwrap();
        for (name, package, main) in [
            ("standard", &truck, "standard"),
            ("high_roof", &truck, "high_roof"),
            ("trailer", &trailer, "base"),
        ] {
            let mut textures = tp_app::vehicle_project::default_textures(&package.manifest);
            textures[0] = main.to_owned();
            let project =
                tp_app::vehicle_project::fleet_project("Sample", package, &textures).unwrap();
            h.state_mut().open_project(project);
            h.run();
            save(&mut h, &format!("sample_{name}_{code}"));
        }
    }
}

/// A fleet: the sample truck's two main textures and the sample trailer,
/// with the sidebar tree, the Add Vehicle and Textures dialogs, the Update
/// Template dialog and the wizard's texture checkboxes, in English and
/// German.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_fleet_screens() {
    use tp_app::layout::PanelKind;
    use tp_app::state::Modal;
    use tp_app::ui::vehicle_dialogs::{AddVehicleDialog, TexturesDialog, VehicleChoice};
    use tp_app::vehicles::{SAMPLE_ID, SAMPLE_TRAILER_ID};
    for language in [tp_i18n::Language::English, tp_i18n::Language::German] {
        let mut prefs = Prefs::default();
        prefs.set_language(Some(language));
        for slot in &mut prefs.layout.panels {
            slot.collapsed = slot.kind != PanelKind::Properties;
        }
        let code = language.code();
        let mut h = common::wgpu_harness_with(prefs, Vec2::new(1440.0, 900.0));
        // Wizard with both main textures checked.
        let both = vec![
            "standard".to_owned(),
            "high_roof".to_owned(),
            "chassis".to_owned(),
            "cab_accessories".to_owned(),
            "side_skirts".to_owned(),
        ];
        h.state_mut().modal = Some(Modal::NewProject(Default::default()));
        if let Some(Modal::NewProject(d)) = &mut h.state_mut().modal {
            d.vehicle = Some(VehicleChoice {
                id: SAMPLE_ID.into(),
                version: "1.1.0".parse().unwrap(),
                textures: both.clone(),
            });
        }
        save(&mut h, &format!("fleet_wizard_{code}"));
        // The fleet project: the truck and the trailer.
        let load =
            |h: &egui_kittest::Harness<'static, tp_app::AppState>, id: &str, version: &str| {
                h.state()
                    .vehicles
                    .load(id, &version.parse().unwrap())
                    .unwrap()
            };
        let truck = load(&h, SAMPLE_ID, "1.1.0");
        let project =
            tp_app::vehicle_project::fleet_project("ACE Logistics", &truck, &both).unwrap();
        h.state_mut().modal = None;
        h.state_mut().open_project(project);
        let trailer = load(&h, SAMPLE_TRAILER_ID, "1.0.0");
        let all = tp_app::vehicle_project::default_textures(&trailer.manifest);
        h.state_mut()
            .workspace_mut()
            .unwrap()
            .add_vehicle(&trailer, &all, 1.0)
            .unwrap();
        let ws = h.state_mut().workspace_mut().unwrap();
        ws.set_active_surface(2);
        // A texture flagged by an update: ⚠ in the tree, the notice in
        // Properties.
        ws.project.surfaces[2].template.as_mut().unwrap().status =
            tp_core::TemplateStatus::LayoutChanged;
        for _ in 0..20 {
            h.step();
        }
        save(&mut h, &format!("fleet_project_{code}"));
        h.state_mut().modal = Some(Modal::AddVehicle(AddVehicleDialog::for_game(Some("ets2"))));
        save(&mut h, &format!("fleet_add_vehicle_{code}"));
        let dialog = {
            let p = &h.state().workspace().unwrap().project;
            TexturesDialog::new(p, &p.vehicles[0])
        };
        h.state_mut().modal = Some(Modal::Textures(dialog));
        save(&mut h, &format!("fleet_textures_{code}"));
    }
}

/// The Custom Vehicle dialog filled in, with a texture missing its game
/// ids, then the Vehicle Library with the custom vehicle (New Version…,
/// Export…), in English and French.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_custom_vehicle() {
    use tp_app::state::Modal;
    use tp_app::ui::custom_vehicle::{CustomVehicleDialog, Origin};
    for language in [tp_i18n::Language::English, tp_i18n::Language::French] {
        let dir = tempfile::tempdir().unwrap();
        let mut prefs = Prefs::default();
        prefs.set_language(Some(language));
        let code = language.code();
        let mut h = common::wgpu_harness_with(prefs, Vec2::new(1440.0, 1000.0));
        h.state_mut().vehicles =
            tp_app::vehicles::VehicleLibrary::open(&dir.path().join("library"));
        let mut dialog = CustomVehicleDialog::new(Origin::NewProject(Default::default()));
        dialog.form.name = "R 2024".into();
        dialog.form.brand = "Scania".into();
        dialog.form.path = "scania.r_2024".into();
        let wide = {
            let mut out = Vec::new();
            image::RgbaImage::new(64, 32)
                .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
                .unwrap();
            out
        };
        dialog.add_files(
            vec![
                ("cabin.png".into(), Ok(tp_vehicles::sample::png(64))),
                ("side_skirts.png".into(), Ok(tp_vehicles::sample::png(32))),
                ("mirrors.png".into(), Ok(wide)),
            ],
            None,
        );
        dialog.form.rows[2].game_ids = "mirror.painted, s_mirror.painted".into();
        h.state_mut().modal = Some(Modal::CustomVehicle(Box::new(dialog)));
        save(&mut h, &format!("custom_vehicle_{code}"));

        // Build it, then show the library.
        let packed = match &h.state().modal {
            Some(Modal::CustomVehicle(d)) => {
                let mut form = d.form.clone();
                form.rows[1].game_ids = "sideskirt.a".into();
                form.pack(&mut || {}).unwrap()
            }
            _ => unreachable!(),
        };
        h.state_mut().vehicles.install_bytes(&packed.bytes).unwrap();
        h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
        save(&mut h, &format!("custom_vehicle_library_{code}"));
    }
}

/// The brand kit: the palette with a linked swatch marked, the Edit Swatch
/// popup, the Styles panel with both sections, and the Copy From Cabin
/// dialog, in English and German.
#[test]
#[ignore = "needs a GPU; run manually for visual QA"]
fn render_brand_kit() {
    use tp_app::layout::PanelKind;
    use tp_app::state::Modal;
    use tp_core::document::{
        CharStyle, ColorStop, Frame, Gradient, GradientKind, Object, ObjectId, Paint, Rgba,
        ShapeKind, StrokeStyle, TextBlock,
    };
    use tp_core::kurbo::{Point, Size};
    for language in [tp_i18n::Language::English, tp_i18n::Language::German] {
        let mut prefs = Prefs::default();
        prefs.set_language(Some(language));
        for slot in &mut prefs.layout.panels {
            slot.collapsed = !matches!(slot.kind, PanelKind::Colors | PanelKind::Styles);
        }
        let code = language.code();
        let mut h = common::wgpu_harness_with(prefs, Vec2::new(1440.0, 1100.0));
        let package = tp_vehicles::Package::read(tp_app::vehicles::SAMPLES[0].bytes).unwrap();
        let mut textures = tp_app::vehicle_project::default_textures(&package.manifest);
        textures.push("high_roof".into());
        let project =
            tp_app::vehicle_project::fleet_project("ACE Logistics", &package, &textures).unwrap();
        h.state_mut().open_project(project);
        let ws = h.state_mut().workspace_mut().unwrap();
        let p = &mut ws.project;
        let (red, _) = p.add_swatch(Rgba::rgb(0xC0, 0x10, 0x20), "Color");
        p.rename_swatch(red, "Company red");
        let (grey, _) = p.add_swatch(Rgba::rgb(0x50, 0x55, 0x5A), "Color");
        p.rename_swatch(grey, "Company grey");
        p.add_swatch(Rgba::rgb(0xF0, 0xB4, 0x4C), "Color");
        let frame = |x, y, w, h| Frame::new(Point::new(x, y), Size::new(w, h), 0.0);
        let mut stripe = Object::new(
            ObjectId(0),
            ShapeKind::rectangle(),
            frame(2048.0, 2600.0, 3600.0, 260.0),
        );
        stripe.name = "Stripe".into();
        let mut stop = ColorStop::new(0.0, Rgba::rgb(0xC0, 0x10, 0x20));
        stop.swatch = Some(red);
        stripe.fill = Paint::Gradient(Gradient::new(
            GradientKind::Linear,
            &[stop, ColorStop::new(1.0, Rgba::rgb(0x50, 0x55, 0x5A))],
        ));
        let stripe = p.add(stripe);
        p.new_graphic_style(stripe, "Style");
        p.rename_style(p.graphic_styles[0].id, "Stripe");
        let mut band = Object::new(
            ObjectId(0),
            ShapeKind::rectangle(),
            frame(2048.0, 3000.0, 3600.0, 120.0),
        );
        band.fill = Paint::Solid(Rgba::rgb(0x50, 0x55, 0x5A));
        band.fill_swatch = Some(grey);
        band.stroke = Some(StrokeStyle {
            paint: Paint::Solid(Rgba::rgb(0xC0, 0x10, 0x20)),
            swatch: Some(red),
            width: 12.0,
            ..StrokeStyle::default()
        });
        let band = p.add(band);
        p.new_graphic_style(band, "Style");
        p.rename_style(p.graphic_styles[1].id, "Band");
        let mut lettering = Object::new(
            ObjectId(0),
            ShapeKind::Text,
            frame(1200.0, 1500.0, 10.0, 10.0),
        );
        lettering.fill = Paint::Solid(Rgba::rgb(0xC0, 0x10, 0x20));
        lettering.fill_swatch = Some(red);
        lettering.text = Some(TextBlock::new(
            "ACE LOGISTICS",
            CharStyle {
                size: 260.0,
                ..CharStyle::default()
            },
        ));
        let lettering = p.add(lettering);
        p.new_text_style(lettering, "Text style");
        p.rename_style(p.text_styles[0].id, "Lettering");
        ws.relayout_all_texts();
        ws.selection = vec![band];
        ws.panels.color_target = tp_app::workspace::ColorTarget::Fill;
        for _ in 0..20 {
            h.step();
        }
        save(&mut h, &format!("brand_palette_styles_{code}"));
        let ws = h.state_mut().workspace_mut().unwrap();
        ws.start_swatch_edit(red);
        save(&mut h, &format!("brand_edit_swatch_{code}"));
        let ws = h.state_mut().workspace_mut().unwrap();
        ws.cancel_swatch_edit();
        let high_roof = ws
            .project
            .surfaces
            .iter()
            .position(|s| s.name == "High roof")
            .unwrap();
        ws.set_active_surface(high_roof);
        let dialog = tp_app::ui::vehicle_dialogs::CopyFromCabinDialog::new(&ws.project);
        h.state_mut().modal = Some(Modal::CopyFromCabin(dialog));
        save(&mut h, &format!("brand_copy_from_cabin_{code}"));
    }
}
