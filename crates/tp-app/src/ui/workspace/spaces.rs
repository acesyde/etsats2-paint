//! The Project and Brand spaces, using the full width between the top bar
//! and the status bar, and the pieces they share: titles, labels and the
//! "+" buttons of section headers.
//!
//! Project ([`project`]): the fleet's vehicle cards with their textures
//! (clicking one shows it in the Workshop) and the Mod information column.
//! Brand ([`brand`]): the project's palette, styles, symbols and images.

pub mod brand;
pub mod project;

use egui::{Response, RichText, Ui, WidgetInfo, WidgetType};
use tp_ui::icons;
use tp_ui::theme::title_style;
use tp_ui::tokens::{color, radius, space};

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
    super::panels::section_heading(ui, text);
}

/// The label above a value.
fn field_label(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).small().color(color::TEXT_SECONDARY));
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

/// Paints the card frame of `rect` (radius 10): the panel surface with a hairline,
/// raised when hovered; a `marked` card has a strong outline.
fn paint_card(ui: &Ui, rect: egui::Rect, hovered: bool, marked: bool) {
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        radius::CARD,
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
        radius::CARD,
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
