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
