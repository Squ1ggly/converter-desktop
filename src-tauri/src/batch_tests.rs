// Whole batches run through `run_batch`, as `convert_files` runs them: events,
// output names, cleanup after failures and cancelling.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use image::{Rgba, RgbaImage};
use serde_json::Value;
use tauri::ipc::{Channel, InvokeResponseBody};
use tokio_util::sync::CancellationToken;

use crate::batch::{BatchContext, ConversionEvent, ConversionRequest, RequestedFile, run_batch};
use crate::ffmpeg::Ffmpeg;
use crate::formats::ImageFormat;
use crate::options::{ConversionSettings, ImageOptions, ResizeOption};
use crate::state::ProducedOutputs;

type RecordedEvents = Arc<Mutex<Vec<Value>>>;

struct TestBatch {
    files: Vec<PathBuf>,
    settings: ConversionSettings,
    output_directory: Option<PathBuf>,
    cancellation: CancellationToken,
}

impl TestBatch {
    fn new(files: Vec<PathBuf>, settings: ConversionSettings) -> TestBatch {
        TestBatch {
            files,
            settings,
            output_directory: None,
            cancellation: CancellationToken::new(),
        }
    }

    // Returns every event sent, in order.
    async fn run(self) -> Vec<Value> {
        let scratch_root = tempfile::tempdir().unwrap();
        let (events, recorded_events) = recording_channel();
        let context = BatchContext {
            ffmpeg: Ffmpeg::from_downloaded_binary(),
            scratch_root: scratch_root.path().to_path_buf(),
            cancellation: self.cancellation,
            events,
            produced_outputs: Arc::new(ProducedOutputs::default()),
        };
        let request = ConversionRequest {
            batch_id: "test".to_string(),
            files: requested_files(self.files),
            settings: self.settings,
            output_directory: self.output_directory,
        };

        run_batch(request, context).await;

        let events = recorded_events.lock().unwrap().clone();
        events
    }
}

fn recording_channel() -> (Channel<ConversionEvent>, RecordedEvents) {
    let recorded_events = RecordedEvents::default();
    let event_sink = recorded_events.clone();
    let channel = Channel::new(move |body| {
        if let InvokeResponseBody::Json(json) = body {
            event_sink
                .lock()
                .unwrap()
                .push(serde_json::from_str(&json).unwrap());
        }
        Ok(())
    });
    (channel, recorded_events)
}

fn requested_files(paths: Vec<PathBuf>) -> Vec<RequestedFile> {
    let files = paths
        .into_iter()
        .enumerate()
        .map(|(index, path)| RequestedFile {
            file_id: format!("file-{index}"),
            path,
        })
        .collect();
    files
}

fn image_settings(format: ImageFormat) -> ConversionSettings {
    ConversionSettings::Image {
        format,
        options: ImageOptions {
            quality: 80,
            resize: ResizeOption::Original,
        },
    }
}

// The last event for each file: completed, failed or cancelled.
fn final_events(events: &[Value]) -> Vec<&Value> {
    let final_events = events
        .iter()
        .filter(|event| event["type"] != "started" && event["type"] != "progress")
        .collect();
    final_events
}

fn event_types(events: &[&Value]) -> Vec<String> {
    let types = events
        .iter()
        .map(|event| event["type"].as_str().unwrap().to_string())
        .collect();
    types
}

fn resolved_temp_folder() -> (tempfile::TempDir, PathBuf) {
    let folder = tempfile::tempdir().unwrap();
    let resolved_folder = dunce::canonicalize(folder.path()).unwrap();
    (folder, resolved_folder)
}

fn write_sample_png(path: &Path) {
    RgbaImage::from_pixel(16, 12, Rgba([200, 30, 30, 255]))
        .save(path)
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn relative_paths_fail_without_writing_anything() {
    let (_folder, folder_path) = resolved_temp_folder();
    let photo_path = folder_path.join("photo.png");
    write_sample_png(&photo_path);

    let relative_input = TestBatch::new(
        vec![PathBuf::from("photo.png")],
        image_settings(ImageFormat::Jpg),
    );
    let mut relative_output =
        TestBatch::new(vec![photo_path.clone()], image_settings(ImageFormat::Jpg));
    relative_output.output_directory = Some(PathBuf::from("converted"));

    for batch in [relative_input, relative_output] {
        let events = batch.run().await;
        let finished = final_events(&events);
        assert_eq!(event_types(&finished), ["failed"]);
        assert_eq!(
            finished[0]["message"],
            "Only files with a full path can be converted"
        );
    }
    assert_eq!(std::fs::read_dir(&folder_path).unwrap().count(), 1);
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn paths_through_a_symlinked_folder_are_resolved_first() {
    let (_folder, real_folder) = resolved_temp_folder();
    let (_links, links_folder) = resolved_temp_folder();
    write_sample_png(&real_folder.join("photo.png"));
    let linked_folder = links_folder.join("linked");
    std::os::unix::fs::symlink(&real_folder, &linked_folder).unwrap();

    let events = TestBatch::new(
        vec![linked_folder.join("photo.png")],
        image_settings(ImageFormat::Jpg),
    )
    .run()
    .await;

    let finished = final_events(&events);
    assert_eq!(event_types(&finished), ["completed"]);
    let output_path = PathBuf::from(finished[0]["outputPath"].as_str().unwrap());
    assert_eq!(output_path, real_folder.join("photo.jpg"));
}

fn only_failure_message(events: &[Value]) -> String {
    let finished = final_events(events);
    assert_eq!(event_types(&finished), ["failed"], "events: {events:?}");
    let message = finished[0]["message"].as_str().unwrap().to_string();
    message
}

#[tokio::test(flavor = "multi_thread")]
async fn a_file_that_has_gone_is_reported_by_name() {
    let (_folder, folder_path) = resolved_temp_folder();

    let events = TestBatch::new(
        vec![folder_path.join("gone.png")],
        image_settings(ImageFormat::Jpg),
    )
    .run()
    .await;

    assert_eq!(
        only_failure_message(&events),
        "gone.png is no longer there. It may have been moved or deleted."
    );
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn a_read_only_folder_is_reported_by_name() {
    use std::os::unix::fs::PermissionsExt;

    let (_folder, folder_path) = resolved_temp_folder();
    let photo_path = folder_path.join("photo.png");
    write_sample_png(&photo_path);
    let read_only_folder = folder_path.join("Read only");
    std::fs::create_dir(&read_only_folder).unwrap();
    std::fs::set_permissions(&read_only_folder, std::fs::Permissions::from_mode(0o555)).unwrap();

    let mut batch = TestBatch::new(vec![photo_path], image_settings(ImageFormat::Jpg));
    batch.output_directory = Some(read_only_folder.clone());
    let events = batch.run().await;

    std::fs::set_permissions(&read_only_folder, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(
        only_failure_message(&events),
        "Can't write to Read only. Check the folder's permissions."
    );
}
