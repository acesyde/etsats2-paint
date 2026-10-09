//! The images list (Resources tab): imported images and SVGs with
//! thumbnail, size, use count, and Place / Rename / Remove actions. Rows can
//! be dragged onto the canvas. The Brand space shows them as cards with the
//! same thumbnail, actions and rename field.

use egui::{
    Align, Key, Layout, Rect, RichText, Sense, Stroke, StrokeKind, TextEdit, Ui, Vec2, WidgetInfo,
    WidgetType,
};
use tp_core::{Asset, AssetKind};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::{color, radius, space};
use tp_ui::widgets::IconButton;

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

/// "1024 × 512 px", or "Vector" for an SVG.
pub fn size_text(asset: &Asset) -> String {
    match asset.kind {
        AssetKind::Svg => tr("assets-vector"),
        AssetKind::Raster => format!("{:.0} × {:.0} px", asset.size.width, asset.size.height),
    }
}

/// The project's images: every asset but the templates, which are vehicle
/// data rather than imported files (listed only when an image uses the
/// same file too).
pub fn listed(project: &tp_core::Project) -> Vec<std::sync::Arc<Asset>> {
    let templates: Vec<_> = project.template_assets().collect();
    project
        .assets
        .values()
        .filter(|a| {
            let as_template = templates.iter().filter(|t| **t == a.id).count();
            as_template == 0 || project.asset_usage(a.id) > as_template
        })
        .cloned()
        .collect()
}

/// The image rows (thumbnail, name, size, uses; Place, Rename, Remove;
/// dragged onto the canvas to place). Returns whether there was any.
pub fn list(ui: &mut Ui, env: &mut PanelEnv<'_>) -> bool {
    let assets = listed(&env.ws.project);
    for asset in &assets {
        row(ui, env, asset);
    }
    !assets.is_empty()
}

/// The drop zone shown by the Resources tab while the project has no image:
/// "Drop a logo here or Import…". Import… runs File › Place…; files dropped
/// from the operating system anywhere off the canvas are placed in the
/// middle of the view (see `AppState::after_frame`), the zone shows where.
pub fn drop_zone(ui: &mut Ui, cmds: &mut CommandUi<'_>) {
    let dragging = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
    let response = egui::Frame::new()
        .inner_margin(egui::Margin::same(space::MD as i8))
        .fill(if dragging {
            color::SURFACE_2
        } else {
            egui::Color32::TRANSPARENT
        })
        .corner_radius(radius::LG)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let text = tr("assets-drop-zone");
            let import = tr("assets-import");
            let font = egui::TextStyle::Body.resolve(ui.style());
            let width = |t: &str| {
                ui.painter()
                    .layout_no_wrap(t.to_owned(), font.clone(), color::TEXT_SECONDARY)
                    .size()
                    .x
            };
            let total = width(&text) + ui.spacing().item_spacing.x + width(&import);
            ui.horizontal_wrapped(|ui| {
                ui.add_space(((ui.available_width() - total) / 2.0).max(0.0));
                ui.label(RichText::new(&text).color(color::TEXT_SECONDARY));
                if ui.link(&import).clicked() {
                    cmds.push(CommandId::Place);
                }
            });
        })
        .response;
    // A dashed outline: a drop target, not a control.
    let rect = response.rect.shrink(0.5);
    let outline = if dragging {
        color::TEXT_SECONDARY
    } else {
        color::BORDER_STRONG
    };
    let corners = [
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
        rect.left_top(),
    ];
    ui.painter().extend(egui::Shape::dashed_line(
        &corners,
        Stroke::new(1.0, outline),
        4.0,
        3.0,
    ));
}

/// Objects using `asset` (a template using the same file is not counted).
pub fn uses_of(project: &tp_core::Project, asset: &Asset) -> usize {
    project.asset_usage(asset.id) - project.template_assets().filter(|a| *a == asset.id).count()
}

/// Paints the thumbnail of `asset` fitted in `rect`, on a raised square.
pub fn paint_thumbnail(ui: &mut Ui, env: &mut PanelEnv<'_>, asset: &Asset, rect: Rect) {
    ui.painter().rect(
        rect,
        radius::SM,
        color::SURFACE_3,
        Stroke::new(1.0, color::BORDER),
        StrokeKind::Inside,
    );
    let ctx = ui.ctx().clone();
    let screen = rect.width().max(rect.height()) * ctx.pixels_per_point();
    if let Some(texture) = env.ws.images.texture(&ctx, asset, screen) {
        let aspect = (asset.size.width / asset.size.height) as f32;
        let inner = rect.shrink(3.0);
        let size = if aspect >= inner.aspect_ratio() {
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
}

/// The asset's Remove, Rename and Place buttons, right to left. Remove is
/// enabled only while the asset is unused. Returns whether Place was
/// clicked (the image is then placed).
pub fn actions(ui: &mut Ui, env: &mut PanelEnv<'_>, asset: &Asset, uses: usize) -> bool {
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
    let place = ui
        .add(IconButton::new(icons::ADD, &tr("assets-place")))
        .clicked();
    if place {
        env.ws.place_asset(asset.id, None, env.now);
    }
    place
}

/// Whether `asset` is being renamed.
pub fn is_renaming(env: &PanelEnv<'_>, asset: &Asset) -> bool {
    env.ws
        .panels
        .renaming_asset
        .as_ref()
        .is_some_and(|(id, _)| *id == asset.id)
}

fn row(ui: &mut Ui, env: &mut PanelEnv<'_>, asset: &Asset) {
    let uses = uses_of(&env.ws.project, asset);
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
    paint_thumbnail(ui, env, asset, thumb);

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
    let renaming = is_renaming(env, asset);
    child.with_layout(Layout::right_to_left(Align::Center), |ui| {
        actions(ui, env, asset, uses);
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

/// The inline Rename field of the asset being renamed.
pub fn rename_field(ui: &mut Ui, env: &mut PanelEnv<'_>) {
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
