//! Application menu bar (File, Edit, Object, Layer, View, Vehicle, Export, Help).

use egui::Ui;
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::{color, size, space};

use super::CommandUi;
use crate::layout::WorkspaceLayout;
use crate::menus::{self, Entry};

/// Menu titles, in order (the titles of [`menus::menus`]).
pub const MENUS: [&str; 8] = [
    "menu-file",
    "menu-edit",
    "menu-object",
    "menu-layer",
    "menu-view",
    "menu-vehicle",
    "menu-export",
    "menu-help",
];

/// Draws the menu bar row. `layout` is `None` on the home screen.
pub fn show(
    ui: &mut Ui,
    cmds: &mut CommandUi<'_>,
    layout: Option<&WorkspaceLayout>,
    aids: crate::prefs::ViewAids,
) {
    ui.set_height(size::MENU_BAR_HEIGHT);
    egui::MenuBar::new().ui(ui, |ui| {
        ui.label(
            icons::rich(icons::VEHICLE)
                .color(color::TEXT_PRIMARY)
                .size(size::ICON_LG),
        )
        .on_hover_text(crate::paths::APP_NAME);
        ui.add_space(space::XS);

        for menu in menus::menus() {
            ui.menu_button(tr(menu.title), |ui| {
                ui.set_min_width(menus::MENU_MIN_WIDTH);
                menu_contents(ui, cmds, &menu.entries, layout, aids);
            });
        }
    });
}

/// Draws `entries` of a menu: items, toggles with their check mark,
/// separators and submenus.
fn menu_contents(
    ui: &mut Ui,
    cmds: &mut CommandUi<'_>,
    entries: &[Entry],
    layout: Option<&WorkspaceLayout>,
    aids: crate::prefs::ViewAids,
) {
    for entry in entries {
        match entry {
            Entry::Item(id) => {
                cmds.menu_item(ui, *id);
            }
            Entry::Toggle(id) => {
                let checked = menus::is_checked(*id, &cmds.edit, layout, aids);
                cmds.menu_toggle(ui, *id, checked);
            }
            Entry::Separator => {
                ui.separator();
            }
            Entry::Submenu {
                title,
                min_width,
                entries,
            } => {
                ui.menu_button(tr(title), |ui| {
                    ui.set_min_width(*min_width);
                    menu_contents(ui, cmds, entries, layout, aids);
                });
            }
        }
    }
}
