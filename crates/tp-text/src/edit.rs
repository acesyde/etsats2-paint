//! Editing state of one text: content, caret, selection and an in-session
//! undo stack. Positions are byte offsets on grapheme boundaries.

use std::ops::Range;

use tp_core::kurbo::{Point, Rect};
use unicode_segmentation::UnicodeSegmentation;

use crate::layout::TextLayout;

/// Caret movements.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    Left,
    Right,
    WordLeft,
    WordRight,
    Up,
    Down,
    /// Start of the line.
    Home,
    /// End of the line.
    End,
    /// Start of the text.
    Start,
    /// End of the text.
    Finish,
}

/// What the last change was, to group consecutive typing in one undo step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChangeKind {
    Typing,
    Deleting,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct State {
    text: String,
    cursor: usize,
    anchor: usize,
}

/// The editing state of one text.
#[derive(Clone, Debug)]
pub struct EditSession {
    text: String,
    /// Caret position.
    cursor: usize,
    /// Other end of the selection (equal to `cursor` when nothing is
    /// selected).
    anchor: usize,
    /// x kept while moving up/down across lines.
    preferred_x: Option<f64>,
    undo: Vec<State>,
    redo: Vec<State>,
    last_change: Option<ChangeKind>,
}

impl EditSession {
    /// Starts editing `text` with the caret at the end.
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let end = text.len();
        Self {
            text,
            cursor: end,
            anchor: end,
            preferred_x: None,
            undo: Vec::new(),
            redo: Vec::new(),
            last_change: None,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Selected byte range (empty when nothing is selected).
    pub fn selection(&self) -> Range<usize> {
        self.cursor.min(self.anchor)..self.cursor.max(self.anchor)
    }

    pub fn has_selection(&self) -> bool {
        self.cursor != self.anchor
    }

    pub fn selected_text(&self) -> &str {
        &self.text[self.selection()]
    }

    fn state(&self) -> State {
        State {
            text: self.text.clone(),
            cursor: self.cursor,
            anchor: self.anchor,
        }
    }

    fn restore(&mut self, state: State) {
        self.text = state.text;
        self.cursor = state.cursor;
        self.anchor = state.anchor;
        self.preferred_x = None;
        self.last_change = None;
    }

    fn record(&mut self, kind: ChangeKind) {
        if kind == ChangeKind::Other || self.last_change != Some(kind) {
            self.undo.push(self.state());
        }
        self.redo.clear();
        self.last_change = Some(kind);
        self.preferred_x = None;
    }

    /// Replaces the selection with `text` (typing, paste, new line).
    pub fn insert(&mut self, text: &str) {
        if text.is_empty() && !self.has_selection() {
            return;
        }
        // Normalize line endings from pasted text.
        let text = text.replace("\r\n", "\n").replace('\r', "\n");
        let kind = if text.chars().count() == 1 && text != "\n" && !self.has_selection() {
            ChangeKind::Typing
        } else {
            ChangeKind::Other
        };
        self.record(kind);
        let range = self.selection();
        self.text.replace_range(range.clone(), &text);
        self.cursor = range.start + text.len();
        self.anchor = self.cursor;
    }

    fn delete_range(&mut self, range: Range<usize>) {
        if range.is_empty() {
            return;
        }
        self.record(ChangeKind::Deleting);
        self.text.replace_range(range.clone(), "");
        self.cursor = range.start;
        self.anchor = range.start;
    }

    /// Backspace: deletes the selection or the grapheme before the caret.
    pub fn backspace(&mut self) {
        let range = if self.has_selection() {
            self.selection()
        } else {
            self.prev_grapheme(self.cursor)..self.cursor
        };
        self.delete_range(range);
    }

    /// Delete: deletes the selection or the grapheme after the caret.
    pub fn delete(&mut self) {
        let range = if self.has_selection() {
            self.selection()
        } else {
            self.cursor..self.next_grapheme(self.cursor)
        };
        self.delete_range(range);
    }

    /// Removes and returns the selected text (Cut).
    pub fn cut(&mut self) -> String {
        let selected = self.selected_text().to_owned();
        if !selected.is_empty() {
            self.record(ChangeKind::Other);
            let range = self.selection();
            self.text.replace_range(range.clone(), "");
            self.cursor = range.start;
            self.anchor = range.start;
        }
        selected
    }

    pub fn select_all(&mut self) {
        self.anchor = 0;
        self.cursor = self.text.len();
        self.end_change_group();
    }

    /// Selects the word around `offset`.
    pub fn select_word(&mut self, offset: usize) {
        let offset = offset.min(self.text.len());
        let mut found = None;
        for (start, word) in self.text.split_word_bound_indices() {
            let end = start + word.len();
            if offset >= start && offset < end || offset == end && found.is_none() {
                found = Some(start..end);
                if offset < end && word.chars().any(char::is_alphanumeric) {
                    break;
                }
            }
        }
        let range = found.unwrap_or(offset..offset);
        self.anchor = range.start;
        self.cursor = range.end;
        self.end_change_group();
    }

    /// Places the caret at `offset`, extending the selection when `extend`.
    pub fn set_cursor(&mut self, offset: usize, extend: bool) {
        self.cursor = self.snap(offset);
        if !extend {
            self.anchor = self.cursor;
        }
        self.preferred_x = None;
        self.end_change_group();
    }

    /// Places the caret at a point in layout coordinates.
    pub fn click(&mut self, layout: &TextLayout, point: Point, extend: bool) {
        self.set_cursor(layout.hit(point), extend);
    }

    /// Moves the caret; with `extend` the selection grows, otherwise an
    /// existing selection collapses to its matching edge.
    pub fn move_cursor(&mut self, motion: Motion, extend: bool, layout: &TextLayout) {
        let selection = self.selection();
        let keep_x = matches!(motion, Motion::Up | Motion::Down);
        let target = match motion {
            Motion::Left if !extend && self.has_selection() => selection.start,
            Motion::Right if !extend && self.has_selection() => selection.end,
            Motion::Left => self.prev_grapheme(self.cursor),
            Motion::Right => self.next_grapheme(self.cursor),
            Motion::WordLeft => self.prev_word(self.cursor),
            Motion::WordRight => self.next_word(self.cursor),
            Motion::Up | Motion::Down => {
                let line = layout.line_of(self.cursor);
                let x = self
                    .preferred_x
                    .unwrap_or_else(|| layout.caret(self.cursor).x0);
                self.preferred_x = Some(x);
                let target_line = if motion == Motion::Up {
                    line.checked_sub(1)
                } else {
                    Some(line + 1).filter(|l| *l < layout.lines.len())
                };
                match target_line {
                    Some(l) => {
                        let row = &layout.lines[l];
                        layout.hit(Point::new(x, row.top + row.height / 2.0))
                    }
                    None if motion == Motion::Up => 0,
                    None => self.text.len(),
                }
            }
            Motion::Home => self.line_start(self.cursor),
            Motion::End => self.line_end(self.cursor),
            Motion::Start => 0,
            Motion::Finish => self.text.len(),
        };
        self.cursor = self.snap(target);
        if !extend {
            self.anchor = self.cursor;
        }
        if !keep_x {
            self.preferred_x = None;
        }
        self.end_change_group();
    }

    /// Starts a new undo group at the next change (caret moved).
    fn end_change_group(&mut self) {
        self.last_change = None;
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Undoes the last change of this session. Returns false when there is
    /// nothing left to undo in the session.
    pub fn undo(&mut self) -> bool {
        let Some(state) = self.undo.pop() else {
            return false;
        };
        self.redo.push(self.state());
        self.restore(state);
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(state) = self.redo.pop() else {
            return false;
        };
        self.undo.push(self.state());
        self.restore(state);
        true
    }

    /// Caret rectangle in layout coordinates.
    pub fn caret_rect(&self, layout: &TextLayout) -> Rect {
        layout.caret(self.cursor)
    }

    /// Selection rectangles in layout coordinates.
    pub fn selection_rects(&self, layout: &TextLayout) -> Vec<Rect> {
        let r = self.selection();
        layout.selection_rects(r.start, r.end)
    }

    fn snap(&self, offset: usize) -> usize {
        let offset = offset.min(self.text.len());
        if self.is_boundary(offset) {
            offset
        } else {
            self.prev_grapheme(offset)
        }
    }

    fn is_boundary(&self, offset: usize) -> bool {
        offset == self.text.len() || self.text.grapheme_indices(true).any(|(i, _)| i == offset)
    }

    fn prev_grapheme(&self, offset: usize) -> usize {
        self.text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .take_while(|i| *i < offset)
            .last()
            .unwrap_or(0)
    }

    fn next_grapheme(&self, offset: usize) -> usize {
        self.text
            .grapheme_indices(true)
            .map(|(i, g)| i + g.len())
            .find(|end| *end > offset)
            .unwrap_or(self.text.len())
    }

    fn is_word(s: &str) -> bool {
        s.chars().any(char::is_alphanumeric)
    }

    fn prev_word(&self, offset: usize) -> usize {
        self.text
            .split_word_bound_indices()
            .rev()
            .find(|(i, w)| *i < offset && Self::is_word(w))
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    fn next_word(&self, offset: usize) -> usize {
        self.text
            .split_word_bound_indices()
            .map(|(i, w)| (i + w.len(), w))
            .find(|(end, w)| *end > offset && Self::is_word(w))
            .map_or(self.text.len(), |(end, _)| end)
    }

    fn line_start(&self, offset: usize) -> usize {
        self.text[..offset].rfind('\n').map_or(0, |i| i + 1)
    }

    fn line_end(&self, offset: usize) -> usize {
        self.text[offset..]
            .find('\n')
            .map_or(self.text.len(), |i| offset + i)
    }
}

#[cfg(test)]
mod tests {
    use tp_core::document::CharStyle;

    use super::*;
    use crate::fonts::FontLibrary;
    use crate::layout::layout;

    fn lay(s: &EditSession) -> TextLayout {
        layout(&mut FontLibrary::bundled(), s.text(), &CharStyle::default())
    }

    fn typed(s: &mut EditSession, text: &str) {
        for c in text.chars() {
            s.insert(&c.to_string());
        }
    }

    #[test]
    fn insert_and_delete() {
        let mut s = EditSession::new("");
        typed(&mut s, "ACE");
        assert_eq!((s.text(), s.cursor()), ("ACE", 3));
        s.backspace();
        assert_eq!(s.text(), "AC");
        s.set_cursor(0, false);
        s.delete();
        assert_eq!((s.text(), s.cursor()), ("C", 0));
    }

    #[test]
    fn accented_and_combined_characters_are_one_step() {
        let mut s = EditSession::new("cafe\u{301}é");
        s.backspace();
        assert_eq!(s.text(), "cafe\u{301}");
        s.backspace();
        assert_eq!(s.text(), "caf");
        let l = lay(&s);
        s.move_cursor(Motion::Left, false, &l);
        assert_eq!(s.cursor(), 2);
    }

    #[test]
    fn select_all_and_replace() {
        let mut s = EditSession::new("Old name");
        s.select_all();
        assert_eq!(s.selected_text(), "Old name");
        typed(&mut s, "New name");
        assert_eq!(s.text(), "New name");
    }

    #[test]
    fn new_line_and_vertical_motion() {
        let mut s = EditSession::new("");
        typed(&mut s, "Line 1");
        s.insert("\n");
        typed(&mut s, "Line 2");
        assert_eq!(s.text(), "Line 1\nLine 2");
        let l = lay(&s);
        assert_eq!(l.lines.len(), 2);
        s.move_cursor(Motion::Up, false, &l);
        assert_eq!(s.cursor(), 6);
        s.move_cursor(Motion::Down, false, &l);
        assert_eq!(s.cursor(), 13);
        s.move_cursor(Motion::Home, false, &l);
        assert_eq!(s.cursor(), 7);
        s.move_cursor(Motion::End, true, &l);
        assert_eq!(s.selected_text(), "Line 2");
    }

    #[test]
    fn shift_motions_extend_and_plain_motions_collapse() {
        let mut s = EditSession::new("ABCD");
        let l = lay(&s);
        s.move_cursor(Motion::Left, true, &l);
        s.move_cursor(Motion::Left, true, &l);
        assert_eq!(s.selected_text(), "CD");
        s.move_cursor(Motion::Left, false, &l);
        assert_eq!((s.cursor(), s.has_selection()), (2, false));
        s.move_cursor(Motion::Start, true, &l);
        assert_eq!(s.selected_text(), "AB");
    }

    #[test]
    fn word_selection_and_motion() {
        let mut s = EditSession::new("ACE Logistics 24");
        s.select_word(6);
        assert_eq!(s.selected_text(), "Logistics");
        let l = lay(&s);
        s.set_cursor(0, false);
        s.move_cursor(Motion::WordRight, false, &l);
        assert_eq!(s.cursor(), 3);
        s.move_cursor(Motion::WordRight, false, &l);
        assert_eq!(s.cursor(), 13);
        s.move_cursor(Motion::WordLeft, false, &l);
        assert_eq!(s.cursor(), 4);
    }

    #[test]
    fn cut_and_paste() {
        let mut s = EditSession::new("ACE Logistics");
        s.select_word(0);
        assert_eq!(s.cut(), "ACE");
        assert_eq!(s.text(), " Logistics");
        s.move_cursor(Motion::Finish, false, &lay(&s));
        s.insert(" ACE\r\n2");
        assert_eq!(s.text(), " Logistics ACE\n2");
    }

    #[test]
    fn undo_groups_typing() {
        let mut s = EditSession::new("ACE");
        typed(&mut s, " Logistics");
        s.backspace();
        s.backspace();
        assert_eq!(s.text(), "ACE Logisti");
        assert!(s.undo());
        assert_eq!(s.text(), "ACE Logistics");
        assert!(s.undo());
        assert_eq!(s.text(), "ACE");
        assert!(!s.undo());
        assert!(s.redo());
        assert_eq!(s.text(), "ACE Logistics");
    }

    #[test]
    fn caret_and_selection_rects_follow_layout() {
        let mut s = EditSession::new("AB");
        let l = lay(&s);
        assert!(s.caret_rect(&l).x0 > 0.0);
        s.click(&l, Point::new(0.0, 10.0), false);
        assert_eq!(s.cursor(), 0);
        s.select_all();
        assert_eq!(s.selection_rects(&l).len(), 1);
    }
}
