//! The New Project dialog, on one page: the project's name and game on
//! top; the official and installed vehicles of that game on the left, with
//! Custom vehicle… and Install…; "Your fleet" on the right, with the chosen
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
use tp_vehicles::catalog::Entry;
use tp_vehicles::{Game, Kind, Manifest};

use super::dialogs::{field_label, footer, labelled, messages, modal, title};
use super::vehicle_dialogs::{
    VehicleChoice, VehicleFilter, game_name, kind_label, summary, textures_to_paint, vehicle_rows,
};
use crate::official::Finished;
use crate::state::{AppState, Modal, NewProjectDraft};
use crate::vehicles::{SAMPLE_ID, VehicleLibrary};

/// Width of the "Your fleet" column.
const FLEET_WIDTH: f32 = 340.0;
/// Height of a row of the vehicle list, and of its column headers.
const ROW_HEIGHT: f32 = 32.0;
const HEADER_HEIGHT: f32 = 30.0;

/// The x positions of the vehicle list's columns in a row `rect`: the
/// choice mark's center, then the left edges of Vehicle, Type and Package.
struct Columns {
    mark: f32,
    name: f32,
    kind: f32,
    package: f32,
}

impl Columns {
    fn of(rect: Rect) -> Self {
        let pad = space::MD + 2.0;
        let gap = space::MD;
        let package = rect.right() - pad - 140.0;
        let kind = package - gap - 100.0;
        Self {
            mark: rect.left() + pad + 8.0,
            name: rect.left() + pad + 28.0 + gap,
            kind,
            package,
        }
    }
}

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
    /// Choose an official vehicle, installing it first when needed.
    PickOfficial(&'static Entry),
    /// Stop installing the official vehicle.
    CancelInstall,
}

/// Shows the New Project dialog held in `state.modal` and runs what it
/// asks for.
pub fn show_modal(ctx: &egui::Context, state: &mut AppState) {
    let Some(Modal::NewProject(draft)) = state.modal.as_mut() else {
        return;
    };
    // The official vehicle installed: chosen, or why it wasn't.
    if let Some(finished) = super::template_pack::poll(&mut draft.installing, &mut state.vehicles) {
        match finished {
            Finished::Installed(choice, message) => {
                if let Some(m) = choice.manifest(&state.vehicles) {
                    draft.filter.game = Some(m.game.id);
                }
                draft.messages = vec![Ok(message)];
                draft.vehicle = Some(choice);
            }
            Finished::Refused(message) => draft.messages = vec![Err(message)],
            Finished::Cancelled => {}
        }
    }
    match show(ctx, draft, &state.vehicles) {
        Outcome::Pending => {}
        Outcome::CancelInstall => super::template_pack::cancel(&mut draft.installing),
        Outcome::PickOfficial(entry) => {
            if let Some(Modal::NewProject(draft)) = state.modal.take() {
                super::template_pack::pick(
                    ctx,
                    state,
                    super::origin::Origin::NewProject(draft),
                    entry,
                );
            }
        }
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
    } else if wanted.cancel_install {
        Outcome::CancelInstall
    } else if let Some(entry) = wanted.official {
        Outcome::PickOfficial(entry)
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
    /// The official vehicle clicked.
    official: Option<&'static Entry>,
    cancel_install: bool,
}

impl Wanted {
    fn any(&self) -> bool {
        self.install || self.sample || self.custom || self.official.is_some() || self.cancel_install
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
    let installing = draft.installing.clone();
    let enabled = installing.is_none();
    ui.add_enabled_ui(enabled, |ui| {
        ui.horizontal(|ui| {
            let search = ui.add(
                TextEdit::singleline(&mut draft.filter.query)
                    .hint_text(format!("{} {}", icons::SEARCH, tr("vehicles-search")))
                    .desired_width(220.0)
                    .margin(Margin::symmetric(8, 5)),
            );
            search.widget_info(|| {
                WidgetInfo::labeled(WidgetType::TextEdit, true, tr("vehicles-search"))
            });
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
    });
    ui.add_space(space::SM);
    messages(ui, &draft.messages);
    if let Some(job) = &installing {
        wanted.cancel_install |= super::template_pack::installing_bar(ui, job);
    }

    // The game's official vehicles, whatever the search and filter.
    let game_filter = VehicleFilter {
        game: Some(draft.game()),
        ..Default::default()
    };
    let has_official = !vehicle_rows(library, &game_filter, &[]).official.is_empty();
    let none_installed = library.vehicles().is_empty();
    // The entry under the list keeps its place: the list takes the rest.
    let entry_height = 78.0;
    let sample_height = if none_installed && has_official {
        ui.spacing().interact_size.y.max(30.0) + space::MD + ui.spacing().item_spacing.y
    } else {
        0.0
    };
    let list_height =
        (height - (ui.cursor().top() - top) - entry_height - sample_height - space::MD).max(120.0);
    ui.add_enabled_ui(enabled, |ui| {
        Frame::new()
            .fill(color::SURFACE_1)
            .stroke(Stroke::new(1.0, color::BORDER))
            .corner_radius(radius::CARD)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(list_height);
                if none_installed && !has_official {
                    no_vehicle(ui, wanted);
                } else {
                    list(ui, draft, library, list_height, wanted);
                }
            });
        if sample_height > 0.0 {
            ui.add_space(space::MD);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(tr("new-project-sample-hint"))
                        .small()
                        .color(color::TEXT_SECONDARY),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    wanted.sample |= ui
                        .add(secondary_button(&tr("vehicles-install-sample")))
                        .clicked();
                });
            });
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

/// The vehicles of the chosen game passing the search and kind filter, one
/// row each: the official ones under Official, then the other installed
/// ones under Installed (the headers only when both have rows).
fn list(
    ui: &mut Ui,
    draft: &mut NewProjectDraft,
    library: &VehicleLibrary,
    height: f32,
    wanted: &mut Wanted,
) {
    let rows = vehicle_rows(library, &draft.filter, &[]);
    if rows.is_empty() {
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
    column_headers(ui);
    let chosen_id = draft.vehicle.as_ref().map(|c| c.id.clone());
    ScrollArea::vertical()
        .id_salt("new_project_vehicles")
        .max_height(height - HEADER_HEIGHT)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            if rows.sections() {
                section_header(ui, &tr("vehicles-section-official"));
            }
            for row in &rows.official {
                let e = row.entry;
                let selected = chosen_id.as_deref() == Some(e.package_id().as_str());
                let cells = Row {
                    name: &e.name,
                    kind: e.kind,
                    package: format!("{} · {}", e.brand, row.package()),
                    tip: row.summary(),
                };
                if vehicle_row(ui, &cells, selected) && !selected {
                    wanted.official = Some(e);
                }
            }
            if rows.sections() {
                section_header(ui, &tr("vehicles-section-installed"));
            }
            for vehicle in rows.installed {
                let m = &vehicle.newest().manifest;
                let selected = chosen_id.as_deref() == Some(vehicle.id.as_str());
                let cells = Row {
                    name: &m.name,
                    kind: m.kind,
                    package: format!("{} {}", m.brand, m.version),
                    tip: summary(m),
                };
                if vehicle_row(ui, &cells, selected) && !selected {
                    draft.vehicle = Some(VehicleChoice::of(vehicle));
                }
            }
        });
}

/// A section's header in the list ("Official", "Installed"): small muted
/// capitals.
fn section_header(ui: &mut Ui, text: &str) {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), HEADER_HEIGHT - 4.0),
        Sense::hover(),
    );
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
    let columns = Columns::of(rect);
    let font = egui::FontId::new(typography::CAPTION, tp_ui::fonts::semibold_family());
    ui.painter().text(
        egui::pos2(columns.mark - 8.0, rect.center().y + 2.0),
        Align2::LEFT_CENTER,
        text.to_uppercase(),
        font,
        color::TEXT_SECONDARY,
    );
}

/// The headers of the list's columns: Vehicle, Type and Package, in small
/// muted capitals over a hairline.
fn column_headers(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), HEADER_HEIGHT),
        Sense::hover(),
    );
    let columns = Columns::of(rect);
    let painter = ui.painter();
    let font = egui::FontId::new(typography::CAPTION, tp_ui::fonts::semibold_family());
    for (x, key) in [
        (columns.name, "new-project-column-vehicle"),
        (columns.kind, "new-project-column-type"),
        (columns.package, "new-project-column-package"),
    ] {
        painter.text(
            egui::pos2(x, rect.center().y),
            Align2::LEFT_CENTER,
            tr(key).to_uppercase(),
            font.clone(),
            color::TEXT_MUTED,
        );
    }
    painter.hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, color::BORDER),
    );
}

/// The cells of a row of the vehicle list.
struct Row<'a> {
    name: &'a str,
    kind: Kind,
    /// The Package column: the brand with the newest version, or
    /// "Official".
    package: String,
    /// The row's tooltip.
    tip: String,
}

/// One vehicle: a choice mark, its name, its kind, and its package.
/// Returns whether it was clicked.
fn vehicle_row(ui: &mut Ui, m: &Row<'_>, selected: bool) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), ROW_HEIGHT), Sense::click());
    response.widget_info(|| WidgetInfo::selected(WidgetType::RadioButton, true, selected, m.name));
    let painter = ui.painter();
    if selected {
        painter.rect_filled(rect, CornerRadius::ZERO, color::CONTROL);
    } else if response.hovered() {
        painter.rect_filled(rect, CornerRadius::ZERO, color::SURFACE_2);
    }
    painter.hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, color::BORDER),
    );
    let columns = Columns::of(rect);
    // The choice mark: a filled disc with a check when chosen, a ring
    // otherwise (the shape tells them apart, not only the color).
    let mark = egui::pos2(columns.mark, rect.center().y);
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
    let y = rect.center().y;
    let cell = |from: f32, to: f32| {
        painter.with_clip_rect(
            Rect::from_x_y_ranges(from..=to - space::SM, rect.y_range()).intersect(ui.clip_rect()),
        )
    };
    cell(columns.name, columns.kind).text(
        egui::pos2(columns.name, y),
        Align2::LEFT_CENTER,
        m.name,
        egui::TextStyle::Body.resolve(ui.style()),
        if selected {
            color::TEXT_PRIMARY
        } else {
            color::TEXT_SECONDARY
        },
    );
    cell(columns.kind, columns.package).text(
        egui::pos2(columns.kind, y),
        Align2::LEFT_CENTER,
        kind_label(m.kind),
        egui::TextStyle::Body.resolve(ui.style()),
        color::TEXT_SECONDARY,
    );
    cell(columns.package, rect.right()).text(
        egui::pos2(columns.package, y),
        Align2::LEFT_CENTER,
        &m.package,
        egui::FontId::monospace(typography::CAPTION),
        color::TEXT_MUTED,
    );
    paint_focus_ring(ui, rect.shrink(1.0), &response, 0);
    response.on_hover_text(&m.tip).clicked()
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
                        .fill(color::CONTROL)
                        .stroke(Stroke::new(1.0, color::BORDER))
                        .corner_radius(radius::CARD)
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
                let size = part.texture.size.label();
                let text = tr!(
                    "new-project-texture",
                    texture = part.name.as_str(),
                    size = size.as_str()
                );
                let row = ui.label(RichText::new(&part.name).color(color::TEXT_SECONDARY));
                row.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(size).monospace().color(color::TEXT_SECONDARY));
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
