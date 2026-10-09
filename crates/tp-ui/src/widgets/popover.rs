use egui::{
    Context, Frame, Id, InnerResponse, Key, Margin, Popup, PopupCloseBehavior, Rect, RectAlign, Ui,
    UiKind, UiStackInfo,
};

use crate::tokens::{color, radius, space, stroke};

/// Where the open popover is remembered: one at a time for the whole
/// window.
fn state_id() -> Id {
    Id::new("tp_ui_open_popover")
}

/// The open popover and the pass it was last shown in.
#[derive(Clone, Copy, Debug)]
struct OpenPopover {
    id: Id,
    shown_pass: u64,
    /// A text field had the keyboard focus at the end of the last pass
    /// (egui drops the focus on Escape before the field sees the key).
    typing: bool,
}

/// Positions tried in turn after the first one: on the left of the anchor
/// (top aligned, then bottom aligned, then centered), then under it and
/// above it when there is no room on the left.
const ALIGNS: [RectAlign; 6] = [
    RectAlign::LEFT_END,
    RectAlign::LEFT,
    RectAlign::BOTTOM_END,
    RectAlign::BOTTOM_START,
    RectAlign::TOP_END,
    RectAlign::TOP_START,
];

/// A small floating panel opened from a row (the inspector's Fill and
/// Stroke rows), holding settings that don't stay open on their own. It
/// opens on the left of its anchor (beside the inspector, over the canvas,
/// when the anchor spans the inspector's width), else under or above it.
///
/// Only one popover is open at a time: opening one closes the other.
/// Escape, a press outside the popover and its anchor, or not drawing it
/// for a frame (its row went away) close it. A press in a menu opened from
/// inside the popover (a context menu, a combo box) does not close it, and
/// neither does Escape while one of its text fields has the focus (the
/// field takes it first).
pub struct Popover {
    id: Id,
    anchor: Rect,
    width: f32,
}

impl Popover {
    /// The id of the popover named `name`, for [`Popover::open`] and
    /// friends.
    pub fn id(name: &str) -> Id {
        Id::new(("tp_ui_popover", name))
    }

    /// The popover `id`, anchored to `anchor` (screen coordinates, usually
    /// the rectangle of its row).
    pub fn new(id: Id, anchor: Rect) -> Self {
        Self {
            id,
            anchor,
            width: 252.0,
        }
    }

    /// Width of the popover's content area and frame.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    fn state(ctx: &Context) -> Option<OpenPopover> {
        ctx.data(|d| d.get_temp::<OpenPopover>(state_id()))
    }

    /// The popover open now, if any.
    pub fn open_id(ctx: &Context) -> Option<Id> {
        let state = Self::state(ctx)?;
        // Not drawn for a whole frame: its row went away.
        (ctx.cumulative_pass_nr() <= state.shown_pass + 1).then_some(state.id)
    }

    pub fn is_open(ctx: &Context, id: Id) -> bool {
        Self::open_id(ctx) == Some(id)
    }

    /// Opens `id`, closing any other popover.
    pub fn open(ctx: &Context, id: Id) {
        let state = OpenPopover {
            id,
            shown_pass: ctx.cumulative_pass_nr(),
            typing: false,
        };
        ctx.data_mut(|d| d.insert_temp(state_id(), state));
    }

    /// Opens `id`, or closes it when it is open.
    pub fn toggle(ctx: &Context, id: Id) {
        if Self::is_open(ctx, id) {
            Self::close(ctx);
        } else {
            Self::open(ctx, id);
        }
    }

    /// Closes the open popover, if any.
    pub fn close(ctx: &Context) {
        ctx.data_mut(|d| d.remove::<OpenPopover>(state_id()));
    }

    /// Closes the open popover when Escape was pressed, and takes the key
    /// so it does nothing else (such as deselecting). For the application's
    /// shortcut handling, which runs before the popover is drawn; call it
    /// only when no text field has the keyboard focus.
    pub fn close_on_escape(ctx: &Context) -> bool {
        if Self::open_id(ctx).is_none() {
            return false;
        }
        let pressed = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape));
        if pressed {
            Self::close(ctx);
        }
        pressed
    }

    /// Draws the popover when it is open; `None` when it is closed.
    pub fn show<R>(self, ui: &Ui, content: impl FnOnce(&mut Ui) -> R) -> Option<InnerResponse<R>> {
        let ctx = ui.ctx();
        let mut state = Self::state(ctx).filter(|s| s.id == self.id)?;
        if Self::open_id(ctx) != Some(self.id) {
            Self::close(ctx);
            return None;
        }
        // Read before the content runs: what is open when the frame starts
        // receives this frame's presses and keys.
        let opening = state.shown_pass == ctx.cumulative_pass_nr();
        let menu_open = Popup::is_any_open(ctx);
        let typing = state.typing || ctx.text_edit_focused();

        let frame = Frame::new()
            .fill(color::CONTROL)
            .stroke(egui::Stroke::new(stroke::HAIRLINE, color::BORDER_STRONG))
            .corner_radius(radius::LG + 4)
            .inner_margin(Margin::same(space::MD as i8))
            .shadow(ui.visuals().popup_shadow);
        let inner_width = self.width - 2.0 * space::MD;
        let response = Popup::new(self.id, ctx.clone(), self.anchor, ui.layer_id())
            .open(true)
            .close_behavior(PopupCloseBehavior::IgnoreClicks)
            // Not a menu: buttons and menus inside behave as anywhere else.
            .info(UiStackInfo::new(UiKind::GenericArea))
            .align(RectAlign::LEFT_START)
            .align_alternatives(&ALIGNS)
            .gap(space::MD)
            .width(self.width)
            .frame(frame)
            .show(|ui| {
                ui.set_min_width(inner_width);
                ui.spacing_mut().item_spacing.y = space::SM;
                content(ui)
            })?;

        let rect = response.response.rect;
        let pressed_outside = ctx.input(|i| {
            i.pointer.any_pressed()
                && i.pointer
                    .press_origin()
                    .is_some_and(|p| !rect.contains(p) && !self.anchor.contains(p))
        });
        let escape = !typing && ctx.input(|i| i.key_pressed(Key::Escape));
        if Self::state(ctx).is_none_or(|s| s.id != self.id) {
            // Its content opened another popover (or closed it).
        } else if escape || (pressed_outside && !opening && !menu_open) {
            Self::close(ctx);
        } else {
            state.shown_pass = ctx.cumulative_pass_nr();
            state.typing = ctx.text_edit_focused();
            ctx.data_mut(|d| d.insert_temp(state_id(), state));
        }
        Some(response)
    }
}
