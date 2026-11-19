//! The air defence consoles as the menu's scenes show them (`menu_scene::consoles`): the same
//! screens drawn from a made-up air picture instead of the simulation's.

use bevy::prelude::*;

use super::ConsoleUi;
use super::scope::{RED, Snapshot, Track, WHITE, YELLOW};
use crate::canvas::Canvas;
use crate::cockpit3d::scene::{SceneScreens, Screen};
use crate::cockpit3d::{ConsoleScreen, EO_CANVAS};
use crate::font::{Align, stroke_text_at};

pub use super::scope::{Kind, Side};

/// What a scene's air defence console shows.
pub struct AirPicture<'a> {
    /// The console's title ("PATRIOT", "GEPARD").
    pub title: &'a str,
    pub gepard: bool,
    /// The radar's pictures, newest first.
    pub history: Vec<Snapshot>,
    /// The locked track (flight unit).
    pub lock: Option<usize>,
    /// The battery's launchers (east, north of the site, m).
    pub launchers: Vec<Vec2>,
    /// The weapon's reach and the scope's range (m, km).
    pub reach: f32,
    pub range_km: f32,
    /// The camera on the lock: its range (m), bearing and elevation (degrees), its zoom's label.
    pub camera: Option<(f32, f32, f32)>,
    pub zoom: &'a str,
    /// The gun or a launcher firing now.
    pub firing: bool,
    /// The signal screen's noise (a battery's), new each redraw.
    pub seed: u32,
}

/// Draws the console's screens from `p`.
pub fn draw_air_defence(screens: &mut SceneScreens, p: &AirPicture) {
    let ui = ConsoleUi {
        snapshots: p.history.iter().cloned().collect(),
        range_km: p.range_km,
        selected: p.lock,
        ..Default::default()
    };
    if let Some(mut tex) = screens.canvas(Screen::Console(ConsoleScreen::Scope)) {
        let view = super::scope::ScopeView {
            title: p.title,
            radar_off: false,
            lock: p.lock,
            launchers: p.launchers.iter().map(|&l| (l, super::scope::GREY)).collect(),
            reach: p.reach,
            gepard: p.gepard,
        };
        super::scope::paint(&mut tex.canvas, &ui, &view);
        tex.dirty = true;
    }
    if let Some(mut tex) = screens.canvas(Screen::Console(ConsoleScreen::Board)) {
        let cv = &mut tex.canvas;
        cv.reset_clip();
        cv.clear(super::scope::BLACK);
        super::board::tabs(cv, super::Tab::Tracks);
        super::board::tracks(cv, &ui);
        super::board::status(cv, (!p.gepard).then_some(false), p.lock, None);
        tex.dirty = true;
    }
    let track = p.lock.and_then(|l| ui.snapshots.front()?.track(l));
    // A battery's radar signal; the Gepard's target data.
    if !p.gepard
        && let Some(mut tex) = screens.canvas(Screen::Console(ConsoleScreen::Info))
    {
        let snap = ui.snapshots.front();
        let echoes = snap.map_or(Vec::new(), |s| {
            s.tracks
                .iter()
                .map(|t| {
                    let rcs = match t.kind {
                        Kind::Missile => 0.08,
                        Kind::Rotary => 8.0,
                        Kind::Fixed => 4.0,
                    };
                    let pos = [t.east as f64, t.north as f64, t.alt as f64];
                    let vel = [t.ve as f64, t.vn as f64, 0.0];
                    super::signal::Echo::of(t.unit, [0.0; 3], pos, vel, rcs, t.kind == Kind::Rotary)
                })
                .collect()
        });
        let missiles =
            snap.map_or(0, |s| s.tracks.iter().filter(|t| t.kind == Kind::Missile).count());
        let view = super::signal::SignalView {
            off: false,
            echoes,
            lock: p.lock,
            range_m: p.range_km as f64 * 1000.0,
            in_reach: true,
            semi: false,
            missiles,
        };
        super::signal::paint(&mut tex.canvas, &view, p.seed);
        tex.dirty = true;
    } else if let Some(mut tex) = screens.canvas(Screen::Console(ConsoleScreen::Info)) {
        let view = super::info::InfoView {
            lock: p.lock,
            track,
            gepard: p.gepard,
            firing: p.firing,
            ready: true,
            reloading: false,
            reach: p.reach,
        };
        super::info::paint(&mut tex.canvas, &view);
        tex.dirty = true;
    }
    if let Some(mut tex) = screens.canvas(Screen::Console(ConsoleScreen::Eo)) {
        eo_overlay(&mut tex.canvas, p, track.and(p.lock));
        tex.transparent = true;
        tex.dirty = true;
    }
}

/// The camera's overlay (`eo.rs`'s, on the track): the reticle, the gate, the lock's number,
/// range, bearing and elevation, the Gepard's reach.
fn eo_overlay(cv: &mut Canvas, p: &AirPicture, lock: Option<usize>) {
    cv.reset_clip();
    cv.clear(crate::canvas::TRANSPARENT);
    let (w, h) = (EO_CANVAS.0 as f32, EO_CANVAS.1 as f32);
    // The camera in FLIR, white hot (as `eo.rs` with FLIR on).
    let col = WHITE;
    let o = Vec2::new(w / 2.0, h / 2.0);
    for d in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
        cv.stroke(o + d * 8.0, o + d * 28.0, 1.2, col);
    }
    stroke_text_at(cv, 8.0, 10.0, "FLIR WHOT", 7.0, 1.0, col, Align::Left);
    stroke_text_at(cv, w - 8.0, 10.0, p.zoom, 7.0, 1.0, col, Align::Right);
    let bottom = h - 10.0;
    let Some((range, brg, elv)) = p.camera else {
        stroke_text_at(cv, w / 2.0, bottom, "NO LOCK", 7.0, 1.0, col, Align::Centre);
        return;
    };
    let g = 18.0;
    for s in
        [Vec2::new(1.0, 1.0), Vec2::new(-1.0, 1.0), Vec2::new(1.0, -1.0), Vec2::new(-1.0, -1.0)]
    {
        cv.stroke(o + Vec2::new(-g, -g) * s, o + Vec2::new(-g + 6.0, -g) * s, 1.2, col);
        cv.stroke(o + Vec2::new(-g, -g) * s, o + Vec2::new(-g, -g + 6.0) * s, 1.2, col);
    }
    let tn = lock.map_or(String::new(), |l| format!("{:04}", 1000 + l));
    stroke_text_at(cv, 8.0, bottom, &format!("TRK {tn}"), 7.0, 1.0, col, Align::Left);
    let km = format!("{:.1} KM", range / 1000.0);
    stroke_text_at(cv, w - 8.0, bottom, &km, 7.0, 1.0, col, Align::Right);
    let be = format!("B{brg:03.0} E{elv:+.0}");
    stroke_text_at(cv, w / 2.0, bottom, &be, 7.0, 1.0, col, Align::Centre);
    if p.gepard {
        let (text, c2) =
            if range <= p.reach { ("IN RANGE", YELLOW) } else { ("OUT OF RANGE", RED) };
        stroke_text_at(cv, w / 2.0, 24.0, text, 7.0, 1.0, c2, Align::Centre);
    }
    if p.firing {
        stroke_text_at(cv, w / 2.0, 36.0, "FIRING", 7.0, 1.0, WHITE, Align::Centre);
    }
}

/// A track of a made-up picture.
#[allow(clippy::too_many_arguments)]
pub fn track(
    unit: Option<usize>,
    east: f32,
    north: f32,
    alt: f32,
    ve: f32,
    vn: f32,
    side: Side,
    kind: Kind,
    name: &str,
) -> Track {
    Track { unit, east, north, alt, ve, vn, side, kind, name: name.into() }
}

/// A picture of tracks.
pub fn snapshot(tracks: Vec<Track>) -> Snapshot {
    Snapshot { tracks }
}
