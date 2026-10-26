//! The target data screen: the locked track as the radar's picture has it (twice a second),
//! yellow on black with the values in yellow fields: track number, type and identity, range,
//! altitude, speed, bearing, and the gun's time of flight and reach (a Patriot's: the PAC-3's).

use bevy::prelude::*;
use stealth_sim::station::{GEPARD_REACH, GEPARD_V0, PATRIOT_MAX_RANGE};

use super::scope::{BLACK, YELLOW};
use super::{ConsoleUi, Manning};
use crate::canvas::{Canvas, CanvasTexture};
use crate::cockpit3d::{ConsoleCanvas, ConsoleScreen, INFO_CANVAS};
use crate::font::{Align, stroke_text_at};

/// The title bar's height, the rows' first top and height, the label column's width, the
/// value fields' width, the bottom bar's top.
const TITLE_H: f32 = 26.0;
const ROWS_TOP: f32 = 34.0;
const ROW_H: f32 = 28.0;
const LABEL_W: f32 = 120.0;
const VALUE_W: f32 = 120.0;
const STATUS_TOP: f32 = 206.0;
/// Text heights and pen: labels, values.
const LABEL: f32 = 7.0;
const VALUE: f32 = 13.0;
const PEN: f32 = 1.1;

pub(super) fn draw(
    manning: Option<Res<Manning>>,
    combat: Option<Res<crate::combat::Combat>>,
    c: Option<Res<crate::campaign::ActiveCampaign>>,
    ui: Res<ConsoleUi>,
    mut screens: Query<(&ConsoleCanvas, &mut CanvasTexture)>,
) {
    let (Some(m), Some(combat), Some(c)) = (manning, combat, c) else { return };
    let Some((_, mut tex)) = screens.iter_mut().find(|(s, _)| s.0 == ConsoleScreen::Info) else {
        return;
    };
    tex.dirty = true;
    let cv = &mut tex.canvas;
    cv.reset_clip();
    cv.clear(BLACK);
    let (w, h) = (INFO_CANVAS.0 as f32, INFO_CANVAS.1 as f32);
    let lock = combat.sim.station_of(c.me.0).and_then(|s| s.lock).map(|l| l as usize);
    let track = lock.and_then(|l| ui.snapshots.front()?.track(l));

    // The title: the track's type and identity.
    cv.rect(2, 2, w as i32 - 4, TITLE_H as i32 - 4, YELLOW);
    let title = match track {
        Some(t) => format!("{}  {}", t.name.to_ascii_uppercase(), t.side.label()),
        None => "TARGET DATA".into(),
    };
    stroke_text_at(cv, w / 2.0, TITLE_H / 2.0, &title, 9.0, PEN, YELLOW, Align::Centre);

    let reach = if m.gepard { GEPARD_REACH } else { PATRIOT_MAX_RANGE } as f32;
    let slant = track.map(|t| t.east.hypot(t.north).hypot(t.alt));
    let rows: [(&str, Option<String>, &str); 6] = [
        ("TRACK", lock.map(|l| format!("{:04}", 1000 + l)), ""),
        ("RANGE", slant.map(|r| format!("{r:.0}")), "m"),
        ("ALTITUDE", track.map(|t| format!("{:.0}", t.alt)), "m"),
        ("SPEED", track.map(|t| format!("{:.0}", t.ve.hypot(t.vn))), "m/s"),
        ("BEARING", track.map(|t| format!("{:03.0}", t.bearing())), "DEG"),
        if m.gepard {
            ("FLIGHT TIME", slant.map(|r| format!("{:.1}", r / GEPARD_V0 as f32)), "s")
        } else {
            ("HEADING", track.map(|t| format!("{:03.0}", t.heading())), "DEG")
        },
    ];
    for (i, (label, value, unit)) in rows.into_iter().enumerate() {
        row(cv, ROWS_TOP + i as f32 * ROW_H, label, value.as_deref(), unit);
    }

    // The bottom bar: in reach (lit) or not, or no lock.
    let (text, lit) = match (lock, slant) {
        (None, _) => ("NO LOCK", false),
        (Some(_), None) => ("NO TRACK DATA", false),
        (Some(_), Some(r)) if r <= reach => ("IN RANGE", true),
        _ => ("OUT OF RANGE", false),
    };
    let (x, y, bw, bh) = (2, STATUS_TOP as i32, w as i32 - 4, (h - STATUS_TOP) as i32 - 3);
    if lit {
        cv.fill_rect(x, y, bw, bh, YELLOW);
    } else {
        cv.rect(x, y, bw, bh, YELLOW);
    }
    let col = if lit { BLACK } else { YELLOW };
    let mid = STATUS_TOP + (h - STATUS_TOP) / 2.0;
    stroke_text_at(cv, w / 2.0, mid, text, 9.0, PEN, col, Align::Centre);
}

/// A row: the label in its frame, the value black in a yellow field (an empty field without
/// one), the unit after it.
fn row(cv: &mut Canvas, top: f32, label: &str, value: Option<&str>, unit: &str) {
    let (y, rh) = (top as i32 + 2, ROW_H as i32 - 4);
    let mid = top + ROW_H / 2.0;
    cv.rect(2, y, LABEL_W as i32 - 4, rh, YELLOW);
    stroke_text_at(cv, 8.0, mid, label, LABEL, PEN, YELLOW, Align::Left);
    let x = LABEL_W as i32;
    match value {
        Some(v) => {
            cv.fill_rect(x, y, VALUE_W as i32, rh, YELLOW);
            stroke_text_at(cv, LABEL_W + VALUE_W - 6.0, mid, v, VALUE, 1.6, BLACK, Align::Right);
        }
        None => cv.rect(x, y, VALUE_W as i32, rh, YELLOW),
    }
    stroke_text_at(cv, LABEL_W + VALUE_W + 8.0, mid, unit, LABEL, PEN, YELLOW, Align::Left);
}
