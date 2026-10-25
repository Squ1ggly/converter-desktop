use std::collections::HashSet;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::formats::{MediaKind, classify_path};

// Adding a huge folder by mistake, such as a home folder or a whole drive,
// stops here instead of reading every file on the disk.
pub const MAX_INSPECTED_FILES: usize = 10_000;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectedFile {
    pub path: PathBuf,
    name: String,
    size_bytes: u64,
    // None for files the app can't convert.
    pub kind: Option<MediaKind>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathInspection {
    pub files: Vec<InspectedFile>,
    // True when MAX_INSPECTED_FILES was reached and the remaining files were
    // left out without being counted.
    pub is_file_limit_reached: bool,
}

// Folders are searched recursively and their unsupported files skipped. Loose
// files are always returned so the UI can report unsupported ones. Paths are
// absolute and resolved, so the same file always has the same path.
pub fn inspect_paths(paths: Vec<PathBuf>) -> PathInspection {
    let mut collector = FileCollector::default();
    let mut pending_folders = Vec::new();

    for path in paths {
        let Some(resolved_path) = resolve_given_path(&path) else {
            continue;
        };
        if resolved_path.is_dir() {
            pending_folders.push(resolved_path);
        } else {
            collector.add(resolved_path);
        }
    }

    // A folder given twice, or inside another given folder, is read once.
    let mut visited_folders = HashSet::new();
    while let Some(folder) = pending_folders.pop() {
        if collector.is_full() {
            collector.is_file_limit_reached = true;
            break;
        }
        if !visited_folders.insert(folder.clone()) {
            continue;
        }

        let (subfolders, convertible_files) = split_folder_entries(&folder);
        pending_folders.extend(subfolders.into_iter().rev());
        convertible_files
            .into_iter()
            .for_each(|file_path| collector.add(file_path));
    }

    collector.into_inspection()
}

#[derive(Default)]
struct FileCollector {
    files: Vec<InspectedFile>,
    seen_paths: HashSet<PathBuf>,
    is_file_limit_reached: bool,
}

impl FileCollector {
    fn is_full(&self) -> bool {
        self.files.len() >= MAX_INSPECTED_FILES
    }

    fn add(&mut self, path: PathBuf) {
        if self.is_full() {
            self.is_file_limit_reached = true;
            return;
        }
        if !self.seen_paths.insert(path.clone()) {
            return;
        }
        if let Some(inspected_file) = inspect_file(path) {
            self.files.push(inspected_file);
        }
    }

    fn into_inspection(self) -> PathInspection {
        PathInspection {
            files: self.files,
            is_file_limit_reached: self.is_file_limit_reached,
        }
    }
}

// Paths come from the native file dialog and drag and drop, which are always
// absolute. Anything else is ignored rather than resolved against the app's
// working folder.
fn resolve_given_path(path: &Path) -> Option<PathBuf> {
    if !path.is_absolute() {
        log::warn!("ignored a path that isn't absolute: {}", path.display());
        return None;
    }
    // dunce avoids Windows' \\?\C:\ form, so paths match the ones the UI has.
    dunce::canonicalize(path).ok()
}

fn split_folder_entries(folder: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut subfolders = Vec::new();
    let mut convertible_files = Vec::new();

    for entry in list_visible_entries(folder) {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let entry_path = entry.path();

        if file_type.is_dir() {
            subfolders.push(entry_path);
        } else if classify_path(&entry_path).is_some() {
            convertible_files.extend(resolve_folder_file(&entry_path, file_type.is_symlink()));
        }
    }

    (subfolders, convertible_files)
}

// Symlinked folders are skipped, so a link to a home folder or to the root of
// the disk can't pull it in. A symlinked file is followed to the real file.
fn resolve_folder_file(entry_path: &Path, is_symlink: bool) -> Option<PathBuf> {
    if !is_symlink {
        return Some(entry_path.to_path_buf());
    }
    let target_path = dunce::canonicalize(entry_path).ok()?;
    if target_path.is_dir() {
        return None;
    }
    Some(target_path)
}

// Sorted by name. Hidden files are skipped, and so are names that aren't
// valid Unicode, which can't be sent to the UI.
fn list_visible_entries(folder: &Path) -> Vec<DirEntry> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };

    let mut visible_entries: Vec<DirEntry> = entries
        .filter_map(Result::ok)
        .filter(|entry| is_listable_name(&entry.file_name()))
        .collect();
    visible_entries.sort_by_key(DirEntry::file_name);
    visible_entries
}

fn is_listable_name(file_name: &std::ffi::OsStr) -> bool {
    let Some(file_name) = file_name.to_str() else {
        return false;
    };
    let is_hidden = file_name.starts_with('.');
    !is_hidden
}

fn inspect_file(path: PathBuf) -> Option<InspectedFile> {
    let metadata = std::fs::metadata(&path).ok()?;
    if !metadata.is_file() {
        return None;
    }
    let name = path.file_name()?.to_str()?.to_string();
    let kind = classify_path(&path);

    let inspected_file = InspectedFile {
        path,
        name,
        size_bytes: metadata.len(),
        kind,
    };
    Some(inspected_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names_of(inspection: &PathInspection) -> Vec<&str> {
        inspection
            .files
            .iter()
            .map(|file| file.name.as_str())
            .collect()
    }

    fn resolved_temp_folder() -> (tempfile::TempDir, PathBuf) {
        let folder = tempfile::tempdir().unwrap();
        let resolved_folder = dunce::canonicalize(folder.path()).unwrap();
        (folder, resolved_folder)
    }

    #[test]
    fn finds_convertible_files_in_nested_folders() {
        let (_root, root_path) = resolved_temp_folder();
        let nested = root_path.join("trip").join("day one");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("beach.jpg"), b"").unwrap();
        std::fs::write(nested.join("notes.txt"), b"").unwrap();
        std::fs::write(root_path.join(".hidden.png"), b"").unwrap();

        let inspection = inspect_paths(vec![root_path.clone()]);

        assert_eq!(names_of(&inspection), ["beach.jpg"]);
        assert!(!inspection.is_file_limit_reached);
    }

    #[test]
    fn a_file_added_directly_and_through_its_folder_is_listed_once() {
        let (_root, root_path) = resolved_temp_folder();
        let photo_path = root_path.join("photo.png");
        std::fs::write(&photo_path, b"").unwrap();

        let inspection = inspect_paths(vec![photo_path.clone(), root_path.clone()]);

        assert_eq!(inspection.files.len(), 1);
        assert_eq!(inspection.files[0].path, photo_path);
    }

    #[test]
    fn relative_paths_are_ignored() {
        let inspection = inspect_paths(vec![PathBuf::from("photo.png"), PathBuf::from(".")]);

        assert!(inspection.files.is_empty());
    }

    #[test]
    fn stops_at_the_file_limit_and_says_so() {
        let (_root, root_path) = resolved_temp_folder();
        for index in 0..=MAX_INSPECTED_FILES {
            std::fs::write(root_path.join(format!("{index}.png")), b"").unwrap();
        }

        let inspection = inspect_paths(vec![root_path.clone()]);

        assert_eq!(inspection.files.len(), MAX_INSPECTED_FILES);
        assert!(inspection.is_file_limit_reached);
    }

    #[cfg(windows)]
    #[test]
    fn windows_paths_are_returned_without_the_extended_length_prefix() {
        let (_root, root_path) = resolved_temp_folder();
        std::fs::write(root_path.join("photo.png"), b"").unwrap();

        let inspection = inspect_paths(vec![root_path.clone()]);

        let path_text = inspection.files[0].path.to_string_lossy().into_owned();
        assert!(!path_text.starts_with(r"\\?\"), "{path_text}");
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_folders_inside_a_folder_are_skipped() {
        let (_root, root_path) = resolved_temp_folder();
        let (_elsewhere, elsewhere_path) = resolved_temp_folder();
        std::fs::write(root_path.join("photo.png"), b"").unwrap();
        std::fs::write(elsewhere_path.join("outside.png"), b"").unwrap();
        std::os::unix::fs::symlink(&root_path, root_path.join("loop")).unwrap();
        std::os::unix::fs::symlink(&elsewhere_path, root_path.join("link")).unwrap();

        let inspection = inspect_paths(vec![root_path.clone()]);

        assert_eq!(names_of(&inspection), ["photo.png"]);
    }

    #[cfg(unix)]
    #[test]
    fn a_symlinked_file_is_listed_under_its_real_path() {
        let (_root, root_path) = resolved_temp_folder();
        let real_path = root_path.join("real.png");
        std::fs::write(&real_path, b"").unwrap();
        let link_folder = root_path.join("links");
        std::fs::create_dir(&link_folder).unwrap();
        std::os::unix::fs::symlink(&real_path, link_folder.join("photo.png")).unwrap();

        let inspection = inspect_paths(vec![link_folder]);

        assert_eq!(inspection.files[0].path, real_path);
    }

    // Linux allows any bytes in a name; such a name can't be sent to the UI and
    // used to make the whole folder fail to add.
    #[cfg(target_os = "linux")]
    #[test]
    fn names_that_are_not_unicode_are_skipped() {
        use std::os::unix::ffi::OsStrExt;

        let (_root, root_path) = resolved_temp_folder();
        std::fs::write(root_path.join("photo.png"), b"").unwrap();
        let invalid_name = std::ffi::OsStr::from_bytes(b"caf\xe9.png");
        std::fs::write(root_path.join(invalid_name), b"").unwrap();

        let inspection = inspect_paths(vec![root_path.clone()]);

        assert_eq!(names_of(&inspection), ["photo.png"]);
        assert!(serde_json::to_string(&inspection).is_ok());
    }
}
