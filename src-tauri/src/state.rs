use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

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
    fn a_finished_batch_id_starts_fresh() {
        let active_batches = ActiveBatches::default();
        active_batches.start("batch").expect("starts");
        active_batches.finish("batch");

        let second_run = active_batches.start("batch").expect("starts again");

        assert!(!second_run.is_cancelled());
    }
}
