//! Headless tests for drop shadows in the Workshop: the inspector's Shadow
//! row and its popover, and the shadows on the canvas (drawn under their
//! object, ignored by selection and snapping, and the same as in the
//! export).

mod common;

use egui::accesskit::Role;
use egui::{Event, Key, Modifiers, PointerButton, Pos2, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use tp_app::AppState;
use tp_app::prefs::Prefs;
use tp_app::viewport::Viewport;
use tp_app::workspace::Workspace;
use tp_core::document::{
    CharStyle, Frame, Object, ObjectId, Paint, Rgba, Shadow, ShapeKind, TextBlock, selection_frame,
};
use tp_core::kurbo::{Point, Size, Vec2 as DocVec};
use tp_ui::widgets::Popover;

type H = Harness<'static, AppState>;

const BUTTON: Role = Role::Button;

/// Tall window, so no section of the inspector scrolls.
fn open() -> H {
    let mut h = common::builder()
        .with_size(Vec2::new(1440.0, 2400.0))
        .with_step_dt(1.0 / 4.0)
        .build_ui_state(
            |ui, state: &mut AppState| state.show(ui),
            AppState::with_prefs(Prefs::default(), None),
        );
    common::create_project(&mut h);
    h.run();
    h
}

fn ws(h: &H) -> &Workspace {
    h.state().workspace().expect("project open")
}

fn ws_mut(h: &mut H) -> &mut Workspace {
    h.state_mut().workspace_mut().expect("project open")
}

fn obj(h: &H, id: ObjectId) -> Object {
    (**ws(h).project.surface().get(id).expect("object")).clone()
}

fn shadow(h: &H, id: ObjectId) -> Option<Shadow> {
    obj(h, id).shadow
}

fn add_text(h: &mut H, x: f64, blur: Option<f64>) -> ObjectId {
    let at = Point::new(x, 1600.0);
    let mut text = Object::text(ObjectId(0), TextBlock::new("ACE", CharStyle::default()), at);
    ws_mut(h).text.place_at(&mut text, at);
    text.shadow = blur.map(|blur| Shadow {
        blur,
        ..Shadow::DEFAULT
    });
    let id = ws_mut(h).project.add(text);
    h.run();
    id
}

fn add_rect(h: &mut H, center: Point, side: f64, shadow: Option<Shadow>) -> ObjectId {
    let mut o = Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(center, Size::new(side, side), 0.0),
    );
    o.fill = Paint::Solid(Rgba::rgb(255, 255, 255));
    o.shadow = shadow;
    let id = ws_mut(h).project.add(o);
    h.run();
    id
}

fn select(h: &mut H, ids: &[ObjectId]) {
    ws_mut(h).selection = ids.to_vec();
    h.run();
}

fn has(h: &H, label: &str) -> bool {
    h.query_by_label(label).is_some()
}

/// What the Shadow row reads.
fn row_value(h: &H) -> String {
    h.get_by_role_and_label(BUTTON, "Shadow")
        .value()
        .unwrap_or_default()
}

fn popover_open(h: &H) -> bool {
    Popover::is_open(&h.ctx, tp_app::ui::workspace::inspector::shadow_popover())
}

fn open_popover(h: &mut H) {
    if !popover_open(h) {
        h.get_by_role_and_label(BUTTON, "Shadow").click();
        h.run();
    }
    assert!(popover_open(h));
}

fn type_into(h: &mut H, name: &str, text: &str) {
    h.get_by_role_and_label(Role::TextInput, name).focus();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.get_by_role_and_label(Role::TextInput, name)
        .type_text(text);
    h.run();
    h.key_press(Key::Enter);
    h.run();
}

/// Presses at `from`, moves in steps to `to` and releases.
fn drag(h: &mut H, from: Pos2, to: Pos2) {
    h.event(Event::PointerMoved(from));
    h.event(Event::PointerButton {
        pos: from,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.step();
    for i in 1..=6 {
        let t = i as f32 / 6.0;
        h.event(Event::PointerMoved(from + (to - from) * t));
        h.step();
    }
    h.event(Event::PointerButton {
        pos: to,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.step();
    h.run();
}

/// A click at `at`, one frame between press and release.
fn click_at(h: &mut H, at: Pos2) {
    h.event(Event::PointerMoved(at));
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.step();
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.step();
    h.run();
}

fn screen(h: &H, p: Point) -> Pos2 {
    ws(h).screen_map(1.0).expect("canvas shown").to_screen(p)
}

// --- Inspector ---------------------------------------------------------------

#[test]
fn add_a_shadow() {
    let mut h = open();
    let text = add_text(&mut h, 800.0, None);
    select(&mut h, &[text]);
    assert!(!has(&h, "Shadow"));
    h.get_by_role_and_label(BUTTON, "+ Add a shadow").click();
    h.run();
    assert_eq!(shadow(&h, text), Some(Shadow::DEFAULT));
    assert_eq!(row_value(&h), "8 / 8 · 8");
    assert!(!has(&h, "+ Add a shadow"));
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(shadow(&h, text), None);
    assert!(has(&h, "+ Add a shadow"));
}

/// How far below its object's center the canvas's shadow of `id` shows.
fn reach(h: &H, id: ObjectId) -> f64 {
    let center = obj(h, id).frame.center;
    (0..2000)
        .map(f64::from)
        .filter(|dy| {
            let at = center + DocVec::new(0.0, *dy);
            ws(h)
                .shadows
                .color_at(id, center, at)
                .is_some_and(|c| c.a > 0)
        })
        .fold(0.0, f64::max)
}

#[test]
fn edit_the_blur_by_dragging() {
    let mut h = open();
    let text = add_text(&mut h, 800.0, Some(8.0));
    select(&mut h, &[text]);
    common::settle_renders(&mut h);
    open_popover(&mut h);
    let label = h.get_by_role_and_label(Role::Label, "Blur").rect().center();
    let steps_before = ws(&h).history.len();
    let renders = ws(&h).shadows.renders;
    let reach_before = reach(&h, text);
    drag(&mut h, label, label + Vec2::new(24.0, 0.0));
    let blur = shadow(&h, text).unwrap().blur;
    assert!((14.0..=26.0).contains(&blur), "blur {blur}");
    assert_eq!(ws(&h).history.len(), steps_before + 1);
    // The canvas follows: the layer is rendered again with the new blur.
    common::settle_renders(&mut h);
    assert!(ws(&h).shadows.renders > renders);
    assert!(reach(&h, text) > reach_before + 4.0);
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(shadow(&h, text).unwrap().blur, 8.0);
}

#[test]
fn mixed_values() {
    let mut h = open();
    let a = add_text(&mut h, 800.0, Some(8.0));
    let b = add_text(&mut h, 2000.0, Some(20.0));
    select(&mut h, &[a, b]);
    assert_eq!(row_value(&h), "Mixed");
    open_popover(&mut h);
    type_into(&mut h, "Blur", "12");
    assert_eq!(shadow(&h, a).unwrap().blur, 12.0);
    assert_eq!(shadow(&h, b).unwrap().blur, 12.0);
    assert_eq!(row_value(&h), "8 / 8 · 12");
}

#[test]
fn remove_the_shadow() {
    let mut h = open();
    let a = add_text(&mut h, 800.0, Some(8.0));
    let b = add_text(&mut h, 2000.0, None);
    select(&mut h, &[a, b]);
    assert_eq!(row_value(&h), "Mixed", "one has none");
    h.get_by_role_and_label(BUTTON, "Remove shadow").click();
    h.run();
    assert_eq!((shadow(&h, a), shadow(&h, b)), (None, None));
    assert!(has(&h, "+ Add a shadow"));
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(shadow(&h, a), Some(Shadow::DEFAULT));
}

#[test]
fn no_shadow_on_a_group_or_an_instance() {
    let mut h = open();
    let a = add_text(&mut h, 800.0, Some(8.0));
    let b = add_text(&mut h, 2000.0, None);
    select(&mut h, &[a, b]);
    ws_mut(&mut h).group_selection(1.0);
    h.run();
    assert!(!has(&h, "Shadow") && !has(&h, "+ Add a shadow"));
    assert_eq!(shadow(&h, a), Some(Shadow::DEFAULT));
    // An instance neither.
    ws_mut(&mut h).convert_to_symbol(2.0).expect("a symbol");
    h.run();
    assert!(!has(&h, "Shadow") && !has(&h, "+ Add a shadow"));
}

#[test]
fn sections_of_an_image() {
    let mut h = open();
    let mut png = Vec::new();
    image::RgbaImage::from_pixel(8, 8, image::Rgba([0, 0, 255, 255]))
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .unwrap();
    let p = &mut ws_mut(&mut h).project;
    let (asset, _) = p.add_asset(
        "picture",
        tp_core::AssetKind::Raster,
        png.into(),
        Size::new(8.0, 8.0),
    );
    let image = p.add(Object::new(
        ObjectId(0),
        ShapeKind::Image { asset },
        Frame::new(Point::new(600.0, 600.0), Size::new(400.0, 400.0), 0.0),
    ));
    select(&mut h, &[image]);
    assert!(has(&h, "+ Add a shadow"));
    assert!(!has(&h, "Fill color") && !has(&h, "Stroke color"));
    h.get_by_role_and_label(BUTTON, "+ Add a shadow").click();
    h.run();
    assert_eq!(shadow(&h, image), Some(Shadow::DEFAULT));
    assert_eq!(row_value(&h), "8 / 8 · 8");
}

#[test]
fn escape_closes_the_popover() {
    let mut h = open();
    let text = add_text(&mut h, 800.0, Some(8.0));
    select(&mut h, &[text]);
    open_popover(&mut h);
    assert!(has(&h, "Shadow opacity") && has(&h, "Offset X") && has(&h, "Offset Y"));
    h.key_press(Key::Escape);
    h.run();
    assert!(!popover_open(&h));
    assert_eq!(ws(&h).selection, vec![text], "the selection stays");
}

#[test]
fn a_palette_click_links_the_shadow_color() {
    let mut h = open();
    let text = add_text(&mut h, 800.0, Some(8.0));
    let night = Rgba::rgb(20, 24, 60);
    let p = &mut ws_mut(&mut h).project;
    let (swatch, _) = p.add_swatch(night, "Color");
    p.rename_swatch(swatch, "Night");
    select(&mut h, &[text]);
    open_popover(&mut h);
    h.get_by_label("Night").click();
    h.run();
    let s = shadow(&h, text).unwrap();
    assert_eq!((s.color, s.swatch), (night, Some(swatch)));
    assert!(has(&h, "Linked to Night"));
    // Another color unlinks it.
    type_into(&mut h, "Hex color", "#FF0000");
    let s = shadow(&h, text).unwrap();
    assert_eq!((s.color, s.swatch), (Rgba::rgb(255, 0, 0), None));
}

// --- Canvas ------------------------------------------------------------------

/// A white square at (500, 500) with a hard black shadow offset 150 / 150,
/// selected and drawn at 100 %.
fn shadowed_square(h: &mut H) -> ObjectId {
    let shadow = Shadow {
        opacity: 1.0,
        offset: DocVec::new(150.0, 150.0),
        blur: 0.0,
        ..Shadow::DEFAULT
    };
    let id = add_rect(h, Point::new(500.0, 500.0), 100.0, Some(shadow));
    common::settle_renders(h);
    id
}

#[test]
fn clicking_the_shadow_selects_nothing() {
    let mut h = open();
    let id = shadowed_square(&mut h);
    ws_mut(&mut h).selection.clear();
    h.run();
    // The shadow alone, then the square.
    let at = screen(&h, Point::new(650.0, 650.0));
    click_at(&mut h, at);
    assert!(ws(&h).selection.is_empty());
    let at = screen(&h, Point::new(500.0, 500.0));
    click_at(&mut h, at);
    assert_eq!(ws(&h).selection, vec![id]);
}

#[test]
fn bounds_and_snapping_ignore_the_shadow() {
    let mut h = open();
    let id = shadowed_square(&mut h);
    select(&mut h, &[id]);
    let frame = selection_frame(&ws(&h).selected_objects()).expect("bounds");
    assert_eq!(frame.bounding_box(), obj(&h, id).frame.bounding_box());
    ws_mut(&mut h).aids.grid = false;
    let snapper = tp_app::snap::Snapper::new(ws(&h), &[], None);
    // The shadow's right edge (700) is no target; the square's (550) is.
    let axis = tp_core::Axis::Vertical;
    assert!(snapper.snap_axis(axis, &[699.0], (0.0, 1.0), 2.0).is_none());
    assert!(snapper.snap_axis(axis, &[549.0], (0.0, 1.0), 2.0).is_some());
}

/// The canvas at 100 % zoom, centered on `center`.
fn zoom_to_100(h: &mut H, center: Point) {
    ws_mut(h).viewport = Some(Viewport {
        center,
        zoom: 1.0,
        fitted: false,
    });
    h.run();
    common::settle_renders(h);
}

/// The export of the active texture at full size.
fn export(h: &H) -> tp_render::Pixmap {
    let ws = ws(h);
    let size = ws.project.surface().size as u32;
    let mut fonts = tp_text::FontLibrary::bundled();
    let options = tp_render::RenderOptions {
        size,
        background: None,
    };
    tp_render::render(
        &ws.project,
        ws.project.active_surface,
        options,
        &mut fonts,
        &mut |_, _| true,
    )
    .expect("not cancelled")
}

/// A blurred text shadow on a texture, at 100 %: the canvas's shadow layer
/// gives the export's pixels around the text (the canvas draws it as is).
#[test]
fn same_in_the_export() {
    let mut h = open();
    let text = add_text(&mut h, 1200.0, Some(12.0));
    let center = obj(&h, text).frame.center;
    zoom_to_100(&mut h, center);
    let pixmap = export(&h);
    let o = obj(&h, text);
    let b = o.frame.bounding_box() + DocVec::new(8.0, 8.0);
    let mut compared = 0;
    for y in (b.y0 as i32 - 20..b.y1 as i32 + 20).step_by(3) {
        for x in (b.x0 as i32 - 20..b.x1 as i32 + 20).step_by(3) {
            let at = Point::new(f64::from(x) + 0.5, f64::from(y) + 0.5);
            // Shadow-only pixels: outside the text's frame.
            if o.frame.bounding_box().inflate(4.0, 4.0).contains(at) {
                continue;
            }
            let canvas = ws(&h)
                .shadows
                .color_at(text, o.frame.center, at)
                .expect("a layer");
            let exported = pixmap.pixel(x as u32, y as u32).unwrap().demultiply();
            assert!(
                (i32::from(canvas.a) - i32::from(exported.alpha())).abs() <= 3,
                "at {at:?}: canvas {canvas:?}, export {exported:?}"
            );
            compared += usize::from(canvas.a > 0);
        }
    }
    assert!(compared > 20, "{compared} shadow pixels compared");
}

/// Same, from the canvas drawn by the GPU: the screen pixels of a
/// square's shadow-only area at 100 % against the export over the
/// artboard's color.
#[test]
#[ignore = "needs a GPU; run manually"]
fn same_in_the_export_on_screen() {
    let mut h = common::wgpu_harness_with(Prefs::default(), Vec2::new(1440.0, 900.0));
    common::create_project(&mut h);
    let shadow = Shadow {
        opacity: 1.0,
        blur: 12.0,
        ..Shadow::DEFAULT
    };
    let text = add_rect(&mut h, Point::new(1200.0, 1600.0), 200.0, Some(shadow));
    let center = obj(&h, text).frame.center;
    zoom_to_100(&mut h, center);
    let image = h.render().expect("render");
    let pixmap = export(&h);
    let artboard = tp_ui::tokens::canvas::ARTBOARD;
    let o = obj(&h, text);
    let frame = o.frame.bounding_box();
    let b = frame + DocVec::new(8.0, 8.0);
    let ppp = h.ctx.pixels_per_point();
    let mut compared = 0;
    for y in (b.y0 as i32 - 20..b.y1 as i32 + 20).step_by(3) {
        for x in (b.x0 as i32 - 20..b.x1 as i32 + 20).step_by(3) {
            let p = Point::new(f64::from(x) + 0.5, f64::from(y) + 0.5);
            let e = pixmap.pixel(x as u32, y as u32).unwrap().demultiply();
            // Shadow-only pixels, where the shadow shows.
            if frame.inflate(1.0, 1.0).contains(p) || e.alpha() < 40 {
                continue;
            }
            let s = screen(&h, p) * ppp;
            let shown = image.get_pixel(s.x as u32, s.y as u32);
            let a = f32::from(e.alpha()) / 255.0;
            let over = |c: u8, bg: u8| f32::from(c) * a + f32::from(bg) * (1.0 - a);
            let expected = [
                over(e.red(), artboard.r()),
                over(e.green(), artboard.g()),
                over(e.blue(), artboard.b()),
            ];
            for (c, want) in shown.0.iter().zip(expected) {
                assert!(
                    (f32::from(*c) - want).abs() <= 8.0,
                    "at {p:?}: screen {shown:?}, export over the artboard {expected:?}"
                );
            }
            compared += 1;
        }
    }
    assert!(compared > 20, "{compared} shadow pixels compared");
}
