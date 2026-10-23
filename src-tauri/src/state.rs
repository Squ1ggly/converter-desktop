use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tokio_util::sync::CancellationToken;

use crate::ffmpeg::Ffmpeg;

pub struct AppState {
    // Err holds a message for the UI if the bundled FFmpeg couldn't be found.
    pub ffmpeg: Result<Ffmpeg, String>,
    // Per-user scratch space for intermediate files, inside the app's cache
    // folder. Emptied every launch.
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
        self.lock()
            .insert(batch_id.to_string(), cancellation.clone());
        cancellation
    }

    pub fn finish(&self, batch_id: &str) {
        self.lock().remove(batch_id);
    }

    pub fn cancel(&self, batch_id: &str) {
        if let Some(cancellation) = self.lock().get(batch_id) {
            cancellation.cancel();
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, CancellationToken>> {
        // A panic while holding this lock can't leave the map inconsistent,
        // so carry on with the data rather than failing every later call.
        self.cancellation_tokens
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

// Files this app has written during the current session. "Open" and "Show in
// folder" only act on these, so the webview can't be used to open arbitrary
// paths on the user's machine.
#[derive(Default)]
pub struct ProducedOutputs {
    paths: Mutex<HashSet<PathBuf>>,
}

impl ProducedOutputs {
    pub fn remember(&self, path: &Path) {
        self.lock().insert(path.to_path_buf());
    }

    pub fn contains(&self, path: &Path) -> bool {
        self.lock().contains(path)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashSet<PathBuf>> {
        self.paths
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
