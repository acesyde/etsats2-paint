//! Native file dialogs, behind a trait so tests can script them.

use std::collections::VecDeque;
use std::path::PathBuf;

pub trait FileDialogs {
    /// A `.truckpaint` file to open.
    fn open_project(&mut self) -> Option<PathBuf>;
    /// Where to save, proposing `suggested` as the file name.
    fn save_project(&mut self, suggested: &str) -> Option<PathBuf>;
    /// Images to place (Place…).
    fn pick_images(&mut self) -> Vec<PathBuf>;
}

/// The operating system's dialogs.
pub struct NativeDialogs;

impl FileDialogs for NativeDialogs {
    fn open_project(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Open Project")
            .add_filter("TruckPaint project", &[tp_file::EXTENSION])
            .pick_file()
    }

    fn save_project(&mut self, suggested: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Save Project")
            .add_filter("TruckPaint project", &[tp_file::EXTENSION])
            .set_file_name(suggested)
            .save_file()
    }

    fn pick_images(&mut self) -> Vec<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Place")
            .add_filter("Images", &["png", "jpg", "jpeg", "svg"])
            .pick_files()
            .unwrap_or_default()
    }
}

/// Dialogs answering from queues (tests). Empty queues mean "cancelled".
#[derive(Debug, Default)]
pub struct ScriptedDialogs {
    pub open: VecDeque<PathBuf>,
    pub save: VecDeque<PathBuf>,
    pub images: VecDeque<Vec<PathBuf>>,
    /// File names proposed by save dialogs, in order.
    pub suggested: Vec<String>,
}

impl FileDialogs for ScriptedDialogs {
    fn open_project(&mut self) -> Option<PathBuf> {
        self.open.pop_front()
    }

    fn save_project(&mut self, suggested: &str) -> Option<PathBuf> {
        self.suggested.push(suggested.to_owned());
        self.save.pop_front()
    }

    fn pick_images(&mut self) -> Vec<PathBuf> {
        self.images.pop_front().unwrap_or_default()
    }
}

/// Adds the `.truckpaint` extension when the user left it out.
pub fn with_extension(path: PathBuf) -> PathBuf {
    let has = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(tp_file::EXTENSION));
    if has {
        path
    } else {
        let mut name = path.into_os_string();
        name.push(".");
        name.push(tp_file::EXTENSION);
        PathBuf::from(name)
    }
}

/// File name proposed when saving a project named `name`.
pub fn suggested_file_name(name: &str) -> String {
    let safe: String = name
        .chars()
        .map(|c| if "/\\:*?\"<>|".contains(c) { '-' } else { c })
        .collect();
    format!("{}.{}", safe.trim(), tp_file::EXTENSION)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_and_suggested_name() {
        assert_eq!(
            with_extension("/a/ace".into()),
            PathBuf::from("/a/ace.truckpaint")
        );
        assert_eq!(
            with_extension("/a/ace.TruckPaint".into()),
            PathBuf::from("/a/ace.TruckPaint")
        );
        assert_eq!(
            suggested_file_name("ACE Logistics"),
            "ACE Logistics.truckpaint"
        );
        assert_eq!(suggested_file_name("A/B"), "A-B.truckpaint");
    }
}
