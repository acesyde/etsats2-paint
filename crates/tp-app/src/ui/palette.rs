//! The command palette: a search field over the window listing every
//! command and the project's textures, run or opened from the keyboard.

use egui::text::{LayoutJob, TextFormat};
use egui::{
    Align, Align2, Color32, CornerRadius, FontId, Frame, Key, Margin, Modifiers, Rect, RichText,
    ScrollArea, Sense, Stroke, TextEdit, TextStyle, Ui, Vec2, WidgetInfo, WidgetType,
};
use tp_core::TextureState;
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::{color, radius, size, space, typography};

use super::command_ui::command_label;
use super::workspace::breadcrumb::Crumbs;
use super::workspace::panels::{section_heading, vehicle};
use crate::commands::ShortcutFormatter;
use crate::layout::Space;
use crate::menus;
use crate::palette::{self, Entry, Row, Target};
use crate::state::{AppState, disabled_reason_for, is_enabled};

/// Widest the palette gets.
const MAX_WIDTH: f32 = 560.0;
/// Height of a result row.
const ROW_HEIGHT: f32 = 28.0;
/// Rows shown before the list scrolls.
const VISIBLE_ROWS: usize = 10;
/// Height of the search field.
const FIELD_HEIGHT: f32 = 32.0;
/// Width of the icon column.
const ICON_COLUMN: f32 = 16.0;
/// Share of a row's text the context may take before it is shortened.
const CONTEXT_SHARE: f32 = 0.45;

/// The palette while it is open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palette {
    pub query: String,
    /// Lists only the project's textures (opened from Search textures).
    pub textures_only: bool,
    /// Index of the highlighted row.
    highlight: usize,
    /// The highlighted row scrolls into view (after a key move).
    scroll_to_highlight: bool,
    /// The query and mode the highlight was set for.
    shown: Option<(String, bool)>,
}

impl Palette {
    pub fn new(textures_only: bool, query: &str) -> Self {
        Self {
            query: query.to_owned(),
            textures_only,
            highlight: 0,
            scroll_to_highlight: false,
            shown: None,
        }
    }
}

/// What a row shows besides its label and context.
struct Item {
    entry: Entry,
    icon: Option<&'static str>,
    shortcut: Option<String>,
    /// A toggle that is on.
    checked: bool,
    /// Why a disabled command is disabled.
    reason: Option<String>,
    texture: Option<TextureRow>,
}

struct TextureRow {
    size: String,
    state: TextureState,
    active: bool,
}

impl Item {
    /// The row's name for assistive technologies and its hover text:
    /// "context › label", then the shortcut, and the reason of a disabled
    /// command or the state of a texture.
    fn name(&self) -> String {
        let mut name = if self.entry.context.is_empty() {
            self.entry.label.clone()
        } else {
            format!("{} › {}", self.entry.context, self.entry.label)
        };
        if let Some(texture) = &self.texture {
            name.push_str(&format!(
                " {}, {}",
                texture.size,
                vehicle::state_label(texture.state)
            ));
            return name;
        }
        if let Some(shortcut) = &self.shortcut {
            name.push_str(", ");
            name.push_str(shortcut);
        }
        if let Some(reason) = &self.reason {
            name.push_str(", ");
            name.push_str(&tr!("palette-unavailable", reason = reason.as_str()));
        }
        name
    }
}

/// The palette's entries in the current state: the commands (unless
/// `textures_only`), then the project's textures.
fn items(state: &AppState, shortcuts: &ShortcutFormatter, textures_only: bool) -> Vec<Item> {
    let edit = state.edit_context();
    let mut items = Vec::new();
    if !textures_only {
        let layout = state.workspace().map(|_| &state.prefs.layout);
        for c in menus::catalog() {
            let enabled = is_enabled(c.id, &edit);
            items.push(Item {
                entry: Entry {
                    target: Target::Command(c.id),
                    label: command_label(c.id, &edit),
                    context: palette::context(&c.path),
                    enabled,
                },
                icon: c.id.meta().icon,
                shortcut: shortcuts.command(c.id),
                checked: c.toggle && menus::is_checked(c.id, &edit, layout, state.prefs.view_aids),
                reason: (!enabled)
                    .then(|| disabled_reason_for(c.id, &edit))
                    .flatten()
                    .map(tr),
                texture: None,
            });
        }
    }
    if let Some(ws) = state.workspace() {
        let project = &ws.project;
        for (i, surface) in project.surfaces.iter().enumerate() {
            let crumbs = Crumbs::of_texture(project, i);
            items.push(Item {
                entry: Entry {
                    target: Target::Texture(i),
                    label: crumbs.name,
                    context: crumbs.path.join(" › "),
                    enabled: true,
                },
                icon: None,
                shortcut: None,
                checked: false,
                reason: None,
                texture: Some(TextureRow {
                    size: crumbs.size.unwrap_or_default(),
                    state: surface.state(),
                    active: project.active_surface == i && !ws.is_editing_symbol(),
                }),
            });
        }
    }
    items
}

/// The selectable row nearest to `from` going `step` (±1) rows at a time,
/// `from` itself when selectable; the ends stop it.
fn selectable(rows: &[Row], from: usize, step: isize) -> Option<usize> {
    let mut i = from as isize;
    while i >= 0 && (i as usize) < rows.len() {
        if rows[i as usize].entry().is_some() {
            return Some(i as usize);
        }
        i += step;
    }
    None
}

/// Moves the highlight by `delta` selectable rows, stopping at the ends.
fn moved(rows: &[Row], highlight: usize, delta: isize) -> usize {
    let mut at = highlight;
    let step = delta.signum();
    for _ in 0..delta.unsigned_abs() {
        let next = at as isize + step;
        if next < 0 {
            break;
        }
        match selectable(rows, next as usize, step) {
            Some(i) => at = i,
            None => break,
        }
    }
    at
}

/// What the user did with the keys this frame.
#[derive(Default)]
struct Keys {
    moves: isize,
    choose: bool,
    close: bool,
    widen: bool,
}

/// Takes the palette's keys before its field sees them: arrows and pages
/// move the highlight, Enter chooses, Escape closes, Tab does nothing (the
/// focus stays), Backspace in an empty textures-only field widens it.
fn take_keys(ctx: &egui::Context, palette: &Palette) -> Keys {
    ctx.input_mut(|i| {
        let mut keys = Keys::default();
        let page = VISIBLE_ROWS as isize;
        while i.consume_key(Modifiers::NONE, Key::ArrowDown) {
            keys.moves += 1;
        }
        while i.consume_key(Modifiers::NONE, Key::ArrowUp) {
            keys.moves -= 1;
        }
        while i.consume_key(Modifiers::NONE, Key::PageDown) {
            keys.moves += page;
        }
        while i.consume_key(Modifiers::NONE, Key::PageUp) {
            keys.moves -= page;
        }
        keys.choose = i.consume_key(Modifiers::NONE, Key::Enter);
        keys.close = i.consume_key(Modifiers::NONE, Key::Escape);
        i.consume_key(Modifiers::NONE, Key::Tab);
        i.consume_key(Modifiers::SHIFT, Key::Tab);
        if palette.textures_only && palette.query.is_empty() {
            keys.widen = i.consume_key(Modifiers::NONE, Key::Backspace);
        }
        keys
    })
}

/// Draws the palette and acts on what the user chose. Returns false when
/// it closes.
pub fn show(ctx: &egui::Context, state: &mut AppState, palette: &mut Palette) -> bool {
    let shortcuts = ShortcutFormatter::new(ctx);
    let keys = take_keys(ctx, palette);
    if keys.close {
        return false;
    }
    if keys.widen {
        palette.textures_only = false;
    }

    let screen = ctx.content_rect();
    let width = MAX_WIDTH.min(screen.width() - 2.0 * space::LG);
    let top = size::TOP_BAR_HEIGHT + size::MENU_BAR_HEIGHT + space::XL;
    let frame = Frame::new()
        .fill(color::CONTROL)
        .stroke(Stroke::new(1.0, color::BORDER_STRONG))
        .corner_radius(CornerRadius::same(radius::CARD))
        .inner_margin(Margin::same(space::SM as i8))
        .shadow(egui::Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: color::SHADOW,
        });
    let id = egui::Id::new("command_palette");
    let area = egui::Modal::default_area(id).anchor(Align2::CENTER_TOP, Vec2::new(0.0, top));

    let mut chosen = None;
    let response = egui::Modal::new(id)
        .area(area)
        .frame(frame)
        .backdrop_color(Color32::TRANSPARENT)
        .show(ctx, |ui| {
            let inner = width - 2.0 * space::SM;
            ui.set_width(inner);
            field(ui, palette);

            let items = items(state, &shortcuts, palette.textures_only);
            let entries: Vec<Entry> = items.iter().map(|i| i.entry.clone()).collect();
            let rows = palette::rows(&entries, &palette.query, &state.palette_recent);
            let shown = (palette.query.clone(), palette.textures_only);
            if palette.shown.as_ref() != Some(&shown) {
                palette.shown = Some(shown);
                palette.highlight = selectable(&rows, 0, 1).unwrap_or(0);
                palette.scroll_to_highlight = true;
            }
            if keys.moves != 0 {
                palette.highlight = moved(&rows, palette.highlight, keys.moves);
                palette.scroll_to_highlight = true;
            }
            if keys.choose {
                chosen = rows.get(palette.highlight).and_then(Row::entry);
            }

            ui.add_space(space::SM);
            if rows.is_empty() {
                let text = tr!("palette-no-match", query = palette.query.trim());
                let label =
                    ui.add(egui::Label::new(RichText::new(&text).color(color::TEXT_MUTED)).wrap());
                label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
            } else {
                ScrollArea::vertical()
                    .id_salt("command_palette_rows")
                    .max_height(ROW_HEIGHT * VISIBLE_ROWS as f32)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        for (r, row) in rows.iter().enumerate() {
                            match row {
                                Row::Heading(key) => heading(ui, key, r == 0),
                                Row::Result { entry, hits } => {
                                    let highlighted = r == palette.highlight;
                                    let response =
                                        result_row(ui, &items[*entry], hits, highlighted);
                                    if highlighted && palette.scroll_to_highlight {
                                        response.scroll_to_me(None);
                                    }
                                    if response.hovered()
                                        && ui.input(|i| i.pointer.delta() != Vec2::ZERO)
                                    {
                                        palette.highlight = r;
                                    }
                                    if response.clicked() {
                                        palette.highlight = r;
                                        chosen = Some(*entry);
                                    }
                                }
                            }
                        }
                    });
                palette.scroll_to_highlight = false;
            }
            let highlighted = rows
                .get(palette.highlight)
                .and_then(Row::entry)
                .map(|i| &items[i]);
            footer(ui, highlighted);
            let chosen = chosen.map(|i| (items[i].entry.target, items[i].entry.enabled));
            (chosen, ui.min_rect().expand(space::SM))
        });
    // A press outside closes it; the backdrop keeps the press from the
    // canvas and the panels.
    let (chosen, panel) = response.inner;
    let pressed_outside = ctx.input(|i| {
        i.pointer.primary_pressed() && i.pointer.press_origin().is_some_and(|p| !panel.contains(p))
    });
    if pressed_outside || response.backdrop_response.clicked() {
        return false;
    }
    match chosen {
        Some((Target::Command(id), true)) => {
            palette::push_recent(&mut state.palette_recent, id);
            state.queue.push(id);
            false
        }
        Some((Target::Texture(i), _)) => {
            if let Some(ws) = state.workspace_mut() {
                ws.space = Space::Workshop;
                ws.set_active_surface(i);
            }
            false
        }
        // A disabled command: the footer says why; the palette stays.
        Some((Target::Command(_), false)) | None => true,
    }
}

/// The search field, with the keyboard focus.
fn field(ui: &mut Ui, palette: &mut Palette) {
    let hint = tr(if palette.textures_only {
        "palette-placeholder-textures"
    } else {
        "palette-placeholder"
    });
    Frame::new()
        .fill(color::FIELD)
        .stroke(Stroke::new(1.0, color::BORDER))
        .corner_radius(CornerRadius::same(radius::MD))
        .inner_margin(Margin::symmetric(space::SM as i8, 0))
        .show(ui, |ui| {
            let size = Vec2::new(ui.available_width(), FIELD_HEIGHT);
            let layout = egui::Layout::left_to_right(Align::Center);
            ui.allocate_ui_with_layout(size, layout, |ui| {
                ui.spacing_mut().item_spacing.x = space::SM;
                ui.label(icons::rich(icons::SEARCH).color(color::TEXT_MUTED));
                let edit = ui.add(
                    TextEdit::singleline(&mut palette.query)
                        .id(egui::Id::new("command_palette_field"))
                        .frame(Frame::NONE)
                        .lock_focus(true)
                        .margin(Margin::ZERO)
                        .desired_width(f32::INFINITY)
                        .hint_text(RichText::new(&hint).color(color::TEXT_MUTED)),
                );
                edit.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, &hint));
                edit.request_focus();
            });
        });
}

/// A heading of the empty query's groups.
fn heading(ui: &mut Ui, key: &str, first: bool) {
    if !first {
        ui.add_space(space::SM);
    }
    ui.horizontal(|ui| {
        ui.add_space(space::SM);
        section_heading(ui, &tr(key));
    });
    ui.add_space(space::XS);
}

/// A result: icon, context and label (matched letters in the strong
/// weight), then the shortcut, the check mark of a toggle that is on, or a
/// texture's size and state marker. Filled when highlighted.
fn result_row(ui: &mut Ui, item: &Item, hits: &[usize], highlighted: bool) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), ROW_HEIGHT), Sense::click());
    let name = item.name();
    let enabled = item.entry.enabled;
    response.widget_info(|| {
        WidgetInfo::selected(WidgetType::SelectableLabel, enabled, highlighted, &name)
    });
    let painter = ui.painter_at(rect);
    if highlighted {
        painter.rect_filled(rect, radius::MD, color::SELECTED);
    }
    let (ink, dim) = if enabled {
        (color::TEXT_PRIMARY, color::TEXT_MUTED)
    } else {
        (color::TEXT_DISABLED, color::TEXT_DISABLED)
    };
    // Muted text lacks contrast on the selection fill.
    let dim = if highlighted && enabled {
        color::TEXT_SECONDARY
    } else {
        dim
    };
    let body = TextStyle::Body.resolve(ui.style());
    let mono = FontId::monospace(typography::MONO);
    let y = rect.center().y;
    let mut left = rect.left() + space::SM;
    let icon_center = egui::pos2(left + ICON_COLUMN / 2.0, y);
    if let Some(icon) = item.icon {
        painter.text(
            icon_center,
            Align2::CENTER_CENTER,
            icon,
            icons::font(size::ICON - 2.0),
            dim,
        );
    }
    // The active texture: a small dot in the icon column.
    if item.texture.as_ref().is_some_and(|t| t.active) {
        painter.circle_filled(icon_center, 3.0, ink);
    }
    left += ICON_COLUMN + space::SM;

    // The right end first, so the texts get what remains.
    let mut right = rect.right() - space::SM;
    if let Some(texture) = &item.texture {
        let marker = Rect::from_center_size(egui::pos2(right - 8.0, y), Vec2::splat(16.0));
        vehicle::paint_state_marker(&painter, marker.center(), texture.state);
        right = marker.left() - space::SM;
        let size = painter.text(
            egui::pos2(right, y),
            Align2::RIGHT_CENTER,
            &texture.size,
            mono,
            dim,
        );
        right = size.left() - space::MD;
    } else {
        if item.checked {
            let check = painter.text(
                egui::pos2(right, y),
                Align2::RIGHT_CENTER,
                icons::CHECK,
                icons::font(size::ICON - 2.0),
                ink,
            );
            // The check mark reads as in the View menu: a checked box.
            let label = item.entry.label.as_str();
            ui.interact(check, response.id.with("check"), Sense::hover())
                .widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, enabled, true, label));
            right = check.left() - space::SM;
        }
        if let Some(shortcut) = &item.shortcut {
            let text = painter.text(
                egui::pos2(right, y),
                Align2::RIGHT_CENTER,
                shortcut,
                mono,
                dim,
            );
            right = text.left() - space::MD;
        }
    }

    // The label truncates last: the context is shortened first, down to
    // its share of the row.
    let room = (right - left).max(0.0);
    let strong = FontId::new(body.size, tp_ui::fonts::semibold_family());
    let mut label = LayoutJob::default();
    for (i, c) in item.entry.label.chars().enumerate() {
        let font = if hits.contains(&i) { &strong } else { &body };
        label.append(&c.to_string(), 0.0, TextFormat::simple(font.clone(), ink));
    }
    let label_width = painter.layout_job(label.clone()).size().x;
    let mut x = left;
    if !item.entry.context.is_empty() {
        let mut job = LayoutJob::single_section(
            format!("{} › ", item.entry.context),
            TextFormat::simple(body.clone(), dim),
        );
        truncate(&mut job, (room - label_width).max(room * CONTEXT_SHARE));
        let galley = painter.layout_job(job);
        let w = galley.size().x;
        painter.galley(egui::pos2(x, y - galley.size().y / 2.0), galley, dim);
        x += w;
    }
    truncate(&mut label, (right - x).max(0.0));
    let galley = painter.layout_job(label);
    painter.galley(egui::pos2(x, y - galley.size().y / 2.0), galley, ink);

    response.on_hover_text(name)
}

/// Lays `job` out on one line at most `width` wide, shortened with an
/// ellipsis.
fn truncate(job: &mut LayoutJob, width: f32) {
    job.wrap.max_width = width;
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    job.halign = Align::Min;
}

/// The footer: the keys, or why the highlighted command is disabled.
fn footer(ui: &mut Ui, highlighted: Option<&Item>) {
    ui.add_space(space::SM);
    let rect = ui.available_rect_before_wrap();
    ui.painter()
        .hline(rect.x_range(), rect.top(), Stroke::new(1.0, color::BORDER));
    ui.add_space(space::SM);
    let caption =
        |text: String, tint: Color32| RichText::new(text).size(typography::CAPTION).color(tint);
    ui.horizontal(|ui| {
        ui.add_space(space::XS);
        if let Some(reason) = highlighted.and_then(|i| i.reason.as_ref()) {
            let label =
                ui.add(egui::Label::new(caption(reason.clone(), color::TEXT_SECONDARY)).wrap());
            label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, reason.as_str()));
        } else {
            ui.spacing_mut().item_spacing.x = space::MD;
            for (keys, hint) in [
                ("↑↓", "palette-hint-move"),
                ("↵", "palette-hint-run"),
                ("Esc", "palette-hint-close"),
            ] {
                ui.label(caption(format!("{keys} {}", tr(hint)), color::TEXT_MUTED));
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> Vec<Row> {
        let result = |entry| Row::Result {
            entry,
            hits: Vec::new(),
        };
        vec![
            Row::Heading("palette-heading-recent"),
            result(0),
            Row::Heading("palette-heading-textures"),
            result(1),
            result(2),
        ]
    }

    #[test]
    fn moves_skip_headings_and_stop_at_the_ends() {
        let rows = rows();
        assert_eq!(selectable(&rows, 0, 1), Some(1));
        assert_eq!(moved(&rows, 1, 1), 3);
        assert_eq!(moved(&rows, 3, -1), 1);
        assert_eq!(moved(&rows, 1, -1), 1);
        assert_eq!(moved(&rows, 3, 10), 4);
        assert_eq!(moved(&rows, 4, -10), 1);
    }
}
