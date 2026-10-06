//! Modal dialogs: New Project wizard, Preferences, Keyboard Shortcuts, About.

use egui::{
    Align, Align2, CornerRadius, Frame, Key, Layout, Margin, RichText, Sense, Stroke, StrokeKind,
    TextEdit, Ui, Vec2, WidgetInfo, WidgetType,
};
use tp_core::{DEFAULT_PROJECT_NAME, Project, TextureResolution};
use tp_ui::icons;
use tp_ui::theme::{TEXT_SCALE_RANGE, UI_SCALE_RANGE, label_strong_style, title_style};
use tp_ui::tokens::{color, radius, space, stroke};
use tp_ui::widgets::{StepIndicator, paint_focus_ring, primary_button, secondary_button};

use crate::commands::{CommandId, ShortcutFormatter};
use crate::state::{AppState, Modal, NewProjectDraft};

/// Titles of the New Project wizard steps. Vehicle steps will be inserted
/// before "Name & resolution".
pub const NEW_PROJECT_STEPS: [&str; 1] = ["Name & resolution"];

fn dialog_frame() -> Frame {
    Frame::new()
        .fill(color::SURFACE_2)
        .stroke(Stroke::new(1.0, color::BORDER_STRONG))
        .corner_radius(CornerRadius::same(radius::LG))
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
    let Some(modal_kind) = state.modal.as_mut() else {
        return;
    };
    let close = match modal_kind {
        Modal::NewProject(draft) => match new_project(ctx, draft) {
            WizardOutcome::Pending => false,
            WizardOutcome::Cancel => true,
            WizardOutcome::Create(project) => {
                state.open_project(project);
                true
            }
        },
        Modal::Preferences => preferences(ctx, &mut state.prefs),
        Modal::KeyboardShortcuts => shortcuts(ctx),
        Modal::About => about(ctx),
        Modal::Message { title, text } => message(ctx, title, text),
        Modal::Export(_) => unreachable!("handled above"),
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
                RichText::new(format!("Save changes to “{name}” before closing?"))
                    .text_style(label_strong_style())
                    .color(color::TEXT_PRIMARY),
            );
        });
        ui.add_space(space::XS);
        ui.label(
            RichText::new("Your changes will be lost if you don't save them.")
                .color(color::TEXT_SECONDARY),
        );
        ui.add_space(space::XL);
        ui.horizontal(|ui| {
            if ui.add(secondary_button("Don't Save")).clicked() {
                answer = PromptAnswer::DontSave;
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(primary_button("Save")).clicked() {
                    answer = PromptAnswer::Save;
                }
                if ui.add(secondary_button("Cancel")).clicked() {
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
        ui.add_space(space::LG);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            close |= ui.add(primary_button("OK")).clicked();
        });
    });
    close
        || ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::NONE, Key::Escape)
                || i.consume_key(egui::Modifiers::NONE, Key::Enter)
        })
}

enum WizardOutcome {
    Pending,
    Cancel,
    Create(Project),
}

fn new_project(ctx: &egui::Context, draft: &mut NewProjectDraft) -> WizardOutcome {
    let mut outcome = WizardOutcome::Pending;
    let response = modal("new_project_modal").show(ctx, |ui| {
        ui.set_width(520.0);
        ui.label(
            RichText::new("New Project")
                .text_style(title_style())
                .color(color::TEXT_PRIMARY),
        );
        ui.add_space(space::XS);
        StepIndicator::new(draft.step, &NEW_PROJECT_STEPS).show(ui);
        ui.add_space(space::LG);

        ui.label(RichText::new("Project name").text_style(label_strong_style()));
        let name = ui.add(
            TextEdit::singleline(&mut draft.name)
                .hint_text(DEFAULT_PROJECT_NAME)
                .desired_width(f32::INFINITY)
                .margin(Margin::symmetric(8, 6)),
        );
        name.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, "Project name"));
        if !draft.focus_requested {
            name.request_focus();
            draft.focus_requested = true;
        }
        ui.add_space(space::LG);

        ui.label(RichText::new("Texture resolution").text_style(label_strong_style()));
        ui.add_space(space::XS);
        ui.horizontal(|ui| {
            for resolution in TextureResolution::ALL {
                if resolution_card(ui, resolution, draft.resolution == resolution).clicked() {
                    draft.resolution = resolution;
                }
            }
        });
        ui.label(
            RichText::new(
                "The project stays vector-based: you can export at any resolution later.",
            )
            .small()
            .color(color::TEXT_SECONDARY),
        );
        ui.add_space(space::XL);

        let mut cancel = false;
        let mut create = false;
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            create |= ui.add(primary_button("Create")).clicked();
            cancel |= ui.add(secondary_button("Cancel")).clicked();
        });
        // Enter confirms from anywhere in the dialog (a focused Cancel button
        // handles Enter as its own click above).
        if !cancel && ui.input(|i| i.key_pressed(Key::Enter)) {
            create = true;
        }
        (cancel, create)
    });

    let (cancel, create) = response.inner;
    let escape = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape));
    if cancel || escape {
        outcome = WizardOutcome::Cancel;
    } else if create {
        outcome = WizardOutcome::Create(Project::new(&draft.name, draft.resolution));
    }
    outcome
}

fn resolution_card(ui: &mut Ui, resolution: TextureResolution, selected: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(156.0, 84.0), Sense::click());
    let label = resolution.label();
    response.widget_info(|| WidgetInfo::selected(WidgetType::RadioButton, true, selected, &label));

    let painter = ui.painter();
    let corner = CornerRadius::same(radius::MD);
    let fill = if selected {
        color::ACCENT_SUBTLE
    } else if response.hovered() {
        color::SURFACE_3
    } else {
        color::SURFACE_1
    };
    let outline = if selected {
        Stroke::new(stroke::FOCUS + 0.5, color::ACCENT)
    } else {
        Stroke::new(1.0, color::BORDER_STRONG)
    };
    painter.rect(rect, corner, fill, outline, StrokeKind::Inside);
    if selected {
        painter.text(
            rect.right_top() + Vec2::new(-14.0, 14.0),
            Align2::CENTER_CENTER,
            icons::CHECK,
            icons::font(14.0),
            color::ACCENT,
        );
    }
    let side = resolution.side();
    painter.text(
        rect.left_top() + Vec2::new(14.0, 24.0),
        Align2::LEFT_CENTER,
        format!("{side}"),
        title_style().resolve(ui.style()),
        color::TEXT_PRIMARY,
    );
    painter.text(
        rect.left_top() + Vec2::new(14.0, 48.0),
        Align2::LEFT_CENTER,
        format!("× {side} px"),
        egui::TextStyle::Body.resolve(ui.style()),
        color::TEXT_SECONDARY,
    );
    let note = match resolution {
        TextureResolution::R2048 => "Light & fast",
        TextureResolution::R4096 => "Recommended",
        TextureResolution::R8192 => "Maximum detail",
    };
    painter.text(
        rect.left_top() + Vec2::new(14.0, 68.0),
        Align2::LEFT_CENTER,
        note,
        egui::TextStyle::Small.resolve(ui.style()),
        if selected {
            color::TEXT_PRIMARY
        } else {
            color::TEXT_SECONDARY
        },
    );
    paint_focus_ring(ui, rect, &response, radius::MD);
    response
}

/// Returns true when the dialog should close.
fn preferences(ctx: &egui::Context, prefs: &mut crate::prefs::Prefs) -> bool {
    let response = modal("preferences_modal").show(ctx, |ui| {
        ui.set_width(440.0);
        ui.label(
            RichText::new("Preferences")
                .text_style(title_style())
                .color(color::TEXT_PRIMARY),
        );
        ui.add_space(space::LG);
        ui.label(RichText::new("Interface").text_style(label_strong_style()));
        ui.add_space(space::XS);
        egui::Grid::new("prefs_grid")
            .num_columns(2)
            .spacing([space::LG, space::SM])
            .show(ui, |ui| {
                ui.label("UI scale");
                let mut ui_pct = (prefs.ui_scale * 100.0).round();
                let r = ui.add(
                    egui::Slider::new(
                        &mut ui_pct,
                        (UI_SCALE_RANGE.start() * 100.0)..=(UI_SCALE_RANGE.end() * 100.0),
                    )
                    .step_by(5.0)
                    .suffix("%"),
                );
                r.widget_info(|| WidgetInfo::labeled(WidgetType::Slider, true, "UI scale"));
                // Apply on release while dragging, so the slider does not move
                // under the pointer as the interface rescales.
                if r.changed() && !r.dragged() || r.drag_stopped() {
                    prefs.ui_scale = ui_pct / 100.0;
                }
                ui.end_row();

                ui.label("Text size");
                let mut text_pct = (prefs.text_scale * 100.0).round();
                let r = ui.add(
                    egui::Slider::new(
                        &mut text_pct,
                        (TEXT_SCALE_RANGE.start() * 100.0)..=(TEXT_SCALE_RANGE.end() * 100.0),
                    )
                    .step_by(5.0)
                    .suffix("%"),
                );
                r.widget_info(|| WidgetInfo::labeled(WidgetType::Slider, true, "Text size"));
                if r.changed() {
                    prefs.text_scale = text_pct / 100.0;
                }
                ui.end_row();
            });
        ui.add_space(space::XS);
        ui.label(
            RichText::new("Changes apply immediately. 100% follows your display's scale factor.")
                .small()
                .color(color::TEXT_SECONDARY),
        );
        ui.add_space(space::LG);
        ui.label(RichText::new("Canvas").text_style(label_strong_style()));
        ui.add_space(space::XS);
        egui::Grid::new("prefs_canvas_grid")
            .num_columns(2)
            .spacing([space::LG, space::SM])
            .show(ui, |ui| {
                ui.label("Grid spacing");
                let range = crate::prefs::GRID_SPACING_RANGE;
                let r = ui.add(
                    egui::DragValue::new(&mut prefs.view_aids.grid_spacing)
                        .range(range)
                        .speed(1.0)
                        .max_decimals(0)
                        .suffix(" px"),
                );
                r.widget_info(|| WidgetInfo::labeled(WidgetType::DragValue, true, "Grid spacing"));
                ui.end_row();
            });
        ui.add_space(space::XL);
        let mut close = false;
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            close |= ui.add(primary_button("Done")).clicked();
            if ui.add(secondary_button("Reset to defaults")).clicked() {
                prefs.reset_scaling();
            }
        });
        close
    });
    response.inner || response.should_close()
}

fn shortcuts(ctx: &egui::Context) -> bool {
    let formatter = ShortcutFormatter::new(ctx);
    let response = modal("shortcuts_modal").show(ctx, |ui| {
        ui.set_width(460.0);
        ui.label(
            RichText::new("Keyboard Shortcuts")
                .text_style(title_style())
                .color(color::TEXT_PRIMARY),
        );
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
                            if let Some(shortcut) = formatter.command(id) {
                                ui.label(id.meta().label);
                                ui.label(RichText::new(shortcut).color(color::TEXT_SECONDARY));
                                ui.end_row();
                            }
                        }
                        ui.label("Temporary Hand tool");
                        ui.label(RichText::new("Hold Space").color(color::TEXT_SECONDARY));
                        ui.end_row();
                    });
            });
        ui.add_space(space::LG);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(primary_button("Close")).clicked()
        })
        .inner
    });
    response.inner || response.should_close()
}

fn about(ctx: &egui::Context) -> bool {
    let response = modal("about_modal").show(ctx, |ui| {
        ui.set_width(360.0);
        ui.horizontal(|ui| {
            ui.label(icons::rich(icons::VEHICLE).size(32.0).color(color::ACCENT));
            ui.label(
                RichText::new(crate::paths::APP_NAME)
                    .text_style(title_style())
                    .color(color::TEXT_PRIMARY),
            );
        });
        ui.label(
            RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION")))
                .color(color::TEXT_SECONDARY),
        );
        ui.add_space(space::SM);
        ui.label("Livery editor for Euro Truck Simulator 2 and American Truck Simulator.");
        ui.add_space(space::LG);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(primary_button("Close")).clicked()
        })
        .inner
    });
    response.inner || response.should_close()
}
