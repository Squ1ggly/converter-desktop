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
    cancellation_tokens: Mutex<HashMap<String, CancellationToken>>,
}

impl ActiveBatches {
    pub fn start(&self, batch_id: &str) -> CancellationToken {
        let cancellation = CancellationToken::new();
        lock_ignoring_poison(&self.cancellation_tokens)
            .insert(batch_id.to_string(), cancellation.clone());
        cancellation
    }

    pub fn finish(&self, batch_id: &str) {
        lock_ignoring_poison(&self.cancellation_tokens).remove(batch_id);
    }

    pub fn cancel(&self, batch_id: &str) {
        if let Some(cancellation) = lock_ignoring_poison(&self.cancellation_tokens).get(batch_id) {
            cancellation.cancel();
        }
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
