//! Background thread writing project files, so saving never freezes the
//! editor. Jobs are written in order; dropping the saver waits for pending
//! writes, so nothing is lost when a project closes or the app quits.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use tp_core::Project;

/// What a write is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobKind {
    /// The user's project file.
    Save,
    /// A crash-recovery copy.
    Recovery,
    /// The personal library.
    Library,
}

struct Job {
    id: u64,
    kind: JobKind,
    project: Project,
    path: PathBuf,
    /// Extra small file written after the project (recovery metadata).
    extra: Option<(PathBuf, String)>,
}

/// Result of a finished write.
#[derive(Debug)]
pub struct Done {
    pub id: u64,
    pub kind: JobKind,
    pub path: PathBuf,
    pub result: Result<(), String>,
}

pub struct Saver {
    jobs: Option<Sender<Job>>,
    done: Receiver<Done>,
    thread: Option<JoinHandle<()>>,
    next_id: u64,
}

impl Saver {
    /// Starts the writer thread; `notify` is called after each write (to
    /// repaint the UI).
    pub fn new(notify: impl Fn() + Send + 'static) -> Self {
        let (jobs, job_rx) = mpsc::channel::<Job>();
        let (done_tx, done) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("project-saver".into())
            .spawn(move || {
                while let Ok(job) = job_rx.recv() {
                    let mut result = match job.kind {
                        JobKind::Library => tp_file::library::write(&job.project, &job.path),
                        JobKind::Save | JobKind::Recovery => {
                            tp_file::write(&job.project, &job.path)
                        }
                    }
                    .map_err(|e| e.to_string());
                    if result.is_ok()
                        && let Some((path, text)) = &job.extra
                    {
                        result =
                            tp_file::write_atomic(path, text.as_bytes()).map_err(|e| e.to_string());
                    }
                    if let Err(err) = &result {
                        tracing::error!(path = %job.path.display(), %err, "write failed");
                    }
                    let _ = done_tx.send(Done {
                        id: job.id,
                        kind: job.kind,
                        path: job.path,
                        result,
                    });
                    notify();
                }
            })
            .ok();
        Self {
            jobs: Some(jobs),
            done,
            thread,
            next_id: 1,
        }
    }

    /// Queues a write; returns its id, or `None` if the writer thread is
    /// not running.
    pub fn write(
        &mut self,
        kind: JobKind,
        project: Project,
        path: PathBuf,
        extra: Option<(PathBuf, String)>,
    ) -> Option<u64> {
        let id = self.next_id;
        self.next_id += 1;
        let job = Job {
            id,
            kind,
            project,
            path: path.clone(),
            extra,
        };
        let sent =
            self.thread.is_some() && self.jobs.as_ref().is_some_and(|tx| tx.send(job).is_ok());
        if !sent {
            tracing::error!("project saver is not running");
        }
        sent.then_some(id)
    }

    /// Writes finished since the last call.
    pub fn finished(&self) -> Vec<Done> {
        self.done.try_iter().collect()
    }

    /// Blocks until every queued write is done and returns them.
    pub fn flush(&mut self) -> Vec<Done> {
        self.jobs = None;
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        self.done.try_iter().collect()
    }
}

impl Drop for Saver {
    fn drop(&mut self) {
        self.flush();
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn writes_in_the_background_and_reports() {
        let dir = tempfile::tempdir().unwrap();
        let mut saver = Saver::new(|| {});
        let path = dir.path().join("a.truckpaint");
        let id = saver
            .write(
                JobKind::Save,
                crate::vehicle_project::test_project("a"),
                path.clone(),
                None,
            )
            .unwrap();
        let done = saver.flush();
        assert_eq!(done.len(), 1);
        assert_eq!((done[0].id, done[0].kind), (id, JobKind::Save));
        assert!(done[0].result.is_ok());
        assert_eq!(tp_file::read(&path).unwrap().project.name, "a");
    }

    #[test]
    fn failures_are_reported() {
        let mut saver = Saver::new(|| {});
        saver.write(
            JobKind::Save,
            crate::vehicle_project::test_project("a"),
            PathBuf::from("/nonexistent-dir/a.truckpaint"),
            None,
        );
        let done = saver.flush();
        assert!(done[0].result.is_err());
    }
}
