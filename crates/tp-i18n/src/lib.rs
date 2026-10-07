//! Interface languages: Fluent messages embedded in the binary, the current
//! language of the calling thread, and localized numbers.
//!
//! English is the source language: a message missing from another language
//! is taken from English. The current language is per thread (the UI thread
//! sets it every frame), so parallel tests never see each other's language.

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use unic_langid::LanguageIdentifier;

pub use fluent_bundle::FluentArgs as Args;

/// An interface language.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Language {
    #[default]
    English,
    French,
    Spanish,
    German,
}

impl Language {
    pub const ALL: [Language; 4] = [
        Language::English,
        Language::French,
        Language::Spanish,
        Language::German,
    ];

    /// The language's name in that language.
    pub fn native_name(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::French => "Français",
            Language::Spanish => "Español",
            Language::German => "Deutsch",
        }
    }

    /// BCP 47 code (`en`, `fr`, `es`, `de`).
    pub fn code(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::French => "fr",
            Language::Spanish => "es",
            Language::German => "de",
        }
    }

    pub fn from_code(code: &str) -> Option<Language> {
        Language::ALL.into_iter().find(|l| l.code() == code)
    }

    /// The language for a system locale (`fr-CA`, `de_AT.UTF-8`, …): its
    /// primary language if supported, English otherwise.
    pub fn from_locale(locale: &str) -> Language {
        let primary = locale
            .split(['-', '_', '.', '@'])
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        Language::from_code(&primary).unwrap_or(Language::English)
    }

    fn index(self) -> usize {
        self as usize
    }
}

/// Message files, in the order they are loaded.
pub const FILES: [&str; 7] = [
    "common", "menus", "commands", "panels", "dialogs", "canvas", "objects",
];

macro_rules! sources {
    ($lang:literal) => {
        [
            include_str!(concat!("../locales/", $lang, "/common.ftl")),
            include_str!(concat!("../locales/", $lang, "/menus.ftl")),
            include_str!(concat!("../locales/", $lang, "/commands.ftl")),
            include_str!(concat!("../locales/", $lang, "/panels.ftl")),
            include_str!(concat!("../locales/", $lang, "/dialogs.ftl")),
            include_str!(concat!("../locales/", $lang, "/canvas.ftl")),
            include_str!(concat!("../locales/", $lang, "/objects.ftl")),
        ]
    };
}

/// The `.ftl` sources of `language`, in [`FILES`] order.
pub fn sources(language: Language) -> [&'static str; 7] {
    match language {
        Language::English => sources!("en"),
        Language::French => sources!("fr"),
        Language::Spanish => sources!("es"),
        Language::German => sources!("de"),
    }
}

fn build_bundle(language: Language) -> FluentBundle<FluentResource> {
    let id: LanguageIdentifier = language.code().parse().expect("valid language code");
    let mut bundle = FluentBundle::new(vec![id]);
    // Unicode isolation marks would show as boxes in the UI.
    bundle.set_use_isolating(false);
    for (file, source) in FILES.iter().zip(sources(language)) {
        let resource = match FluentResource::try_new(source.to_owned()) {
            Ok(r) => r,
            Err((r, errors)) => {
                tracing::error!(?errors, "syntax errors in {}/{file}.ftl", language.code());
                r
            }
        };
        if let Err(errors) = bundle.add_resource(resource) {
            tracing::error!(
                ?errors,
                "duplicate messages in {}/{file}.ftl",
                language.code()
            );
        }
    }
    bundle
}

thread_local! {
    static CURRENT: Cell<Language> = const { Cell::new(Language::English) };
    static BUNDLES: RefCell<[Option<FluentBundle<FluentResource>>; 4]> =
        const { RefCell::new([None, None, None, None]) };
    /// Formatted messages without arguments.
    static CACHE: RefCell<HashMap<(Language, String), String>> = RefCell::new(HashMap::new());
    static WARNED: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

/// Sets the language of the calling thread.
pub fn set_language(language: Language) {
    CURRENT.with(|c| c.set(language));
}

/// The language of the calling thread (English unless set).
pub fn current() -> Language {
    CURRENT.with(Cell::get)
}

/// Formats message `id` in `language`, if it exists there.
fn format_in(language: Language, id: &str, args: Option<&FluentArgs<'_>>) -> Option<String> {
    BUNDLES.with(|bundles| {
        let mut bundles = bundles.borrow_mut();
        let bundle = bundles[language.index()].get_or_insert_with(|| build_bundle(language));
        let pattern = bundle.get_message(id)?.value()?;
        let mut errors = Vec::new();
        let text = bundle.format_pattern(pattern, args, &mut errors);
        if !errors.is_empty() {
            tracing::warn!(?errors, "formatting {id} in {}", language.code());
        }
        Some(text.into_owned())
    })
}

fn format(id: &str, args: Option<&FluentArgs<'_>>) -> String {
    let language = current();
    if let Some(text) = format_in(language, id, args) {
        return text;
    }
    if language != Language::English
        && let Some(text) = format_in(Language::English, id, args)
    {
        return text;
    }
    WARNED.with(|w| {
        if w.borrow_mut().insert(id.to_owned()) {
            tracing::warn!("missing message {id}");
        }
    });
    id.to_owned()
}

/// Message `id` in the current language (English when missing there).
pub fn tr(id: &str) -> String {
    let key = (current(), id.to_owned());
    if let Some(hit) = CACHE.with(|c| c.borrow().get(&key).cloned()) {
        return hit;
    }
    let text = format(id, None);
    CACHE.with(|c| c.borrow_mut().insert(key, text.clone()));
    text
}

/// Message `id` with arguments, in the current language.
pub fn tr_args(id: &str, args: &FluentArgs<'_>) -> String {
    format(id, Some(args))
}

/// Whether message `id` exists in English (the source language).
pub fn exists(id: &str) -> bool {
    format_in(Language::English, id, None).is_some()
}

/// `tr!("id")` or `tr!("id", name = value, …)`: a message in the current
/// language. Values may be numbers or strings.
#[macro_export]
macro_rules! tr {
    ($id:expr) => {
        $crate::tr($id)
    };
    ($id:expr, $($name:ident = $value:expr),+ $(,)?) => {{
        let mut args = $crate::Args::new();
        $( args.set(stringify!($name), $crate::value($value)); )+
        $crate::tr_args($id, &args)
    }};
}

/// Converts a macro argument to a Fluent value.
pub fn value<'a>(v: impl Into<FluentValue<'a>>) -> FluentValue<'a> {
    v.into()
}

/// Decimal separator of the current language.
pub fn decimal_separator() -> char {
    match current() {
        Language::English => '.',
        Language::French | Language::Spanish | Language::German => ',',
    }
}

/// `v` with at most `decimals` decimals (trailing zeros dropped) and the
/// current language's decimal separator.
pub fn format_number(v: f64, decimals: usize) -> String {
    let text = format!("{v:.decimals$}");
    let text = if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.').to_owned()
    } else {
        text
    };
    let text = if text == "-0" { "0".to_owned() } else { text };
    localize_number(&text).into_owned()
}

/// `text` (a number written with a point) with the current decimal separator.
pub fn localize_number(text: &str) -> Cow<'_, str> {
    match decimal_separator() {
        '.' => Cow::Borrowed(text),
        sep => Cow::Owned(text.replace('.', &sep.to_string())),
    }
}

#[cfg(test)]
mod tests;
