//! The settings of the mod a project exports as.

use crate::document::AssetId;

/// Version of a new project's mod.
pub const DEFAULT_MOD_VERSION: &str = "1.0";
/// Price of a new project's paint job, in the game's currency.
pub const DEFAULT_PRICE: u32 = 5000;
/// Longest internal name: the game's names have at most 12 characters.
pub const INTERNAL_NAME_MAX: usize = 12;
/// Longest internal name when a truck has several main textures, whose
/// paint jobs are named `<internal name>_<letter>`.
pub const SPLIT_INTERNAL_NAME_MAX: usize = 10;
/// Internal name of a Name that gives nothing.
pub const FALLBACK_INTERNAL_NAME: &str = "paintjob";

/// What the player sets for the exported mod.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModSettings {
    /// The paint job's name in the shop and the mod's in the Mod Manager.
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    /// The internal name as typed by the player; `None` while it follows
    /// the Name (see [`ModSettings::internal_name`]).
    pub internal_name: Option<String>,
    pub price: u32,
    pub unlock_level: u32,
    /// Chosen shop icon; `None`: generated from the artwork.
    pub icon: Option<AssetId>,
    /// Chosen Mod Manager image; `None`: generated from the artwork.
    pub image: Option<AssetId>,
}

impl ModSettings {
    /// The settings of a new project named `name`.
    pub fn for_project(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            version: DEFAULT_MOD_VERSION.to_owned(),
            author: String::new(),
            description: String::new(),
            internal_name: None,
            price: DEFAULT_PRICE,
            unlock_level: 0,
            icon: None,
            image: None,
        }
    }

    /// The internal name: as typed, or derived from the Name with at most
    /// `max_len` characters.
    pub fn internal_name(&self, max_len: usize) -> String {
        self.internal_name
            .clone()
            .unwrap_or_else(|| derive_internal_name(&self.name, max_len))
    }
}

/// The internal name derived from `name`: lowercased, each run of
/// characters other than `a`–`z` and `0`–`9` replaced by `_`, trimmed of
/// `_` and cut to `max_len` characters; [`FALLBACK_INTERNAL_NAME`] when
/// nothing is left.
pub fn derive_internal_name(name: &str, max_len: usize) -> String {
    let mut out = String::new();
    for c in name.to_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            out.push(c);
        } else if !out.ends_with('_') {
            out.push('_');
        }
    }
    let mut out: String = out.trim_matches('_').chars().take(max_len).collect();
    while out.ends_with('_') {
        out.pop();
    }
    if out.is_empty() {
        FALLBACK_INTERNAL_NAME.chars().take(max_len).collect()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_of_a_new_project() {
        let s = ModSettings::for_project("ACE Logistics");
        assert_eq!(s.name, "ACE Logistics");
        assert_eq!(s.version, "1.0");
        assert_eq!((s.author.as_str(), s.description.as_str()), ("", ""));
        assert_eq!((s.price, s.unlock_level), (5000, 0));
        assert_eq!((s.icon, s.image), (None, None));
        assert_eq!(s.internal_name(INTERNAL_NAME_MAX), "ace_logistic");
        assert_eq!(s.internal_name(SPLIT_INTERNAL_NAME_MAX), "ace_logist");
    }

    #[test]
    fn edited_internal_name_is_kept() {
        let mut s = ModSettings::for_project("ACE");
        s.internal_name = Some("acelog".into());
        s.name = "ACE Freight".into();
        assert_eq!(s.internal_name(INTERNAL_NAME_MAX), "acelog");
    }

    #[test]
    fn derived_names() {
        assert_eq!(derive_internal_name("ACE Logistics", 12), "ace_logistic");
        assert_eq!(derive_internal_name("ACE Logistics", 10), "ace_logist");
        assert_eq!(derive_internal_name("  Été--Express! ", 12), "t_express");
        assert_eq!(derive_internal_name("!!!", 12), "paintjob");
        // No trailing `_` after the cut.
        assert_eq!(derive_internal_name("abcd efgh ij", 5), "abcd");
    }
}
