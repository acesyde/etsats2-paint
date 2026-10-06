//! TruckPaint design system: tokens, theme, fonts, icons and widgets.
//!
//! Application code should build its UI from these widgets so every screen
//! shares the same look, states and accessibility guarantees.

pub mod contrast;
pub mod fonts;
pub mod icons;
pub mod theme;
pub mod tokens;
pub mod widgets;

pub use theme::ThemeSettings;
