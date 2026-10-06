//! Font library: bundled fonts (always available) and, on desktop, the fonts
//! installed on the computer, loaded in the background.

use std::sync::mpsc::{self, Receiver};
use std::thread;

use cosmic_text::FontSystem;
use fontdb::Database;

/// Family used when a text's family is not available.
pub const FALLBACK_FAMILY: &str = "Inter";

/// Bundled families, in the order shown to users.
pub const BUNDLED_FAMILIES: [&str; 5] = [
    "Inter",
    "Barlow Condensed",
    "Oswald",
    "Bebas Neue",
    "Montserrat",
];

macro_rules! font {
    ($file:literal) => {
        include_bytes!(concat!("../../../assets/fonts/", $file)).as_slice()
    };
}

const BUNDLED: &[&[u8]] = &[
    font!("Inter-Regular.ttf"),
    font!("Inter-SemiBold.ttf"),
    font!("Inter-Bold.ttf"),
    font!("Inter-Italic.ttf"),
    font!("Inter-BoldItalic.ttf"),
    font!("BarlowCondensed-Regular.ttf"),
    font!("BarlowCondensed-SemiBold.ttf"),
    font!("BarlowCondensed-Bold.ttf"),
    font!("BarlowCondensed-Italic.ttf"),
    font!("BarlowCondensed-BoldItalic.ttf"),
    font!("Oswald-Variable.ttf"),
    font!("BebasNeue-Regular.ttf"),
    font!("Montserrat-Variable.ttf"),
    font!("Montserrat-Italic-Variable.ttf"),
];

/// Bytes of every bundled font file (to share them with other renderers).
pub fn bundled_font_data() -> &'static [&'static [u8]] {
    BUNDLED
}

fn bundled_database() -> Database {
    let mut db = Database::new();
    for data in BUNDLED {
        db.load_font_data(data.to_vec());
    }
    db.set_sans_serif_family(FALLBACK_FAMILY);
    db
}

fn font_system(db: Database) -> FontSystem {
    FontSystem::new_with_locale_and_db("en-US".to_owned(), db)
}

/// All fonts available to texts.
pub struct FontLibrary {
    pub(crate) system: FontSystem,
    families: Vec<String>,
    pending: Option<Receiver<Database>>,
    system_requested: bool,
    /// Bumped whenever the set of fonts changes (cached glyphs are stale).
    generation: u64,
}

impl std::fmt::Debug for FontLibrary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontLibrary")
            .field("families", &self.families.len())
            .field("loading", &self.pending.is_some())
            .finish()
    }
}

impl FontLibrary {
    /// Bundled fonts only (tests, and the first frames before system fonts
    /// are ready).
    pub fn bundled() -> Self {
        let mut library = Self {
            system: font_system(bundled_database()),
            families: Vec::new(),
            pending: None,
            system_requested: false,
            generation: 0,
        };
        library.refresh_families();
        library
    }

    /// Starts loading the fonts installed on the computer in a background
    /// thread; `ready` is called when they can be picked up with
    /// [`Self::poll`]. Does nothing if already requested.
    pub fn load_system_fonts(&mut self, ready: impl Fn() + Send + 'static) {
        if self.system_requested {
            return;
        }
        self.system_requested = true;
        let (tx, rx) = mpsc::channel();
        let spawned = thread::Builder::new()
            .name("system-fonts".into())
            .spawn(move || {
                let mut db = bundled_database();
                db.load_system_fonts();
                let _ = tx.send(db);
                ready();
            });
        if spawned.is_ok() {
            self.pending = Some(rx);
        }
    }

    pub fn system_requested(&self) -> bool {
        self.system_requested
    }

    /// Installs system fonts when the background load has finished.
    /// Returns true when the font set changed.
    pub fn poll(&mut self) -> bool {
        let Some(rx) = &self.pending else {
            return false;
        };
        match rx.try_recv() {
            Ok(db) => {
                self.system = font_system(db);
                self.pending = None;
                self.generation += 1;
                self.refresh_families();
                true
            }
            Err(mpsc::TryRecvError::Empty) => false,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.pending = None;
                false
            }
        }
    }

    /// An independent library with the same fonts (for a worker thread).
    pub fn fork(&self) -> Self {
        Self {
            system: font_system(self.system.db().clone()),
            families: self.families.clone(),
            pending: None,
            system_requested: true,
            generation: self.generation,
        }
    }

    pub fn is_loading(&self) -> bool {
        self.pending.is_some()
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    fn refresh_families(&mut self) {
        let mut others: Vec<String> = self
            .system
            .db()
            .faces()
            .filter_map(|f| f.families.first().map(|(name, _)| name.clone()))
            .filter(|name| {
                !BUNDLED_FAMILIES
                    .iter()
                    .any(|b| b.eq_ignore_ascii_case(name))
            })
            .filter(|name| !name.starts_with('.'))
            .collect();
        others.sort_by_key(|n| n.to_lowercase());
        others.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
        self.families = BUNDLED_FAMILIES.iter().map(|s| (*s).to_owned()).collect();
        self.families.extend(others);
    }

    /// Family names: bundled first, then installed fonts, deduplicated.
    pub fn families(&self) -> &[String] {
        &self.families
    }

    /// Whether a family is available (case-insensitive).
    pub fn has_family(&self, family: &str) -> bool {
        self.families.iter().any(|f| f.eq_ignore_ascii_case(family))
    }

    /// The family name to lay out with, and whether the requested one was
    /// missing (then the fallback is used).
    pub fn resolve_family<'a>(&'a self, family: &'a str) -> (&'a str, bool) {
        match self
            .families
            .iter()
            .find(|f| f.eq_ignore_ascii_case(family))
        {
            Some(found) => (found.as_str(), false),
            None => (FALLBACK_FAMILY, true),
        }
    }

    /// Weights available for a family (static faces, or the variable range
    /// sampled at standard steps).
    pub fn weights(&self, family: &str) -> Vec<u16> {
        let (family, _) = self.resolve_family(family);
        let mut weights: Vec<u16> = Vec::new();
        for face in self.system.db().faces() {
            if !face
                .families
                .iter()
                .any(|(n, _)| n.eq_ignore_ascii_case(family))
            {
                continue;
            }
            let variable = self
                .system
                .db()
                .with_face_data(face.id, |data, index| {
                    use skrifa::MetadataProvider;
                    skrifa::FontRef::from_index(data, index)
                        .ok()
                        .and_then(|f| {
                            f.axes()
                                .iter()
                                .find(|a| a.tag() == skrifa::Tag::new(b"wght"))
                        })
                        .map(|axis| (axis.min_value(), axis.max_value()))
                })
                .flatten();
            match variable {
                Some((min, max)) => weights.extend(
                    (1..=9)
                        .map(|w| w * 100)
                        .filter(|w| f32::from(*w) >= min && f32::from(*w) <= max),
                ),
                None => weights.push(face.weight.0),
            }
        }
        weights.sort_unstable();
        weights.dedup();
        if weights.is_empty() {
            weights.push(400);
        }
        weights
    }

    /// Nearest available weight for a family.
    pub fn nearest_weight(&self, family: &str, weight: u16) -> u16 {
        self.weights(family)
            .into_iter()
            .min_by_key(|w| (i32::from(*w) - i32::from(weight)).abs())
            .unwrap_or(400)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_families_come_first() {
        let lib = FontLibrary::bundled();
        assert_eq!(&lib.families()[..5], &BUNDLED_FAMILIES.map(String::from));
        assert!(lib.has_family("oswald"));
    }

    #[test]
    fn missing_family_falls_back_to_inter() {
        let lib = FontLibrary::bundled();
        assert_eq!(lib.resolve_family("Some Missing Font"), ("Inter", true));
        assert_eq!(lib.resolve_family("bebas neue"), ("Bebas Neue", false));
    }

    #[test]
    fn weights_and_nearest_weight() {
        let lib = FontLibrary::bundled();
        assert_eq!(lib.weights("Bebas Neue"), vec![400]);
        assert_eq!(lib.nearest_weight("Bebas Neue", 900), 400);
        let oswald = lib.weights("Oswald");
        assert!(oswald.contains(&200) && oswald.contains(&700), "{oswald:?}");
        assert_eq!(lib.nearest_weight("Barlow Condensed", 650), 600);
    }
}
