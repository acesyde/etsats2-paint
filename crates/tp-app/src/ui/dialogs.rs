//! Modal dialogs: New Project wizard, Preferences, Keyboard Shortcuts, About.

use egui::{
    Align, Align2, CornerRadius, Frame, Key, Layout, Margin, RichText, Sense, Stroke, StrokeKind,
    TextEdit, Ui, Vec2, WidgetInfo, WidgetType,
};
use tp_core::{Project, TextureResolution};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::theme::{TEXT_SCALE_RANGE, UI_SCALE_RANGE, label_strong_style, title_style};
use tp_ui::tokens::{color, radius, space, stroke};
use tp_ui::widgets::{StepIndicator, paint_focus_ring, primary_button, secondary_button};

use crate::commands::{CommandId, ShortcutFormatter};
use crate::state::{AppState, Modal, NewProjectDraft};
use crate::vehicles::VehicleLibrary;

/// Titles of the New Project wizard steps. Vehicle steps will be inserted
/// before "Name & resolution".
pub const NEW_PROJECT_STEPS: [&str; 2] = ["new-project-step-vehicle", "new-project-step-name"];

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
    if matches!(state.modal, Some(Modal::VehicleLibrary(_))) {
        let Some(Modal::VehicleLibrary(mut dialog)) = state.modal.take() else {
            unreachable!()
        };
        if super::vehicle_dialogs::library(ctx, state, &mut dialog) && state.modal.is_none() {
            state.modal = Some(Modal::VehicleLibrary(dialog));
        }
        return;
    }
    if matches!(state.modal, Some(Modal::UpdateTemplate(_))) {
        let Some(Modal::UpdateTemplate(dialog)) = state.modal.take() else {
            unreachable!()
        };
        if super::vehicle_dialogs::update(ctx, state, &dialog) && state.modal.is_none() {
            state.modal = Some(Modal::UpdateTemplate(dialog));
        }
        return;
    }
    let Some(modal_kind) = state.modal.as_mut() else {
        return;
    };
    let close = match modal_kind {
        Modal::NewProject(draft) => match new_project(ctx, draft, &state.vehicles) {
            WizardOutcome::Pending => false,
            WizardOutcome::Cancel => true,
            WizardOutcome::Create(project) => {
                state.open_project(project);
                true
            }
            WizardOutcome::Install => {
                let paths = state.dialogs.pick_packages();
                let results = super::vehicle_dialogs::install(state, &paths);
                if let Some(Modal::NewProject(draft)) = &mut state.modal {
                    draft.messages = results;
                }
                false
            }
            WizardOutcome::CreateVehicle {
                name,
                id,
                version,
                variant,
            } => {
                let loaded = state.vehicles.load(&id, &version);
                match loaded.map(|p| crate::vehicle_project::vehicle_project(&name, &p, &variant)) {
                    Ok(Some(project)) => {
                        state.open_project(project);
                        true
                    }
                    Ok(None) => true,
                    Err(err) => {
                        state.modal = Some(Modal::Message {
                            title: tr("home-new-project"),
                            text: crate::vehicles::install_error_message(&err, &name),
                        });
                        false
                    }
                }
            }
        },
        Modal::Preferences => preferences(ctx, &mut state.prefs),
        Modal::KeyboardShortcuts => shortcuts(ctx),
        Modal::About => about(ctx),
        Modal::Message { title, text } => message(ctx, title, text),
        Modal::Export(_) | Modal::VehicleLibrary(_) | Modal::UpdateTemplate(_) => {
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
        ui.add_space(space::LG);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            close |= ui.add(primary_button(&tr("button-ok"))).clicked();
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
    /// Create a project from an installed vehicle.
    CreateVehicle {
        name: String,
        id: String,
        version: semver::Version,
        variant: String,
    },
    /// Install packages, then come back to the Vehicle step.
    Install,
}

/// Vehicle step: the installed vehicles (or a blank texture) and variants.
fn vehicle_step(ui: &mut Ui, draft: &mut NewProjectDraft, library: &VehicleLibrary) {
    draft.filter.show(ui, "wizard");
    ui.add_space(space::SM);
    for message in &draft.messages {
        let (text, tint) = match message {
            Ok(text) => (text, color::SUCCESS),
            Err(text) => (text, color::ERROR),
        };
        ui.label(RichText::new(text).small().color(tint));
    }
    egui::ScrollArea::vertical()
        .max_height(280.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            let blank = draft.vehicle.is_none();
            let row = ui.selectable_label(blank, tr("new-project-blank"));
            row.widget_info(|| {
                WidgetInfo::selected(
                    WidgetType::RadioButton,
                    true,
                    blank,
                    tr("new-project-blank"),
                )
            });
            if row.clicked() {
                draft.vehicle = None;
            }
            for vehicle in super::vehicle_dialogs::filtered(library, &draft.filter) {
                let newest = vehicle.newest();
                let m = &newest.manifest;
                let selected = draft
                    .vehicle
                    .as_ref()
                    .is_some_and(|(id, ..)| *id == vehicle.id);
                let text = format!("{}   {}", m.name, super::vehicle_dialogs::summary(m));
                let row = ui.selectable_label(selected, text);
                row.widget_info(|| {
                    WidgetInfo::selected(WidgetType::RadioButton, true, selected, &m.name)
                });
                if row.clicked() && !selected {
                    draft.vehicle = Some((
                        vehicle.id.clone(),
                        m.version.clone(),
                        m.variants[0].id.clone(),
                    ));
                }
                if selected && m.variants.len() > 1 {
                    ui.indent(("variants", &vehicle.id), |ui| {
                        for variant in &m.variants {
                            let on = draft
                                .vehicle
                                .as_ref()
                                .is_some_and(|(.., v)| *v == variant.id);
                            if ui.radio(on, &variant.name).clicked()
                                && let Some((.., v)) = &mut draft.vehicle
                            {
                                *v = variant.id.clone();
                            }
                        }
                    });
                }
            }
        });
}

/// The variant chosen in the draft, if it is installed.
fn chosen<'a>(
    draft: &NewProjectDraft,
    library: &'a VehicleLibrary,
) -> Option<(&'a tp_vehicles::Manifest, &'a tp_vehicles::Variant)> {
    let (id, version, variant) = draft.vehicle.as_ref()?;
    let installed = library
        .get(id)?
        .versions
        .iter()
        .find(|v| v.manifest.version == *version)?;
    let v = installed.manifest.variant(variant)?;
    Some((&installed.manifest, v))
}

fn new_project(
    ctx: &egui::Context,
    draft: &mut NewProjectDraft,
    library: &VehicleLibrary,
) -> WizardOutcome {
    let mut outcome = WizardOutcome::Pending;
    let last = draft.step >= NEW_PROJECT_STEPS.len() - 1;
    let response = modal("new_project_modal").show(ctx, |ui| {
        ui.set_width(560.0);
        ui.label(
            RichText::new(tr("home-new-project"))
                .text_style(title_style())
                .color(color::TEXT_PRIMARY),
        );
        ui.add_space(space::XS);
        let steps = NEW_PROJECT_STEPS.map(tr);
        let steps = steps.each_ref().map(String::as_str);
        StepIndicator::new(draft.step, &steps).show(ui);
        ui.add_space(space::LG);

        if draft.step == 0 {
            vehicle_step(ui, draft, library);
        } else {
            ui.label(RichText::new(tr("new-project-name")).text_style(label_strong_style()));
            let hint = chosen(draft, library)
                .map_or_else(|| tr("object-untitled"), |(m, _)| m.name.clone());
            let name = ui.add(
                TextEdit::singleline(&mut draft.name)
                    .hint_text(hint)
                    .desired_width(f32::INFINITY)
                    .margin(Margin::symmetric(8, 6)),
            );
            name.widget_info(|| {
                WidgetInfo::labeled(WidgetType::TextEdit, true, tr("new-project-name"))
            });
            if !draft.focus_requested {
                name.request_focus();
                draft.focus_requested = true;
            }
            ui.add_space(space::LG);
            match chosen(draft, library) {
                Some((_, variant)) => {
                    ui.label(
                        RichText::new(tr("new-project-textures")).text_style(label_strong_style()),
                    );
                    for t in &variant.textures {
                        ui.label(
                            RichText::new(format!("{} · {} × {} px", t.name, t.size, t.size))
                                .color(color::TEXT_SECONDARY),
                        );
                    }
                }
                None => {
                    ui.label(
                        RichText::new(tr("new-project-resolution"))
                            .text_style(label_strong_style()),
                    );
                    ui.add_space(space::XS);
                    ui.horizontal(|ui| {
                        for resolution in TextureResolution::ALL {
                            if resolution_card(ui, resolution, draft.resolution == resolution)
                                .clicked()
                            {
                                draft.resolution = resolution;
                            }
                        }
                    });
                    ui.label(
                        RichText::new(tr("new-project-resolution-hint"))
                            .small()
                            .color(color::TEXT_SECONDARY),
                    );
                }
            }
        }
        ui.add_space(space::XL);

        let (mut cancel, mut next, mut back, mut install) = (false, false, false, false);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let label = if last {
                tr("button-create")
            } else {
                tr("button-next")
            };
            next |= ui.add(primary_button(&label)).clicked();
            if draft.step > 0 {
                back |= ui.add(secondary_button(&tr("button-back"))).clicked();
            }
            cancel |= ui.add(secondary_button(&tr("button-cancel"))).clicked();
            if draft.step == 0 {
                install |= ui.add(secondary_button(&tr("vehicles-install"))).clicked();
            }
        });
        // Enter confirms the current step (a focused button handles Enter as
        // its own click above).
        if !cancel && !back && !install && ui.input(|i| i.key_pressed(Key::Enter)) {
            next = true;
        }
        (cancel, next, back, install)
    });

    let (cancel, next, back, install) = response.inner;
    let escape = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape));
    if cancel || escape {
        outcome = WizardOutcome::Cancel;
    } else if install {
        outcome = WizardOutcome::Install;
    } else if back {
        draft.step = draft.step.saturating_sub(1);
    } else if next && !last {
        draft.step += 1;
        draft.focus_requested = false;
    } else if next {
        let typed = draft.name.trim().to_owned();
        outcome = match (chosen(draft, library), draft.vehicle.clone()) {
            (Some((m, _)), Some((id, version, variant))) => WizardOutcome::CreateVehicle {
                name: if typed.is_empty() {
                    m.name.clone()
                } else {
                    typed
                },
                id,
                version,
                variant,
            },
            _ => {
                let name = if typed.is_empty() {
                    tr("object-untitled")
                } else {
                    draft.name.clone()
                };
                let mut project = Project::new(&name, draft.resolution);
                for surface in &mut project.surfaces {
                    surface.name = tr("object-main-texture");
                }
                WizardOutcome::Create(project)
            }
        };
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
        tr!("new-project-resolution-side", side = side.to_string()),
        egui::TextStyle::Body.resolve(ui.style()),
        color::TEXT_SECONDARY,
    );
    let note = match resolution {
        TextureResolution::R2048 => tr("resolution-light"),
        TextureResolution::R4096 => tr("resolution-recommended"),
        TextureResolution::R8192 => tr("resolution-maximum"),
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
            RichText::new(tr("home-preferences"))
                .text_style(title_style())
                .color(color::TEXT_PRIMARY),
        );
        ui.add_space(space::LG);
        ui.label(RichText::new(tr("prefs-interface")).text_style(label_strong_style()));
        ui.add_space(space::XS);
        egui::Grid::new("prefs_grid")
            .num_columns(2)
            .spacing([space::LG, space::SM])
            .show(ui, |ui| {
                ui.label(tr("prefs-ui-scale"));
                let mut ui_pct = (prefs.ui_scale * 100.0).round();
                let r = ui.add(
                    egui::Slider::new(
                        &mut ui_pct,
                        (UI_SCALE_RANGE.start() * 100.0)..=(UI_SCALE_RANGE.end() * 100.0),
                    )
                    .step_by(5.0)
                    .suffix("%")
                    .custom_formatter(|v, _| tp_i18n::format_number(v, 0)),
                );
                r.widget_info(|| {
                    WidgetInfo::labeled(WidgetType::Slider, true, tr("prefs-ui-scale"))
                });
                // Apply on release while dragging, so the slider does not move
                // under the pointer as the interface rescales.
                if r.changed() && !r.dragged() || r.drag_stopped() {
                    prefs.ui_scale = ui_pct / 100.0;
                }
                ui.end_row();

                ui.label(tr("prefs-text-size"));
                let mut text_pct = (prefs.text_scale * 100.0).round();
                let r = ui.add(
                    egui::Slider::new(
                        &mut text_pct,
                        (TEXT_SCALE_RANGE.start() * 100.0)..=(TEXT_SCALE_RANGE.end() * 100.0),
                    )
                    .step_by(5.0)
                    .suffix("%")
                    .custom_formatter(|v, _| tp_i18n::format_number(v, 0)),
                );
                r.widget_info(|| {
                    WidgetInfo::labeled(WidgetType::Slider, true, tr("prefs-text-size"))
                });
                if r.changed() {
                    prefs.text_scale = text_pct / 100.0;
                }
                ui.end_row();

                ui.label(tr("prefs-language"));
                let current = prefs.language();
                let name = |l: Option<tp_i18n::Language>| {
                    l.map_or_else(|| tr("language-system"), |l| l.native_name().to_owned())
                };
                let mut picked = current;
                let combo = egui::ComboBox::from_id_salt("prefs_language")
                    .width(160.0)
                    .selected_text(name(current))
                    .show_ui(ui, |ui| {
                        let choices = std::iter::once(None)
                            .chain(tp_i18n::Language::ALL.into_iter().map(Some));
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
                ui.end_row();
            });
        ui.add_space(space::XS);
        ui.label(
            RichText::new(tr("prefs-scale-hint"))
                .small()
                .color(color::TEXT_SECONDARY),
        );
        ui.add_space(space::LG);
        ui.label(RichText::new(tr("prefs-canvas")).text_style(label_strong_style()));
        ui.add_space(space::XS);
        egui::Grid::new("prefs_canvas_grid")
            .num_columns(2)
            .spacing([space::LG, space::SM])
            .show(ui, |ui| {
                ui.label(tr("prefs-grid-spacing"));
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
                ui.end_row();
            });
        ui.add_space(space::XL);
        let mut close = false;
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            close |= ui.add(primary_button(&tr("button-done"))).clicked();
            if ui.add(secondary_button(&tr("prefs-reset"))).clicked() {
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
            RichText::new(tr("cmd-keyboard-shortcuts"))
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
                                ui.label(tr(id.meta().label));
                                ui.label(RichText::new(shortcut).color(color::TEXT_SECONDARY));
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
        ui.add_space(space::LG);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(primary_button(&tr("button-close"))).clicked()
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
            RichText::new(tr!("about-version", version = env!("CARGO_PKG_VERSION")))
                .color(color::TEXT_SECONDARY),
        );
        ui.add_space(space::SM);
        ui.label(tr("about-tagline"));
        ui.add_space(space::LG);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(primary_button(&tr("button-close"))).clicked()
        })
        .inner
    });
    response.inner || response.should_close()
}
