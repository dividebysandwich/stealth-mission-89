//! English text to the IPA phonemes Piper voices are trained on (eSpeak's inventory), without
//! eSpeak: a small lexicon of our own first, then CMUdict through `piper-plus-g2p`, then
//! spelling rules for the words neither has (most often an airfield's name: better a guessed
//! "Gutersloh" than one read letter by letter). Short all-capital words (GCA, SAM) are spelled.

use std::collections::HashMap;
use std::sync::OnceLock;

use piper_plus_g2p::english::EnglishPhonemizer;
use piper_plus_g2p::phonemizer::Phonemizer;

/// CMUdict (BSD-2-Clause, its notice in `assets/LICENSE-cmudict.txt`, which ships with the
/// game): lowercase words, one pronunciation each, in ARPAbet.
const CMUDICT: &str = include_str!("../../assets-src/cmudict/cmudict.dict");

fn cmudict() -> &'static HashMap<String, String> {
    static DICT: OnceLock<HashMap<String, String>> = OnceLock::new();
    DICT.get_or_init(|| {
        CMUDICT
            .lines()
            .filter_map(|l| l.split_once(' '))
            .map(|(w, a)| (w.to_string(), a.to_string()))
            .collect()
    })
}

/// Words the dictionary lacks or says wrong for the radio, in IPA.
const LEXICON: &[(&str, &str)] = &[
    ("bruggen", "bɹˈʊɡən"),
    ("gutersloh", "ɡˈuːtɚsloʊ"),
    ("helipad", "hˈɛlɪpæd"),
    ("kiowa", "kˈaɪəwə"),
    ("laarbruch", "lˈɑːbɹʊk"),
    ("niner", "nˈaɪnɚ"),
    ("wildenrath", "vˈɪldənɹɑːt"),
];

fn lexicon(word: &str) -> Option<&'static str> {
    LEXICON.iter().find(|(w, _)| *w == word).map(|(_, ipa)| *ipa)
}

pub struct Phonemes {
    en: EnglishPhonemizer,
}

impl Phonemes {
    /// Parses the dictionary (a fraction of a second: on the speech thread).
    pub fn new() -> Self {
        Self { en: EnglishPhonemizer::new_with_hashmap(cmudict().clone()) }
    }

    /// A phrase in IPA; punctuation stays (Piper pauses at it).
    pub fn to_ipa(&self, text: &str) -> String {
        let mut out = String::new();
        for word in text.split_whitespace() {
            let is_word = |c: char| c.is_alphabetic() || c == '\'';
            let lead = word.len() - word.trim_start_matches(|c| !is_word(c)).len();
            let (lead, rest) = word.split_at(lead);
            let core = rest.trim_end_matches(|c| !is_word(c)).len();
            let (core, tail) = rest.split_at(core);
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(lead);
            if !core.is_empty() {
                out.push_str(&self.word(core));
            }
            out.push_str(tail);
        }
        out
    }

    fn word(&self, word: &str) -> String {
        let lower = word.to_lowercase();
        if let Some(ipa) = lexicon(&lower) {
            return ipa.to_string();
        }
        let acronym =
            word.len() <= 4 && word.len() > 1 && word.chars().all(|c| c.is_ascii_uppercase());
        if acronym || word.len() == 1 {
            return self.spell(&lower);
        }
        let known = |w: &str| cmudict().contains_key(w);
        let derived = ["s", "es", "ed", "ing", "ly", "er"].iter().any(|suf| {
            lower
                .strip_suffix(suf)
                .is_some_and(|stem| stem.len() >= 3 && (known(stem) || known(&format!("{stem}e"))))
        });
        if known(&lower) || derived {
            let ipa = self.dictionary(&lower);
            if !ipa.is_empty() {
                return ipa;
            }
        }
        rules(&lower)
    }

    fn dictionary(&self, word: &str) -> String {
        self.en.phonemize_with_prosody(word).map(|(t, _)| t.concat()).unwrap_or_default()
    }

    /// Letter by letter, a beat between.
    fn spell(&self, word: &str) -> String {
        word.chars()
            .filter_map(letter_name)
            .map(|n| self.dictionary(n))
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// A letter's name, as a word in the dictionary.
fn letter_name(c: char) -> Option<&'static str> {
    const NAMES: [&str; 26] = [
        "ay",
        "bee",
        "see",
        "dee",
        "e",
        "f",
        "gee",
        "h",
        "i",
        "jay",
        "kay",
        "l",
        "m",
        "n",
        "o",
        "pea",
        "queue",
        "r",
        "s",
        "tea",
        "you",
        "vee",
        "double-you",
        "x",
        "why",
        "zee",
    ];
    c.is_ascii_lowercase().then(|| NAMES[(c as u8 - b'a') as usize])
}

/// A guess from the spelling, for names the dictionary doesn't have: the longest matching
/// letter group first, the stress on the first vowel.
fn rules(word: &str) -> String {
    const GROUPS: &[(&str, &str)] = &[
        ("tsch", "tʃ"),
        ("sch", "ʃ"),
        ("tch", "tʃ"),
        ("ch", "tʃ"),
        ("sh", "ʃ"),
        ("th", "θ"),
        ("ph", "f"),
        ("ck", "k"),
        ("ng", "ŋ"),
        ("qu", "kw"),
        ("gh", "ɡ"),
        ("kh", "k"),
        ("zh", "ʒ"),
        ("wh", "w"),
        ("ee", "iː"),
        ("ea", "iː"),
        ("ie", "iː"),
        ("oo", "uː"),
        ("ou", "aʊ"),
        ("ow", "oʊ"),
        ("oh", "oʊ"),
        ("oa", "oʊ"),
        ("ai", "eɪ"),
        ("ay", "eɪ"),
        ("ei", "aɪ"),
        ("au", "ɔː"),
        ("aa", "ɑː"),
        ("ah", "ɑː"),
        ("ue", "uː"),
        ("oi", "ɔɪ"),
        ("oy", "ɔɪ"),
        ("eu", "ɔɪ"),
        ("er", "ɚ"),
        ("ar", "ɑːɹ"),
        ("or", "ɔːɹ"),
    ];
    let chars: Vec<char> = word.chars().filter(|c| c.is_alphabetic()).collect();
    let mut out = String::new();
    let mut stressed = false;
    let mut i = 0;
    while i < chars.len() {
        let rest: String = chars[i..].iter().collect();
        let next = chars.get(i + 1).copied();
        let vowel = |c: char| "aeiouy".contains(c);
        let (ipa, len) = if let Some((g, ipa)) = GROUPS.iter().find(|(g, _)| rest.starts_with(g)) {
            (ipa.to_string(), g.chars().count())
        } else {
            let c = chars[i];
            let ipa = match c {
                // A silent final e.
                'e' if i + 1 == chars.len() && i >= 3 => "",
                'a' => "a",
                'e' => "ɛ",
                'i' => "ɪ",
                'o' => "ɒ",
                'u' => "ʊ",
                'y' if i == 0 => "j",
                'y' => "i",
                'c' if next.is_some_and(|n| "eiy".contains(n)) => "s",
                'c' | 'k' | 'q' => "k",
                'g' => "ɡ",
                'j' => "dʒ",
                'x' => "ks",
                'r' => "ɹ",
                'b' => "b",
                'd' => "d",
                'f' => "f",
                'h' => "h",
                'l' => "l",
                'm' => "m",
                'n' => "n",
                'p' => "p",
                's' => "s",
                't' => "t",
                'v' => "v",
                'w' => "w",
                'z' => "z",
                _ => "",
            };
            // A doubled consonant is one.
            if !vowel(c) && next == Some(c) {
                i += 1;
            }
            let ipa = ipa.to_string();
            (ipa, 1)
        };
        let starts_vowel = ipa.chars().next().is_some_and(|c| "aɛɪɒʊiɚɑɔoueə".contains(c));
        if starts_vowel && !stressed {
            out.push('ˈ');
            stressed = true;
        }
        out.push_str(&ipa);
        i += len;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dictionary_lexicon_rules_and_spelling() {
        let p = Phonemes::new();
        assert!(!p.to_ipa("runway").is_empty());
        assert_eq!(p.to_ipa("Gutersloh"), "ɡˈuːtɚsloʊ");
        // Unknown names are guessed, not spelled; acronyms are spelled.
        let guess = p.to_ipa("Zweibrucken");
        assert!(guess.contains('ˈ') && !guess.contains(' '), "{guess}");
        assert_eq!(p.to_ipa("GCA").split(' ').count(), 3);
        // Punctuation stays where it was.
        assert!(p.to_ipa("tower, cleared.").contains(", ") && p.to_ipa("cleared.").ends_with('.'));
    }
}
