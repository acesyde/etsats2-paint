//! Renders commands as menu items, buttons and tool buttons, all feeding the
//! same command queue.

use egui::{Response, Ui};
use tp_ui::widgets::{IconButton, MenuRow, ToolButton};

use crate::commands::{CommandId, EditContext, ShortcutFormatter};
use crate::state::{disabled_reason, is_enabled};

/// Per-frame helper bound to the command queue.
pub struct CommandUi<'q> {
    pub queue: &'q mut Vec<CommandId>,
    pub edit: EditContext,
    pub shortcuts: ShortcutFormatter,
}

impl<'q> CommandUi<'q> {
    pub fn new(ctx: &egui::Context, queue: &'q mut Vec<CommandId>, edit: EditContext) -> Self {
        Self {
            queue,
            edit,
            shortcuts: ShortcutFormatter::new(ctx),
        }
    }

    pub fn enabled(&self, id: CommandId) -> bool {
        is_enabled(id, &self.edit)
    }

    /// Menu label, naming the operation for Undo/Redo ("Undo Move").
    pub fn label(&self, id: CommandId) -> String {
        let base = id.meta().label;
        let detail = match id {
            CommandId::Undo => self.edit.undo_label,
            CommandId::Redo => self.edit.redo_label,
            _ => None,
        };
        match detail {
            Some(detail) => format!("{base} {detail}"),
            None => base.to_owned(),
        }
    }

    pub fn push(&mut self, id: CommandId) {
        self.queue.push(id);
    }

    /// A menu row for `id`; closes the menu when clicked.
    pub fn menu_item(&mut self, ui: &mut Ui, id: CommandId) -> Response {
        self.menu_row(ui, id, None)
    }

    /// A menu row with a check mark reflecting `checked`.
    pub fn menu_toggle(&mut self, ui: &mut Ui, id: CommandId, checked: bool) -> Response {
        self.menu_row(ui, id, Some(checked))
    }

    fn menu_row(&mut self, ui: &mut Ui, id: CommandId, checked: Option<bool>) -> Response {
        let meta = id.meta();
        let label = self.label(id);
        let shortcut = self.shortcuts.command(id);
        let enabled = self.enabled(id);
        let mut response = ui.add_enabled(
            enabled,
            MenuRow::new(&label)
                .icon(meta.icon)
                .shortcut(shortcut.as_deref())
                .checked(checked),
        );
        if !enabled && let Some(reason) = disabled_reason(id) {
            response = response.on_disabled_hover_text(reason);
        }
        if response.clicked() {
            self.push(id);
            ui.close();
        }
        response
    }

    /// Icon-only button for `id`.
    pub fn icon_button(&mut self, ui: &mut Ui, id: CommandId, selected: bool) -> Response {
        let meta = id.meta();
        let shortcut = self.shortcuts.command(id);
        let enabled = self.enabled(id);
        let mut button = IconButton::new(meta.icon.unwrap_or("?"), meta.label)
            .shortcut(shortcut.as_deref())
            .selected(selected);
        if let Some(reason) = disabled_reason(id) {
            button = button.disabled_reason(reason);
        }
        let response = ui.add_enabled(enabled, button);
        if response.clicked() {
            self.push(id);
        }
        response
    }

    /// Tool bar button for a tool command.
    pub fn tool_button(&mut self, ui: &mut Ui, id: CommandId, active: bool) -> Response {
        let meta = id.meta();
        let shortcut = self.shortcuts.command(id);
        let response = ui.add_enabled(
            self.enabled(id),
            ToolButton::new(meta.icon.unwrap_or("?"), meta.label)
                .shortcut(shortcut.as_deref())
                .active(active),
        );
        if response.clicked() {
            self.push(id);
        }
        response
    }
}
