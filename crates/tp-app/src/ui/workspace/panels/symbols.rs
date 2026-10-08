//! Symbols panel: the project's symbols with their number of instances,
//! Place and Edit, and Rename / Duplicate / Delete. Rows can be dragged
//! onto the canvas.

use egui::{Align, Key, Layout, RichText, Sense, TextEdit, Ui, Vec2, WidgetInfo, WidgetType};
use tp_core::document::SymbolId;
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::{color, radius, space};
use tp_ui::widgets::{IconButton, MenuRow, primary_button, secondary_button};

use super::PanelEnv;
use crate::commands::CommandId;
use crate::ui::CommandUi;

const ROW_HEIGHT: f32 = 28.0;

/// "1 instance", "3 instances".
pub fn instances_text(count: usize) -> String {
    tr!("symbols-instances", count = count)
}

pub fn show(ui: &mut Ui, cmds: &mut CommandUi<'_>, env: &mut PanelEnv<'_>) {
    let symbols: Vec<(SymbolId, String)> = env
        .ws
        .project
        .symbols
        .iter()
        .map(|s| (s.id, s.name.clone()))
        .collect();
    if symbols.is_empty() {
        ui.label(
            RichText::new(tr("symbols-empty"))
                .small()
                .color(color::TEXT_DISABLED),
        );
        ui.vertical_centered(|ui| {
            let convert = ui.add_enabled(
                cmds.enabled(CommandId::ConvertToSymbol),
                secondary_button(&tr("cmd-convert-to-symbol")),
            );
            if convert.clicked() {
                cmds.push(CommandId::ConvertToSymbol);
            }
        });
        return;
    }
    for (id, name) in &symbols {
        row(ui, env, *id, name);
    }
    confirm_delete(ui, env);
}

fn row(ui: &mut Ui, env: &mut PanelEnv<'_>, id: SymbolId, name: &str) {
    let count = env.ws.project.instance_count(id);
    let renaming = env
        .ws
        .panels
        .renaming_symbol
        .as_ref()
        .is_some_and(|(r, _)| *r == id);
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), ROW_HEIGHT),
        Sense::click_and_drag(),
    );
    let label = tr!("symbols-item", name = name);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Other, true, &label));
    if !env.ws.is_editing_symbol() {
        response.dnd_set_drag_payload(id);
    }
    let editing = env.ws.project.editing_symbol == Some(id);
    if editing {
        ui.painter()
            .rect_filled(rect, radius::SM, color::ACCENT_SUBTLE);
    } else if response.hovered() {
        ui.painter().rect_filled(rect, radius::SM, color::SURFACE_2);
    }
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(Vec2::new(space::XS, 0.0)))
            .layout(Layout::left_to_right(Align::Center)),
    );
    child.label(icons::rich(icons::SYMBOL).color(color::TEXT_SECONDARY));
    if renaming {
        rename_field(&mut child, env, id);
    } else {
        child.add(egui::Label::new(name).truncate());
        child.label(
            RichText::new(instances_text(count))
                .small()
                .color(color::TEXT_SECONDARY),
        );
    }
    child.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = space::XXS;
        let edit = tr!("symbols-edit", name = name);
        if ui.add(IconButton::new(icons::RENAME, &edit)).clicked() {
            env.ws.edit_symbol(id, env.now);
        }
        let place = tr!("symbols-place", name = name);
        let enabled = !env.ws.is_editing_symbol();
        let reason = tr("reason-editing-symbol");
        let button = IconButton::new(icons::ADD, &place).disabled_reason(&reason);
        if ui.add_enabled(enabled, button).clicked() {
            env.ws.place_symbol(id, None, env.now);
        }
    });
    response.context_menu(|ui| {
        if ui.add(MenuRow::new(&tr("symbols-rename"))).clicked() {
            env.ws.panels.renaming_symbol = Some((id, name.to_owned()));
            ui.close();
        }
        if ui.add(MenuRow::new(&tr("symbols-duplicate"))).clicked() {
            env.ws.duplicate_symbol(id, env.now);
            ui.close();
        }
        ui.separator();
        if ui.add(MenuRow::new(&tr("symbols-delete"))).clicked() {
            if count == 0 {
                env.ws.delete_symbol(id, env.now);
            } else {
                env.ws.panels.confirm_delete_symbol = Some(id);
            }
            ui.close();
        }
    });
}

/// "Delete <name>? Its N instances become groups." with Delete / Cancel.
fn confirm_delete(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    let Some(id) = env.ws.panels.confirm_delete_symbol else {
        return;
    };
    let Some(name) = env.ws.project.symbol(id).map(|s| s.name.clone()) else {
        env.ws.panels.confirm_delete_symbol = None;
        return;
    };
    let count = env.ws.project.instance_count(id);
    ui.add_space(space::XS);
    let text = tr!(
        "symbols-delete-confirm",
        name = name.as_str(),
        count = count
    );
    ui.label(RichText::new(&text).small().color(color::WARNING))
        .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
    ui.horizontal(|ui| {
        if ui.add(primary_button(&tr("symbols-delete"))).clicked() {
            env.ws.panels.confirm_delete_symbol = None;
            env.ws.delete_symbol(id, env.now);
        }
        if ui.add(secondary_button(&tr("button-cancel"))).clicked() {
            env.ws.panels.confirm_delete_symbol = None;
        }
    });
}

fn rename_field(ui: &mut Ui, env: &mut PanelEnv<'_>, id: SymbolId) {
    let Some((_, mut buffer)) = env.ws.panels.renaming_symbol.clone() else {
        return;
    };
    let edit = ui.add(
        TextEdit::singleline(&mut buffer)
            .desired_width(140.0)
            .id_salt(("rename_symbol", id.0)),
    );
    edit.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, tr("symbols-name")));
    if !edit.has_focus() && !edit.lost_focus() {
        edit.request_focus();
    }
    if ui.input(|i| i.key_pressed(Key::Escape)) {
        env.ws.panels.renaming_symbol = None;
    } else if edit.lost_focus() {
        env.ws.panels.renaming_symbol = None;
        env.ws.rename_symbol(id, &buffer, env.now);
    } else {
        env.ws.panels.renaming_symbol = Some((id, buffer));
    }
}
