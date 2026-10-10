//! How the top of the window is drawn: on macOS the native frame with the
//! top bar in its title strip, elsewhere the title bar drawn by the app or,
//! as an option, the system's title bar with an in-window menu row.

use egui::{Context, ViewportBuilder, ViewportCommand};

/// The environment variable forcing the system title bar (set to `1`), a
/// rescue when the drawn title bar can't be used.
pub const SYSTEM_TITLE_BAR_VAR: &str = "TRUCKPAINT_SYSTEM_TITLE_BAR";

/// Width kept for the traffic lights at the left of the macOS title strip,
/// in native points.
pub const TRAFFIC_LIGHTS_WIDTH: f32 = 78.0;

/// How the top of the window is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleBarMode {
    /// macOS: native frame, title hidden, the top bar in the title strip.
    MacNative,
    /// Windows and Linux: no system frame, the app draws the title bar.
    Drawn,
    /// Windows and Linux: the system's title bar, then the menu row.
    System,
}

/// The mode on macOS (`mac`), or with the Use the system title bar
/// preference (`pref`) or the environment variable (`forced`) elsewhere.
pub fn title_bar_mode(mac: bool, pref: bool, forced: bool) -> TitleBarMode {
    if mac {
        TitleBarMode::MacNative
    } else if pref || forced {
        TitleBarMode::System
    } else {
        TitleBarMode::Drawn
    }
}

/// Whether the environment variable forces the system title bar.
pub fn forced_by_environment() -> bool {
    std::env::var(SYSTEM_TITLE_BAR_VAR).is_ok_and(|v| v.trim() == "1")
}

/// The window as it opens in `mode`.
pub fn initial_viewport(viewport: ViewportBuilder, mode: TitleBarMode) -> ViewportBuilder {
    match mode {
        TitleBarMode::MacNative => viewport
            .with_fullsize_content_view(true)
            .with_title_shown(false)
            .with_titlebar_shown(false),
        TitleBarMode::Drawn => viewport.with_decorations(false),
        TitleBarMode::System => viewport,
    }
}

/// What a double-click on the title bar does (macOS: the system setting
/// "Double-click a window's title bar").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DoubleClickAction {
    /// Maximize (zoom on macOS) or restore.
    #[default]
    Maximize,
    Minimize,
    None,
}

impl DoubleClickAction {
    /// From the value of the `AppleActionOnDoubleClick` default.
    pub fn from_setting(value: Option<&str>) -> Self {
        match value {
            Some("Minimize") => Self::Minimize,
            Some("None") => Self::None,
            _ => Self::Maximize,
        }
    }

    /// The system's setting (macOS); maximize elsewhere.
    pub fn of_system() -> Self {
        #[cfg(target_os = "macos")]
        {
            use objc2_foundation::{NSString, NSUserDefaults};
            let key = NSString::from_str("AppleActionOnDoubleClick");
            let value = NSUserDefaults::standardUserDefaults()
                .stringForKey(&key)
                .map(|v| v.to_string());
            Self::from_setting(value.as_deref())
        }
        #[cfg(not(target_os = "macos"))]
        Self::Maximize
    }

    /// Does it to the window.
    pub fn apply(self, ctx: &Context) {
        match self {
            Self::Maximize => {
                let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
            }
            Self::Minimize => ctx.send_viewport_cmd(ViewportCommand::Minimized(true)),
            Self::None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_is_native_whatever_the_preference() {
        for pref in [false, true] {
            for forced in [false, true] {
                assert_eq!(title_bar_mode(true, pref, forced), TitleBarMode::MacNative);
            }
        }
    }

    #[test]
    fn elsewhere_drawn_by_default_system_by_choice() {
        assert_eq!(title_bar_mode(false, false, false), TitleBarMode::Drawn);
        assert_eq!(title_bar_mode(false, true, false), TitleBarMode::System);
        assert_eq!(title_bar_mode(false, false, true), TitleBarMode::System);
        assert_eq!(title_bar_mode(false, true, true), TitleBarMode::System);
    }

    #[test]
    fn double_click_setting() {
        assert_eq!(
            DoubleClickAction::from_setting(None),
            DoubleClickAction::Maximize
        );
        assert_eq!(
            DoubleClickAction::from_setting(Some("Maximize")),
            DoubleClickAction::Maximize
        );
        assert_eq!(
            DoubleClickAction::from_setting(Some("Minimize")),
            DoubleClickAction::Minimize
        );
        assert_eq!(
            DoubleClickAction::from_setting(Some("None")),
            DoubleClickAction::None
        );
    }

    #[test]
    fn native_window_hides_its_title() {
        let mac = initial_viewport(ViewportBuilder::default(), TitleBarMode::MacNative);
        assert_eq!(mac.fullsize_content_view, Some(true));
        assert_eq!(mac.title_shown, Some(false));
        assert_eq!(mac.titlebar_shown, Some(false));
        let drawn = initial_viewport(ViewportBuilder::default(), TitleBarMode::Drawn);
        assert_eq!(drawn.decorations, Some(false));
        let system = initial_viewport(ViewportBuilder::default(), TitleBarMode::System);
        assert_eq!(system, ViewportBuilder::default());
    }
}
