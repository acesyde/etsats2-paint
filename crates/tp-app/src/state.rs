//! Application state machine and per-frame orchestration.

use egui::{Key, Ui, ViewportCommand};
use tp_core::document::Object;
use tp_core::{Project, TextureResolution};
use tp_ui::ThemeSettings;

use crate::commands::{self, Availability, CommandId, EditContext};
use crate::file_dialogs::{FileDialogs, NativeDialogs};
use crate::layout::ViewMode;
use crate::paths::APP_NAME;
use crate::prefs::{Prefs, PrefsStore, recent_exists};
pub use crate::project_io::PendingAction;
use crate::recovery::{Recovered, RecoveryStore};
use crate::saver::Saver;
use crate::text_engine::TextEngine;
use crate::tool::Tool;
use crate::ui;
use crate::viewport::Viewport;
pub use crate::workspace::{COPY_OFFSET, ColorTarget, SaveState, Workspace};

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
    /// "Save changes before closing?", then the action.
    UnsavedChanges(PendingAction),
    /// An error or information message.
    Message {
        title: String,
        text: String,
    },
    /// Export Texture dialog.
    Export(Box<crate::ui::export_dialog::ExportDialog>),
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
    /// A text field had focus at the end of the previous frame. egui drops
    /// focus at the start of the frame where Escape is pressed, so this keeps
    /// that Escape for the field instead of canvas shortcuts.
    text_focus_last_frame: bool,
    title: String,
    /// Fonts kept between projects (system fonts load once).
    fonts: Option<tp_text::FontLibrary>,
    /// Load the fonts installed on the computer (off in tests, so results
    /// do not depend on the machine).
    pub system_fonts: bool,
    /// Open / Save / Place dialogs (scripted in tests).
    pub dialogs: Box<dyn FileDialogs>,
    /// Background writer of project files and recovery copies.
    pub(crate) saver: Option<Saver>,
    /// This session's crash-recovery files.
    pub(crate) recovery: Option<RecoveryStore>,
    /// Copies left by earlier sessions, offered on the home screen.
    pub recovered: Vec<Recovered>,
    /// Action to run once the save started from the prompt succeeds.
    pub(crate) after_save: Option<PendingAction>,
    /// The window may close (the user already answered the prompt).
    pub(crate) allow_close: bool,
    /// Last export choices, for the session.
    pub export_settings: crate::export::ExportSettings,
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
        let mut state = Self::with_prefs(prefs, store);
        state.system_fonts = true;
        state.dialogs = Box::new(NativeDialogs);
        state
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
            text_focus_last_frame: false,
            title: String::new(),
            fonts: None,
            system_fonts: false,
            // Tests never open real dialogs; `new` installs the native ones.
            dialogs: Box::new(crate::file_dialogs::ScriptedDialogs::default()),
            saver: None,
            recovery: None,
            recovered: Vec::new(),
            after_save: None,
            allow_close: false,
            export_settings: crate::export::ExportSettings::default(),
        };
        state.refresh_recent_availability();
        state
    }

    /// File names proposed by scripted dialogs (tests).
    pub fn dialogs_suggested(&self) -> Vec<String> {
        self.dialogs.suggested()
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
        let fonts = self
            .fonts
            .take()
            .unwrap_or_else(tp_text::FontLibrary::bundled);
        let ws = Workspace::with_text_engine(project, TextEngine::new(fonts));
        self.screen = Screen::Workspace(Box::new(ws));
    }

    pub fn close_project(&mut self) {
        self.clear_recovery();
        if let Screen::Workspace(ws) = std::mem::replace(&mut self.screen, Screen::Home) {
            self.fonts = Some(ws.text.fonts);
        }
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
        self.handle_close_request(&ctx);
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
        self.after_frame(&ctx);
        self.flush_recent_color(&ctx);
        self.text_focus_last_frame = ctx.text_edit_focused();
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
                selection_has_group: ws
                    .selection
                    .iter()
                    .any(|id| ws.project.surface().get(*id).is_some_and(|o| o.is_group())),
                single_text: matches!(ws.selection.as_slice(), [id] if ws.can_edit_text(*id)),
                editing_text: ws.is_editing_text(),
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
        let field_focused =
            ctx.text_edit_focused() || self.text_focus_last_frame || self.modal.is_some();
        let now = ctx.input(|i| i.time);
        let mut editing_text = false;
        if let Some(ws) = self.workspace_mut()
            && ws.is_editing_text()
        {
            editing_text = true;
            if !field_focused {
                crate::text_input::handle(ctx, ws, now);
            }
        }
        let typing = field_focused || editing_text;
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
        // A panel interaction in progress becomes one step before any command.
        if let Some(ws) = self.workspace_mut() {
            ws.commit_pending(now);
            if ws.is_editing_text() {
                match id {
                    CommandId::Undo => {
                        ws.text_undo(now);
                        return;
                    }
                    CommandId::Redo => {
                        ws.text_redo(now);
                        return;
                    }
                    _ if !keeps_text_session(id) => ws.end_text_session(now),
                    _ => {}
                }
            }
        }
        match id {
            CommandId::Place => self.with_workspace(|ws| ws.place_request = Some(None)),
            CommandId::EditText => self.with_workspace(|ws| {
                if let [id] = ws.selection[..] {
                    ws.start_editing(id, None, now);
                }
            }),
            CommandId::NewProject => self.guard(ctx, PendingAction::NewProject),
            CommandId::OpenProject => self.guard(ctx, PendingAction::OpenDialog),
            CommandId::ExportTexture => {
                self.modal = Some(Modal::Export(Box::new(
                    crate::ui::export_dialog::ExportDialog::new(self.export_settings),
                )));
            }
            CommandId::Save => {
                self.save(ctx, false);
            }
            CommandId::SaveAs => {
                self.save(ctx, true);
            }
            CommandId::CloseProject => self.guard(ctx, PendingAction::CloseProject),
            CommandId::Preferences => self.modal = Some(Modal::Preferences),
            CommandId::Quit => self.guard(ctx, PendingAction::Quit),
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
            CommandId::Delete | CommandId::DeleteLayer => {
                self.with_workspace(|ws| ws.delete_selection(now));
            }
            CommandId::Duplicate | CommandId::DuplicateLayer => {
                self.with_workspace(|ws| ws.duplicate_selection(now));
            }
            CommandId::Group => self.with_workspace(|ws| ws.group_selection(now)),
            CommandId::Ungroup => self.with_workspace(|ws| ws.ungroup_selection(now)),
            CommandId::NewLayer => self.with_workspace(|ws| ws.new_layer(now)),
            CommandId::SwapColorTarget => self.with_workspace(|ws| {
                ws.panels.color_target = match ws.panels.color_target {
                    ColorTarget::Fill => ColorTarget::Stroke,
                    ColorTarget::Stroke => ColorTarget::Fill,
                };
            }),
            CommandId::SwapFillStroke => self.with_workspace(|ws| ws.swap_fill_stroke(now)),
            CommandId::DefaultColors => self.with_workspace(|ws| ws.default_colors(now)),
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

    /// Adds the last applied color to recent colors once the user is no
    /// longer dragging or typing (so a picker drag adds one color).
    fn flush_recent_color(&mut self, ctx: &egui::Context) {
        let busy = ctx.input(|i| i.pointer.any_down()) || ctx.text_edit_focused();
        if busy {
            return;
        }
        let Some(ws) = self.workspace_mut() else {
            return;
        };
        if let Some(c) = ws.recent_candidate.take() {
            self.prefs.push_recent_color([c.r, c.g, c.b, c.a]);
        }
    }

    /// Per-frame workspace upkeep after the UI: ends a text session whose
    /// text lost the selection, picks up fonts, places requested or dropped
    /// files.
    fn after_frame(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        let hover = ctx.input(|i| i.pointer.latest_pos());
        let ppp = ctx.pixels_per_point();
        self.poll_saves(ctx);
        self.write_recovery(ctx, now);
        let Screen::Workspace(ws) = &mut self.screen else {
            return;
        };
        ws.check_text_session(now);
        if self.system_fonts && !ws.text.fonts.system_requested() {
            let ctx = ctx.clone();
            ws.text
                .fonts
                .load_system_fonts(move || ctx.request_repaint());
        }
        if ws.text.poll_fonts() {
            ws.relayout_all_texts();
        }
        if let Some(at) = ws.place_request.take() {
            let paths = self.dialogs.pick_images();
            if !paths.is_empty() {
                let files = crate::import::read_files(&paths);
                ws.place_files(files, at, now);
            }
        }
        if !dropped.is_empty() {
            let at = match (hover, ws.canvas_rect, ws.screen_map(ppp)) {
                (Some(p), Some(rect), Some(map)) if rect.contains(p) => Some(map.to_doc(p)),
                _ => None,
            };
            let files = dropped
                .iter()
                .map(|file| {
                    let path = file.path();
                    let name = path.file_name().map_or_else(
                        || path.display().to_string(),
                        |n| n.to_string_lossy().into_owned(),
                    );
                    match file.bytes() {
                        Ok(bytes) => crate::import::read_bytes(&name, bytes),
                        Err(reason) => Err(crate::import::ImportError {
                            file: name,
                            reason: format!("cannot be read ({reason})"),
                        }),
                    }
                })
                .collect();
            ws.place_files(files, at, now);
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

/// Commands that leave a text editing session running (view changes).
fn keeps_text_session(id: CommandId) -> bool {
    matches!(
        id,
        CommandId::ZoomIn
            | CommandId::ZoomOut
            | CommandId::FitToScreen
            | CommandId::ActualSize
            | CommandId::SetViewMode(_)
            | CommandId::TogglePreview
            | CommandId::TogglePanel(_)
            | CommandId::ResetWorkspace
            | CommandId::DesignGallery
            | CommandId::KeyboardShortcuts
            | CommandId::About
            | CommandId::Preferences
            | CommandId::Save
            | CommandId::SaveAs
    )
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
