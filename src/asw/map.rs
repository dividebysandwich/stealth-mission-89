//! The MAP page: the tactical picture, north up, round the aircraft (or where it was panned):
//! the ships, the reported datum and the blockade line, the buoys with their latest bearings,
//! the dipping sonar, the tracker's fixes and tracks (with their course and error), the MAD
//! marks, the torpedoes, the drops waiting for the aircraft, and the route.
//!
//! Touch the sea to add a waypoint after the selected one (or D, C or B first: drop that buoy
//! there as the aircraft passes), touch a waypoint to select it, drag it to move it, drag the sea to pan. DEL removes
//! the selected waypoint (or the mark whose letter was touched), CLR the whole route, CTR puts
//! the aircraft back in the middle.
//!
//! The operator's marks (`marks`): a bearing line from each marked sensor with its letter;
//! a letter on two or more sensors also its estimated position, a ring as wide as its error.

use bevy::prelude::*;
use stealth_sim::asw::{AswCmd, AswCrew, BuoyKind, TorpPhase};

use super::{AswUi, Drag, M_PER_WU, marks, owner_at};
use crate::canvas::Canvas;
use crate::cockpit3d::DragPhase;
use crate::font::{Align, stroke_text_at};
use crate::station::scope::{BLACK, CYAN, DARK, GREEN, GREY, RED, WHITE, YELLOW};

/// The map's area (canvas units) and its centre.
const TOP: f32 = 26.0;
const BOTTOM: f32 = 456.0;
const MID: Vec2 = Vec2::new(320.0, (TOP + BOTTOM) / 2.0);
/// A touch this close (units) picks a waypoint; a drag this long pans.
const PICK: f32 = 10.0;
const MOVED: f32 = 6.0;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Btn {
    Centre,
    Delete,
    Clear,
    RangeIn,
    RangeOut,
    /// A buoy of this kind (1 DIFAR, 2 DICASS, 3 BT) to drop at the next point touched.
    Drop(u8),
}

fn buttons() -> [(Btn, [f32; 8], &'static str); 8] {
    [
        (Btn::Drop(1), [176.0, 3.0, 36.0, 19.0, 0.0, 0.0, 0.0, 0.0], "D"),
        (Btn::Drop(2), [216.0, 3.0, 36.0, 19.0, 0.0, 0.0, 0.0, 0.0], "C"),
        (Btn::Drop(3), [256.0, 3.0, 36.0, 19.0, 0.0, 0.0, 0.0, 0.0], "B"),
        (Btn::Centre, [300.0, 3.0, 50.0, 19.0, 0.0, 0.0, 0.0, 0.0], "CTR"),
        (Btn::Delete, [356.0, 3.0, 50.0, 19.0, 0.0, 0.0, 0.0, 0.0], "DEL"),
        (Btn::Clear, [412.0, 3.0, 50.0, 19.0, 0.0, 0.0, 0.0, 0.0], "CLR"),
        (Btn::RangeIn, [520.0, 3.0, 50.0, 19.0, 0.0, 0.0, 0.0, 0.0], "RNG-"),
        (Btn::RangeOut, [576.0, 3.0, 56.0, 19.0, 0.0, 0.0, 0.0, 0.0], "RNG+"),
    ]
}

fn inside(r: [f32; 8], p: Vec2) -> bool {
    p.x >= r[0] && p.x <= r[0] + r[2] && p.y >= r[1] && p.y <= r[1] + r[3]
}

/// Metres per canvas unit.
fn scale(ui: &AswUi) -> f64 {
    ui.range_km * 1000.0 / ((BOTTOM - TOP) / 2.0) as f64
}

fn centre(ui: &AswUi, sim: &stealth_sim::Sim, owner: u8) -> [f64; 2] {
    ui.centre.or_else(|| owner_at(sim, owner).map(|p| p.0)).unwrap_or_default()
}

/// A point (m east, north) on the map.
fn to_screen(ui: &AswUi, c: [f64; 2], p: [f64; 2]) -> Vec2 {
    let k = scale(ui);
    MID + Vec2::new(((p[0] - c[0]) / k) as f32, (-(p[1] - c[1]) / k) as f32)
}

fn to_map(ui: &AswUi, c: [f64; 2], s: Vec2) -> [f64; 2] {
    let k = scale(ui);
    [c[0] + (s.x - MID.x) as f64 * k, c[1] - (s.y - MID.y) as f64 * k]
}

fn wp_metres(p: (i32, i32)) -> [f64; 2] {
    [p.0 as f64 * M_PER_WU, p.1 as f64 * M_PER_WU]
}

fn wp_units(p: [f64; 2]) -> (i32, i32) {
    ((p[0] / M_PER_WU) as i32, (p[1] / M_PER_WU) as i32)
}

fn waypoint_at(ui: &AswUi, c: [f64; 2], s: Vec2) -> Option<usize> {
    ui.route
        .iter()
        .enumerate()
        .map(|(i, &w)| (i, to_screen(ui, c, wp_metres(w)).distance(s)))
        .filter(|&(_, d)| d < PICK)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

fn send_route(ui: &mut AswUi) {
    ui.route_rev = ui.route_rev.wrapping_add(1);
    let (rev, points) = (ui.route_rev, ui.route.clone());
    ui.outbox.push(AswCmd::Route { rev, points });
}

/// A touch on the page (canvas units).
pub fn touch(ui: &mut AswUi, p: Vec2, phase: DragPhase, _crew: &AswCrew, sim: &stealth_sim::Sim, owner: u8) {
    let c = centre(ui, sim, owner);
    match phase {
        DragPhase::Start => {
            if let Some((b, _, _)) = buttons().into_iter().find(|(_, r, _)| inside(*r, p)) {
                match b {
                    Btn::Centre => ui.centre = None,
                    Btn::Delete => {
                        if let Some(i) = ui.sel_mark.take().filter(|&i| i < ui.marks.len()) {
                            ui.marks.remove(i);
                        } else if let Some(i) = ui.sel_wp.filter(|&i| i < ui.route.len()) {
                            ui.route.remove(i);
                            ui.sel_wp = i.checked_sub(1).or((!ui.route.is_empty()).then_some(0));
                            send_route(ui);
                        }
                    }
                    Btn::Clear => {
                        ui.route.clear();
                        ui.sel_wp = None;
                        send_route(ui);
                    }
                    Btn::RangeIn => ui.range_km = (ui.range_km / 2.0).max(1.0),
                    Btn::RangeOut => ui.range_km = (ui.range_km * 2.0).min(64.0),
                    Btn::Drop(n) => {
                        let kind = BuoyKind::from_key(n);
                        ui.drop = if ui.drop == kind { None } else { kind };
                    }
                }
                return;
            }
            if p.y < TOP || p.y > BOTTOM {
                return;
            }
            // A mark's letter on its line: that mark selected (DEL deletes it).
            if let Some(i) = mark_at(ui, sim, _crew, owner, c, p) {
                ui.sel_mark = Some(i);
                ui.sel_wp = None;
                return;
            }
            ui.sel_mark = None;
            ui.drag = Some(match waypoint_at(ui, c, p) {
                Some(i) => {
                    ui.sel_wp = Some(i);
                    Drag::Wp(i)
                }
                None => Drag::Pan { from: p, centre: c, moved: false },
            });
        }
        DragPhase::Move => match ui.drag {
            Some(Drag::Wp(i)) if i < ui.route.len() => {
                ui.route[i] = wp_units(to_map(ui, c, p));
            }
            Some(Drag::Pan { from, centre, moved }) => {
                if moved || p.distance(from) > MOVED {
                    let k = scale(ui);
                    let d = p - from;
                    ui.centre = Some([centre[0] - d.x as f64 * k, centre[1] + d.y as f64 * k]);
                    ui.drag = Some(Drag::Pan { from, centre, moved: true });
                }
            }
            _ => {}
        },
        DragPhase::End => {
            match ui.drag.take() {
                Some(Drag::Wp(_)) => send_route(ui),
                // A touch on the sea: a buoy dropped there, else a waypoint after the selected.
                Some(Drag::Pan { moved: false, .. }) => {
                    let at = to_map(ui, c, p);
                    if let Some(kind) = ui.drop.take() {
                        ui.outbox.push(AswCmd::DropBuoy {
                            kind,
                            deep: ui.deep,
                            at: Some((at[0] as i32, at[1] as i32)),
                        });
                    } else if ui.route.len() < 16 {
                        let i = ui.sel_wp.map_or(ui.route.len(), |i| (i + 1).min(ui.route.len()));
                        ui.route.insert(i, wp_units(at));
                        ui.sel_wp = Some(i);
                        send_route(ui);
                    }
                }
                _ => {}
            }
        }
    }
}

/// Where a mark's sensor is (m east, north).
fn sensor_at(sim: &stealth_sim::Sim, crew: &AswCrew, owner: u8, sensor: u16) -> Option<[f64; 2]> {
    match sensor {
        0 => crew.dip.wet().then_some(crew.dip.at),
        id => sim.asw().buoys_of(owner).find(|b| b.id == id).map(|b| b.at),
    }
}

/// Where a mark's letter sits on its line (canvas units): this far out from its sensor (m).
const LABEL_OUT: f64 = 1200.0;

fn mark_label(ui: &AswUi, c: [f64; 2], at: [f64; 2], bearing: f32) -> Vec2 {
    let t = (bearing as f64).to_radians();
    to_screen(ui, c, [at[0] + t.sin() * LABEL_OUT, at[1] + t.cos() * LABEL_OUT])
}

/// The mark whose letter is under `s`.
fn mark_at(ui: &AswUi, sim: &stealth_sim::Sim, crew: &AswCrew, owner: u8, c: [f64; 2], s: Vec2) -> Option<usize> {
    ui.marks
        .iter()
        .enumerate()
        .filter_map(|(i, m)| {
            let at = sensor_at(sim, crew, owner, m.sensor)?;
            Some((i, mark_label(ui, c, at, m.bearing).distance(s)))
        })
        .filter(|&(_, d)| d < PICK)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// A short line `len` units from `a` at `bearing` (rad true).
fn ray(a: Vec2, bearing: f32, len: f32) -> Vec2 {
    a + Vec2::new(bearing.sin(), -bearing.cos()) * len
}

pub fn draw(
    cv: &mut Canvas,
    ui: &AswUi,
    crew: &AswCrew,
    sim: &stealth_sim::Sim,
    owner: u8,
    hunt: Option<&stealth_mission::asw::Hunt>,
) {
    cv.reset_clip();
    cv.clear(BLACK);
    let c = centre(ui, sim, owner);
    let k = scale(ui) as f32;
    let at = |p: [f64; 2]| to_screen(ui, c, p);
    stroke_text_at(cv, 8.0, 13.0, "TACTICAL", 10.0, 1.5, YELLOW, Align::Left);
    let rng = format!("{:.0} KM", ui.range_km);
    stroke_text_at(cv, 120.0, 13.0, &rng, 8.0, 1.2, GREEN, Align::Left);
    if let Some(kind) = ui.drop {
        let t = format!("TOUCH THE MAP TO DROP A {}", kind.name());
        stroke_text_at(cv, 320.0, BOTTOM - 10.0, &t, 8.0, 1.2, YELLOW, Align::Centre);
    }
    for (b, r, label) in buttons() {
        let lit = match b {
            Btn::Centre => ui.centre.is_none(),
            Btn::Drop(n) => ui.drop.is_some_and(|k| k.index() as u8 + 1 == n),
            _ => false,
        };
        let col = if matches!(b, Btn::Drop(_)) && lit { YELLOW } else if lit { GREEN } else { GREY };
        cv.rect(r[0] as i32, r[1] as i32, r[2] as i32, r[3] as i32, col);
        stroke_text_at(cv, r[0] + r[2] / 2.0, r[1] + r[3] / 2.0 + 1.0, label, 7.0, 1.1, WHITE, Align::Centre);
    }
    cv.set_clip(0, TOP as i32, 640, BOTTOM as i32);
    // Range rings round the aircraft, a kilometre grid's ticks.
    let here = owner_at(sim, owner);
    if let Some((p, _)) = here {
        let s = at(p);
        for f in [0.5f32, 1.0] {
            cv.ring(s, (BOTTOM - TOP) / 2.0 * f, 0.8, DARK);
        }
    }
    // The ships (the carrier larger).
    let objects = sim.objects();
    for (i, o) in objects.iter().enumerate() {
        let carrier = o.flags & stealth_sim::types::oflags::CARRIER != 0;
        if !carrier && !sim.object_is_ship(i) {
            continue;
        }
        let s = at(stealth_sim::asw::object_metres(o));
        if s.x < -20.0 || s.x > 660.0 || s.y < TOP - 20.0 || s.y > BOTTOM + 20.0 {
            continue;
        }
        let friendly = o.flags & (stealth_sim::types::oflags::FRIENDLY | stealth_sim::types::oflags::NEUTRAL) != 0
            || carrier;
        let col = if o.flags & stealth_sim::types::oflags::DESTROYED != 0 { GREY } else if friendly { CYAN } else { RED };
        let r = if carrier { 6.0 } else { 3.5 };
        cv.stroke(s - Vec2::new(r, 0.0), s + Vec2::new(r, 0.0), 2.0, col);
        cv.stroke(s - Vec2::new(r * 0.5, 2.5), s + Vec2::new(r * 0.5, -2.5), 1.5, col);
    }
    // The datum, the blockade line.
    if let Some(h) = hunt {
        let d = at(stealth_sim::asw::map_metres(h.datum));
        cv.ring(d, 6.0, 1.0, YELLOW);
        cv.stroke(d - Vec2::new(9.0, 0.0), d + Vec2::new(9.0, 0.0), 1.0, YELLOW);
        cv.stroke(d - Vec2::new(0.0, 9.0), d + Vec2::new(0.0, 9.0), 1.0, YELLOW);
        stroke_text_at(cv, d.x + 10.0, d.y - 8.0, "DATUM", 6.0, 1.0, YELLOW, Align::Left);
        for s in &h.subs {
            if let stealth_mission::asw::SubIntent::Blockade { a, b, .. } = s.intent {
                let (a, b) = (at(stealth_sim::asw::map_metres(a)), at(stealth_sim::asw::map_metres(b)));
                cv.dashed(a, b, 10.0, 6.0, 1.2, RED);
                break;
            }
        }
    }
    // The buoys and their bearings.
    for b in sim.asw().buoys_of(owner) {
        let s = at(b.at);
        let col = match b.kind {
            BuoyKind::Difar => GREEN,
            BuoyKind::Dicass => CYAN,
            BuoyKind::Bt => GREY,
        };
        if b.fall > 0.0 {
            cv.ring(s, 4.0, 0.8, GREY);
        } else {
            cv.ring(s, 5.0, 1.4, if ui.sensor == Some(b.id) { YELLOW } else { col });
        }
        let letter = &b.kind.name()[..1];
        stroke_text_at(cv, s.x, s.y + 0.5, letter, 5.0, 1.0, col, Align::Centre);
        stroke_text_at(cv, s.x + 8.0, s.y - 7.0, &format!("B{}", b.id), 5.0, 1.0, col, Align::Left);
        for l in &b.sample.lines {
            if let Some(br) = l.bearing {
                let len = l.range.map_or(20_000.0 / k, |r| r as f32 / k);
                cv.dashed(s, ray(s, (br as f32).to_radians(), len), 4.0, 3.0, 0.8, col);
            }
        }
        if b.active {
            cv.ring(s, 9.0, 0.6, CYAN);
        }
    }
    // The dipping sonar.
    if crew.dip.wet() {
        let s = at(crew.dip.at);
        cv.ring(s, 6.0, 1.2, CYAN);
        stroke_text_at(cv, s.x + 9.0, s.y + 8.0, "DIP", 5.0, 1.0, CYAN, Align::Left);
        for l in &crew.dip.sample.lines {
            if let Some(br) = l.bearing {
                let len = l.range.map_or(20_000.0 / k, |r| r as f32 / k);
                cv.dashed(s, ray(s, (br as f32).to_radians(), len), 4.0, 3.0, 0.8, CYAN);
            }
        }
    }
    // Drops waiting for the aircraft.
    for (kind, _, p) in &crew.pending {
        let s = at(*p);
        cv.rect(s.x as i32 - 4, s.y as i32 - 4, 8, 8, YELLOW);
        stroke_text_at(cv, s.x + 7.0, s.y + 7.0, &kind.name()[..1], 5.0, 1.0, YELLOW, Align::Left);
    }
    // The tracker's fixes: crossed bearings and echoes faint dots, MAD marks red crosses (under
    // the rest).
    for f in &crew.fixes {
        let s = at(f.at);
        if f.how == 2 {
            cv.stroke(s - Vec2::splat(5.0), s + Vec2::splat(5.0), 0.8, RED);
            cv.stroke(s + Vec2::new(-5.0, 5.0), s + Vec2::new(5.0, -5.0), 0.8, RED);
        } else {
            cv.fill_rect(s.x as i32, s.y as i32, 1, 1, DARK);
        }
    }
    // The operator's marks: a bearing line from each marked sensor, its letter on it.
    for (i, m) in ui.marks.iter().enumerate() {
        let Some(from) = sensor_at(sim, crew, owner, m.sensor) else { continue };
        let col = if ui.sel_mark == Some(i) {
            YELLOW
        } else if m.lost(ui.t) {
            GREY
        } else {
            CYAN
        };
        let s = at(from);
        let t = m.bearing.to_radians();
        cv.dashed(s, ray(s, t, 30_000.0 / k), 7.0, 3.0, 1.0, col);
        let l = mark_label(ui, c, from, m.bearing);
        cv.fill_rect(l.x as i32 - 6, l.y as i32 - 6, 12, 12, BLACK);
        cv.rect(l.x as i32 - 6, l.y as i32 - 6, 12, 12, col);
        stroke_text_at(cv, l.x, l.y + 0.5, &m.letter().to_string(), 7.0, 1.2, col, Align::Centre);
    }
    // Each letter on two or more sensors: where its lines cross, the ring its error.
    for id in 0..marks::IDS as u8 {
        let lines: Vec<([f64; 2], f32)> = ui
            .marks
            .iter()
            .filter(|m| m.id == id && !m.lost(ui.t))
            .filter_map(|m| sensor_at(sim, crew, owner, m.sensor).map(|p| (p, m.bearing)))
            .collect();
        let Some((p, err)) = marks::estimate(&lines) else { continue };
        let s = at(p);
        cv.ring(s, (err as f32 / k).max(4.0), 1.4, YELLOW);
        cv.stroke(s - Vec2::new(3.0, 0.0), s + Vec2::new(3.0, 0.0), 1.2, YELLOW);
        cv.stroke(s - Vec2::new(0.0, 3.0), s + Vec2::new(0.0, 3.0), 1.2, YELLOW);
        let t = format!("{} {}x {:.1}KM", marks::letter(id), lines.len(), err / 1000.0);
        // Below its ring (a track's label is above).
        let below = (err as f32 / k).max(4.0) + 9.0;
        stroke_text_at(cv, s.x, s.y + below, &t, 6.0, 1.0, YELLOW, Align::Centre);
    }
    // The tracks: a diamond, its course for two minutes, its error ring.
    for t in &crew.tracks {
        let s = at(t.at);
        let d = 6.0;
        cv.polyline(
            &[s + Vec2::new(0.0, -d), s + Vec2::new(d, 0.0), s + Vec2::new(0.0, d), s + Vec2::new(-d, 0.0), s + Vec2::new(0.0, -d)],
            1.6,
            YELLOW,
        );
        let ahead = at([t.at[0] + t.vel[0] * 120.0, t.at[1] + t.vel[1] * 120.0]);
        cv.stroke(s, ahead, 1.2, YELLOW);
        cv.ring(s, (t.err / k).max(3.0), 0.7, YELLOW);
        let kt = t.vel[0].hypot(t.vel[1]) / 0.514;
        let course = (t.vel[0].atan2(t.vel[1]).to_degrees().rem_euclid(360.0)) as i32;
        stroke_text_at(cv, s.x + 10.0, s.y - 10.0, &format!("S{} {course:03}/{kt:.0}", t.n), 6.0, 1.0, YELLOW, Align::Left);
    }
    // The torpedoes.
    for t in sim.asw().torpedoes.iter().filter(|t| t.owner == owner) {
        let s = at([t.p[0], t.p[1]]);
        let col = if t.phase == TorpPhase::Home { RED } else { WHITE };
        cv.ring(s, 3.0, 1.4, col);
        cv.stroke(s, ray(s, t.heading as f32, 9.0), 1.2, col);
    }
    // The mission's own waypoints (flown after the route), grey.
    let mission: Vec<Vec2> = crew.mission.iter().map(|&w| at(wp_metres(w))).collect();
    for w in mission.windows(2) {
        cv.dashed(w[0], w[1], 6.0, 4.0, 0.8, GREY);
    }
    if let (Some(last), Some(first)) = (ui.route.last(), mission.first()) {
        cv.dashed(at(wp_metres(*last)), *first, 6.0, 4.0, 0.8, GREY);
    }
    for (i, s) in mission.iter().enumerate() {
        cv.rect(s.x as i32 - 3, s.y as i32 - 3, 6, 6, GREY);
        stroke_text_at(cv, s.x + 7.0, s.y + 8.0, &format!("M{}", i + 1), 5.0, 1.0, GREY, Align::Left);
    }
    // The route.
    let pts: Vec<Vec2> = ui.route.iter().map(|&w| at(wp_metres(w))).collect();
    if let Some((p, _)) = here
        && let Some(first) = pts.first()
    {
        cv.dashed(at(p), *first, 5.0, 4.0, 0.8, WHITE);
    }
    if pts.len() > 1 {
        cv.polyline(&pts, 1.0, WHITE);
    }
    for (i, s) in pts.iter().enumerate() {
        let col = if ui.sel_wp == Some(i) { YELLOW } else { WHITE };
        cv.ring(*s, 5.0, 1.2, col);
        stroke_text_at(cv, s.x + 8.0, s.y + 8.0, &format!("{}", i + 1), 6.0, 1.0, col, Align::Left);
    }
    // The aircraft.
    if let Some((p, heading)) = here {
        let s = at(p);
        let h = heading as f32;
        let tip = ray(s, h, 10.0);
        let l = ray(s, h + 2.5, 6.0);
        let r = ray(s, h - 2.5, 6.0);
        cv.polyline(&[tip, l, r, tip], 1.6, GREEN);
    }
    cv.reset_clip();
    cv.line(0, TOP as i32, 640, TOP as i32, DARK);
    cv.line(0, BOTTOM as i32, 640, BOTTOM as i32, DARK);
}
