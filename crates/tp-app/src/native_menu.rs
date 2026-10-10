//! The macOS menu bar, built with muda from the menus' data
//! ([`menus::menus_for`]), with the application and Window menus.
//!
//! A menu item whose shortcut has Command or Control takes its key from the
//! window: its click or key comes back as the input egui would have seen for
//! that key (a key press and release, or Copy, Cut and Paste), so the
//! shortcut dispatch, the text fields and the text being typed on the canvas
//! act as they do without the system menu, and the command runs once. Other
//! keys (Shift+H, Backspace, G…) are not taken by the menu and reach egui as
//! before; their items' clicks queue the command as the drawn menus do.

use std::collections::HashMap;
use std::sync::mpsc;

use egui::{Event, Key, KeyboardShortcut, Modifiers, RawInput};
use muda::accelerator::{Accelerator, Code, Modifiers as Mods};
use muda::{
    CheckMenuItem, IsMenuItem, Menu as NsMenu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
};
use tp_i18n::{Language, tr};

use crate::commands::{CommandId, EditContext};
use crate::layout::WorkspaceLayout;
use crate::menus::{self, Entry, Menu};
use crate::prefs::ViewAids;
use crate::state::{AppState, is_enabled};
use crate::ui::command_ui::command_label;

/// The shortcut `shortcut` as a muda accelerator; `None` for a key the menu
/// can't show.
pub fn accelerator(shortcut: &KeyboardShortcut) -> Option<Accelerator> {
    let m = shortcut.modifiers;
    let mut mods = Mods::empty();
    if m.command || m.mac_cmd {
        mods |= Mods::META;
    }
    if m.ctrl {
        mods |= Mods::CONTROL;
    }
    if m.alt {
        mods |= Mods::ALT;
    }
    if m.shift {
        mods |= Mods::SHIFT;
    }
    Some(Accelerator::new(mods, code(shortcut.logical_key)?))
}

fn code(key: Key) -> Option<Code> {
    use Code as C;
    Some(match key {
        Key::A => C::KeyA,
        Key::B => C::KeyB,
        Key::C => C::KeyC,
        Key::D => C::KeyD,
        Key::E => C::KeyE,
        Key::F => C::KeyF,
        Key::G => C::KeyG,
        Key::H => C::KeyH,
        Key::I => C::KeyI,
        Key::J => C::KeyJ,
        Key::K => C::KeyK,
        Key::L => C::KeyL,
        Key::M => C::KeyM,
        Key::N => C::KeyN,
        Key::O => C::KeyO,
        Key::P => C::KeyP,
        Key::Q => C::KeyQ,
        Key::R => C::KeyR,
        Key::S => C::KeyS,
        Key::T => C::KeyT,
        Key::U => C::KeyU,
        Key::V => C::KeyV,
        Key::W => C::KeyW,
        Key::X => C::KeyX,
        Key::Y => C::KeyY,
        Key::Z => C::KeyZ,
        Key::Num0 => C::Digit0,
        Key::Num1 => C::Digit1,
        Key::Num2 => C::Digit2,
        Key::Num3 => C::Digit3,
        Key::Num4 => C::Digit4,
        Key::Num5 => C::Digit5,
        Key::Num6 => C::Digit6,
        Key::Num7 => C::Digit7,
        Key::Num8 => C::Digit8,
        Key::Num9 => C::Digit9,
        Key::Comma => C::Comma,
        Key::Period => C::Period,
        Key::Slash => C::Slash,
        Key::Backslash => C::Backslash,
        Key::Quote => C::Quote,
        Key::Semicolon => C::Semicolon,
        Key::OpenBracket => C::BracketLeft,
        Key::CloseBracket => C::BracketRight,
        Key::Equals => C::Equal,
        Key::Minus => C::Minus,
        Key::Backtick => C::Backquote,
        Key::Space => C::Space,
        Key::Tab => C::Tab,
        Key::Enter => C::Enter,
        Key::Escape => C::Escape,
        Key::Backspace => C::Backspace,
        Key::Delete => C::Delete,
        Key::ArrowLeft => C::ArrowLeft,
        Key::ArrowRight => C::ArrowRight,
        Key::ArrowUp => C::ArrowUp,
        Key::ArrowDown => C::ArrowDown,
        Key::PageUp => C::PageUp,
        Key::PageDown => C::PageDown,
        Key::Home => C::Home,
        Key::End => C::End,
        _ => return None,
    })
}

/// Whether the menu takes `shortcut`'s key from the window: AppKit gives
/// the menu the keys with Command or Control only (checked by posting key
/// events: Shift+H, Backspace, Escape, Tab, Enter, G, Alt+A and 1 reach the
/// window even with an item bound to them).
pub fn menu_takes(shortcut: &KeyboardShortcut) -> bool {
    let m = shortcut.modifiers;
    m.command || m.mac_cmd || m.ctrl
}

/// The input egui-winit produces when `shortcut` is pressed on macOS: Copy,
/// Cut and Paste (of `clipboard`'s text, nothing when it holds none) for
/// Cmd+C, X and V, else the key pressed then released.
pub fn injected_events(shortcut: &KeyboardShortcut, clipboard: Option<String>) -> Vec<Event> {
    let m = shortcut.modifiers;
    let modifiers = Modifiers {
        alt: m.alt,
        ctrl: m.ctrl,
        shift: m.shift,
        mac_cmd: m.command || m.mac_cmd,
        command: m.command || m.mac_cmd,
    };
    let key = shortcut.logical_key;
    if modifiers.command {
        match key {
            Key::C => return vec![Event::Copy],
            Key::X => return vec![Event::Cut],
            Key::V => {
                return clipboard
                    .map(|text| text.replace("\r\n", "\n"))
                    .filter(|text| !text.is_empty())
                    .map(Event::Paste)
                    .into_iter()
                    .collect();
            }
            _ => {}
        }
    }
    [true, false]
        .map(|pressed| Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers,
        })
        .to_vec()
}

/// The commands that act on the text being typed (in a field or on the
/// canvas): their items stay enabled then, so their keys reach the text.
const TEXT_COMMANDS: [CommandId; 6] = [
    CommandId::Undo,
    CommandId::Redo,
    CommandId::Cut,
    CommandId::Copy,
    CommandId::Paste,
    CommandId::SelectAll,
];

/// Whether `id`'s item is enabled: as the command is, and for the text
/// commands also while a text is typed (`typing`).
pub fn item_enabled(id: CommandId, edit: &EditContext, typing: bool) -> bool {
    is_enabled(id, edit) || (typing && TEXT_COMMANDS.contains(&id))
}

/// A system item of the application or Window menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemItem {
    Services,
    Hide,
    HideOthers,
    ShowAll,
    Minimize,
    Zoom,
    Fullscreen,
    BringAllToFront,
}

/// A row of a menu of the menu bar, before it is made.
#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Command {
        id: CommandId,
        /// Shows a check mark ([`menus::is_checked`]).
        toggle: bool,
        label: String,
        accelerator: Option<Accelerator>,
    },
    System(SystemItem),
    Separator,
    Submenu {
        title: String,
        nodes: Vec<Node>,
    },
}

/// What a menu of the menu bar is to macOS.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    App,
    Plain,
    Window,
    Help,
}

/// A menu of the menu bar, before it is made.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuPlan {
    pub title: String,
    pub role: Role,
    pub nodes: Vec<Node>,
}

/// The menu bar in the current language: the menus of `menus` (the
/// application menu first, as [`menus::menus_for`] gives them) with the
/// system items of the application menu, and the Window menu before Help.
pub fn plan(menus: &[Menu]) -> Vec<MenuPlan> {
    fn nodes(entries: &[Entry], app: bool) -> Vec<Node> {
        let mut out = Vec::new();
        for entry in entries {
            match entry {
                Entry::Item(id) | Entry::Toggle(id) => {
                    if app && *id == CommandId::Quit {
                        out.extend([
                            Node::System(SystemItem::Services),
                            Node::Separator,
                            Node::System(SystemItem::Hide),
                            Node::System(SystemItem::HideOthers),
                            Node::System(SystemItem::ShowAll),
                            Node::Separator,
                        ]);
                    }
                    let label = match id {
                        CommandId::About if app => tr("app-menu-about"),
                        CommandId::Preferences if app => tr("app-menu-settings"),
                        CommandId::Quit if app => tr("app-menu-quit"),
                        _ => command_label(*id, &EditContext::default()),
                    };
                    out.push(Node::Command {
                        id: *id,
                        toggle: matches!(entry, Entry::Toggle(_)),
                        label,
                        accelerator: id.shortcut().as_ref().and_then(accelerator),
                    });
                }
                Entry::Separator => out.push(Node::Separator),
                Entry::Submenu { title, entries, .. } => out.push(Node::Submenu {
                    title: tr(title),
                    nodes: nodes(entries, false),
                }),
            }
        }
        out
    }
    let mut out = Vec::new();
    for menu in menus {
        let role = match menu.title {
            "menu-app" => Role::App,
            "menu-help" => Role::Help,
            _ => Role::Plain,
        };
        if role == Role::Help {
            out.push(MenuPlan {
                title: tr("menu-window"),
                role: Role::Window,
                nodes: vec![
                    Node::System(SystemItem::Minimize),
                    Node::System(SystemItem::Zoom),
                    Node::System(SystemItem::Fullscreen),
                    Node::Separator,
                    Node::System(SystemItem::BringAllToFront),
                ],
            });
        }
        out.push(MenuPlan {
            title: tr(menu.title),
            role,
            nodes: nodes(&menu.entries, role == Role::App),
        });
    }
    out
}

/// The menu id of `id`'s item.
fn item_id(id: CommandId) -> String {
    format!("{id:?}")
}

/// A command's item.
enum Item {
    Plain(MenuItem),
    Check(CheckMenuItem),
}

impl Item {
    fn set_enabled(&self, enabled: bool) {
        match self {
            Self::Plain(item) => item.set_enabled(enabled),
            Self::Check(item) => item.set_enabled(enabled),
        }
    }

    fn set_text(&self, text: &str) {
        match self {
            Self::Plain(item) => item.set_text(text),
            Self::Check(item) => item.set_text(text),
        }
    }
}

/// The command items of a menu bar being built.
#[derive(Default)]
struct Items {
    items: HashMap<CommandId, Item>,
    commands: HashMap<String, CommandId>,
}

impl Items {
    fn append(&mut self, parent: &Submenu, nodes: &[Node]) {
        for node in nodes {
            match node {
                Node::Command {
                    id,
                    toggle,
                    label,
                    accelerator,
                } => {
                    let key = item_id(*id);
                    let item = if *toggle {
                        let item =
                            CheckMenuItem::with_id(key.as_str(), label, true, false, *accelerator);
                        append(parent, &item);
                        Item::Check(item)
                    } else {
                        let item = MenuItem::with_id(key.as_str(), label, true, *accelerator);
                        append(parent, &item);
                        Item::Plain(item)
                    };
                    self.items.insert(*id, item);
                    self.commands.insert(key, *id);
                }
                Node::System(system) => append(parent, &system_item(*system)),
                Node::Separator => append(parent, &PredefinedMenuItem::separator()),
                Node::Submenu { title, nodes } => {
                    let submenu = Submenu::new(title, true);
                    self.append(&submenu, nodes);
                    append(parent, &submenu);
                }
            }
        }
    }
}

/// The installed menu bar.
struct Bar {
    /// Keeps the menus alive.
    _menu: NsMenu,
    items: HashMap<CommandId, Item>,
    /// Commands of the item ids.
    commands: HashMap<String, CommandId>,
    language: Language,
    /// Enabled and checked state last set on each item.
    shown: HashMap<CommandId, (bool, bool)>,
    /// Labels last set on the items whose label changes (Undo, Redo).
    labels: HashMap<CommandId, String>,
}

/// The macOS menu bar of the app.
pub struct NativeMenu {
    bar: Bar,
    events: mpsc::Receiver<MenuEvent>,
}

impl NativeMenu {
    /// Builds the menu bar in the current language and installs it in
    /// place of the default one. Call on the main thread.
    pub fn install(ctx: &egui::Context, language: Language) -> Self {
        let (tx, events) = mpsc::channel();
        let ctx = ctx.clone();
        // A menu event comes without a window event: ask for a frame.
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let _ = tx.send(event);
            ctx.request_repaint();
        }));
        Self {
            bar: Bar::build(language),
            events,
        }
    }

    /// Adds the menu's clicks and keys since the last frame to `raw` (the
    /// commands with a Command or Control shortcut) or `queue` (the
    /// others). Call from `raw_input_hook`.
    pub fn take_input(&mut self, raw: &mut RawInput, queue: &mut Vec<CommandId>) {
        for event in self.events.try_iter() {
            let Some(&id) = self.bar.commands.get(&event.id.0) else {
                continue;
            };
            match id.shortcut().filter(menu_takes) {
                Some(shortcut) => {
                    let clipboard = (shortcut.logical_key == Key::V)
                        .then(|| {
                            arboard::Clipboard::new()
                                .and_then(|mut c| c.get_text())
                                .ok()
                        })
                        .flatten();
                    raw.events.extend(injected_events(&shortcut, clipboard));
                }
                None => queue.push(id),
            }
        }
    }

    /// Brings the items up to date after the frame: rebuilt when the
    /// language changed, then the enabled and checked states and the Undo
    /// and Redo labels that changed.
    pub fn sync(&mut self, ctx: &egui::Context, state: &AppState) {
        let language = state.language();
        if language != self.bar.language {
            self.bar = Bar::build(language);
        }
        let edit = state.edit_context();
        let typing = ctx.egui_wants_keyboard_input() || edit.editing_text;
        let layout = state.has_project().then_some(&state.prefs.layout);
        self.bar.sync(&edit, typing, layout, state.prefs.view_aids);
    }
}

impl Bar {
    fn build(language: Language) -> Self {
        let started = std::time::Instant::now();
        let menu = NsMenu::new();
        let mut items = Items::default();
        let mut window_menu = None;
        let mut help_menu = None;
        for plan in plan(&menus::menus_for(true)) {
            let submenu = Submenu::new(&plan.title, true);
            items.append(&submenu, &plan.nodes);
            append(&menu, &submenu);
            match plan.role {
                Role::Window => window_menu = Some(submenu),
                Role::Help => help_menu = Some(submenu),
                Role::App | Role::Plain => {}
            }
        }
        menu.init_for_nsapp();
        if let Some(window) = window_menu {
            window.set_as_windows_menu_for_nsapp();
        }
        if let Some(help) = help_menu {
            help.set_as_help_menu_for_nsapp();
        }
        tracing::debug!(
            language = language.code(),
            elapsed = ?started.elapsed(),
            "menu bar built"
        );
        Self {
            _menu: menu,
            items: items.items,
            commands: items.commands,
            language,
            shown: HashMap::new(),
            labels: HashMap::new(),
        }
    }

    fn sync(
        &mut self,
        edit: &EditContext,
        typing: bool,
        layout: Option<&WorkspaceLayout>,
        aids: ViewAids,
    ) {
        for (id, item) in &self.items {
            let state = (
                item_enabled(*id, edit, typing),
                menus::is_checked(*id, edit, layout, aids),
            );
            if self.shown.get(id) != Some(&state) {
                item.set_enabled(state.0);
                if let Item::Check(check) = item {
                    check.set_checked(state.1);
                }
                self.shown.insert(*id, state);
            }
            if matches!(id, CommandId::Undo | CommandId::Redo) {
                let label = command_label(*id, edit);
                if self.labels.get(id) != Some(&label) {
                    item.set_text(&label);
                    self.labels.insert(*id, label);
                }
            }
        }
    }
}

fn system_item(item: SystemItem) -> PredefinedMenuItem {
    let text = |id| Some(tr(id));
    match item {
        SystemItem::Services => PredefinedMenuItem::services(text("app-menu-services").as_deref()),
        SystemItem::Hide => PredefinedMenuItem::hide(text("app-menu-hide").as_deref()),
        SystemItem::HideOthers => {
            PredefinedMenuItem::hide_others(text("app-menu-hide-others").as_deref())
        }
        SystemItem::ShowAll => PredefinedMenuItem::show_all(text("app-menu-show-all").as_deref()),
        SystemItem::Minimize => {
            PredefinedMenuItem::minimize(text("menu-window-minimize").as_deref())
        }
        SystemItem::Zoom => PredefinedMenuItem::zoom(text("menu-window-zoom").as_deref()),
        SystemItem::Fullscreen => {
            PredefinedMenuItem::fullscreen(text("menu-window-fullscreen").as_deref())
        }
        SystemItem::BringAllToFront => {
            PredefinedMenuItem::bring_all_to_front(text("menu-window-front").as_deref())
        }
    }
}

/// Appends `item` to `parent` (a menu or submenu); muda only fails on an
/// item already in it.
fn append(parent: &dyn Append, item: &dyn IsMenuItem) {
    if let Err(err) = parent.append_item(item) {
        tracing::warn!(%err, "menu item not added");
    }
}

trait Append {
    fn append_item(&self, item: &dyn IsMenuItem) -> muda::Result<()>;
}

impl Append for NsMenu {
    fn append_item(&self, item: &dyn IsMenuItem) -> muda::Result<()> {
        self.append(item)
    }
}

impl Append for Submenu {
    fn append_item(&self, item: &dyn IsMenuItem) -> muda::Result<()> {
        self.append(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::KeyboardShortcut as S;

    const CMD: Modifiers = Modifiers::COMMAND;

    #[test]
    fn accelerators() {
        let acc = |m, k| accelerator(&S::new(m, k)).unwrap();
        assert_eq!(acc(CMD, Key::S), Accelerator::new(Mods::META, Code::KeyS));
        assert_eq!(
            acc(CMD | Modifiers::SHIFT, Key::E),
            Accelerator::new(Mods::META | Mods::SHIFT, Code::KeyE)
        );
        assert_eq!(
            acc(CMD, Key::CloseBracket),
            Accelerator::new(Mods::META, Code::BracketRight)
        );
        assert_eq!(
            acc(CMD, Key::Comma),
            Accelerator::new(Mods::META, Code::Comma)
        );
        assert_eq!(
            acc(Modifiers::SHIFT, Key::H),
            Accelerator::new(Mods::SHIFT, Code::KeyH)
        );
        // Every displayed shortcut of the menus is shown in the menu bar.
        for plan in plan(&menus::menus_for(true)) {
            fn check(nodes: &[Node]) {
                for node in nodes {
                    match node {
                        Node::Command {
                            id, accelerator, ..
                        } => assert_eq!(accelerator.is_some(), id.shortcut().is_some(), "{id:?}"),
                        Node::Submenu { nodes, .. } => check(nodes),
                        _ => {}
                    }
                }
            }
            check(&plan.nodes);
        }
    }

    #[test]
    fn keys_come_back_as_egui_winit_sends_them() {
        let mac_cmd = Modifiers {
            mac_cmd: true,
            command: true,
            ..Modifiers::NONE
        };
        let key = |key, pressed| Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers: mac_cmd,
        };
        assert_eq!(
            injected_events(&S::new(CMD, Key::D), None),
            [key(Key::D, true), key(Key::D, false)]
        );
        assert_eq!(injected_events(&S::new(CMD, Key::C), None), [Event::Copy]);
        assert_eq!(injected_events(&S::new(CMD, Key::X), None), [Event::Cut]);
        assert_eq!(
            injected_events(&S::new(CMD, Key::V), Some("a\r\nb".into())),
            [Event::Paste("a\nb".into())]
        );
        assert!(injected_events(&S::new(CMD, Key::V), Some(String::new())).is_empty());
        assert!(injected_events(&S::new(CMD, Key::V), None).is_empty());
    }

    #[test]
    fn only_command_and_control_keys_are_taken() {
        assert!(menu_takes(&CommandId::Duplicate.shortcut().unwrap()));
        assert!(menu_takes(&CommandId::Quit.shortcut().unwrap()));
        for id in [
            CommandId::Flip(tp_core::document::FlipAxis::Horizontal),
            CommandId::Delete,
            CommandId::Deselect,
            CommandId::TogglePanels,
            CommandId::EditText,
            CommandId::Align(tp_core::document::Edge::Left),
        ] {
            assert!(!menu_takes(&id.shortcut().unwrap()), "{id:?}");
        }
    }

    fn commands(nodes: &[Node], out: &mut Vec<CommandId>) {
        for node in nodes {
            match node {
                Node::Command { id, .. } => out.push(*id),
                Node::Submenu { nodes, .. } => commands(nodes, out),
                _ => {}
            }
        }
    }

    #[test]
    fn every_menu_command_gets_one_item() {
        tp_i18n::set_language(Language::English);
        let menus = menus::menus_for(true);
        let mut listed = Vec::new();
        for menu in &menus {
            fn walk(entries: &[Entry], out: &mut Vec<CommandId>) {
                for entry in entries {
                    match entry {
                        Entry::Item(id) | Entry::Toggle(id) => out.push(*id),
                        Entry::Separator => {}
                        Entry::Submenu { entries, .. } => walk(entries, out),
                    }
                }
            }
            walk(&menu.entries, &mut listed);
        }
        let plans = plan(&menus);
        let mut items = Vec::new();
        for plan in &plans {
            commands(&plan.nodes, &mut items);
        }
        assert_eq!(items, listed);
        let mut unique = items.clone();
        unique.sort_by_key(|id| item_id(*id));
        unique.dedup();
        assert_eq!(unique.len(), items.len());

        // TruckPaint, the menus, Window before Help.
        let titles: Vec<_> = plans.iter().map(|p| (p.title.as_str(), p.role)).collect();
        assert_eq!(titles[0], ("TruckPaint", Role::App));
        assert_eq!(titles[titles.len() - 2], ("Window", Role::Window));
        assert_eq!(titles[titles.len() - 1], ("Help", Role::Help));
        assert_eq!(titles.len(), menus.len() + 1);
        // The application menu: About, Settings…, the system items, Quit.
        let app = &plans[0].nodes;
        let label = |n: &Node| match n {
            Node::Command { label, .. } => Some(label.clone()),
            _ => None,
        };
        assert_eq!(label(&app[0]).as_deref(), Some("About TruckPaint"));
        assert_eq!(label(&app[2]).as_deref(), Some("Settings…"));
        assert_eq!(
            label(app.last().unwrap()).as_deref(),
            Some("Quit TruckPaint")
        );
        assert!(app.contains(&Node::System(SystemItem::Services)));
        assert!(app.contains(&Node::System(SystemItem::HideOthers)));

        // Built again in German, the menu bar reads German.
        tp_i18n::set_language(Language::German);
        let titles: Vec<_> = plan(&menus).into_iter().map(|p| p.title).collect();
        tp_i18n::set_language(Language::English);
        assert_eq!(
            titles,
            [
                "TruckPaint",
                "Datei",
                "Bearbeiten",
                "Objekt",
                "Ebene",
                "Ansicht",
                "Fahrzeug",
                "Fenster",
                "Hilfe"
            ]
        );
    }

    #[test]
    fn enabled_follows_the_command_and_the_text_typed() {
        let selected = EditContext {
            has_project: true,
            has_selection: true,
            selection_count: 1,
            ..EditContext::default()
        };
        let mut ids = Vec::new();
        for plan in plan(&menus::menus_for(true)) {
            commands(&plan.nodes, &mut ids);
        }
        for edit in [EditContext::default(), selected] {
            for &id in &ids {
                assert_eq!(
                    item_enabled(id, &edit, false),
                    is_enabled(id, &edit),
                    "{id:?}"
                );
            }
        }
        let nothing = EditContext::default();
        assert!(!item_enabled(CommandId::Duplicate, &nothing, false));
        assert!(item_enabled(CommandId::Duplicate, &selected, false));
        for id in [CommandId::Undo, CommandId::Copy, CommandId::Paste] {
            assert!(!item_enabled(id, &nothing, false), "{id:?}");
            assert!(item_enabled(id, &nothing, true), "{id:?}");
        }
        assert!(!item_enabled(CommandId::Duplicate, &nothing, true));
    }
}
