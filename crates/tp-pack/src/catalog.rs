//! Official vehicles: building a catalog entry's package from the template
//! pack's files, checking the catalog against a pack (`tpv catalog check`)
//! and refreshing its game data from Paintjob Packer (`tpv catalog sync`).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Deserialize;
use tp_vehicles::catalog::{self, Cabin, Entry, GameData};
use tp_vehicles::{Game, ImageKind, TextureSize};

use crate::{PackError, Packed, pack_entries};

/// The pack's file that tells its version.
pub const PACK_INFO: &str = "Info and links.txt";

/// The folder holding the pack of `game`: `folder` itself or a folder
/// directly inside it, when it holds the folder of at least one official
/// vehicle of `game`.
pub fn find_pack(folder: &Path, game: Game) -> Option<PathBuf> {
    find_pack_of(folder, &catalog::entries(game))
}

/// [`find_pack`] for the vehicles `entries`.
pub fn find_pack_of(folder: &Path, entries: &[&Entry]) -> Option<PathBuf> {
    let holds = |dir: &Path| entries.iter().any(|e| vehicle_dir(dir, e).is_some());
    if holds(folder) {
        return Some(folder.to_path_buf());
    }
    let mut inside: Vec<PathBuf> = std::fs::read_dir(folder)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    inside.sort();
    inside.into_iter().find(|dir| holds(dir))
}

/// The entry `name` of `parent`: `parent/name` when it exists, else the
/// entry whose name is the same once folded (`Kögel` composed or
/// decomposed, as unzip tools differ).
fn resolve(parent: &Path, name: &str) -> Option<PathBuf> {
    let exact = parent.join(name);
    if exact.exists() {
        return Some(exact);
    }
    let folded = tp_vehicles::fold(name);
    std::fs::read_dir(parent)
        .ok()?
        .flatten()
        .find(|e| tp_vehicles::fold(&e.file_name().to_string_lossy()) == folded)
        .map(|e| e.path())
}

/// The file `file` (`/`-separated) under `dir`, each part resolved as
/// [`resolve`] does.
fn resolve_file(dir: &Path, file: &str) -> Option<PathBuf> {
    file.split('/')
        .try_fold(dir.to_path_buf(), |p, part| resolve(&p, part))
}

/// The folder of `entry` in the pack `pack`.
fn vehicle_dir(pack: &Path, entry: &Entry) -> Option<PathBuf> {
    resolve(pack, &entry.folder).filter(|d| d.is_dir())
}

/// Why an official vehicle could not be built from the pack. Each names
/// the vehicle (its catalog name) and, for a file, its path relative to
/// the vehicle's folder in the pack (`Accessories/Sun Visor.png`).
#[derive(Debug)]
pub enum CatalogError {
    /// The pack has no folder for the vehicle.
    NoVehicleFolder {
        vehicle: String,
        folder: PathBuf,
    },
    MissingFile {
        vehicle: String,
        file: String,
    },
    /// The file's size in pixels differs from the entry's: the pack may be
    /// for another game version.
    WrongSize {
        vehicle: String,
        file: String,
        expected: TextureSize,
        actual: (u32, u32),
    },
    /// The file can't be read or isn't a PNG.
    Unreadable {
        vehicle: String,
        file: String,
        reason: String,
    },
    /// Stopped by the caller.
    Cancelled,
    /// The package was built but does not pass validation.
    Pack(PackError),
}

impl std::fmt::Display for CatalogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CatalogError::NoVehicleFolder { vehicle, folder } => {
                write!(f, "{vehicle}: no folder {}", folder.display())
            }
            CatalogError::MissingFile { vehicle, file } => {
                write!(f, "{vehicle}: \"{file}\" is missing")
            }
            CatalogError::WrongSize {
                vehicle,
                file,
                expected,
                actual,
            } => write!(
                f,
                "{vehicle}: \"{file}\" is {} × {} instead of {} (the pack may be for another game version)",
                actual.0,
                actual.1,
                expected.label()
            ),
            CatalogError::Unreadable {
                vehicle,
                file,
                reason,
            } => write!(f, "{vehicle}: \"{file}\" can't be read: {reason}"),
            CatalogError::Cancelled => f.write_str("cancelled"),
            CatalogError::Pack(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for CatalogError {}

/// The pixel size of a PNG, from its header.
fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    let (w, h) = tp_vehicles::image_size(ImageKind::Png, bytes)?;
    Some((w as u32, h as u32))
}

/// Reads texture file `file` of `entry` from its folder `dir`, checking
/// it is a PNG of `size`.
fn read_texture(
    entry: &Entry,
    dir: &Path,
    file: &str,
    size: TextureSize,
) -> Result<Vec<u8>, CatalogError> {
    let Some(path) = resolve_file(dir, file).filter(|p| p.is_file()) else {
        return Err(CatalogError::MissingFile {
            vehicle: entry.name.clone(),
            file: file.to_owned(),
        });
    };
    let unreadable = |reason: String| CatalogError::Unreadable {
        vehicle: entry.name.clone(),
        file: file.to_owned(),
        reason,
    };
    let bytes = std::fs::read(&path).map_err(|e| unreadable(e.to_string()))?;
    let actual = png_size(&bytes).ok_or_else(|| unreadable("not a PNG image".into()))?;
    if actual != (size.width, size.height) {
        return Err(CatalogError::WrongSize {
            vehicle: entry.name.clone(),
            file: file.to_owned(),
            expected: size,
            actual,
        });
    }
    Ok(bytes)
}

/// Builds the package of `entry` from the pack folder `pack` (see
/// [`find_pack`]). Nothing is written to disk.
pub fn build(entry: &Entry, pack: &Path) -> Result<Packed, CatalogError> {
    build_with(entry, pack, &mut |_, _| {}, &AtomicBool::new(false))
}

/// [`build`], calling `progress(done, total)` after each file is read
/// (`total` is the number of files) and stopping with
/// [`CatalogError::Cancelled`] once `cancel` is set.
pub fn build_with(
    entry: &Entry,
    pack: &Path,
    progress: &mut dyn FnMut(usize, usize),
    cancel: &AtomicBool,
) -> Result<Packed, CatalogError> {
    let Some(dir) = vehicle_dir(pack, entry) else {
        return Err(CatalogError::NoVehicleFolder {
            vehicle: entry.name.clone(),
            folder: pack.join(&entry.folder),
        });
    };
    let total = entry.textures().count();
    let mut files = BTreeMap::new();
    for (done, t) in entry.textures().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            return Err(CatalogError::Cancelled);
        }
        let bytes = read_texture(entry, &dir, t.file, t.size)?;
        files.insert(Entry::template_path(t.id), bytes);
        progress(done + 1, total);
    }
    if cancel.load(Ordering::Relaxed) {
        return Err(CatalogError::Cancelled);
    }
    pack_entries(catalog::manifest(entry), files).map_err(CatalogError::Pack)
}

/// The pack's version from its `Info and links.txt` ("current for game
/// version 1.59, updated 2026-05-28" gives "1.59 (2026-05-28)").
pub fn pack_version(pack: &Path) -> Option<String> {
    let text = std::fs::read_to_string(pack.join(PACK_INFO)).ok()?;
    let after = |key: &str| {
        let start = text.find(key)? + key.len();
        let word: String = text[start..]
            .chars()
            .take_while(|c| !c.is_whitespace() && *c != ',')
            .collect();
        let word = word.trim_end_matches('.').to_owned();
        (!word.is_empty()).then_some(word)
    };
    let version = after("game version ")?;
    Some(match after("updated ") {
        Some(date) => format!("{version} ({date})"),
        None => version,
    })
}

/// What `tpv catalog check` found.
#[derive(Debug, Default)]
pub struct CheckReport {
    /// The pack folder checked (the nested folder when found inside).
    pub pack: PathBuf,
    /// The checked vehicles' names.
    pub checked: Vec<String>,
    /// Problems, each starting with the vehicle's name.
    pub problems: Vec<String>,
    /// PNG files of the checked vehicles' folders that no checked entry
    /// using that folder uses, relative to the pack
    /// (`Krone Cool Liner/Accessories/…png`).
    pub unused: Vec<String>,
    /// The pack's version, when its `Info and links.txt` tells it.
    pub pack_version: Option<String>,
    /// The vehicles validated with another pack version: (name, version).
    pub other_version: Vec<(String, String)>,
}

/// PNG files under `dir`, relative to `root`, `/`-separated. macOS
/// resource files (`._Sun Visor.png`) are skipped.
pub(crate) fn pngs(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in read.flatten() {
        let path = entry.path();
        if entry.file_name().to_string_lossy().starts_with("._") {
            continue;
        }
        if path.is_dir() {
            pngs(root, &path, out);
        } else if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("png"))
            && let Ok(relative) = path.strip_prefix(root)
        {
            let parts: Vec<String> = relative
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            out.push(parts.join("/"));
        }
    }
}

/// Checks `entries` against the pack folder `pack` (or the pack found
/// inside it): folders, files and sizes, and each entry's completeness.
pub fn check(pack: &Path, entries: &[&Entry]) -> CheckReport {
    let pack = find_pack_of(pack, entries).unwrap_or_else(|| pack.to_path_buf());
    let mut report = CheckReport {
        pack_version: pack_version(&pack),
        ..CheckReport::default()
    };
    for entry in entries {
        let name = &entry.name;
        report.checked.push(name.clone());
        for p in catalog::completeness(entry) {
            report.problems.push(format!("{name}: {p}"));
        }
        if let Some(version) = &report.pack_version
            && *version != entry.source.pack
        {
            report
                .other_version
                .push((name.clone(), entry.source.pack.clone()));
        }
        let Some(dir) = vehicle_dir(&pack, entry) else {
            report.problems.push(format!(
                "{name}: no folder \"{}\" in the pack",
                entry.folder
            ));
            continue;
        };
        for t in entry.textures() {
            if let Err(e) = read_texture(entry, &dir, t.file, t.size) {
                report.problems.push(e.to_string());
            }
        }
    }
    // Unused files, once per folder: a shared folder's file is used when
    // any entry using that folder uses it.
    let mut folders: Vec<&str> = Vec::new();
    for entry in entries {
        if folders
            .iter()
            .any(|f| tp_vehicles::fold(f) == tp_vehicles::fold(&entry.folder))
        {
            continue;
        }
        folders.push(&entry.folder);
        let Some(dir) = vehicle_dir(&pack, entry) else {
            continue;
        };
        let used: Vec<String> = entries
            .iter()
            .filter(|e| tp_vehicles::fold(&e.folder) == tp_vehicles::fold(&entry.folder))
            .flat_map(|e| e.textures().map(|t| tp_vehicles::fold(t.file)))
            .collect();
        let mut found = Vec::new();
        pngs(&dir, &dir, &mut found);
        found.sort();
        for file in found {
            if !used.contains(&tp_vehicles::fold(&file)) {
                report.unused.push(format!("{}/{file}", entry.folder));
            }
        }
    }
    report.pack = pack;
    report
}

impl CheckReport {
    /// The report as `tpv catalog check` prints it.
    pub fn text(&self) -> String {
        let mut out = format!("pack: {}\n", self.pack.display());
        match &self.pack_version {
            Some(v) => out.push_str(&format!("pack version: {v}\n")),
            None => out.push_str(&format!(
                "pack version: unknown (no readable {PACK_INFO})\n"
            )),
        }
        out.push_str(&format!("checked: {}\n", self.checked.join(", ")));
        for p in &self.problems {
            out.push_str(&format!("problem: {p}\n"));
        }
        for file in &self.unused {
            out.push_str(&format!("unused:  {file}\n"));
        }
        for (name, version) in &self.other_version {
            out.push_str(&format!(
                "info:    {name} was validated with the pack {version}\n"
            ));
        }
        if self.problems.is_empty() {
            out.push_str("no problem\n");
        } else {
            out.push_str(&format!("{} problem(s)\n", self.problems.len()));
        }
        out
    }
}

/// The catalog entries under `dir` (`<game>/<path>.json`), sorted by path.
pub fn read_catalog(dir: &Path) -> Result<Vec<(PathBuf, Entry)>, String> {
    let mut out = Vec::new();
    for game in Game::ALL {
        let Ok(read) = std::fs::read_dir(dir.join(game.code())) else {
            continue;
        };
        let mut paths: Vec<PathBuf> = read
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "json"))
            .collect();
        paths.sort();
        for path in paths {
            let text =
                std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let entry = Entry::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
            out.push((path, entry));
        }
    }
    Ok(out)
}

/// A vehicle of Paintjob Packer (the fields the catalog uses).
#[derive(Deserialize)]
struct PpkVehicle {
    name: String,
    path: String,
    #[serde(rename = "mod")]
    is_mod: bool,
    #[serde(default)]
    alt_uv: bool,
    #[serde(default)]
    colour_picker: bool,
    #[serde(default)]
    cabins: Vec<PpkCabin>,
    #[serde(default)]
    accessories: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct PpkCabin {
    code: String,
    name: String,
    internal_name: Vec<String>,
}

impl PpkVehicle {
    fn game_data(&self) -> GameData {
        GameData {
            alt_uv: self.alt_uv,
            colour_picker: self.colour_picker,
            cabins: self
                .cabins
                .iter()
                .map(|c| Cabin {
                    code: c.code.clone(),
                    name: c.name.clone(),
                    internal_names: c.internal_name.clone(),
                })
                .collect(),
            accessories: self.accessories.clone(),
        }
    }
}

/// Paintjob Packer's folder of the vehicles of `game`.
fn ppk_game_folder(game: Game) -> &'static str {
    match game {
        Game::Ets2 => "ets",
        Game::Ats => "ats",
    }
}

/// The official vehicles (`mod: false`) of Paintjob Packer for `game`, by
/// path. Every JSON file of the game's vehicles must be readable.
fn ppk_official(ppk: &Path, game: Game) -> Result<BTreeMap<String, PpkVehicle>, String> {
    #[derive(Deserialize)]
    struct Head {
        #[serde(rename = "mod")]
        is_mod: bool,
    }
    let root = ppk.join("vehicles").join(ppk_game_folder(game));
    let mut out = BTreeMap::new();
    let authors = std::fs::read_dir(&root).map_err(|e| format!("{}: {e}", root.display()))?;
    for author in authors.flatten().map(|e| e.path()).filter(|p| p.is_dir()) {
        let files = std::fs::read_dir(&author).map_err(|e| format!("{}: {e}", author.display()))?;
        for path in files
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "json"))
        {
            let text =
                std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let bad = |e: serde_json::Error| format!("{}: {e}", path.display());
            if serde_json::from_str::<Head>(&text).map_err(bad)?.is_mod {
                continue;
            }
            let vehicle: PpkVehicle = serde_json::from_str(&text).map_err(bad)?;
            if !vehicle.is_mod {
                out.insert(vehicle.path.clone(), vehicle);
            }
        }
    }
    Ok(out)
}

/// What `tpv catalog sync` changed in an entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    CabinAdded(String),
    CabinRemoved(String),
    AccessoryAdded(String),
    AccessoryRemoved(String),
    AltUv(bool),
    ColourPicker(bool),
    /// Other changes to the game data (names, groups).
    Other,
}

impl std::fmt::Display for Change {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Change::CabinAdded(id) => write!(f, "gained cabin {id}"),
            Change::CabinRemoved(id) => write!(f, "lost cabin {id}"),
            Change::AccessoryAdded(id) => write!(f, "gained {id}"),
            Change::AccessoryRemoved(id) => write!(f, "lost {id}"),
            Change::AltUv(on) => write!(f, "alt_uv is now {on}"),
            Change::ColourPicker(on) => write!(f, "colour_picker is now {on}"),
            Change::Other => f.write_str("names or groups changed"),
        }
    }
}

/// The changes from `old` to `new`.
fn changes(old: &GameData, new: &GameData) -> Vec<Change> {
    let diff = |a: BTreeSet<&String>, b: BTreeSet<&String>| -> (Vec<String>, Vec<String>) {
        (
            b.difference(&a).map(|s| (*s).clone()).collect(),
            a.difference(&b).map(|s| (*s).clone()).collect(),
        )
    };
    let mut out = Vec::new();
    let (added, removed) = diff(old.cabin_names().collect(), new.cabin_names().collect());
    out.extend(added.into_iter().map(Change::CabinAdded));
    out.extend(removed.into_iter().map(Change::CabinRemoved));
    let (added, removed) = diff(old.accessory_ids().collect(), new.accessory_ids().collect());
    out.extend(added.into_iter().map(Change::AccessoryAdded));
    out.extend(removed.into_iter().map(Change::AccessoryRemoved));
    if old.alt_uv != new.alt_uv {
        out.push(Change::AltUv(new.alt_uv));
    }
    if old.colour_picker != new.colour_picker {
        out.push(Change::ColourPicker(new.colour_picker));
    }
    if out.is_empty() && old != new {
        out.push(Change::Other);
    }
    out
}

/// What `tpv catalog sync` did.
#[derive(Debug, Default)]
pub struct SyncReport {
    /// Each entry's name and changes (none: unchanged).
    pub entries: Vec<(String, Vec<Change>)>,
    /// Official vehicles of Paintjob Packer the catalog doesn't have:
    /// (game, name, path).
    pub not_in_catalog: Vec<(Game, String, String)>,
}

impl SyncReport {
    /// The report as `tpv catalog sync` prints it.
    pub fn text(&self) -> String {
        let mut out = String::new();
        for (name, changes) in &self.entries {
            if changes.is_empty() {
                out.push_str(&format!("{name}: unchanged\n"));
            }
            for c in changes {
                out.push_str(&format!("{name}: {c}\n"));
            }
        }
        for (game, name, path) in &self.not_in_catalog {
            out.push_str(&format!(
                "not in the catalog: {name} ({}, {path})\n",
                game.code()
            ));
        }
        out
    }
}

/// Replaces the game data of each entry of the catalog folder `catalog_dir`
/// with that of the Paintjob Packer copy `ppk` at commit `commit`, and
/// records the commit. Nothing else of an entry changes. When an entry's
/// vehicle is missing from Paintjob Packer or a file can't be read, no
/// entry is written.
pub fn sync(ppk: &Path, commit: &str, catalog_dir: &Path) -> Result<SyncReport, String> {
    let entries = read_catalog(catalog_dir)?;
    let mut official = BTreeMap::new();
    for game in Game::ALL {
        if ppk.join("vehicles").join(ppk_game_folder(game)).is_dir() {
            official.insert(game, ppk_official(ppk, game)?);
        }
    }
    let mut report = SyncReport::default();
    let mut updated = Vec::new();
    for (path, mut entry) in entries {
        let vehicle = official
            .get(&entry.game)
            .and_then(|v| v.get(&entry.path))
            .ok_or_else(|| {
                format!(
                    "{} ({}, {}) is not an official vehicle of Paintjob Packer",
                    entry.name,
                    entry.game.code(),
                    entry.path
                )
            })?;
        let data = vehicle.game_data();
        report
            .entries
            .push((entry.name.clone(), changes(&entry.game_data, &data)));
        entry.game_data = data;
        entry.source.paintjob_packer = commit.to_owned();
        updated.push((path, entry));
    }
    for (game, vehicles) in &official {
        for (path, v) in vehicles {
            if !updated
                .iter()
                .any(|(_, e)| e.game == *game && e.path == *path)
            {
                report
                    .not_in_catalog
                    .push((*game, v.name.clone(), path.clone()));
            }
        }
    }
    for (path, entry) in &updated {
        std::fs::write(path, entry.to_json()).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(report)
}

#[cfg(test)]
mod tests;
