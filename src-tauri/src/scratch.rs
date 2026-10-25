use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const SCRATCH_FOLDER_NAME: &str = "scratch";
const LAUNCH_FOLDER_PREFIX: &str = "launch-";

// Clears what earlier launches left, then makes a folder for this launch. A
// file that can't be deleted, such as one another process still has open on
// Windows, stays behind until a later launch instead of stopping this one.
pub fn prepare_scratch_root(cache_directory: &Path) -> PathBuf {
    let scratch_parent = cache_directory.join(SCRATCH_FOLDER_NAME);
    remove_earlier_scratch(&scratch_parent);

    match create_launch_folder(&scratch_parent) {
        Ok(launch_folder) => launch_folder,
        Err(error) => {
            log::warn!(
                "couldn't create a scratch folder in {}: {error}",
                scratch_parent.display()
            );
            std::env::temp_dir()
        }
    }
}

fn remove_earlier_scratch(scratch_parent: &Path) {
    let Err(error) = std::fs::remove_dir_all(scratch_parent) else {
        return;
    };
    if error.kind() != ErrorKind::NotFound {
        log::warn!(
            "couldn't clear the scratch folder {}: {error}",
            scratch_parent.display()
        );
    }
}

fn create_launch_folder(scratch_parent: &Path) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(scratch_parent)?;
    let launch_folder = tempfile::Builder::new()
        .prefix(LAUNCH_FOLDER_PREFIX)
        .tempdir_in(scratch_parent)?
        .keep();
    Ok(launch_folder)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_launch_gets_an_empty_folder_and_earlier_ones_are_cleared() {
        let cache_directory = tempfile::tempdir().unwrap();
        let first_launch = prepare_scratch_root(cache_directory.path());
        std::fs::write(first_launch.join("leftover.png"), b"").unwrap();

        let second_launch = prepare_scratch_root(cache_directory.path());

        assert!(!first_launch.exists());
        assert!(second_launch.is_dir());
        assert_eq!(std::fs::read_dir(&second_launch).unwrap().count(), 0);
    }

    // A folder without write permission can't be emptied, like a file still
    // open on Windows.
    #[cfg(unix)]
    #[test]
    fn a_leftover_that_cannot_be_removed_does_not_stop_the_launch() {
        use std::os::unix::fs::PermissionsExt;

        let cache_directory = tempfile::tempdir().unwrap();
        let locked_folder = cache_directory
            .path()
            .join(SCRATCH_FOLDER_NAME)
            .join("locked");
        std::fs::create_dir_all(&locked_folder).unwrap();
        std::fs::write(locked_folder.join("stuck.png"), b"").unwrap();
        std::fs::set_permissions(&locked_folder, std::fs::Permissions::from_mode(0o555)).unwrap();

        let launch_folder = prepare_scratch_root(cache_directory.path());

        std::fs::set_permissions(&locked_folder, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(launch_folder.is_dir());
        assert!(launch_folder.starts_with(cache_directory.path()));
    }
}
