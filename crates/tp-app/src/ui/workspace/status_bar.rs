//! Status bar. In the Workshop: zoom, pointer position, the active texture's
//! template (shown, opacity, key G), the Snapping, Grid and Guides toggles
//! and the save state; in the Project and Brand spaces, the save state only.
//! In every space, while the local MCP server is on, its state.

use egui::{Align, Layout, RichText, Sense, Ui, WidgetInfo, WidgetType};
use tp_i18n::tr;
use tp_mcp::ServerState;
use tp_ui::icons;
use tp_ui::tokens::{color, space, typography};
use tp_ui::widgets::{SWITCH_SIZE, ThinSlider, paint_switch};

use crate::commands::CommandId;
use crate::layout::Space;
use crate::mcp::McpView;
use crate::prefs::ViewAids;
use crate::ui::CommandUi;
use crate::workspace::{SaveState, Workspace};

/// Width of the template opacity slider.
const OPACITY_WIDTH: f32 = 72.0;

fn item(ui: &mut Ui, text: impl Into<String>) {
    ui.label(
        RichText::new(text.into())
            .monospace()
            .size(typography::CAPTION)
            .color(color::TEXT_MUTED),
    );
}

/// An item as wide as `widest` at least, so the controls after it stay in
/// place while its value changes (the pointer position as it moves).
fn fixed_item(ui: &mut Ui, text: String, widest: &str) {
    let font = egui::FontId::monospace(typography::CAPTION);
    let width = ui
        .painter()
        .layout_no_wrap(widest.to_owned(), font, color::TEXT_SECONDARY)
        .size()
        .x;
    ui.allocate_ui_with_layout(
        egui::vec2(width, ui.available_height()),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.set_min_width(width);
            item(ui, text);
        },
    );
}

/// A short vertical hairline between groups.
fn divider(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, 14.0), Sense::hover());
    ui.painter().vline(
        rect.center().x,
        rect.y_range(),
        egui::Stroke::new(1.0, color::BORDER),
    );
}

/// Formats a zoom percentage compactly (`12.5%`, `67%`).
pub fn format_zoom(percent: f32) -> String {
    if percent < 10.0 {
        format!("{}%", tp_i18n::localize_number(&format!("{percent:.1}")))
    } else {
        format!("{percent:.0}%")
    }
}

/// Draws the status bar; returns true when the MCP item was clicked.
pub fn show(
    ui: &mut Ui,
    cmds: &mut CommandUi<'_>,
    ws: &mut Workspace,
    aids: ViewAids,
    mcp: &McpView,
) -> bool {
    let mut mcp_clicked = false;
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM;
        if ws.space == Space::Workshop {
            view(ui, ws);
            ui.add_space(space::SM);
            divider(ui);
            ui.add_space(space::SM);
            template(ui, cmds, ws);
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            save_state(ui, ws);
            if mcp.state != ServerState::Off {
                ui.add_space(space::SM);
                divider(ui);
                ui.add_space(space::SM);
                mcp_clicked = mcp_item(ui, mcp);
            }
            if ws.space == Space::Workshop {
                ui.add_space(space::SM);
                divider(ui);
                ui.add_space(space::SM);
                // Right to left: Guides, Grid, Snapping read left to right.
                toggle(
                    ui,
                    cmds,
                    CommandId::ShowGuides,
                    "status-guides",
                    aids.guides,
                );
                toggle(ui, cmds, CommandId::ShowGrid, "status-grid", aids.grid);
                toggle(
                    ui,
                    cmds,
                    CommandId::Snapping,
                    "status-snapping",
                    aids.snapping,
                );
            }
        });
    });
    mcp_clicked
}

/// The MCP server's state: "MCP", "MCP · connected" or "MCP · 3 changes",
/// with an icon; its tooltip gives the address. Returns whether it was
/// clicked.
fn mcp_item(ui: &mut Ui, mcp: &McpView) -> bool {
    let (icon, text, tint) = match mcp.state {
        ServerState::Listening if mcp.recent_writes > 0 => (
            icons::MCP_CONNECTED,
            tr!("status-mcp-changes", count = mcp.recent_writes),
            color::TEXT_PRIMARY,
        ),
        ServerState::Listening if mcp.connected => (
            icons::MCP_CONNECTED,
            tr("status-mcp-connected"),
            color::TEXT_PRIMARY,
        ),
        ServerState::Listening => (icons::MCP, tr("status-mcp"), color::TEXT_MUTED),
        ServerState::PortInUse | ServerState::Failed(_) => {
            (icons::WARNING, tr("status-mcp"), color::WARNING)
        }
        ServerState::Off => return false,
    };
    let response = ui
        .add(
            egui::Button::new((
                RichText::new(&text)
                    .size(typography::CAPTION)
                    .color(color::TEXT_MUTED),
                icons::rich(icon).size(typography::BODY).color(tint),
            ))
            .frame(false)
            .sense(Sense::click()),
        )
        .on_hover_text(tr!("status-mcp-tooltip", url = tp_mcp::url(mcp.port)));
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, &text));
    response.clicked()
}

/// Zoom and pointer position in texture pixels.
fn view(ui: &mut Ui, ws: &Workspace) {
    let zoom = ws.viewport.map_or_else(
        || "—".to_owned(),
        |view| format_zoom((view.zoom * 100.0) as f32),
    );
    fixed_item(
        ui,
        tr!("status-zoom", zoom = zoom),
        &tr!("status-zoom", zoom = "6400%"),
    );
    ui.add_space(space::SM);
    divider(ui);
    ui.add_space(space::SM);

    let ppp = ui.ctx().pixels_per_point();
    let pointer = ui.ctx().pointer_hover_pos();
    let size = ws.project.surface().size;
    let position = match (ws.viewport, ws.canvas_rect, pointer) {
        (Some(view), Some(canvas), Some(pos)) if canvas.contains(pos) => {
            let p = view.map(canvas, ppp).to_doc(pos);
            (p.x >= 0.0 && p.y >= 0.0 && p.x < size.width && p.y < size.height)
                .then(|| (p.x.floor() as i64, p.y.floor() as i64))
        }
        _ => None,
    };
    let position = position.map_or_else(
        || "x —  y —".to_owned(),
        |(x, y)| format!("x {x}  y {y} px"),
    );
    fixed_item(ui, position, "x 00000  y 00000 px");
}

/// The active texture's template: shown or hidden (View › Show Template,
/// G), and its opacity. Neither is recorded in the undo history. Disabled
/// without a template, for a texture not in its vehicle's version, or
/// while a symbol is edited.
fn template(ui: &mut Ui, cmds: &mut CommandUi<'_>, ws: &mut Workspace) {
    let enabled = cmds.enabled(CommandId::ShowTemplate);
    // The active texture's, also while one of its symbols is edited (the
    // settings are then disabled, but keep their values).
    let template = ws
        .project
        .surfaces
        .get(ws.project.active_surface)
        .and_then(|s| s.template.clone());
    let visible = template.as_ref().is_some_and(|t| t.visible);
    ui.add_enabled_ui(enabled, |ui| {
        template_switch(ui, cmds, visible);
        let mut percent = template
            .as_ref()
            .map_or(0.0, |t| (t.opacity * 100.0).round());
        let name = tr("vehicle-panel-opacity-name");
        let slider = ui.add(
            ThinSlider::new(&mut percent, 0.0..=100.0, &name)
                .width(OPACITY_WIDTH)
                .step(1.0),
        );
        if slider.changed()
            && let Some(t) = ws.project.surface_mut().template.as_mut()
        {
            t.opacity = (percent / 100.0).clamp(0.0, 1.0);
            ws.settings_changed = true;
        }
        // No value without a template.
        let value = match template {
            Some(_) => format!("{} %", tp_i18n::format_number(f64::from(percent), 0)),
            None => "—".to_owned(),
        };
        item(ui, value);
        if let Some(key) = cmds.shortcuts.command(CommandId::ShowTemplate) {
            ui.label(
                RichText::new(key)
                    .monospace()
                    .size(typography::CAPTION)
                    .color(color::TEXT_DISABLED),
            );
        }
    });
}

/// Show Template as a switch (its knob's side shows the state) and a
/// label, announced as a checkbox.
fn template_switch(ui: &mut Ui, cmds: &mut CommandUi<'_>, on: bool) {
    let id = CommandId::ShowTemplate;
    let label = tr("status-template");
    let enabled = ui.is_enabled() && cmds.enabled(id);
    let font = egui::FontId::proportional(typography::CAPTION);
    let ink = if !enabled {
        color::TEXT_DISABLED
    } else if on {
        color::TEXT_PRIMARY
    } else {
        color::TEXT_MUTED
    };
    let galley = ui.painter().layout_no_wrap(label.clone(), font, ink);
    let gap = space::SM;
    let size = egui::vec2(
        SWITCH_SIZE.x + gap + galley.size().x,
        tp_ui::tokens::size::HIT_MIN,
    );
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let switch = egui::Rect::from_min_size(
        egui::pos2(rect.left(), rect.center().y - SWITCH_SIZE.y / 2.0),
        SWITCH_SIZE,
    );
    paint_switch(ui, switch, on, enabled);
    ui.painter().galley(
        egui::pos2(
            switch.right() + gap,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        ink,
    );
    tp_ui::widgets::paint_focus_ring(ui, rect, &response, tp_ui::tokens::radius::SM);
    let response = response.on_hover_text(cmds.shortcuts.command(id).unwrap_or_default());
    response.widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, enabled, on, &label));
    if enabled && response.clicked() {
        cmds.push(id);
    }
}

/// A toggle running `id`: a switch icon (its knob's side shows the state)
/// and a label, announced as a checkbox.
fn toggle(ui: &mut Ui, cmds: &mut CommandUi<'_>, id: CommandId, label: &str, on: bool) {
    let label = tr(label);
    let (icon, ink) = if on {
        (icons::TOGGLE_ON, color::TEXT_PRIMARY)
    } else {
        (icons::TOGGLE_OFF, color::TEXT_MUTED)
    };
    let enabled = ui.is_enabled() && cmds.enabled(id);
    let response = ui
        .add_enabled(
            enabled,
            egui::Button::new((
                icons::rich(icon).size(typography::BODY + 3.0).color(ink),
                RichText::new(&label).size(typography::CAPTION).color(ink),
            ))
            .frame(false)
            .sense(Sense::click()),
        )
        .on_hover_text(cmds.shortcuts.command(id).unwrap_or_default());
    response.widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, enabled, on, &label));
    if response.clicked() {
        cmds.push(id);
    }
}

/// "Saved" / "Unsaved changes" / "Saving…", always with its icon.
fn save_state(ui: &mut Ui, ws: &Workspace) {
    let (icon, text, tint) = match ws.save_state() {
        SaveState::Saved => (icons::SAVED, tr("status-saved"), color::SUCCESS),
        SaveState::Unsaved => (icons::UNSAVED, tr("status-unsaved"), color::WARNING),
        SaveState::Saving => (icons::SAVE, tr("status-saving"), color::TEXT_SECONDARY),
    };
    ui.label(
        RichText::new(text)
            .size(typography::CAPTION)
            .color(color::TEXT_MUTED),
    );
    ui.label(icons::rich(icon).size(typography::BODY).color(tint));
}

#[cfg(test)]
mod tests {
    use super::format_zoom;

    #[test]
    fn zoom_formatting() {
        assert_eq!(format_zoom(8.0), "8.0%");
        assert_eq!(format_zoom(66.66), "67%");
    }
}
