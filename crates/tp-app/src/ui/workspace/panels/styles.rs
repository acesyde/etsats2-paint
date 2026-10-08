//! Styles panel: the project's graphic styles (fill, stroke, opacity) and
//! text styles (character settings). Clicking a style applies it to the
//! selection; the styles the selection follows are marked.

use egui::{Align, Key, Layout, Rect, RichText, Sense, TextEdit, Ui, Vec2, WidgetInfo, WidgetType};
use tp_core::Look;
use tp_core::document::{Paint, ShapeKind, StyleId};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::{color, radius, space};
use tp_ui::widgets::{ColorSwatch, IconButton, MenuRow, SwatchColor};

use super::PanelEnv;
use super::properties::gradient_preview;

const ROW_HEIGHT: f32 = 24.0;

fn swatch_of(paint: &Paint) -> SwatchColor {
    match paint {
        Paint::Solid(c) => {
            SwatchColor::Solid(egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a))
        }
        Paint::Gradient(g) => SwatchColor::Gradient(gradient_preview(g)),
    }
}

/// Why New Style from Selection is disabled, if it is.
fn new_style_blocked(env: &PanelEnv<'_>, text: bool) -> Option<String> {
    let ok = match env.ws.selection.as_slice() {
        [id] => env.ws.project.surface().get(*id).is_some_and(|o| {
            if text {
                o.text.is_some()
            } else {
                !matches!(
                    o.kind,
                    ShapeKind::Group | ShapeKind::Image { .. } | ShapeKind::Instance { .. }
                )
            }
        }),
        _ => false,
    };
    (!ok).then(|| {
        tr(if text {
            "styles-need-text"
        } else {
            "styles-need-shape"
        })
    })
}

fn section_header(ui: &mut Ui, env: &mut PanelEnv<'_>, title: &str, text: bool) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(title).small().color(color::TEXT_SECONDARY));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let blocked = new_style_blocked(env, text);
            let name = tr(if text {
                "styles-new-text"
            } else {
                "styles-new-graphic"
            });
            let button = IconButton::new(icons::ADD, &name)
                .disabled_reason(blocked.as_deref().unwrap_or_default());
            if ui.add_enabled(blocked.is_none(), button).clicked() {
                if text {
                    env.ws.new_text_style(env.now);
                } else {
                    env.ws.new_graphic_style(env.now);
                }
            }
        });
    });
}

pub fn show(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    let followed = env.ws.project.styles_of(&env.ws.selection);

    section_header(ui, env, &tr("styles-graphic"), false);
    let graphic = env.ws.project.graphic_styles.clone();
    if graphic.is_empty() {
        hint(ui, &tr("styles-graphic-empty"));
    }
    for style in &graphic {
        let label = tr!("styles-graphic-item", name = style.name.as_str());
        let row = style_row(
            ui,
            env,
            style.id,
            &style.name,
            &label,
            followed.contains(&style.id),
            |ui, rect| {
                let chip = look_chip(ui, rect, &style.look, &style.name);
                ui.painter().text(
                    egui::pos2(chip.right() + space::SM, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    &style.name,
                    egui::TextStyle::Body.resolve(ui.style()),
                    color::TEXT_PRIMARY,
                );
            },
        );
        if row {
            env.ws.apply_style(style.id, env.now);
        }
    }

    ui.add_space(space::SM);
    section_header(ui, env, &tr("styles-text"), true);
    let text = env.ws.project.text_styles.clone();
    if text.is_empty() {
        hint(ui, &tr("styles-text-empty"));
    }
    for style in &text {
        let label = tr!("styles-text-item", name = style.name.as_str());
        let s = &style.style;
        let preview = env
            .ws
            .text
            .styled_preview(&style.name, &s.family, s.weight, s.italic);
        let size = format!("{} px", s.size);
        let row = style_row(
            ui,
            env,
            style.id,
            &style.name,
            &label,
            followed.contains(&style.id),
            |ui, rect| {
                let chip = look_chip(ui, rect, &style.look, &style.name);
                let name_rect = Rect::from_min_max(
                    egui::pos2(chip.right(), rect.top()),
                    egui::pos2(rect.right() - 56.0, rect.bottom()),
                );
                super::character::draw_preview_mesh(ui, &preview, name_rect);
                ui.painter().text(
                    egui::pos2(rect.right() - space::SM, rect.center().y),
                    egui::Align2::RIGHT_CENTER,
                    &size,
                    egui::TextStyle::Small.resolve(ui.style()),
                    color::TEXT_SECONDARY,
                );
            },
        );
        if row {
            env.ws.apply_style(style.id, env.now);
        }
    }
}

/// A style's fill, ringed by its stroke when it has one, at the left of a
/// row; returns the chip's rectangle.
fn look_chip(ui: &mut Ui, rect: Rect, look: &Look, name: &str) -> Rect {
    let chip = Rect::from_center_size(
        egui::pos2(rect.left() + space::SM + 9.0, rect.center().y),
        Vec2::splat(18.0),
    );
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(chip));
    ColorSwatch::new(swatch_of(&look.fill), name)
        .size(14.0)
        .show(&mut child);
    if let Some(s) = &look.stroke {
        let c = s.paint.first_color();
        ui.painter().rect_stroke(
            chip.shrink(1.0),
            radius::SM,
            egui::Stroke::new(
                2.0,
                egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a),
            ),
            egui::StrokeKind::Inside,
        );
    }
    chip
}

fn hint(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).small().color(color::TEXT_DISABLED));
}

/// A clickable style row drawn by `draw`, with its context menu and inline
/// rename. Returns whether it was clicked (apply).
fn style_row(
    ui: &mut Ui,
    env: &mut PanelEnv<'_>,
    id: StyleId,
    name: &str,
    label: &str,
    followed: bool,
    draw: impl FnOnce(&mut Ui, Rect),
) -> bool {
    if env
        .ws
        .panels
        .renaming_style
        .as_ref()
        .is_some_and(|(r, _)| *r == id)
    {
        rename_field(ui, env, id);
        return false;
    }
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), ROW_HEIGHT), Sense::click());
    response.widget_info(|| WidgetInfo::selected(WidgetType::Button, true, followed, label));
    if followed {
        ui.painter()
            .rect_filled(rect, radius::SM, color::ACCENT_SUBTLE);
    } else if response.hovered() {
        ui.painter().rect_filled(rect, radius::SM, color::SURFACE_2);
    }
    draw(ui, rect);
    if followed {
        // Marked by an icon too, not by color alone.
        ui.painter().text(
            egui::pos2(rect.right() - 2.0, rect.top() + 2.0),
            egui::Align2::RIGHT_TOP,
            icons::LINKED,
            tp_ui::icons::font(11.0),
            color::ACCENT,
        );
    }
    let has_one = env.ws.selection.len() == 1;
    response.context_menu(|ui| {
        if ui.add(MenuRow::new(&tr("styles-rename"))).clicked() {
            env.ws.panels.renaming_style = Some((id, name.to_owned()));
            ui.close();
        }
        if ui
            .add_enabled(has_one, MenuRow::new(&tr("styles-redefine")))
            .clicked()
        {
            env.ws.redefine_style(id, env.now);
            ui.close();
        }
        if ui.add(MenuRow::new(&tr("styles-select-users"))).clicked() {
            env.ws.select_style_users(id);
            ui.close();
        }
        ui.separator();
        if ui.add(MenuRow::new(&tr("styles-delete"))).clicked() {
            env.ws.delete_style(id, env.now);
            ui.close();
        }
    });
    response.clicked()
}

fn rename_field(ui: &mut Ui, env: &mut PanelEnv<'_>, id: StyleId) {
    let Some((_, mut buffer)) = env.ws.panels.renaming_style.clone() else {
        return;
    };
    let edit = ui.add(
        TextEdit::singleline(&mut buffer)
            .desired_width(f32::INFINITY)
            .id_salt(("rename_style", id.0)),
    );
    edit.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, tr("styles-name")));
    if !edit.has_focus() && !edit.lost_focus() {
        edit.request_focus();
    }
    if ui.input(|i| i.key_pressed(Key::Escape)) {
        env.ws.panels.renaming_style = None;
    } else if edit.lost_focus() {
        env.ws.panels.renaming_style = None;
        env.ws.rename_style(id, &buffer, env.now);
    } else {
        env.ws.panels.renaming_style = Some((id, buffer));
    }
}
