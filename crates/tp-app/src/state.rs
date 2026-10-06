//! Application state machine and per-frame orchestration.

use egui::{Key, Ui, ViewportCommand};
use tp_core::document::Object;
use tp_core::{Project, TextureResolution};
use tp_ui::ThemeSettings;

use crate::commands::{self, Availability, CommandId, EditContext};
use crate::layout::ViewMode;
use crate::paths::APP_NAME;
use crate::prefs::{Prefs, PrefsStore, recent_exists};
use crate::tool::Tool;
use crate::ui;
use crate::viewport::Viewport;
pub use crate::workspace::{COPY_OFFSET, SaveState, Workspace};

/// Delay after the last preference change before it is written to disk.
const PREFS_SAVE_DELAY: f64 = 1.0;

/// The top-level screen.
pub enum Screen {
    Home,
    Workspace(Box<Workspace>),
}

/// Draft of the New Project wizard.
#[derive(Clone, Debug, Default)]
pub struct NewProjectDraft {
    pub step: usize,
    pub name: String,
    pub resolution: TextureResolution,
    pub focus_requested: bool,
}

pub enum Modal {
    NewProject(NewProjectDraft),
    Preferences,
    KeyboardShortcuts,
    About,
}

pub struct AppState {
    pub prefs: Prefs,
    store: Option<PrefsStore>,
    /// Last version written to (or read from) disk.
    saved_prefs: Prefs,
    prefs_changed_at: Option<f64>,
    applied_theme: Option<ThemeSettings>,
    fonts_installed: bool,
    pub screen: Screen,
    pub modal: Option<Modal>,
    pub show_gallery: bool,
    /// Commands triggered this frame, executed at the end of the frame.
    pub queue: Vec<CommandId>,
    /// Whether each recent project's file exists (aligned with `prefs.recent`).
    pub recent_available: Vec<bool>,
    /// Objects copied or cut, shared by every project in this session.
    pub clipboard: Vec<Object>,
    /// Pastes since the last copy, for the repeated-paste offset.
    pub paste_count: u32,
    title: String,
}

impl AppState {
    /// Creates the state, loading preferences from `store` when given.
    pub fn new(store: Option<PrefsStore>) -> Self {
        let prefs = match &store {
            Some(store) => {
                let loaded = store.load();
                if let Some(issue) = &loaded.issue {
                    tracing::warn!(?issue, backup = ?loaded.backup, "preferences reset to defaults");
                }
                loaded.prefs
            }
            None => Prefs::default(),
        };
        Self::with_prefs(prefs, store)
    }

    /// Creates the state from in-memory preferences (tests).
    pub fn with_prefs(prefs: Prefs, store: Option<PrefsStore>) -> Self {
        let mut state = Self {
            saved_prefs: prefs.clone(),
            prefs,
            store,
            prefs_changed_at: None,
            applied_theme: None,
            fonts_installed: false,
            screen: Screen::Home,
            modal: None,
            show_gallery: false,
            queue: Vec::new(),
            recent_available: Vec::new(),
            clipboard: Vec::new(),
            paste_count: 0,
            title: String::new(),
        };
        state.refresh_recent_availability();
        state
    }

    pub fn has_project(&self) -> bool {
        matches!(self.screen, Screen::Workspace(_))
    }

    pub fn workspace(&self) -> Option<&Workspace> {
        match &self.screen {
            Screen::Workspace(ws) => Some(ws),
            Screen::Home => None,
        }
    }

    pub fn workspace_mut(&mut self) -> Option<&mut Workspace> {
        match &mut self.screen {
            Screen::Workspace(ws) => Some(ws),
            Screen::Home => None,
        }
    }

    pub fn refresh_recent_availability(&mut self) {
        self.recent_available = self
            .prefs
            .recent
            .iter()
            .map(|r| recent_exists(&r.path))
            .collect();
    }

    /// Opens the editor on a new project.
    pub fn open_project(&mut self, project: Project) {
        tracing::info!(name = %project.name, side = project.resolution.side(), "project created");
        self.screen = Screen::Workspace(Box::new(Workspace::new(project)));
    }

    pub fn close_project(&mut self) {
        self.screen = Screen::Home;
        self.refresh_recent_availability();
    }

    /// Runs one frame of the whole application.
    pub fn show(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        if !self.fonts_installed {
            // Fonts registered now are only usable from the next pass.
            self.install_theme(&ctx);
            ctx.request_repaint();
            return;
        }
        if self.sync_theme(&ctx) {
            // Context style changes apply from the next frame; style this
            // frame's root `Ui` directly so the first frame is themed too.
            tp_ui::theme::configure_style(ui.style_mut(), self.prefs.theme().text_scale);
        }
        self.handle_keyboard(&ctx);

        match self.screen {
            Screen::Home => ui::home::show(ui, self),
            Screen::Workspace(_) => ui::workspace::show(ui, self),
        }
        ui::dialogs::show_modal(&ctx, self);
        if self.show_gallery {
            ui::gallery::show(&ctx, &mut self.show_gallery);
        }

        for id in std::mem::take(&mut self.queue) {
            self.dispatch(&ctx, id);
        }
        self.sync_title(&ctx);
        let now = ctx.input(|i| i.time);
        self.persist_prefs(now, false);
        if self.prefs_changed_at.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(PREFS_SAVE_DELAY + 0.1));
        }
    }

    /// Installs fonts and theme. Call before the first frame when possible
    /// (fonts become usable from the next pass).
    pub fn install_theme(&mut self, ctx: &egui::Context) {
        let theme = self.prefs.theme();
        tp_ui::theme::install(ctx, theme);
        self.fonts_installed = true;
        self.applied_theme = Some(theme);
        // Cmd/Ctrl +/-/0 zoom the canvas, not the whole interface.
        ctx.options_mut(|o| o.zoom_with_keyboard = false);
    }

    /// What the editing commands can do right now.
    pub fn edit_context(&self) -> EditContext {
        match self.workspace() {
            Some(ws) => EditContext {
                has_project: true,
                has_selection: !ws.selection.is_empty(),
                can_undo: ws.history.can_undo(),
                can_redo: ws.history.can_redo(),
                has_clipboard: !self.clipboard.is_empty(),
                gesture_active: ws.gesture.is_active(),
                undo_label: ws.history.undo_label(),
                redo_label: ws.history.redo_label(),
            },
            None => EditContext::default(),
        }
    }

    /// Re-applies the theme when scaling preferences changed; returns true
    /// when it did.
    fn sync_theme(&mut self, ctx: &egui::Context) -> bool {
        let theme = self.prefs.theme();
        if self.applied_theme == Some(theme) {
            return false;
        }
        tp_ui::theme::apply(ctx, theme);
        self.applied_theme = Some(theme);
        true
    }

    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        let typing = ctx.text_edit_focused() || self.modal.is_some();
        let has_project = self.has_project();
        let edit = self.edit_context();
        let triggered = ctx.input_mut(|i| {
            commands::take_triggered(i, typing, has_project, |id| is_enabled(id, &edit))
        });
        self.queue.extend(triggered);

        // Holding Space temporarily activates the Hand tool.
        let space_down = !typing && ctx.input(|i| i.key_down(Key::Space));
        if let Some(ws) = self.workspace_mut() {
            match (space_down, ws.tool_before_space) {
                (true, None) => {
                    ws.tool_before_space = Some(ws.tool);
                    ws.tool = Tool::Hand;
                }
                (false, Some(previous)) => {
                    ws.tool = previous;
                    ws.tool_before_space = None;
                }
                _ => {}
            }
        }
    }

    /// Executes a command. Disabled commands are ignored.
    pub fn dispatch(&mut self, ctx: &egui::Context, id: CommandId) {
        if !is_enabled(id, &self.edit_context()) {
            return;
        }
        tracing::debug!(?id, "command");
        let now = ctx.input(|i| i.time);
        let ppp = ctx.pixels_per_point();
        match id {
            CommandId::NewProject => {
                self.modal = Some(Modal::NewProject(NewProjectDraft::default()));
            }
            CommandId::CloseProject => self.close_project(),
            CommandId::Preferences => self.modal = Some(Modal::Preferences),
            CommandId::Quit => ctx.send_viewport_cmd(ViewportCommand::Close),
            CommandId::SetViewMode(mode) => self.prefs.layout.view_mode = mode,
            CommandId::TogglePreview => {
                let layout = &mut self.prefs.layout;
                layout.view_mode = if layout.view_mode.shows_preview() {
                    ViewMode::TwoD
                } else {
                    ViewMode::Split
                };
            }
            CommandId::TogglePanel(kind) => self.prefs.layout.toggle_open(kind),
            CommandId::ResetWorkspace => self.prefs.layout.reset(),
            CommandId::DesignGallery => self.show_gallery = !self.show_gallery,
            CommandId::KeyboardShortcuts => self.modal = Some(Modal::KeyboardShortcuts),
            CommandId::About => self.modal = Some(Modal::About),
            CommandId::SelectTool(tool) => {
                if let Some(ws) = self.workspace_mut() {
                    ws.tool = tool;
                    ws.tool_before_space = None;
                }
            }
            CommandId::Undo => self.with_workspace(Workspace::undo),
            CommandId::Redo => self.with_workspace(Workspace::redo),
            CommandId::SelectAll => self.with_workspace(Workspace::select_all),
            CommandId::Deselect => self.with_workspace(Workspace::deselect),
            CommandId::Delete => self.with_workspace(|ws| ws.delete_selection(now)),
            CommandId::Duplicate => self.with_workspace(|ws| ws.duplicate_selection(now)),
            CommandId::BringForward => self.with_workspace(|ws| ws.bring_forward(now)),
            CommandId::SendBackward => self.with_workspace(|ws| ws.send_backward(now)),
            CommandId::Copy | CommandId::Cut => {
                if let Some(ws) = self.workspace() {
                    self.clipboard = ws.selected_objects();
                    self.paste_count = 0;
                }
                if id == CommandId::Cut
                    && let Some(ws) = self.workspace_mut()
                {
                    ws.edit("Cut", now, false, |project, selection| {
                        project.surface_mut().remove(selection);
                        selection.clear();
                    });
                }
            }
            CommandId::Paste => {
                let objects = self.clipboard.clone();
                let offset = COPY_OFFSET * f64::from(self.paste_count);
                self.paste_count += 1;
                self.with_workspace(|ws| ws.paste(&objects, offset, now));
            }
            CommandId::Nudge(direction, big) => {
                let (dx, dy) = direction.delta();
                let step = if big { 10.0 } else { 1.0 };
                self.with_workspace(|ws| ws.nudge(dx * step, dy * step, now));
            }
            CommandId::ZoomIn
            | CommandId::ZoomOut
            | CommandId::FitToScreen
            | CommandId::ActualSize => {
                self.with_workspace(|ws| {
                    let (Some(canvas), Some(view)) = (ws.canvas_rect, ws.viewport.as_mut()) else {
                        return;
                    };
                    match id {
                        CommandId::ZoomIn => {
                            let zoom = Viewport::step_zoom(view.zoom, 1);
                            view.zoom_at(canvas, ppp, canvas.center(), zoom);
                        }
                        CommandId::ZoomOut => {
                            let zoom = Viewport::step_zoom(view.zoom, -1);
                            view.zoom_at(canvas, ppp, canvas.center(), zoom);
                        }
                        CommandId::ActualSize => view.set_zoom_centered(1.0),
                        _ => *view = Viewport::fit(ws.project.surface().size, canvas, ppp),
                    }
                });
            }
            // Not-yet-available commands never reach here (disabled).
            _ => {}
        }
    }

    fn with_workspace(&mut self, f: impl FnOnce(&mut Workspace)) {
        if let Some(ws) = self.workspace_mut() {
            f(ws);
        }
    }

    fn sync_title(&mut self, ctx: &egui::Context) {
        let title = match self.workspace() {
            Some(ws) => format!("{} — {APP_NAME}", ws.project.name),
            None => APP_NAME.to_owned(),
        };
        if title != self.title {
            ctx.send_viewport_cmd(ViewportCommand::Title(title.clone()));
            self.title = title;
        }
    }

    /// Current window title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Debounced preference saving; `force` writes immediately (on exit).
    pub fn persist_prefs(&mut self, now: f64, force: bool) {
        if self.prefs == self.saved_prefs {
            self.prefs_changed_at = None;
            return;
        }
        let changed_at = *self.prefs_changed_at.get_or_insert(now);
        if !force && now - changed_at < PREFS_SAVE_DELAY {
            return;
        }
        if let Some(store) = &self.store {
            match store.save(&self.prefs) {
                Ok(()) => tracing::debug!(path = %store.path().display(), "preferences saved"),
                Err(err) => tracing::error!(%err, "failed to save preferences"),
            }
        }
        self.saved_prefs = self.prefs.clone();
        self.prefs_changed_at = None;
    }
}

/// Whether a command can run in the current state.
pub fn is_enabled(id: CommandId, edit: &EditContext) -> bool {
    match id.meta().availability {
        Availability::Always => true,
        Availability::NeedsProject => edit.has_project,
        Availability::When(check, _) => check(edit),
        Availability::NotYet(_) => false,
    }
}

/// Tooltip explaining why a command is disabled.
pub fn disabled_reason(id: CommandId) -> Option<&'static str> {
    match id.meta().availability {
        Availability::NotYet(reason) => Some(reason),
        Availability::When(_, reason) if !reason.is_empty() => Some(reason),
        Availability::NeedsProject => Some("Open or create a project first."),
        Availability::Always | Availability::When(..) => None,
    }
}
