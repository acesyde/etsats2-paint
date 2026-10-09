//! Panel bodies: the functions drawing each part of the workspace (layers,
//! colors, stroke, transform, styles, symbols, assets, fleet, properties),
//! placed in the left panel's tabs, the inspector and the spaces.

pub mod assets;
pub mod character;
pub mod colors;
pub mod layers;
pub mod line_style;
pub mod properties;
pub mod stroke;
pub mod styles;
pub mod symbols;
pub mod transform;
pub mod vehicle;

use egui::Ui;
use tp_i18n::tr;
use tp_ui::widgets::FieldEvent;

use crate::commands::CommandId;
use crate::ui::CommandUi;
use crate::workspace::Workspace;

/// What panel bodies can read and edit.
pub struct PanelEnv<'a> {
    pub ws: &'a mut Workspace,
    /// Preferences' recent colors (RGBA), most recent first.
    pub recent_colors: &'a [[u8; 4]],
    /// Installed vehicle packages.
    pub vehicles: &'a crate::vehicles::VehicleLibrary,
    /// An action on one of the project's vehicles, run after the frame.
    pub vehicle_request: &'a mut Option<crate::state::VehicleRequest>,
    /// The personal library (Add to Library).
    pub library: &'a mut crate::library::LibraryStore,
    pub now: f64,
}

/// Routes a field event to a live edit: `Live` applies, `Commit` applies and
/// records one undo step, `Revert` restores the document.
pub fn apply_field(
    env: &mut PanelEnv<'_>,
    event: FieldEvent,
    apply: impl FnOnce(&mut Workspace, f64),
) {
    match event {
        FieldEvent::Live(v) => apply(env.ws, v),
        FieldEvent::Commit(v) => {
            apply(env.ws, v);
            env.ws.commit_pending(env.now);
        }
        FieldEvent::Revert => env.ws.cancel_pending(),
        FieldEvent::None => {}
    }
}

/// Add to Library or Update in Library, in an element's context menu.
pub fn library_item(ui: &mut Ui, env: &mut PanelEnv<'_>, element: crate::library::Element) {
    let label = env.library.menu_label(&env.ws.project, element);
    if ui.add(tp_ui::widgets::MenuRow::new(&label)).clicked() {
        env.ws.add_to_library(env.library, element, env.now);
        ui.close();
    }
}

/// "Import from Library…" (the Resources tab, the Brand space), disabled
/// with a tooltip saying why while a symbol is edited.
pub fn import_from_library_button(ui: &mut Ui, cmds: &mut CommandUi<'_>) {
    let id = CommandId::ImportFromLibrary;
    let enabled = cmds.enabled(id);
    let mut button = ui.add_enabled(
        enabled,
        tp_ui::widgets::secondary_button(&tr("cmd-import-from-library")),
    );
    if !enabled && let Some(reason) = crate::state::disabled_reason_for(id, &cmds.edit) {
        button = button.on_disabled_hover_text(tr(reason));
    }
    if button.clicked() {
        cmds.push(id);
    }
}

/// The heading of a list (the Textures and Layers tabs): a short title in
/// the muted color, with `actions` at its right end.
pub fn list_heading(ui: &mut Ui, title: &str, actions: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.set_min_height(tp_ui::tokens::size::HIT_MIN);
        ui.add(
            egui::Label::new(
                egui::RichText::new(title)
                    .size(tp_ui::tokens::typography::CONTROL)
                    .color(tp_ui::tokens::color::TEXT_MUTED),
            )
            .truncate(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = tp_ui::tokens::space::XXS;
            actions(ui);
        });
    });
}

/// A section heading (the inspector's sections, the Resources tab's, the
/// Project space's columns): small semibold capitals in the muted color,
/// read as written by assistive technologies.
pub fn section_heading(ui: &mut Ui, text: &str) -> egui::Response {
    let label = ui.label(
        egui::RichText::new(text.to_uppercase())
            .size(tp_ui::tokens::typography::CAPTION)
            .family(tp_ui::fonts::semibold_family())
            .extra_letter_spacing(0.6)
            .color(tp_ui::tokens::color::TEXT_MUTED),
    );
    label.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, text));
    label
}
