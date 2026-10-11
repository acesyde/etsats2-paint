//! `tpv catalog match`: proposes the catalog entries of official vehicles
//! from the names of the template pack's files and Paintjob Packer's game
//! data. What it is not sure of is reported as a doubt, and what it can't
//! place is left uncovered, so the entry stays incomplete until a person
//! resolves it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use tp_vehicles::catalog::{AccessoryTexture, Entry, LeftOut, MainTexture, Source};
use tp_vehicles::{Game, Kind, TextureSize, fold};

use super::{PpkVehicle, pack_version, png_size, pngs, ppk_official, resolve_file};

/// The reason given to the groups no file matches.
pub const NO_TEMPLATE: &str = "no template in the pack";

/// Brands of more than one word.
const MULTI_WORD_BRANDS: &[&str] = &["Schmitz Cargobull"];

/// One vehicle's proposed entry and what to check by hand.
#[derive(Debug)]
pub struct VehicleMatch {
    pub entry: Entry,
    /// Doubtful matches, each saying what and why.
    pub doubts: Vec<String>,
    /// Cabins and accessory ids placed nowhere.
    pub uncovered: Vec<String>,
    /// The vehicle's PNG files no texture uses, relative to its folder.
    pub unused: Vec<String>,
    /// Files that can't be used (unreadable, not a texture size).
    pub problems: Vec<String>,
}

/// What `tpv catalog match` found.
#[derive(Debug, Default)]
pub struct MatchReport {
    pub vehicles: Vec<VehicleMatch>,
    /// Vehicles already in the catalog, left unchanged (names).
    pub kept: Vec<String>,
    /// Vehicles with no folder in the pack: (name, path).
    pub not_found: Vec<(String, String)>,
    /// The pack's folders that no official vehicle uses.
    pub spare_folders: Vec<String>,
}

/// A name's words: folded, split on anything but letters and digits, a
/// plural "s" dropped (`Main Mirrors` → `main`, `mirror`).
fn words(name: &str) -> Vec<String> {
    fold(name)
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| match w.strip_suffix('s') {
            Some(stem) if w.len() > 3 && !stem.ends_with('s') => stem.to_owned(),
            _ => w.to_owned(),
        })
        .collect()
}

/// A name's words, plus each two neighbours joined (`Side Skirt` also
/// gives `sideskirt`).
fn word_set(name: &str) -> BTreeSet<String> {
    let w = words(name);
    let joined = w
        .windows(2)
        .map(|p| words(&format!("{}{}", p[0], p[1])).concat());
    w.iter().cloned().chain(joined).collect()
}

/// A file name without its folder and extension.
fn stem(file: &str) -> &str {
    let name = file.rsplit('/').next().unwrap_or(file);
    name.rsplit_once('.').map_or(name, |(s, _)| s)
}

/// A texture id from a name: lowercase ASCII words joined by `_`.
fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in fold(name).chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.is_empty() && !out.ends_with('_') {
            out.push('_');
        }
    }
    out.trim_end_matches('_').to_owned()
}

/// The brand of a vehicle named `name`.
fn brand(name: &str) -> String {
    MULTI_WORD_BRANDS
        .iter()
        .find(|b| name.starts_with(&format!("{b} ")))
        .map_or_else(
            || name.split(' ').next().unwrap_or(name).to_owned(),
            |b| (*b).to_owned(),
        )
}

/// Where a vehicle's files are.
struct Folder {
    /// The folder's name in the pack, NFC.
    name: String,
    /// The vehicle's prefix in a shared folder (`R` in `Scania R & S`).
    prefix: Option<String>,
    /// The other vehicles' prefixes in that folder.
    others: Vec<String>,
    /// Why it isn't certain, when it isn't.
    doubt: Option<String>,
}

/// The names a shared folder `A & B` stands for, with their prefixes:
/// `Scania R & S` gives ("Scania R", "R") and ("Scania S", "S").
fn shared_names(folder: &str) -> Option<[(String, String); 2]> {
    let (left, right) = folder.split_once(" & ")?;
    let (brand, left_part) = left.split_once(' ')?;
    Some([
        (left.to_owned(), left_part.to_owned()),
        (format!("{brand} {right}"), right.to_owned()),
    ])
}

/// Finds the folder of each vehicle among the pack's `folders` (design
/// Decision 3): exact name, shared folder, name prefix, game path words.
fn find_folders(vehicles: &[&PpkVehicle], folders: &[String]) -> BTreeMap<String, Folder> {
    let mut found = BTreeMap::new();
    let exact: BTreeSet<String> = folders
        .iter()
        .filter(|f| vehicles.iter().any(|v| fold(&v.name) == fold(f)))
        .map(|f| fold(f))
        .collect();
    for v in vehicles {
        let name = fold(&v.name);
        if let Some(f) = folders.iter().find(|f| fold(f) == name) {
            found.insert(
                v.path.clone(),
                Folder {
                    name: nfc(f),
                    prefix: None,
                    others: Vec::new(),
                    doubt: None,
                },
            );
            continue;
        }
        let shared = folders.iter().find_map(|f| {
            let names = shared_names(f)?;
            let i = names.iter().position(|(n, _)| fold(n) == name)?;
            Some(Folder {
                name: nfc(f),
                prefix: Some(names[i].1.clone()),
                others: vec![names[1 - i].1.clone()],
                doubt: Some(format!("folder \"{}\" is shared", nfc(f))),
            })
        });
        if let Some(folder) = shared {
            found.insert(v.path.clone(), folder);
            continue;
        }
        let free = || folders.iter().filter(|f| !exact.contains(&fold(f)));
        let starts = free()
            .filter(|f| name.starts_with(&format!("{} ", fold(f))))
            .max_by_key(|f| f.len());
        let model: Vec<String> = v
            .path
            .split_once('.')
            .map_or("", |(_, m)| m)
            .split('_')
            .map(fold)
            .collect();
        let first_word = name.split(' ').next().unwrap_or("").to_owned();
        let by_path = || {
            free().find(|f| {
                let w: Vec<String> = fold(f).split(' ').map(str::to_owned).collect();
                w.first() == Some(&first_word) && model.iter().all(|m| w.contains(m))
            })
        };
        let (f, why) = match starts {
            Some(f) => (f, "the vehicle's name starts with it"),
            None => match by_path() {
                Some(f) => (f, "it holds the words of the game path"),
                None => continue,
            },
        };
        found.insert(
            v.path.clone(),
            Folder {
                name: nfc(f),
                prefix: None,
                others: Vec::new(),
                doubt: Some(format!("folder \"{}\" chosen because {why}", nfc(f))),
            },
        );
    }
    found
}

fn nfc(name: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    name.nfc().collect()
}

/// A file of the vehicle: its path in the folder and the name it is matched
/// by (without a shared folder's prefix or qualifier).
#[derive(Clone, Debug)]
struct File {
    path: String,
    name: String,
}

/// The vehicle's files of its folder: root files with its prefix (or none
/// of the other vehicles'), accessory files without another vehicle's
/// qualifier.
fn own_files(all: &[String], folder: &Folder) -> (Vec<File>, Vec<File>) {
    let starts = |s: &str, p: &str| fold(s).starts_with(&format!("{} ", fold(p)));
    let mut root = Vec::new();
    let mut accessories = Vec::new();
    for path in all {
        let name = stem(path).to_owned();
        if path.contains('/') {
            let qualifier = |p: &str| fold(&name).contains(&format!("({})", fold(p)));
            if folder.others.iter().any(|o| qualifier(o)) {
                continue;
            }
            let name = match &folder.prefix {
                Some(p) if qualifier(p) => {
                    let cut = format!("({p})");
                    nfc(&name).replace(&cut, "").trim().to_owned()
                }
                _ => name,
            };
            accessories.push(File {
                path: path.clone(),
                name,
            });
        } else {
            if folder.others.iter().any(|o| starts(&name, o)) {
                continue;
            }
            let name = match &folder.prefix {
                Some(p) if starts(&name, p) => name[p.len()..].trim().to_owned(),
                _ => name,
            };
            root.push(File {
                path: path.clone(),
                name,
            });
        }
    }
    (root, accessories)
}

/// A texture being proposed: its file and the ids or cabins it covers.
struct Proposed {
    file: File,
    ids: Vec<String>,
}

/// A truck cabin file: `Cabin <code>[ Aero] (<name>)`.
fn cabin_file(name: &str) -> Option<(String, String)> {
    let rest = name.strip_prefix("Cabin ")?;
    let (code, rest) = rest.split_once(' ')?;
    let rest = rest.strip_prefix("Aero ").unwrap_or(rest);
    let cabin = rest.strip_prefix('(')?.strip_suffix(')')?;
    Some((code.to_owned(), cabin.to_owned()))
}

/// Matches `files` to the `groups` (name → ids), each file to the group
/// sharing most words with it. Returns each group's files and the files
/// matched to none.
fn match_groups<'a>(
    files: &[File],
    groups: &[(&'a String, &'a Vec<String>)],
    doubts: &mut Vec<String>,
) -> (BTreeMap<&'a String, Vec<File>>, Vec<File>) {
    let mut by_group: BTreeMap<&String, Vec<File>> = BTreeMap::new();
    let mut spare = Vec::new();
    for file in files {
        let fw = word_set(&file.name);
        let scored: Vec<(usize, &String)> = groups
            .iter()
            .map(|(g, ids)| (score(&fw, g, ids), *g))
            .filter(|(n, _)| *n > 0)
            .collect();
        let Some(best) = scored.iter().map(|(n, _)| *n).max() else {
            spare.push(file.clone());
            continue;
        };
        let top: Vec<&String> = scored
            .iter()
            .filter(|(n, _)| *n == best)
            .map(|(_, g)| *g)
            .collect();
        let group = top[0];
        if top.len() > 1 {
            doubts.push(format!(
                "\"{}\" matches the groups {} equally; \"{group}\" chosen",
                file.path,
                quoted(&top)
            ));
        } else if !words(group).iter().all(|w| fw.contains(w)) {
            doubts.push(format!(
                "\"{}\" matched to the group \"{group}\" by some of its words",
                file.path
            ));
        }
        by_group.entry(group).or_default().push(file.clone());
    }
    (by_group, spare)
}

/// Words telling where a part is rather than what it is: they count less.
const PLACE_WORDS: &[&str] = &["front", "rear", "mid", "middle", "left", "right", "top"];

/// Words of a variant that name the plain version of a part.
const PLAIN_WORDS: &[&str] = &[
    "stock", "standard", "basic", "normal", "paint", "painted", "main",
];

/// How well a file with words `fw` fits group `group` with `ids`: words in
/// common with the group's name (2 each, 1 for a place word), and the
/// file's other words found in the ids (2 each).
fn score(fw: &BTreeSet<String>, group: &str, ids: &[String]) -> usize {
    let gw = word_set(group);
    let weight = |w: &String| {
        if PLACE_WORDS.contains(&w.as_str()) {
            1
        } else {
            2
        }
    };
    let names: usize = fw.intersection(&gw).map(weight).sum();
    let in_ids: usize = fw
        .iter()
        .filter(|w| !gw.contains(*w) && !PLACE_WORDS.contains(&w.as_str()))
        .filter(|w| ids.iter().any(|id| id_has(id, w)))
        .map(|_| 2)
        .sum();
    names + in_ids
}

fn quoted(names: &[&String]) -> String {
    names
        .iter()
        .map(|n| format!("\"{n}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Whether word `v` of a file names something in id `id`: one of its
/// words is `v`, starts with it (`4x` ↔ `pnt_4x2`), or begins it
/// (`camera` ↔ `cam_semi`).
fn id_has(id: &str, v: &str) -> bool {
    fold(id)
        .split(|c: char| !c.is_alphanumeric())
        .any(|w| w == v || w.starts_with(v) || (w.len() >= 3 && v.starts_with(w)))
}

/// Splits `group`'s `ids` between its `files` by the variant words of each
/// file (its words not in the group's name nor in every file's name). An id
/// goes to the file whose variant words it all holds, the most specific
/// one. The ids no single file names go to the group's plainest file: in
/// Paintjob Packer a group is one texture, so its ids share one layout.
fn split(group: &str, ids: &[String], files: &[File], doubts: &mut Vec<String>) -> Vec<Proposed> {
    if files.len() == 1 {
        return vec![Proposed {
            file: files[0].clone(),
            ids: ids.to_vec(),
        }];
    }
    let gw = word_set(group);
    let all: Vec<BTreeSet<String>> = files
        .iter()
        .map(|f| words(&f.name).into_iter().filter(|w| w != "and").collect())
        .collect();
    let common: BTreeSet<String> = all.iter().skip(1).fold(all[0].clone(), |acc, w| {
        acc.intersection(w).cloned().collect()
    });
    let variants: Vec<Vec<String>> = all
        .iter()
        .map(|w| {
            w.iter()
                .filter(|w| !gw.contains(*w) && !common.contains(*w))
                .cloned()
                .collect()
        })
        .collect();
    let mut out: Vec<Proposed> = files
        .iter()
        .map(|f| Proposed {
            file: f.clone(),
            ids: Vec::new(),
        })
        .collect();
    let mut unplaced = Vec::new();
    let mut partial = Vec::new();
    for id in ids {
        let fits: Vec<usize> = (0..files.len())
            .filter(|&i| !variants[i].is_empty() && variants[i].iter().all(|v| id_has(id, v)))
            .collect();
        let most = fits.iter().map(|&i| variants[i].len()).max();
        let best: Vec<usize> = fits
            .into_iter()
            .filter(|&i| Some(variants[i].len()) == most)
            .collect();
        if let [i] = best[..] {
            out[i].ids.push(id.clone());
            continue;
        }
        // No file names it fully: the file naming most of it, alone.
        let scores: Vec<usize> = variants.iter().map(|v| partial_score(id, v)).collect();
        let top = scores.iter().copied().max().unwrap_or(0);
        let at_top: Vec<usize> = (0..files.len()).filter(|&i| scores[i] == top).collect();
        match at_top[..] {
            [i] if top > 0 => {
                out[i].ids.push(id.clone());
                partial.push(id.clone());
            }
            _ => unplaced.push(id.clone()),
        }
    }
    if !partial.is_empty() {
        doubts.push(format!(
            "in the group \"{group}\", {} placed by some of their file's words",
            partial.join(", ")
        ));
    }
    let names: Vec<&String> = files.iter().map(|f| &f.path).collect();
    doubts.push(format!(
        "the group \"{group}\" is split between {}",
        quoted(&names)
    ));
    if !unplaced.is_empty() {
        let plain = |i: &usize| {
            (
                variants[*i].len(),
                !variants[*i]
                    .iter()
                    .any(|v| PLAIN_WORDS.contains(&v.as_str())),
                files[*i].path.clone(),
            )
        };
        let i = (0..files.len()).min_by_key(plain).unwrap_or(0);
        doubts.push(format!(
            "no file of the group \"{group}\" names {}; given to \"{}\", as the group shares one layout",
            unplaced.join(", "),
            files[i].path
        ));
        out[i].ids.extend(unplaced);
    }
    out
}

/// How much of `variant` names id `id`: 2 per word found in it, 1 when a
/// word's initial is one of the id's one-letter words: the last one for
/// left and right (`chs_f_l`), the first one otherwise (`front` ↔
/// `chs_f_l`, `rear` ↔ `chs_r_l`, `side` ↔ `s_mirror`).
fn partial_score(id: &str, variant: &[String]) -> usize {
    let folded = fold(id);
    let letters: Vec<&str> = folded
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() == 1)
        .collect();
    variant
        .iter()
        .map(|v| {
            if !PLACE_WORDS.contains(&v.as_str()) && id_has(id, v) {
                2
            } else {
                let letter = if v == "left" || v == "right" {
                    letters.last()
                } else {
                    letters.first()
                };
                usize::from(letter.copied() == v.get(..1))
            }
        })
        .sum()
}

/// Proposes the entry of `v` from its folder `dir` in the pack.
fn match_vehicle(
    v: &PpkVehicle,
    game: Game,
    folder: &Folder,
    dir: &Path,
    commit: &str,
    pack: &str,
) -> VehicleMatch {
    let mut doubts: Vec<String> = folder.doubt.iter().cloned().collect();
    let mut uncovered = Vec::new();
    let mut problems = Vec::new();
    let mut all = Vec::new();
    pngs(dir, dir, &mut all);
    all.sort();
    let (root, accessory_files) = own_files(&all, folder);
    let mut own_paths: Vec<String> = root
        .iter()
        .chain(&accessory_files)
        .map(|f| f.path.clone())
        .collect();
    own_paths.sort();
    let data = v.game_data();
    let kind = if v.kind == "truck" {
        Kind::Truck
    } else {
        Kind::Trailer
    };
    let plain = |f: &File| {
        let n = fold(&f.name);
        n == fold(&v.name)
            || n == fold(&folder.name)
            || folder.prefix.as_deref().is_some_and(|p| n == fold(p))
    };

    // Main textures: (file, cabins) for a truck; bodies for a trailer.
    let mut mains: Vec<(File, Vec<String>, String, String)> = Vec::new(); // file, cabins, id, name
    let mut body: Option<Proposed> = None;
    let mut other_bodies: Vec<Proposed> = Vec::new();
    let mut leftovers: Vec<File> = Vec::new();
    let mut bodies_without_file: Vec<&String> = Vec::new();
    let body_groups: Vec<(&String, &Vec<String>)> = data
        .accessories
        .iter()
        .filter(|(g, _)| kind == Kind::Trailer && fold(g).starts_with("body"))
        .collect();
    match kind {
        Kind::Truck => {
            let mut taken: BTreeSet<&str> = BTreeSet::new();
            for f in &root {
                let Some((code, cabin)) = cabin_file(&f.name) else {
                    leftovers.push(f.clone());
                    continue;
                };
                let code = if code == "8x4" {
                    "8".to_owned()
                } else {
                    code.to_lowercase()
                };
                let named: Vec<_> = data
                    .cabins
                    .iter()
                    .filter(|c| fold(&c.name) == fold(&cabin))
                    .collect();
                let chosen = match named[..] {
                    [one] => Some(one),
                    [] => {
                        let by_code = data.cabins.iter().find(|c| c.code == code);
                        if let Some(c) = by_code {
                            doubts.push(format!(
                                "\"{}\" matched to the cabin {} ({}) by its code only",
                                f.path, c.name, c.code
                            ));
                        }
                        by_code
                    }
                    _ => named
                        .iter()
                        .find(|c| c.code == code)
                        .or_else(|| named.iter().find(|c| c.code != "8"))
                        .copied(),
                };
                let Some(c) = chosen else {
                    leftovers.push(f.clone());
                    continue;
                };
                if !taken.insert(&c.code) {
                    doubts.push(format!(
                        "\"{}\" matches the cabin {}, already covered",
                        f.path, c.name
                    ));
                    leftovers.push(f.clone());
                    continue;
                }
                let twin = data
                    .cabins
                    .iter()
                    .any(|o| o.code != c.code && fold(&o.name) == fold(&c.name));
                let name = if twin && c.code == "8" {
                    format!("{} 8x4", c.name)
                } else {
                    c.name.clone()
                };
                let id = c
                    .internal_names
                    .first()
                    .cloned()
                    .unwrap_or_else(|| slug(&name));
                mains.push((f.clone(), c.internal_names.clone(), id, name));
            }
            if mains.is_empty() {
                let singles: Vec<&File> = leftovers.iter().filter(|f| plain(f)).collect();
                if let [single] = singles[..] {
                    if let [cabin] = &data.cabins[..] {
                        let id = cabin
                            .internal_names
                            .first()
                            .cloned()
                            .unwrap_or_else(|| slug(&cabin.name));
                        mains.push(((*single).clone(), Vec::new(), id, cabin.name.clone()));
                    } else {
                        doubts.push(format!(
                            "\"{}\" is the only main file but the truck has {} cabins",
                            single.path,
                            data.cabins.len()
                        ));
                    }
                    let path = single.path.clone();
                    leftovers.retain(|f| f.path != path);
                }
            }
            // Keep the game data's cabin order.
            mains.sort_by_key(|(_, cabins, _, _)| {
                data.cabins
                    .iter()
                    .position(|c| c.internal_names == *cabins)
                    .unwrap_or(0)
            });
            let covered: BTreeSet<&String> = mains.iter().flat_map(|m| &m.1).collect();
            let every = mains.len() == 1 && mains[0].1.is_empty();
            if !every {
                uncovered.extend(data.cabin_names().filter(|n| !covered.contains(n)).cloned());
            }
        }
        Kind::Trailer => {
            // Root files against body groups: equal words first, then the
            // most words in common, then a single group left takes the
            // files left.
            let bw = |name: &str| -> BTreeSet<String> {
                words(name).into_iter().filter(|w| w != "body").collect()
            };
            let file_words = |f: &File| {
                if plain(f) {
                    BTreeSet::new()
                } else {
                    bw(&f.name)
                }
            };
            let mut by_group: BTreeMap<&String, Vec<File>> = BTreeMap::new();
            let mut rest: Vec<File> = Vec::new();
            for f in &root {
                match body_groups.iter().find(|(g, _)| bw(g) == file_words(f)) {
                    Some((g, _)) => by_group.entry(g).or_default().push(f.clone()),
                    None => rest.push(f.clone()),
                }
            }
            let mut still = Vec::new();
            for f in rest {
                let fw = file_words(&f);
                let best = body_groups
                    .iter()
                    .filter(|(g, _)| !by_group.contains_key(g))
                    .map(|(g, _)| (bw(g).intersection(&fw).count(), *g))
                    .filter(|(n, _)| *n > 0)
                    .max_by_key(|(n, _)| *n);
                match best {
                    Some((_, g)) => {
                        doubts.push(format!(
                            "\"{}\" matched to the group \"{g}\" by some of its words",
                            f.path
                        ));
                        by_group.entry(g).or_default().push(f);
                    }
                    None => still.push(f),
                }
            }
            let free: Vec<&String> = body_groups
                .iter()
                .map(|(g, _)| *g)
                .filter(|g| !by_group.contains_key(g))
                .collect();
            if let [g] = free[..]
                && !still.is_empty()
            {
                let names: Vec<&String> = still.iter().map(|f| &f.path).collect();
                doubts.push(format!(
                    "the group \"{g}\" has no file of its name; {} given to it",
                    quoted(&names)
                ));
                by_group.insert(g, std::mem::take(&mut still));
            }
            leftovers.extend(still);
            let mut textures = Vec::new();
            for (g, ids) in &body_groups {
                match by_group.get(g) {
                    Some(files) => textures.extend(split(g, ids, files, &mut doubts)),
                    None => bodies_without_file.push(g),
                }
            }
            let first = body_groups.first().and_then(|(_, ids)| ids.first());
            let main_at = textures
                .iter()
                .position(|t| first.is_some_and(|id| t.ids.contains(id)))
                .or_else(|| (!textures.is_empty()).then_some(0));
            if let Some(i) = main_at {
                body = Some(textures.remove(i));
            }
            other_bodies = textures;
        }
    }

    // Accessories: the accessory files and the root files left, against
    // the groups that aren't bodies.
    let groups: Vec<(&String, &Vec<String>)> = data
        .accessories
        .iter()
        .filter(|(g, _)| !body_groups.iter().any(|(b, _)| b == g))
        .collect();
    let mut candidates = accessory_files;
    candidates.extend(leftovers);
    let (by_group, _) = match_groups(&candidates, &groups, &mut doubts);
    let mut accessories: Vec<Proposed> = other_bodies;
    let mut left_out = Vec::new();
    for (g, ids) in &groups {
        match by_group.get(g) {
            Some(files) => accessories.extend(split(g, ids, files, &mut doubts)),
            None => left_out.push(LeftOut {
                ids: (*ids).clone(),
                reason: NO_TEMPLATE.to_owned(),
            }),
        }
    }
    for g in bodies_without_file {
        left_out.push(LeftOut {
            ids: data.accessories[g].clone(),
            reason: NO_TEMPLATE.to_owned(),
        });
        doubts.push(format!("the body group \"{g}\" has no file"));
    }

    // Sizes and ids.
    let mut size_of = |file: &File| -> Option<TextureSize> {
        let path = resolve_file(dir, &file.path)?;
        let bytes = std::fs::read(&path).ok()?;
        let Some((w, h)) = png_size(&bytes) else {
            problems.push(format!("\"{}\" is not a PNG image", file.path));
            return None;
        };
        let size = TextureSize::new(w, h);
        if !size.is_allowed() {
            problems.push(format!(
                "\"{}\" is {w} × {h}, not a texture size (powers of two from 64 to 8192)",
                file.path
            ));
            return None;
        }
        Some(size)
    };
    let mut used_ids: BTreeSet<String> = BTreeSet::new();
    let mut unique = |id: String| {
        let mut out = id.clone();
        let mut n = 2;
        while !used_ids.insert(out.clone()) {
            out = format!("{id}_{n}");
            n += 1;
        }
        out
    };
    let mut main = Vec::new();
    for (file, cabins, id, name) in mains {
        let Some(size) = size_of(&file) else {
            uncovered.extend(cabins);
            continue;
        };
        main.push(MainTexture {
            id: unique(id),
            name,
            file: file.path,
            size,
            layout_version: 1,
            cabins,
            also_paints: Vec::new(),
        });
    }
    if let Some(b) = body {
        match size_of(&b.file) {
            Some(size) => main.push(MainTexture {
                id: unique("body".to_owned()),
                name: b.file.name.clone(),
                file: b.file.path,
                size,
                layout_version: 1,
                cabins: Vec::new(),
                also_paints: b.ids,
            }),
            None => uncovered.extend(b.ids),
        }
    }
    let mut accessory_textures = Vec::new();
    accessories.sort_by(|a, b| a.file.path.cmp(&b.file.path));
    for t in accessories {
        if t.ids.is_empty() {
            continue;
        }
        let Some(size) = size_of(&t.file) else {
            uncovered.extend(t.ids);
            continue;
        };
        accessory_textures.push(AccessoryTexture {
            id: unique(slug(&t.file.name)),
            name: t.file.name.clone(),
            file: t.file.path,
            size,
            layout_version: 1,
            ids: t.ids,
        });
    }
    let used: BTreeSet<String> = main
        .iter()
        .map(|t| t.file.clone())
        .chain(accessory_textures.iter().map(|t| t.file.clone()))
        .collect();
    let unused = own_paths
        .into_iter()
        .filter(|f| !used.contains(f))
        .collect();
    let game_versions = pack
        .split(' ')
        .next()
        .and_then(|v| semver::VersionReq::parse(&format!(">={v}")).ok())
        .unwrap_or(semver::VersionReq::STAR);
    let entry = Entry {
        format: tp_vehicles::catalog::FORMAT,
        game,
        path: v.path.clone(),
        name: nfc(&v.name),
        brand: nfc(&brand(&v.name)),
        kind,
        version: semver::Version::new(1, 0, 0),
        game_versions,
        folder: folder.name.clone(),
        source: Source {
            paintjob_packer: commit.to_owned(),
            pack: pack.to_owned(),
        },
        game_data: data,
        main,
        accessories: accessory_textures,
        left_out,
    };
    VehicleMatch {
        entry,
        doubts,
        uncovered,
        unused,
        problems,
    }
}

/// Proposes an entry for each official vehicle of `game` in the Paintjob
/// Packer copy `ppk` that `existing` doesn't have (only those whose path is
/// in `only`, when it isn't empty), from the pack folder `pack`.
pub fn match_vehicles(
    pack: &Path,
    ppk: &Path,
    game: Game,
    commit: &str,
    only: &[String],
    existing: &[Entry],
) -> Result<MatchReport, String> {
    let official = ppk_official(ppk, game)?;
    let mut folders: Vec<String> = std::fs::read_dir(pack)
        .map_err(|e| format!("{}: {e}", pack.display()))?
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| !n.starts_with('.'))
        .collect();
    folders.sort();
    let vehicles: Vec<&PpkVehicle> = official.values().collect();
    let found = find_folders(&vehicles, &folders);
    let version = pack_version(pack).unwrap_or_else(|| "unknown".to_owned());
    let mut report = MatchReport::default();
    for v in &vehicles {
        if !only.is_empty() && !only.contains(&v.path) {
            continue;
        }
        if existing.iter().any(|e| e.game == game && e.path == v.path) {
            report.kept.push(nfc(&v.name));
            continue;
        }
        let Some(folder) = found.get(&v.path) else {
            report.not_found.push((nfc(&v.name), v.path.clone()));
            continue;
        };
        let Some(dir) = super::resolve(pack, &folder.name) else {
            report.not_found.push((nfc(&v.name), v.path.clone()));
            continue;
        };
        report
            .vehicles
            .push(match_vehicle(v, game, folder, &dir, commit, &version));
    }
    let used: BTreeSet<String> = found
        .values()
        .map(|f| fold(&f.name))
        .chain(existing.iter().map(|e| fold(&e.folder)))
        .collect();
    report.spare_folders = folders
        .iter()
        .filter(|f| !used.contains(&fold(f)))
        .map(|f| nfc(f))
        .collect();
    Ok(report)
}

impl MatchReport {
    /// The report as `tpv catalog match` prints it.
    pub fn text(&self) -> String {
        let mut out = String::new();
        for m in &self.vehicles {
            let e = &m.entry;
            out.push_str(&format!("== {} ({}) in \"{}\"\n", e.name, e.path, e.folder));
            for t in &e.main {
                let covers = if !t.cabins.is_empty() {
                    t.cabins.join(", ")
                } else if !t.also_paints.is_empty() {
                    format!("every cabin, also {}", t.also_paints.join(", "))
                } else {
                    "every cabin".to_owned()
                };
                out.push_str(&format!(
                    "main:      {} <- \"{}\" ({}): {covers}\n",
                    t.name,
                    t.file,
                    t.size.label()
                ));
            }
            for t in &e.accessories {
                out.push_str(&format!(
                    "accessory: {} <- \"{}\" ({}): {}\n",
                    t.name,
                    t.file,
                    t.size.label(),
                    t.ids.join(", ")
                ));
            }
            for l in &e.left_out {
                out.push_str(&format!("left out:  {} ({})\n", l.ids.join(", "), l.reason));
            }
            for d in &m.doubts {
                out.push_str(&format!("doubt:     {d}\n"));
            }
            for id in &m.uncovered {
                out.push_str(&format!("uncovered: {id}\n"));
            }
            for p in &m.problems {
                out.push_str(&format!("problem:   {p}\n"));
            }
            for f in &m.unused {
                out.push_str(&format!("unused:    {f}\n"));
            }
        }
        for name in &self.kept {
            out.push_str(&format!("kept:      {name} (already in the catalog)\n"));
        }
        for (name, path) in &self.not_found {
            out.push_str(&format!(
                "not found: {name} ({path}): no folder in the pack\n"
            ));
        }
        for f in &self.spare_folders {
            out.push_str(&format!("no vehicle: folder \"{f}\"\n"));
        }
        let doubts: usize = self.vehicles.iter().map(|m| m.doubts.len()).sum();
        let uncovered: usize = self.vehicles.iter().map(|m| m.uncovered.len()).sum();
        out.push_str(&format!(
            "{} vehicle(s) matched, {doubts} doubt(s), {uncovered} uncovered id(s)\n",
            self.vehicles.len()
        ));
        out
    }
}
