//! The personal library: symbols, swatches and styles shared by every
//! project, stored in the application's data folder.
//!
//! It loads the first time it is used and is written after each change, on
//! the background saver. A file that can't be read is set aside as
//! `library.tplib.bak-<timestamp>`; the library then starts empty and the
//! problem is reported once.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use tp_core::document::{Object, StyleId, SwatchId, SymbolId};
use tp_core::import::{ImportMode, Picks, import};
use tp_core::{LibraryKey, Project};
use tp_i18n::tr;

use crate::workspace::Workspace;

/// A new library key: 32 hex characters from the clock, the process and a
/// counter (tp-core has no clock).
pub fn new_key() -> LibraryKey {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let mut hasher = blake3::Hasher::new();
    hasher.update(&nanos.to_le_bytes());
    hasher.update(&std::process::id().to_le_bytes());
    hasher.update(&COUNTER.fetch_add(1, Ordering::Relaxed).to_le_bytes());
    let hash = hasher.finalize();
    let hex: String = hash.as_bytes()[..16]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    LibraryKey(hex)
}

/// The library could not be read.
#[derive(Clone, Debug, PartialEq)]
pub struct LibraryIssue {
    pub error: String,
    /// Where the unreadable file was moved.
    pub backup: Option<PathBuf>,
}

/// The library, loaded on first use.
#[derive(Debug)]
pub struct LibraryStore {
    /// Its file (`None`: kept in memory only, as in tests).
    path: Option<PathBuf>,
    library: Option<Project>,
    /// Not yet reported.
    issue: Option<LibraryIssue>,
    /// Changed since it was last queued for writing.
    dirty: bool,
}

impl Default for LibraryStore {
    fn default() -> Self {
        Self::new(None)
    }
}

impl LibraryStore {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self {
            path,
            library: None,
            issue: None,
            dirty: false,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// The library, read from its file the first time.
    pub fn library(&mut self) -> &Project {
        self.loaded()
    }

    fn loaded(&mut self) -> &mut Project {
        if self.library.is_none() {
            let (library, issue) = self.load();
            self.library = Some(library);
            self.issue = issue;
        }
        self.library.as_mut().expect("just loaded")
    }

    fn load(&self) -> (Project, Option<LibraryIssue>) {
        let Some(path) = &self.path else {
            return (tp_file::library::empty(), None);
        };
        match tp_file::library::read(path) {
            Ok(library) => (library, None),
            Err(tp_file::Error::Io(err)) if err.kind() == std::io::ErrorKind::NotFound => {
                (tp_file::library::empty(), None)
            }
            Err(err) => {
                let backup = backup_path(path);
                let backup = std::fs::rename(path, &backup).ok().map(|()| backup);
                tracing::warn!(%err, ?backup, "the library could not be read");
                let issue = LibraryIssue {
                    error: err.to_string(),
                    backup,
                };
                (tp_file::library::empty(), Some(issue))
            }
        }
    }

    /// The reading problem of this session, once it is loaded; given once.
    pub fn take_issue(&mut self) -> Option<LibraryIssue> {
        self.issue.take()
    }

    /// Whether the library has entry `key`.
    pub fn contains(&mut self, key: &LibraryKey) -> bool {
        self.loaded().has_origin(key)
    }

    /// Adds `picks` of `project` to the library with everything they use,
    /// updating the entries they are linked to. Returns the keys to record
    /// in the project.
    pub fn publish(&mut self, project: &Project, picks: &Picks) -> Vec<(u64, LibraryKey)> {
        let mut key = new_key;
        let done = import(project, picks, self.loaded(), ImportMode::Publish(&mut key));
        self.dirty = true;
        done.origins
    }

    /// Deletes entry `key`; returns whether it existed. What it used stays
    /// until nothing uses it.
    pub fn remove(&mut self, key: &LibraryKey) -> bool {
        let library = self.loaded();
        let is = |o: &Option<LibraryKey>| o.as_ref() == Some(key);
        let swatch = library.palette.iter().find(|s| is(&s.origin)).map(|s| s.id);
        let graphic = library.graphic_styles.iter().map(|s| (s.id, &s.origin));
        let text = library.text_styles.iter().map(|s| (s.id, &s.origin));
        let style = graphic.chain(text).find(|(_, o)| is(o)).map(|(id, _)| id);
        let symbol = library.symbols.iter().find(|s| is(&s.origin)).map(|s| s.id);
        match (swatch, style, symbol) {
            (Some(id), ..) => library.delete_swatch(id),
            (_, Some(id), _) => library.delete_style(id),
            (.., Some(id)) => library.delete_symbol(id),
            _ => return false,
        }
        self.dirty = true;
        true
    }

    /// The library and its file, when a change is waiting to be written.
    pub fn take_write(&mut self) -> Option<(Project, PathBuf)> {
        if !self.dirty {
            return None;
        }
        self.dirty = false;
        let path = self.path.clone()?;
        Some((self.library.clone()?, path))
    }
}

/// One element of a project, as its panels' context menus name it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Element {
    Symbol(SymbolId),
    Swatch(SwatchId),
    /// A graphic or a text style.
    Style(StyleId),
}

impl Element {
    /// The picks bringing this element.
    pub fn picks(self, project: &Project) -> Picks {
        let mut picks = Picks::default();
        match self {
            Self::Symbol(id) => picks.symbols.push(id),
            Self::Swatch(id) => picks.swatches.push(id),
            Self::Style(id) if project.graphic_style(id).is_some() => {
                picks.graphic_styles.push(id);
            }
            Self::Style(id) => picks.text_styles.push(id),
        }
        picks
    }

    /// The library entry it is linked to, if any.
    pub fn origin(self, project: &Project) -> Option<&LibraryKey> {
        match self {
            Self::Symbol(id) => project.symbol(id)?.origin.as_ref(),
            Self::Swatch(id) => project.swatch(id)?.origin.as_ref(),
            Self::Style(id) => match project.graphic_style(id) {
                Some(s) => s.origin.as_ref(),
                None => project.text_style(id)?.origin.as_ref(),
            },
        }
    }
}

impl LibraryStore {
    /// Whether `element` of `project` is linked to an entry of the library
    /// (its menu then offers Update in Library).
    pub fn has_element(&mut self, project: &Project, element: Element) -> bool {
        element
            .origin(project)
            .cloned()
            .is_some_and(|key| self.contains(&key))
    }

    /// "Add to Library" or "Update in Library" for `element`.
    pub fn menu_label(&mut self, project: &Project, element: Element) -> String {
        tr(if self.has_element(project, element) {
            "library-update"
        } else {
            "library-add"
        })
    }
}

impl Workspace {
    /// Import from Library: copies `picks` of `library` into the project
    /// with everything they use, as one undo step. Returns how many
    /// elements were added.
    pub fn import_from_library(&mut self, library: &Project, picks: &Picks, now: f64) -> usize {
        if self.is_editing_symbol() || picks.is_empty() {
            return 0;
        }
        self.commit_pending(now);
        let mut added = 0;
        // Imported texts are laid out with this computer's fonts.
        self.text_style_edit("undo-import-from-library", now, |project, _| {
            added = import(library, picks, project, ImportMode::Import).added;
        });
        added
    }

    /// Pastes `objects` copied in another project, `source`, offset by
    /// `offset`: their images, swatches, styles and symbols are imported
    /// too, all in the Paste undo step.
    pub fn paste_from(&mut self, source: &Project, objects: &[Object], offset: f64, now: f64) {
        let layer = self.active_layer();
        let picks = Picks {
            objects: objects.to_vec(),
            ..Picks::default()
        };
        self.text_style_edit("cmd-paste", now, |project, selection| {
            let done = import(source, &picks, project, ImportMode::Import);
            let offset = tp_core::kurbo::Vec2::new(offset, offset);
            *selection = project.add_copies(&done.objects, offset, layer);
        });
    }

    /// Add to Library (or Update in Library): copies `element` and what it
    /// uses into the library, and links them to their entries. The links
    /// are saved with the project but are not part of the undo history.
    pub fn add_to_library(&mut self, library: &mut LibraryStore, element: Element, now: f64) {
        self.commit_pending(now);
        let update = library.has_element(&self.project, element);
        let picks = element.picks(&self.project);
        let origins = library.publish(&self.project, &picks);
        self.project.record_origins(&origins);
        self.show_hint(
            tr(if update {
                "library-updated"
            } else {
                "library-added"
            }),
            now,
        );
    }
}

/// `<file>.bak-<timestamp>`, numbered when taken.
fn backup_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let stamp = crate::prefs::now_unix();
    let mut candidate = path.with_file_name(format!("{name}.bak-{stamp}"));
    let mut n = 1;
    while candidate.exists() {
        candidate = path.with_file_name(format!("{name}.bak-{stamp}-{n}"));
        n += 1;
    }
    candidate
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use tp_core::document::Rgba;

    use super::*;

    fn store_in(dir: &Path) -> LibraryStore {
        LibraryStore::new(Some(dir.join(tp_file::library::FILE_NAME)))
    }

    /// A project with the swatch "Vert Ardent".
    fn project() -> (Project, tp_core::document::SwatchId) {
        let mut p = crate::vehicle_project::test_project("A");
        let (vert, _) = p.add_swatch(Rgba::rgb(0x1E, 0x8C, 0x3A), "Color");
        p.rename_swatch(vert, "Vert Ardent");
        (p, vert)
    }

    #[test]
    fn a_missing_file_is_an_empty_library() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = store_in(dir.path());
        assert!(store.library().palette.is_empty());
        assert_eq!(store.take_issue(), None);
        assert!(store.take_write().is_none(), "nothing to write");
    }

    #[test]
    fn a_damaged_file_is_backed_up_and_reported_once() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(tp_file::library::FILE_NAME);
        std::fs::write(&path, b"PK\x03\x04 damaged").unwrap();
        let mut store = store_in(dir.path());
        assert!(store.library().symbols.is_empty());
        let issue = store.take_issue().expect("reported");
        let backup = issue.backup.expect("backed up");
        assert_eq!(std::fs::read(&backup).unwrap(), b"PK\x03\x04 damaged");
        assert!(!path.exists());
        assert_eq!(store.take_issue(), None, "only once");
    }

    #[test]
    fn published_entries_are_there_after_a_reload() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = store_in(dir.path());
        let (mut p, vert) = project();
        let picks = Picks {
            swatches: vec![vert],
            ..Picks::default()
        };
        let origins = store.publish(&p, &picks);
        p.record_origins(&origins);
        let key = p.swatch(vert).unwrap().origin.clone().expect("linked");
        assert!(store.contains(&key));
        let (library, path) = store.take_write().expect("a write is due");
        assert!(store.take_write().is_none());
        tp_file::library::write(&library, &path).unwrap();
        let mut again = store_in(dir.path());
        assert_eq!(again.library().palette[0].name, "Vert Ardent");
        assert!(again.contains(&key));
        assert!(again.remove(&key));
        assert!(again.library().palette.is_empty());
        assert!(!again.remove(&key));
    }

    #[test]
    fn importing_is_one_undo_step() {
        let mut store = LibraryStore::default();
        let (mut source, vert) = project();
        let mut stripe = tp_core::document::Object::new(
            tp_core::document::ObjectId(0),
            tp_core::document::ShapeKind::rectangle(),
            tp_core::document::Frame::new(
                tp_core::kurbo::Point::new(100.0, 100.0),
                tp_core::kurbo::Size::new(80.0, 20.0),
                0.0,
            ),
        );
        stripe.fill = tp_core::document::Paint::Solid(source.swatch(vert).unwrap().color);
        stripe.fill_swatch = Some(vert);
        let stripe = source.add(stripe);
        let (logo, _) = source.convert_to_symbol(&[stripe], "Logo").unwrap();
        store.publish(
            &source,
            &Picks {
                symbols: vec![logo],
                ..Picks::default()
            },
        );
        let library = store.library().clone();
        let mut ws = Workspace::new(crate::vehicle_project::test_project("B"));
        let before = ws.snapshot();
        let picks = Picks {
            symbols: vec![library.symbols[0].id],
            ..Picks::default()
        };
        assert_eq!(ws.import_from_library(&library, &picks, 1.0), 2);
        assert_eq!(ws.history.len(), 1);
        assert_eq!(ws.history.undo_label(), Some("undo-import-from-library"));
        assert_eq!(ws.project.symbols.len(), 1);
        assert_eq!(ws.project.palette.len(), 1);
        ws.undo();
        assert!(ws.snapshot().same_saved(&before));
        assert!(ws.project.symbols.is_empty() && ws.project.palette.is_empty());
    }

    #[test]
    fn keys_are_unique() {
        let keys: HashSet<LibraryKey> = (0..10_000).map(|_| new_key()).collect();
        assert_eq!(keys.len(), 10_000);
        let key = keys.iter().next().unwrap();
        assert_eq!(key.0.len(), 32);
        assert!(key.0.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
