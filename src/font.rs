//! The flight font (3x5 pixels, advance 4, drawn below as ASCII art) and text drawing into
//! palette canvases.
//!
//! Strings are byte strings: bytes >= 0x80 change the foreground colour to `b & 0x7F`
//! (use [`ink`]), 0 ends the string. Lower case is drawn as upper case.

use bevy::prelude::*;
use stealth_formats::font::{Font, Glyph};

use crate::canvas::Canvas;

/// Font used for all HUD, MFD and panel text in flight.
pub const HUD_FONT: usize = 0;

#[derive(Resource, Clone)]
pub struct Fonts(pub Vec<Font>);

impl Fonts {
    pub fn builtin() -> Self {
        Self(vec![flight_font()])
    }

    pub fn font(&self, i: usize) -> &Font {
        &self.0[i.min(self.0.len() - 1)]
    }

    /// Draws `s` with the flight font (transparent background). Returns the x after the text.
    pub fn text(&self, c: &mut Canvas, x: i32, y: i32, s: &[u8], fg: u8) -> i32 {
        draw_text(c, self.font(HUD_FONT), x, y, s, fg, None)
    }

    /// `text` with every font pixel drawn as a `scale` x `scale` block.
    pub fn text_scaled(&self, c: &mut Canvas, x: i32, y: i32, s: &[u8], fg: u8, scale: i32) -> i32 {
        draw_text_scaled(c, self.font(HUD_FONT), x, y, s, fg, None, scale)
    }

    /// Width of `s` in unscaled pixels (colour-change bytes take no room).
    pub fn width(&self, s: &[u8]) -> i32 {
        let f = self.font(HUD_FONT);
        s.iter()
            .take_while(|&&b| b != 0)
            .filter(|&&b| b < 0x80)
            .filter_map(|&b| f.glyph(b))
            .map(|g| g.advance as i32)
            .sum()
    }

    /// Glyph height of the flight font in unscaled pixels.
    pub fn height(&self) -> i32 {
        self.font(HUD_FONT).height as i32
    }
}

/// Glyphs of the flight font, `#` = set pixel.
const GLYPHS: &[(char, [&str; 5])] = &[
    (' ', ["...", "...", "...", "...", "..."]),
    ('!', [".#.", ".#.", ".#.", "...", ".#."]),
    ('"', ["#.#", "#.#", "...", "...", "..."]),
    ('#', ["#.#", "###", "#.#", "###", "#.#"]),
    ('$', [".##", "##.", ".#.", ".##", "##."]),
    ('%', ["#..", "..#", ".#.", "#..", "..#"]),
    ('&', [".#.", "#.#", ".#.", "#.#", ".##"]),
    ('\'', [".#.", ".#.", "...", "...", "..."]),
    ('(', ["..#", ".#.", ".#.", ".#.", "..#"]),
    (')', ["#..", ".#.", ".#.", ".#.", "#.."]),
    ('*', ["...", "#.#", ".#.", "#.#", "..."]),
    ('+', ["...", ".#.", "###", ".#.", "..."]),
    (',', ["...", "...", "...", ".#.", "#.."]),
    ('-', ["...", "...", "###", "...", "..."]),
    ('.', ["...", "...", "...", "...", ".#."]),
    ('/', ["..#", "..#", ".#.", "#..", "#.."]),
    ('0', ["###", "#.#", "#.#", "#.#", "###"]),
    ('1', [".#.", "##.", ".#.", ".#.", "###"]),
    ('2', ["###", "..#", "###", "#..", "###"]),
    ('3', ["###", "..#", ".##", "..#", "###"]),
    ('4', ["#.#", "#.#", "###", "..#", "..#"]),
    ('5', ["###", "#..", "###", "..#", "###"]),
    ('6', ["###", "#..", "###", "#.#", "###"]),
    ('7', ["###", "..#", "..#", ".#.", ".#."]),
    ('8', ["###", "#.#", "###", "#.#", "###"]),
    ('9', ["###", "#.#", "###", "..#", "###"]),
    (':', ["...", ".#.", "...", ".#.", "..."]),
    (';', ["...", ".#.", "...", ".#.", "#.."]),
    ('<', ["..#", ".#.", "#..", ".#.", "..#"]),
    ('=', ["...", "###", "...", "###", "..."]),
    ('>', ["#..", ".#.", "..#", ".#.", "#.."]),
    ('?', ["###", "..#", ".##", "...", ".#."]),
    ('@', ["###", "#.#", "###", "#..", "###"]),
    ('A', [".#.", "#.#", "###", "#.#", "#.#"]),
    ('B', ["##.", "#.#", "##.", "#.#", "##."]),
    ('C', [".##", "#..", "#..", "#..", ".##"]),
    ('D', ["##.", "#.#", "#.#", "#.#", "##."]),
    ('E', ["###", "#..", "##.", "#..", "###"]),
    ('F', ["###", "#..", "##.", "#..", "#.."]),
    ('G', [".##", "#..", "#.#", "#.#", ".##"]),
    ('H', ["#.#", "#.#", "###", "#.#", "#.#"]),
    ('I', ["###", ".#.", ".#.", ".#.", "###"]),
    ('J', ["..#", "..#", "..#", "#.#", ".#."]),
    ('K', ["#.#", "#.#", "##.", "#.#", "#.#"]),
    ('L', ["#..", "#..", "#..", "#..", "###"]),
    ('M', ["#.#", "###", "###", "#.#", "#.#"]),
    ('N', ["##.", "#.#", "#.#", "#.#", "#.#"]),
    ('O', [".#.", "#.#", "#.#", "#.#", ".#."]),
    ('P', ["##.", "#.#", "##.", "#..", "#.."]),
    ('Q', [".#.", "#.#", "#.#", "##.", ".##"]),
    ('R', ["##.", "#.#", "##.", "#.#", "#.#"]),
    ('S', [".##", "#..", ".#.", "..#", "##."]),
    ('T', ["###", ".#.", ".#.", ".#.", ".#."]),
    ('U', ["#.#", "#.#", "#.#", "#.#", "###"]),
    ('V', ["#.#", "#.#", "#.#", "#.#", ".#."]),
    ('W', ["#.#", "#.#", "###", "###", "#.#"]),
    ('X', ["#.#", "#.#", ".#.", "#.#", "#.#"]),
    ('Y', ["#.#", "#.#", ".#.", ".#.", ".#."]),
    ('Z', ["###", "..#", ".#.", "#..", "###"]),
    ('[', ["##.", "#..", "#..", "#..", "##."]),
    ('\\', ["#..", "#..", ".#.", "..#", "..#"]),
    (']', [".##", "..#", "..#", "..#", ".##"]),
    ('^', [".#.", "#.#", "...", "...", "..."]),
    ('_', ["...", "...", "...", "...", "###"]),
    ('`', ["#..", ".#.", "...", "...", "..."]),
    ('{', [".##", ".#.", "##.", ".#.", ".##"]),
    ('|', [".#.", ".#.", ".#.", ".#.", ".#."]),
    ('}', ["##.", ".#.", ".##", ".#.", "##."]),
    ('~', ["...", ".##", "##.", "...", "..."]),
];

/// Builds the flight font: printable ASCII, lower case sharing the upper-case glyphs, and a
/// blank row below each glyph for opaque backgrounds.
fn flight_font() -> Font {
    let (first, last) = (b' ', b'~');
    let glyphs = (first..=last)
        .map(|code| {
            let ch = code.to_ascii_uppercase() as char;
            let art = GLYPHS.iter().find(|(c, _)| *c == ch).map_or(["..."; 5], |(_, a)| *a);
            let mut rows: Vec<u16> = art
                .iter()
                .map(|r| {
                    r.bytes()
                        .enumerate()
                        .filter(|&(_, b)| b == b'#')
                        .fold(0, |m, (x, _)| m | 0x8000 >> x)
                })
                .collect();
            rows.push(0);
            Glyph { code, width: 3, advance: 4, rows }
        })
        .collect();
    Font { index: 0, first, last, height: 5, extra_height: 1, fixed_width: 3, spacing: 1, glyphs }
}

/// Colour-change byte for strings.
pub const fn ink(colour: u8) -> u8 {
    0x80 | (colour & 0x7F)
}

/// Upper-cases a string before drawing (flight text is upper case): the translations' accents
/// dropped, so that no byte of a UTF-8 letter is taken for a colour change.
pub fn upper(s: &str) -> Vec<u8> {
    stealth_i18n::ascii_upper(s).into_bytes()
}

pub fn draw_text(
    c: &mut Canvas,
    f: &Font,
    x: i32,
    y: i32,
    s: &[u8],
    fg: u8,
    bg: Option<u8>,
) -> i32 {
    draw_text_scaled(c, f, x, y, s, fg, bg, 1)
}

#[allow(clippy::too_many_arguments)]
pub fn draw_text_scaled(
    c: &mut Canvas,
    f: &Font,
    mut x: i32,
    y: i32,
    s: &[u8],
    mut fg: u8,
    bg: Option<u8>,
    scale: i32,
) -> i32 {
    let rows = if bg.is_some() { (f.height + f.extra_height) as i32 } else { f.height as i32 };
    for &b in s {
        if b == 0 {
            break;
        }
        if b >= 0x80 {
            fg = b & 0x7F;
            continue;
        }
        let Some(g) = f.glyph(b) else { continue };
        if let Some(bg) = bg {
            c.fill_rect(x, y, g.advance as i32 * scale, rows * scale, bg);
        }
        for r in 0..f.height as usize {
            for col in 0..g.width as usize {
                if g.pixel(col, r) {
                    if scale == 1 {
                        c.set(x + col as i32, y + r as i32, fg);
                    } else {
                        c.fill_rect(x + col as i32 * scale, y + r as i32 * scale, scale, scale, fg);
                    }
                }
            }
        }
        x += g.advance as i32 * scale;
    }
    x
}

/// Vector stroke font of the high-resolution HUD: each glyph is polylines on a 4 x 6 grid
/// (y down), separated by `|`.
const STROKES: &[(char, &str)] = &[
    ('0', "0,0 4,0 4,6 0,6 0,0"),
    ('1', "1,1 2,0 2,6|1,6 3,6"),
    ('2', "0,0 4,0 4,3 0,3 0,6 4,6"),
    ('3', "0,0 4,0 4,6 0,6|1,3 4,3"),
    ('4', "0,0 0,3 4,3|4,0 4,6"),
    ('5', "4,0 0,0 0,3 4,3 4,6 0,6"),
    ('6', "4,0 0,0 0,6 4,6 4,3 0,3"),
    ('7', "0,0 4,0 4,1 2,6"),
    ('8', "0,0 4,0 4,6 0,6 0,0|0,3 4,3"),
    ('9', "4,3 0,3 0,0 4,0 4,6 0,6"),
    ('A', "0,6 0,2 2,0 4,2 4,6|0,3.5 4,3.5"),
    ('B', "0,0 3,0 4,1 4,2 3,3 0,3|3,3 4,4 4,5 3,6 0,6 0,0"),
    ('C', "4,0 0,0 0,6 4,6"),
    ('D', "0,0 3,0 4,1 4,5 3,6 0,6 0,0"),
    ('E', "4,0 0,0 0,6 4,6|0,3 3,3"),
    ('F', "4,0 0,0 0,6|0,3 3,3"),
    ('G', "4,0 0,0 0,6 4,6 4,3 2,3"),
    ('H', "0,0 0,6|4,0 4,6|0,3 4,3"),
    ('I', "1,0 3,0|2,0 2,6|1,6 3,6"),
    ('J', "4,0 4,6 0,6 0,4"),
    ('K', "0,0 0,6|4,0 0,3 4,6"),
    ('L', "0,0 0,6 4,6"),
    ('M', "0,6 0,0 2,3 4,0 4,6"),
    ('N', "0,6 0,0 4,6 4,0"),
    ('O', "0,0 4,0 4,6 0,6 0,0"),
    ('P', "0,6 0,0 4,0 4,3 0,3"),
    ('Q', "0,0 4,0 4,6 0,6 0,0|2.5,4.5 4,6"),
    ('R', "0,6 0,0 4,0 4,3 0,3 4,6"),
    ('S', "4,0 0,0 0,3 4,3 4,6 0,6"),
    ('T', "0,0 4,0|2,0 2,6"),
    ('U', "0,0 0,6 4,6 4,0"),
    ('V', "0,0 2,6 4,0"),
    ('W', "0,0 1,6 2,3 3,6 4,0"),
    ('X', "0,0 4,6|4,0 0,6"),
    ('Y', "0,0 2,3 4,0|2,3 2,6"),
    ('Z', "0,0 4,0 0,6 4,6"),
    ('-', "1,3 3,3"),
    ('+', "1,3 3,3|2,2 2,4"),
    ('.', "2,5.7 2,6"),
    (',', "2,5 1.5,7"),
    (':', "2,1.7 2,2|2,4.3 2,4.6"),
    ('/', "0,6 4,0"),
    ('>', "0,0 4,3 0,6"),
    ('<', "4,0 0,3 4,6"),
    ('(', "3,0 1,2 1,4 3,6"),
    (')', "1,0 3,2 3,4 1,6"),
    ('=', "1,2 3,2|1,4 3,4"),
    ('\'', "2,0 2,1.5"),
    ('!', "2,0 2,4|2,5.7 2,6"),
    ('?', "0,1 1,0 3,0 4,1 4,2 2,3 2,4|2,5.7 2,6"),
    ('%', "0,6 4,0|0,0 1,0 1,1 0,1 0,0|3,5 4,5 4,6 3,6 3,5"),
    ('#', "1,0 1,6|3,0 3,6|0,2 4,2|0,4 4,4"),
    ('*', "2,1 2,5|0,2 4,4|0,4 4,2"),
    ('[', "3,0 1,0 1,6 3,6"),
    (']', "1,0 3,0 3,6 1,6"),
    (';', "2,1.7 2,2|2,5 1.5,7"),
    ('\\', "0,0 4,6"),
    ('`', "1.5,0 2.5,1.5"),
    ('_', "0,6 4,6"),
    ('"', "1,0 1,1.5|3,0 3,1.5"),
    ('^', "1,1.5 2,0 3,1.5"),
    ('&', "4,6 1,2.5 1,0.5 1.5,0 2.5,0 3,0.5 3,1.5 0,4 0,6 2,6 4,3.5"),
    ('$', "4,1 0,1 0,3 4,3 4,5 0,5|2,0 2,6"),
    ('§', "3,0 1,0 1,2 3,3 3,4|1,2 1,3 3,4 3,6 1,6"),
    ('²', "1,0 3,0 3,1.5 1,1.5 1,3 3,3"),
    ('µ', "0,2 0,7|0,6 4,6 4,2"),
    ('£', "4,1 3,0 2,0 1,1 1,5 0,6 4,6|0,3 3,3"),
    ('¤', "1,2 3,2 3,4 1,4 1,2|0,1 1,2|4,1 3,2|0,5 1,4|4,5 3,4"),
    ('¨', "1,0.5 1,0.8|3,0.5 3,0.8"),
    ('°', "1.5,0 2.5,0 2.5,1 1.5,1 1.5,0"),
    ('´', "1.5,1.5 2.5,0"),
    ('~', "0.5,3.5 1.5,2.5 2.5,3.5 3.5,2.5"),
    ('|', "2,-0.5 2,6.5"),
    ('½', "0,1 1,0 1,3|0,6 4,0|2.5,3.5 4,3.5 4,4.8 2.5,4.8 2.5,6 4,6"),
    ('º', "1,0 3,0 3,2 1,2 1,0|1,3 3,3"),
    ('¡', "2,0 2,0.3|2,2 2,6"),
    ('«', "2,1.5 0,3 2,4.5|4,1.5 2,3 4,4.5"),
    ('»', "0,1.5 2,3 0,4.5|2,1.5 4,3 2,4.5"),
    ('ß', "0,6 0,1 1,0 3,0 4,1 4,2 2,3 4,4 4,5 3,6 1,6"),
    ('Æ', "0,6 0,2 2,0 4,0|2,0 2,6 4,6|0,3.5 2,3.5|2,3 3.5,3"),
    ('Ø', "0,0 4,0 4,6 0,6 0,0|0,6.5 4,-0.5"),
];

/// Small letters (x-height 2 to 6, l and t rise to 0), the bases of the accented letters that
/// the French, Italian, Hungarian, Czech and Slovak keycaps print in lower case (and of the
/// translations' Polish letters).
const SMALL: &[(char, &str)] = &[
    ('a', "0,2 4,2 4,6 0,6 0,4 4,4"),
    ('c', "4,2 0,2 0,6 4,6"),
    ('e', "4,6 0,6 0,2 4,2 4,4 0,4"),
    ('i', "2,2 2,6"),
    ('l', "2,0 2,6"),
    ('n', "0,6 0,2 4,2 4,6"),
    ('o', "0,2 4,2 4,6 0,6 0,2"),
    ('r', "0,6 0,2 4,2"),
    ('s', "4,2 0,2 0,4 4,4 4,6 0,6"),
    ('t', "2,0 2,6 4,6|0,2 4,2"),
    ('u', "0,2 0,6 4,6 4,2"),
    ('y', "0,2 0,4 4,4|4,2 4,7.5 1,7.5"),
    ('z', "0,2 4,2 0,6 4,6"),
];

/// The diacritics: strokes over the x-height (y 0 to 1.3) or under the baseline, whether they
/// are raised over a capital, and the base letters with their accented letters, char by char.
const MARKS: &[(&str, bool, &str, &str)] = &[
    ("1.5,1.3 2.5,0", true, "aeiouycnszAEIOUYCNSZ", "áéíóúýćńśźÁÉÍÓÚÝĆŃŚŹ"),
    ("1.5,0 2.5,1.3", true, "aeiouAEIOU", "àèìòùÀÈÌÒÙ"),
    ("1,1.3 2,0 3,1.3", true, "aeiouAEIOU", "âêîôûÂÊÎÔÛ"),
    ("1,0.5 1,0.8|3,0.5 3,0.8", true, "aeiouyAEIOUY", "äëïöüÿÄËÏÖÜŸ"),
    ("1,0 2,1.3 3,0", true, "censrzCENSRZLT", "čěňšřžČĚŇŠŘŽĽŤ"),
    ("1,1.3 1.8,0|2.5,1.3 3.3,0", true, "ouOU", "őűŐŰ"),
    ("1.5,0 2.5,0 2.5,1 1.5,1 1.5,0", true, "auAU", "åůÅŮ"),
    ("0.5,1 1.5,0.3 2.5,1 3.5,0.3", true, "anoANO", "ãñõÃÑÕ"),
    ("2,0.4 2,0.9", true, "zZ", "żŻ"),
    ("3,6 2.5,7 3.5,7.5", false, "aeAE", "ąęĄĘ"),
    ("1,4.5 3,2.5", false, "lL", "łŁ"),
    // The caron of ľ and ť is an apostrophe beside the stem.
    ("3.5,0 3,1.3", false, "lt", "ľť"),
    ("2,6 2.5,7 1.5,7.5", false, "cC", "çÇ"),
];

/// Raise of a diacritic over a capital (grid units).
const CAPITAL_MARK: f32 = -1.7;

/// The strokes of `ch` (ASCII upper-cased): its glyph, or an accented letter's base and its
/// diacritic with the diacritic's vertical offset.
fn glyph(ch: char) -> Option<(&'static str, Option<(&'static str, f32)>)> {
    let ch = ch.to_ascii_uppercase();
    if let Some((_, g)) = STROKES.iter().find(|(k, _)| *k == ch) {
        return Some((g, None));
    }
    MARKS.iter().find_map(|&(mark, raise, bases, accented)| {
        let (base, _) = bases.chars().zip(accented.chars()).find(|&(_, a)| a == ch)?;
        let table = if base.is_ascii_uppercase() { STROKES } else { SMALL };
        let (_, g) = table.iter().find(|(k, _)| *k == base)?;
        let dy = if raise && base.is_ascii_uppercase() { CAPITAL_MARK } else { 0.0 };
        Some((*g, Some((mark, dy))))
    })
}

/// Whether the stroke font draws `ch` (ASCII upper-cased; a space is blank).
pub fn stroke_has(ch: char) -> bool {
    ch == ' ' || glyph(ch).is_some()
}

/// Advance of the stroke font in grid units (4 wide + 2 apart).
const STROKE_ADVANCE: f32 = 6.0;

/// `s` with the characters neither font draws replaced by their letters without accents
/// ("œ" -> "oe"); the others are left out when drawn.
fn stroke_safe(s: &str) -> std::borrow::Cow<'_, str> {
    let drawn = |c: char| c.is_ascii() || stroke_has(c) || stroke_has(capital(c)) || outlined(c);
    if s.chars().all(drawn) {
        return s.into();
    }
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if drawn(c) {
            out.push(c);
        } else {
            out.push_str(&stealth_i18n::fold_accents(&c.to_string()));
        }
    }
    out.into()
}

/// Width in pixels of `s` drawn with [`stroke_text`] at glyph height `h`.
pub fn stroke_width(s: &str, h: f32) -> f32 {
    let safe = stroke_safe(s);
    if safe.is_empty() {
        return 0.0;
    }
    safe.chars().map(|c| advance(c, h)).sum::<f32>() - 2.0 * h / 6.0
}

/// `ch` in capitals if a font has its capital, else as it is ("ß", whose capital is "SS", and
/// the micro sign, whose capital is Greek, stay): as [`stroke_text`] draws it.
pub fn capital(ch: char) -> char {
    let mut upper = ch.to_uppercase();
    match (upper.next(), upper.next()) {
        (Some(u), None) if glyph(u).is_some() || (glyph(ch).is_none() && outlined(u)) => u,
        _ => ch,
    }
}

/// Draws `s` (upper-cased) with the stroke font, top-left at (x, y), glyph height `h`, pen
/// width `w`. Returns the x after the text.
pub fn stroke_text(c: &mut Canvas, x: f32, y: f32, s: &str, h: f32, w: f32, col: u8) -> f32 {
    stroke_chars(c, x, y, stroke_safe(s).chars().map(capital), h, w, col)
}

/// [`stroke_text`] keeping the accented small letters small: the key labels of the layouts
/// that print them so ("é" on AZERTY), as the CONTROLS dialog shows them.
pub fn stroke_text_keep_case(
    c: &mut Canvas,
    x: f32,
    y: f32,
    s: &str,
    h: f32,
    w: f32,
    col: u8,
) -> f32 {
    stroke_chars(c, x, y, stroke_safe(s).chars(), h, w, col)
}

fn stroke_chars(
    c: &mut Canvas,
    x: f32,
    y: f32,
    s: impl Iterator<Item = char>,
    h: f32,
    w: f32,
    col: u8,
) -> f32 {
    let u = h / 6.0;
    let mut x = x;
    for ch in s {
        if let Some((g, mark)) = glyph(ch) {
            stroke_polylines(c, Vec2::new(x, y), g, u, w, col);
            if let Some((m, dy)) = mark {
                stroke_polylines(c, Vec2::new(x, y + dy * u), m, u, w, col);
            }
        } else if outlined(ch) {
            draw_outlined(c, Vec2::new(x, y + h), ch, h, w, col);
        }
        x += advance(ch, h);
    }
    x
}

/// The fonts that draw what the stroke font cannot (Greek, Cyrillic, Han characters, kana,
/// hangul) as outlines: Hack, then the CJK subsets.
fn outline_faces() -> &'static [(OutlineFont, ttf_parser::Face<'static>)] {
    static FACES: std::sync::LazyLock<Vec<(OutlineFont, ttf_parser::Face<'static>)>> =
        std::sync::LazyLock::new(|| {
            let hack = ttf_parser::Face::parse(epaint_default_fonts::HACK_REGULAR, 0).ok();
            let cjk = crate::i18n::CJK.into_iter().filter_map(|l| {
                let face = ttf_parser::Face::parse(crate::i18n::cjk_font(l)?, 0).ok()?;
                Some((OutlineFont::Cjk(l), face))
            });
            hack.map(|f| (OutlineFont::Hack, f)).into_iter().chain(cjk).collect()
        });
    &FACES
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum OutlineFont {
    Hack,
    Cjk(stealth_i18n::Language),
}

impl OutlineFont {
    /// The font's em in pixels for capitals `h` high: Hack's capitals are 0.73 em high, the CJK
    /// characters (about 0.9 em) are drawn as big as the capitals.
    fn em(self, h: f32) -> f32 {
        match self {
            OutlineFont::Hack => h / 0.73,
            OutlineFont::Cjk(_) => h * 1.1,
        }
    }
}

/// Whether `c` is drawn from a font's outline: a letter or sign of the scripts after the Latin
/// ones (the accented Latin letters have their strokes, or lose their accents).
fn outlined(c: char) -> bool {
    face_of(c).is_some()
}

/// The font that draws `c` and its glyph: Hack, then the player's language's CJK font (the
/// same Han characters are drawn differently in Chinese, Japanese and Korean), then the others.
fn face_of(
    c: char,
) -> Option<(OutlineFont, &'static ttf_parser::Face<'static>, ttf_parser::GlyphId)> {
    let code = c as u32;
    // Not the Latin letters, nor the general punctuation and symbols (they are folded).
    if code < 0x370 || (0x1E00..0x2C00).contains(&code) {
        return None;
    }
    let lang = stealth_i18n::language();
    let faces = outline_faces();
    let hack = faces.iter().filter(|(f, _)| *f == OutlineFont::Hack);
    let own = faces.iter().filter(|(f, _)| *f == OutlineFont::Cjk(lang));
    let rest = faces.iter().filter(|(f, _)| matches!(f, OutlineFont::Cjk(l) if *l != lang));
    hack.chain(own)
        .chain(rest)
        .find_map(|(font, face)| Some((*font, face, face.glyph_index(c).filter(|g| g.0 != 0)?)))
}

/// The pen's advance after `c` in pixels, for capitals `h` high.
fn advance(c: char, h: f32) -> f32 {
    if glyph(c).is_none()
        && let Some((font, face, id)) = face_of(c)
    {
        let k = font.em(h) / face.units_per_em() as f32;
        // A little room between the characters, as the stroke font has.
        return face.glyph_hor_advance(id).map_or(h, |a| a as f32 * k) + h / 12.0;
    }
    STROKE_ADVANCE * h / 6.0
}

/// Fills `ch`'s outline with the baseline at `origin`. The outline is also stroked (by a quarter
/// of the pen's width): the fonts' Regular weight is thinner than the strokes.
fn draw_outlined(c: &mut Canvas, origin: Vec2, ch: char, h: f32, w: f32, col: u8) {
    let Some((font, face, id)) = face_of(ch) else { return };
    let mut outline = Outline::default();
    if face.outline_glyph(id, &mut outline).is_none() {
        return;
    }
    let k = font.em(h) / face.units_per_em() as f32;
    // Font units point up.
    let contours: Vec<Vec<Vec2>> = outline
        .0
        .into_iter()
        .map(|c| c.into_iter().map(|p| origin + p * Vec2::new(k, -k)).collect())
        .collect();
    c.fill_contours(&contours, |under, share| if share >= 0.4 { col } else { under });
    for mut contour in contours {
        if let Some(&first) = contour.first() {
            contour.push(first);
        }
        c.polyline(&contour, w / 4.0, col);
    }
}

/// A glyph's outline as closed polylines (its curves in short segments), in font units.
#[derive(Default)]
pub(crate) struct Outline(pub(crate) Vec<Vec<Vec2>>);

impl Outline {
    /// Segments per curve.
    const STEPS: usize = 8;

    fn last(&self) -> Vec2 {
        self.0.last().and_then(|c| c.last()).copied().unwrap_or_default()
    }

    fn push(&mut self, p: Vec2) {
        if let Some(c) = self.0.last_mut() {
            c.push(p);
        }
    }
}

impl ttf_parser::OutlineBuilder for Outline {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.push(vec![Vec2::new(x, y)]);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.push(Vec2::new(x, y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (p0, p1, p2) = (self.last(), Vec2::new(x1, y1), Vec2::new(x, y));
        for i in 1..=Self::STEPS {
            let t = i as f32 / Self::STEPS as f32;
            self.push(p0.lerp(p1, t).lerp(p1.lerp(p2, t), t));
        }
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (p0, p1, p2, p3) = (self.last(), Vec2::new(x1, y1), Vec2::new(x2, y2), Vec2::new(x, y));
        for i in 1..=Self::STEPS {
            let t = i as f32 / Self::STEPS as f32;
            let (a, b, c) = (p0.lerp(p1, t), p1.lerp(p2, t), p2.lerp(p3, t));
            self.push(a.lerp(b, t).lerp(b.lerp(c, t), t));
        }
    }

    fn close(&mut self) {}
}

/// Draws a glyph's polylines (grid units of `u` pixels) from `origin`.
fn stroke_polylines(c: &mut Canvas, origin: Vec2, g: &str, u: f32, w: f32, col: u8) {
    for poly in g.split('|') {
        let pts: Vec<Vec2> = poly
            .split(' ')
            .filter_map(|p| p.split_once(','))
            .map(|(a, b)| {
                origin
                    + Vec2::new(a.parse::<f32>().unwrap_or(0.0), b.parse::<f32>().unwrap_or(0.0))
                        * u
            })
            .collect();
        if pts.len() == 1 {
            c.stroke(pts[0], pts[0], w, col);
        }
        c.polyline(&pts, w, col);
    }
}

/// Horizontal anchor of [`stroke_text_at`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Centre,
    Right,
}

/// [`stroke_text`] anchored at (x, y): x by `align`, y at the text's vertical middle.
#[allow(clippy::too_many_arguments)]
pub fn stroke_text_at(
    c: &mut Canvas,
    x: f32,
    y: f32,
    s: &str,
    h: f32,
    w: f32,
    col: u8,
    align: Align,
) -> f32 {
    let width = stroke_width(s, h);
    let x0 = match align {
        Align::Left => x,
        Align::Centre => x - width / 2.0,
        Align::Right => x - width,
    };
    stroke_text(c, x0, y - h / 2.0, s, h, w, col)
}

#[cfg(test)]
mod tests {
    use super::*;
    use stealth_i18n::Language;

    #[test]
    fn flight_font_covers_ascii() {
        let fonts = Fonts::builtin();
        let f = fonts.font(HUD_FONT);
        for code in b' '..=b'~' {
            let g = f.glyph(code).unwrap_or_else(|| panic!("no glyph for {:?}", code as char));
            assert_eq!((g.width, g.advance, g.rows.len()), (3, 4, 6));
            assert!(code == b' ' || g.rows.iter().any(|&r| r != 0), "{:?} is blank", code as char);
        }
        assert_ne!(f.glyph(b'+').unwrap().rows, f.glyph(b'-').unwrap().rows);
        assert_eq!(f.glyph(b'a').unwrap().rows, f.glyph(b'A').unwrap().rows);
        assert_eq!(fonts.width(b"ZM+"), 12);
    }

    #[test]
    fn stroke_font_covers_hud_text() {
        for ch in "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ-+.,:/><()=".chars() {
            assert!(STROKES.iter().any(|(k, _)| *k == ch), "no stroke glyph for {ch:?}");
        }
    }

    /// The keycaps of the European layouts, as the CONTROLS dialog names them (the kneeboard's
    /// key guide falls back to the US names for the others).
    #[test]
    fn stroke_font_covers_european_keys() {
        for ch in crate::keyboard_layout::EUROPEAN_KEYCAPS.chars() {
            assert!(stroke_has(ch), "no stroke glyph for {ch:?}");
        }
        for (_, _, bases, accented) in MARKS {
            assert_eq!(bases.chars().count(), accented.chars().count(), "{accented}");
        }
        assert!(!stroke_has('ф'));
    }

    /// The translations' letters (French, German, Spanish, Italian, Polish, Portuguese) are
    /// drawn, the others without their accents.
    #[test]
    fn stroke_font_covers_translations() {
        for ch in "àâäçéèêëîïôöùûüÿñãõáíóúßąćęłńśźżÀÂÄÇÉÈÊËÎÏÔÖÙÛÜŸÑÃÕÁÍÓÚĄĆĘŁŃŚŹŻ".chars()
        {
            assert!(stroke_has(capital(ch)), "no stroke glyph for {ch:?}");
        }
        assert_eq!(stroke_safe("ŒUVRE"), "OEUVRE");
        assert_eq!(stroke_width("Œ", 6.0), stroke_width("OE", 6.0));
        assert_eq!(upper("Siège"), b"SIEGE");
    }

    /// Text is drawn in capitals where the font has them.
    #[test]
    fn capitals() {
        assert_eq!(capital('a'), 'A');
        assert_eq!(capital('ç'), 'Ç');
        assert_eq!(capital('ő'), 'Ő');
        assert_eq!(capital('ß'), 'ß');
        assert_eq!(capital('µ'), 'µ');
        assert_eq!(capital('_'), '_');
    }

    /// The scripts the stroke font has no strokes for are drawn from the menus' fonts: wide as
    /// their fonts say, and dark pixels on the canvas.
    #[test]
    fn outlines_draw_the_other_scripts() {
        for (ch, l) in [('Ж', "ru"), ('Ω', "el"), ('日', "ja"), ('語', "zh-Hans"), ('한', "ko")]
        {
            assert!(outlined(ch), "{ch:?} ({l}) is not drawn");
            assert!(stroke_width(&ch.to_string(), 20.0) > 10.0, "{ch:?} has no width");
            let mut c = Canvas::new(64, 64);
            c.clear(0);
            stroke_text(&mut c, 8.0, 8.0, &ch.to_string(), 24.0, 3.0, 1);
            assert!((0..64).any(|y| (0..64).any(|x| c.get(x, y) == 1)), "{ch:?} is blank");
        }
        // The Latin letters with accents keep their strokes or lose their accents.
        assert!(!outlined('é') && !outlined('œ'));
        assert_eq!(stroke_width("ab", 12.0), 20.0);
    }

    /// Everything the translations say can be drawn, the in-flight menu's too (the key guide,
    /// the kneeboard): no character is left out of what a language's texts use.
    #[test]
    fn stroke_text_draws_every_translation() {
        let mut missing = std::collections::BTreeSet::new();
        for l in [
            Language::Ru,
            Language::Uk,
            Language::Bg,
            Language::El,
            Language::Zh,
            Language::Ja,
            Language::Ko,
        ] {
            let dir = format!("assets/i18n/{}", l.code());
            for file in std::fs::read_dir(&dir).unwrap() {
                let path = file.unwrap().path();
                if path.extension().is_none_or(|e| e != "ftl") {
                    continue;
                }
                let text = std::fs::read_to_string(path).unwrap();
                missing.extend(text.chars().filter(|&c| {
                    !c.is_ascii()
                        && !c.is_whitespace()
                        && !c.is_control()
                        && !(stroke_has(c) || stroke_has(capital(c)) || outlined(c))
                        && stealth_i18n::fold_accents(&c.to_string()) == c.to_string()
                }));
            }
        }
        // Whatever is left has no script of its own (arrows, quotes: folded or left out).
        assert!(missing.iter().all(|c| !c.is_alphabetic()), "{missing:?}");
    }
}
