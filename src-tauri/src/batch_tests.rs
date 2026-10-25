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
use crate::formats::{ImageFormat, VideoFormat};
use crate::options::{ConversionSettings, ImageOptions, ResizeOption, VideoOptions, VideoQuality};
use crate::state::ProducedOutputs;

type RecordedEvents = Arc<Mutex<Vec<Value>>>;

struct TestBatch {
    files: Vec<PathBuf>,
    settings: ConversionSettings,
    output_directory: Option<PathBuf>,
    cancellation: CancellationToken,
    is_cancelled_on_first_progress: bool,
}

impl TestBatch {
    fn new(files: Vec<PathBuf>, settings: ConversionSettings) -> TestBatch {
        TestBatch {
            files,
            settings,
            output_directory: None,
            cancellation: CancellationToken::new(),
            is_cancelled_on_first_progress: false,
        }
    }

    // Returns every event sent, in order.
    async fn run(self) -> Vec<Value> {
        let scratch_root = tempfile::tempdir().unwrap();
        let cancel_on_progress = self
            .is_cancelled_on_first_progress
            .then(|| self.cancellation.clone());
        let (events, recorded_events) = recording_channel(cancel_on_progress);
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

// Cancels `cancel_on_progress` once a file reports progress, which means
// FFmpeg is running and writing the output.
fn recording_channel(
    cancel_on_progress: Option<CancellationToken>,
) -> (Channel<ConversionEvent>, RecordedEvents) {
    let recorded_events = RecordedEvents::default();
    let event_sink = recorded_events.clone();
    let channel = Channel::new(move |body| {
        let InvokeResponseBody::Json(json) = body else {
            return Ok(());
        };
        let event: Value = serde_json::from_str(&json).unwrap();
        if let (Some(cancellation), "progress") =
            (&cancel_on_progress, event["type"].as_str().unwrap())
        {
            cancellation.cancel();
        }
        event_sink.lock().unwrap().push(event);
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

fn video_settings(format: VideoFormat) -> ConversionSettings {
    ConversionSettings::Video {
        format,
        options: VideoOptions {
            quality: VideoQuality::Balanced,
            max_height: None,
            frame_rate: None,
        },
    }
}

fn file_names_in(folder: &Path) -> Vec<String> {
    let mut file_names: Vec<String> = std::fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    file_names.sort();
    file_names
}

fn assert_no_empty_files(folder: &Path) {
    for entry in std::fs::read_dir(folder).unwrap() {
        let entry = entry.unwrap();
        let size_bytes = entry.metadata().unwrap().len();
        assert!(size_bytes > 0, "{} is empty", entry.path().display());
    }
}

async fn make_clip(folder: &Path, file_name: &str, duration_seconds: u32) -> PathBuf {
    let clip_path = folder.join(file_name);
    let source = format!("testsrc=duration={duration_seconds}:size=640x360:rate=30");
    let arguments: Vec<std::ffi::OsString> = [
        "-f",
        "lavfi",
        "-i",
        source.as_str(),
        "-c:v",
        "libx264",
        "-preset",
        "ultrafast",
    ]
    .iter()
    .map(std::ffi::OsString::from)
    .chain([clip_path.clone().into_os_string()])
    .collect();
    Ffmpeg::from_downloaded_binary()
        .run(arguments, &CancellationToken::new(), |_| {})
        .await
        .expect("make a test clip");
    clip_path
}

#[tokio::test(flavor = "multi_thread")]
async fn files_with_the_same_name_get_numbered_outputs_in_parallel() {
    let (_folder, folder_path) = resolved_temp_folder();
    let inputs: Vec<PathBuf> = ["png", "bmp", "gif", "tiff", "webp"]
        .iter()
        .map(|extension| {
            let input_path = folder_path.join(format!("photo.{extension}"));
            let format = image::ImageFormat::from_extension(extension).unwrap();
            RgbaImage::from_pixel(16, 12, Rgba([200, 30, 30, 255]))
                .save_with_format(&input_path, format)
                .unwrap();
            input_path
        })
        .collect();

    let events = TestBatch::new(inputs, image_settings(ImageFormat::Jpg))
        .run()
        .await;

    let finished = final_events(&events);
    assert_eq!(event_types(&finished), ["completed"; 5]);
    let output_names: Vec<String> = file_names_in(&folder_path)
        .into_iter()
        .filter(|name| {
            Path::new(name)
                .extension()
                .is_some_and(|extension| extension == "jpg")
        })
        .collect();
    assert_eq!(
        output_names,
        [
            "photo (2).jpg",
            "photo (3).jpg",
            "photo (4).jpg",
            "photo (5).jpg",
            "photo.jpg"
        ]
    );
    assert_no_empty_files(&folder_path);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_batch_cancelled_before_it_starts_writes_nothing() {
    let (_folder, folder_path) = resolved_temp_folder();
    let photo_path = folder_path.join("photo.png");
    write_sample_png(&photo_path);
    let clip_path = make_clip(&folder_path, "clip.mp4", 1).await;

    for (input_path, settings) in [
        (photo_path, image_settings(ImageFormat::Webp)),
        (clip_path, video_settings(VideoFormat::Mkv)),
    ] {
        let batch = TestBatch::new(vec![input_path], settings);
        batch.cancellation.cancel();

        let events = batch.run().await;

        assert_eq!(event_types(&final_events(&events)), ["cancelled"]);
    }
    assert_eq!(file_names_in(&folder_path), ["clip.mp4", "photo.png"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn cancelling_a_running_video_removes_its_partial_output() {
    let (_folder, folder_path) = resolved_temp_folder();
    let clip_path = make_clip(&folder_path, "clip.mp4", 20).await;
    let waiting_clip_path = folder_path.join("waiting.mp4");
    std::fs::copy(&clip_path, &waiting_clip_path).unwrap();

    let mut batch = TestBatch::new(
        vec![clip_path, waiting_clip_path],
        video_settings(VideoFormat::Webm),
    );
    batch.is_cancelled_on_first_progress = true;
    let events = batch.run().await;

    let finished = final_events(&events);
    assert_eq!(event_types(&finished), ["cancelled", "cancelled"]);
    assert_eq!(file_names_in(&folder_path), ["clip.mp4", "waiting.mp4"]);
}

// Each message must be short and plain: no FFmpeg log prefixes, memory
// addresses, operating system error codes, folder paths or panics.
fn assert_clean_message(message: &str, folder_path: &Path) {
    let folder_text = folder_path.to_string_lossy();
    let unwanted_fragments = ["panicked", "os error", " @ 0x", "[", folder_text.as_ref()];
    for fragment in unwanted_fragments {
        assert!(
            !message.contains(fragment),
            "{message:?} contains {fragment:?}"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn empty_corrupt_and_truncated_files_fail_cleanly() {
    let (_folder, folder_path) = resolved_temp_folder();
    let real_png_path = folder_path.join("real.png");
    RgbaImage::from_fn(64, 64, |x, y| {
        Rgba([u8::try_from(x).unwrap(), u8::try_from(y).unwrap(), 90, 255])
    })
    .save(&real_png_path)
    .unwrap();
    let real_png = std::fs::read(&real_png_path).unwrap();
    let real_clip = std::fs::read(make_clip(&folder_path, "real.mp4", 1).await).unwrap();

    let broken_images = [
        ("empty.png", Vec::new()),
        ("noise.jpg", b"this is not an image at all".repeat(20)),
        ("truncated.png", real_png[..real_png.len() / 2].to_vec()),
        ("empty.heic", Vec::new()),
        ("noise.avif", b"not an avif".repeat(20)),
        ("empty.svg", Vec::new()),
        ("noise.pdf", b"%PDF-1.7 broken".to_vec()),
    ];
    let broken_videos = [
        ("empty.mov", Vec::new()),
        ("noise.mkv", b"not a video".repeat(50)),
        ("truncated.mp4", real_clip[..real_clip.len() / 3].to_vec()),
    ];

    let image_paths = write_files(&folder_path, &broken_images);
    let video_paths = write_files(&folder_path, &broken_videos);
    let names_before = file_names_in(&folder_path);

    let image_events = TestBatch::new(image_paths, image_settings(ImageFormat::Png))
        .run()
        .await;
    let video_events = TestBatch::new(video_paths, video_settings(VideoFormat::Mp4))
        .run()
        .await;

    for events in [image_events, video_events] {
        for finished_event in final_events(&events) {
            assert_eq!(finished_event["type"], "failed", "{finished_event}");
            assert_clean_message(finished_event["message"].as_str().unwrap(), &folder_path);
        }
    }
    assert_eq!(file_names_in(&folder_path), names_before);
}

fn write_files(folder: &Path, files: &[(&str, Vec<u8>)]) -> Vec<PathBuf> {
    let paths = files
        .iter()
        .map(|(file_name, bytes)| {
            let file_path = folder.join(file_name);
            std::fs::write(&file_path, bytes).unwrap();
            file_path
        })
        .collect();
    paths
}
