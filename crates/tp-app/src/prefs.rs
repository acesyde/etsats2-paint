//! User preferences: versioned, stored as RON in the config directory, written
//! atomically, and robust to missing or corrupted files.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tp_ui::ThemeSettings;

use crate::layout::WorkspaceLayout;

/// Current preferences schema version.
pub const PREFS_VERSION: u32 = 1;
const FILE_NAME: &str = "prefs.ron";
/// Maximum number of entries kept in the recent projects list.
pub const MAX_RECENT: usize = 12;

/// A previously opened project.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecentProject {
    pub name: String,
    pub path: PathBuf,
    /// Seconds since the Unix epoch.
    pub last_opened: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub version: u32,
    pub ui_scale: f32,
    pub text_scale: f32,
    pub layout: WorkspaceLayout,
    pub recent: Vec<RecentProject>,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            version: PREFS_VERSION,
            ui_scale: 1.0,
            text_scale: 1.0,
            layout: WorkspaceLayout::default(),
            recent: Vec::new(),
        }
    }
}

impl Prefs {
    pub fn theme(&self) -> ThemeSettings {
        ThemeSettings {
            ui_scale: self.ui_scale,
            text_scale: self.text_scale,
        }
        .clamped()
    }

    /// Restores UI scale and text size to 100%.
    pub fn reset_scaling(&mut self) {
        self.ui_scale = 1.0;
        self.text_scale = 1.0;
    }

    /// Repairs values coming from disk.
    fn sanitized(mut self) -> Self {
        let theme = self.theme();
        self.ui_scale = theme.ui_scale;
        self.text_scale = theme.text_scale;
        self.layout = self.layout.sanitized();
        self.recent.truncate(MAX_RECENT);
        self
    }
}

/// Why loading fell back to defaults (for logging).
#[derive(Debug)]
pub enum LoadIssue {
    Unreadable(io::Error),
    Invalid(String),
    UnknownVersion(u32),
}

/// Result of [`PrefsStore::load`].
pub struct Loaded {
    pub prefs: Prefs,
    /// Set when the file existed but could not be used.
    pub issue: Option<LoadIssue>,
    /// Where the unusable file was moved to.
    pub backup: Option<PathBuf>,
}

/// Reads and writes preferences in one directory.
#[derive(Clone, Debug)]
pub struct PrefsStore {
    dir: PathBuf,
}

impl PrefsStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub fn path(&self) -> PathBuf {
        self.dir.join(FILE_NAME)
    }

    /// Loads preferences. Never fails: problems yield defaults, and an unusable
    /// file is preserved as `prefs.ron.bak-<timestamp>`.
    pub fn load(&self) -> Loaded {
        let path = self.path();
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                return Loaded {
                    prefs: Prefs::default(),
                    issue: None,
                    backup: None,
                };
            }
            Err(err) => {
                return Loaded {
                    prefs: Prefs::default(),
                    issue: Some(LoadIssue::Unreadable(err)),
                    backup: None,
                };
            }
        };

        let issue = match ron::from_str::<Prefs>(&text) {
            Ok(prefs) if prefs.version == PREFS_VERSION => {
                return Loaded {
                    prefs: prefs.sanitized(),
                    issue: None,
                    backup: None,
                };
            }
            Ok(prefs) => LoadIssue::UnknownVersion(prefs.version),
            Err(err) => LoadIssue::Invalid(err.to_string()),
        };

        let backup = self.backup_path();
        let backup = fs::rename(&path, &backup).ok().map(|()| backup);
        Loaded {
            prefs: Prefs::default(),
            issue: Some(issue),
            backup,
        }
    }

    /// Saves atomically: write a temporary file next to the target, flush it
    /// to disk, then rename over the previous file.
    pub fn save(&self, prefs: &Prefs) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let text = ron::ser::to_string_pretty(prefs, ron::ser::PrettyConfig::default())
            .map_err(io::Error::other)?;
        let tmp = self.dir.join(format!("{FILE_NAME}.tmp"));
        {
            let mut file = fs::File::create(&tmp)?;
            file.write_all(text.as_bytes())?;
            file.sync_all()?;
        }
        fs::rename(&tmp, self.path())
    }

    fn backup_path(&self) -> PathBuf {
        let stamp = now_unix();
        let mut candidate = self.dir.join(format!("{FILE_NAME}.bak-{stamp}"));
        let mut n = 1;
        while candidate.exists() {
            candidate = self.dir.join(format!("{FILE_NAME}.bak-{stamp}-{n}"));
            n += 1;
        }
        candidate
    }
}

/// Seconds since the Unix epoch (0 if the clock is before 1970).
pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Whether a recent project's file is still present.
pub fn recent_exists(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{PanelKind, ViewMode};

    #[test]
    fn missing_file_gives_defaults_without_issue() {
        let dir = tempfile::tempdir().unwrap();
        let loaded = PrefsStore::new(dir.path()).load();
        assert_eq!(loaded.prefs, Prefs::default());
        assert!(loaded.issue.is_none());
    }

    #[test]
    fn round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let store = PrefsStore::new(dir.path().join("nested"));
        let mut prefs = Prefs {
            ui_scale: 1.25,
            text_scale: 1.1,
            ..Prefs::default()
        };
        prefs.layout.toggle_open(PanelKind::Assets);
        prefs.layout.view_mode = ViewMode::Split;
        prefs.recent.push(RecentProject {
            name: "ACE".into(),
            path: "/tmp/ace.truckpaint".into(),
            last_opened: 42,
        });
        store.save(&prefs).unwrap();
        let loaded = store.load();
        assert!(loaded.issue.is_none());
        assert_eq!(loaded.prefs, prefs);
        assert!(!dir.path().join("nested/prefs.ron.tmp").exists());
    }

    #[test]
    fn corrupted_file_falls_back_and_is_backed_up() {
        let dir = tempfile::tempdir().unwrap();
        let store = PrefsStore::new(dir.path());
        fs::write(store.path(), "this is ( not ron").unwrap();
        let loaded = store.load();
        assert_eq!(loaded.prefs, Prefs::default());
        assert!(matches!(loaded.issue, Some(LoadIssue::Invalid(_))));
        let backup = loaded.backup.expect("backup created");
        assert_eq!(fs::read_to_string(backup).unwrap(), "this is ( not ron");
        assert!(!store.path().exists());
    }

    #[test]
    fn unknown_version_falls_back() {
        let dir = tempfile::tempdir().unwrap();
        let store = PrefsStore::new(dir.path());
        let future = Prefs {
            version: PREFS_VERSION + 1,
            ..Prefs::default()
        };
        store.save(&future).unwrap();
        let loaded = store.load();
        assert!(
            matches!(loaded.issue, Some(LoadIssue::UnknownVersion(v)) if v == PREFS_VERSION + 1)
        );
        assert!(loaded.backup.is_some());
    }

    #[test]
    fn out_of_range_values_are_clamped() {
        let dir = tempfile::tempdir().unwrap();
        let store = PrefsStore::new(dir.path());
        store
            .save(&Prefs {
                ui_scale: 9.0,
                text_scale: 0.0,
                ..Prefs::default()
            })
            .unwrap();
        let prefs = store.load().prefs;
        assert_eq!(prefs.ui_scale, 2.0);
        assert_eq!(prefs.text_scale, 0.85);
    }
}
