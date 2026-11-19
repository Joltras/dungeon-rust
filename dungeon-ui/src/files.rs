//! Opening and saving floors as json files.

use std::fs;
use std::path::{Path, PathBuf};

use dungeon_generator::Floor;
use dungeon_generator::utils::constants::{DEFAULT_FLOOR_NAME, JSON_SUFFIX};

/// File name of a generated floor until it is saved under another name.
pub fn default_file_name() -> String {
    format!("{DEFAULT_FLOOR_NAME}{JSON_SUFFIX}")
}

/// Adds `.json` to the path if it has another or no extension.
pub fn with_json_suffix(path: PathBuf) -> PathBuf {
    if path.to_string_lossy().ends_with(JSON_SUFFIX) {
        return path;
    }
    let mut name = path.into_os_string();
    name.push(JSON_SUFFIX);
    PathBuf::from(name)
}

/// File name for texts shown to the user.
pub fn display_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

/// Reads a floor from a json file of the Python or Rust generator.
pub fn read_floor(path: &Path) -> Result<Floor, String> {
    let json = fs::read_to_string(path).map_err(|error| error.to_string())?;
    Floor::from_json(&json).map_err(|error| format!("It is not a valid floor file: {error}"))
}

/// Writes the floor to the file and creates its folder if needed.
pub fn write_floor(floor: &Floor, path: &Path) -> Result<(), String> {
    if let Some(folder) = path
        .parent()
        .filter(|folder| !folder.as_os_str().is_empty())
    {
        fs::create_dir_all(folder).map_err(|error| error.to_string())?;
    }
    fs::write(path, floor.to_json()).map_err(|error| error.to_string())
}

/// Lets the user choose a file. The window is passed so the dialog belongs to it.
pub trait FilePicker {
    fn pick_open(&self, window: &eframe::Frame, folder: &Path) -> Option<PathBuf>;
    fn pick_save(&self, window: &eframe::Frame, folder: &Path, name: &str) -> Option<PathBuf>;
}

/// File dialog of the operating system, on KDE and GNOME through the XDG portal.
pub struct NativeFilePicker;

impl NativeFilePicker {
    fn dialog(window: &eframe::Frame, folder: &Path, title: &str) -> rfd::FileDialog {
        let folder = std::path::absolute(folder).unwrap_or_else(|_| folder.to_path_buf());
        rfd::FileDialog::new()
            .set_title(title)
            .add_filter("Json", &["json"])
            .set_directory(folder)
            .set_parent(window)
    }
}

impl FilePicker for NativeFilePicker {
    fn pick_open(&self, window: &eframe::Frame, folder: &Path) -> Option<PathBuf> {
        Self::dialog(window, folder, "Open File").pick_file()
    }

    fn pick_save(&self, window: &eframe::Frame, folder: &Path, name: &str) -> Option<PathBuf> {
        Self::dialog(window, folder, "Save File")
            .set_file_name(name)
            .save_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dungeon_generator::Generator;

    #[test]
    fn adds_the_json_suffix_only_once() {
        assert_eq!(
            with_json_suffix("a/floor".into()),
            PathBuf::from("a/floor.json")
        );
        assert_eq!(
            with_json_suffix("a/floor.json".into()),
            PathBuf::from("a/floor.json")
        );
        assert_eq!(
            with_json_suffix("floor.v2".into()),
            PathBuf::from("floor.v2.json")
        );
    }

    #[test]
    fn writes_and_reads_a_floor_in_a_new_folder() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("generation").join("floor.json");
        let floor = Generator::new("abc", 3).generate().clone();
        write_floor(&floor, &path).unwrap();
        assert_eq!(read_floor(&path).unwrap().rooms(), floor.rooms());
        assert_eq!(display_name(&path), "floor.json");
    }

    #[test]
    fn reports_files_that_are_no_floors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("broken.json");
        fs::write(&path, "{}").unwrap();
        let error = read_floor(&path).unwrap_err();
        assert!(error.starts_with("It is not a valid floor file"), "{error}");
        assert!(read_floor(&dir.path().join("missing.json")).is_err());
    }
}
