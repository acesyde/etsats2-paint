//! The Workshop's left panel: the Textures, Layers and Resources tabs.

use egui::{Frame, Margin, Panel, ScrollArea, Ui};
use tp_i18n::tr;
use tp_ui::tokens::{color, size, space};
use tp_ui::widgets::SegmentedControl;

use super::panels::{self, PanelEnv};
use crate::commands::CommandId;
use crate::layout::{LeftTab, WorkspaceLayout};
use crate::ui::CommandUi;

/// The panel, resizable by its right edge within bounds; its width is
/// written back to `layout`.
pub fn show(
    ui: &mut Ui,
    cmds: &mut CommandUi<'_>,
    layout: &mut WorkspaceLayout,
    env: &mut PanelEnv<'_>,
) {
    let range = size::LEFT_PANEL_MIN..=size::LEFT_PANEL_MAX;
    let frame = Frame::new()
        .fill(color::SURFACE_1)
        .inner_margin(Margin::same(space::SM as i8 + 2));
    let tab = layout.left_tab;
    let panel = Panel::left(egui::Id::new(("left_panel", layout.generation)))
        .resizable(true)
        .default_size(layout.left_width)
        .size_range(range.clone())
        .frame(frame)
        .show(ui, |ui| {
            tabs(ui, cmds, tab);
            ui.add_space(space::SM + 2.0);
            if tab == LeftTab::Resources {
                resources_footer(ui);
            }
            ScrollArea::vertical()
                .id_salt(("left_panel_scroll", tab))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = space::XS + 2.0;
                    match tab {
                        LeftTab::Textures => panels::vehicle::show(ui, cmds, env),
                        LeftTab::Layers => panels::layers::show(ui, cmds, env),
                        LeftTab::Resources => resources(ui, cmds, env),
                    }
                });
        });
    let width = panel.response.rect.width().round();
    if (width - layout.left_width).abs() >= 1.0 {
        layout.left_width = width.clamp(*range.start(), *range.end());
    }
}

/// The tab strip; clicking a tab runs its View command.
fn tabs(ui: &mut Ui, cmds: &mut CommandUi<'_>, shown: LeftTab) {
    let labels = LeftTab::ALL.map(|tab| tr(tab.label()));
    let mut control = SegmentedControl::new();
    for (tab, label) in LeftTab::ALL.into_iter().zip(&labels) {
        let shortcut = cmds.shortcuts.command(CommandId::ShowLeftTab(tab));
        control = control.segment(tab, "", label, shortcut);
    }
    // Three equal tabs across the panel.
    if let Some(tab) = control.fill().show(ui, shown) {
        cmds.push(CommandId::ShowLeftTab(tab));
    }
}

/// Side of a swatch of the Resources tab's palette, in points.
const SWATCH: f32 = 28.0;

/// The heading of a Resources section: its title and number of elements in
/// small capitals (as the inspector's sections), its actions at the right.
fn heading(ui: &mut Ui, title: &str, count: usize, actions: impl FnOnce(&mut Ui)) {
    ui.add_space(space::SM);
    ui.horizontal(|ui| {
        ui.set_min_height(size::HIT_MIN);
        panels::section_heading(ui, &format!("{title} · {count}"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = space::XXS;
            actions(ui);
        });
    });
}

/// The Resources tab's footer, pinned under its list: how its elements are
/// placed, and where they are managed.
fn resources_footer(ui: &mut Ui) {
    let margin = space::SM + 2.0;
    egui::Panel::bottom("resources_footer")
        .resizable(false)
        .show_separator_line(false)
        .frame(Frame::new().inner_margin(Margin {
            top: (space::MD) as i8,
            ..Margin::ZERO
        }))
        .show(ui, |ui| {
            // A hairline across the whole panel, over its margins.
            let rect = ui.max_rect();
            let top = rect.top() - space::MD;
            ui.painter().hline(
                (rect.left() - margin)..=(rect.right() + margin),
                top,
                egui::Stroke::new(1.0, color::BORDER),
            );
            ui.add(
                egui::Label::new(
                    egui::RichText::new(tr("resources-footer"))
                        .size(tp_ui::tokens::typography::CONTROL)
                        .color(color::TEXT_MUTED),
                )
                .wrap(),
            );
        });
}

/// The Resources tab: the project's Palette, Symbols, Styles and Images,
/// compact, then Import from Library…. Symbols and images are dragged onto
/// the canvas to place them.
fn resources(ui: &mut Ui, cmds: &mut CommandUi<'_>, env: &mut PanelEnv<'_>) {
    let project = &env.ws.project;
    let palette = project.palette.len();
    let symbols = project.symbols.len();
    let styles = project.graphic_styles.len() + project.text_styles.len();
    let images = panels::assets::listed(project).len();

    heading(ui, &tr("colors-palette"), palette, |ui| {
        panels::colors::add_to_palette_button(ui, env);
    });
    if palette == 0 {
        panels::colors::palette_empty_hint(ui);
    } else {
        panels::colors::palette_swatches(ui, env, SWATCH, false);
        panels::colors::linked_swatch_label(ui, env);
    }

    heading(ui, &tr("panel-symbols"), symbols, |_| {});
    panels::symbols::show(ui, cmds, env);

    heading(ui, &tr("panel-styles"), styles, |_| {});
    panels::styles::show(ui, env);

    heading(ui, &tr("resources-images"), images, |_| {});
    if !panels::assets::list(ui, env) {
        panels::assets::drop_zone(ui, cmds);
    }

    ui.add_space(space::LG);
    ui.vertical_centered(|ui| panels::import_from_library_button(ui, cmds));
}
