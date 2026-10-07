//! Building a custom vehicle's package on a worker thread: DDS templates
//! are decoded and PNG-encoded there, so the dialog stays responsive.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use tp_pack::custom::CustomVehicle;
use tp_pack::{PackError, Packed};

/// How a build ended.
#[derive(Debug)]
pub enum PackOutcome {
    Built(Box<Packed>),
    Failed(PackError),
    /// The worker stopped without an answer.
    Stopped,
}

/// A package being built. Dropping it abandons the build: its result is
/// never installed.
pub struct PackJob {
    converted: Arc<AtomicUsize>,
    total: usize,
    done: Receiver<PackOutcome>,
}

impl PackJob {
    /// Builds `vehicle` (a copy of the form: editing may continue).
    pub fn start(vehicle: CustomVehicle, notify: impl Fn() + Send + Sync + 'static) -> Self {
        let converted = Arc::new(AtomicUsize::new(0));
        let total = vehicle.conversions();
        let (tx, done) = mpsc::channel();
        let count = converted.clone();
        let spawned = thread::Builder::new()
            .name("custom-vehicle".into())
            .spawn(move || {
                let result = vehicle.pack(&mut || {
                    count.fetch_add(1, Ordering::Relaxed);
                    notify();
                });
                let outcome = match result {
                    Ok(packed) => PackOutcome::Built(Box::new(packed)),
                    Err(err) => PackOutcome::Failed(err),
                };
                // The dialog may be gone: the result is then dropped.
                let _ = tx.send(outcome);
                notify();
            });
        if let Err(err) = spawned {
            tracing::error!(%err, "cannot start building the package");
        }
        Self {
            converted,
            total,
            done,
        }
    }

    /// Templates converted so far, and how many will be.
    pub fn progress(&self) -> (usize, usize) {
        (self.converted.load(Ordering::Relaxed), self.total)
    }

    /// The outcome once finished.
    pub fn poll(&self) -> Option<PackOutcome> {
        match self.done.try_recv() {
            Ok(outcome) => Some(outcome),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(PackOutcome::Stopped),
        }
    }

    /// Blocks until the build ends (tests).
    pub fn wait(&self) -> PackOutcome {
        self.done.recv().unwrap_or(PackOutcome::Stopped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tp_pack::custom::probe;
    use tp_vehicles::Game;

    fn png() -> Vec<u8> {
        tp_vehicles::sample::png(8)
    }

    fn truck() -> CustomVehicle {
        let mut v = CustomVehicle::new(Game::Ets2);
        v.name = "R 2024".into();
        v.brand = "Scania".into();
        v.path = "scania.r_2024".into();
        v.add(probe("cabin.png", png().into()).unwrap());
        v
    }

    #[test]
    fn a_completed_job_returns_the_package() {
        let job = PackJob::start(truck(), || {});
        let PackOutcome::Built(packed) = job.wait() else {
            panic!("built");
        };
        assert_eq!(packed.manifest.id, "custom.scania.r_2024");
        assert_eq!(job.progress(), (0, 0));
    }

    #[test]
    fn a_failing_job_reports_why() {
        let mut v = truck();
        v.path = "Not A Path".into();
        let job = PackJob::start(v, || {});
        assert!(matches!(
            job.wait(),
            PackOutcome::Failed(PackError::Package(_))
        ));
    }

    #[test]
    fn a_dropped_job_installs_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let library = crate::vehicles::VehicleLibrary::open(dir.path());
        let (tx, rx) = mpsc::channel();
        let job = PackJob::start(truck(), move || {
            let _ = tx.send(());
        });
        drop(job);
        // The worker finishes without anyone to answer.
        while rx.recv().is_ok() {}
        assert!(library.vehicles().is_empty());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }
}
