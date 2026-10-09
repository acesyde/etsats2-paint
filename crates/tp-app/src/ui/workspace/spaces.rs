//! The Project and Brand spaces, using the full width between the top bar
//! and the status bar, and the pieces they share: titles, labelled
//! read-only values and the "+" buttons of section headers.
//!
//! Project ([`project`]): the fleet's vehicle cards with their textures
//! (clicking one shows it in the Workshop) and the Mod information column.
//! Brand ([`brand`]): the project's palette, styles, symbols and images.

pub mod brand;
pub mod project;

use egui::{Frame, Margin, Response, RichText, Ui, WidgetInfo, WidgetType};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::theme::title_style;
use tp_ui::tokens::{color, radius, space, typography};

/// Size of a section title of a space ("Palette", "Symbols").
const SECTION_TITLE: f32 = 20.0;

/// A space's title ("Vehicles", "Brand").
fn title(ui: &mut Ui, text: &str) -> Response {
    ui.label(
        RichText::new(text)
            .text_style(title_style())
            .color(color::TEXT_PRIMARY),
    )
}

/// A section's title.
fn section_title(ui: &mut Ui, text: &str) -> Response {
    ui.label(
        RichText::new(text)
            .size(SECTION_TITLE)
            .family(tp_ui::fonts::semibold_family())
            .color(color::TEXT_PRIMARY),
    )
}

/// A column's heading: small semibold capitals ("MOD INFORMATION").
fn caps_heading(ui: &mut Ui, text: &str) {
    let label = ui.label(
        RichText::new(text.to_uppercase())
            .size(typography::CAPTION)
            .family(tp_ui::fonts::semibold_family())
            .extra_letter_spacing(0.6)
            .color(color::TEXT_SECONDARY),
    );
    label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
}

/// The label above a value.
fn field_label(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).small().color(color::TEXT_SECONDARY));
}

/// A value shown read only under its `label`, in a field-like box: it
/// can't be focused nor typed in. An empty value reads "Not set" in the
/// secondary text color.
fn read_only(ui: &mut Ui, label: &str, value: &str, mono: bool, multiline: bool) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = space::XS;
        field_label(ui, label);
        Frame::new()
            .fill(color::SURFACE_0)
            .stroke(egui::Stroke::new(1.0, color::BORDER))
            .corner_radius(radius::MD)
            .inner_margin(Margin::symmetric(10, 7))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                let empty = value.trim().is_empty();
                let mut text = if empty {
                    RichText::new(tr("project-not-set")).color(color::TEXT_SECONDARY)
                } else {
                    RichText::new(value).color(color::TEXT_PRIMARY)
                };
                if mono && !empty {
                    text = text.monospace();
                }
                let label = egui::Label::new(text).selectable(false);
                ui.add(if multiline {
                    label.wrap()
                } else {
                    label.truncate()
                });
            });
    });
}

/// A "+ <label>" button of a section header, named `name` for assistive
/// technologies; when disabled, its tooltip says why.
fn add_button(ui: &mut Ui, label: &str, name: &str, disabled_reason: Option<&str>) -> Response {
    let enabled = disabled_reason.is_none();
    let button = egui::Button::new((
        icons::rich(icons::ADD).color(color::TEXT_PRIMARY),
        RichText::new(label).color(color::TEXT_PRIMARY),
    ))
    .corner_radius(radius::MD)
    .min_size(egui::vec2(0.0, 30.0));
    let mut response = ui.add_enabled(enabled, button);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, name));
    if let Some(reason) = disabled_reason {
        response = response.on_disabled_hover_text(reason);
    }
    response
}

/// Paints the card frame of `rect`: the panel surface with a hairline,
/// raised when hovered; a `marked` card has a strong outline.
fn paint_card(ui: &Ui, rect: egui::Rect, hovered: bool, marked: bool) {
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        radius::LG,
        if hovered {
            color::SURFACE_2
        } else {
            color::SURFACE_1
        },
    );
    let (width, outline) = if marked {
        (1.5, color::ACCENT_PRIMARY)
    } else if hovered {
        (1.0, color::BORDER_STRONG)
    } else {
        (1.0, color::BORDER)
    };
    painter.rect_stroke(
        rect,
        radius::LG,
        egui::Stroke::new(width, outline),
        egui::StrokeKind::Inside,
    );
}

/// The gap between cards of a grid.
const GRID_GAP: f32 = space::MD;

/// Lays out `count` cells of at least `min_width` in rows filling the
/// available width; `cell` draws cell `i` at the given width.
fn grid(ui: &mut Ui, min_width: f32, count: usize, mut cell: impl FnMut(&mut Ui, usize, f32)) {
    let available = ui.available_width();
    let columns = (((available + GRID_GAP) / (min_width + GRID_GAP)).floor() as usize).max(1);
    let width = (available - GRID_GAP * (columns - 1) as f32) / columns as f32;
    for start in (0..count).step_by(columns) {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = GRID_GAP;
            for i in start..(start + columns).min(count) {
                cell(ui, i, width.floor());
            }
        });
    }
}
