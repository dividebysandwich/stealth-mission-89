//! The console's air picture: the tracks around the site twice a second, in APP-6 air symbols
//! (friend a dome, hostile a half diamond, unknown a half quatrefoil) with the type inside,
//! a heading line as long as a minute's flight and the last positions as dots. North is up.

use bevy::prelude::*;
use stealth_sim::{AircraftKind, fflags};

use super::{ConsoleUi, Manning, flight_metres};
use crate::canvas::Canvas;
use crate::canvas::CanvasTexture;
use crate::cockpit3d::{ConsoleCanvas, ConsoleScreen, SCOPE_CANVAS};
use crate::combat::Combat;
use crate::font::{Align, stroke_text_at};

/// The radar's picture this often (real seconds).
const PERIOD: f32 = 0.5;
/// Earlier pictures kept (the history dots).
const HISTORY: usize = 6;
/// Tracks this far out at most (m).
pub const RADAR_REACH: f64 = 160_000.0;
/// Heading lines: this long a flight (s).
const LEADER_SECONDS: f32 = 60.0;

pub const BLACK: u8 = 5;
pub const DARK: u8 = 8;
pub const GREY: u8 = 7;
pub const GREEN: u8 = 10;
pub const CYAN: u8 = 11;
pub const RED: u8 = 13;
pub const WHITE: u8 = 15;
pub const YELLOW: u8 = 17;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Friend,
    Hostile,
    Unknown,
}

impl Side {
    pub fn colour(self) -> u8 {
        match self {
            Side::Friend => CYAN,
            Side::Hostile => RED,
            Side::Unknown => YELLOW,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Side::Friend => "FRD",
            Side::Hostile => "HOS",
            Side::Unknown => "UNK",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Fixed,
    Rotary,
    Missile,
}

/// A track of the picture.
#[derive(Debug, Clone)]
pub struct Track {
    /// Flight unit (None: a missile).
    pub unit: Option<usize>,
    /// East, north of the site (m); altitude (m).
    pub east: f32,
    pub north: f32,
    pub alt: f32,
    /// Velocity east, north (m/s).
    pub ve: f32,
    pub vn: f32,
    pub side: Side,
    pub kind: Kind,
    pub name: String,
}

impl Track {
    /// Track number.
    pub fn number(&self) -> String {
        self.unit.map_or_else(|| "MSL".into(), |u| format!("{:04}", 1000 + u))
    }

    pub fn range_km(&self) -> f32 {
        self.east.hypot(self.north) / 1000.0
    }

    /// Bearing from the site (degrees).
    pub fn bearing(&self) -> f32 {
        self.east.atan2(self.north).to_degrees().rem_euclid(360.0)
    }

    pub fn speed_kt(&self) -> f32 {
        self.ve.hypot(self.vn) / 0.514
    }

    pub fn heading(&self) -> f32 {
        self.ve.atan2(self.vn).to_degrees().rem_euclid(360.0)
    }
}

/// One picture of the radar.
#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub tracks: Vec<Track>,
}

impl Snapshot {
    pub fn track(&self, unit: usize) -> Option<&Track> {
        self.tracks.iter().find(|t| t.unit == Some(unit))
    }
}

/// The radar's picture, twice a second.
pub(super) fn snapshot(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    mut ui: ResMut<ConsoleUi>,
    time: Res<Time<Real>>,
) {
    let (Some(m), Some(combat)) = (manning, combat) else { return };
    ui.since_snapshot += time.delta_secs();
    if ui.since_snapshot < PERIOD {
        return;
    }
    ui.since_snapshot = 0.0;
    let sim = &combat.sim;
    let Some(site) = sim.aaa_units().get(m.unit).map(|u| u.metres()) else { return };
    let mut tracks = Vec::new();
    for (i, u) in sim.flights().iter().enumerate() {
        let up = fflags::EXISTS | fflags::AIRBORNE;
        if u.flags & up != up || u.flags & fflags::DESTROYED != 0 || u.speed == 0 {
            continue;
        }
        let p = flight_metres(u);
        let (e, n) = (p[0] - site[0], p[1] - site[1]);
        if e.hypot(n) > RADAR_REACH {
            continue;
        }
        let rotary = sim.aircraft_kind(u.spec) == AircraftKind::Helicopter;
        let kt = if rotary { u.speed as f32 * 0.6 } else { u.speed as f32 };
        let h = u.heading as u16 as f32 / 65536.0 * std::f32::consts::TAU;
        let v = kt * 0.514;
        tracks.push(Track {
            unit: Some(i),
            east: e as f32,
            north: n as f32,
            alt: p[2] as f32,
            ve: h.sin() * v,
            vn: h.cos() * v,
            // Not the campaign's and not friendly: traffic, say.
            side: if sim.friendly_unit(i, u.spec) {
                Side::Friend
            } else if sim.campaign_flight(i).is_some() {
                Side::Hostile
            } else {
                Side::Unknown
            },
            kind: if rotary { Kind::Rotary } else { Kind::Fixed },
            name: sim.aircraft_name(u.spec),
        });
    }
    for msl in sim.ai_missiles() {
        let (e, n) = (msl.pos[0] - site[0], msl.pos[1] - site[1]);
        if e.hypot(n) > RADAR_REACH {
            continue;
        }
        tracks.push(Track {
            unit: None,
            east: e as f32,
            north: n as f32,
            alt: msl.pos[2] as f32,
            ve: msl.vel[0] as f32,
            vn: msl.vel[1] as f32,
            side: if msl.blue { Side::Friend } else { Side::Hostile },
            kind: Kind::Missile,
            name: "Missile".into(),
        });
    }
    ui.snapshots.push_front(Snapshot { tracks });
    ui.snapshots.truncate(HISTORY + 1);
    // A selected track gone from the picture stays selected (it may come back), but not for
    // long.
    if let Some(s) = ui.selected
        && ui.snapshots.iter().all(|p| p.track(s).is_none())
    {
        ui.selected = None;
    }
}

/// The picture's frame: centre and pixels per metre.
fn frame(ui: &ConsoleUi) -> (Vec2, f32) {
    let (w, h) = (SCOPE_CANVAS.0 as f32, SCOPE_CANVAS.1 as f32);
    let r = w.min(h) / 2.0 - 14.0;
    (Vec2::new(w / 2.0, h / 2.0 + 4.0), r / (ui.range_km * 1000.0))
}

fn to_screen(centre: Vec2, k: f32, east: f32, north: f32) -> Vec2 {
    centre + Vec2::new(east, -north) * k
}

/// The track at a click on the picture (uv from its top left).
pub(super) fn pick(ui: &ConsoleUi, uv: Vec2) -> Option<usize> {
    let at = uv * Vec2::new(SCOPE_CANVAS.0 as f32, SCOPE_CANVAS.1 as f32);
    let (centre, k) = frame(ui);
    let snap = ui.snapshots.front()?;
    snap.tracks
        .iter()
        .filter_map(|t| Some((t.unit?, to_screen(centre, k, t.east, t.north).distance(at))))
        .filter(|&(_, d)| d < 14.0)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(u, _)| u)
}

pub(super) fn draw(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    c: Option<Res<crate::campaign::ActiveCampaign>>,
    ui: Res<ConsoleUi>,
    mut screens: Query<(&ConsoleCanvas, &mut CanvasTexture)>,
) {
    let (Some(m), Some(combat), Some(c)) = (manning, combat, c) else { return };
    let Some((_, mut tex)) = screens.iter_mut().find(|(s, _)| s.0 == ConsoleScreen::Scope) else {
        return;
    };
    tex.dirty = true;
    let cv = &mut tex.canvas;
    cv.reset_clip();
    cv.clear(BLACK);
    let sim = &combat.sim;
    let lock = sim.station_of(c.me.0).and_then(|s| s.lock).map(|l| l as usize);
    let (centre, k) = frame(&ui);
    let r = (ui.range_km * 1000.0) * k;
    // Range rings (the outer one solid), the cardinal ticks, the range.
    for i in 1..=4 {
        let rr = r * i as f32 / 4.0;
        if i == 4 {
            cv.ring(centre, rr, 1.0, GREEN);
        } else {
            cv.ring(centre, rr, 0.6, DARK);
        }
    }
    for (d, label) in [(Vec2::NEG_Y, "N"), (Vec2::X, "E"), (Vec2::Y, "S"), (Vec2::NEG_X, "W")] {
        cv.stroke(centre + d * (r - 6.0), centre + d * r, 1.0, GREEN);
        stroke_text_at(
            cv,
            centre.x + d.x * (r + 8.0),
            centre.y + d.y * (r + 8.0),
            label,
            7.0,
            1.0,
            GREEN,
            Align::Centre,
        );
    }
    let w = SCOPE_CANVAS.0 as f32;
    stroke_text_at(cv, 6.0, 8.0, &format!("{:.0} KM", ui.range_km), 7.0, 1.0, GREEN, Align::Left);
    let title = if m.gepard { "GEPARD" } else { "PATRIOT" };
    stroke_text_at(cv, w - 6.0, 8.0, title, 7.0, 1.0, GREEN, Align::Right);
    // The site and, for a battery, its launchers.
    cv.frame(centre - Vec2::splat(3.0), centre + Vec2::splat(3.0), 1.0, WHITE);
    if let Some(site) = sim.aaa_units().get(m.unit).map(|u| u.metres()) {
        if !m.gepard {
            for l in sim.battery_launchers(m.unit) {
                let u = sim.aaa_units()[l];
                let p = u.metres();
                let s = to_screen(centre, k, (p[0] - site[0]) as f32, (p[1] - site[1]) as f32);
                let col = if u.destroyed {
                    RED
                } else if ui.launcher == Some(l) {
                    WHITE
                } else {
                    GREY
                };
                cv.stroke(s - Vec2::new(2.0, 0.0), s + Vec2::new(2.0, 0.0), 1.0, col);
            }
        }
        // The battery's missile reach, the Gepard's.
        if !m.gepard {
            let reach = stealth_sim::station::PATRIOT_MAX_RANGE as f32 * k;
            if reach < r {
                cv.ring(centre, reach, 0.6, GREY);
            }
        }
        if m.gepard {
            let reach = stealth_sim::station::GEPARD_REACH as f32 * k;
            if reach > 4.0 {
                cv.ring(centre, reach, 0.6, GREY);
            }
        }
    }
    let Some(snap) = ui.snapshots.front() else { return };
    cv.set_clip(0, 0, SCOPE_CANVAS.0 as i32 - 1, SCOPE_CANVAS.1 as i32 - 1);
    // History dots (the earlier pictures).
    for old in ui.snapshots.iter().skip(1) {
        for t in &old.tracks {
            if t.unit.is_some() {
                let p = to_screen(centre, k, t.east, t.north);
                if p.distance(centre) <= r {
                    cv.circle(p.x, p.y, 0.9, t.side.colour());
                }
            }
        }
    }
    for t in &snap.tracks {
        let p = to_screen(centre, k, t.east, t.north);
        if p.distance(centre) > r {
            continue;
        }
        let col = t.side.colour();
        if t.kind == Kind::Missile {
            missile(cv, p, Vec2::new(t.ve, -t.vn).normalize_or(Vec2::NEG_Y), col);
            continue;
        }
        // Heading line from the symbol's centre.
        let lead = Vec2::new(t.ve, -t.vn) * LEADER_SECONDS * k;
        if lead.length() > 1.0 {
            cv.stroke(p, p + lead, 1.0, col);
        }
        symbol(cv, p, t.side, t.kind, col);
        let unit = t.unit.unwrap_or(usize::MAX);
        if ui.selected == Some(unit) {
            cv.frame(p - Vec2::splat(10.0), p + Vec2::splat(10.0), 1.0, WHITE);
        }
        if lock == Some(unit) {
            for d in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
                cv.stroke(p + d * 12.0, p + d * 16.0, 1.4, WHITE);
            }
        }
        stroke_text_at(cv, p.x + 11.0, p.y - 6.0, &t.number(), 5.0, 0.8, col, Align::Left);
    }
}

/// An APP-6 air track: the frame by side, the type inside (fixed wing, rotary wing).
fn symbol(cv: &mut Canvas, p: Vec2, side: Side, kind: Kind, col: u8) {
    let s = 7.0;
    match side {
        // Friend: a dome.
        Side::Friend => {
            cv.arc(
                p + Vec2::new(0.0, 2.0),
                s,
                std::f32::consts::PI,
                std::f32::consts::TAU,
                1.2,
                col,
            );
            cv.stroke(p + Vec2::new(-s, 2.0), p + Vec2::new(-s, 4.0), 1.2, col);
            cv.stroke(p + Vec2::new(s, 2.0), p + Vec2::new(s, 4.0), 1.2, col);
        }
        // Hostile: the upper half of a diamond.
        Side::Hostile => {
            let pts = [
                p + Vec2::new(-s, 4.0),
                p + Vec2::new(-s, 1.0),
                p + Vec2::new(0.0, -s),
                p + Vec2::new(s, 1.0),
                p + Vec2::new(s, 4.0),
            ];
            cv.polyline(&pts, 1.2, col);
        }
        // Unknown: the upper half of a quatrefoil.
        Side::Unknown => {
            let l = s * 0.5;
            cv.arc(
                p + Vec2::new(-l, 1.0),
                l,
                std::f32::consts::PI,
                1.5 * std::f32::consts::PI,
                1.2,
                col,
            );
            cv.arc(
                p + Vec2::new(0.0, -l + 1.0),
                l,
                std::f32::consts::PI,
                std::f32::consts::TAU,
                1.2,
                col,
            );
            cv.arc(
                p + Vec2::new(l, 1.0),
                l,
                1.5 * std::f32::consts::PI,
                std::f32::consts::TAU,
                1.2,
                col,
            );
        }
    }
    // The icon: a fixed wing aircraft's wings and fuselage, a rotary wing's bow tie.
    match kind {
        Kind::Fixed => {
            cv.stroke(p + Vec2::new(-3.0, 0.0), p + Vec2::new(3.0, 0.0), 1.0, col);
            cv.stroke(p + Vec2::new(0.0, -2.5), p + Vec2::new(0.0, 2.5), 1.0, col);
        }
        Kind::Rotary => {
            cv.triangle([p, p + Vec2::new(-3.5, -2.0), p + Vec2::new(-3.5, 2.0)], col);
            cv.triangle([p, p + Vec2::new(3.5, -2.0), p + Vec2::new(3.5, 2.0)], col);
        }
        Kind::Missile => {}
    }
}

/// A missile: a short dart along its flight.
fn missile(cv: &mut Canvas, p: Vec2, dir: Vec2, col: u8) {
    let side = Vec2::new(-dir.y, dir.x);
    cv.stroke(p - dir * 3.0, p + dir * 3.0, 1.0, col);
    cv.stroke(p - dir * 3.0, p - dir * 4.5 + side * 1.5, 0.8, col);
    cv.stroke(p - dir * 3.0, p - dir * 4.5 - side * 1.5, 0.8, col);
}
