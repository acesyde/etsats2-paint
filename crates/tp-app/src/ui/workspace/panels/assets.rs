//! Assets panel: imported images and SVGs with thumbnail, size, use count,
//! and Place / Rename / Remove actions. Rows can be dragged onto the canvas.

use egui::{
    Align, Key, Layout, Rect, RichText, Sense, Stroke, StrokeKind, TextEdit, Ui, Vec2, WidgetInfo,
    WidgetType,
};
use tp_core::{Asset, AssetKind};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::{color, radius, space};
use tp_ui::widgets::{EmptyState, IconButton, secondary_button};

use super::PanelEnv;
use crate::commands::CommandId;
use crate::ui::CommandUi;

const THUMB: f32 = 36.0;

/// "1 use", "3 uses".
pub fn uses_text(count: usize) -> String {
    tr!("assets-uses", count = count)
}

/// Why Remove is disabled for a used asset.
pub fn used_by_text(count: usize) -> String {
    tr!("assets-used-by", count = count)
}

fn size_text(asset: &Asset) -> String {
    match asset.kind {
        AssetKind::Svg => tr("assets-vector"),
        AssetKind::Raster => format!("{:.0} × {:.0} px", asset.size.width, asset.size.height),
    }
}

pub fn show(ui: &mut Ui, cmds: &mut CommandUi<'_>, env: &mut PanelEnv<'_>) {
    // Templates are vehicle data, not imported files: they are listed only
    // when an image uses the same file too.
    let project = &env.ws.project;
    let templates: Vec<_> = project.template_assets().collect();
    let assets: Vec<_> = project
        .assets
        .values()
        .filter(|a| {
            let as_template = templates.iter().filter(|t| **t == a.id).count();
            as_template == 0 || project.asset_usage(a.id) > as_template
        })
        .cloned()
        .collect();
    if assets.is_empty() {
        EmptyState::new(icons::ASSETS, &tr("empty-assets"), &tr("empty-assets-hint")).show(ui);
        ui.vertical_centered(|ui| {
            if ui.add(secondary_button(&tr("cmd-place"))).clicked() {
                cmds.push(CommandId::Place);
            }
        });
        return;
    }
    for asset in assets {
        row(ui, env, &asset);
    }
}

fn row(ui: &mut Ui, env: &mut PanelEnv<'_>, asset: &Asset) {
    // Uses by images (a template using the same file is not counted).
    let project = &env.ws.project;
    let uses = project.asset_usage(asset.id)
        - project.template_assets().filter(|a| *a == asset.id).count();
    let height = THUMB + space::XS;
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), height),
        Sense::click_and_drag(),
    );
    let label = tr!("assets-item", name = asset.name.as_str());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Other, true, &label));
    response.dnd_set_drag_payload(asset.id);
    if response.hovered() {
        ui.painter().rect_filled(rect, radius::SM, color::SURFACE_2);
    }

    // Thumbnail, fitted in a square.
    let thumb = Rect::from_min_size(
        rect.left_top() + Vec2::new(0.0, space::XXS),
        Vec2::splat(THUMB),
    );
    ui.painter().rect(
        thumb,
        radius::SM,
        color::SURFACE_3,
        Stroke::new(1.0, color::BORDER),
        StrokeKind::Inside,
    );
    let ctx = ui.ctx().clone();
    let screen = THUMB * ctx.pixels_per_point();
    if let Some(texture) = env.ws.images.texture(&ctx, asset, screen) {
        let aspect = (asset.size.width / asset.size.height) as f32;
        let inner = thumb.shrink(3.0);
        let size = if aspect >= 1.0 {
            Vec2::new(inner.width(), inner.width() / aspect)
        } else {
            Vec2::new(inner.height() * aspect, inner.height())
        };
        let image = Rect::from_center_size(inner.center(), size);
        ui.painter().image(
            texture.id(),
            image,
            Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }

    // Name and details, then actions on the right.
    let text_rect = Rect::from_min_max(
        egui::pos2(thumb.right() + space::SM, rect.top()),
        rect.right_bottom(),
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(text_rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    let renaming = env
        .ws
        .panels
        .renaming_asset
        .as_ref()
        .is_some_and(|(id, _)| *id == asset.id);
    child.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = space::XXS;
        let remove = ui.add_enabled(
            uses == 0,
            IconButton::new(icons::REMOVE, &tr("undo-remove-asset")),
        );
        let remove = if uses > 0 {
            remove.on_disabled_hover_text(used_by_text(uses))
        } else {
            remove
        };
        if remove.clicked() {
            env.ws.remove_asset(asset.id, env.now);
        }
        if ui
            .add(IconButton::new(icons::RENAME, &tr("undo-rename-asset")))
            .clicked()
        {
            env.ws.panels.renaming_asset = Some((asset.id, asset.name.clone()));
        }
        if ui
            .add(IconButton::new(icons::ADD, &tr("assets-place")))
            .clicked()
        {
            env.ws.place_asset(asset.id, None, env.now);
        }
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.add_space(space::XS);
            if renaming {
                rename_field(ui, env);
            } else {
                ui.add(egui::Label::new(&asset.name).truncate());
            }
            ui.label(
                RichText::new(format!("{} · {}", size_text(asset), uses_text(uses)))
                    .small()
                    .color(color::TEXT_SECONDARY),
            );
        });
    });
}

fn rename_field(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    let Some((id, mut buffer)) = env.ws.panels.renaming_asset.clone() else {
        return;
    };
    let edit = ui.add(
        TextEdit::singleline(&mut buffer)
            .desired_width(f32::INFINITY)
            .id_salt(("rename_asset", id.0)),
    );
    edit.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, tr("assets-name")));
    if !edit.has_focus() && !edit.lost_focus() {
        edit.request_focus();
    }
    let escape = ui.input(|i| i.key_pressed(Key::Escape));
    if escape {
        env.ws.panels.renaming_asset = None;
    } else if edit.lost_focus() {
        env.ws.panels.renaming_asset = None;
        env.ws.rename_asset(id, &buffer, env.now);
    } else {
        env.ws.panels.renaming_asset = Some((id, buffer));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn use_counts_read_naturally() {
        assert_eq!(uses_text(1), "1 use");
        assert_eq!(uses_text(2), "2 uses");
        assert_eq!(used_by_text(1), "Used by 1 object");
    }
}
