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
    Group,
    Ungroup,
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
    Snapping,
    DesignGallery,
    // Vehicle
    ChooseVehicle,
    VehicleInfo,
    // Export
    ExportPng,
    ExportMod,
    // Help
    KeyboardShortcuts,
    About,
    // Tools
    SelectTool(Tool),
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

/// Whether a command can run, independent of the current state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Availability {
    Always,
    NeedsProject,
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

const SOON_EDITING: &str = "Available once canvas editing is added.";
const SOON_FILES: &str = "Opening and saving projects is not available yet.";
const SOON_VEHICLES: &str = "Vehicle templates are not available yet.";
const SOON_EXPORT: &str = "Export is not available yet.";

impl CommandId {
    /// Every command, including each parameterized variant.
    pub fn all() -> Vec<Self> {
        use CommandId::*;
        let mut all = vec![
            NewProject,
            OpenProject,
            Save,
            SaveAs,
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
            Group,
            Ungroup,
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
            Snapping,
            DesignGallery,
            ChooseVehicle,
            VehicleInfo,
            ExportPng,
            ExportMod,
            KeyboardShortcuts,
            About,
        ];
        all.extend([ViewMode::TwoD, ViewMode::ThreeD, ViewMode::Split].map(SetViewMode));
        all.extend(PanelKind::ALL.map(TogglePanel));
        all.extend(Tool::ALL.map(SelectTool));
        all
    }

    pub fn meta(self) -> CommandMeta {
        use Availability::{Always, NeedsProject, NotYet};
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
                NotYet(SOON_FILES),
            ),
            Save => m(
                "Save",
                Some(icons::SAVE),
                const { &[sc(CMD, Key::S)] },
                Global,
                NotYet(SOON_FILES),
            ),
            SaveAs => m(
                "Save As…",
                None,
                const { &[sc(CMD_SHIFT, Key::S)] },
                Global,
                NotYet(SOON_FILES),
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
                NotYet(SOON_EDITING),
            ),
            Redo => m(
                "Redo",
                None,
                const { &[sc(CMD_SHIFT, Key::Z), sc(CMD, Key::Y)] },
                Workspace,
                NotYet(SOON_EDITING),
            ),
            Cut => m(
                "Cut",
                None,
                const { &[sc(CMD, Key::X)] },
                Workspace,
                NotYet(SOON_EDITING),
            ),
            Copy => m(
                "Copy",
                None,
                const { &[sc(CMD, Key::C)] },
                Workspace,
                NotYet(SOON_EDITING),
            ),
            Paste => m(
                "Paste",
                None,
                const { &[sc(CMD, Key::V)] },
                Workspace,
                NotYet(SOON_EDITING),
            ),
            Duplicate => m(
                "Duplicate",
                None,
                const { &[sc(CMD, Key::D)] },
                Workspace,
                NotYet(SOON_EDITING),
            ),
            Delete => m(
                "Delete",
                None,
                const { &[sc(NONE, Key::Delete)] },
                Workspace,
                NotYet(SOON_EDITING),
            ),
            SelectAll => m(
                "Select All",
                None,
                const { &[sc(CMD, Key::A)] },
                Workspace,
                NotYet(SOON_EDITING),
            ),

            Group => m(
                "Group",
                None,
                const { &[sc(CMD, Key::G)] },
                Workspace,
                NotYet(SOON_EDITING),
            ),
            Ungroup => m(
                "Ungroup",
                None,
                const { &[sc(CMD_SHIFT, Key::G)] },
                Workspace,
                NotYet(SOON_EDITING),
            ),
            BringForward => m(
                "Bring Forward",
                None,
                const { &[sc(CMD, Key::CloseBracket)] },
                Workspace,
                NotYet(SOON_EDITING),
            ),
            SendBackward => m(
                "Send Backward",
                None,
                const { &[sc(CMD, Key::OpenBracket)] },
                Workspace,
                NotYet(SOON_EDITING),
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
                None,
                const { &[sc(CMD_SHIFT, Key::N)] },
                Workspace,
                NotYet(SOON_EDITING),
            ),
            DuplicateLayer => m(
                "Duplicate Layer",
                None,
                &[],
                Workspace,
                NotYet(SOON_EDITING),
            ),
            DeleteLayer => m("Delete Layer", None, &[], Workspace, NotYet(SOON_EDITING)),

            ZoomIn => m(
                "Zoom In",
                None,
                const { &[sc(CMD, Key::Equals), sc(CMD, Key::Plus)] },
                Workspace,
                NotYet(SOON_EDITING),
            ),
            ZoomOut => m(
                "Zoom Out",
                None,
                const { &[sc(CMD, Key::Minus)] },
                Workspace,
                NotYet(SOON_EDITING),
            ),
            FitToScreen => m(
                "Fit to Screen",
                None,
                const { &[sc(CMD, Key::Num0)] },
                Workspace,
                NotYet(SOON_EDITING),
            ),
            ActualSize => m(
                "Actual Size (100%)",
                None,
                const { &[sc(CMD, Key::Num1)] },
                Workspace,
                NotYet(SOON_EDITING),
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
                NotYet(SOON_EDITING),
            ),
            ShowGuides => m(
                "Show Guides",
                None,
                const { &[sc(CMD, Key::Semicolon)] },
                Workspace,
                NotYet(SOON_EDITING),
            ),
            Snapping => m(
                "Snapping",
                None,
                const { &[sc(CMD_SHIFT, Key::Semicolon)] },
                Workspace,
                NotYet(SOON_EDITING),
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

            ExportPng => m(
                "Export PNG…",
                None,
                const { &[sc(CMD, Key::E)] },
                Workspace,
                NotYet(SOON_EXPORT),
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
    fn plain_key_does_not_match_with_command() {
        assert!(!modifiers_match(Modifiers::COMMAND, Modifiers::NONE));
        assert!(modifiers_match(Modifiers::NONE, Modifiers::NONE));
        assert!(!modifiers_match(Modifiers::SHIFT, Modifiers::NONE));
    }
}
