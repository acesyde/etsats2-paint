//! Command registry: every user action (menu, tool bar, context menu,
//! shortcut) is a [`CommandId`] with metadata and a single dispatch path.

use egui::{Key, KeyboardShortcut, Modifiers};
use tp_ui::icons;

use crate::layout::{PanelKind, ViewMode};
use crate::tool::Tool;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CommandId {
    // File
    NewProject,
    OpenProject,
    Save,
    SaveAs,
    Place,
    CloseProject,
    Preferences,
    Quit,
    // Edit
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Duplicate,
    Delete,
    SelectAll,
    // Object
    EditText,
    Group,
    Ungroup,
    ConvertToPath,
    BringForward,
    SendBackward,
    MirrorToOtherSide,
    // Layer
    NewLayer,
    DuplicateLayer,
    DeleteLayer,
    // View
    ZoomIn,
    ZoomOut,
    FitToScreen,
    ActualSize,
    SetViewMode(ViewMode),
    TogglePreview,
    TogglePanel(PanelKind),
    ResetWorkspace,
    ShowGrid,
    ShowGuides,
    ClearGuides,
    Snapping,
    DesignGallery,
    // Vehicle
    ChooseVehicle,
    VehicleInfo,
    // Export
    ExportTexture,
    ExportMod,
    // Help
    KeyboardShortcuts,
    About,
    // Colors
    SwapColorTarget,
    SwapFillStroke,
    DefaultColors,
    // Canvas
    Deselect,
    Nudge(Direction, bool),
    // Tools
    SelectTool(Tool),
}

/// Arrow-key nudge direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub const ALL: [Self; 4] = [Self::Left, Self::Right, Self::Up, Self::Down];

    /// Unit step in texture pixels.
    pub fn delta(self) -> (f64, f64) {
        match self {
            Self::Left => (-1.0, 0.0),
            Self::Right => (1.0, 0.0),
            Self::Up => (0.0, -1.0),
            Self::Down => (0.0, 1.0),
        }
    }
}

/// State that decides whether commands can run, built once per frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct EditContext {
    pub has_project: bool,
    pub has_selection: bool,
    pub can_undo: bool,
    pub can_redo: bool,
    pub has_clipboard: bool,
    /// A canvas drag (move, resize, draw…) is in progress.
    pub gesture_active: bool,
    pub undo_label: Option<&'static str>,
    pub redo_label: Option<&'static str>,
    /// At least one selected object is a group.
    pub selection_has_group: bool,
    /// Exactly one editable (visible, unlocked) text is selected.
    pub single_text: bool,
    /// A text is being edited on the canvas.
    pub editing_text: bool,
    /// The selection holds a rectangle, ellipse or polygon (maybe in a group).
    pub selection_has_convertible: bool,
    /// The active surface has guides.
    pub has_guides: bool,
}

fn editable_selection(c: &EditContext) -> bool {
    c.has_project && c.has_selection && !c.gesture_active
}

/// Where a command's shortcut is active.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// Always, even while typing in a text field.
    Global,
    /// On any screen, but not while typing.
    App,
    /// Only with a project open, and not while typing.
    Workspace,
}

/// Whether a command can run.
#[derive(Clone, Copy, Debug)]
pub enum Availability {
    Always,
    NeedsProject,
    /// Depends on the editing state; the string explains when it is disabled.
    When(fn(&EditContext) -> bool, &'static str),
    /// Feature not built yet; the string explains it in tooltips.
    NotYet(&'static str),
}

#[derive(Clone, Copy, Debug)]
pub struct CommandMeta {
    pub label: &'static str,
    pub icon: Option<&'static str>,
    /// First entry is the one displayed; others are alternatives.
    pub shortcuts: &'static [KeyboardShortcut],
    pub scope: Scope,
    pub availability: Availability,
}

const fn sc(modifiers: Modifiers, key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(modifiers, key)
}

const CMD: Modifiers = Modifiers::COMMAND;
const CMD_SHIFT: Modifiers = Modifiers::COMMAND.plus(Modifiers::SHIFT);
const CMD_ALT: Modifiers = Modifiers::COMMAND.plus(Modifiers::ALT);
const NONE: Modifiers = Modifiers::NONE;
const SHIFT: Modifiers = Modifiers::SHIFT;

const NEEDS_SELECTION: &str = "Select one or more objects first.";
const SOON_VEHICLES: &str = "Vehicle templates are not available yet.";
const SOON_EXPORT: &str = "Game mods need vehicle templates, which are not available yet.";

impl CommandId {
    /// Every command, including each parameterized variant.
    pub fn all() -> Vec<Self> {
        use CommandId::*;
        let mut all = vec![
            NewProject,
            OpenProject,
            Save,
            SaveAs,
            Place,
            CloseProject,
            Preferences,
            Quit,
            Undo,
            Redo,
            Cut,
            Copy,
            Paste,
            Duplicate,
            Delete,
            SelectAll,
            EditText,
            Group,
            Ungroup,
            ConvertToPath,
            BringForward,
            SendBackward,
            MirrorToOtherSide,
            NewLayer,
            DuplicateLayer,
            DeleteLayer,
            ZoomIn,
            ZoomOut,
            FitToScreen,
            ActualSize,
            TogglePreview,
            ResetWorkspace,
            ShowGrid,
            ShowGuides,
            ClearGuides,
            Snapping,
            DesignGallery,
            ChooseVehicle,
            VehicleInfo,
            ExportTexture,
            ExportMod,
            KeyboardShortcuts,
            About,
        ];
        all.extend([ViewMode::TwoD, ViewMode::ThreeD, ViewMode::Split].map(SetViewMode));
        all.extend(PanelKind::ALL.map(TogglePanel));
        all.extend([SwapColorTarget, SwapFillStroke, DefaultColors, Deselect]);
        for direction in Direction::ALL {
            all.push(Nudge(direction, false));
            all.push(Nudge(direction, true));
        }
        all.extend(Tool::ALL.map(SelectTool));
        all
    }

    pub fn meta(self) -> CommandMeta {
        use Availability::{Always, NeedsProject, NotYet, When};
        use CommandId::*;
        use Scope::{App, Global, Workspace};

        let m = |label, icon, shortcuts, scope, availability| CommandMeta {
            label,
            icon,
            shortcuts,
            scope,
            availability,
        };
        match self {
            NewProject => m(
                "New Project…",
                Some(icons::NEW_PROJECT),
                const { &[sc(CMD, Key::N)] },
                Global,
                Always,
            ),
            OpenProject => m(
                "Open Project…",
                Some(icons::OPEN_PROJECT),
                const { &[sc(CMD, Key::O)] },
                Global,
                When(|c| !c.gesture_active, ""),
            ),
            Save => m(
                "Save",
                Some(icons::SAVE),
                const { &[sc(CMD, Key::S)] },
                Global,
                When(
                    |c| c.has_project && !c.gesture_active,
                    "Open or create a project first.",
                ),
            ),
            SaveAs => m(
                "Save As…",
                None,
                const { &[sc(CMD_SHIFT, Key::S)] },
                Global,
                When(
                    |c| c.has_project && !c.gesture_active,
                    "Open or create a project first.",
                ),
            ),
            Place => m(
                "Place…",
                Some(icons::IMAGE),
                const { &[sc(CMD_SHIFT, Key::P)] },
                Workspace,
                When(|c| c.has_project && !c.gesture_active, ""),
            ),
            CloseProject => m(
                "Close",
                None,
                const { &[sc(CMD, Key::W)] },
                Global,
                NeedsProject,
            ),
            Preferences => m(
                "Preferences…",
                Some(icons::SETTINGS),
                const { &[sc(CMD, Key::Comma)] },
                Global,
                Always,
            ),
            Quit => m("Quit", None, const { &[sc(CMD, Key::Q)] }, Global, Always),

            Undo => m(
                "Undo",
                None,
                const { &[sc(CMD, Key::Z)] },
                Workspace,
                When(
                    |c| c.has_project && (c.can_undo || c.editing_text) && !c.gesture_active,
                    "Nothing to undo.",
                ),
            ),
            Redo => m(
                "Redo",
                None,
                const { &[sc(CMD_SHIFT, Key::Z), sc(CMD, Key::Y)] },
                Workspace,
                When(
                    |c| c.has_project && (c.can_redo || c.editing_text) && !c.gesture_active,
                    "Nothing to redo.",
                ),
            ),
            Cut => m(
                "Cut",
                None,
                const { &[sc(CMD, Key::X)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            Copy => m(
                "Copy",
                None,
                const { &[sc(CMD, Key::C)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            Paste => m(
                "Paste",
                None,
                const { &[sc(CMD, Key::V)] },
                Workspace,
                When(
                    |c| c.has_project && c.has_clipboard && !c.gesture_active,
                    "The clipboard is empty.",
                ),
            ),
            Duplicate => m(
                "Duplicate",
                None,
                const { &[sc(CMD, Key::D)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            Delete => m(
                "Delete",
                Some(icons::REMOVE),
                const { &[sc(NONE, Key::Delete), sc(NONE, Key::Backspace)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            SelectAll => m(
                "Select All",
                None,
                const { &[sc(CMD, Key::A)] },
                Workspace,
                When(|c| c.has_project && !c.gesture_active, ""),
            ),

            EditText => m(
                "Edit Text",
                Some(icons::TEXT),
                const { &[sc(NONE, Key::Enter)] },
                Workspace,
                When(
                    |c| c.single_text && !c.gesture_active,
                    "Select one text first.",
                ),
            ),
            Group => m(
                "Group",
                Some(icons::GROUP),
                const { &[sc(CMD, Key::G)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            Ungroup => m(
                "Ungroup",
                Some(icons::UNGROUP),
                const { &[sc(CMD_SHIFT, Key::G)] },
                Workspace,
                When(
                    |c| editable_selection(c) && c.selection_has_group,
                    "Select a group first.",
                ),
            ),
            ConvertToPath => m(
                "Convert to Path",
                Some(icons::PEN),
                &[],
                Workspace,
                When(
                    |c| editable_selection(c) && c.selection_has_convertible,
                    "Select a rectangle, ellipse or polygon first.",
                ),
            ),
            BringForward => m(
                "Bring Forward",
                None,
                const { &[sc(CMD, Key::CloseBracket)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            SendBackward => m(
                "Send Backward",
                None,
                const { &[sc(CMD, Key::OpenBracket)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            MirrorToOtherSide => m(
                "Mirror to Other Side",
                None,
                &[],
                Workspace,
                NotYet(SOON_VEHICLES),
            ),

            NewLayer => m(
                "New Layer",
                Some(icons::NEW_LAYER),
                const { &[sc(CMD_SHIFT, Key::N)] },
                Workspace,
                When(|c| c.has_project && !c.gesture_active, ""),
            ),
            DuplicateLayer => m(
                "Duplicate Layer",
                None,
                &[],
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            DeleteLayer => m(
                "Delete Layer",
                None,
                &[],
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),

            ZoomIn => m(
                "Zoom In",
                None,
                const { &[sc(CMD, Key::Equals), sc(CMD, Key::Plus)] },
                Workspace,
                NeedsProject,
            ),
            ZoomOut => m(
                "Zoom Out",
                None,
                const { &[sc(CMD, Key::Minus)] },
                Workspace,
                NeedsProject,
            ),
            FitToScreen => m(
                "Fit to Screen",
                None,
                const { &[sc(CMD, Key::Num0)] },
                Workspace,
                NeedsProject,
            ),
            ActualSize => m(
                "Actual Size (100%)",
                None,
                const { &[sc(CMD, Key::Num1)] },
                Workspace,
                NeedsProject,
            ),
            SetViewMode(ViewMode::TwoD) => m(
                "2D Canvas",
                Some(icons::VIEW_2D),
                const { &[sc(CMD_ALT, Key::Num1)] },
                Workspace,
                NeedsProject,
            ),
            SetViewMode(ViewMode::ThreeD) => m(
                "3D Preview",
                Some(icons::VIEW_3D),
                const { &[sc(CMD_ALT, Key::Num2)] },
                Workspace,
                NeedsProject,
            ),
            SetViewMode(ViewMode::Split) => m(
                "Split View",
                Some(icons::VIEW_SPLIT),
                const { &[sc(CMD_ALT, Key::Num3)] },
                Workspace,
                NeedsProject,
            ),
            TogglePreview => m(
                "Show 3D Preview",
                Some(icons::PREVIEW_3D),
                &[],
                Workspace,
                NeedsProject,
            ),
            TogglePanel(kind) => m(
                kind.title(),
                Some(kind.icon()),
                match kind {
                    PanelKind::Colors => const { &[sc(NONE, Key::F6)] },
                    PanelKind::Layers => const { &[sc(NONE, Key::F7)] },
                    PanelKind::Properties => const { &[sc(NONE, Key::F8)] },
                    _ => &[],
                },
                Workspace,
                NeedsProject,
            ),
            ResetWorkspace => m(
                "Reset Workspace",
                Some(icons::RESET),
                &[],
                Workspace,
                NeedsProject,
            ),
            ShowGrid => m(
                "Show Grid",
                None,
                const { &[sc(CMD, Key::Quote)] },
                Workspace,
                NeedsProject,
            ),
            ShowGuides => m(
                "Show Guides",
                None,
                const { &[sc(CMD, Key::Semicolon)] },
                Workspace,
                NeedsProject,
            ),
            ClearGuides => m(
                "Clear Guides",
                None,
                &[],
                Workspace,
                When(
                    |c| c.has_project && c.has_guides && !c.gesture_active,
                    "The texture has no guides.",
                ),
            ),
            Snapping => m(
                "Snapping",
                None,
                const { &[sc(CMD_SHIFT, Key::Semicolon)] },
                Workspace,
                NeedsProject,
            ),
            DesignGallery => m("Design System Gallery", None, &[], App, Always),

            ChooseVehicle => m(
                "Choose Vehicle…",
                Some(icons::VEHICLE),
                &[],
                Workspace,
                NotYet(SOON_VEHICLES),
            ),
            VehicleInfo => m(
                "Vehicle Information",
                None,
                &[],
                Workspace,
                NotYet(SOON_VEHICLES),
            ),

            ExportTexture => m(
                "Export Texture…",
                Some(icons::EXPORT),
                const { &[sc(CMD, Key::E)] },
                Workspace,
                When(
                    |c| c.has_project && !c.gesture_active,
                    "Open or create a project first.",
                ),
            ),
            ExportMod => m(
                "Export Mod…",
                None,
                const { &[sc(CMD_SHIFT, Key::E)] },
                Workspace,
                NotYet(SOON_EXPORT),
            ),

            KeyboardShortcuts => m(
                "Keyboard Shortcuts",
                None,
                const { &[sc(CMD, Key::Slash)] },
                App,
                Always,
            ),
            About => m("About TruckPaint", None, &[], App, Always),

            SwapColorTarget => m(
                "Switch Fill/Stroke Target",
                None,
                const { &[sc(NONE, Key::X)] },
                Workspace,
                NeedsProject,
            ),
            SwapFillStroke => m(
                "Swap Fill and Stroke",
                None,
                const { &[sc(SHIFT, Key::X)] },
                Workspace,
                NeedsProject,
            ),
            DefaultColors => m(
                "Default Colors",
                None,
                const { &[sc(NONE, Key::D)] },
                Workspace,
                NeedsProject,
            ),
            Deselect => m(
                "Deselect",
                None,
                const { &[sc(NONE, Key::Escape)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            Nudge(direction, big) => m(
                match (direction, big) {
                    (Direction::Left, false) => "Nudge Left",
                    (Direction::Right, false) => "Nudge Right",
                    (Direction::Up, false) => "Nudge Up",
                    (Direction::Down, false) => "Nudge Down",
                    (Direction::Left, true) => "Nudge Left ×10",
                    (Direction::Right, true) => "Nudge Right ×10",
                    (Direction::Up, true) => "Nudge Up ×10",
                    (Direction::Down, true) => "Nudge Down ×10",
                },
                None,
                nudge_shortcut(direction, big),
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            SelectTool(tool) => m(
                tool.name(),
                Some(tool.icon()),
                tool_shortcut(tool),
                Workspace,
                NeedsProject,
            ),
        }
    }

    /// Primary (displayed) shortcut.
    pub fn shortcut(self) -> Option<KeyboardShortcut> {
        self.meta().shortcuts.first().copied()
    }
}

fn nudge_shortcut(direction: Direction, big: bool) -> &'static [KeyboardShortcut] {
    match (direction, big) {
        (Direction::Left, false) => const { &[sc(NONE, Key::ArrowLeft)] },
        (Direction::Right, false) => const { &[sc(NONE, Key::ArrowRight)] },
        (Direction::Up, false) => const { &[sc(NONE, Key::ArrowUp)] },
        (Direction::Down, false) => const { &[sc(NONE, Key::ArrowDown)] },
        (Direction::Left, true) => const { &[sc(SHIFT, Key::ArrowLeft)] },
        (Direction::Right, true) => const { &[sc(SHIFT, Key::ArrowRight)] },
        (Direction::Up, true) => const { &[sc(SHIFT, Key::ArrowUp)] },
        (Direction::Down, true) => const { &[sc(SHIFT, Key::ArrowDown)] },
    }
}

fn tool_shortcut(tool: Tool) -> &'static [KeyboardShortcut] {
    match tool {
        Tool::Select => const { &[sc(NONE, Key::V)] },
        Tool::DirectSelect => const { &[sc(NONE, Key::A)] },
        Tool::Move => const { &[sc(NONE, Key::M)] },
        Tool::Rectangle => const { &[sc(NONE, Key::R)] },
        Tool::Ellipse => const { &[sc(NONE, Key::E)] },
        Tool::Polygon => const { &[sc(NONE, Key::Y)] },
        Tool::Pen => const { &[sc(NONE, Key::P)] },
        Tool::Line => const { &[sc(NONE, Key::Backslash)] },
        Tool::Text => const { &[sc(NONE, Key::T)] },
        Tool::Image => const { &[sc(SHIFT, Key::I)] },
        Tool::Eyedropper => const { &[sc(NONE, Key::I)] },
        Tool::Zoom => const { &[sc(NONE, Key::Z)] },
        Tool::Hand => const { &[sc(NONE, Key::H)] },
    }
}

/// Formats a shortcut in the platform's notation.
///
/// macOS uses Apple's symbol order `⌃⌥⇧⌘` followed by the key (e.g. `⇧⌘Z`)
/// when `symbols` is true; other platforms use `Ctrl+Alt+Shift+Z`.
pub fn format_shortcut(shortcut: &KeyboardShortcut, is_mac: bool, symbols: bool) -> String {
    let m = shortcut.modifiers;
    let key = key_label(shortcut.logical_key);
    if is_mac {
        let parts = [
            (m.ctrl, "⌃", "Ctrl"),
            (m.alt, "⌥", "Option"),
            (m.shift, "⇧", "Shift"),
            (m.command || m.mac_cmd, "⌘", "Cmd"),
        ];
        if symbols {
            let mut s: String = parts.iter().filter(|p| p.0).map(|p| p.1).collect();
            s.push_str(key);
            s
        } else {
            let mut names: Vec<&str> = parts.iter().filter(|p| p.0).map(|p| p.2).collect();
            names.push(key);
            names.join("+")
        }
    } else {
        let mut names = Vec::new();
        if m.ctrl || m.command {
            names.push("Ctrl");
        }
        if m.alt {
            names.push("Alt");
        }
        if m.shift {
            names.push("Shift");
        }
        names.push(key);
        names.join("+")
    }
}

fn key_label(key: Key) -> &'static str {
    match key {
        Key::Comma => ",",
        Key::Slash => "/",
        Key::Backslash => "\\",
        Key::Quote => "'",
        Key::Semicolon => ";",
        Key::OpenBracket => "[",
        Key::CloseBracket => "]",
        Key::Equals => "=",
        Key::Minus => "-",
        Key::Plus => "+",
        Key::Delete => "Del",
        Key::Escape => "Esc",
        other => other.symbol_or_name(),
    }
}

/// Glyphs needed to display macOS modifier symbols.
pub const MAC_SYMBOL_GLYPHS: &str = "⌃⌥⇧⌘";

/// Formats shortcuts for the current OS. Build it once per frame: it checks
/// whether the fonts can display the macOS modifier symbols, and falls back
/// to names otherwise.
#[derive(Clone, Copy, Debug)]
pub struct ShortcutFormatter {
    is_mac: bool,
    symbols: bool,
}

impl ShortcutFormatter {
    pub fn new(ctx: &egui::Context) -> Self {
        let is_mac = ctx.os().is_mac();
        let symbols = is_mac
            && ctx.fonts_mut(|f| {
                let font = egui::FontId::proportional(12.0);
                MAC_SYMBOL_GLYPHS.chars().all(|c| f.has_glyph(&font, c))
            });
        Self { is_mac, symbols }
    }

    pub fn format(&self, shortcut: &KeyboardShortcut) -> String {
        format_shortcut(shortcut, self.is_mac, self.symbols)
    }

    /// The command's displayed shortcut, if any.
    pub fn command(&self, id: CommandId) -> Option<String> {
        id.shortcut().map(|s| self.format(&s))
    }
}

/// Exact modifier match: `COMMAND` stands for Cmd on macOS and Ctrl elsewhere,
/// and extra modifiers are not tolerated (so `R` does not fire on `Cmd+R`).
fn modifiers_match(pressed: Modifiers, pattern: Modifiers) -> bool {
    pressed.command == pattern.command
        && pressed.shift == pattern.shift
        && pressed.alt == pattern.alt
        && (pattern.command || !(pressed.ctrl || pressed.mac_cmd))
}

/// Consumes key presses matching enabled commands' shortcuts and returns the
/// triggered commands. Shortcuts with more modifiers are matched first.
pub fn take_triggered(
    input: &mut egui::InputState,
    typing: bool,
    has_project: bool,
    is_enabled: impl Fn(CommandId) -> bool,
) -> Vec<CommandId> {
    let mut candidates: Vec<(CommandId, KeyboardShortcut)> = CommandId::all()
        .into_iter()
        .filter(|id| {
            let meta = id.meta();
            match meta.scope {
                Scope::Global => true,
                Scope::App => !typing,
                Scope::Workspace => !typing && has_project,
            }
        })
        .flat_map(|id| id.meta().shortcuts.iter().map(move |s| (id, *s)))
        .collect();
    let modifier_count = |m: Modifiers| u8::from(m.command) + u8::from(m.shift) + u8::from(m.alt);
    candidates.sort_by_key(|(_, s)| std::cmp::Reverse(modifier_count(s.modifiers)));

    let mut triggered = Vec::new();
    input.events.retain(|event| {
        let egui::Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } = event
        else {
            return true;
        };
        let hit = candidates
            .iter()
            .find(|(_, s)| s.logical_key == *key && modifiers_match(*modifiers, s.modifiers));
        match hit {
            Some((id, _)) if is_enabled(*id) => {
                triggered.push(*id);
                false
            }
            _ => true,
        }
    });
    triggered
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn every_command_has_a_label() {
        for id in CommandId::all() {
            assert!(!id.meta().label.is_empty(), "{id:?}");
        }
    }

    #[test]
    fn no_duplicate_default_shortcuts() {
        let mut seen: HashMap<(bool, bool, bool, Key), CommandId> = HashMap::new();
        for id in CommandId::all() {
            for s in id.meta().shortcuts {
                let m = s.modifiers;
                let key = (m.command, m.shift, m.alt, s.logical_key);
                if let Some(other) = seen.insert(key, id) {
                    panic!("{id:?} and {other:?} share shortcut {s:?}");
                }
            }
        }
    }

    #[test]
    fn mac_redo_uses_symbols_in_apple_order() {
        let redo = CommandId::Redo.shortcut().unwrap();
        assert_eq!(format_shortcut(&redo, true, true), "⇧⌘Z");
        assert_eq!(format_shortcut(&redo, true, false), "Shift+Cmd+Z");
    }

    #[test]
    fn other_platforms_use_ctrl_names() {
        let undo = CommandId::Undo.shortcut().unwrap();
        let redo = CommandId::Redo.shortcut().unwrap();
        assert_eq!(format_shortcut(&undo, false, false), "Ctrl+Z");
        assert_eq!(format_shortcut(&redo, false, false), "Ctrl+Shift+Z");
        assert_eq!(format_shortcut(&undo, true, true), "⌘Z");
    }

    #[test]
    fn editing_commands_follow_the_edit_context() {
        use crate::state::is_enabled;
        let idle = EditContext {
            has_project: true,
            ..EditContext::default()
        };
        assert!(!is_enabled(CommandId::Delete, &idle));
        assert!(!is_enabled(CommandId::Paste, &idle));
        assert!(!is_enabled(CommandId::Undo, &idle));
        assert!(is_enabled(CommandId::SelectAll, &idle));
        assert!(is_enabled(CommandId::FitToScreen, &idle));

        let selected = EditContext {
            has_selection: true,
            has_clipboard: true,
            can_undo: true,
            ..idle
        };
        for id in [
            CommandId::Delete,
            CommandId::Duplicate,
            CommandId::Copy,
            CommandId::Paste,
            CommandId::Undo,
        ] {
            assert!(is_enabled(id, &selected), "{id:?}");
        }
        let dragging = EditContext {
            gesture_active: true,
            ..selected
        };
        assert!(!is_enabled(CommandId::Delete, &dragging));
        assert!(!is_enabled(CommandId::Undo, &dragging));
        assert!(!is_enabled(CommandId::Deselect, &dragging));
    }

    #[test]
    fn grouping_commands_follow_the_selection() {
        use crate::state::is_enabled;
        let one = EditContext {
            has_project: true,
            has_selection: true,
            ..EditContext::default()
        };
        assert!(is_enabled(CommandId::Group, &one));
        assert!(!is_enabled(CommandId::Ungroup, &one));
        let group = EditContext {
            selection_has_group: true,
            ..one
        };
        assert!(is_enabled(CommandId::Ungroup, &group));
        assert!(is_enabled(
            CommandId::NewLayer,
            &EditContext {
                has_project: true,
                ..EditContext::default()
            }
        ));
    }

    #[test]
    fn plain_key_does_not_match_with_command() {
        assert!(!modifiers_match(Modifiers::COMMAND, Modifiers::NONE));
        assert!(modifiers_match(Modifiers::NONE, Modifiers::NONE));
        assert!(!modifiers_match(Modifiers::SHIFT, Modifiers::NONE));
    }
}
