//! Undo/redo history of snapshots.

use std::collections::VecDeque;

/// Default number of steps kept.
pub const DEFAULT_MAX_STEPS: usize = 200;
/// Same-label changes closer than this (seconds) can be merged.
pub const COALESCE_WINDOW: f64 = 1.0;

#[derive(Clone, Debug)]
struct Entry<S> {
    label: &'static str,
    before: S,
    after: S,
    at: f64,
    coalescable: bool,
}

/// Bounded undo/redo stack of `before`/`after` snapshots.
#[derive(Clone, Debug)]
pub struct History<S> {
    undo: VecDeque<Entry<S>>,
    redo: Vec<Entry<S>>,
    max: usize,
}

impl<S: Clone> Default for History<S> {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_STEPS)
    }
}

impl<S: Clone> History<S> {
    pub fn new(max: usize) -> Self {
        Self {
            undo: VecDeque::new(),
            redo: Vec::new(),
            max: max.max(1),
        }
    }

    /// Records a change. With `coalesce`, a change with the same label made
    /// within [`COALESCE_WINDOW`] of the previous one extends it instead of
    /// adding a step. Clears the redo stack.
    pub fn record(&mut self, label: &'static str, before: S, after: S, now: f64, coalesce: bool) {
        self.redo.clear();
        if coalesce
            && let Some(last) = self.undo.back_mut()
            && last.coalescable
            && last.label == label
            && now - last.at < COALESCE_WINDOW
        {
            last.after = after;
            last.at = now;
            return;
        }
        self.undo.push_back(Entry {
            label,
            before,
            after,
            at: now,
            coalescable: coalesce,
        });
        while self.undo.len() > self.max {
            self.undo.pop_front();
        }
    }

    /// Steps back: returns the state to restore.
    pub fn undo(&mut self) -> Option<S> {
        let entry = self.undo.pop_back()?;
        let state = entry.before.clone();
        self.redo.push(entry);
        Some(state)
    }

    /// Steps forward: returns the state to restore.
    pub fn redo(&mut self) -> Option<S> {
        let entry = self.redo.pop()?;
        let state = entry.after.clone();
        self.undo.push_back(entry);
        Some(state)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Label of the step [`Self::undo`] would revert.
    pub fn undo_label(&self) -> Option<&'static str> {
        self.undo.back().map(|e| e.label)
    }

    /// Label of the step [`Self::redo`] would reapply.
    pub fn redo_label(&self) -> Option<&'static str> {
        self.redo.last().map(|e| e.label)
    }

    pub fn len(&self) -> usize {
        self.undo.len()
    }

    pub fn is_empty(&self) -> bool {
        self.undo.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_redo_round_trip() {
        let mut h = History::default();
        h.record("Move", 1, 2, 0.0, false);
        assert_eq!(h.undo_label(), Some("Move"));
        assert_eq!(h.undo(), Some(1));
        assert_eq!(h.redo_label(), Some("Move"));
        assert_eq!(h.redo(), Some(2));
        assert!(!h.can_redo());
    }

    #[test]
    fn new_change_clears_redo() {
        let mut h = History::default();
        h.record("Move", 1, 2, 0.0, false);
        h.undo();
        h.record("Create Ellipse", 1, 3, 1.0, false);
        assert!(!h.can_redo());
    }

    #[test]
    fn nudges_coalesce_within_window() {
        let mut h = History::default();
        h.record("Nudge", 0, 1, 0.0, true);
        h.record("Nudge", 1, 2, 0.5, true);
        h.record("Nudge", 2, 3, 1.2, true);
        assert_eq!(h.len(), 1);
        h.record("Nudge", 3, 4, 3.0, true);
        assert_eq!(h.len(), 2);
        assert_eq!(h.undo(), Some(3));
        assert_eq!(h.undo(), Some(0));
    }

    #[test]
    fn non_coalescable_steps_stay_separate() {
        let mut h = History::default();
        h.record("Move", 0, 1, 0.0, false);
        h.record("Move", 1, 2, 0.1, false);
        assert_eq!(h.len(), 2);
    }

    #[test]
    fn history_is_bounded() {
        let mut h = History::new(200);
        for i in 0..250 {
            h.record("Move", i, i + 1, f64::from(i), false);
        }
        assert_eq!(h.len(), 200);
        let mut last = None;
        while let Some(s) = h.undo() {
            last = Some(s);
        }
        assert_eq!(last, Some(50));
    }
}
