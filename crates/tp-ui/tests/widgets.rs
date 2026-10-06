//! Interaction tests for design-system widgets.

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_ui::icons;
use tp_ui::tokens::size;
use tp_ui::widgets::{IconButton, MenuRow, ToolButton};

/// Harness whose first frame only installs the theme (fonts registered in a
/// frame become usable from the next one).
pub fn themed<S>(
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

mod editing_widgets {
    use egui::accesskit::Role;
    use egui::{Event, Key, Modifiers, PointerButton, Pos2, Vec2};
    use egui_kittest::kittest::Queryable;
    use tp_ui::widgets::{
        ColorSwatch, FieldEvent, FillOrStroke, FillStrokeSwatches, Hsv, NumericField, SwatchColor,
        hue_slider, sv_square,
    };

    use super::themed;

    #[derive(Default)]
    struct FieldState {
        value: f64,
        events: Vec<FieldEvent>,
    }

    fn field_harness() -> egui_kittest::Harness<'static, FieldState> {
        themed(
            |ui, state: &mut FieldState| {
                let event = NumericField::new("W", "Width", Some(state.value))
                    .suffix("px")
                    .range(1.0..=10_000.0)
                    .show(ui);
                if event != FieldEvent::None {
                    state.events.push(event);
                    if let FieldEvent::Live(v) | FieldEvent::Commit(v) = event {
                        state.value = v;
                    }
                }
            },
            FieldState {
                value: 400.0,
                events: Vec::new(),
            },
        )
    }

    #[test]
    fn typing_and_enter_commits() {
        let mut h = field_harness();
        h.run();
        let field = h.get_by_role_and_label(Role::TextInput, "Width");
        field.focus();
        h.run();
        h.key_press_modifiers(Modifiers::COMMAND, Key::A);
        h.get_by_role_and_label(Role::TextInput, "Width")
            .type_text("600");
        h.run();
        h.key_press(Key::Enter);
        h.run();
        assert_eq!(h.state().events.last(), Some(&FieldEvent::Commit(600.0)));
        assert_eq!(h.state().value, 600.0);
    }

    #[test]
    fn escape_reverts() {
        let mut h = field_harness();
        h.run();
        h.get_by_role_and_label(Role::TextInput, "Width").focus();
        h.run();
        h.get_by_role_and_label(Role::TextInput, "Width")
            .type_text("999");
        h.run();
        h.key_press(Key::Escape);
        h.run();
        assert_eq!(h.state().events.last(), Some(&FieldEvent::Revert));
        assert_eq!(h.state().value, 400.0);
    }

    #[test]
    fn scrubbing_the_label_reports_live_then_commit() {
        let mut h = field_harness();
        h.run();
        let label = h.get_by_label("W").rect().center();
        h.event(Event::PointerMoved(label));
        h.event(Event::PointerButton {
            pos: label,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        });
        h.step();
        for i in 1..=5 {
            h.event(Event::PointerMoved(label + Vec2::new(i as f32 * 10.0, 0.0)));
            h.step();
        }
        let end = label + Vec2::new(50.0, 0.0);
        h.event(Event::PointerButton {
            pos: end,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        });
        h.run();
        let events = &h.state().events;
        assert!(events.iter().any(|e| matches!(e, FieldEvent::Live(_))));
        match events.last() {
            Some(FieldEvent::Commit(v)) => assert!(*v > 400.0, "{v}"),
            other => panic!("expected commit, got {other:?}"),
        }
    }

    #[test]
    fn mixed_value_shows_placeholder() {
        let mut h = themed(
            |ui, _: &mut ()| {
                NumericField::new("R", "Rotation", None).show(ui);
            },
            (),
        );
        h.run();
        let node = h.get_by_role_and_label(Role::TextInput, "Rotation");
        assert_eq!(node.value().unwrap_or_default(), "");
    }

    #[test]
    fn swatches_report_clicks() {
        let mut h = themed(
            |ui, clicked: &mut Vec<String>| {
                if ColorSwatch::new(SwatchColor::None, "No stroke")
                    .show(ui)
                    .clicked()
                {
                    clicked.push("none".into());
                }
                let pair = FillStrokeSwatches {
                    fill: SwatchColor::Solid(egui::Color32::RED),
                    stroke: SwatchColor::Mixed,
                    active: FillOrStroke::Fill,
                };
                if let Some(which) = pair.show(ui) {
                    clicked.push(format!("{which:?}"));
                }
            },
            Vec::new(),
        );
        h.run();
        h.get_by_label("No stroke").click();
        h.run();
        h.get_by_label("Stroke").click();
        h.run();
        assert_eq!(h.state(), &vec!["none".to_owned(), "Stroke".to_owned()]);
    }

    #[test]
    fn picker_drag_changes_values() {
        let mut h = themed(
            |ui, hsv: &mut (Hsv, bool)| {
                ui.set_width(200.0);
                if sv_square(ui, &mut hsv.0, 120.0).changed() {
                    hsv.1 = true;
                }
                hue_slider(ui, &mut hsv.0);
            },
            (Hsv::new(0.0, 0.0, 0.0, 1.0), false),
        );
        h.run();
        let square = h.get_by_label("Saturation and value").rect();
        let target = Pos2::new(square.right() - 1.0, square.top() + 1.0);
        h.event(Event::PointerMoved(target));
        h.event(Event::PointerButton {
            pos: target,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        });
        h.event(Event::PointerButton {
            pos: target,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        });
        h.run();
        let (hsv, changed) = *h.state();
        assert!(changed);
        assert!(hsv.s > 0.95 && hsv.v > 0.95, "{hsv:?}");
        let hue = h.get_by_label("Hue").rect();
        let mid = Pos2::new(hue.center().x, hue.center().y);
        h.event(Event::PointerMoved(mid));
        h.event(Event::PointerButton {
            pos: mid,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        });
        h.event(Event::PointerButton {
            pos: mid,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        });
        h.run();
        assert!((h.state().0.h - 0.5).abs() < 0.02, "{:?}", h.state().0);
    }

    #[test]
    fn hsv_to_color() {
        assert_eq!(
            Hsv::new(0.0, 1.0, 1.0, 1.0).to_color32(),
            egui::Color32::RED
        );
        assert_eq!(
            Hsv::new(1.0 / 3.0, 1.0, 1.0, 1.0).to_color32(),
            egui::Color32::GREEN
        );
        assert_eq!(
            Hsv::new(0.0, 0.0, 1.0, 1.0).to_color32(),
            egui::Color32::WHITE
        );
    }
}
