use std::collections::{BTreeMap, BTreeSet};

use super::*;

fn ids(l: Language) -> BTreeMap<String, BTreeSet<String>> {
    let mut map = BTreeMap::new();
    for (id, vars) in messages(l) {
        assert!(map.insert(id.clone(), vars).is_none(), "{}: {id} twice", l.code());
    }
    map
}

/// Every language has every English message, nothing English lacks, and no variable that
/// English does not pass.
#[test]
fn every_language_is_complete() {
    let en = ids(Language::En);
    assert!(!en.is_empty());
    let mut problems = Vec::new();
    for l in Language::ALL.into_iter().skip(1) {
        let other = ids(l);
        if other.is_empty() {
            continue;
        }
        for id in en.keys().filter(|id| !other.contains_key(*id)) {
            problems.push(format!("{}: {id} missing", l.code()));
        }
        for (id, vars) in &other {
            match en.get(id) {
                None => problems.push(format!("{}: {id} is not in English", l.code())),
                Some(en_vars) => {
                    for v in vars.difference(en_vars) {
                        problems.push(format!("{}: {id} uses ${v}, English does not", l.code()));
                    }
                }
            }
        }
    }
    assert!(problems.is_empty(), "{} problems:\n{}", problems.len(), problems.join("\n"));
}

/// Every language offered has its files.
#[test]
fn every_language_is_translated() {
    for l in Language::ALL {
        assert!(!messages(l).is_empty(), "{}: no translation", l.code());
    }
}

/// The texts drawn with the flight font in a Latin language fold to ASCII without "?".
#[test]
fn flight_texts_fold_to_ascii() {
    for l in Language::ALL.into_iter().filter(|l| l.latin()) {
        for (id, _) in messages(l).into_iter().filter(|(id, _)| id.starts_with("hud-")) {
            let s = tr_in(l, &id);
            assert!(!ascii_upper(s).contains('?') || s.contains('?'), "{}: {id}: {s}", l.code());
        }
    }
}

#[test]
fn parse_codes_and_locales() {
    assert_eq!(Language::parse("fr"), Some(Language::Fr));
    assert_eq!(Language::parse("fr-CA"), Some(Language::Fr));
    assert_eq!(Language::parse("de_DE.UTF-8"), Some(Language::De));
    assert_eq!(Language::parse("pt"), Some(Language::PtBr));
    assert_eq!(Language::parse("PT-br"), Some(Language::PtBr));
    assert_eq!(Language::parse("pseudo"), Some(Language::Pseudo));
    assert_eq!(Language::parse("ja-JP"), Some(Language::Ja));
    assert_eq!(Language::parse("zh-CN"), Some(Language::Zh));
    assert_eq!(Language::parse("ko-KR"), Some(Language::Ko));
    assert_eq!(Language::parse("ar-EG"), None);
    for l in Language::EVERY {
        assert_eq!(Language::parse(l.code()), Some(l));
    }
    let json = serde_json::to_string(&Language::PtBr).unwrap();
    assert_eq!(json, "\"pt-BR\"");
    assert_eq!(serde_json::from_str::<Language>(&json).unwrap(), Language::PtBr);
    assert_eq!(serde_json::from_str::<Language>("\"xx\"").unwrap(), Language::En);
}

#[test]
fn messages_arguments_and_fallback() {
    assert_eq!(tr_in(Language::En, "language"), "Language");
    assert_eq!(tr_in(Language::Fr, "language"), "Langue");
    assert_eq!(tr_in(Language::Fr, "no-such-message"), "no-such-message");
    // No isolation marks around the arguments.
    let s = t_in!(Language::En, "i18n-test-count", n = 1);
    assert_eq!(s, "1 hit");
    assert_eq!(t_in!(Language::En, "i18n-test-count", n = 3), "3 hits");
    assert_eq!(t_in!(Language::Fr, "i18n-test-count", n = 3), "3 impacts");
    assert_eq!(tr_lines_in(Language::En, "i18n-test-lines"), ["First line", "Second line"]);
    assert_ne!(tr_in(Language::Pseudo, "language"), "Language");
}

#[test]
fn texts_translate_where_shown_and_read_old_saves() {
    let t = text!("i18n-test-count", n = 2);
    assert_eq!(t.tr_in(Language::En), "2 hits");
    assert_eq!(t.tr_in(Language::Fr), "2 impacts");
    let back: Text = serde_json::from_str(&serde_json::to_string(&t).unwrap()).unwrap();
    assert_eq!(back, t);
    // A message saved as a plain string before the translations.
    let old: Text = serde_json::from_str("\"Ship sunk\"").unwrap();
    assert_eq!(old, Text::raw("Ship sunk"));
    assert_eq!(old.tr_in(Language::Fr), "Ship sunk");
    let nested = text!("i18n-test-nested", what = text!("language"));
    assert_eq!(nested.tr_in(Language::Fr), "Choisir : Langue");
}

#[test]
fn folding() {
    assert_eq!(fold_accents("Élève à l'école, Größe, Łódź"), "Eleve a l'ecole, Grosse, Lodz");
    assert_eq!(ascii_upper("Siège éjectable"), "SIEGE EJECTABLE");
    assert_eq!(ascii_upper("Straße"), "STRASSE");
    assert_eq!(ascii_upper("Посадка"), "???????");
    assert_eq!(ascii_upper("« Prêt »"), "\" PRET \"");
}
