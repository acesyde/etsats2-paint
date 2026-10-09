use egui::{
    Align, Align2, CursorIcon, FontId, Id, Key, Margin, Rect, Response, RichText, Sense, Stroke,
    StrokeKind, TextEdit, Ui, Vec2, WidgetInfo, WidgetType,
};
use tp_i18n::tr;

use crate::tokens::{color, radius, space, stroke, typography};

/// What happened to a [`NumericField`] this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FieldEvent {
    None,
    /// Value changing during a label scrub (apply live, don't record yet).
    Live(f64),
    /// Final value: typed and confirmed, or end of a scrub.
    Commit(f64),
    /// Edit abandoned with Escape (revert any live change).
    Revert,
}

/// Compact numeric input: a short label that can be dragged to scrub the
/// value, and a text box committed on Enter / Tab / focus loss.
///
/// By default the label and the suffix sit outside the text box. In the
/// [`inset`](Self::inset) mode, one sunken box holds them all: the label as
/// a muted prefix (still dragged to scrub), the value in the monospace face
/// and the suffix at the right end.
pub struct NumericField<'a> {
    label: &'a str,
    name: &'a str,
    /// `None` shows "Mixed".
    value: Option<f64>,
    suffix: &'a str,
    decimals: usize,
    range: std::ops::RangeInclusive<f64>,
    /// Value change per point of horizontal drag.
    speed: f64,
    width: f32,
    /// Text box as wide as the room left by the label and the suffix.
    fill: bool,
    /// Height of the inset box; `None`: label and suffix outside the box.
    inset: Option<f32>,
}

#[derive(Clone, Default)]
struct ScrubState {
    start: f64,
    travel: f32,
}

impl<'a> NumericField<'a> {
    /// `label` is the short visible text ("W"); `name` is the accessible and
    /// tooltip name ("Width").
    pub fn new(label: &'a str, name: &'a str, value: Option<f64>) -> Self {
        Self {
            label,
            name,
            value,
            suffix: "",
            decimals: 0,
            range: f64::MIN..=f64::MAX,
            speed: 1.0,
            width: 64.0,
            fill: false,
            inset: None,
        }
    }

    pub fn suffix(mut self, suffix: &'a str) -> Self {
        self.suffix = suffix;
        self
    }

    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    pub fn range(mut self, range: std::ops::RangeInclusive<f64>) -> Self {
        self.range = range;
        self
    }

    pub fn speed(mut self, speed: f64) -> Self {
        self.speed = speed;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Makes the text box take the width left in the row by the label and
    /// the suffix (at least the set width), so a field fills a column.
    pub fn fill(mut self) -> Self {
        self.fill = true;
        self
    }

    /// Draws the field as one sunken box of the given height holding the
    /// label (a prefix), the value and the suffix.
    pub fn inset(mut self, height: f32) -> Self {
        self.inset = Some(height);
        self
    }

    fn format(&self, v: f64) -> String {
        tp_i18n::format_number(v, self.decimals)
    }

    fn clamp(&self, v: f64) -> f64 {
        v.clamp(*self.range.start(), *self.range.end())
    }

    pub fn show(self, ui: &mut Ui) -> FieldEvent {
        if let Some(height) = self.inset {
            return self.show_inset(ui, height);
        }
        let id = ui.id().with(("numeric_field", self.name));
        let mut event = FieldEvent::None;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = space::XS;
            // Scrubbable label (none when the field is labelled above, so
            // that it lines up with the fields around it).
            if !self.label.is_empty() {
                let label = ui
                    .add(
                        egui::Label::new(
                            RichText::new(self.label)
                                .small()
                                .color(color::TEXT_SECONDARY),
                        )
                        .sense(Sense::drag()),
                    )
                    .on_hover_cursor(CursorIcon::ResizeHorizontal)
                    .on_hover_text(self.name);
                event = self.scrub(ui, &label, id);
            }

            // Text box.
            let text_id = id.with("text");
            remember_escape(ui, text_id);
            let mut buffer = ui
                .data(|d| d.get_temp::<String>(text_id))
                .unwrap_or_else(|| self.value.map(|v| self.format(v)).unwrap_or_default());
            let hint = if self.value.is_none() {
                tr("mixed")
            } else {
                String::new()
            };
            // Wide enough for the "Mixed" hint in every language.
            let hint_width = ui
                .painter()
                .layout_no_wrap(
                    hint.clone(),
                    egui::TextStyle::Monospace.resolve(ui.style()),
                    egui::Color32::WHITE,
                )
                .size()
                .x;
            let margin = Margin::symmetric(6, 3);
            let mut width = self.width.max(hint_width + 14.0);
            if self.fill {
                let suffix = if self.suffix.is_empty() {
                    0.0
                } else {
                    let font = egui::TextStyle::Small.resolve(ui.style());
                    ui.painter()
                        .layout_no_wrap(self.suffix.to_owned(), font, egui::Color32::WHITE)
                        .size()
                        .x
                        + ui.spacing().item_spacing.x
                };
                // The desired width of a `TextEdit` includes its margin.
                let room = ui.available_width() - suffix;
                width = width.max(room.floor());
            }
            // Values are set in the monospace face (JetBrains Mono).
            let edit = TextEdit::singleline(&mut buffer)
                .id(text_id)
                .font(egui::TextStyle::Monospace)
                .desired_width(width)
                .margin(margin)
                .hint_text(hint);
            let response = ui.add(edit);
            response.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, self.name));
            if !self.suffix.is_empty() {
                ui.label(
                    RichText::new(self.suffix)
                        .small()
                        .color(color::TEXT_SECONDARY),
                );
            }
            if let FieldEvent::None = event {
                event = self.text_events(ui, &response, text_id, buffer);
            }
        });
        event
    }

    /// The inset box: a sunken fill outlined by a hairline (ink while the
    /// value is typed), the prefix and the suffix in the muted color.
    fn show_inset(self, ui: &mut Ui, height: f32) -> FieldEvent {
        let id = ui.id().with(("numeric_field", self.name));
        let width = if self.fill {
            ui.available_width().max(self.width)
        } else {
            self.width
        };
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        let pad = space::SM;
        let painter = ui.painter().clone();
        painter.rect_filled(rect, radius::MD, color::FIELD);

        // The prefix, dragged to scrub.
        let mut left = rect.left() + pad;
        let mut event = FieldEvent::None;
        if !self.label.is_empty() {
            let galley = painter.layout_no_wrap(
                self.label.to_owned(),
                FontId::proportional(typography::CONTROL),
                color::TEXT_MUTED,
            );
            let pos = egui::pos2(left, rect.center().y - galley.size().y / 2.0);
            painter.galley(pos, galley.clone(), color::TEXT_MUTED);
            let area = Rect::from_min_max(
                egui::pos2(rect.left(), rect.top()),
                egui::pos2(left + galley.size().x + space::XS, rect.bottom()),
            );
            let label = ui
                .interact(area, id.with("prefix"), Sense::drag())
                .on_hover_cursor(CursorIcon::ResizeHorizontal)
                .on_hover_text(self.name);
            event = self.scrub(ui, &label, id);
            left = area.right() + space::XXS;
        }
        // The suffix at the right end.
        let mut right = rect.right() - pad;
        if !self.suffix.is_empty() {
            let suffix = painter.text(
                egui::pos2(right, rect.center().y),
                Align2::RIGHT_CENTER,
                self.suffix,
                FontId::proportional(typography::CAPTION),
                color::TEXT_MUTED,
            );
            right = suffix.left() - space::XS;
        }

        let text_id = id.with("text");
        remember_escape(ui, text_id);
        let mut buffer = ui
            .data(|d| d.get_temp::<String>(text_id))
            .unwrap_or_else(|| self.value.map(|v| self.format(v)).unwrap_or_default());
        let hint = if self.value.is_none() {
            tr("mixed")
        } else {
            String::new()
        };
        let text_rect = Rect::from_min_max(
            egui::pos2(left, rect.top() + 1.0),
            egui::pos2(right.max(left + 8.0), rect.bottom() - 1.0),
        );
        let edit = TextEdit::singleline(&mut buffer)
            .id(text_id)
            .font(egui::TextStyle::Monospace)
            .frame(egui::Frame::NONE)
            .vertical_align(Align::Center)
            .desired_width(text_rect.width())
            .hint_text(hint);
        let response = ui.put(text_rect, edit);
        response.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, self.name));
        let outline = if response.has_focus() {
            Stroke::new(stroke::FOCUS, color::FOCUS)
        } else if ui.rect_contains_pointer(rect) {
            Stroke::new(stroke::HAIRLINE, color::BORDER_STRONG)
        } else {
            Stroke::new(stroke::HAIRLINE, color::BORDER)
        };
        painter.rect_stroke(rect, radius::MD, outline, StrokeKind::Inside);
        if let FieldEvent::None = event {
            event = self.text_events(ui, &response, text_id, buffer);
        }
        event
    }

    fn scrub(&self, ui: &Ui, label: &Response, id: Id) -> FieldEvent {
        let scrub_id = id.with("scrub");
        let Some(value) = self.value else {
            return FieldEvent::None;
        };
        if label.drag_started() {
            ui.data_mut(|d| {
                d.insert_temp(
                    scrub_id,
                    ScrubState {
                        start: value,
                        travel: 0.0,
                    },
                )
            });
        }
        if label.dragged() || label.drag_stopped() {
            let modifiers = ui.input(|i| i.modifiers);
            let step = if modifiers.shift {
                10.0
            } else if modifiers.alt {
                0.1
            } else {
                1.0
            };
            let dx = label.drag_delta().x;
            let state = ui.data_mut(|d| {
                let s = d.get_temp_mut_or_default::<ScrubState>(scrub_id);
                s.travel += dx * step as f32;
                s.clone()
            });
            let new = self.clamp(state.start + f64::from(state.travel) * self.speed);
            if label.drag_stopped() {
                ui.data_mut(|d| d.remove::<ScrubState>(scrub_id));
                return FieldEvent::Commit(new);
            }
            return FieldEvent::Live(new);
        }
        FieldEvent::None
    }

    fn text_events(&self, ui: &Ui, response: &Response, text_id: Id, buffer: String) -> FieldEvent {
        if response.has_focus() || response.gained_focus() {
            ui.data_mut(|d| d.insert_temp(text_id, buffer.clone()));
        }
        if !response.lost_focus() {
            return FieldEvent::None;
        }
        ui.data_mut(|d| d.remove::<String>(text_id));
        if take_escape(ui, text_id) {
            return FieldEvent::Revert;
        }
        match parse_number(&buffer) {
            Some(v) => {
                let v = self.clamp(v);
                if self.value.is_some_and(|old| (old - v).abs() < 1e-9) {
                    FieldEvent::None
                } else {
                    FieldEvent::Commit(v)
                }
            }
            None => FieldEvent::None,
        }
    }
}

/// Records an Escape press while the text field `text_id` has focus. The
/// field loses focus on Escape, but `lost_focus` is only reported on the next
/// frame, when the key press is gone; call this before adding the field.
pub fn remember_escape(ui: &Ui, text_id: Id) {
    let focused = ui.memory(|m| m.has_focus(text_id));
    if focused && ui.input(|i| i.key_pressed(Key::Escape)) {
        ui.data_mut(|d| d.insert_temp(text_id.with("escape"), true));
    }
}

/// Whether the field was left with Escape (see [`remember_escape`]); clears
/// the record.
pub fn take_escape(ui: &Ui, text_id: Id) -> bool {
    let flag = text_id.with("escape");
    let escaped = ui
        .data_mut(|d| d.remove_temp::<bool>(flag))
        .unwrap_or(false);
    escaped || ui.input(|i| i.key_pressed(Key::Escape))
}

/// Parses a number typed by a user: spaces and unit suffixes (px, %, °) are
/// ignored, a comma is accepted as decimal separator.
pub fn parse_number(text: &str) -> Option<f64> {
    let cleaned: String = text
        .trim()
        .trim_end_matches(['%', '°'])
        .trim_end_matches("px")
        .trim()
        .replace(',', ".");
    cleaned.parse::<f64>().ok().filter(|v| v.is_finite())
}

#[cfg(test)]
mod tests {
    use super::parse_number;

    #[test]
    fn parses_user_numbers() {
        assert_eq!(parse_number(" 12.5 "), Some(12.5));
        assert_eq!(parse_number("40%"), Some(40.0));
        assert_eq!(parse_number("15°"), Some(15.0));
        assert_eq!(parse_number("300 px"), Some(300.0));
        assert_eq!(parse_number("1,5"), Some(1.5));
        assert_eq!(parse_number("abc"), None);
        assert_eq!(parse_number(""), None);
    }
}
