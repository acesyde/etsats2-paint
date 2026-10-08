//! Native file dialogs, behind a trait so tests can script them.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use tp_i18n::tr;

pub trait FileDialogs {
    /// A `.truckpaint` file to open.
    fn open_project(&mut self) -> Option<PathBuf>;
    /// Where to save, proposing `suggested` as the file name.
    fn save_project(&mut self, suggested: &str) -> Option<PathBuf>;
    /// Images to place (Place…).
    fn pick_images(&mut self) -> Vec<PathBuf>;
    /// Where to export a texture, proposing `suggested` (with its extension).
    fn save_export(&mut self, suggested: &str) -> Option<PathBuf>;
    /// Vehicle packages to install.
    fn pick_packages(&mut self) -> Vec<PathBuf>;
    /// Template images of a custom vehicle (PNG, DDS or SVG).
    fn pick_templates(&mut self) -> Vec<PathBuf>;
    /// Where to export a vehicle package, proposing `suggested`.
    fn save_package(&mut self, suggested: &str) -> Option<PathBuf>;
    /// Where to export a mod, proposing `suggested` in `folder`.
    fn save_mod(&mut self, suggested: &str, folder: Option<&Path>) -> Option<PathBuf>;
    /// A PNG or JPEG picture for the mod.
    fn pick_mod_image(&mut self) -> Option<PathBuf>;
    /// Names proposed so far (scripted dialogs only).
    fn suggested(&self) -> Vec<String> {
        Vec::new()
    }
}

/// The operating system's dialogs.
pub struct NativeDialogs;

impl FileDialogs for NativeDialogs {
    fn open_project(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title(tr("dialog-open-project"))
            .add_filter(tr("filter-project"), &[tp_file::EXTENSION])
            .pick_file()
    }

    fn save_project(&mut self, suggested: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title(tr("dialog-save-project"))
            .add_filter(tr("filter-project"), &[tp_file::EXTENSION])
            .set_file_name(suggested)
            .save_file()
    }

    fn pick_packages(&mut self) -> Vec<PathBuf> {
        rfd::FileDialog::new()
            .set_title(tr("cmd-vehicle-library"))
            .add_filter(tr("filter-packages"), &[tp_vehicles::EXTENSION])
            .pick_files()
            .unwrap_or_default()
    }

    fn pick_templates(&mut self) -> Vec<PathBuf> {
        rfd::FileDialog::new()
            .set_title(tr("custom-add-templates"))
            .add_filter(tr("filter-templates"), &["png", "dds", "svg"])
            .pick_files()
            .unwrap_or_default()
    }

    fn save_package(&mut self, suggested: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title(tr("dialog-export-package"))
            .add_filter(tr("filter-packages"), &[tp_vehicles::EXTENSION])
            .set_file_name(suggested)
            .save_file()
    }

    fn pick_images(&mut self) -> Vec<PathBuf> {
        rfd::FileDialog::new()
            .set_title(tr("cmd-place"))
            .add_filter(tr("filter-images"), &["png", "jpg", "jpeg", "svg"])
            .pick_files()
            .unwrap_or_default()
    }

    fn save_mod(&mut self, suggested: &str, folder: Option<&Path>) -> Option<PathBuf> {
        let mut dialog = rfd::FileDialog::new()
            .set_title(tr("dialog-export-mod"))
            .add_filter(tr("filter-mods"), &[crate::mod_export::EXTENSION])
            .set_file_name(suggested);
        if let Some(folder) = folder {
            dialog = dialog.set_directory(folder);
        }
        dialog.save_file()
    }

    fn pick_mod_image(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title(tr("mod-choose-image"))
            .add_filter(tr("filter-mod-images"), &["png", "jpg", "jpeg"])
            .pick_file()
    }

    fn save_export(&mut self, suggested: &str) -> Option<PathBuf> {
        let ext = std::path::Path::new(suggested)
            .extension()
            .map(|e| e.to_string_lossy().into_owned())
            .unwrap_or_default();
        rfd::FileDialog::new()
            .set_title(tr("dialog-export-texture"))
            .add_filter(ext.to_uppercase(), &[ext.as_str()])
            .set_file_name(suggested)
            .save_file()
    }
}

/// Dialogs answering from queues (tests). Empty queues mean "cancelled".
#[derive(Debug, Default)]
pub struct ScriptedDialogs {
    pub open: VecDeque<PathBuf>,
    pub save: VecDeque<PathBuf>,
    pub images: VecDeque<Vec<PathBuf>>,
    pub export: VecDeque<PathBuf>,
    pub packages: VecDeque<Vec<PathBuf>>,
    pub templates: VecDeque<Vec<PathBuf>>,
    pub package_save: VecDeque<PathBuf>,
    pub mod_save: VecDeque<PathBuf>,
    pub mod_images: VecDeque<PathBuf>,
    /// File names proposed by save dialogs, in order.
    pub suggested: Vec<String>,
    /// Folders the mod save dialogs opened in, in order.
    pub mod_folders: Vec<Option<PathBuf>>,
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

    fn save_export(&mut self, suggested: &str) -> Option<PathBuf> {
        self.suggested.push(suggested.to_owned());
        self.export.pop_front()
    }

    fn suggested(&self) -> Vec<String> {
        self.suggested.clone()
    }

    fn pick_packages(&mut self) -> Vec<PathBuf> {
        self.packages.pop_front().unwrap_or_default()
    }

    fn pick_templates(&mut self) -> Vec<PathBuf> {
        self.templates.pop_front().unwrap_or_default()
    }

    fn save_package(&mut self, suggested: &str) -> Option<PathBuf> {
        self.suggested.push(suggested.to_owned());
        self.package_save.pop_front()
    }

    fn save_mod(&mut self, suggested: &str, folder: Option<&Path>) -> Option<PathBuf> {
        self.suggested.push(suggested.to_owned());
        self.mod_folders.push(folder.map(Path::to_path_buf));
        self.mod_save.pop_front()
    }

    fn pick_mod_image(&mut self) -> Option<PathBuf> {
        self.mod_images.pop_front()
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
