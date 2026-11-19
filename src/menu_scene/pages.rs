//! The MFD pages the menu's cockpits show (the flight's need the flight's aircraft): the engine
//! page, the signature page, the threats and warnings, and pages in the MFDs' style drawn from
//! the scene's own flight: the radar's B-scope, the HSI, the stores.

use bevy::prelude::*;

use crate::aircraft::AircraftKind;
use crate::canvas::Canvas;
use crate::font::{Align, stroke_text_at};

const BLACK: u8 = 5;
const DARK: u8 = 8;
const GREY: u8 = 7;
const GREEN: u8 = 10;
const CYAN: u8 = 11;
const RED: u8 = 13;
const WHITE: u8 = 15;
const AMBER: u8 = 18;
/// The bezel band the OSB labels sit in (drawing units).
const BAND: f32 = 16.0;

/// A page.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Engines,
    Signature,
    Threats,
    Radar,
    Hsi,
    Stores,
}

/// The pages a type's MFDs show, a different one on each (as many as `n`), the scene's pick
/// (`seed`).
pub fn pick(kind: AircraftKind, n: usize, seed: u32) -> Vec<Page> {
    let mut pool = vec![Page::Engines, Page::Hsi, Page::Stores, Page::Threats];
    if kind.rcs_page() {
        pool.push(Page::Signature);
    } else {
        pool.push(Page::Radar);
    }
    let mut h = seed as u64 | 1;
    for i in (1..pool.len()).rev() {
        h = h.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        pool.swap(i, (h >> 33) as usize % (i + 1));
    }
    pool.into_iter().take(n).collect()
}

/// What the pages show of the flight: heading (rad), speed (m/s), the aircraft, the time.
pub struct Flight {
    pub heading: f32,
    pub speed: f32,
    pub kind: AircraftKind,
    pub t: f32,
}

/// Draws a page.
pub fn draw(c: &mut Canvas, fonts: &crate::font::Fonts, page: Page, f: &Flight) {
    c.reset_clip();
    c.clear(BLACK);
    match page {
        Page::Engines => {
            crate::eng_page::draw(c, f.kind, &super::cockpits::cruise_engines(f.kind, f.t));
        }
        Page::Signature => super::cockpits::draw_rcs(c, f.t),
        Page::Threats => {
            let (emitters, emv, history) = crate::mpd::threat_fixture();
            let th = crate::mpd::Threats {
                emitters: &emitters,
                emv: &emv,
                history: &history,
                bay_open: false,
                gear_down: false,
                pitch: 0.05,
                roll: 0.0,
            };
            let blink = (f.t * 2.0) as i32 % 2 == 0;
            let cm = crate::panel::Countermeasures::default();
            crate::mpd::draw_left(c, fonts, &th, &[], &[], blink, &cm);
        }
        Page::Radar => radar(c, f),
        Page::Hsi => hsi(c, f),
        Page::Stores => stores(c, f),
    }
}

/// Text in the page's ink.
fn text(c: &mut Canvas, x: f32, y: f32, s: &str, size: f32, col: u8, align: Align) {
    stroke_text_at(c, x, y, s, size, 1.0, col, align);
}

/// The OSB labels: top row and bottom row, the selected one boxed.
fn labels(c: &mut Canvas, top: &[&str], bottom: &[&str], selected: &str) {
    let (w, h) = (c.w as f32, c.h as f32);
    for (row, y) in [(top, BAND / 2.0 + 2.0), (bottom, h - BAND / 2.0 - 1.0)] {
        for (i, l) in row.iter().enumerate() {
            let x = w * (2 * i + 1) as f32 / 10.0;
            text(c, x, y, l, 5.5, GREEN, Align::Centre);
            if *l == selected {
                let half = crate::font::stroke_width(l, 5.5) / 2.0 + 2.0;
                c.frame(Vec2::new(x - half, y - 5.0), Vec2::new(x + half, y + 5.0), 0.8, GREEN);
            }
        }
    }
}

/// The fire control radar's B-scope: the azimuth and range grid, the sweeping antenna, three
/// contacts with their heights, the range and mode.
fn radar(c: &mut Canvas, f: &Flight) {
    let (w, h) = (c.w as f32, c.h as f32);
    let (l, r, t, b) = (w * 0.12, w * 0.88, BAND + 12.0, h - BAND - 12.0);
    labels(c, &["CRM", "RWS", "", "", "SWAP"], &["SMS", "FCR", "HSD", "TGP", "DCLT"], "FCR");
    for k in 1..4 {
        let y = t + (b - t) * k as f32 / 4.0;
        c.dashed(Vec2::new(l, y), Vec2::new(r, y), 2.0, 4.0, 0.6, DARK);
        let x = l + (r - l) * k as f32 / 4.0;
        c.dashed(Vec2::new(x, t), Vec2::new(x, b), 2.0, 4.0, 0.6, DARK);
    }
    c.frame(Vec2::new(l, t), Vec2::new(r, b), 0.8, GREEN);
    text(c, l - 3.0, t + 4.0, "40", 5.0, GREEN, Align::Right);
    text(c, l + 4.0, t + 8.0, "A4", 5.0, GREEN, Align::Left);
    // The antenna sweeping ±60 degrees.
    let sweep = (f.t * 1.4).sin();
    let x = (l + r) / 2.0 + sweep * (r - l) / 2.0;
    c.stroke(Vec2::new(x, b), Vec2::new(x, b - 6.0), 1.4, GREEN);
    c.stroke(Vec2::new(l - 4.0, (t + b) / 2.0), Vec2::new(l, (t + b) / 2.0), 1.2, GREEN);
    // Contacts closing slowly; the nearest one locked.
    for (i, (az, rng, alt)) in
        [(-0.3f32, 0.55f32, "22"), (0.18, 0.35, "18"), (0.45, 0.72, "31")].into_iter().enumerate()
    {
        let p = Vec2::new((l + r) / 2.0 + az * (r - l) / 2.0, b - (rng - f.t * 0.01) * (b - t));
        if i == 1 {
            c.ring(p, 4.5, 1.2, AMBER);
            c.stroke(p, p + Vec2::new(-6.0, -6.0), 1.0, AMBER);
        } else {
            c.fill_rect(p.x as i32 - 2, p.y as i32 - 2, 4, 4, GREEN);
        }
        text(c, p.x, p.y + 9.0, alt, 4.5, GREEN, Align::Centre);
    }
    text(c, r, b + 7.0, &format!("{:.0}", f.speed * 1.944), 5.0, GREEN, Align::Right);
}

/// The HSI: the compass rose turning with the heading, the course arrow, the steerpoint's
/// distance and time.
fn hsi(c: &mut Canvas, f: &Flight) {
    let (w, h) = (c.w as f32, c.h as f32);
    labels(c, &["HSI", "", "", "", "SWAP"], &["SMS", "FCR", "HSD", "TGP", "DCLT"], "HSI");
    let o = Vec2::new(w / 2.0, h / 2.0 + 4.0);
    let r = (h / 2.0 - BAND - 10.0).min(w / 2.0 - 30.0);
    c.ring(o, r, 1.0, GREEN);
    for k in 0..36 {
        let a = k as f32 * 10f32.to_radians() - f.heading;
        let d = Vec2::new(a.sin(), -a.cos());
        let long = k % 3 == 0;
        c.stroke(o + d * r, o + d * (r - if long { 7.0 } else { 4.0 }), 0.9, GREEN);
        if long {
            let label = match k {
                0 => "N".to_string(),
                9 => "E".to_string(),
                18 => "S".to_string(),
                27 => "W".to_string(),
                _ => format!("{}", k),
            };
            text(
                c,
                o.x + d.x * (r - 14.0),
                o.y + d.y * (r - 14.0),
                &label,
                5.5,
                WHITE,
                Align::Centre,
            );
        }
    }
    // The course to the steerpoint, the aircraft in the middle.
    let course = 0.6f32 - f.heading;
    let d = Vec2::new(course.sin(), -course.cos());
    c.stroke(o - d * (r - 18.0), o + d * (r - 18.0), 1.4, CYAN);
    c.triangle(
        [
            o + d * (r - 14.0),
            o + d * (r - 22.0) + d.perp() * 3.0,
            o + d * (r - 22.0) - d.perp() * 3.0,
        ],
        CYAN,
    );
    c.stroke(o + Vec2::new(-7.0, 0.0), o + Vec2::new(7.0, 0.0), 1.2, WHITE);
    c.stroke(o + Vec2::new(0.0, -5.0), o + Vec2::new(0.0, 6.0), 1.2, WHITE);
    c.stroke(Vec2::new(o.x, o.y - r - 2.0), Vec2::new(o.x, o.y - r + 6.0), 1.4, WHITE);
    let hdg = (f.heading.to_degrees().rem_euclid(360.0)).round();
    text(c, o.x, BAND + 10.0, &format!("{hdg:03.0}"), 6.5, WHITE, Align::Centre);
    let dist = 24.3 - f.t * f.speed / 1852.0;
    text(c, 8.0, BAND + 12.0, &format!("{dist:.1} NM"), 5.0, GREEN, Align::Left);
    text(c, 8.0, BAND + 22.0, "STPT 3", 5.0, GREEN, Align::Left);
    text(c, w - 8.0, BAND + 12.0, "ETA 19:42", 5.0, GREEN, Align::Right);
}

/// The stores: the aircraft's stations with what hangs there, the gun's rounds, the master
/// arm.
fn stores(c: &mut Canvas, f: &Flight) {
    let (w, h) = (c.w as f32, c.h as f32);
    labels(c, &["AAM", "AG", "", "INV", "SWAP"], &["SMS", "FCR", "HSD", "TGP", "DCLT"], "SMS");
    let rows: &[(&str, &str)] = match f.kind {
        AircraftKind::F117 => &[("BAY 1", "GBU-27"), ("BAY 2", "GBU-27")],
        AircraftKind::A10 => {
            &[("1", "AIM-9"), ("3", "AGM-65D x3"), ("5", "MK-82 x2"), ("GUN", "1150")]
        }
        AircraftKind::Tornado => &[("WING", "AIM-9 x2"), ("BELLY", "JP233 x2"), ("GUN", "180")],
        AircraftKind::AH64 | AircraftKind::RAH66 => {
            &[("L", "AGM-114 x4"), ("R", "HYDRA x19"), ("GUN", "1200")]
        }
        _ => &[
            ("1", "AIM-9"),
            ("2", "AIM-120"),
            ("4", "GBU-12 x2"),
            ("6", "AIM-120"),
            ("GUN", "510"),
        ],
    };
    text(c, w / 2.0, BAND + 12.0, "STBY", 6.0, AMBER, Align::Centre);
    text(c, 10.0, BAND + 12.0, "MASTER ARM", 5.0, GREEN, Align::Left);
    let top = BAND + 30.0;
    let step = ((h - BAND * 2.0 - 40.0) / rows.len().max(1) as f32).min(22.0);
    for (i, (st, what)) in rows.iter().enumerate() {
        let y = top + i as f32 * step;
        text(c, w * 0.2, y, st, 5.5, GREY, Align::Left);
        let col = if i == 1 { WHITE } else { GREEN };
        text(c, w * 0.42, y, what, 6.0, col, Align::Left);
        if i == 1 {
            c.frame(Vec2::new(w * 0.4, y - 6.0), Vec2::new(w * 0.85, y + 6.0), 0.8, WHITE);
        }
    }
    let _ = RED;
}
