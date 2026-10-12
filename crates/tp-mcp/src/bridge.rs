//! The channel between the server thread and the app. The server sends a
//! [`Job`] per tool call and waits for its answer; the app takes jobs when
//! the user is not in the middle of something, runs them on the project and
//! answers. A job that waited too long is cancelled and never run.

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use serde_json::Value;

use crate::call::Call;

/// How long a call waits for the app before failing.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// What a call failed with when the app didn't take it in time.
pub const BUSY: &str = "TruckPaint is busy (a dialog is open or the user is in the middle of \
an edit); nothing was changed. Try again in a moment.";

/// What a tool call returns.
pub enum Outcome {
    /// A structured result, sent to the client as JSON text.
    Json(Value),
    /// A PNG image and a line about it.
    Image { png: Vec<u8>, text: String },
    /// Work the server thread finishes (rendering), so the app's thread
    /// never waits for it.
    Deferred(Box<dyn FnOnce() -> Result<Outcome, String> + Send>),
}

impl std::fmt::Debug for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(v) => f.debug_tuple("Json").field(v).finish(),
            Self::Image { png, text } => f
                .debug_struct("Image")
                .field("bytes", &png.len())
                .field("text", text)
                .finish(),
            Self::Deferred(_) => f.write_str("Deferred"),
        }
    }
}

/// A call's result, or why it failed (the document is then unchanged).
pub type Reply = Result<Outcome, String>;

const PENDING: u8 = 0;
const TAKEN: u8 = 1;
const CANCELLED: u8 = 2;

/// A tool call waiting for the app.
pub struct Job {
    pub call: Call,
    reply: Sender<Reply>,
    state: Arc<AtomicU8>,
}

impl Job {
    /// Takes the job to run it; false when it was cancelled (it timed out),
    /// in which case it must not run.
    pub fn claim(&self) -> bool {
        self.state
            .compare_exchange(PENDING, TAKEN, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    /// Sends the result to the waiting call.
    pub fn answer(self, reply: Reply) {
        let _ = self.reply.send(reply);
    }
}

/// The app's end: the calls waiting to run.
pub struct Inbox {
    jobs: Receiver<Job>,
}

impl Inbox {
    /// The next call waiting, if any.
    pub fn next(&self) -> Option<Job> {
        self.jobs.try_recv().ok()
    }
}

/// The server's end.
#[derive(Clone)]
pub struct Bridge {
    jobs: Sender<Job>,
    /// Wakes the app (an idle egui app doesn't run frames).
    wake: Arc<dyn Fn() + Send + Sync>,
    timeout: Duration,
}

/// A bridge and its inbox. `wake` is called after each call is sent.
pub fn bridge(wake: impl Fn() + Send + Sync + 'static, timeout: Duration) -> (Bridge, Inbox) {
    let (jobs, rx) = mpsc::channel();
    (
        Bridge {
            jobs,
            wake: Arc::new(wake),
            timeout,
        },
        Inbox { jobs: rx },
    )
}

impl Bridge {
    /// Sends `call` to the app and waits for its result, finishing deferred
    /// work on this thread.
    pub fn call(&self, call: Call) -> Reply {
        let (reply, answer) = mpsc::channel();
        let state = Arc::new(AtomicU8::new(PENDING));
        let job = Job {
            call,
            reply,
            state: state.clone(),
        };
        self.jobs
            .send(job)
            .map_err(|_| "the TruckPaint server is stopping".to_owned())?;
        (self.wake)();
        let reply = match answer.recv_timeout(self.timeout) {
            Ok(reply) => reply,
            Err(RecvTimeoutError::Timeout) => {
                let cancelled = state
                    .compare_exchange(PENDING, CANCELLED, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok();
                if cancelled {
                    return Err(BUSY.to_owned());
                }
                // Taken just now: its answer is on its way.
                answer
                    .recv()
                    .map_err(|_| "TruckPaint stopped before answering".to_owned())?
            }
            Err(RecvTimeoutError::Disconnected) => {
                return Err("TruckPaint stopped before answering".to_owned());
            }
        };
        match reply {
            Ok(Outcome::Deferred(work)) => work(),
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn answered_call() {
        let (bridge, inbox) = bridge(|| {}, Duration::from_secs(5));
        let app = std::thread::spawn(move || {
            loop {
                if let Some(job) = inbox.next() {
                    assert!(job.claim());
                    job.answer(Ok(Outcome::Deferred(Box::new(|| {
                        Ok(Outcome::Json(json!({"done": true})))
                    }))));
                    break;
                }
                std::thread::yield_now();
            }
        });
        let Ok(Outcome::Json(v)) = bridge.call(Call::GetProject) else {
            panic!("json");
        };
        assert_eq!(v, json!({"done": true}));
        app.join().unwrap();
    }

    #[test]
    fn late_job_is_cancelled_and_never_runs() {
        let (bridge, inbox) = bridge(|| {}, Duration::from_millis(20));
        let err = bridge.call(Call::GetProject).unwrap_err();
        assert_eq!(err, BUSY);
        let job = inbox.next().expect("still queued");
        assert!(!job.claim());
    }

    #[test]
    fn stopped_app() {
        let (bridge, inbox) = bridge(|| {}, Duration::from_secs(5));
        drop(inbox);
        assert!(bridge.call(Call::GetProject).is_err());
    }
}
