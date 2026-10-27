//! The game's translations: Fluent files (`assets/i18n/<language>/*.ftl`, embedded by build.rs),
//! the language the player chose, and [`Text`], a message that crosses the network or goes into
//! a save untranslated and is translated where it is shown.
//!
//! English is the source: a message missing from a language is taken from English, one missing
//! from English shows its id. The cockpit's own words (HUD, MFD and panel labels, acronyms) are
//! not translated (`fichiers_md/translation.md`).
//!
//! ```ignore
//! t!("options-title")                      // "OPTIONS"
//! t!("options-tooltips.desc")              // an attribute
//! t!("debrief-hits", hits = 3)             // with arguments
//! ```

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{LazyLock, OnceLock, RwLock};

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentResource, FluentValue};
use serde::{Deserialize, Serialize};

pub use fluent_bundle::FluentArgs as Args;

mod fold;
mod text;

pub use fold::{ascii_upper, fold_accents};
pub use text::{Arg, Text};

/// `(language, file, text)` of every Fluent file.
const FILES: &[(&str, &str, &str)] = include!(concat!(env!("OUT_DIR"), "/ftl.rs"));

/// A language of the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub enum Language {
    #[default]
    En,
    /// `--lang pseudo`: English with accents and 30% longer, to see the untranslated texts and
    /// the ones that do not fit.
    Pseudo,
}

impl Language {
    /// The languages offered in OPTIONS, in this order.
    pub const ALL: [Language; 1] = [
        Language::En,
    ];

    const EVERY: [Language; 2] = [
        Language::En,
        Language::Pseudo,
    ];

    /// The folder under `assets/i18n` (a BCP 47 tag).
    pub fn code(self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Pseudo => "pseudo",
        }
    }

    /// The language's name in itself, as the OPTIONS list shows it.
    pub fn native_name(self) -> &'static str {
        match self {
            Language::En => "English",
            Language::Pseudo => "Ƥşḗŭḓǿ",
        }
    }

    /// A code or a system locale ("fr", "fr-FR", "fr_CA.UTF-8", "pt", "PT-br"): its language,
    /// Portuguese being Brazilian.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim().to_ascii_lowercase();
        if s == "pseudo" {
            return Some(Language::Pseudo);
        }
        let primary = s.split(['-', '_', '.', '@']).next()?;
        Language::ALL.into_iter().find(|l| l.code().split('-').next() == Some(primary))
    }

    /// Written in the Latin alphabet: the flight font (ASCII capitals) can show its words
    /// without their accents. The others' HUD and display messages stay in English.
    pub fn latin(self) -> bool {
        let _ = self;
        true
    }

    fn index(self) -> usize {
        self as usize
    }

    fn from_index(i: u8) -> Self {
        Language::EVERY.get(i as usize).copied().unwrap_or_default()
    }
}

impl Serialize for Language {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.code())
    }
}

impl<'de> Deserialize<'de> for Language {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(Language::parse(&s).unwrap_or_default())
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

/// The language the texts are shown in.
pub fn language() -> Language {
    Language::from_index(CURRENT.load(Ordering::Relaxed))
}

pub fn set_language(l: Language) {
    CURRENT.store(l.index() as u8, Ordering::Relaxed);
}

/// The language of the texts drawn with the flight font: the player's, or English where its
/// alphabet is not Latin.
pub fn flight_language() -> Language {
    let l = language();
    if l.latin() { l } else { Language::En }
}

type Bundle = FluentBundle<FluentResource>;

fn pseudo(s: &str) -> Cow<'_, str> {
    fluent_pseudo::transform(s, false, true)
}

fn build(l: Language) -> Bundle {
    let source = if l == Language::Pseudo { Language::En } else { l };
    let id = source.code().parse().expect("language id");
    let mut b = Bundle::new_concurrent(vec![id]);
    // No Unicode isolation marks around the arguments: the flight fonts would draw them.
    b.set_use_isolating(false);
    if l == Language::Pseudo {
        b.set_transform(Some(pseudo));
    }
    for (lang, file, text) in FILES {
        if *lang != source.code() {
            continue;
        }
        let res = match FluentResource::try_new(text.to_string()) {
            Ok(r) => r,
            Err((r, errors)) => {
                eprintln!("i18n: {lang}/{file}: {errors:?}");
                r
            }
        };
        if let Err(errors) = b.add_resource(res) {
            eprintln!("i18n: {lang}/{file}: {errors:?}");
        }
    }
    b
}

fn bundle(l: Language) -> &'static Bundle {
    static BUNDLES: [OnceLock<Bundle>; 9] = [const { OnceLock::new() }; 9];
    BUNDLES[l.index()].get_or_init(|| build(l))
}

/// Formats `id` ("message" or "message.attribute") in `l`; None if `l` does not have it.
fn format(l: Language, id: &str, args: Option<&Args>) -> Option<String> {
    let b = bundle(l);
    let (msg, attr) = match id.split_once('.') {
        Some((m, a)) => (m, Some(a)),
        None => (id, None),
    };
    let m = b.get_message(msg)?;
    let pattern = match attr {
        Some(a) => m.get_attribute(a)?.value(),
        None => m.value()?,
    };
    let mut errors = Vec::new();
    let s = b.format_pattern(pattern, args, &mut errors).into_owned();
    if cfg!(debug_assertions) && !errors.is_empty() {
        eprintln!("i18n: {}/{id}: {errors:?}", l.code());
    }
    Some(s)
}

/// `id` in `l`, else in English, else the id itself.
pub fn tr_args_in(l: Language, id: &str, args: Option<&Args>) -> String {
    if let Some(s) = format(l, id, args) {
        return s;
    }
    if l != Language::En
        && l != Language::Pseudo
        && let Some(s) = format(Language::En, id, args)
    {
        return s;
    }
    missing(id);
    id.to_string()
}

fn missing(id: &str) {
    if !cfg!(debug_assertions) {
        return;
    }
    static SEEN: LazyLock<RwLock<std::collections::HashSet<String>>> =
        LazyLock::new(Default::default);
    if SEEN.write().unwrap().insert(id.to_string()) {
        eprintln!("i18n: no message {id}");
    }
}

/// [`tr_args_in`] in the player's language.
pub fn tr_args(id: &str, args: &Args) -> String {
    tr_args_in(language(), id, Some(args))
}

/// A message without arguments in `l`, kept for the rest of the run (texts drawn every frame).
pub fn tr_in(l: Language, id: &str) -> &'static str {
    static CACHE: LazyLock<RwLock<HashMap<(Language, String), &'static str>>> =
        LazyLock::new(Default::default);
    if let Some(s) = CACHE.read().unwrap().get(&(l, id.to_string())) {
        return s;
    }
    let s: &'static str = Box::leak(tr_args_in(l, id, None).into_boxed_str());
    CACHE.write().unwrap().insert((l, id.to_string()), s);
    s
}

/// A message without arguments in the player's language.
pub fn tr(id: &str) -> &'static str {
    tr_in(language(), id)
}

/// A message of several lines (the facts under a menu choice), one string per line, kept for
/// the rest of the run.
pub fn tr_lines_in(l: Language, id: &str) -> &'static [&'static str] {
    type Lines = HashMap<(Language, String), &'static [&'static str]>;
    static CACHE: LazyLock<RwLock<Lines>> = LazyLock::new(Default::default);
    if let Some(s) = CACHE.read().unwrap().get(&(l, id.to_string())) {
        return s;
    }
    let lines: Vec<&'static str> =
        tr_in(l, id).lines().map(str::trim).filter(|s| !s.is_empty()).collect();
    let s: &'static [&'static str] = Box::leak(lines.into_boxed_slice());
    CACHE.write().unwrap().insert((l, id.to_string()), s);
    s
}

/// [`tr_lines_in`] in the player's language.
pub fn tr_lines(id: &str) -> &'static [&'static str] {
    tr_lines_in(language(), id)
}

/// Whether English has `id`.
pub fn exists(id: &str) -> bool {
    exists_in(Language::En, id)
}

/// Whether language `l` has `id` itself (no English fallback).
pub fn exists_in(l: Language, id: &str) -> bool {
    let (msg, attr) = match id.split_once('.') {
        Some((m, a)) => (m, Some(a)),
        None => (id, None),
    };
    bundle(l).get_message(msg).is_some_and(|m| match attr {
        Some(a) => m.get_attribute(a).is_some(),
        None => m.value().is_some(),
    })
}

/// Converts a value for [`Args::set`] (numbers, strings).
pub fn value<'a>(v: impl Into<FluentValue<'a>>) -> FluentValue<'a> {
    v.into()
}

/// A message in the player's language: `t!("id")` (a `&'static str`) or
/// `t!("id", name = value, ...)` (a `String`).
#[macro_export]
macro_rules! t {
    ($id:expr) => {
        $crate::tr($id)
    };
    ($id:expr, $($k:ident = $v:expr),+ $(,)?) => {{
        let mut a = $crate::Args::new();
        $(a.set(stringify!($k), $v);)+
        $crate::tr_args($id, &a)
    }};
}

/// [`t!`] in a given language: `t_in!(lang, "id")`, `t_in!(lang, "id", name = value)`.
#[macro_export]
macro_rules! t_in {
    ($lang:expr, $id:expr) => {
        $crate::tr_in($lang, $id)
    };
    ($lang:expr, $id:expr, $($k:ident = $v:expr),+ $(,)?) => {{
        let mut a = $crate::Args::new();
        $(a.set(stringify!($k), $v);)+
        $crate::tr_args_in($lang, $id, Some(&a))
    }};
}

/// [`t!`] for the texts drawn with the flight fonts (HUD, displays, the in-flight menu): in
/// the player's language if it is written in Latin letters, else in English.
#[macro_export]
macro_rules! tf {
    ($id:expr) => {
        $crate::tr_in($crate::flight_language(), $id)
    };
    ($id:expr, $($k:ident = $v:expr),+ $(,)?) => {
        $crate::t_in!($crate::flight_language(), $id, $($k = $v),+)
    };
}

/// The lines of a message in the player's language (a `&'static [&'static str]`).
#[macro_export]
macro_rules! t_lines {
    ($id:expr) => {
        $crate::tr_lines($id)
    };
}

/// For the tests: the messages of a language, with their attributes and the variables each
/// one uses.
#[doc(hidden)]
pub fn messages(l: Language) -> Vec<(String, std::collections::BTreeSet<String>)> {
    let mut out = Vec::new();
    for (lang, file, text) in FILES {
        if *lang != l.code() {
            continue;
        }
        let res = FluentResource::try_new(text.to_string())
            .unwrap_or_else(|(_, e)| panic!("{lang}/{file}: {e:?}"));
        for entry in res.entries() {
            if let fluent_syntax::ast::Entry::Message(m) = entry {
                let mut vars = Default::default();
                if let Some(p) = &m.value {
                    text::variables(p, &mut vars);
                }
                out.push((m.id.name.to_string(), vars));
                for a in &m.attributes {
                    let mut vars = Default::default();
                    text::variables(&a.value, &mut vars);
                    out.push((format!("{}.{}", m.id.name, a.id.name), vars));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests;
