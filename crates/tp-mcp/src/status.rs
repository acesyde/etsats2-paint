//! What the server is doing, shared with the app (Preferences, status bar).

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// How long a client counts as connected after a call, and the window of
/// the write count.
pub const RECENT: Duration = Duration::from_secs(60);

/// Whether the server runs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ServerState {
    #[default]
    Off,
    Listening,
    /// The port is taken by another program.
    PortInUse,
    /// It could not start for another reason.
    Failed(String),
}

/// The server's state and its recent calls.
#[derive(Clone, Debug, Default)]
pub struct Status {
    pub state: ServerState,
    /// The port it listens on (or tried to).
    pub port: u16,
    last_call: Option<Instant>,
    writes: VecDeque<Instant>,
}

impl Status {
    /// Notes a call (and, for a change applied, a write).
    pub fn note_call(&mut self, applied_write: bool, now: Instant) {
        self.last_call = Some(now);
        if applied_write {
            self.writes.push_back(now);
        }
        self.forget_old(now);
    }

    fn forget_old(&mut self, now: Instant) {
        while self
            .writes
            .front()
            .is_some_and(|t| now.duration_since(*t) > RECENT)
        {
            self.writes.pop_front();
        }
    }

    /// Whether a client called in the last minute.
    pub fn connected(&self, now: Instant) -> bool {
        self.last_call
            .is_some_and(|t| now.duration_since(t) <= RECENT)
    }

    /// Changes applied in the last minute.
    pub fn recent_writes(&self, now: Instant) -> usize {
        self.writes
            .iter()
            .filter(|t| now.duration_since(**t) <= RECENT)
            .count()
    }

    /// When the counts change next by themselves (to repaint then).
    pub fn next_change(&self, now: Instant) -> Option<Duration> {
        let next = self
            .writes
            .front()
            .copied()
            .into_iter()
            .chain(self.last_call);
        next.map(|t| (t + RECENT).saturating_duration_since(now))
            .filter(|d| !d.is_zero())
            .min()
    }

    /// The server stopped: back to off, history cleared.
    pub fn reset(&mut self) {
        *self = Self {
            port: self.port,
            ..Self::default()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_the_last_minute() {
        let t0 = Instant::now();
        let mut s = Status::default();
        assert!(!s.connected(t0));
        s.note_call(false, t0);
        assert!(s.connected(t0));
        assert_eq!(s.recent_writes(t0), 0);
        s.note_call(true, t0);
        s.note_call(true, t0 + Duration::from_secs(30));
        assert_eq!(s.recent_writes(t0 + Duration::from_secs(40)), 2);
        assert_eq!(s.recent_writes(t0 + Duration::from_secs(70)), 1);
        assert!(!s.connected(t0 + Duration::from_secs(100)));
    }
}
