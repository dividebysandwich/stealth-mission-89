//! Latin letters without their accents: the flight font has ASCII capitals only, and the
//! KEY GUIDE search finds "écran" from "ecran".

/// The ASCII letters of a Latin letter with diacritics or a ligature, None for the others.
fn base(c: char) -> Option<&'static str> {
    Some(match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => "a",
        'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'Ā' | 'Ă' | 'Ą' => "A",
        'æ' => "ae",
        'Æ' => "AE",
        'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => "c",
        'Ç' | 'Ć' | 'Ĉ' | 'Ċ' | 'Č' => "C",
        'ď' | 'đ' => "d",
        'Ď' | 'Đ' => "D",
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => "e",
        'È' | 'É' | 'Ê' | 'Ë' | 'Ē' | 'Ĕ' | 'Ė' | 'Ę' | 'Ě' => "E",
        'ĝ' | 'ğ' | 'ġ' | 'ģ' => "g",
        'Ĝ' | 'Ğ' | 'Ġ' | 'Ģ' => "G",
        'ĥ' | 'ħ' => "h",
        'Ĥ' | 'Ħ' => "H",
        'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' => "i",
        'Ì' | 'Í' | 'Î' | 'Ï' | 'Ĩ' | 'Ī' | 'Ĭ' | 'Į' | 'İ' => "I",
        'ĵ' => "j",
        'Ĵ' => "J",
        'ķ' => "k",
        'Ķ' => "K",
        'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => "l",
        'Ĺ' | 'Ļ' | 'Ľ' | 'Ŀ' | 'Ł' => "L",
        'ñ' | 'ń' | 'ņ' | 'ň' => "n",
        'Ñ' | 'Ń' | 'Ņ' | 'Ň' => "N",
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ŏ' | 'ő' => "o",
        'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' | 'Ō' | 'Ŏ' | 'Ő' => "O",
        'œ' => "oe",
        'Œ' => "OE",
        'ŕ' | 'ŗ' | 'ř' => "r",
        'Ŕ' | 'Ŗ' | 'Ř' => "R",
        'ś' | 'ŝ' | 'ş' | 'š' | 'ș' => "s",
        'Ś' | 'Ŝ' | 'Ş' | 'Š' | 'Ș' => "S",
        'ß' => "ss",
        'ţ' | 'ť' | 'ŧ' | 'ț' => "t",
        'Ţ' | 'Ť' | 'Ŧ' | 'Ț' => "T",
        'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => "u",
        'Ù' | 'Ú' | 'Û' | 'Ü' | 'Ũ' | 'Ū' | 'Ŭ' | 'Ů' | 'Ű' | 'Ų' => "U",
        'ŵ' => "w",
        'Ŵ' => "W",
        'ý' | 'ÿ' | 'ŷ' => "y",
        'Ý' | 'Ÿ' | 'Ŷ' => "Y",
        'ź' | 'ż' | 'ž' => "z",
        'Ź' | 'Ż' | 'Ž' => "Z",
        // Typography of the translations.
        '\u{a0}' | '\u{202f}' | '\u{2009}' => " ",
        '‘' | '’' | '‚' | '′' => "'",
        '“' | '”' | '„' | '«' | '»' | '″' => "\"",
        '–' | '—' | '‑' | '−' => "-",
        '…' => "...",
        '¡' => "!",
        '¿' => "?",
        'º' | '°' => "o",
        _ => return None,
    })
}

/// `s` with its Latin letters stripped of their accents ("Élève" -> "Eleve"); other characters
/// stay.
pub fn fold_accents(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match base(c) {
            Some(b) => out.push_str(b),
            None => out.push(c),
        }
    }
    out
}

/// `s` in ASCII capitals for the flight font: accents dropped, "ß" as "SS", any other
/// character as "?".
pub fn ascii_upper(s: &str) -> String {
    fold_accents(s)
        .chars()
        .map(|c| if c.is_ascii() { c.to_ascii_uppercase() } else { '?' })
        .collect()
}
