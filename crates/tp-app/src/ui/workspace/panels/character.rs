//! Character section of the Properties panel and the font family picker.

use egui::{
    Align2, Area, Frame, Id, Key, Order, Pos2, RichText, ScrollArea, Sense, Stroke, StrokeKind,
    TextEdit, Ui, Vec2, WidgetInfo, WidgetType,
};
use tp_core::document::{CharStyle, TextAlign};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::{color, radius, space};
use tp_ui::widgets::{IconButton, NumericField};

use super::{PanelEnv, apply_field};
use crate::text_engine::PREVIEW_SIZE;
use crate::workspace::{FontPicker, Workspace};

/// The value shared by all items, or `None` when they differ (or are empty).
fn common<T: PartialEq + Clone>(mut values: impl Iterator<Item = T>) -> Option<T> {
    let first = values.next()?;
    values.all(|v| v == first).then_some(first)
}

fn weight_name(weight: u16) -> String {
    tr(weight_id(weight))
}

fn weight_id(weight: u16) -> &'static str {
    match weight {
        0..=149 => "weight-thin",
        150..=249 => "weight-extra-light",
        250..=349 => "weight-light",
        350..=449 => "weight-regular",
        450..=549 => "weight-medium",
        550..=649 => "weight-semi-bold",
        650..=749 => "weight-bold",
        750..=849 => "weight-extra-bold",
        _ => "weight-black",
    }
}

/// Styles being edited: the selected texts', or the style for new texts.
fn styles(ws: &Workspace) -> Vec<CharStyle> {
    if ws.selection.is_empty() {
        return vec![ws.text_style.clone()];
    }
    ws.selected_texts()
        .into_iter()
        .filter_map(|o| o.text.map(|t| t.style))
        .collect()
}

/// Applies a discrete change (dropdown, toggle) as one undo step.
fn set(env: &mut PanelEnv<'_>, label: &'static str, f: impl Fn(&mut CharStyle)) {
    env.ws.set_char_style(label, f);
    env.ws.commit_pending(env.now);
}

pub fn show(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    let styles = styles(env.ws);
    if styles.is_empty() {
        return;
    }
    ui.label(
        RichText::new(tr("char-title"))
            .small()
            .color(color::TEXT_SECONDARY),
    );

    // Family.
    let family = common(styles.iter().map(|s| s.family.clone()));
    let missing = family
        .as_deref()
        .is_some_and(|f| !env.ws.text.fonts.has_family(f));
    let button = family_button(ui, family.as_deref(), missing);
    if button.clicked() {
        env.ws.panels.font_picker = match env.ws.panels.font_picker {
            Some(_) => None,
            None => Some(FontPicker {
                focus_requested: true,
                ..FontPicker::default()
            }),
        };
    }
    if env.ws.panels.font_picker.is_some() {
        font_picker(ui, env, &button);
    }

    // Weight and italic.
    ui.horizontal(|ui| {
        let weight = common(styles.iter().map(|s| s.weight));
        let available = env
            .ws
            .text
            .fonts
            .weights(family.as_deref().unwrap_or(tp_text::FALLBACK_FAMILY));
        let text = weight.map_or(tr("mixed"), |w| format!("{} {w}", weight_name(w)));
        let combo = egui::ComboBox::from_id_salt("font_weight")
            .width(150.0)
            .selected_text(text)
            .show_ui(ui, |ui| {
                for w in available {
                    let label = format!("{} {w}", weight_name(w));
                    if ui.selectable_label(weight == Some(w), label).clicked() {
                        set(env, "undo-change-font-weight", |s| s.weight = w);
                    }
                }
            });
        combo.response.widget_info(|| {
            WidgetInfo::labeled(WidgetType::ComboBox, true, tr("char-font-weight"))
        });
        let italic = common(styles.iter().map(|s| s.italic));
        let on = italic == Some(true);
        if ui
            .add(IconButton::new(icons::ITALIC, &tr("char-italic")).selected(on))
            .clicked()
        {
            set(env, "undo-change-italic", |s| s.italic = !on);
        }
    });

    // Size and alignment.
    ui.horizontal(|ui| {
        let size = common(styles.iter().map(|s| s.size));
        let e = NumericField::new(&tr("char-size"), &tr("char-font-size"), size)
            .suffix("px")
            .range(1.0..=10_000.0)
            .decimals(1)
            .width(56.0)
            .show(ui);
        apply_field(env, e, |ws, v| {
            ws.set_char_style("undo-change-text-size", |s| s.size = v);
        });
        let align = common(styles.iter().map(|s| s.align));
        for (value, icon, name) in [
            (TextAlign::Left, icons::ALIGN_LEFT, "char-align-left"),
            (TextAlign::Center, icons::ALIGN_CENTER, "char-align-center"),
            (TextAlign::Right, icons::ALIGN_RIGHT, "char-align-right"),
        ] {
            if ui
                .add(IconButton::new(icon, &tr(name)).selected(align == Some(value)))
                .clicked()
            {
                set(env, "undo-change-alignment", |s| s.align = value);
            }
        }
    });

    // Letter spacing and line height.
    ui.horizontal(|ui| {
        let tracking = common(styles.iter().map(|s| s.letter_spacing));
        let e = NumericField::new(&tr("char-tracking"), &tr("char-letter-spacing"), tracking)
            .suffix("‰")
            .range(-200.0..=1000.0)
            .width(44.0)
            .show(ui);
        apply_field(env, e, |ws, v| {
            ws.set_char_style("undo-change-letter-spacing", |s| s.letter_spacing = v);
        });
        let leading = common(styles.iter().map(|s| s.line_height));
        let e = NumericField::new(&tr("char-line"), &tr("char-line-height"), leading)
            .suffix("%")
            .range(50.0..=300.0)
            .width(44.0)
            .show(ui);
        apply_field(env, e, |ws, v| {
            ws.set_char_style("undo-change-line-height", |s| s.line_height = v);
        });
    });
}

fn family_button(ui: &mut Ui, family: Option<&str>, missing: bool) -> egui::Response {
    let height = 26.0;
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::click());
    let label = family.map_or_else(|| tr("mixed"), str::to_owned);
    let name = if missing {
        tr!("char-family-missing-name", family = label.as_str())
    } else {
        tr!("char-family-name", family = label.as_str())
    };
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, &name));
    let painter = ui.painter();
    let fill = if response.hovered() {
        color::SURFACE_3
    } else {
        color::SURFACE_2
    };
    painter.rect(
        rect,
        radius::SM,
        fill,
        Stroke::new(1.0, color::BORDER),
        StrokeKind::Inside,
    );
    let mut x = rect.left() + space::SM;
    if missing {
        painter.text(
            Pos2::new(x, rect.center().y),
            Align2::LEFT_CENTER,
            icons::WARNING,
            icons::font(14.0),
            color::WARNING,
        );
        x += 18.0;
    }
    painter.text(
        Pos2::new(x, rect.center().y),
        Align2::LEFT_CENTER,
        &label,
        egui::TextStyle::Body.resolve(ui.style()),
        color::TEXT_PRIMARY,
    );
    painter.text(
        Pos2::new(rect.right() - space::SM, rect.center().y),
        Align2::RIGHT_CENTER,
        icons::EXPANDED,
        icons::font(12.0),
        color::TEXT_SECONDARY,
    );
    if missing {
        response.on_hover_text(tr!("char-font-not-found", family = label.as_str()))
    } else {
        response.on_hover_text(tr("char-font-family"))
    }
}

/// Families matching the picker query (case-insensitive substring).
pub fn filter_families<'a>(families: &'a [String], query: &str) -> Vec<&'a String> {
    let query = query.trim().to_lowercase();
    families
        .iter()
        .filter(|f| query.is_empty() || f.to_lowercase().contains(&query))
        .collect()
}

const ROW_HEIGHT: f32 = 28.0;

fn font_picker(ui: &Ui, env: &mut PanelEnv<'_>, button: &egui::Response) {
    let ctx = ui.ctx().clone();
    let families: Vec<String> = env.ws.text.fonts.families().to_vec();
    let width = button.rect.width().max(220.0);
    let area = Area::new(Id::new("font_picker"))
        .order(Order::Foreground)
        .fixed_pos(button.rect.left_bottom() + Vec2::new(0.0, space::XS))
        .show(&ctx, |ui| {
            Frame::popup(ui.style())
                .show(ui, |ui| {
                    ui.set_width(width);
                    let picker = env.ws.panels.font_picker.as_mut()?;
                    let search = ui.add(
                        TextEdit::singleline(&mut picker.query)
                            .hint_text(format!("{} {}", icons::SEARCH, tr("char-search-fonts")))
                            .desired_width(f32::INFINITY),
                    );
                    search.widget_info(|| {
                        WidgetInfo::labeled(WidgetType::TextEdit, true, tr("char-search-fonts"))
                    });
                    if std::mem::take(&mut picker.focus_requested) {
                        search.request_focus();
                    }
                    if search.changed() {
                        picker.highlighted = 0;
                    }
                    let matches = filter_families(&families, &picker.query);
                    let (up, down, enter, escape) = ui.input(|i| {
                        (
                            i.key_pressed(Key::ArrowUp),
                            i.key_pressed(Key::ArrowDown),
                            i.key_pressed(Key::Enter),
                            i.key_pressed(Key::Escape),
                        )
                    });
                    if down && picker.highlighted + 1 < matches.len() {
                        picker.highlighted += 1;
                    }
                    if up {
                        picker.highlighted = picker.highlighted.saturating_sub(1);
                    }
                    let mut chosen = None;
                    if enter && let Some(f) = matches.get(picker.highlighted) {
                        chosen = Some((*f).clone());
                    }
                    if escape {
                        return Some(None);
                    }
                    if matches.is_empty() {
                        ui.label(
                            RichText::new(tr("char-no-fonts"))
                                .small()
                                .color(color::TEXT_SECONDARY),
                        );
                    }
                    let highlighted = picker.highlighted;
                    let mut rows = Vec::new();
                    ScrollArea::vertical()
                        .max_height(ROW_HEIGHT * 10.0)
                        .show_rows(ui, ROW_HEIGHT, matches.len(), |ui, range| {
                            for i in range {
                                let family = matches[i];
                                let (rect, response) = ui.allocate_exact_size(
                                    Vec2::new(ui.available_width(), ROW_HEIGHT),
                                    Sense::click(),
                                );
                                response.widget_info(|| {
                                    WidgetInfo::selected(
                                        WidgetType::SelectableLabel,
                                        true,
                                        i == highlighted,
                                        family.as_str(),
                                    )
                                });
                                if i == highlighted {
                                    if down || up {
                                        response.scroll_to_me(None);
                                    }
                                    ui.painter().rect_filled(
                                        rect,
                                        radius::SM,
                                        color::ACCENT_SUBTLE,
                                    );
                                } else if response.hovered() {
                                    ui.painter().rect_filled(rect, radius::SM, color::SURFACE_3);
                                }
                                rows.push((family.clone(), rect));
                                if response.clicked() {
                                    chosen = Some(family.clone());
                                }
                            }
                        });
                    for (family, rect) in rows {
                        draw_preview(ui, env.ws, &family, rect);
                    }
                    chosen.map(Some)
                })
                .inner
        });
    let outcome = area.inner;
    let clicked_outside = ctx.input(|i| i.pointer.any_pressed())
        && ctx
            .pointer_interact_pos()
            .is_some_and(|p| !area.response.rect.contains(p) && !button.rect.contains(p));
    match outcome {
        Some(Some(family)) => {
            env.ws.panels.font_picker = None;
            let weights = env.ws.text.fonts.weights(&family);
            set(env, "undo-change-font", |s| {
                s.family.clone_from(&family);
                if !weights.contains(&s.weight) {
                    s.weight = weights
                        .iter()
                        .copied()
                        .min_by_key(|w| (i32::from(*w) - i32::from(s.weight)).abs())
                        .unwrap_or(400);
                }
            });
        }
        Some(None) => env.ws.panels.font_picker = None,
        None if clicked_outside => env.ws.panels.font_picker = None,
        None => {}
    }
}

/// Draws a family name in its own font inside `rect`.
fn draw_preview(ui: &Ui, ws: &mut Workspace, family: &str, rect: egui::Rect) {
    if !ui.is_rect_visible(rect) {
        return;
    }
    let preview = ws.text.preview(family);
    draw_preview_mesh(ui, &preview, rect);
}

/// Draws a laid-out preview at a 15 pt size, left-aligned in `rect`.
pub fn draw_preview_mesh(ui: &Ui, preview: &crate::text_engine::FontPreview, rect: egui::Rect) {
    let target = 15.0;
    let scale = target / PREVIEW_SIZE as f32;
    let origin = Pos2::new(
        rect.left() + space::SM,
        rect.center().y - (preview.baseline as f32 * scale) + target * 0.35,
    );
    let mut mesh = egui::Mesh::default();
    mesh.indices.clone_from(&preview.mesh.indices);
    mesh.vertices = preview
        .mesh
        .vertices
        .iter()
        .map(|[x, y]| egui::epaint::Vertex {
            pos: origin + Vec2::new(x * scale, y * scale),
            uv: egui::epaint::WHITE_UV,
            color: color::TEXT_PRIMARY,
        })
        .collect();
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    painter.add(egui::Shape::mesh(mesh));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_is_case_insensitive() {
        let families: Vec<String> = ["Inter", "Bebas Neue", "Oswald"].map(String::from).to_vec();
        assert_eq!(filter_families(&families, "bebas"), vec!["Bebas Neue"]);
        assert_eq!(filter_families(&families, "").len(), 3);
    }
}
