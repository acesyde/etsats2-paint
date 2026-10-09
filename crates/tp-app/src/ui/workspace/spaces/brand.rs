//! The Brand space: the project's palette, graphic styles, text styles,
//! symbols and images, as cards, with the actions of their lists (the
//! menus, rename fields and confirmations are the lists' own), a section
//! index on the left, and Import from Library… in the header.
//!
//! Only the actions that place something or open an edit view show the
//! Workshop: Place (a symbol, an image), Edit (a symbol), Select Users on
//! This Texture, and the images imported with Import… (see
//! `Workspace::place_files`). The others leave the Brand space shown.

use egui::{
    Align, Align2, CentralPanel, CornerRadius, Frame, Id, Layout, Margin, Panel, Rect, Response,
    RichText, ScrollArea, Sense, Stroke, Ui, UiBuilder, Vec2, WidgetInfo, WidgetType,
};
use tp_core::document::{Paint, StyleId};
use tp_core::{Asset, Look};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::{color, radius, space, typography};
use tp_ui::widgets::{IconButton, paint_checkerboard, paint_focus_ring};

use super::super::panels::{self, PanelEnv, assets, colors, styles, symbols};
use crate::commands::CommandId;
use crate::layout::Space;
use crate::state::disabled_reason_for;
use crate::ui::CommandUi;

/// Width of the section index.
const INDEX_WIDTH: f32 = 220.0;
/// Height of a card's text area (title and detail).
const CARD_TEXT: f32 = 56.0;
/// Padding of a card's text area.
const CARD_PAD: Vec2 = Vec2::new(12.0, 10.0);

/// The sections, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Section {
    Palette,
    GraphicStyles,
    TextStyles,
    Symbols,
    Images,
}

impl Section {
    const ALL: [Self; 5] = [
        Self::Palette,
        Self::GraphicStyles,
        Self::TextStyles,
        Self::Symbols,
        Self::Images,
    ];

    fn title(self) -> String {
        tr(match self {
            Self::Palette => "colors-palette",
            Self::GraphicStyles => "styles-graphic",
            Self::TextStyles => "styles-text",
            Self::Symbols => "panel-symbols",
            Self::Images => "resources-images",
        })
    }

    /// Number of elements of the section in the open project.
    fn count(self, project: &tp_core::Project) -> usize {
        match self {
            Self::Palette => project.palette.len(),
            Self::GraphicStyles => project.graphic_styles.len(),
            Self::TextStyles => project.text_styles.len(),
            Self::Symbols => project.symbols.len(),
            Self::Images => assets::listed(project).len(),
        }
    }
}

/// The entry of the index marked as the current one.
fn current_id() -> Id {
    Id::new("brand_current_section")
}

/// The section to scroll into view at the next frame.
fn scroll_id() -> Id {
    Id::new("brand_scroll_to")
}

pub fn show(ui: &mut Ui, cmds: &mut CommandUi<'_>, env: &mut PanelEnv<'_>) {
    Panel::left("brand_index")
        .exact_size(INDEX_WIDTH)
        .resizable(false)
        .frame(
            Frame::new()
                .fill(color::SURFACE_1)
                .inner_margin(Margin::symmetric(10, space::LG as i8)),
        )
        .show(ui, |ui| index(ui, env));
    CentralPanel::no_frame()
        .frame(Frame::new().fill(color::SURFACE_0))
        .show(ui, |ui| {
            ScrollArea::vertical()
                .id_salt("brand_space")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    Frame::new()
                        .inner_margin(Margin::symmetric(space::XXL as i8, 28))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.spacing_mut().item_spacing.y = space::MD;
                            sections(ui, cmds, env);
                        });
                });
        });
}

/// The section index: each section with its number of elements; clicking
/// one scrolls it into view and marks it.
fn index(ui: &mut Ui, env: &PanelEnv<'_>) {
    let current = ui
        .data(|d| d.get_temp::<Section>(current_id()))
        .unwrap_or(Section::Palette);
    ui.spacing_mut().item_spacing.y = space::XXS;
    for section in Section::ALL {
        let count = section.count(&env.ws.project);
        if index_entry(ui, &section.title(), count, section == current).clicked() {
            ui.data_mut(|d| {
                d.insert_temp(current_id(), section);
                d.insert_temp(scroll_id(), section);
            });
        }
    }
}

/// "Palette · 5": the current entry is filled and its title drawn in ink.
fn index_entry(ui: &mut Ui, title: &str, count: usize, current: bool) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 34.0), Sense::click());
    let label = format!("{title} · {count}");
    response
        .widget_info(|| WidgetInfo::selected(WidgetType::SelectableLabel, true, current, &label));
    let painter = ui.painter();
    if current {
        painter.rect_filled(rect, radius::MD, color::SURFACE_3);
    } else if response.hovered() {
        painter.rect_filled(rect, radius::MD, color::SURFACE_2);
    }
    paint_focus_ring(ui, rect, &response, radius::MD);
    let body = egui::TextStyle::Body.resolve(ui.style());
    let title_rect = painter.text(
        egui::pos2(rect.left() + 10.0, rect.center().y),
        Align2::LEFT_CENTER,
        title,
        body.clone(),
        if current {
            color::TEXT_PRIMARY
        } else {
            color::TEXT_SECONDARY
        },
    );
    painter.text(
        egui::pos2(title_rect.right(), rect.center().y),
        Align2::LEFT_CENTER,
        format!(" · {count}"),
        body,
        color::TEXT_DISABLED,
    );
    response
}

/// The header, then each section; the one asked by the index is scrolled
/// into view.
fn sections(ui: &mut Ui, cmds: &mut CommandUi<'_>, env: &mut PanelEnv<'_>) {
    ui.horizontal(|ui| {
        super::title(ui, &tr("cmd-space-brand"));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            panels::import_from_library_button(ui, cmds);
        });
    });
    let scroll_to = ui.data_mut(|d| {
        let section = d.get_temp::<Section>(scroll_id());
        d.remove::<Section>(scroll_id());
        section
    });
    let mut workshop = false;
    for section in Section::ALL {
        ui.add_space(space::XL);
        let heading = match section {
            Section::Palette => palette(ui, env),
            Section::GraphicStyles => style_section(ui, env, false, &mut workshop),
            Section::TextStyles => style_section(ui, env, true, &mut workshop),
            Section::Symbols => symbol_section(ui, cmds, env, &mut workshop),
            Section::Images => image_section(ui, cmds, env, &mut workshop),
        };
        if scroll_to == Some(section) {
            heading.scroll_to_me(Some(Align::TOP));
        }
    }
    if workshop {
        env.ws.space = Space::Workshop;
    }
}

/// A section's title with its header actions at the right end; returns the
/// title.
fn section_header(ui: &mut Ui, title: &str, actions: impl FnOnce(&mut Ui)) -> Response {
    ui.horizontal(|ui| {
        let title = super::section_title(ui, title);
        ui.with_layout(Layout::right_to_left(Align::Center), actions);
        title
    })
    .inner
}

/// What a section says when it has no element.
fn hint(ui: &mut Ui, text: &str) {
    ui.add(egui::Label::new(RichText::new(text).color(color::TEXT_SECONDARY)).wrap());
}

/// A card of `width` with a preview area `preview` high over its text
/// area, named `label` for assistive technologies, which also read its
/// `detail`; returns its response (hover, context menu), the preview's and
/// the text area's rectangles.
fn card(
    ui: &mut Ui,
    width: f32,
    preview: f32,
    label: &str,
    detail: &str,
    marked: bool,
) -> (Response, Rect, Rect) {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(width, preview + CARD_TEXT), Sense::click());
    response.widget_info(|| {
        let mut info = WidgetInfo::selected(WidgetType::Other, true, marked, label);
        info.current_text_value = Some(detail.to_owned());
        info
    });
    super::paint_card(ui, rect, response.hovered(), marked);
    paint_focus_ring(ui, rect, &response, radius::LG);
    let preview_rect = Rect::from_min_size(rect.min, Vec2::new(width, preview));
    let text = Rect::from_min_max(egui::pos2(rect.left(), preview_rect.bottom()), rect.max)
        .shrink2(CARD_PAD);
    (response, preview_rect, text)
}

/// A card's title and detail lines in `rect`, clipped at `right`.
fn card_text(ui: &Ui, rect: Rect, right: f32, title: &str, detail: &str, mono: bool) {
    let clip = Rect::from_min_max(rect.min, egui::pos2(right, rect.bottom()));
    let painter = ui.painter().with_clip_rect(clip.intersect(ui.clip_rect()));
    painter.text(
        rect.left_top(),
        Align2::LEFT_TOP,
        title,
        egui::FontId::new(typography::BODY, tp_ui::fonts::semibold_family()),
        color::TEXT_PRIMARY,
    );
    let font = if mono {
        egui::FontId::monospace(typography::CAPTION)
    } else {
        egui::FontId::proportional(typography::CAPTION + 1.0)
    };
    painter.text(
        rect.left_bottom(),
        Align2::LEFT_BOTTOM,
        detail,
        font,
        color::TEXT_SECONDARY,
    );
}

/// A Ui laid out right to left in a card's text area, for its buttons.
fn buttons(ui: &mut Ui, text: Rect) -> Ui {
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(text)
            .layout(Layout::right_to_left(Align::Center)),
    );
    child.spacing_mut().item_spacing.x = space::XXS;
    child
}

/// The ⋯ menu button of a card, showing the same `items` as its context
/// menu.
fn card_menu<R>(ui: &mut Ui, name: &str, items: impl FnOnce(&mut Ui) -> R) -> Option<R> {
    let menu = ui.menu_button(icons::rich(icons::MORE), items);
    menu.response.widget_info(|| {
        WidgetInfo::labeled(
            WidgetType::Button,
            true,
            tr!("brand-actions-named", name = name),
        )
    });
    menu.inner
}

/// The top corners of a card, for a preview filling its top.
fn top_corners() -> CornerRadius {
    CornerRadius {
        nw: radius::LG,
        ne: radius::LG,
        sw: 0,
        se: 0,
    }
}

/// The Palette: New Color, then a card per swatch (its color, name and
/// hex value) with Edit Swatch…, Add to / Update in Library and Delete
/// Swatch in its menu.
fn palette(ui: &mut Ui, env: &mut PanelEnv<'_>) -> Response {
    let heading = section_header(ui, &Section::Palette.title(), |ui| {
        let current = colors::color_to_add(env);
        let label = tr("brand-new-color");
        let reason = tr("colors-differ");
        let enabled = current.is_some();
        let add = super::add_button(ui, &label, &label, (!enabled).then_some(reason.as_str()));
        if add.clicked()
            && let Some(c) = current
        {
            colors::add_color(env, c);
        }
    });
    let palette = env.ws.project.palette.clone();
    if palette.is_empty() {
        hint(ui, &tr("colors-palette-empty"));
        return heading;
    }
    super::grid(ui, 150.0, palette.len(), |ui, i, width| {
        let swatch = &palette[i];
        let c = swatch.color;
        let hex = c.to_hex();
        let (response, preview, text) = card(ui, width, 72.0, &swatch.name, &hex, false);
        let block = preview.shrink(1.0);
        if c.a < 255 {
            paint_checkerboard(ui.painter(), block, 8.0);
        }
        ui.painter().rect_filled(
            block,
            top_corners(),
            egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a),
        );
        let mut actions = buttons(ui, text);
        card_menu(&mut actions, &swatch.name, |ui| {
            colors::swatch_menu(ui, env, swatch.id);
        });
        let right = actions.min_rect().left() - space::XS;
        card_text(ui, text, right, &swatch.name, &hex, true);
        response.context_menu(|ui| colors::swatch_menu(ui, env, swatch.id));
    });
    heading
}

/// Paints a style's look in `rect`: its fill, outlined by its stroke.
fn paint_look(ui: &Ui, rect: Rect, look: &Look) {
    let painter = ui.painter();
    match &look.fill {
        Paint::Solid(c) => {
            if c.a < 255 {
                paint_checkerboard(painter, rect, 6.0);
            }
            painter.rect_filled(
                rect,
                radius::SM,
                egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a),
            );
        }
        Paint::Gradient(g) => panels::properties::gradient_preview(g).paint(painter, rect),
    }
    if let Some(s) = &look.stroke {
        let c = s.paint.first_color();
        painter.rect_stroke(
            rect,
            radius::SM,
            Stroke::new(
                3.0,
                egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a),
            ),
            egui::StrokeKind::Inside,
        );
    }
}

/// Graphic styles, or text styles when `text`: New Style from Selection,
/// then a card per style (its look, its name, in its own font for a text
/// style, and a text style's size). The styles the selection follows are
/// outlined and show a link icon. Select Users on This Texture sets
/// `workshop`.
fn style_section(ui: &mut Ui, env: &mut PanelEnv<'_>, text: bool, workshop: &mut bool) -> Response {
    let section = if text {
        Section::TextStyles
    } else {
        Section::GraphicStyles
    };
    let heading = section_header(ui, &section.title(), |ui| {
        let blocked = styles::new_style_blocked(env, text);
        let name = tr(if text {
            "styles-new-text"
        } else {
            "styles-new-graphic"
        });
        if super::add_button(ui, &tr("brand-new-style"), &name, blocked.as_deref()).clicked() {
            styles::new_style(env, text);
        }
    });
    struct Item {
        id: StyleId,
        name: String,
        look: Look,
        /// A text style's family, weight, italic and size.
        font: Option<(String, u16, bool, String)>,
    }
    let items: Vec<Item> = if text {
        env.ws
            .project
            .text_styles
            .iter()
            .map(|s| Item {
                id: s.id,
                name: s.name.clone(),
                look: s.look,
                font: Some((
                    s.style.family.clone(),
                    s.style.weight,
                    s.style.italic,
                    format!("{} px", s.style.size),
                )),
            })
            .collect()
    } else {
        env.ws
            .project
            .graphic_styles
            .iter()
            .map(|s| Item {
                id: s.id,
                name: s.name.clone(),
                look: s.look,
                font: None,
            })
            .collect()
    };
    if items.is_empty() {
        hint(
            ui,
            &tr(if text {
                "styles-text-empty"
            } else {
                "styles-graphic-empty"
            }),
        );
        return heading;
    }
    let followed = env.ws.project.styles_of(&env.ws.selection);
    super::grid(ui, 200.0, items.len(), |ui, i, width| {
        let item = &items[i];
        let label = tr!(
            if text {
                "styles-text-item"
            } else {
                "styles-graphic-item"
            },
            name = item.name.as_str()
        );
        let marked = followed.contains(&item.id);
        let size = item.font.as_ref().map_or("", |f| f.3.as_str());
        let (response, preview, area) = card(ui, width, 56.0, &label, size, marked);
        paint_look(ui, preview.shrink(space::MD), &item.look);
        if marked {
            // Marked by an icon too, not by the outline alone.
            ui.painter().text(
                preview.right_top() + Vec2::new(-space::SM, space::SM),
                Align2::RIGHT_TOP,
                icons::LINKED,
                icons::font(14.0),
                color::LINK,
            );
        }
        let mut actions = buttons(ui, area);
        if let Some(true) = card_menu(&mut actions, &item.name, |ui| {
            styles::menu_items(ui, env, item.id, &item.name)
        }) {
            *workshop = true;
        }
        let right = actions.min_rect().left() - space::XS;
        if styles::is_renaming(env, item.id) {
            let mut field = ui.new_child(
                UiBuilder::new()
                    .max_rect(Rect::from_min_max(area.min, egui::pos2(right, area.max.y)))
                    .layout(Layout::left_to_right(Align::Center)),
            );
            styles::rename_field(&mut field, env, item.id);
        } else if let Some((family, weight, italic, size)) = &item.font {
            // The name in the style's own font, then its size.
            let preview = env
                .ws
                .text
                .styled_preview(&item.name, family, *weight, *italic);
            let title = Rect::from_min_max(
                area.min - Vec2::new(space::SM, 0.0),
                egui::pos2(right, area.top() + 20.0),
            );
            panels::character::draw_preview_mesh(ui, &preview, title);
            card_text(ui, area, right, "", size, false);
        } else {
            card_text(ui, area, right, &item.name, "", false);
        }
        let mut select_users = false;
        response.context_menu(|ui| {
            select_users = styles::menu_items(ui, env, item.id, &item.name);
        });
        *workshop |= select_users;
    });
    heading
}

/// Symbols: Create from Selection (Convert to Symbol), then a card per
/// symbol (its icon, name and number of instances) with Place and Edit,
/// which show the Workshop, and Rename, Duplicate, Add to / Update in
/// Library and Delete in its menu.
fn symbol_section(
    ui: &mut Ui,
    cmds: &mut CommandUi<'_>,
    env: &mut PanelEnv<'_>,
    workshop: &mut bool,
) -> Response {
    let heading = section_header(ui, &Section::Symbols.title(), |ui| {
        let id = CommandId::ConvertToSymbol;
        let label = tr("brand-create-from-selection");
        let reason = (!cmds.enabled(id)).then(|| {
            disabled_reason_for(id, &cmds.edit)
                .map(tr)
                .unwrap_or_default()
        });
        if super::add_button(ui, &label, &label, reason.as_deref()).clicked() {
            cmds.push(id);
        }
    });
    let list: Vec<_> = env
        .ws
        .project
        .symbols
        .iter()
        .map(|s| (s.id, s.name.clone()))
        .collect();
    if list.is_empty() {
        hint(ui, &tr("symbols-empty"));
        return heading;
    }
    super::grid(ui, 200.0, list.len(), |ui, i, width| {
        let (id, name) = (list[i].0, list[i].1.as_str());
        let count = env.ws.project.instance_count(id);
        let editing = env.ws.project.editing_symbol == Some(id);
        let label = tr!("symbols-item", name = name);
        let instances = symbols::instances_text(count);
        let (response, preview, area) = card(ui, width, 64.0, &label, &instances, editing);
        let inner = preview.shrink(1.0);
        ui.painter()
            .rect_filled(inner, top_corners(), color::SURFACE_2);
        ui.painter().text(
            inner.center(),
            Align2::CENTER_CENTER,
            icons::SYMBOL,
            icons::font(28.0),
            color::LINK,
        );
        let mut actions = buttons(ui, area);
        card_menu(&mut actions, name, |ui| {
            symbols::menu_items(ui, env, id, name, count);
        });
        let edit = tr!("symbols-edit", name = name);
        if actions.add(IconButton::new(icons::RENAME, &edit)).clicked() {
            env.ws.edit_symbol(id, env.now);
            *workshop = true;
        }
        let place = tr!("symbols-place", name = name);
        let reason = tr("reason-editing-symbol");
        let button = IconButton::new(icons::ADD, &place).disabled_reason(&reason);
        if actions
            .add_enabled(!env.ws.is_editing_symbol(), button)
            .clicked()
        {
            env.ws.place_symbol(id, None, env.now);
            *workshop = true;
        }
        let right = actions.min_rect().left() - space::XS;
        if env
            .ws
            .panels
            .renaming_symbol
            .as_ref()
            .is_some_and(|(r, _)| *r == id)
        {
            let mut field = ui.new_child(
                UiBuilder::new()
                    .max_rect(Rect::from_min_max(area.min, egui::pos2(right, area.max.y)))
                    .layout(Layout::left_to_right(Align::Center)),
            );
            symbols::rename_field(&mut field, env, id);
        } else {
            card_text(ui, area, right, name, &instances, false);
        }
        response.context_menu(|ui| symbols::menu_items(ui, env, id, name, count));
    });
    symbols::confirm_delete(ui, env);
    heading
}

/// Images: Import… (File › Place…), then a card per image (its thumbnail,
/// name, pixel size or "Vector" and number of uses) with Place, which
/// shows the Workshop, Rename and Remove.
fn image_section(
    ui: &mut Ui,
    cmds: &mut CommandUi<'_>,
    env: &mut PanelEnv<'_>,
    workshop: &mut bool,
) -> Response {
    let heading = section_header(ui, &Section::Images.title(), |ui| {
        cmds.secondary_button(ui, CommandId::Place, &tr("assets-import"));
    });
    let list: Vec<std::sync::Arc<Asset>> = assets::listed(&env.ws.project);
    if list.is_empty() {
        hint(ui, &tr("empty-assets-hint"));
        return heading;
    }
    super::grid(ui, 180.0, list.len(), |ui, i, width| {
        let asset = &list[i];
        let uses = assets::uses_of(&env.ws.project, asset);
        let label = tr!("assets-item", name = asset.name.as_str());
        let detail = format!("{} · {}", assets::size_text(asset), assets::uses_text(uses));
        let (_, preview, area) = card(ui, width, 96.0, &label, &detail, false);
        assets::paint_thumbnail(ui, env, asset, preview.shrink(space::SM));
        let mut actions = buttons(ui, area);
        *workshop |= assets::actions(&mut actions, env, asset, uses);
        let right = actions.min_rect().left() - space::XS;
        if assets::is_renaming(env, asset) {
            let mut field = ui.new_child(
                UiBuilder::new()
                    .max_rect(Rect::from_min_max(area.min, egui::pos2(right, area.max.y)))
                    .layout(Layout::left_to_right(Align::Center)),
            );
            assets::rename_field(&mut field, env);
        } else {
            card_text(ui, area, right, &asset.name, &detail, false);
        }
    });
    heading
}
