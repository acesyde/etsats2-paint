//! Dash, cap and join controls, shared by the Stroke panel (outlines of
//! shapes and texts) and the inspector's Appearance section (lines).

use egui::{Checkbox, Ui, WidgetInfo, WidgetType};
use tp_core::document::{Cap, Dash, Join, LineStyle};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::color;
use tp_ui::widgets::{FieldEvent, MenuRow, NumericField, SegmentedControl};

use super::properties::common;

/// Dash lengths of the Dashed preset (also the first lengths used).
pub const DASHED: Dash = Dash {
    dash: 20.0,
    gap: 10.0,
};

/// A preset of the dash menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    /// 20/10.
    Dashed,
    /// Round dots 12 px apart.
    Dotted,
    /// 60/20.
    LongDash,
}

impl Preset {
    pub const ALL: [Preset; 3] = [Preset::Dashed, Preset::Dotted, Preset::LongDash];

    pub fn label(self) -> &'static str {
        match self {
            Preset::Dashed => "line-preset-dashed",
            Preset::Dotted => "line-preset-dotted",
            Preset::LongDash => "line-preset-long-dash",
        }
    }
}

/// One change made with the controls.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LineEdit {
    /// Turns dashes on (with `last` lengths) or off.
    Dashed(bool, Dash),
    Dash(f64),
    Gap(f64),
    Preset(Preset),
    Cap(Cap),
    Join(Join),
    MiterLimit(f64),
}

impl LineEdit {
    /// Undo label for a change of `what` ("Stroke" or "Line").
    pub fn label(self, line: bool) -> &'static str {
        match (self, line) {
            (LineEdit::Cap(_), false) => "undo-change-stroke-caps",
            (LineEdit::Join(_) | LineEdit::MiterLimit(_), false) => "undo-change-stroke-joins",
            (_, false) => "undo-change-stroke-dashes",
            (LineEdit::Cap(_), true) => "undo-change-line-caps",
            (LineEdit::Join(_) | LineEdit::MiterLimit(_), true) => "undo-change-line-joins",
            (_, true) => "undo-change-line-dashes",
        }
    }

    pub fn apply(self, style: &mut LineStyle) {
        match self {
            LineEdit::Dashed(on, last) => style.dash = on.then_some(last),
            LineEdit::Dash(v) => {
                if let Some(d) = &mut style.dash {
                    d.dash = v.clamp(0.0, 2000.0);
                }
            }
            LineEdit::Gap(v) => {
                if let Some(d) = &mut style.dash {
                    d.gap = v.clamp(0.5, 2000.0);
                }
            }
            LineEdit::Preset(p) => {
                style.dash = Some(match p {
                    Preset::Dashed => DASHED,
                    Preset::Dotted => Dash {
                        dash: 0.0,
                        gap: 12.0,
                    },
                    Preset::LongDash => Dash {
                        dash: 60.0,
                        gap: 20.0,
                    },
                });
                if p == Preset::Dotted {
                    style.cap = Cap::Round;
                }
            }
            LineEdit::Cap(c) => style.cap = c,
            LineEdit::Join(j) => style.join = j,
            LineEdit::MiterLimit(v) => style.miter_limit = v.clamp(1.0, 20.0),
        }
    }
}

/// What the controls asked for this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Change {
    /// Apply the edit; `commit` records it as one undo step.
    Apply { edit: LineEdit, commit: bool },
    /// Restore the document (Escape in a field).
    Revert,
}

fn field_change(event: FieldEvent, edit: impl Fn(f64) -> LineEdit) -> Option<Change> {
    match event {
        FieldEvent::Live(v) => Some(Change::Apply {
            edit: edit(v),
            commit: false,
        }),
        FieldEvent::Commit(v) => Some(Change::Apply {
            edit: edit(v),
            commit: true,
        }),
        FieldEvent::Revert => Some(Change::Revert),
        FieldEvent::None => None,
    }
}

fn row_label(ui: &mut Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .small()
            .color(color::TEXT_SECONDARY),
    );
}

/// Dashed toggle, Dash and Gap fields, presets, cap, join and miter limit
/// for `styles` (several values show "Mixed"). `last` are the dash lengths
/// restored when Dashed is turned on.
pub fn controls(ui: &mut Ui, styles: &[LineStyle], last: Dash) -> Option<Change> {
    let mut change = None;
    let mut set = |c: Option<Change>| {
        if c.is_some() {
            change = c;
        }
    };
    let dashed = common(styles.iter().map(|s| s.dash.is_some()));
    let dashes: Vec<Dash> = styles.iter().filter_map(|s| s.dash).collect();
    ui.horizontal(|ui| {
        let on = dashed == Some(true);
        let mut checked = on;
        let response =
            ui.add(Checkbox::new(&mut checked, tr("line-dashed")).indeterminate(dashed.is_none()));
        response.widget_info(|| {
            WidgetInfo::selected(WidgetType::Checkbox, true, on, tr("line-dashed"))
        });
        if response.clicked() {
            // Mixed or off: turn on; on: turn off.
            let last = common(dashes.iter().copied()).unwrap_or(last);
            set(Some(Change::Apply {
                edit: LineEdit::Dashed(!on, last),
                commit: true,
            }));
        }
        let presets = ui.menu_button(tr("line-presets"), |ui| {
            for preset in Preset::ALL {
                if ui.add(MenuRow::new(&tr(preset.label()))).clicked() {
                    set(Some(Change::Apply {
                        edit: LineEdit::Preset(preset),
                        commit: true,
                    }));
                    ui.close();
                }
            }
        });
        presets
            .response
            .widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, tr("line-dash-presets")));
    });
    if !dashes.is_empty() {
        ui.horizontal(|ui| {
            let dash = common(dashes.iter().map(|d| d.dash));
            let e = NumericField::new(&tr("line-dash"), &tr("line-dash-length"), dash)
                .suffix("px")
                .decimals(1)
                .range(0.0..=2000.0)
                .width(44.0)
                .show(ui);
            set(field_change(e, LineEdit::Dash));
            let gap = common(dashes.iter().map(|d| d.gap));
            let e = NumericField::new(&tr("line-gap"), &tr("line-gap-length"), gap)
                .suffix("px")
                .decimals(1)
                .range(0.5..=2000.0)
                .width(44.0)
                .show(ui);
            set(field_change(e, LineEdit::Gap));
        });
    }
    ui.horizontal(|ui| {
        row_label(ui, &tr("line-cap"));
        let cap = common(styles.iter().map(|s| s.cap));
        let picked = ui
            .push_id("cap", |ui| {
                SegmentedControl::new()
                    .named_segment(
                        Some(Cap::Butt),
                        icons::CAP_BUTT,
                        &tr("line-butt"),
                        &tr("line-butt-cap"),
                    )
                    .named_segment(
                        Some(Cap::Round),
                        icons::CAP_ROUND,
                        &tr("line-round"),
                        &tr("line-round-cap"),
                    )
                    .named_segment(
                        Some(Cap::Square),
                        icons::CAP_SQUARE,
                        &tr("line-square"),
                        &tr("line-square-cap"),
                    )
                    .show(ui, cap)
            })
            .inner;
        if let Some(Some(cap)) = picked {
            set(Some(Change::Apply {
                edit: LineEdit::Cap(cap),
                commit: true,
            }));
        }
    });
    let join = common(styles.iter().map(|s| s.join));
    ui.horizontal(|ui| {
        row_label(ui, &tr("line-join"));
        let picked = ui
            .push_id("join", |ui| {
                SegmentedControl::new()
                    .named_segment(
                        Some(Join::Miter),
                        icons::JOIN_MITER,
                        &tr("line-miter"),
                        &tr("line-miter-join"),
                    )
                    .named_segment(
                        Some(Join::Round),
                        icons::JOIN_ROUND,
                        &tr("line-round"),
                        &tr("line-round-join"),
                    )
                    .named_segment(
                        Some(Join::Bevel),
                        icons::JOIN_BEVEL,
                        &tr("line-bevel"),
                        &tr("line-bevel-join"),
                    )
                    .show(ui, join)
            })
            .inner;
        if let Some(Some(join)) = picked {
            set(Some(Change::Apply {
                edit: LineEdit::Join(join),
                commit: true,
            }));
        }
    });
    if join == Some(Join::Miter) {
        let limit = common(styles.iter().map(|s| s.miter_limit));
        let e = NumericField::new(&tr("line-limit"), &tr("line-miter-limit"), limit)
            .decimals(1)
            .speed(0.1)
            .range(1.0..=20.0)
            .width(44.0)
            .show(ui);
        set(field_change(e, LineEdit::MiterLimit));
    }
    change
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashed_off_and_on_restores_the_lengths() {
        let mut s = LineStyle::default();
        let last = Dash {
            dash: 30.0,
            gap: 15.0,
        };
        LineEdit::Dashed(true, last).apply(&mut s);
        assert_eq!(s.dash, Some(last));
        LineEdit::Dashed(false, last).apply(&mut s);
        assert_eq!(s.dash, None);
        // Lengths only change a dashed style.
        LineEdit::Gap(40.0).apply(&mut s);
        assert_eq!(s.dash, None);
    }

    #[test]
    fn dotted_preset_rounds_the_caps() {
        let mut s = LineStyle {
            cap: Cap::Butt,
            ..LineStyle::default()
        };
        LineEdit::Preset(Preset::Dotted).apply(&mut s);
        assert_eq!(
            s.dash,
            Some(Dash {
                dash: 0.0,
                gap: 12.0
            })
        );
        assert_eq!(s.cap, Cap::Round);
    }
}
