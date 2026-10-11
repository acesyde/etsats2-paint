//! Official vehicles: what picking one does, and installing it from the
//! game's template pack on a worker thread (the dialog stays responsive and
//! can cancel). The package is built from the catalog entry and the pack's
//! files, then installed like any package.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;

use tp_i18n::tr;
use tp_pack::Packed;
use tp_pack::catalog::CatalogError;
use tp_vehicles::Game;
use tp_vehicles::catalog::Entry;

use crate::prefs::Prefs;
use crate::ui::vehicle_dialogs::VehicleChoice;
use crate::vehicles::VehicleLibrary;

/// The forum thread of Carsmaniac's template pack for ETS2.
pub const ETS2_DOWNLOAD_PAGE: &str = "https://forum.scssoft.com/viewtopic.php?f=33&t=272386";
/// The forum thread of Carsmaniac's template pack for ATS.
pub const ATS_DOWNLOAD_PAGE: &str = "https://forum.scssoft.com/viewtopic.php?f=199&t=288778";

/// The download page of `game`'s template pack.
pub fn download_page(game: Game) -> &'static str {
    match game {
        Game::Ets2 => ETS2_DOWNLOAD_PAGE,
        Game::Ats => ATS_DOWNLOAD_PAGE,
    }
}

/// What picking an official vehicle does.
#[derive(Clone, Debug, PartialEq)]
pub enum Pick {
    /// Installed at the catalog's version or newer: chosen at once,
    /// without reading the pack.
    Choose(VehicleChoice),
    /// Installed from the pack folder, which holds the vehicle's folder.
    Install(PathBuf),
    /// The template pack dialog asks for the pack; `moved` when the
    /// remembered folder no longer holds the vehicle.
    AskPack { moved: bool },
}

/// What picking `entry` does, with the installed vehicles and the
/// remembered pack folders.
pub fn pick(library: &VehicleLibrary, prefs: &Prefs, entry: &Entry) -> Pick {
    if let Some(vehicle) = library.get(&entry.package_id())
        && vehicle.newest().manifest.version >= entry.version
    {
        return Pick::Choose(VehicleChoice::of(vehicle));
    }
    match prefs.template_pack(entry.game) {
        Some(pack) if pack.join(&entry.folder).is_dir() => Pick::Install(pack.to_path_buf()),
        Some(_) => Pick::AskPack { moved: true },
        None => Pick::AskPack { moved: false },
    }
}

/// How a build ended.
#[derive(Debug)]
pub enum InstallOutcome {
    Built(Box<Packed>),
    Failed(CatalogError),
    /// The worker stopped without an answer.
    Stopped,
}

/// An official vehicle's package being built from the pack. Dropping it
/// (or [`InstallJob::cancel`]) stops the build: its result is never
/// installed.
pub struct InstallJob {
    entry: &'static Entry,
    done_files: Arc<AtomicUsize>,
    cancel: Arc<AtomicBool>,
    /// In a mutex so the job can be shared ([`Installing`]).
    done: Mutex<Receiver<InstallOutcome>>,
}

impl std::fmt::Debug for InstallJob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstallJob")
            .field("vehicle", &self.entry.package_id())
            .field("progress", &self.progress())
            .finish()
    }
}

impl InstallJob {
    /// Builds `entry` from the pack folder `pack`.
    pub fn start(
        entry: &'static Entry,
        pack: PathBuf,
        notify: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        let done_files = Arc::new(AtomicUsize::new(0));
        let cancel = Arc::new(AtomicBool::new(false));
        let (tx, done) = mpsc::channel();
        let (count, stop) = (done_files.clone(), cancel.clone());
        let spawned = thread::Builder::new()
            .name("official-vehicle".into())
            .spawn(move || {
                while is_held(&pack) && !stop.load(Ordering::Relaxed) {
                    thread::sleep(std::time::Duration::from_millis(5));
                }
                let result = tp_pack::catalog::build_with(
                    entry,
                    &pack,
                    &mut |files, _| {
                        count.store(files, Ordering::Relaxed);
                        notify();
                    },
                    &stop,
                );
                let outcome = match result {
                    Ok(packed) => InstallOutcome::Built(Box::new(packed)),
                    Err(err) => InstallOutcome::Failed(err),
                };
                // The dialog may be gone: the result is then dropped.
                let _ = tx.send(outcome);
                notify();
            });
        if let Err(err) = spawned {
            tracing::error!(%err, "cannot start installing an official vehicle");
        }
        Self {
            entry,
            done_files,
            cancel,
            done: Mutex::new(done),
        }
    }

    /// The vehicle being installed.
    pub fn entry(&self) -> &'static Entry {
        self.entry
    }

    /// Files read so far, and how many will be.
    pub fn progress(&self) -> (usize, usize) {
        (
            self.done_files.load(Ordering::Relaxed),
            self.entry.textures().count(),
        )
    }

    /// Stops the build.
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    /// The outcome once finished.
    pub fn poll(&self) -> Option<InstallOutcome> {
        let Ok(done) = self.done.lock() else {
            return Some(InstallOutcome::Stopped);
        };
        match done.try_recv() {
            Ok(outcome) => Some(outcome),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(InstallOutcome::Stopped),
        }
    }

    /// Blocks until the build ends (tests).
    pub fn wait(&self) -> InstallOutcome {
        match self.done.lock() {
            Ok(done) => done.recv().unwrap_or(InstallOutcome::Stopped),
            Err(_) => InstallOutcome::Stopped,
        }
    }
}

impl Drop for InstallJob {
    fn drop(&mut self) {
        self.cancel();
    }
}

/// Pack folders whose builds wait before reading anything (tests).
static HELD: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

fn is_held(pack: &std::path::Path) -> bool {
    HELD.lock().is_ok_and(|held| held.iter().any(|p| p == pack))
}

/// Makes the builds from `pack` wait until released (or cancelled), so a
/// test can see an installation in progress.
#[doc(hidden)]
pub fn hold_builds_from(pack: &std::path::Path, held: bool) {
    if let Ok(mut list) = HELD.lock() {
        list.retain(|p| p != pack);
        if held {
            list.push(pack.to_path_buf());
        }
    }
}

/// An installation shown by a dialog (New Project, Add Vehicle…): shared,
/// so the dialog's state stays cloneable; the build stops once the last
/// copy is dropped.
pub type Installing = Arc<InstallJob>;

/// How an installation ended, for the dialog that started it.
#[derive(Debug, PartialEq)]
pub enum Finished {
    /// Installed and chosen, with the confirmation.
    Installed(VehicleChoice, String),
    /// Nothing installed, and why.
    Refused(String),
    /// Cancelled: nothing installed, nothing to say.
    Cancelled,
}

/// Installs what `job` built, or says why it can't be.
pub fn finish(library: &mut VehicleLibrary, entry: &Entry, outcome: InstallOutcome) -> Finished {
    let packed = match outcome {
        InstallOutcome::Built(packed) => packed,
        InstallOutcome::Failed(CatalogError::Cancelled) => return Finished::Cancelled,
        InstallOutcome::Failed(err) => return Finished::Refused(error_message(&err, entry)),
        InstallOutcome::Stopped => {
            return Finished::Refused(tr!("custom-build-stopped", name = entry.name.as_str()));
        }
    };
    match library.install_bytes(&packed.bytes) {
        Ok(m) => match library.get(&m.id) {
            Some(vehicle) => Finished::Installed(
                VehicleChoice::of(vehicle),
                tr!(
                    "vehicles-installed",
                    name = m.name.as_str(),
                    version = m.version.to_string()
                ),
            ),
            None => Finished::Cancelled,
        },
        Err(err) => Finished::Refused(crate::vehicles::install_error_message(&err, &entry.name)),
    }
}

/// Why `entry` couldn't be built from the pack, in the current language.
pub fn error_message(err: &CatalogError, entry: &Entry) -> String {
    let vehicle = entry.name.as_str();
    match err {
        CatalogError::NoVehicleFolder { folder, .. } => tr!(
            "official-no-folder",
            vehicle = vehicle,
            folder = folder.display().to_string()
        ),
        CatalogError::MissingFile { file, .. } => {
            tr!(
                "official-missing-file",
                vehicle = vehicle,
                file = file.as_str()
            )
        }
        CatalogError::WrongSize {
            file,
            expected,
            actual,
            ..
        } => tr!(
            "official-other-version",
            file = file.as_str(),
            size = format!("{} × {}", actual.0, actual.1),
            expected = expected.label()
        ),
        CatalogError::Unreadable { file, reason, .. } => tr!(
            "official-unreadable",
            vehicle = vehicle,
            file = file.as_str(),
            reason = reason.as_str()
        ),
        CatalogError::Pack(err) => crate::vehicles::pack_error_message(err, vehicle),
        CatalogError::Cancelled => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// A temporary pack of `entry`, its files blank PNGs of their sizes.
    fn pack(root: &Path, entry: &Entry) {
        let dir = root.join(&entry.folder);
        for t in entry.textures() {
            let path = t.file.split('/').fold(dir.clone(), |p, s| p.join(s));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            let mut out = Vec::new();
            image::GrayImage::new(t.size.width, t.size.height)
                .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
                .unwrap();
            std::fs::write(path, out).unwrap();
        }
    }

    fn krone() -> &'static Entry {
        tp_vehicles::catalog::by_package_id("official.ets2.krone.coolliner").unwrap()
    }

    #[test]
    fn picking_asks_for_the_pack_installs_or_chooses() {
        let dir = tempfile::tempdir().unwrap();
        let mut library = VehicleLibrary::open(&dir.path().join("library"));
        let mut prefs = Prefs::default();
        let entry = krone();
        assert_eq!(
            pick(&library, &prefs, entry),
            Pick::AskPack { moved: false }
        );
        let packs = dir.path().join("pack");
        prefs.set_template_pack(Game::Ets2, Some(packs.clone()));
        assert_eq!(pick(&library, &prefs, entry), Pick::AskPack { moved: true });
        pack(&packs, entry);
        assert_eq!(pick(&library, &prefs, entry), Pick::Install(packs.clone()));
        let job = InstallJob::start(entry, packs.clone(), || {});
        let outcome = job.wait();
        assert_eq!(job.progress(), (5, 5));
        let Finished::Installed(choice, _) = finish(&mut library, entry, outcome) else {
            panic!("installed");
        };
        assert_eq!(choice.id, "official.ets2.krone.coolliner");
        // Installed: chosen without the pack.
        std::fs::remove_dir_all(&packs).unwrap();
        assert_eq!(pick(&library, &prefs, entry), Pick::Choose(choice));
    }

    #[test]
    fn a_cancelled_job_installs_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let mut library = VehicleLibrary::open(&dir.path().join("library"));
        let entry = krone();
        pack(dir.path(), entry);
        let job = InstallJob::start(entry, dir.path().to_path_buf(), || {});
        job.cancel();
        let outcome = job.wait();
        // Cancelled before the end, or built but not installed by anyone.
        if matches!(outcome, InstallOutcome::Failed(_)) {
            assert_eq!(finish(&mut library, entry, outcome), Finished::Cancelled);
        }
        assert!(library.vehicles().is_empty());
    }

    #[test]
    fn a_missing_file_is_refused_with_its_name() {
        let dir = tempfile::tempdir().unwrap();
        let mut library = VehicleLibrary::open(&dir.path().join("library"));
        let entry = krone();
        pack(dir.path(), entry);
        std::fs::remove_file(
            dir.path()
                .join(&entry.folder)
                .join("Accessories/Chassis Rear - Pallet Box.png"),
        )
        .unwrap();
        let job = InstallJob::start(entry, dir.path().to_path_buf(), || {});
        let Finished::Refused(message) = finish(&mut library, entry, job.wait()) else {
            panic!("refused");
        };
        assert!(message.contains("Krone Cool Liner"), "{message}");
        assert!(
            message.contains("Accessories/Chassis Rear - Pallet Box.png"),
            "{message}"
        );
        assert!(library.vehicles().is_empty());
    }
}
