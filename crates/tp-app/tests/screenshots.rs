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
