//! The installed vehicle packages: stored unchanged as
//! `<root>/<id>/<version>.tpv`, listed from their manifests.

use std::path::{Path, PathBuf};

use tp_core::VehicleRef;
use tp_i18n::tr;
use tp_vehicles::{Manifest, Package, PackageError};

/// Id of the built-in sample vehicle.
pub const SAMPLE_ID: &str = "community.truckpaint.sample_truck";
/// File name of the built-in sample vehicle (its newest version).
pub const SAMPLE_FILE: &str = "community.truckpaint.sample_truck-1.1.0.tpv";
/// The built-in sample vehicle, offered while the library is empty.
pub const SAMPLE: &[u8] =
    include_bytes!("../../../examples/vehicles/community.truckpaint.sample_truck-1.1.0.tpv");

/// One installed version of a vehicle.
#[derive(Clone, Debug)]
pub struct InstalledVersion {
    pub manifest: Manifest,
    pub path: PathBuf,
}

/// An installed vehicle and its versions, newest first.
#[derive(Clone, Debug)]
pub struct InstalledVehicle {
    pub id: String,
    pub versions: Vec<InstalledVersion>,
}

impl InstalledVehicle {
    pub fn newest(&self) -> &InstalledVersion {
        &self.versions[0]
    }
}

/// Why a package could not be installed or loaded.
#[derive(Debug)]
pub enum InstallError {
    Package(PackageError),
    Io(std::io::Error),
    /// No library folder (no home directory).
    NoLibrary,
}

impl From<PackageError> for InstallError {
    fn from(e: PackageError) -> Self {
        InstallError::Package(e)
    }
}

impl From<std::io::Error> for InstallError {
    fn from(e: std::io::Error) -> Self {
        InstallError::Io(e)
    }
}

/// Why `file` could not be installed, in the current language.
pub fn install_error_message(err: &InstallError, file: &str) -> String {
    let reason = match err {
        InstallError::NoLibrary => tr("pkg-no-library"),
        InstallError::Io(e) => tr!("pkg-io", reason = e.to_string()),
        InstallError::Package(e) => match e {
            PackageError::NotAZip => tr("pkg-not-a-zip"),
            PackageError::NoManifest => tr("pkg-no-manifest"),
            PackageError::BadManifest(why) => tr!("pkg-bad-manifest", reason = why.as_str()),
            PackageError::NewerFormat(_) => tr("pkg-newer-format"),
            PackageError::BadId(id) => tr!("pkg-bad-id", id = id.as_str()),
            PackageError::NoVariant => tr("pkg-no-variant"),
            PackageError::EmptyVariant(v) => tr!("pkg-empty-variant", variant = v.as_str()),
            PackageError::DuplicateVariant(v) => {
                tr!("pkg-duplicate-variant", variant = v.as_str())
            }
            PackageError::DuplicateTexture { texture, .. } => {
                tr!("pkg-duplicate-texture", texture = texture.as_str())
            }
            PackageError::BadSize { texture, size } => {
                tr!("pkg-bad-size", texture = texture.as_str(), size = *size)
            }
            PackageError::UnsafePath(path) => tr!("pkg-unsafe-path", path = path.as_str()),
            PackageError::TooLarge => tr("pkg-too-large"),
            PackageError::MissingTemplate { texture, .. } => {
                tr!("pkg-missing-template", texture = texture.as_str())
            }
            PackageError::BadTemplate { texture, .. } => {
                tr!("pkg-bad-template", texture = texture.as_str())
            }
            PackageError::TemplateTooLarge { texture } => {
                tr!("pkg-template-too-large", texture = texture.as_str())
            }
        },
    };
    tr!("pkg-install-failed", file = file, reason = reason)
}

/// The installed packages.
#[derive(Debug, Default)]
pub struct VehicleLibrary {
    root: Option<PathBuf>,
    vehicles: Vec<InstalledVehicle>,
}

impl VehicleLibrary {
    /// The library stored in `root` (created on first install).
    pub fn open(root: &Path) -> Self {
        let mut library = Self {
            root: Some(root.to_path_buf()),
            vehicles: Vec::new(),
        };
        library.refresh();
        library
    }

    /// Installed vehicles, sorted by name.
    pub fn vehicles(&self) -> &[InstalledVehicle] {
        &self.vehicles
    }

    pub fn get(&self, id: &str) -> Option<&InstalledVehicle> {
        self.vehicles.iter().find(|v| v.id == id)
    }

    /// Lists the library folder again (unreadable files are skipped).
    pub fn refresh(&mut self) {
        self.vehicles.clear();
        let Some(root) = &self.root else {
            return;
        };
        let Ok(dirs) = std::fs::read_dir(root) else {
            return;
        };
        for dir in dirs.flatten().filter(|d| d.path().is_dir()) {
            let mut versions = Vec::new();
            for file in std::fs::read_dir(dir.path())
                .into_iter()
                .flatten()
                .flatten()
            {
                let path = file.path();
                if path.extension().is_none_or(|e| e != tp_vehicles::EXTENSION) {
                    continue;
                }
                match std::fs::read(&path)
                    .map_err(InstallError::from)
                    .and_then(|b| Package::read_manifest(&b).map_err(InstallError::from))
                {
                    Ok(manifest) => versions.push(InstalledVersion { manifest, path }),
                    Err(err) => tracing::warn!(path = %path.display(), ?err, "skipped package"),
                }
            }
            if versions.is_empty() {
                continue;
            }
            versions.sort_by(|a, b| b.manifest.version.cmp(&a.manifest.version));
            self.vehicles.push(InstalledVehicle {
                id: versions[0].manifest.id.clone(),
                versions,
            });
        }
        self.vehicles
            .sort_by_key(|v| v.newest().manifest.name.to_lowercase());
    }

    /// Validates and installs a package; returns its manifest. The same
    /// version is replaced; other versions are kept.
    pub fn install_bytes(&mut self, bytes: &[u8]) -> Result<Manifest, InstallError> {
        let root = self.root.clone().ok_or(InstallError::NoLibrary)?;
        let package = Package::read(bytes)?;
        let m = package.manifest;
        let dir = root.join(&m.id);
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.{}", m.version, tp_vehicles::EXTENSION));
        tp_file::write_atomic(&path, bytes).map_err(|e| match e {
            tp_file::Error::Io(e) => InstallError::Io(e),
            other => InstallError::Io(std::io::Error::other(other.to_string())),
        })?;
        self.refresh();
        Ok(m)
    }

    /// Installs the built-in sample vehicle.
    pub fn install_sample(&mut self) -> Result<Manifest, InstallError> {
        self.install_bytes(SAMPLE)
    }

    pub fn install_file(&mut self, path: &Path) -> Result<Manifest, InstallError> {
        let bytes = std::fs::read(path)?;
        self.install_bytes(&bytes)
    }

    /// Removes one installed version.
    pub fn remove(&mut self, id: &str, version: &semver::Version) -> std::io::Result<()> {
        if let Some(path) = self.path_of(id, version) {
            std::fs::remove_file(&path)?;
            if let Some(dir) = path.parent()
                && std::fs::read_dir(dir).is_ok_and(|mut d| d.next().is_none())
            {
                let _ = std::fs::remove_dir(dir);
            }
        }
        self.refresh();
        Ok(())
    }

    fn path_of(&self, id: &str, version: &semver::Version) -> Option<PathBuf> {
        self.get(id)?
            .versions
            .iter()
            .find(|v| v.manifest.version == *version)
            .map(|v| v.path.clone())
    }

    /// Reads an installed version completely (templates included).
    pub fn load(&self, id: &str, version: &semver::Version) -> Result<Package, InstallError> {
        let path = self
            .path_of(id, version)
            .ok_or_else(|| InstallError::Io(std::io::ErrorKind::NotFound.into()))?;
        Ok(Package::read(&std::fs::read(path)?)?)
    }

    /// The newest installed version newer than the project's, if it still
    /// has the project's variant.
    pub fn update_for(&self, vehicle: &VehicleRef) -> Option<&InstalledVersion> {
        let current = semver::Version::parse(&vehicle.version).ok()?;
        self.get(&vehicle.package_id)?.versions.iter().find(|v| {
            v.manifest.version > current && v.manifest.variant(&vehicle.variant_id).is_some()
        })
    }
}

#[cfg(test)]
mod tests {
    use tp_vehicles::sample;

    use super::*;

    fn pkg(version: &str) -> Vec<u8> {
        sample::package(
            "scs.sample.truck",
            "Sample Truck",
            version,
            &sample::truck_textures(),
        )
    }

    #[test]
    fn install_versions_and_remove() {
        let dir = tempfile::tempdir().unwrap();
        let mut lib = VehicleLibrary::open(dir.path());
        assert!(lib.vehicles().is_empty());
        lib.install_bytes(&pkg("1.2.0")).unwrap();
        lib.install_bytes(&pkg("1.2.0")).unwrap();
        lib.install_bytes(&pkg("1.3.0")).unwrap();
        let v = lib.get("scs.sample.truck").unwrap();
        let versions: Vec<String> = v
            .versions
            .iter()
            .map(|v| v.manifest.version.to_string())
            .collect();
        assert_eq!(
            versions,
            ["1.3.0", "1.2.0"],
            "newest first, reinstall replaces"
        );
        // Persisted: a new library sees the same.
        let lib2 = VehicleLibrary::open(dir.path());
        assert_eq!(lib2.get("scs.sample.truck").unwrap().versions.len(), 2);
        let package = lib
            .load("scs.sample.truck", &"1.2.0".parse().unwrap())
            .unwrap();
        assert_eq!(package.templates.len(), 3);
        lib.remove("scs.sample.truck", &"1.3.0".parse().unwrap())
            .unwrap();
        assert_eq!(lib.get("scs.sample.truck").unwrap().versions.len(), 1);
        lib.remove("scs.sample.truck", &"1.2.0".parse().unwrap())
            .unwrap();
        assert!(lib.vehicles().is_empty());
        assert!(!dir.path().join("scs.sample.truck").exists());
    }

    fn example(version: &str) -> Vec<u8> {
        let path = format!(
            "{}/../../examples/vehicles/community.truckpaint.sample_truck-{version}.tpv",
            env!("CARGO_MANIFEST_DIR")
        );
        std::fs::read(path).unwrap()
    }

    #[test]
    fn sample_vehicle_installs() {
        let dir = tempfile::tempdir().unwrap();
        let mut lib = VehicleLibrary::open(dir.path());
        let m = lib.install_sample().unwrap();
        assert_eq!(
            (m.id.as_str(), m.version.to_string().as_str()),
            ("community.truckpaint.sample_truck", "1.1.0")
        );
        assert_eq!(SAMPLE, example("1.1.0").as_slice());
        lib.install_bytes(&example("1.0.0")).unwrap();
        let v = lib.get("community.truckpaint.sample_truck").unwrap();
        assert_eq!(v.versions.len(), 2);
        assert_eq!(v.newest().manifest.name, "TruckPaint Sample Truck");
    }

    #[test]
    fn invalid_packages_are_not_installed_or_listed() {
        let dir = tempfile::tempdir().unwrap();
        let mut lib = VehicleLibrary::open(dir.path());
        let err = lib.install_bytes(b"nope").unwrap_err();
        assert!(matches!(err, InstallError::Package(PackageError::NotAZip)));
        assert!(install_error_message(&err, "x.tpv").contains("x.tpv"));
        // A damaged file dropped in the folder is skipped.
        std::fs::create_dir_all(dir.path().join("a.b")).unwrap();
        std::fs::write(dir.path().join("a.b/1.0.0.tpv"), b"junk").unwrap();
        lib.refresh();
        assert!(lib.vehicles().is_empty());
        let mut none = VehicleLibrary::default();
        assert!(matches!(
            none.install_bytes(&pkg("1.0.0")),
            Err(InstallError::NoLibrary)
        ));
    }

    #[test]
    fn updates_need_a_newer_version_with_the_variant() {
        let dir = tempfile::tempdir().unwrap();
        let mut lib = VehicleLibrary::open(dir.path());
        lib.install_bytes(&pkg("1.2.0")).unwrap();
        let mut vehicle = VehicleRef {
            package_id: "scs.sample.truck".into(),
            version: "1.2.0".into(),
            variant_id: "standard".into(),
            name: "Sample Truck".into(),
            brand: "Sample".into(),
            kind: "truck".into(),
            game: "ets2".into(),
        };
        assert!(lib.update_for(&vehicle).is_none());
        lib.install_bytes(&pkg("1.3.0")).unwrap();
        assert_eq!(
            lib.update_for(&vehicle)
                .unwrap()
                .manifest
                .version
                .to_string(),
            "1.3.0"
        );
        vehicle.variant_id = "other".into();
        assert!(lib.update_for(&vehicle).is_none());
    }
}
