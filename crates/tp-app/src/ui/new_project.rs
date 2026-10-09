//! The New Project dialog, on one page: the project's name and game on
//! top; the installed vehicles of that game on the left, with Custom
//! vehicle… and Install…; "Your fleet" on the right, with the chosen
//! vehicle's textures to tick and the textures that will be created.

use egui::{
    Align, Align2, CornerRadius, Frame, Key, Layout, Margin, Rect, RichText, ScrollArea, Sense,
    Stroke, StrokeKind, TextEdit, Ui, Vec2, WidgetInfo, WidgetType,
};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::theme::label_strong_style;
use tp_ui::tokens::{color, radius, space, typography};
use tp_ui::widgets::{SegmentedControl, paint_focus_ring, primary_button, secondary_button};
use tp_vehicles::{Game, Kind, Manifest};

use super::dialogs::{field_label, footer, labelled, messages, modal, title};
use super::vehicle_dialogs::{
    VehicleChoice, filtered, game_name, kind_label, summary, textures_to_paint,
};
use crate::state::{AppState, Modal, NewProjectDraft};
use crate::vehicles::{SAMPLE_ID, VehicleLibrary};

/// Width of the "Your fleet" column.
const FLEET_WIDTH: f32 = 340.0;
/// Height of a row of the vehicle list.
const ROW_HEIGHT: f32 = 44.0;

/// What the dialog asks for this frame.
enum Outcome {
    Pending,
    Cancel,
    /// Create a project from an installed vehicle and its checked textures.
    Create {
        name: String,
        choice: VehicleChoice,
    },
    /// Install packages, then come back.
    Install,
    /// Install the built-in sample vehicles and choose the truck.
    InstallSample,
    /// Open the Custom Vehicle dialog, then come back.
    CustomVehicle,
}

/// Shows the New Project dialog held in `state.modal` and runs what it
/// asks for.
pub fn show_modal(ctx: &egui::Context, state: &mut AppState) {
    let Some(Modal::NewProject(draft)) = state.modal.as_mut() else {
        return;
    };
    match show(ctx, draft, &state.vehicles) {
        Outcome::Pending => {}
        Outcome::Cancel => state.modal = None,
        Outcome::Install => {
            let paths = state.dialogs.pick_packages();
            let results = super::vehicle_dialogs::install(state, &paths);
            if let Some(Modal::NewProject(draft)) = &mut state.modal {
                draft.messages = results;
            }
        }
        Outcome::CustomVehicle => {
            if let Some(Modal::NewProject(draft)) = state.modal.take() {
                super::custom_vehicle::open(
                    state,
                    super::custom_vehicle::Origin::NewProject(draft),
                );
            }
        }
        Outcome::InstallSample => {
            let results = super::vehicle_dialogs::install_samples(state);
            let sample = results.first().is_some_and(Result::is_ok);
            let pick = state
                .vehicles
                .vehicles()
                .iter()
                .find(|v| sample && v.newest().manifest.id == SAMPLE_ID)
                .map(|v| (VehicleChoice::of(v), v.newest().manifest.game.id));
            if let Some(Modal::NewProject(draft)) = &mut state.modal {
                draft.messages = results;
                if let Some((choice, game)) = pick {
                    draft.filter.game = Some(game);
                    draft.vehicle = Some(choice);
                }
            }
        }
        Outcome::Create { name, choice } => {
            let loaded = state.vehicles.load(&choice.id, &choice.version);
            match loaded.map(|p| crate::vehicle_project::fleet_project(&name, &p, &choice.textures))
            {
                Ok(Ok(project)) => {
                    state.modal = None;
                    state.open_project(project);
                }
                Ok(Err(_)) => state.modal = None,
                Err(err) => {
                    state.modal = Some(Modal::Message {
                        title: tr("home-new-project"),
                        text: crate::vehicles::install_error_message(&err, &name),
                    });
                }
            }
        }
    }
}

/// The vehicle chosen in the draft, if it is installed and ready.
fn chosen<'a>(draft: &NewProjectDraft, library: &'a VehicleLibrary) -> Option<&'a Manifest> {
    let choice = draft.vehicle.as_ref()?;
    let m = choice.manifest(library)?;
    choice.is_complete(library).then_some(m)
}

fn show(ctx: &egui::Context, draft: &mut NewProjectDraft, library: &VehicleLibrary) -> Outcome {
    // The game is always one of the two (a draft made by hand has none).
    let game = draft.game();
    draft.filter.game = Some(game);
    let screen = ctx.content_rect();
    let width = (screen.width() - 96.0).clamp(720.0, 1080.0);
    let body_height = (screen.height() - 300.0).clamp(300.0, 560.0);

    let response = modal("new_project_modal").show(ctx, |ui| {
        ui.set_width(width);
        title(ui, &tr("home-new-project"), None);
        ui.add_space(space::LG);
        header(ui, draft, library);
        ui.add_space(space::LG);

        let mut wanted = Wanted::default();
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = space::XL;
            let list_width = ui.available_width() - FLEET_WIDTH - space::XL;
            ui.allocate_ui_with_layout(
                Vec2::new(list_width, body_height),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.set_width(list_width);
                    vehicle_column(ui, draft, library, body_height, &mut wanted);
                },
            );
            ui.allocate_ui_with_layout(
                Vec2::new(FLEET_WIDTH, body_height),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.set_width(FLEET_WIDTH);
                    ui.set_min_height(body_height);
                    fleet(ui, draft, library);
                },
            );
        });

        let ready = chosen(draft, library).is_some();
        let mut cancel = false;
        let mut create = false;
        footer(ui, |ui| {
            create |= ui
                .add_enabled(ready, primary_button(&tr("new-project-create")))
                .on_disabled_hover_text(tr("reason-choose-vehicle"))
                .clicked();
            cancel |= ui.add(secondary_button(&tr("button-cancel"))).clicked();
        });
        // Enter creates the project (a focused button handles Enter as its
        // own click above).
        if ready && !cancel && !wanted.any() && ui.input(|i| i.key_pressed(Key::Enter)) {
            create = true;
        }
        (cancel, create, wanted)
    });

    let (cancel, create, wanted) = response.inner;
    let escape = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape));
    if cancel || escape {
        Outcome::Cancel
    } else if wanted.install {
        Outcome::Install
    } else if wanted.sample {
        Outcome::InstallSample
    } else if wanted.custom {
        Outcome::CustomVehicle
    } else if create
        && let Some(m) = chosen(draft, library)
        && let Some(choice) = draft.vehicle.clone()
    {
        let typed = draft.name.trim().to_owned();
        Outcome::Create {
            name: if typed.is_empty() {
                m.name.clone()
            } else {
                typed
            },
            choice,
        }
    } else {
        Outcome::Pending
    }
}

/// Buttons of the vehicle column clicked this frame.
#[derive(Default)]
struct Wanted {
    install: bool,
    sample: bool,
    custom: bool,
}

impl Wanted {
    fn any(&self) -> bool {
        self.install || self.sample || self.custom
    }
}

/// The project's name and game, and the note that vehicles can be added
/// later.
fn header(ui: &mut Ui, draft: &mut NewProjectDraft, library: &VehicleLibrary) {
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = space::XL;
        ui.allocate_ui_with_layout(Vec2::new(320.0, 0.0), Layout::top_down(Align::Min), |ui| {
            ui.set_width(320.0);
            labelled(ui, &tr("new-project-name"), |ui| {
                let hint = chosen(draft, library)
                    .map_or_else(|| tr("object-untitled"), |m| m.name.clone());
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
            });
        });
        labelled(ui, &tr("custom-game"), |ui| {
            let (ets2, ats) = (game_name(Game::Ets2), game_name(Game::Ats));
            let picked = SegmentedControl::new()
                .named_segment(Game::Ets2, "", "ETS2", &ets2)
                .named_segment(Game::Ats, "", "ATS", &ats)
                .show(ui, draft.game());
            if let Some(game) = picked {
                draft.filter.game = Some(game);
                // A vehicle of the other game is no longer listed.
                if draft
                    .vehicle
                    .as_ref()
                    .and_then(|c| c.manifest(library))
                    .is_some_and(|m| m.game.id != game)
                {
                    draft.vehicle = None;
                }
            }
        });
        // Level with the fields, under their labels.
        ui.vertical(|ui| {
            ui.add_space(ui.text_style_height(&egui::TextStyle::Small) + space::XS + 6.0);
            ui.add(
                egui::Label::new(
                    RichText::new(tr("new-project-add-later")).color(color::TEXT_SECONDARY),
                )
                .wrap(),
            );
        });
    });
}

/// Search, kind filter and Install…, the vehicle list, then the Custom
/// vehicle… entry.
fn vehicle_column(
    ui: &mut Ui,
    draft: &mut NewProjectDraft,
    library: &VehicleLibrary,
    height: f32,
    wanted: &mut Wanted,
) {
    let top = ui.cursor().top();
    ui.horizontal(|ui| {
        let search = ui.add(
            TextEdit::singleline(&mut draft.filter.query)
                .hint_text(format!("{} {}", icons::SEARCH, tr("vehicles-search")))
                .desired_width(220.0)
                .margin(Margin::symmetric(8, 5)),
        );
        search
            .widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, tr("vehicles-search")));
        let (all, trucks, trailers) = (
            tr("vehicles-kind-all"),
            tr("vehicles-kind-trucks"),
            tr("vehicles-kind-trailers"),
        );
        if let Some(kind) = SegmentedControl::new()
            .segment(None, "", &all, None)
            .segment(Some(Kind::Truck), "", &trucks, None)
            .segment(Some(Kind::Trailer), "", &trailers, None)
            .show(ui, draft.filter.kind)
        {
            draft.filter.kind = kind;
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            wanted.install |= ui.add(secondary_button(&tr("vehicles-install"))).clicked();
        });
    });
    ui.add_space(space::SM);
    messages(ui, &draft.messages);

    // The entry under the list keeps its place: the list takes the rest.
    let entry_height = 78.0;
    let list_height = (height - (ui.cursor().top() - top) - entry_height - space::MD).max(120.0);
    Frame::new()
        .fill(color::SURFACE_1)
        .stroke(Stroke::new(1.0, color::BORDER))
        .corner_radius(radius::LG)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(list_height);
            if library.vehicles().is_empty() {
                no_vehicle(ui, wanted);
            } else {
                list(ui, draft, library, list_height);
            }
        });
    ui.add_space(space::MD);
    custom_entry(ui, wanted);
}

/// With no vehicle installed: what packages are, and the sample vehicles.
fn no_vehicle(ui: &mut Ui, wanted: &mut Wanted) {
    ui.vertical_centered(|ui| {
        ui.add_space(space::XXL);
        ui.label(
            icons::rich(icons::VEHICLE)
                .size(28.0)
                .color(color::TEXT_SECONDARY),
        );
        ui.label(RichText::new(tr("vehicles-empty")).text_style(label_strong_style()));
        ui.add(
            egui::Label::new(
                RichText::new(tr("new-project-no-vehicles-hint")).color(color::TEXT_SECONDARY),
            )
            .wrap(),
        );
        ui.add_space(space::SM);
        wanted.sample |= ui
            .add(primary_button(&tr("vehicles-install-sample")))
            .clicked();
    });
}

/// The installed vehicles of the chosen game passing the search and kind
/// filter, one row each.
fn list(ui: &mut Ui, draft: &mut NewProjectDraft, library: &VehicleLibrary, height: f32) {
    let vehicles = filtered(library, &draft.filter);
    if vehicles.is_empty() {
        ui.add_space(space::LG);
        ui.vertical_centered(|ui| {
            let text = if draft.filter.query.trim().is_empty() && draft.filter.kind.is_none() {
                tr!(
                    "new-project-no-game-vehicles",
                    game = game_name(draft.game())
                )
            } else {
                tr("new-project-no-match")
            };
            ui.label(RichText::new(text).color(color::TEXT_SECONDARY));
        });
        return;
    }
    ScrollArea::vertical()
        .id_salt("new_project_vehicles")
        .max_height(height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for vehicle in vehicles {
                let m = &vehicle.newest().manifest;
                let selected = draft.vehicle.as_ref().is_some_and(|c| c.id == vehicle.id);
                if vehicle_row(ui, m, selected) && !selected {
                    draft.vehicle = Some(VehicleChoice::of(vehicle));
                }
            }
        });
}

/// One vehicle: a choice mark, its name and brand, its kind and its newest
/// version. Returns whether it was clicked.
fn vehicle_row(ui: &mut Ui, m: &Manifest, selected: bool) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), ROW_HEIGHT), Sense::click());
    response.widget_info(|| WidgetInfo::selected(WidgetType::RadioButton, true, selected, &m.name));
    let painter = ui.painter();
    if selected {
        painter.rect_filled(rect, CornerRadius::ZERO, color::SELECTED);
    } else if response.hovered() {
        painter.rect_filled(rect, CornerRadius::ZERO, color::SURFACE_2);
    }
    painter.hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, color::BORDER),
    );
    // The choice mark: a filled disc with a check when chosen, a ring
    // otherwise (the shape tells them apart, not only the color).
    let mark = egui::pos2(rect.left() + space::LG + 8.0, rect.center().y);
    if selected {
        painter.circle_filled(mark, 8.0, color::ACCENT_PRIMARY);
        painter.text(
            mark,
            Align2::CENTER_CENTER,
            icons::CHECK,
            icons::font(11.0),
            color::TEXT_ON_PRIMARY,
        );
    } else {
        painter.circle_stroke(mark, 7.5, Stroke::new(1.0, color::BORDER_STRONG));
    }
    let text_x = mark.x + 8.0 + space::MD;
    let right = rect.right() - space::LG;
    let version = painter.text(
        egui::pos2(right, rect.center().y),
        Align2::RIGHT_CENTER,
        m.version.to_string(),
        egui::FontId::monospace(typography::MONO),
        color::TEXT_SECONDARY,
    );
    let kind = painter.text(
        egui::pos2(version.left() - space::XL, rect.center().y),
        Align2::RIGHT_CENTER,
        kind_label(m.kind),
        egui::TextStyle::Body.resolve(ui.style()),
        color::TEXT_SECONDARY,
    );
    let clip = Rect::from_min_max(
        egui::pos2(text_x, rect.top()),
        egui::pos2(kind.left() - space::MD, rect.bottom()),
    );
    let painter = painter.with_clip_rect(clip.intersect(ui.clip_rect()));
    painter.text(
        egui::pos2(text_x, rect.top() + 14.0),
        Align2::LEFT_CENTER,
        &m.name,
        egui::TextStyle::Body.resolve(ui.style()),
        color::TEXT_PRIMARY,
    );
    painter.text(
        egui::pos2(text_x, rect.top() + 30.0),
        Align2::LEFT_CENTER,
        &m.brand,
        egui::TextStyle::Small.resolve(ui.style()),
        color::TEXT_SECONDARY,
    );
    paint_focus_ring(ui, rect.shrink(1.0), &response, 0);
    response.on_hover_text(summary(m)).clicked()
}

/// "Vehicle without a package": what Custom vehicle… does, and the button.
fn custom_entry(ui: &mut Ui, wanted: &mut Wanted) {
    let frame = Frame::new()
        .inner_margin(Margin::symmetric(space::LG as i8, space::MD as i8))
        .corner_radius(radius::LG)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let button = tr("custom-open");
                let font = egui::TextStyle::Button.resolve(ui.style());
                let button_width = ui
                    .painter()
                    .layout_no_wrap(button.clone(), font, color::TEXT_PRIMARY)
                    .size()
                    .x
                    + 2.0 * ui.spacing().button_padding.x;
                let text_width =
                    (ui.available_width() - button_width.max(88.0) - space::LG).max(120.0);
                ui.allocate_ui_with_layout(
                    Vec2::new(text_width, 0.0),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.set_width(text_width);
                        ui.spacing_mut().item_spacing.y = space::XXS;
                        ui.label(
                            RichText::new(tr("new-project-custom-title"))
                                .color(color::TEXT_PRIMARY),
                        );
                        ui.add(
                            egui::Label::new(
                                RichText::new(tr("new-project-custom-hint"))
                                    .small()
                                    .color(color::TEXT_SECONDARY),
                            )
                            .wrap(),
                        );
                    },
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    wanted.custom |= ui.add(secondary_button(&button)).clicked();
                });
            });
        });
    dashed_outline(ui, frame.response.rect);
}

/// A dashed hairline around `rect` (an entry that leads elsewhere).
fn dashed_outline(ui: &Ui, rect: Rect) {
    let rect = rect.shrink(0.5);
    let stroke = Stroke::new(1.0, color::BORDER_STRONG);
    let corners = [
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
        rect.left_top(),
    ];
    for pair in corners.windows(2) {
        ui.painter()
            .extend(egui::Shape::dashed_line(pair, stroke, 5.0, 4.0));
    }
}

/// "Your fleet": the chosen vehicle, its textures to tick, its main
/// texture mode, and the textures that will be created.
fn fleet(ui: &mut Ui, draft: &mut NewProjectDraft, library: &VehicleLibrary) {
    let rect = ui.max_rect();
    ui.painter().rect(
        rect,
        CornerRadius::same(radius::LG),
        color::SURFACE_1,
        Stroke::new(1.0, color::BORDER),
        StrokeKind::Inside,
    );
    Frame::new()
        .inner_margin(Margin::same(space::LG as i8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = space::SM;
            ui.label(
                RichText::new(tr("new-project-fleet"))
                    .text_style(egui::TextStyle::Heading)
                    .color(color::TEXT_PRIMARY),
            );
            let Some(choice) = draft.vehicle.as_mut() else {
                ui.add(
                    egui::Label::new(
                        RichText::new(tr("new-project-pick-vehicle")).color(color::TEXT_SECONDARY),
                    )
                    .wrap(),
                );
                return;
            };
            let Some(m) = choice.manifest(library) else {
                return;
            };
            // Room for "Textures created" under the vehicle.
            let row = ui.text_style_height(&egui::TextStyle::Body) + ui.spacing().item_spacing.y;
            let created_height = (choice.painted(m).len() + 2) as f32 * row + space::LG;
            ScrollArea::vertical()
                .id_salt("new_project_fleet")
                .max_height((ui.available_height() - created_height).max(80.0))
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.spacing_mut().item_spacing.y = space::SM;
                    Frame::new()
                        .fill(color::SURFACE_2)
                        .corner_radius(radius::LG)
                        .inner_margin(Margin::same(space::MD as i8 + 2))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.spacing_mut().item_spacing.y = space::SM;
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = space::XXS;
                                ui.label(
                                    RichText::new(&m.name)
                                        .text_style(label_strong_style())
                                        .color(color::TEXT_PRIMARY),
                                );
                                ui.label(
                                    RichText::new(summary(m))
                                        .small()
                                        .color(color::TEXT_SECONDARY),
                                );
                            });
                            let main_heading = match m.kind {
                                Kind::Truck => tr("new-project-cabins"),
                                Kind::Trailer => tr("vehicles-main-textures"),
                            };
                            textures_to_paint(
                                ui,
                                m,
                                &mut choice.textures,
                                &main_heading,
                                Some(&mode_text(m)),
                            );
                            let painted = choice.painted(m);
                            let main = painted
                                .iter()
                                .filter(|p| m.paint_job.main.iter().any(|q| q.id == p.id))
                                .count();
                            let counts = tr!(
                                "new-project-counts",
                                main = main,
                                accessories = painted.len() - main
                            );
                            let label = ui.label(
                                RichText::new(format!("→ {counts}"))
                                    .monospace()
                                    .color(color::TEXT_SECONDARY),
                            );
                            label.widget_info(|| {
                                WidgetInfo::labeled(WidgetType::Label, true, &counts)
                            });
                        });
                });
            created(ui, m, choice);
        });
}

/// The main texture mode, given by the package: "One per cabin layout" with
/// several main textures, "One for every cabin" for a truck with a single
/// one, "Single main texture" for a trailer.
fn mode_text(m: &Manifest) -> String {
    tr(match m.kind {
        Kind::Trailer => "project-mode-single",
        Kind::Truck if m.paint_job.main.len() > 1 => "project-mode-per-layout",
        Kind::Truck => "project-mode-every-cabin",
    })
}

/// "Textures created": each texture that will be created with its size,
/// then their total.
fn created(ui: &mut Ui, m: &Manifest, choice: &VehicleChoice) {
    ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
        ui.spacing_mut().item_spacing.y = space::XS;
        let painted = choice.painted(m);
        let total = tr!("new-project-total", count = painted.len());
        let label = ui.label(RichText::new(&total).text_style(label_strong_style()));
        label.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &total));
        // Bottom-up: the last texture first.
        for part in painted.iter().rev() {
            ui.horizontal(|ui| {
                let size = part.texture.size;
                let text = tr!(
                    "new-project-texture",
                    texture = part.name.as_str(),
                    size = size
                );
                let row = ui.label(RichText::new(&part.name).color(color::TEXT_SECONDARY));
                row.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{size}²"))
                            .monospace()
                            .color(color::TEXT_SECONDARY),
                    );
                });
            });
        }
        field_label(ui, &tr("new-project-created"));
        let rect = ui.available_rect_before_wrap();
        ui.painter().hline(
            rect.x_range(),
            rect.bottom() - space::XS,
            Stroke::new(1.0, color::BORDER),
        );
    });
}
