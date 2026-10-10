//! Modal dialogs: Preferences, Keyboard Shortcuts, About, messages, and
//! the pieces every dialog shares (title, labelled fields, problems,
//! footer).

use egui::{
    Align, CornerRadius, Frame, Key, Layout, Margin, RichText, Stroke, TextEdit, Ui, WidgetInfo,
    WidgetType,
};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::theme::{TEXT_SCALE_RANGE, UI_SCALE_RANGE, label_strong_style, title_style};
use tp_ui::tokens::{color, radius, space};
use tp_ui::widgets::{primary_button, secondary_button};

use crate::commands::{CommandId, ShortcutFormatter};
use crate::state::{AppState, Modal};

fn dialog_frame() -> Frame {
    Frame::new()
        .fill(color::SURFACE_1)
        .stroke(Stroke::new(1.0, color::OUTLINE))
        .corner_radius(CornerRadius::same(radius::DIALOG))
        .inner_margin(Margin::same(space::XL as i8))
        .shadow(egui::Shadow {
            offset: [0, 12],
            blur: 32,
            spread: 0,
            color: color::SHADOW,
        })
}

pub(crate) fn modal(id: &str) -> egui::Modal {
    egui::Modal::new(egui::Id::new(id))
        .frame(dialog_frame())
        .backdrop_color(color::BACKDROP)
}

/// A dialog's title, with the line under it (what it applies to), if any.
pub(crate) fn title(ui: &mut Ui, title: &str, subtitle: Option<&str>) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = space::XS;
        ui.label(
            RichText::new(title)
                .text_style(title_style())
                .color(color::TEXT_PRIMARY),
        );
        if let Some(subtitle) = subtitle {
            ui.label(RichText::new(subtitle).color(color::TEXT_SECONDARY));
        }
    });
}

/// The label above a field.
pub(crate) fn field_label(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).small().color(color::TEXT_SECONDARY));
}

/// A field under its label: labels sit above their fields, so a longer
/// translation never squeezes the field.
pub(crate) fn labelled<R>(ui: &mut Ui, label: &str, body: impl FnOnce(&mut Ui) -> R) -> R {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = space::XS;
        field_label(ui, label);
        body(ui)
    })
    .inner
}

/// Height of a dialog's single-line fields.
pub(crate) const FIELD_HEIGHT: f32 = 34.0;

/// How a [`committed_text`] field shows its text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextKind {
    Single,
    /// Single line, in the monospace face (versions).
    Mono,
    /// Several lines: Enter starts a new line.
    Multiline,
}

/// What a [`committed_text`] field did this frame.
pub(crate) struct Committed {
    pub response: egui::Response,
    /// The text being typed, while the field has the focus.
    pub typing: Option<String>,
    /// The text to commit: the field was left (Enter on a single line,
    /// Tab, a click elsewhere), not with Escape.
    pub committed: Option<String>,
    /// The field was left this frame, committed or not.
    pub left: bool,
}

/// A text field committed when it is left: it shows `current` unless it
/// has the focus, when it shows what is typed (kept in egui's temporary
/// data under `id`, so Undo and Redo show at once). Enter on a single line
/// and leaving it commit; Escape restores `current`. As tall as the
/// dialogs' fields (34 pt), named `label` for assistive technologies.
/// `width`: its width, the available width when `None`.
pub(crate) fn committed_text(
    ui: &mut Ui,
    id: egui::Id,
    label: &str,
    current: &str,
    kind: TextKind,
    hint: &str,
    width: Option<f32>,
) -> Committed {
    tp_ui::widgets::remember_escape(ui, id);
    let focused = ui.memory(|m| m.has_focus(id));
    let mut buffer = ui
        .data(|d| d.get_temp::<String>(id))
        .unwrap_or_else(|| current.to_owned());
    let width = width.unwrap_or(f32::INFINITY);
    let edit = match kind {
        TextKind::Multiline => TextEdit::multiline(&mut buffer)
            .desired_rows(2)
            .margin(Margin::symmetric(10, 8)),
        TextKind::Single | TextKind::Mono => TextEdit::singleline(&mut buffer)
            .margin(Margin::symmetric(10, 0))
            .vertical_align(Align::Center)
            .min_size(egui::vec2(0.0, FIELD_HEIGHT)),
    };
    let mut edit = edit.id(id).desired_width(width).hint_text(hint);
    if kind == TextKind::Mono {
        edit = edit.font(egui::TextStyle::Monospace);
    }
    let response = ui.add(edit);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, label));
    let mut out = Committed {
        typing: None,
        committed: None,
        left: false,
        response,
    };
    if out.response.has_focus() {
        ui.data_mut(|d| d.insert_temp(id, buffer.clone()));
        out.typing = Some(buffer);
    } else if out.response.lost_focus() {
        ui.data_mut(|d| d.remove::<String>(id));
        out.left = true;
        if !tp_ui::widgets::take_escape(ui, id) {
            out.committed = Some(buffer);
        }
    } else if !focused {
        // Left without this field seeing it (it wasn't shown): what it
        // held was committed by then.
        ui.data_mut(|d| d.remove::<String>(id));
    }
    out
}

/// `columns` side by side, each `ratio` of the width (the gaps aside).
pub(crate) fn columns(ui: &mut Ui, ratios: &[f32], mut body: impl FnMut(&mut Ui, usize)) {
    let gap = space::MD;
    let total: f32 = ratios.iter().sum();
    let width = ui.available_width() - gap * (ratios.len().saturating_sub(1)) as f32;
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for (i, ratio) in ratios.iter().enumerate() {
            let w = (width * ratio / total).floor();
            ui.allocate_ui_with_layout(egui::vec2(w, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(w);
                body(ui, i);
            });
        }
    });
}

/// A problem that blocks the dialog's action: an icon and its text.
pub(crate) fn problem(ui: &mut Ui, text: &str) {
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM;
        ui.label(icons::rich(icons::WARNING).color(color::ERROR));
        let label = ui.add(egui::Label::new(RichText::new(text).color(color::ERROR)).wrap());
        label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
    });
}

/// A warning that doesn't block the action, with its own icon color so it
/// can't be mistaken for a [`problem`].
pub(crate) fn warning(ui: &mut Ui, text: &str) {
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = space::SM;
        ui.label(icons::rich(icons::WARNING).color(color::WARNING));
        let label =
            ui.add(egui::Label::new(RichText::new(text).color(color::TEXT_SECONDARY)).wrap());
        label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
    });
}

/// Results of the last action (packages installed, files written): a
/// success or an error each, with an icon so they don't rely on color.
pub(crate) fn messages(ui: &mut Ui, messages: &[Result<String, String>]) {
    for message in messages {
        let (icon, text, tint) = match message {
            Ok(text) => (icons::CHECK, text, color::SUCCESS),
            Err(text) => (icons::WARNING, text, color::ERROR),
        };
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = space::SM;
            ui.label(icons::rich(icon).color(tint));
            let label = ui.add(egui::Label::new(RichText::new(text).color(tint)).wrap());
            label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
        });
    }
}

/// The dialog's buttons, right-aligned under a hairline, the primary
/// action last (rightmost): `body` adds them from the right.
pub(crate) fn footer<R>(ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> R {
    ui.add_space(space::LG);
    let rect = ui.available_rect_before_wrap();
    ui.painter()
        .hline(rect.x_range(), rect.top(), Stroke::new(1.0, color::BORDER));
    ui.add_space(space::LG);
    ui.with_layout(Layout::right_to_left(Align::Center), body)
        .inner
}

/// Shows the active modal, if any.
pub fn show_modal(ctx: &egui::Context, state: &mut AppState) {
    if matches!(state.modal, Some(Modal::Export(_))) {
        let Some(Modal::Export(mut dialog)) = state.modal.take() else {
            unreachable!()
        };
        let keep = super::export_dialog::show(ctx, state, &mut dialog);
        // The dialog may have opened another modal (an error message).
        if keep && state.modal.is_none() {
            state.modal = Some(Modal::Export(dialog));
        }
        return;
    }
    if matches!(state.modal, Some(Modal::ExportMod(_))) {
        let Some(Modal::ExportMod(mut dialog)) = state.modal.take() else {
            unreachable!()
        };
        let keep = super::mod_export_dialog::show(ctx, state, &mut dialog);
        if keep && state.modal.is_none() {
            state.modal = Some(Modal::ExportMod(dialog));
        }
        return;
    }
    if matches!(state.modal, Some(Modal::CopyFromCabin(_))) {
        let Some(Modal::CopyFromCabin(mut dialog)) = state.modal.take() else {
            unreachable!()
        };
        if super::vehicle_dialogs::copy_from_cabin(ctx, state, &mut dialog) && state.modal.is_none()
        {
            state.modal = Some(Modal::CopyFromCabin(dialog));
        }
        return;
    }
    if matches!(state.modal, Some(Modal::CustomVehicle(_))) {
        super::custom_vehicle::show_modal(ctx, state);
        return;
    }
    if matches!(state.modal, Some(Modal::NewProject(_))) {
        super::new_project::show_modal(ctx, state);
        return;
    }
    if matches!(state.modal, Some(Modal::VehicleLibrary(_))) {
        let Some(Modal::VehicleLibrary(mut dialog)) = state.modal.take() else {
            unreachable!()
        };
        if super::vehicle_dialogs::library(ctx, state, &mut dialog) && state.modal.is_none() {
            state.modal = Some(Modal::VehicleLibrary(dialog));
        }
        return;
    }
    if matches!(state.modal, Some(Modal::ImportFromLibrary(_))) {
        let Some(Modal::ImportFromLibrary(mut dialog)) = state.modal.take() else {
            unreachable!()
        };
        if super::library_dialog::show(ctx, state, &mut dialog) && state.modal.is_none() {
            state.modal = Some(Modal::ImportFromLibrary(dialog));
        }
        return;
    }
    if matches!(state.modal, Some(Modal::UpdateTemplate(_))) {
        let Some(Modal::UpdateTemplate(mut dialog)) = state.modal.take() else {
            unreachable!()
        };
        if super::vehicle_dialogs::update(ctx, state, &mut dialog) && state.modal.is_none() {
            state.modal = Some(Modal::UpdateTemplate(dialog));
        }
        return;
    }
    if matches!(state.modal, Some(Modal::AddVehicle(_))) {
        let Some(Modal::AddVehicle(mut dialog)) = state.modal.take() else {
            unreachable!()
        };
        if super::vehicle_dialogs::add_vehicle(ctx, state, &mut dialog) && state.modal.is_none() {
            state.modal = Some(Modal::AddVehicle(dialog));
        }
        return;
    }
    if matches!(state.modal, Some(Modal::Textures(_))) {
        let Some(Modal::Textures(mut dialog)) = state.modal.take() else {
            unreachable!()
        };
        if super::vehicle_dialogs::textures(ctx, state, &mut dialog) && state.modal.is_none() {
            state.modal = Some(Modal::Textures(dialog));
        }
        return;
    }
    if matches!(state.modal, Some(Modal::RemoveVehicle { .. })) {
        let Some(Modal::RemoveVehicle { package_id, name }) = state.modal.take() else {
            unreachable!()
        };
        if super::vehicle_dialogs::remove_vehicle(ctx, state, &package_id, &name)
            && state.modal.is_none()
        {
            state.modal = Some(Modal::RemoveVehicle { package_id, name });
        }
        return;
    }
    let Some(modal_kind) = state.modal.as_mut() else {
        return;
    };
    let close = match modal_kind {
        Modal::Preferences => preferences(ctx, &mut state.prefs),
        Modal::KeyboardShortcuts => shortcuts(ctx),
        Modal::About => about(ctx),
        Modal::Message { title, text } => message(ctx, title, text),
        Modal::NewProject(_)
        | Modal::Export(_)
        | Modal::ExportMod(_)
        | Modal::VehicleLibrary(_)
        | Modal::ImportFromLibrary(_)
        | Modal::UpdateTemplate(_)
        | Modal::AddVehicle(_)
        | Modal::Textures(_)
        | Modal::CustomVehicle(_)
        | Modal::CopyFromCabin(_)
        | Modal::RemoveVehicle { .. } => {
            unreachable!("handled above")
        }
        Modal::UnsavedChanges(action) => {
            let action = action.clone();
            let name = state
                .workspace()
                .map(|ws| ws.project.name.clone())
                .unwrap_or_default();
            match unsaved_changes(ctx, &name) {
                PromptAnswer::Pending => false,
                answer => {
                    state.modal = None;
                    match answer {
                        PromptAnswer::Save => state.save_then(ctx, action),
                        PromptAnswer::DontSave => state.run_action(ctx, action),
                        PromptAnswer::Cancel | PromptAnswer::Pending => {}
                    }
                    return;
                }
            }
        }
    };
    if close {
        state.modal = None;
    }
}

enum PromptAnswer {
    Pending,
    Save,
    DontSave,
    Cancel,
}

/// "Save changes to “<name>” before closing?" with Save / Don't Save / Cancel.
fn unsaved_changes(ctx: &egui::Context, name: &str) -> PromptAnswer {
    let mut answer = PromptAnswer::Pending;
    modal("unsaved_changes_modal").show(ctx, |ui| {
        ui.set_width(440.0);
        ui.horizontal(|ui| {
            ui.label(icons::rich(icons::WARNING).size(22.0).color(color::WARNING));
            ui.label(
                RichText::new(tr!("unsaved-title", name = name))
                    .text_style(label_strong_style())
                    .color(color::TEXT_PRIMARY),
            );
        });
        ui.add_space(space::XS);
        ui.label(RichText::new(tr("unsaved-hint")).color(color::TEXT_SECONDARY));
        ui.add_space(space::XL);
        ui.horizontal(|ui| {
            if ui.add(secondary_button(&tr("button-dont-save"))).clicked() {
                answer = PromptAnswer::DontSave;
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(primary_button(&tr("cmd-save"))).clicked() {
                    answer = PromptAnswer::Save;
                }
                if ui.add(secondary_button(&tr("button-cancel"))).clicked() {
                    answer = PromptAnswer::Cancel;
                }
            });
        });
    });
    if matches!(answer, PromptAnswer::Pending) {
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape)) {
            answer = PromptAnswer::Cancel;
        } else if ctx.memory(|m| m.focused().is_none())
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter))
        {
            answer = PromptAnswer::Save;
        }
    }
    answer
}

/// An error or information message with an OK button.
fn message(ctx: &egui::Context, title: &str, text: &str) -> bool {
    let mut close = false;
    modal("message_modal").show(ctx, |ui| {
        ui.set_width(420.0);
        ui.horizontal(|ui| {
            ui.label(icons::rich(icons::WARNING).size(22.0).color(color::WARNING));
            ui.label(
                RichText::new(title)
                    .text_style(label_strong_style())
                    .color(color::TEXT_PRIMARY),
            );
        });
        ui.add_space(space::XS);
        let label = ui.label(RichText::new(text).color(color::TEXT_SECONDARY));
        label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
        footer(ui, |ui| {
            close |= ui.add(primary_button(&tr("button-ok"))).clicked();
        });
    });
    close
        || ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::NONE, Key::Escape)
                || i.consume_key(egui::Modifiers::NONE, Key::Enter)
        })
}

/// A scale in percent: a thin slider by steps of 5 % and its value, which
/// can be dragged or typed. Returns the slider's response and whether the
/// value was changed through the value.
fn scale_slider(
    ui: &mut Ui,
    pct: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    name: &str,
    width: f32,
) -> (egui::Response, bool) {
    let r = ui.add(
        tp_ui::widgets::ThinSlider::new(pct, range.clone(), name)
            .width(width)
            .step(5.0),
    );
    let value = ui.add(
        egui::DragValue::new(pct)
            .range(range)
            .speed(1.0)
            .suffix("%")
            .custom_formatter(|v, _| tp_i18n::format_number(v, 0)),
    );
    (r, value.changed())
}

/// Returns true when the dialog should close.
fn preferences(ctx: &egui::Context, prefs: &mut crate::prefs::Prefs) -> bool {
    let response = modal("preferences_modal").show(ctx, |ui| {
        ui.set_width(440.0);
        title(ui, &tr("home-preferences"), None);
        ui.add_space(space::LG);
        ui.spacing_mut().item_spacing.y = space::MD;
        section(ui, &tr("prefs-interface"));
        let slider_width = ui.available_width() - 60.0;
        labelled(ui, &tr("prefs-ui-scale"), |ui| {
            let mut ui_pct = (prefs.ui_scale * 100.0).round();
            let range = (UI_SCALE_RANGE.start() * 100.0)..=(UI_SCALE_RANGE.end() * 100.0);
            let name = tr("prefs-ui-scale");
            let (r, typed) = scale_slider(ui, &mut ui_pct, range, &name, slider_width);
            // Apply on release while dragging, so the slider does not move
            // under the pointer as the interface rescales.
            if r.changed() && !r.dragged() || r.drag_stopped() || typed {
                prefs.ui_scale = ui_pct / 100.0;
            }
        });
        labelled(ui, &tr("prefs-text-size"), |ui| {
            let mut text_pct = (prefs.text_scale * 100.0).round();
            let range = (TEXT_SCALE_RANGE.start() * 100.0)..=(TEXT_SCALE_RANGE.end() * 100.0);
            let name = tr("prefs-text-size");
            let (r, typed) = scale_slider(ui, &mut text_pct, range, &name, slider_width);
            if r.changed() || typed {
                prefs.text_scale = text_pct / 100.0;
            }
        });
        ui.label(
            RichText::new(tr("prefs-scale-hint"))
                .small()
                .color(color::TEXT_SECONDARY),
        );
        labelled(ui, &tr("prefs-language"), |ui| {
            let current = prefs.language();
            let name = |l: Option<tp_i18n::Language>| {
                l.map_or_else(|| tr("language-system"), |l| l.native_name().to_owned())
            };
            let mut picked = current;
            let combo = egui::ComboBox::from_id_salt("prefs_language")
                .icon(tp_ui::widgets::dropdown_icon)
                .width(220.0)
                .selected_text(name(current))
                .show_ui(ui, |ui| {
                    let choices =
                        std::iter::once(None).chain(tp_i18n::Language::ALL.into_iter().map(Some));
                    for choice in choices {
                        ui.selectable_value(&mut picked, choice, name(choice));
                    }
                });
            combo.response.widget_info(|| {
                WidgetInfo::labeled(WidgetType::ComboBox, true, tr("prefs-language"))
            });
            if picked != current {
                prefs.set_language(picked);
                ctx.request_repaint();
            }
        });
        ui.add_space(space::SM);
        section(ui, &tr("prefs-canvas"));
        labelled(ui, &tr("prefs-grid-spacing"), |ui| {
            let range = crate::prefs::GRID_SPACING_RANGE;
            let r = ui.add(
                egui::DragValue::new(&mut prefs.view_aids.grid_spacing)
                    .range(range)
                    .speed(1.0)
                    .max_decimals(0)
                    .suffix(" px"),
            );
            r.widget_info(|| {
                WidgetInfo::labeled(WidgetType::DragValue, true, tr("prefs-grid-spacing"))
            });
        });
        let mut close = false;
        footer(ui, |ui| {
            close |= ui.add(primary_button(&tr("button-done"))).clicked();
            if ui.add(secondary_button(&tr("prefs-reset"))).clicked() {
                prefs.reset_scaling();
            }
        });
        close
    });
    response.inner || response.should_close()
}

/// A section's heading in a dialog ("Interface", "Canvas").
fn section(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .text_style(label_strong_style())
            .color(color::TEXT_PRIMARY),
    );
}

fn shortcuts(ctx: &egui::Context) -> bool {
    let formatter = ShortcutFormatter::new(ctx);
    let response = modal("shortcuts_modal").show(ctx, |ui| {
        ui.set_width(480.0);
        title(ui, &tr("cmd-keyboard-shortcuts"), None);
        ui.add_space(space::MD);
        egui::ScrollArea::vertical()
            .max_height(420.0)
            .show(ui, |ui| {
                egui::Grid::new("shortcuts_grid")
                    .num_columns(2)
                    .striped(true)
                    .spacing([space::XL, space::XS + 2.0])
                    .show(ui, |ui| {
                        for id in CommandId::all() {
                            // Every key of the command, alternates included.
                            let keys: Vec<String> = id
                                .meta()
                                .shortcuts
                                .iter()
                                .map(|s| formatter.format(s))
                                .collect();
                            if !keys.is_empty() {
                                ui.label(tr(id.meta().label));
                                ui.label(
                                    RichText::new(keys.join(", "))
                                        .monospace()
                                        .color(color::TEXT_SECONDARY),
                                );
                                ui.end_row();
                            }
                        }
                        ui.label(tr("shortcuts-temporary-hand"));
                        ui.label(
                            RichText::new(tr("shortcuts-hold-space")).color(color::TEXT_SECONDARY),
                        );
                        ui.end_row();
                    });
            });
        footer(ui, |ui| {
            ui.add(primary_button(&tr("button-close"))).clicked()
        })
    });
    response.inner || response.should_close()
}

fn about(ctx: &egui::Context) -> bool {
    let response = modal("about_modal").show(ctx, |ui| {
        ui.set_width(360.0);
        ui.horizontal(|ui| {
            ui.label(
                icons::rich(icons::VEHICLE)
                    .size(32.0)
                    .color(color::TEXT_PRIMARY),
            );
            ui.label(
                RichText::new(crate::paths::APP_NAME)
                    .text_style(title_style())
                    .color(color::TEXT_PRIMARY),
            );
        });
        ui.label(
            RichText::new(tr!("about-version", version = env!("CARGO_PKG_VERSION")))
                .color(color::TEXT_SECONDARY),
        );
        ui.add_space(space::SM);
        ui.label(tr("about-tagline"));
        footer(ui, |ui| {
            ui.add(primary_button(&tr("button-close"))).clicked()
        })
    });
    response.inner || response.should_close()
}
