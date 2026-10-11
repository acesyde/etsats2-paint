//! Headless tests of a rectangular texture (8192 × 1024) in the Workshop:
//! the artboard, rulers, grid, snapping, placement, thumbnails and size
//! labels follow its real shape.

mod common;

use egui::epaint::{ClippedShape, Shape};
use egui::{Pos2, Rect};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use tp_app::AppState;
use tp_app::layout::LeftTab;
use tp_app::workspace::Workspace;
use tp_core::Axis;
use tp_core::kurbo::{Point, Size};
use tp_vehicles::sample::{self, SampleTexture};

type H = Harness<'static, AppState>;

const LINER: &str = "Liner";

/// A project painting a trailer whose base is 8192 × 1024, with its
/// 1024² mudflaps, shown in the Workshop with the Textures tab.
fn open() -> H {
    let tex = |id, name, w, h| SampleTexture {
        id,
        name,
        size: tp_vehicles::TextureSize::new(w, h),
        layout: 1,
    };
    let package = tp_vehicles::Package::read(&sample::package_with(
        "community.jdoe.liner",
        LINER,
        "1.0.0",
        "trailer",
        &[tex("base", "Base", 8192, 1024)],
        &[tex("mudflaps", "Mudflaps", 1024, 1024)],
    ))
    .unwrap();
    let textures = tp_app::vehicle_project::default_textures(&package.manifest);
    let mut project =
        tp_app::vehicle_project::fleet_project("Fleet", &[(&package, &textures)]).unwrap();
    for surface in &mut project.surfaces {
        if let Some(t) = surface.template.as_mut() {
            t.visible = false;
        }
    }
    let mut h = common::harness();
    common::open_project(&mut h, project);
    h.run();
    common::show_tab(&mut h, LeftTab::Textures);
    common::settle_renders(&mut h);
    assert_eq!(ws(&h).project.surface().size, Size::new(8192.0, 1024.0));
    h
}

fn ws(h: &H) -> &Workspace {
    h.state().workspace().expect("project open")
}

fn ws_mut(h: &mut H) -> &mut Workspace {
    h.state_mut().workspace_mut().expect("project open")
}

fn shapes(h: &H) -> impl Iterator<Item = &Shape> {
    h.output()
        .shapes
        .iter()
        .map(|ClippedShape { shape, .. }| shape)
}

/// workspace-layout "Rectangular artboard".
#[test]
fn rectangular_artboard_is_centered_and_whole() {
    let h = open();
    let canvas = h.get_by_label("Canvas").rect();
    let artboard = ws(&h).artboard_rect(1.0).unwrap();
    assert!(canvas.contains_rect(artboard), "{artboard:?} in {canvas:?}");
    assert!(
        (artboard.width() / artboard.height() - 8.0).abs() < 0.01,
        "{artboard:?}"
    );
    assert!((artboard.center().x - canvas.center().x).abs() < 0.5);
    // Limited by the canvas's width, not its height.
    assert!(artboard.width() > canvas.width() * 0.8, "{artboard:?}");
}

/// workspace-layout "Rectangular artboard": the rulers run from 0 to 8192
/// across and from 0 to 1024 down.
#[test]
fn rulers_end_at_the_texture_sides() {
    let h = open();
    let map = ws(&h).screen_map(1.0).unwrap();
    let artboard = ws(&h).artboard_rect(1.0).unwrap();
    assert_eq!(map.to_doc(artboard.max), Point::new(8192.0, 1024.0));
    // The last labels within the artboard's extent on each ruler.
    let last = |label: &str, along: fn(Pos2) -> f32, end: f32| {
        let ruler = h.get_by_label(label).rect();
        shapes(&h)
            .filter_map(|s| match s {
                Shape::Text(t) if ruler.contains(t.pos) => Some(t),
                _ => None,
            })
            .filter(|t| along(t.pos) <= end + 1.0)
            .filter_map(|t| t.galley.text().parse::<f64>().ok())
            .fold(f64::MIN, f64::max)
    };
    let across = last("Horizontal ruler", |p| p.x, artboard.right());
    assert!((7000.0..=8192.0).contains(&across), "{across}");
    // Vertical labels read bottom to top, below their tick.
    let down = last("Vertical ruler", |p| p.y, artboard.bottom() + 40.0);
    assert!((800.0..=1024.0).contains(&down), "{down}");
}

/// Grid lines stop at the texture's height: one at y = 1024, none beyond.
#[test]
fn grid_stops_at_the_bottom_edge() {
    let mut h = open();
    h.state_mut().prefs.view_aids.grid = true;
    h.run();
    let map = ws(&h).screen_map(1.0).unwrap();
    let bottom = map.to_screen(Point::new(0.0, 1024.0)).y;
    let right = map.to_screen(Point::new(8192.0, 0.0)).x;
    let grid = [
        tp_ui::tokens::canvas::GRID_MINOR,
        tp_ui::tokens::canvas::GRID_MAJOR,
    ];
    let lines: Vec<[Pos2; 2]> = shapes(&h)
        .filter_map(|s| match s {
            Shape::LineSegment { points, stroke } if grid.contains(&stroke.color) => Some(*points),
            _ => None,
        })
        .collect();
    let rows: Vec<f32> = lines
        .iter()
        .filter(|[a, b]| (a.y - b.y).abs() < 0.01)
        .map(|[a, _]| a.y)
        .collect();
    assert!(!rows.is_empty());
    let lowest = rows.iter().copied().fold(f32::MIN, f32::max);
    assert!((lowest - bottom).abs() < 0.5, "{lowest} vs {bottom}");
    // Columns run down to 1024 and reach 8192.
    let columns: Vec<[Pos2; 2]> = lines
        .into_iter()
        .filter(|[a, b]| (a.x - b.x).abs() < 0.01)
        .collect();
    assert!(columns.iter().all(|[a, b]| a.y.max(b.y) <= bottom + 0.5));
    let rightmost = columns.iter().map(|[a, _]| a.x).fold(f32::MIN, f32::max);
    assert!((rightmost - right).abs() < 0.5, "{rightmost} vs {right}");
}

/// Snapping to the artboard's bottom edge, at 1024, and to its center.
#[test]
fn snapping_to_the_bottom_edge() {
    let h = open();
    let snapper = tp_app::snap::Snapper::new(ws(&h), &[], None);
    let (d, _) = snapper
        .snap_axis(Axis::Horizontal, &[1020.0], (100.0, 200.0), 8.0)
        .expect("snaps");
    assert_eq!(1020.0 + d, 1024.0);
    let (p, _) = snapper.snap_point(Point::new(4090.0, 515.0), 8.0);
    assert_eq!(p, Point::new(4096.0, 512.0));
    // Not to where a square of 8192 would end.
    assert!(
        snapper
            .snap_axis(Axis::Horizontal, &[8190.0], (100.0, 200.0), 8.0)
            .is_none()
    );
}

/// Place puts an image in the middle of the visible part.
#[test]
fn place_in_the_middle_of_the_view() {
    let mut h = open();
    let canvas = h.get_by_label("Canvas").rect();
    let files = vec![tp_app::import::read_bytes(
        "logo.png",
        tp_app::import::solid_png(400, 200, [20, 60, 220, 255]),
    )];
    let now = h.ctx.input(|i| i.time);
    ws_mut(&mut h).place_files(files, None, now);
    h.run();
    let ws = ws(&h);
    let placed = ws.project.surface().get(ws.selection[0]).unwrap();
    let map = ws.screen_map(1.0).unwrap();
    let middle = map.to_doc(canvas.center());
    assert!(placed.frame.center.distance(middle) < 1.0, "{middle:?}");
    assert!(placed.frame.size.height <= 512.0, "fits half the height");
}

/// Thumbnails keep the texture's shape: the Textures tab draws the base's
/// eight times wider than tall.
#[test]
fn textures_tab_thumbnail_keeps_the_shape() {
    let h = open();
    assert_eq!(ws(&h).thumbnails.size_of(0), Some([128, 16]));
    let id = ws(&h).thumbnails.texture(0).unwrap().id();
    let rect = shapes(&h)
        .filter_map(|s| match s {
            Shape::Mesh(m) if m.texture_id == id => Some(
                m.vertices
                    .iter()
                    .fold(Rect::NOTHING, |r, v| r.union(Rect::from_pos(v.pos))),
            ),
            _ => None,
        })
        .next()
        .expect("the base's thumbnail is drawn");
    assert!(
        (rect.width() / rect.height() - 8.0).abs() < 0.01,
        "{rect:?}"
    );
}

/// Size labels: the breadcrumb and the Textures tab row read
/// "8192 × 1024".
#[test]
fn size_labels_read_width_by_height() {
    let mut h = open();
    let crumb = format!("{LINER} › Main textures › Base 8192 × 1024");
    assert_eq!(h.query_all_by_label(&crumb).count(), 1, "{crumb}");
    let texts: Vec<String> = shapes(&h)
        .filter_map(|s| match s {
            Shape::Text(t) => Some(t.galley.text().to_owned()),
            _ => None,
        })
        .collect();
    assert!(texts.iter().any(|t| t == "8192 × 1024"), "Textures tab row");
    assert!(texts.iter().any(|t| t == "1024²"), "the mudflaps' row");
    ws_mut(&mut h).set_active_surface(1);
    h.run();
    let crumb = format!("{LINER} › Accessories › Mudflaps 1024²");
    assert_eq!(h.query_all_by_label(&crumb).count(), 1, "{crumb}");
}

/// Texture rows: the sizes are right-aligned in one slot, so the state
/// markers before them line up, whatever the size's width ("8192 × 1024"
/// and "1024²").
#[test]
fn textures_tab_state_markers_line_up() {
    let h = open();
    let markers: Vec<Rect> = h
        .query_all_by_label("Empty")
        .map(|n| n.rect())
        .filter(|r| r.left() < 400.0)
        .collect();
    assert_eq!(markers.len(), 2, "{markers:?}");
    assert!(
        (markers[0].center().x - markers[1].center().x).abs() < 0.5,
        "{markers:?}"
    );
    let right_edges: Vec<f32> = shapes(&h)
        .filter_map(|s| match s {
            Shape::Text(t) if ["8192 × 1024", "1024²"].contains(&t.galley.text()) => {
                Some(t.pos.x + t.galley.rect.right())
            }
            _ => None,
        })
        .collect();
    assert_eq!(right_edges.len(), 2, "{right_edges:?}");
    assert!(
        (right_edges[0] - right_edges[1]).abs() < 0.5,
        "{right_edges:?}"
    );
    // The marker keeps clear of the widest size.
    assert!(markers[0].right() < right_edges[0] - 60.0, "{markers:?}");
}
