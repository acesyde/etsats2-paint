//! Interaction tests for design-system widgets.

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_ui::icons;
use tp_ui::tokens::size;
use tp_ui::widgets::{IconButton, MenuRow, ToolButton};

/// Harness whose first frame only installs the theme (fonts registered in a
/// frame become usable from the next one).
fn themed<S>(
    mut app: impl FnMut(&mut egui::Ui, &mut S) + 'static,
    state: S,
) -> Harness<'static, S> {
    let mut installed = false;
    Harness::new_ui_state(
        move |ui, state| {
            if !installed {
                tp_ui::theme::install(ui.ctx(), tp_ui::ThemeSettings::default());
                installed = true;
                ui.ctx().request_repaint();
                return;
            }
            app(ui, state);
        },
        state,
    )
}

#[test]
fn tool_button_click_active_and_hit_area() {
    let mut h = themed(
        |ui, clicks: &mut i32| {
            if ui
                .add(ToolButton::new(icons::RECTANGLE, "Rectangle").active(true))
                .clicked()
            {
                *clicks += 1;
            }
            ui.add_enabled(false, ToolButton::new(icons::PEN, "Pen"));
        },
        0,
    );
    h.run();

    let rect = h.get_by_label("Rectangle").rect();
    assert!(rect.width() >= size::HIT_MIN && rect.height() >= size::HIT_MIN);
    // The active state is exposed to assistive technology, not only drawn.
    assert_eq!(
        h.get_by_label("Rectangle").accesskit_node().toggled(),
        Some(egui::accesskit::Toggled::True)
    );

    h.get_by_label("Rectangle").click();
    h.run();
    h.get_by_label("Pen").click();
    h.run();
    assert_eq!(*h.state(), 1, "disabled tool button must ignore clicks");
    assert!(h.get_by_label("Pen").accesskit_node().is_disabled());
}

#[test]
fn icon_button_hit_area_is_at_least_minimum() {
    let mut h = themed(
        |ui, _: &mut ()| {
            ui.add(IconButton::new(icons::CLOSE, "Close"));
        },
        (),
    );
    h.run();
    let rect = h.get_by_label("Close").rect();
    assert!(rect.width() >= size::HIT_MIN && rect.height() >= size::HIT_MIN);
}

#[test]
fn disabled_menu_row_ignores_clicks() {
    let mut h = themed(
        |ui, clicked: &mut bool| {
            *clicked |= ui
                .add_enabled(false, MenuRow::new("Undo").shortcut(Some("Ctrl+Z")))
                .clicked();
        },
        false,
    );
    h.run();
    h.get_by_label("Undo").click();
    h.run();
    assert!(!*h.state());
}

#[test]
fn hover_shows_tooltip_with_shortcut() {
    let mut h = themed(
        |ui, _: &mut ()| {
            ui.add(ToolButton::new(icons::ELLIPSE, "Ellipse").shortcut(Some("E")));
        },
        (),
    );
    h.run();
    h.get_by_label("Ellipse").hover();
    // Tooltips appear after the hover delay.
    for _ in 0..60 {
        h.step();
    }
    h.run();
    h.get_by_label("E");
}
