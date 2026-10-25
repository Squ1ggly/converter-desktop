use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::ffmpeg::Ffmpeg;

pub struct AppState {
    // Err holds a message for the UI if the bundled FFmpeg couldn't be found.
    pub ffmpeg: Result<Ffmpeg, String>,
    // Per-user, cleared on launch.
    pub scratch_root: PathBuf,
    pub active_batches: ActiveBatches,
    pub produced_outputs: Arc<ProducedOutputs>,
}

#[derive(Default)]
pub struct ActiveBatches {
    batches: Mutex<HashMap<String, BatchEntry>>,
    batch_finished: Condvar,
}

struct BatchEntry {
    cancellation: CancellationToken,
    is_running: bool,
}

impl BatchEntry {
    fn not_started() -> BatchEntry {
        BatchEntry {
            cancellation: CancellationToken::new(),
            is_running: false,
        }
    }
}

impl ActiveBatches {
    // A cancel that arrived before the batch started is kept, so the batch
    // starts already cancelled.
    pub fn start(&self, batch_id: &str) -> Result<CancellationToken, String> {
        let mut batches = lock_ignoring_poison(&self.batches);
        let entry = batches
            .entry(batch_id.to_string())
            .or_insert_with(BatchEntry::not_started);

        if entry.is_running {
            return Err("This conversion is already running".to_string());
        }
        entry.is_running = true;

        let cancellation = entry.cancellation.clone();
        Ok(cancellation)
    }

    pub fn finish(&self, batch_id: &str) {
        lock_ignoring_poison(&self.batches).remove(batch_id);
        self.batch_finished.notify_all();
    }

    pub fn is_any_running(&self) -> bool {
        let batches = lock_ignoring_poison(&self.batches);
        let is_any_running = batches.values().any(|entry| entry.is_running);
        is_any_running
    }

    pub fn cancel_all(&self) {
        for entry in lock_ignoring_poison(&self.batches).values() {
            entry.cancellation.cancel();
        }
    }

    // Returns false if batches were still running when the time ran out.
    pub fn wait_until_finished(&self, timeout: Duration) -> bool {
        let batches = lock_ignoring_poison(&self.batches);
        let (batches, _timeout_result) = self
            .batch_finished
            .wait_timeout_while(batches, timeout, |batches| {
                batches.values().any(|entry| entry.is_running)
            })
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let has_finished = !batches.values().any(|entry| entry.is_running);
        has_finished
    }

    // `convert_files` runs on the async runtime and `cancel_conversion` on the
    // main thread, so a cancel can arrive before the batch has started.
    pub fn cancel(&self, batch_id: &str) {
        let mut batches = lock_ignoring_poison(&self.batches);
        let entry = batches
            .entry(batch_id.to_string())
            .or_insert_with(BatchEntry::not_started);
        entry.cancellation.cancel();
    }
}

// Files written this session. Open and reveal only accept these paths.
#[derive(Default)]
pub struct ProducedOutputs {
    paths: Mutex<HashSet<PathBuf>>,
}

impl ProducedOutputs {
    pub fn remember(&self, path: &Path) {
        lock_ignoring_poison(&self.paths).insert(path.to_path_buf());
    }

    pub fn contains(&self, path: &Path) -> bool {
        lock_ignoring_poison(&self.paths).contains(path)
    }
}

// Every value guarded here is replaced in one step, so a panic elsewhere can't
// leave it half-updated and poisoning can be ignored.
fn lock_ignoring_poison<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cancel_that_arrives_first_still_cancels_the_batch() {
        let active_batches = ActiveBatches::default();

        active_batches.cancel("batch");
        let cancellation = active_batches.start("batch").expect("starts");

        assert!(cancellation.is_cancelled());
    }

    #[test]
    fn a_running_batch_id_cannot_start_twice() {
        let active_batches = ActiveBatches::default();
        let first_run = active_batches.start("batch").expect("starts");

        assert!(active_batches.start("batch").is_err());
        active_batches.cancel("batch");
        assert!(first_run.is_cancelled());
    }

    #[test]
    fn quitting_cancels_every_batch_and_waits_for_them_to_finish() {
        let active_batches = Arc::new(ActiveBatches::default());
        let first_batch = active_batches.start("images").expect("starts");
        let second_batch = active_batches.start("videos").expect("starts");
        assert!(active_batches.is_any_running());

        let batches_for_worker = active_batches.clone();
        let worker = std::thread::spawn(move || {
            while !(first_batch.is_cancelled() && second_batch.is_cancelled()) {
                std::thread::yield_now();
            }
            batches_for_worker.finish("images");
            batches_for_worker.finish("videos");
        });

        active_batches.cancel_all();
        let has_finished = active_batches.wait_until_finished(Duration::from_secs(5));

        worker.join().unwrap();
        assert!(has_finished);
        assert!(!active_batches.is_any_running());
    }

    #[test]
    fn waiting_gives_up_after_the_timeout() {
        let active_batches = ActiveBatches::default();
        active_batches.start("stuck").expect("starts");

        let has_finished = active_batches.wait_until_finished(Duration::from_millis(50));

        assert!(!has_finished);
    }

    #[test]
    fn a_finished_batch_id_starts_fresh() {
        let active_batches = ActiveBatches::default();
        active_batches.start("batch").expect("starts");
        active_batches.finish("batch");

        let second_run = active_batches.start("batch").expect("starts again");

        assert!(!second_run.is_cancelled());
    }
}
