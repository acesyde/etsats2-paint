//! Crash-recovery copies: each running session writes
//! `<session>.truckpaint` and `<session>.ron` (metadata) in the recovery
//! folder and holds an OS lock on `<session>.lock` while it runs. Copies of
//! sessions whose lock is free (process gone, e.g. after a crash) are
//! offered at the next launch.

use std::fs::File;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Seconds between two recovery copies.
pub const INTERVAL: f64 = 120.0;

/// What is known about a recovery copy without reading it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecoveryMeta {
    pub name: String,
    /// The project's own file, if it was ever saved.
    pub original: Option<PathBuf>,
    /// Seconds since the Unix epoch.
    pub saved_at: u64,
}

/// A copy left by a previous session.
#[derive(Clone, Debug, PartialEq)]
pub struct Recovered {
    pub session: String,
    /// `None` when the metadata or the copy cannot be read.
    pub meta: Option<RecoveryMeta>,
}

/// This session's recovery files.
pub struct RecoveryStore {
    dir: PathBuf,
    session: String,
    /// Held for the whole session.
    _lock: Option<File>,
}

fn new_session_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}-{:x}", nanos, std::process::id())
}

impl RecoveryStore {
    /// Opens the recovery folder for a new session and locks it.
    pub fn open(dir: &Path) -> std::io::Result<Self> {
        std::fs::create_dir_all(dir)?;
        let session = new_session_id();
        let lock = File::create(dir.join(format!("{session}.lock")))?;
        let locked = lock.try_lock().is_ok();
        if !locked {
            tracing::warn!("could not lock the recovery session");
        }
        Ok(Self {
            dir: dir.to_path_buf(),
            session,
            _lock: Some(lock),
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn session(&self) -> &str {
        &self.session
    }

    fn file(&self, session: &str, ext: &str) -> PathBuf {
        self.dir.join(format!("{session}.{ext}"))
    }

    /// Where this session's copy goes.
    pub fn copy_path(&self) -> PathBuf {
        self.file(&self.session, tp_file::EXTENSION)
    }

    /// Where this session's metadata goes, and its content.
    pub fn meta_file(&self, meta: &RecoveryMeta) -> (PathBuf, String) {
        let text =
            ron::ser::to_string_pretty(meta, ron::ser::PrettyConfig::default()).unwrap_or_default();
        (self.file(&self.session, "ron"), text)
    }

    /// Deletes this session's copy (project saved or closed).
    pub fn clear(&self) {
        self.discard_files(&self.session);
    }

    fn discard_files(&self, session: &str) {
        for ext in [tp_file::EXTENSION, "ron"] {
            let _ = std::fs::remove_file(self.file(session, ext));
        }
    }

    /// Copies left by sessions that are no longer running.
    pub fn scan(&self) -> Vec<Recovered> {
        let Ok(entries) = std::fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut sessions: Vec<String> = entries
            .filter_map(Result::ok)
            .filter_map(|e| {
                let path = e.path();
                let ext = path.extension()?.to_str()?;
                (ext == "ron" || ext == tp_file::EXTENSION)
                    .then(|| path.file_stem()?.to_str().map(str::to_owned))?
            })
            .filter(|s| *s != self.session)
            .collect();
        sessions.sort();
        sessions.dedup();
        let mut found: Vec<Recovered> = sessions
            .into_iter()
            .filter(|s| !self.is_running(s))
            .map(|session| {
                let meta = std::fs::read_to_string(self.file(&session, "ron"))
                    .ok()
                    .and_then(|t| ron::from_str::<RecoveryMeta>(&t).ok())
                    .filter(|_| self.file(&session, tp_file::EXTENSION).is_file());
                Recovered { session, meta }
            })
            .collect();
        found.sort_by_key(|r| std::cmp::Reverse(r.meta.as_ref().map_or(0, |m| m.saved_at)));
        found
    }

    /// Whether another running process holds that session's lock.
    fn is_running(&self, session: &str) -> bool {
        let path = self.file(session, "lock");
        let Ok(file) = File::options().read(true).write(true).open(&path) else {
            return false;
        };
        match file.try_lock() {
            Ok(()) => {
                // Owner gone: the lock file is stale.
                drop(file);
                let _ = std::fs::remove_file(path);
                false
            }
            Err(_) => true,
        }
    }

    /// Reads a recovered copy.
    pub fn read(&self, session: &str) -> Result<tp_file::Opened, tp_file::Error> {
        tp_file::read(&self.file(session, tp_file::EXTENSION))
    }

    /// Makes a restored copy this session's own (kept until saved/closed).
    pub fn adopt(&self, session: &str) {
        for ext in [tp_file::EXTENSION, "ron"] {
            let _ = std::fs::rename(self.file(session, ext), self.file(&self.session, ext));
        }
    }

    pub fn discard(&self, session: &str) {
        self.discard_files(session);
        let _ = std::fs::remove_file(self.file(session, "lock"));
    }
}

impl Drop for RecoveryStore {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.file(&self.session, "lock"));
    }
}

#[cfg(test)]
mod tests {
    use tp_core::{Project, TextureResolution};

    use super::*;

    fn write_copy(store: &RecoveryStore, name: &str) {
        tp_file::write(
            &Project::new(name, TextureResolution::R2048),
            &store.copy_path(),
        )
        .unwrap();
        let (path, text) = store.meta_file(&RecoveryMeta {
            name: name.into(),
            original: None,
            saved_at: 1,
        });
        std::fs::write(path, text).unwrap();
    }

    #[test]
    fn running_sessions_are_skipped_and_finished_ones_offered() {
        let dir = tempfile::tempdir().unwrap();
        let first = RecoveryStore::open(dir.path()).unwrap();
        write_copy(&first, "ACE");
        let second = RecoveryStore::open(dir.path()).unwrap();
        assert!(second.scan().is_empty(), "first session still running");
        let session = first.session().to_owned();
        drop(first); // ends the session without clearing its copy (crash-like)
        let found = second.scan();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].session, session);
        assert_eq!(found[0].meta.as_ref().unwrap().name, "ACE");
        assert_eq!(second.read(&session).unwrap().project.name, "ACE");
        second.adopt(&session);
        assert!(second.scan().is_empty());
        assert!(second.copy_path().is_file());
        second.clear();
        assert!(!second.copy_path().is_file());
    }

    #[test]
    fn unreadable_copies_are_listed_as_damaged_and_discardable() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("old.ron"), "not ron").unwrap();
        let store = RecoveryStore::open(dir.path()).unwrap();
        let found = store.scan();
        assert_eq!(
            found,
            vec![Recovered {
                session: "old".into(),
                meta: None
            }]
        );
        store.discard("old");
        assert!(store.scan().is_empty());
    }
}
