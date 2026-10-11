//! The catalog of official vehicles: for each vehicle of the base games,
//! its game data (from Paintjob Packer by Carsmaniac) and its mapping to the
//! files of Carsmaniac's template pack. The entries are JSON files under
//! `catalog/<game>/<path>.json` in the repository, embedded at build time;
//! a package is built from an entry and the pack's files on the player's
//! computer (`tp_pack::catalog`).
//!
//! `game_data` is what `tpv catalog sync` rewrites; `main`, `accessories`
//! and `left_out` are made by hand.

use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{Game, Kind, TextureSize};

/// Highest catalog entry format this version understands.
pub const FORMAT: u32 = 1;
/// First word of the package id of an official vehicle.
pub const ID_PREFIX: &str = "official.";

/// The embedded entries: path under `catalog/`, and JSON.
pub const ENTRIES: &[(&str, &str)] = &[
    (
        "ets2/daf.2021.json",
        include_str!("../../../catalog/ets2/daf.2021.json"),
    ),
    (
        "ets2/krone.coolliner.json",
        include_str!("../../../catalog/ets2/krone.coolliner.json"),
    ),
];

/// Paintjob Packer's license (MIT), whose data the entries hold.
pub const PAINTJOB_PACKER_LICENSE: &str =
    include_str!("../../../catalog/LICENSE-paintjob-packer.txt");
/// File name of [`PAINTJOB_PACKER_LICENSE`] under `catalog/`.
pub const PAINTJOB_PACKER_LICENSE_FILE: &str = "LICENSE-paintjob-packer.txt";

/// One official vehicle.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub format: u32,
    pub game: Game,
    /// The vehicle's path in the game's definitions (`daf.2021`).
    pub path: String,
    pub name: String,
    pub brand: String,
    pub kind: Kind,
    /// The package version, raised by hand when the mapping or a template
    /// changes.
    pub version: semver::Version,
    /// The game versions the mapping is made for.
    pub game_versions: semver::VersionReq,
    /// The vehicle's folder in the template pack.
    pub folder: String,
    pub source: Source,
    pub game_data: GameData,
    pub main: Vec<MainTexture>,
    #[serde(default)]
    pub accessories: Vec<AccessoryTexture>,
    #[serde(default)]
    pub left_out: Vec<LeftOut>,
}

/// Where an entry's data comes from.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    /// The Paintjob Packer commit `game_data` comes from.
    pub paintjob_packer: String,
    /// The template pack version the mapping was validated with
    /// (`1.59 (2026-05-28)`).
    pub pack: String,
}

/// The vehicle as Paintjob Packer describes it.
#[derive(Clone, Debug, PartialEq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameData {
    pub alt_uv: bool,
    pub colour_picker: bool,
    pub cabins: Vec<Cabin>,
    /// Accessory ids by Paintjob Packer group.
    pub accessories: BTreeMap<String, Vec<String>>,
}

impl GameData {
    /// Every cabin internal name, in order.
    pub fn cabin_names(&self) -> impl Iterator<Item = &String> {
        self.cabins.iter().flat_map(|c| &c.internal_names)
    }

    /// Every accessory id, by group then in group order.
    pub fn accessory_ids(&self) -> impl Iterator<Item = &String> {
        self.accessories.values().flatten()
    }
}

/// A cabin: its code, its name and the internal names sharing it.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Cabin {
    pub code: String,
    pub name: String,
    pub internal_names: Vec<String>,
}

/// A main texture and the pack file it comes from.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MainTexture {
    pub id: String,
    pub name: String,
    /// The file, relative to the vehicle's folder in the pack, `/`-separated.
    pub file: String,
    pub size: TextureSize,
    pub layout_version: u32,
    /// The cabin internal names it covers; none: every cabin (a single
    /// main texture only).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cabins: Vec<String>,
    /// Accessory ids painted with this texture.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub also_paints: Vec<String>,
}

/// An accessory texture and the pack file it comes from.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AccessoryTexture {
    pub id: String,
    pub name: String,
    /// The file, relative to the vehicle's folder in the pack, `/`-separated.
    pub file: String,
    pub size: TextureSize,
    pub layout_version: u32,
    /// The accessory ids it covers.
    pub ids: Vec<String>,
}

/// Ids left out on purpose, with the reason ("no template in the pack").
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LeftOut {
    pub ids: Vec<String>,
    pub reason: String,
}

/// A texture of an entry, main or accessory.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextureFile<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub file: &'a str,
    pub size: TextureSize,
}

impl Entry {
    /// Reads an entry from its JSON.
    pub fn parse(json: &str) -> Result<Entry, String> {
        let entry: Entry = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if entry.format > FORMAT {
            return Err(format!(
                "format {} is newer than this version supports ({FORMAT})",
                entry.format
            ));
        }
        Ok(entry)
    }

    /// The entry as written in the catalog: pretty JSON ending with a line
    /// break.
    pub fn to_json(&self) -> String {
        let mut out = serde_json::to_string_pretty(self).expect("serializable");
        out.push('\n');
        out
    }

    /// The package id: `official.<game>.<path>`.
    pub fn package_id(&self) -> String {
        format!("{ID_PREFIX}{}.{}", self.game.code(), self.path)
    }

    /// The entry's file under `catalog/`: `<game>/<path>.json`.
    pub fn catalog_path(&self) -> String {
        format!("{}/{}.json", self.game.code(), self.path)
    }

    /// Every texture, main textures first, in entry order.
    pub fn textures(&self) -> impl Iterator<Item = TextureFile<'_>> {
        let main = self.main.iter().map(|t| TextureFile {
            id: &t.id,
            name: &t.name,
            file: &t.file,
            size: t.size,
        });
        let accessories = self.accessories.iter().map(|t| TextureFile {
            id: &t.id,
            name: &t.name,
            file: &t.file,
            size: t.size,
        });
        main.chain(accessories)
    }

    /// Path of texture `id`'s template inside the package.
    pub fn template_path(id: &str) -> String {
        format!("templates/{id}.png")
    }
}

/// Whether `id` is the package id of an official vehicle.
pub fn is_official(id: &str) -> bool {
    id.starts_with(ID_PREFIX)
}

/// What makes an entry incomplete.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    NoMainTexture,
    /// One of several main textures covers no cabin (its name).
    MainWithoutCabins(String),
    /// A cabin internal name no main texture covers.
    UncoveredCabin(String),
    /// An accessory id no texture covers and that isn't left out.
    UncoveredAccessory(String),
    /// An id covered twice (by textures or left out).
    CoveredTwice(String),
    /// An id covered but absent from the game data.
    NotInGameData(String),
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Problem::NoMainTexture => f.write_str("no main texture"),
            Problem::MainWithoutCabins(name) => write!(
                f,
                "the main texture \"{name}\" lists no cabin, but there are several main textures"
            ),
            Problem::UncoveredCabin(id) => write!(f, "the cabin \"{id}\" is not covered"),
            Problem::UncoveredAccessory(id) => write!(
                f,
                "the accessory id \"{id}\" is neither covered nor left out"
            ),
            Problem::CoveredTwice(id) => write!(f, "\"{id}\" is covered twice"),
            Problem::NotInGameData(id) => {
                write!(f, "\"{id}\" is covered but not in the game data")
            }
        }
    }
}

/// Counts how often each id is covered: those of `game` (in order) and
/// those absent from it.
fn coverage<'a>(
    game: &[&'a String],
    covered: impl Iterator<Item = &'a String>,
    problems: &mut Vec<Problem>,
) -> HashMap<&'a String, usize> {
    let mut counts: HashMap<&String, usize> = game.iter().map(|id| (*id, 0)).collect();
    let mut reported = Vec::new();
    for id in covered {
        match counts.get_mut(id) {
            Some(n) => *n += 1,
            None if !reported.contains(&id) => {
                reported.push(id);
                problems.push(Problem::NotInGameData(id.clone()));
            }
            None => problems.push(Problem::CoveredTwice(id.clone())),
        }
    }
    counts
}

/// Whether every cabin is covered by exactly one main texture and every
/// accessory id by exactly one texture or left out, and every covered id
/// is in the game data. Problems in game data order.
pub fn completeness(entry: &Entry) -> Vec<Problem> {
    let mut problems = Vec::new();
    if entry.main.is_empty() {
        problems.push(Problem::NoMainTexture);
    }
    if entry.main.len() > 1 {
        for t in entry.main.iter().filter(|t| t.cabins.is_empty()) {
            problems.push(Problem::MainWithoutCabins(t.name.clone()));
        }
    }
    let every_cabin = entry.main.len() == 1 && entry.main[0].cabins.is_empty();
    let cabins: Vec<&String> = entry.game_data.cabin_names().collect();
    let counts = coverage(
        &cabins,
        entry.main.iter().flat_map(|t| &t.cabins),
        &mut problems,
    );
    for id in &cabins {
        match counts[id] {
            0 if !every_cabin => problems.push(Problem::UncoveredCabin((*id).clone())),
            0 | 1 => {}
            _ => problems.push(Problem::CoveredTwice((*id).clone())),
        }
    }
    let ids: Vec<&String> = entry.game_data.accessory_ids().collect();
    let covered = entry
        .accessories
        .iter()
        .flat_map(|t| &t.ids)
        .chain(entry.main.iter().flat_map(|t| &t.also_paints))
        .chain(entry.left_out.iter().flat_map(|l| &l.ids));
    let counts = coverage(&ids, covered, &mut problems);
    for id in &ids {
        match counts[id] {
            0 => problems.push(Problem::UncoveredAccessory((*id).clone())),
            1 => {}
            _ => problems.push(Problem::CoveredTwice((*id).clone())),
        }
    }
    problems.dedup();
    problems
}

/// Every embedded entry, complete or not, in [`ENTRIES`] order. An entry
/// that can't be read is left out (the tests refuse it).
pub fn embedded() -> &'static [Entry] {
    static ALL: OnceLock<Vec<Entry>> = OnceLock::new();
    ALL.get_or_init(|| {
        ENTRIES
            .iter()
            .filter_map(|(_, json)| Entry::parse(json).ok())
            .collect()
    })
}

/// The complete embedded entries of `game`, sorted by brand then name.
pub fn entries(game: Game) -> Vec<&'static Entry> {
    let mut out: Vec<&Entry> = embedded()
        .iter()
        .filter(|e| e.game == game && completeness(e).is_empty())
        .collect();
    sort(&mut out);
    out
}

/// Sorts `entries` by brand then name, ignoring case and accents.
pub fn sort(entries: &mut [&Entry]) {
    entries.sort_by_cached_key(|e| (crate::fold(&e.brand), crate::fold(&e.name)));
}

/// The embedded entry whose package id is `id`.
pub fn by_package_id(id: &str) -> Option<&'static Entry> {
    embedded().iter().find(|e| e.package_id() == id)
}

/// The package manifest of `entry`: id `official.<game>.<path>`, the
/// entry's version, game data and texture game ids, and each template at
/// `templates/<texture id>.png`.
pub fn manifest(entry: &Entry) -> Value {
    let texture = |id: &str, size: TextureSize, layout_version: u32| {
        json!({
            "size": size,
            "template": Entry::template_path(id),
            "layout_version": layout_version,
        })
    };
    let main: Vec<Value> = entry
        .main
        .iter()
        .map(|t| {
            json!({
                "id": t.id,
                "name": t.name,
                "game_ids": t.cabins,
                "also_paints": t.also_paints,
                "texture": texture(&t.id, t.size, t.layout_version),
            })
        })
        .collect();
    let accessories: Vec<Value> = entry
        .accessories
        .iter()
        .map(|t| {
            json!({
                "id": t.id,
                "name": t.name,
                "game_ids": t.ids,
                "texture": texture(&t.id, t.size, t.layout_version),
            })
        })
        .collect();
    json!({
        "format": crate::FORMAT,
        "id": entry.package_id(),
        "version": entry.version.to_string(),
        "name": entry.name,
        "brand": entry.brand,
        "kind": entry.kind,
        "game": {
            "id": entry.game,
            "versions": entry.game_versions.to_string(),
            "path": entry.path,
            "alt_uv": entry.game_data.alt_uv,
            "colour_picker": entry.game_data.colour_picker,
        },
        "paint_job": {
            "main": main,
            "accessories": accessories,
        },
    })
}

#[cfg(test)]
mod tests;
