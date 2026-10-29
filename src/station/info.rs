//! A Gepard's target data screen (a Patriot's shows the radar signal, `signal`), laid out as
//! its fire control page: the locked track as the
//! radar's picture has it (twice a second). Labels yellow on black, values big and black on a
//! yellow panel.
//!
//! * A title row (the page, the lock's state) and under it the type in a yellow bar.
//! * Down the left the state boxes, lit when on: tracking, slewed by hand, firing, rounds
//!   ready, refilling.
//! * The panel: the identity and, in its four squares, bearing and time of flight; then the
//!   track number, range, altitude and speed with their units.
//! * At the bottom the gun's reach (a Patriot's: the PAC-3's) and whether the track is in it.

use bevy::prelude::*;
use stealth_sim::station::{GEPARD_REACH, GEPARD_V0, PATRIOT_MAX_RANGE};

use super::scope::{BLACK, YELLOW};
use super::{ConsoleUi, Manning};
use crate::canvas::{Canvas, CanvasTexture};
use crate::cockpit3d::{ConsoleCanvas, ConsoleScreen, INFO_CANVAS};
use crate::font::{Align, stroke_text_at};

/// Rows: the title's, the type bar's, the state boxes' and the panel's (top, bottom), the
/// bottom row's.
const TITLE: (i32, i32) = (4, 22);
const TYPE: (i32, i32) = (26, 44);
const PANEL: (i32, i32) = (48, 208);
const BOTTOM: (i32, i32) = (213, 235);
/// Columns: the state boxes' right edge, the panel's left edge, the title's small box's left
/// edge, the right edge.
const LEFT: i32 = 52;
const PANEL_X: i32 = 56;
const TITLE_BOX: i32 = 266;
const RIGHT: i32 = 316;
/// The panel's squares: first left edge, width, step; the value rows' labels (left, right),
/// the values' right edge.
const SQUARES: (i32, i32, i32) = (156, 36, 40);
const LABELS: (i32, i32) = (62, 144);
const VALUES_RIGHT: f32 = 268.0;
/// Text heights and pens: labels, values.
const TEXT: f32 = 6.5;
const PEN: f32 = 1.1;
const VALUE: f32 = 19.0;
const VALUE_PEN: f32 = 2.2;

pub(super) fn draw(
    manning: Option<Res<Manning>>,
    combat: Option<Res<crate::combat::Combat>>,
    c: Option<Res<crate::campaign::ActiveCampaign>>,
    ui: Res<ConsoleUi>,
    mut screens: Query<(&ConsoleCanvas, &mut CanvasTexture)>,
) {
    let (Some(m), Some(combat), Some(c)) = (manning, combat, c) else { return };
    if !m.gepard() {
        return;
    }
    let Some((_, mut tex)) = screens.iter_mut().find(|(s, _)| s.0 == ConsoleScreen::Info) else {
        return;
    };
    tex.dirty = true;
    let cv = &mut tex.canvas;
    cv.reset_clip();
    cv.clear(BLACK);
    let sim = &combat.sim;
    let lock = sim.station_of(c.me.0).and_then(|s| s.lock).map(|l| l as usize);
    let track = lock.and_then(|l| ui.snapshots.front()?.track(l));
    let (w, h) = (INFO_CANVAS.0 as i32, INFO_CANVAS.1 as i32);
    cv.rect(1, 1, w - 2, h - 2, YELLOW);

    // The title row and the type bar.
    framed(cv, (4, TITLE.0, TITLE_BOX - 4, TITLE.1), "TARGET DATA", false);
    framed(
        cv,
        (TITLE_BOX, TITLE.0, RIGHT, TITLE.1),
        if lock.is_some() { "TRK" } else { "---" },
        false,
    );
    framed(cv, (4, TYPE.0, LEFT, TYPE.1), "AIR", false);
    cv.fill_rect(PANEL_X, TYPE.0, RIGHT - PANEL_X, TYPE.1 - TYPE.0, YELLOW);
    let name = track.map_or(String::new(), |t| t.name.to_ascii_uppercase());
    framed(cv, (PANEL_X + 36, TYPE.0 + 3, RIGHT - 60, TYPE.1 - 3), &name, false);

    // The state boxes.
    let u = sim.aaa_units().get(m.unit).copied();
    let units: Vec<usize> = if m.gepard() { vec![m.unit] } else { sim.battery_launchers(m.unit) };
    let states = [
        ("TRK", lock.is_some()),
        ("MAN", m.gepard() && lock.is_none()),
        ("FIRE", u.is_some_and(|u| u.burst > 0)),
        ("RDY", units.iter().any(|&k| sim.aaa_units()[k].rounds > 0)),
        ("RLD", units.iter().any(|&k| sim.refill_left(k) > 0.0)),
    ];
    let step = (PANEL.1 - PANEL.0) / states.len() as i32;
    for (i, (label, on)) in states.into_iter().enumerate() {
        let top = PANEL.0 + i as i32 * step;
        framed(cv, (4, top, LEFT, top + step - 4), label, on);
    }

    // The panel: identity and the squares, then the values.
    cv.fill_rect(PANEL_X, PANEL.0, RIGHT - PANEL_X, PANEL.1 - PANEL.0, YELLOW);
    let ident = track.map_or("", |t| t.side.label());
    let strip = (PANEL.0 + 4, PANEL.0 + 26);
    framed(cv, (LABELS.0, strip.0, LABELS.0 + 40, strip.1), ident, false);
    let slant = track.map(|t| t.east.hypot(t.north).hypot(t.alt));
    let squares = [
        "BRG".to_string(),
        track.map_or(String::new(), |t| format!("{:03.0}", t.bearing())),
        if m.gepard() { "TOF" } else { "HDG" }.to_string(),
        if m.gepard() {
            slant.map_or(String::new(), |r| format!("{:.1}", r / GEPARD_V0 as f32))
        } else {
            track.map_or(String::new(), |t| format!("{:03.0}", t.heading()))
        },
    ];
    let (x0, sw, sstep) = SQUARES;
    for (i, text) in squares.iter().enumerate() {
        let x = x0 + i as i32 * sstep;
        framed(cv, (x, strip.0, x + sw, strip.1), text, false);
    }
    let rows: [(&str, Option<String>, &str); 4] = [
        ("TRACK", lock.map(|l| format!("{:04}", 1000 + l)), "-"),
        ("RANGE", slant.map(|r| format!("{r:.0}")), "m"),
        ("ALTITUDE", track.map(|t| format!("{:.0}", t.alt)), "m"),
        ("SPEED", track.map(|t| format!("{:.0}", t.ve.hypot(t.vn))), "m/s"),
    ];
    let first = strip.1 + 4;
    let row_h = (PANEL.1 - 4 - first) / rows.len() as i32;
    for (i, (label, value, unit)) in rows.into_iter().enumerate() {
        let top = first + i as i32 * row_h;
        // A thin black line over each field (the first under the squares).
        let y = top as f32 - 0.5;
        cv.stroke(Vec2::new(PANEL_X as f32 + 2.0, y), Vec2::new(RIGHT as f32 - 2.0, y), 0.8, BLACK);
        framed(cv, (LABELS.0, top + 2, LABELS.1, top + row_h - 2), label, false);
        let mid = (top + row_h / 2) as f32;
        if let Some(v) = value {
            stroke_text_at(cv, VALUES_RIGHT, mid, &v, VALUE, VALUE_PEN, BLACK, Align::Right);
            stroke_text_at(cv, VALUES_RIGHT + 12.0, mid, unit, TEXT, PEN * 1.6, BLACK, Align::Left);
        }
    }

    // The bottom row: in reach or not, and the reach.
    let reach = if m.gepard() { GEPARD_REACH } else { PATRIOT_MAX_RANGE } as f32;
    let status = match (lock, slant) {
        (None, _) => "NO LOCK",
        (Some(_), None) => "NO TRACK DATA",
        (Some(_), Some(r)) if r <= reach => "IN RANGE",
        _ => "OUT OF RANGE",
    };
    let split = 156;
    framed(cv, (4, BOTTOM.0, split - 4, BOTTOM.1), status, status == "IN RANGE");
    cv.fill_rect(split, BOTTOM.0, RIGHT - split, BOTTOM.1 - BOTTOM.0, YELLOW);
    let mid = (BOTTOM.0 + BOTTOM.1) as f32 / 2.0;
    let text = format!("REACH {reach:.0} m");
    stroke_text_at(
        cv,
        (split + RIGHT) as f32 / 2.0,
        mid,
        &text,
        TEXT,
        PEN * 1.6,
        BLACK,
        Align::Centre,
    );
}

/// A box (left, top, right, bottom) with its text in the middle: yellow framed on black, or
/// lit (black on yellow).
fn framed(cv: &mut Canvas, (l, t, r, b): (i32, i32, i32, i32), text: &str, lit: bool) {
    if lit {
        cv.fill_rect(l, t, r - l, b - t, YELLOW);
    } else {
        cv.fill_rect(l, t, r - l, b - t, BLACK);
        cv.rect(l, t, r - l, b - t, YELLOW);
    }
    let col = if lit { BLACK } else { YELLOW };
    let (x, y) = ((l + r) as f32 / 2.0, (t + b) as f32 / 2.0);
    stroke_text_at(cv, x, y, text, TEXT, PEN, col, Align::Centre);
}
