//! Command registry: every user action (menu, tool bar, context menu,
//! shortcut) is a [`CommandId`] with metadata and a single dispatch path.

use egui::{Key, KeyboardShortcut, Modifiers};
use tp_i18n::tr;
use tp_ui::icons;

use tp_core::document::{BooleanOp, DistributeAxis, DistributeMode, Edge};

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
    CreateOutlines,
    Align(Edge),
    Distribute(DistributeAxis, DistributeMode),
    Combine(BooleanOp),
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
    VehicleLibrary,
    AddVehicle,
    VehicleInfo,
    UpdateTemplate,
    ShowTemplate,
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
    /// Number of selected objects.
    pub selection_count: usize,
    /// The selection holds a text (maybe in a group).
    pub selection_has_text: bool,
    /// Why the selection cannot be combined (None: it can).
    pub combine_block: Option<&'static str>,
    /// Align to: Key object (aligning needs two objects).
    pub align_to_key: bool,
    /// The active surface has a template, and whether it is shown.
    pub has_template: bool,
    pub template_visible: bool,
    /// A newer installed version of the project's vehicle can be applied.
    pub update_available: bool,
}

fn can_align(c: &EditContext) -> bool {
    editable_selection(c)
        && !c.editing_text
        && c.selection_count >= if c.align_to_key { 2 } else { 1 }
}

fn can_distribute(c: &EditContext) -> bool {
    editable_selection(c) && !c.editing_text && c.selection_count >= 3
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
const ALT: Modifiers = Modifiers::ALT;
const ALT_SHIFT: Modifiers = Modifiers::ALT.plus(Modifiers::SHIFT);
const NONE: Modifiers = Modifiers::NONE;
const SHIFT: Modifiers = Modifiers::SHIFT;

const NEEDS_SELECTION: &str = "reason-no-selection";
const SOON_VEHICLES: &str = "reason-soon-vehicles";
const SOON_EXPORT: &str = "reason-soon-mods";

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
            CreateOutlines,
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
            VehicleLibrary,
            AddVehicle,
            VehicleInfo,
            UpdateTemplate,
            ShowTemplate,
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
        all.extend(Edge::ALL.map(Align));
        all.extend(BooleanOp::ALL.map(Combine));
        for axis in [DistributeAxis::Horizontal, DistributeAxis::Vertical] {
            for mode in [DistributeMode::Centers, DistributeMode::Spacing] {
                all.push(Distribute(axis, mode));
            }
        }
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
                "cmd-new-project",
                Some(icons::NEW_PROJECT),
                const { &[sc(CMD, Key::N)] },
                Global,
                Always,
            ),
            OpenProject => m(
                "cmd-open-project",
                Some(icons::OPEN_PROJECT),
                const { &[sc(CMD, Key::O)] },
                Global,
                When(|c| !c.gesture_active, ""),
            ),
            Save => m(
                "cmd-save",
                Some(icons::SAVE),
                const { &[sc(CMD, Key::S)] },
                Global,
                When(|c| c.has_project && !c.gesture_active, "reason-no-project"),
            ),
            SaveAs => m(
                "cmd-save-as",
                None,
                const { &[sc(CMD_SHIFT, Key::S)] },
                Global,
                When(|c| c.has_project && !c.gesture_active, "reason-no-project"),
            ),
            Place => m(
                "cmd-place",
                Some(icons::IMAGE),
                const { &[sc(CMD_SHIFT, Key::P)] },
                Workspace,
                When(|c| c.has_project && !c.gesture_active, ""),
            ),
            CloseProject => m(
                "cmd-close",
                None,
                const { &[sc(CMD, Key::W)] },
                Global,
                NeedsProject,
            ),
            Preferences => m(
                "cmd-preferences",
                Some(icons::SETTINGS),
                const { &[sc(CMD, Key::Comma)] },
                Global,
                Always,
            ),
            Quit => m(
                "cmd-quit",
                None,
                const { &[sc(CMD, Key::Q)] },
                Global,
                Always,
            ),

            Undo => m(
                "cmd-undo",
                None,
                const { &[sc(CMD, Key::Z)] },
                Workspace,
                When(
                    |c| c.has_project && (c.can_undo || c.editing_text) && !c.gesture_active,
                    "reason-nothing-to-undo",
                ),
            ),
            Redo => m(
                "cmd-redo",
                None,
                const { &[sc(CMD_SHIFT, Key::Z), sc(CMD, Key::Y)] },
                Workspace,
                When(
                    |c| c.has_project && (c.can_redo || c.editing_text) && !c.gesture_active,
                    "reason-nothing-to-redo",
                ),
            ),
            Cut => m(
                "cmd-cut",
                None,
                const { &[sc(CMD, Key::X)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            Copy => m(
                "cmd-copy",
                None,
                const { &[sc(CMD, Key::C)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            Paste => m(
                "cmd-paste",
                None,
                const { &[sc(CMD, Key::V)] },
                Workspace,
                When(
                    |c| c.has_project && c.has_clipboard && !c.gesture_active,
                    "reason-clipboard-empty",
                ),
            ),
            Duplicate => m(
                "cmd-duplicate",
                None,
                const { &[sc(CMD, Key::D)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            Delete => m(
                "cmd-delete",
                Some(icons::REMOVE),
                const { &[sc(NONE, Key::Delete), sc(NONE, Key::Backspace)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            SelectAll => m(
                "cmd-select-all",
                None,
                const { &[sc(CMD, Key::A)] },
                Workspace,
                When(|c| c.has_project && !c.gesture_active, ""),
            ),

            EditText => m(
                "cmd-edit-text",
                Some(icons::TEXT),
                const { &[sc(NONE, Key::Enter)] },
                Workspace,
                When(
                    |c| c.single_text && !c.gesture_active,
                    "reason-select-one-text",
                ),
            ),
            Group => m(
                "cmd-group",
                Some(icons::GROUP),
                const { &[sc(CMD, Key::G)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            Ungroup => m(
                "cmd-ungroup",
                Some(icons::UNGROUP),
                const { &[sc(CMD_SHIFT, Key::G)] },
                Workspace,
                When(
                    |c| editable_selection(c) && c.selection_has_group,
                    "reason-select-group",
                ),
            ),
            ConvertToPath => m(
                "cmd-convert-to-path",
                Some(icons::PEN),
                &[],
                Workspace,
                When(
                    |c| editable_selection(c) && c.selection_has_convertible,
                    "reason-select-convertible",
                ),
            ),
            CreateOutlines => m(
                "cmd-create-outlines",
                Some(icons::TEXT),
                const { &[sc(CMD_SHIFT, Key::O)] },
                Workspace,
                When(
                    |c| editable_selection(c) && c.selection_has_text,
                    "reason-select-text",
                ),
            ),
            Combine(op) => m(
                crate::combine::combine_label(op),
                Some(match op {
                    BooleanOp::Unite => icons::UNITE,
                    BooleanOp::MinusFront => icons::MINUS_FRONT,
                    BooleanOp::Intersect => icons::INTERSECT,
                    BooleanOp::Exclude => icons::EXCLUDE,
                }),
                match op {
                    BooleanOp::Unite => const { &[sc(CMD_SHIFT, Key::U)] },
                    BooleanOp::MinusFront => const { &[sc(CMD_SHIFT, Key::Minus)] },
                    _ => &[],
                },
                Workspace,
                When(
                    |c| editable_selection(c) && !c.editing_text && c.combine_block.is_none(),
                    "reason-select-two-shapes",
                ),
            ),
            Align(edge) => m(
                crate::arrange::align_label(edge),
                Some(match edge {
                    Edge::Left => icons::OBJ_ALIGN_LEFT,
                    Edge::HCenter => icons::OBJ_ALIGN_HCENTER,
                    Edge::Right => icons::OBJ_ALIGN_RIGHT,
                    Edge::Top => icons::OBJ_ALIGN_TOP,
                    Edge::VCenter => icons::OBJ_ALIGN_VCENTER,
                    Edge::Bottom => icons::OBJ_ALIGN_BOTTOM,
                }),
                match edge {
                    Edge::Left => const { &[sc(ALT, Key::A)] },
                    Edge::HCenter => const { &[sc(ALT, Key::H)] },
                    Edge::Right => const { &[sc(ALT, Key::D)] },
                    Edge::Top => const { &[sc(ALT, Key::W)] },
                    Edge::VCenter => const { &[sc(ALT, Key::V)] },
                    Edge::Bottom => const { &[sc(ALT, Key::S)] },
                },
                Workspace,
                When(can_align, "reason-select-to-align"),
            ),
            Distribute(axis, mode) => m(
                crate::arrange::distribute_label(axis, mode),
                Some(match (axis, mode) {
                    (DistributeAxis::Horizontal, DistributeMode::Centers) => {
                        icons::DISTRIBUTE_H_CENTERS
                    }
                    (DistributeAxis::Vertical, DistributeMode::Centers) => {
                        icons::DISTRIBUTE_V_CENTERS
                    }
                    (DistributeAxis::Horizontal, DistributeMode::Spacing) => {
                        icons::DISTRIBUTE_H_SPACING
                    }
                    (DistributeAxis::Vertical, DistributeMode::Spacing) => {
                        icons::DISTRIBUTE_V_SPACING
                    }
                }),
                match (axis, mode) {
                    (DistributeAxis::Horizontal, DistributeMode::Spacing) => {
                        const { &[sc(ALT_SHIFT, Key::H)] }
                    }
                    (DistributeAxis::Vertical, DistributeMode::Spacing) => {
                        const { &[sc(ALT_SHIFT, Key::V)] }
                    }
                    _ => &[],
                },
                Workspace,
                When(can_distribute, "reason-select-three"),
            ),
            BringForward => m(
                "cmd-bring-forward",
                None,
                const { &[sc(CMD, Key::CloseBracket)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            SendBackward => m(
                "cmd-send-backward",
                None,
                const { &[sc(CMD, Key::OpenBracket)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            MirrorToOtherSide => m(
                "cmd-mirror-to-other-side",
                None,
                &[],
                Workspace,
                NotYet(SOON_VEHICLES),
            ),

            NewLayer => m(
                "cmd-new-layer",
                Some(icons::NEW_LAYER),
                const { &[sc(CMD_SHIFT, Key::N)] },
                Workspace,
                When(|c| c.has_project && !c.gesture_active, ""),
            ),
            DuplicateLayer => m(
                "cmd-duplicate-layer",
                None,
                &[],
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            DeleteLayer => m(
                "cmd-delete-layer",
                None,
                &[],
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),

            ZoomIn => m(
                "cmd-zoom-in",
                None,
                const { &[sc(CMD, Key::Equals), sc(CMD, Key::Plus)] },
                Workspace,
                NeedsProject,
            ),
            ZoomOut => m(
                "cmd-zoom-out",
                None,
                const { &[sc(CMD, Key::Minus)] },
                Workspace,
                NeedsProject,
            ),
            FitToScreen => m(
                "cmd-fit-to-screen",
                None,
                const { &[sc(CMD, Key::Num0)] },
                Workspace,
                NeedsProject,
            ),
            ActualSize => m(
                "cmd-actual-size-100pct",
                None,
                const { &[sc(CMD, Key::Num1)] },
                Workspace,
                NeedsProject,
            ),
            SetViewMode(ViewMode::TwoD) => m(
                "cmd-2d-canvas",
                Some(icons::VIEW_2D),
                const { &[sc(CMD_ALT, Key::Num1)] },
                Workspace,
                NeedsProject,
            ),
            SetViewMode(ViewMode::ThreeD) => m(
                "cmd-3d-preview",
                Some(icons::VIEW_3D),
                const { &[sc(CMD_ALT, Key::Num2)] },
                Workspace,
                NeedsProject,
            ),
            SetViewMode(ViewMode::Split) => m(
                "cmd-split-view",
                Some(icons::VIEW_SPLIT),
                const { &[sc(CMD_ALT, Key::Num3)] },
                Workspace,
                NeedsProject,
            ),
            TogglePreview => m(
                "cmd-show-3d-preview",
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
                "cmd-reset-workspace",
                Some(icons::RESET),
                &[],
                Workspace,
                NeedsProject,
            ),
            ShowGrid => m(
                "cmd-show-grid",
                None,
                const { &[sc(CMD, Key::Quote)] },
                Workspace,
                NeedsProject,
            ),
            ShowGuides => m(
                "cmd-show-guides",
                None,
                const { &[sc(CMD, Key::Semicolon)] },
                Workspace,
                NeedsProject,
            ),
            ClearGuides => m(
                "cmd-clear-guides",
                None,
                &[],
                Workspace,
                When(
                    |c| c.has_project && c.has_guides && !c.gesture_active,
                    "reason-no-guides",
                ),
            ),
            Snapping => m(
                "cmd-snapping",
                None,
                const { &[sc(CMD_SHIFT, Key::Semicolon)] },
                Workspace,
                NeedsProject,
            ),
            DesignGallery => m("cmd-design-system-gallery", None, &[], App, Always),

            VehicleLibrary => m(
                "cmd-vehicle-library",
                Some(icons::VEHICLE),
                &[],
                App,
                Always,
            ),
            AddVehicle => m(
                "cmd-add-vehicle",
                Some(icons::ADD),
                &[],
                Workspace,
                When(|c| c.has_project && !c.gesture_active, "reason-no-project"),
            ),
            VehicleInfo => m(
                "cmd-vehicle-information",
                None,
                &[],
                Workspace,
                NeedsProject,
            ),
            UpdateTemplate => m(
                "cmd-update-template",
                None,
                &[],
                Workspace,
                When(
                    |c| c.update_available && !c.gesture_active,
                    "reason-no-update",
                ),
            ),
            ShowTemplate => m(
                "cmd-show-template",
                None,
                const { &[sc(SHIFT, Key::T)] },
                Workspace,
                When(|c| c.has_template, "reason-no-template"),
            ),

            ExportTexture => m(
                "cmd-export-texture",
                Some(icons::EXPORT),
                const { &[sc(CMD, Key::E)] },
                Workspace,
                When(|c| c.has_project && !c.gesture_active, "reason-no-project"),
            ),
            ExportMod => m(
                "cmd-export-mod",
                None,
                const { &[sc(CMD_SHIFT, Key::E)] },
                Workspace,
                NotYet(SOON_EXPORT),
            ),

            KeyboardShortcuts => m(
                "cmd-keyboard-shortcuts",
                None,
                const { &[sc(CMD, Key::Slash)] },
                App,
                Always,
            ),
            About => m("cmd-about-truckpaint", None, &[], App, Always),

            SwapColorTarget => m(
                "cmd-switch-fill-stroke-target",
                None,
                const { &[sc(NONE, Key::X)] },
                Workspace,
                NeedsProject,
            ),
            SwapFillStroke => m(
                "cmd-swap-fill-and-stroke",
                None,
                const { &[sc(SHIFT, Key::X)] },
                Workspace,
                NeedsProject,
            ),
            DefaultColors => m(
                "cmd-default-colors",
                None,
                const { &[sc(NONE, Key::D)] },
                Workspace,
                NeedsProject,
            ),
            Deselect => m(
                "cmd-deselect",
                None,
                const { &[sc(NONE, Key::Escape)] },
                Workspace,
                When(editable_selection, NEEDS_SELECTION),
            ),
            Nudge(direction, big) => m(
                match (direction, big) {
                    (Direction::Left, false) => "cmd-nudge-left",
                    (Direction::Right, false) => "cmd-nudge-right",
                    (Direction::Up, false) => "cmd-nudge-up",
                    (Direction::Down, false) => "cmd-nudge-down",
                    (Direction::Left, true) => "cmd-nudge-left-10",
                    (Direction::Right, true) => "cmd-nudge-right-10",
                    (Direction::Up, true) => "cmd-nudge-up-10",
                    (Direction::Down, true) => "cmd-nudge-down-10",
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
        Tool::Gradient => const { &[sc(NONE, Key::G)] },
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
            s.push_str(&key);
            s
        } else {
            let mut names: Vec<&str> = parts.iter().filter(|p| p.0).map(|p| p.2).collect();
            names.push(&key);
            names.join("+")
        }
    } else {
        // Windows and Linux: modifier names of the current language.
        let mut names = Vec::new();
        if m.ctrl || m.command {
            names.push(tr("key-ctrl"));
        }
        if m.alt {
            names.push(tr("key-alt"));
        }
        if m.shift {
            names.push(tr("key-shift"));
        }
        names.push(key.into_owned());
        names.join("+")
    }
}

fn key_label(key: Key) -> std::borrow::Cow<'static, str> {
    let named = match key {
        Key::Delete => "key-delete",
        Key::Escape => "key-escape",
        Key::Enter => "key-enter",
        Key::Backspace => "key-backspace",
        Key::Space => "key-space",
        Key::Tab => "key-tab",
        _ => "",
    };
    if !named.is_empty() {
        return tr(named).into();
    }
    std::borrow::Cow::Borrowed(match key {
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
        other => other.symbol_or_name(),
    })
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
