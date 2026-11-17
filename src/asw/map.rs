//! The MAP page: the tactical picture, north up, round the aircraft (or where it was panned):
//! the ships, the reported datum and the blockade line, the buoys, the dipping sonar, the
//! tracker's fixes and tracks from echoes and the MAD (with their course and error), the MAD
//! marks, the torpedoes, the drops waiting for the aircraft, the route; with the MAD's bird out
//! its trail (the path coloured by the reading, `mad::trail_colour`) and its trace in a window. No bearing lines of
//! their own: the operator draws those with his marks on the waterfalls.
//!
//! Touch the sea to add a waypoint after the selected one (or D, C or B first: a waypoint where
//! that buoy is dropped as the aircraft passes), touch a waypoint to select it, drag it to move
//! it, drag the sea to pan. The mission's waypoints (M1..) move and go the same way, all but
//! the landing. DEL removes the selected waypoint with its drop (or the mark whose letter was
//! touched), CLR the operator's route and its drops, CTR puts the aircraft back in the middle.
//!
//! The operator's marks (`marks`): a bearing line from each marked sensor with its letter;
//! a letter on two or more sensors also its estimated position, a ring as wide as its error.

use bevy::prelude::*;
use stealth_sim::asw::{AswCmd, AswCrew, BuoyKind, TorpPhase};

use super::{AswUi, Drag, M_PER_WU, marks, owner_at, send_mission, send_route};
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
    /// The dip hover asked of the pilot, or no longer.
    Hover,
}

fn buttons() -> [(Btn, [f32; 8], &'static str); 9] {
    [
        (Btn::Drop(1), [176.0, 3.0, 36.0, 19.0, 0.0, 0.0, 0.0, 0.0], "D"),
        (Btn::Drop(2), [216.0, 3.0, 36.0, 19.0, 0.0, 0.0, 0.0, 0.0], "C"),
        (Btn::Drop(3), [256.0, 3.0, 36.0, 19.0, 0.0, 0.0, 0.0, 0.0], "B"),
        (Btn::Centre, [300.0, 3.0, 50.0, 19.0, 0.0, 0.0, 0.0, 0.0], "CTR"),
        (Btn::Delete, [356.0, 3.0, 50.0, 19.0, 0.0, 0.0, 0.0, 0.0], "DEL"),
        (Btn::Clear, [412.0, 3.0, 50.0, 19.0, 0.0, 0.0, 0.0, 0.0], "CLR"),
        (Btn::Hover, [466.0, 3.0, 50.0, 19.0, 0.0, 0.0, 0.0, 0.0], "HOVER"),
        (Btn::RangeIn, [520.0, 3.0, 50.0, 19.0, 0.0, 0.0, 0.0, 0.0], "RNG-"),
        (Btn::RangeOut, [576.0, 3.0, 56.0, 19.0, 0.0, 0.0, 0.0, 0.0], "RNG+"),
    ]
}

/// A buoy kind's letter, as on the drop buttons (D DIFAR, C DICASS, B BT).
fn key_letter(kind: BuoyKind) -> &'static str {
    match kind {
        BuoyKind::Difar => "D",
        BuoyKind::Dicass => "C",
        BuoyKind::Bt => "B",
    }
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

/// A waypoint the operator may move or delete: the route's (index from 0), then the mission's
/// but its last, the landing.
fn waypoint_at(ui: &AswUi, c: [f64; 2], s: Vec2) -> Option<usize> {
    let movable = ui.route.len() + ui.mission.len().saturating_sub(1);
    ui.route
        .iter()
        .chain(&ui.mission)
        .take(movable)
        .enumerate()
        .map(|(i, &w)| (i, to_screen(ui, c, wp_metres(w)).distance(s)))
        .filter(|&(_, d)| d < PICK)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// Waypoint `i` (the route's, then the mission's).
fn point_mut(ui: &mut AswUi, i: usize) -> Option<&mut (i32, i32)> {
    let n = ui.route.len();
    if i < n { ui.route.get_mut(i) } else { ui.mission.get_mut(i - n) }
}

/// The edited waypoint `i` to the simulation: the route or the mission's.
fn send_point(ui: &mut AswUi, i: usize) {
    if i < ui.route.len() { send_route(ui) } else { send_mission(ui) }
}

/// A new buoy drop on the route can be had: one of the kind aboard and not already waiting,
/// and no more in the water and waiting than the receivers follow.
fn drop_refused(
    ui: &AswUi,
    crew: &AswCrew,
    sim: &stealth_sim::Sim,
    owner: u8,
    kind: BuoyKind,
) -> Option<String> {
    let waiting = ui.drops.iter().flatten().filter(|d| d.0 == kind).count();
    if waiting >= crew.buoys_left[kind.index()] as usize {
        return Some(format!("NO {} LEFT", kind.name()));
    }
    let live = sim.asw().buoys_of(owner).count() + ui.drops.iter().flatten().count();
    if live >= stealth_sim::asw::MAX_BUOYS {
        return Some(format!("{} BUOYS ALREADY", stealth_sim::asw::MAX_BUOYS));
    }
    None
}

/// A touch on the page (canvas units).
pub fn touch(ui: &mut AswUi, p: Vec2, phase: DragPhase, crew: &AswCrew, sim: &stealth_sim::Sim, owner: u8) {
    let c = centre(ui, sim, owner);
    match phase {
        DragPhase::Start => {
            if let Some((b, _, _)) = buttons().into_iter().find(|(_, r, _)| inside(*r, p)) {
                match b {
                    Btn::Centre => ui.centre = None,
                    Btn::Delete => {
                        if let Some(i) = ui.sel_mark.take().filter(|&i| i < ui.marks.len()) {
                            ui.marks.remove(i);
                        } else if let Some(i) = ui.sel_wp {
                            let n = ui.route.len();
                            if i < n {
                                ui.route.remove(i);
                                ui.drops.resize(n, None);
                                ui.drops.remove(i);
                                send_route(ui);
                            } else if i - n + 1 < ui.mission.len() {
                                // Not the landing.
                                ui.mission.remove(i - n);
                                send_mission(ui);
                            }
                            let all = ui.route.len() + ui.mission.len();
                            ui.sel_wp = i.checked_sub(1).or((all > 0).then_some(0));
                        }
                    }
                    Btn::Clear => {
                        ui.route.clear();
                        ui.drops.clear();
                        ui.sel_wp = None;
                        send_route(ui);
                    }
                    Btn::RangeIn => ui.range_km = (ui.range_km / 2.0).max(1.0),
                    Btn::RangeOut => ui.range_km = (ui.range_km * 2.0).min(64.0),
                    Btn::Drop(n) => {
                        let kind = BuoyKind::from_key(n);
                        ui.drop = if ui.drop == kind { None } else { kind };
                    }
                    Btn::Hover => ui.outbox.push(AswCmd::Hover(!crew.hover_req)),
                }
                return;
            }
            if p.y < TOP || p.y > BOTTOM {
                return;
            }
            // A mark's letter on its line: that mark selected (DEL deletes it).
            if let Some(i) = mark_at(ui, sim, crew, owner, c, p) {
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
            Some(Drag::Wp(i)) => {
                let at = wp_units(to_map(ui, c, p));
                if let Some(w) = point_mut(ui, i) {
                    *w = at;
                }
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
                Some(Drag::Wp(i)) => send_point(ui, i),
                // A touch on the sea: a waypoint after the selected one (a buoy dropped there,
                // on the route), else at the route's end.
                Some(Drag::Pan { moved: false, .. }) => {
                    let at = wp_units(to_map(ui, c, p));
                    let n = ui.route.len();
                    let drop = ui.drop.take();
                    if let Some(why) = drop.and_then(|k| drop_refused(ui, crew, sim, owner, k)) {
                        ui.note(why);
                        return;
                    }
                    let in_mission = ui.sel_wp.filter(|&i| i >= n && i - n + 1 < ui.mission.len());
                    match in_mission {
                        // After a mission point (not past the landing).
                        Some(i) if drop.is_none() && ui.mission.len() < 32 => {
                            ui.mission.insert(i - n + 1, at);
                            ui.sel_wp = Some(i + 1);
                            send_mission(ui);
                        }
                        _ if n < stealth_sim::asw::MAX_ROUTE => {
                            let i = ui.sel_wp.filter(|&i| i < n).map_or(n, |i| i + 1);
                            ui.route.insert(i, at);
                            ui.drops.resize(n, None);
                            ui.drops.insert(i, drop.map(|k| (k, ui.deep)));
                            ui.sel_wp = Some(i);
                            send_route(ui);
                        }
                        _ => ui.note("ROUTE FULL"),
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

#[allow(clippy::too_many_arguments)]
pub fn draw(
    cv: &mut Canvas,
    ui: &AswUi,
    crew: &AswCrew,
    sim: &stealth_sim::Sim,
    owner: u8,
    hunt: Option<&stealth_mission::asw::Hunt>,
    mad: &super::mad::MadTrace,
    gain: u8,
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
        if b == Btn::Hover {
            super::sonar::hover_button(cv, [r[0], r[1], r[2], r[3]], crew.hover_req);
            continue;
        }
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
    // A campaign barrier hunt's line.
    if let Some((a, b)) = crew.barrier {
        cv.dashed(at(a), at(b), 10.0, 6.0, 1.2, RED);
    }
    // The buoys.
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
        let letter = key_letter(b.kind);
        stroke_text_at(cv, s.x, s.y + 0.5, letter, 5.0, 1.0, col, Align::Centre);
        stroke_text_at(cv, s.x + 8.0, s.y - 7.0, &format!("B{}", b.id), 5.0, 1.0, col, Align::Left);
        if b.active {
            cv.ring(s, 9.0, 0.6, CYAN);
        }
    }
    // The dipping sonar.
    if crew.dip.wet() {
        let s = at(crew.dip.at);
        cv.ring(s, 6.0, 1.2, CYAN);
        stroke_text_at(cv, s.x + 9.0, s.y + 8.0, "DIP", 5.0, 1.0, CYAN, Align::Left);
    }
    // The tracker's fixes: echoes faint dots, MAD marks red crosses (under the rest), fading
    // over five minutes.
    let now = sim.asw().clock;
    let keep = stealth_sim::asw::FIX_KEEP as f32;
    for f in &crew.fixes {
        let s = at(f.at);
        let age = (now - f.t) as f32;
        if f.how == 2 {
            let Some(col) = marks::faded_colour(RED, age, keep) else { continue };
            cv.stroke(s - Vec2::splat(5.0), s + Vec2::splat(5.0), 0.8, col);
            cv.stroke(s + Vec2::new(-5.0, 5.0), s + Vec2::new(5.0, -5.0), 0.8, col);
        } else if age < keep {
            cv.fill_rect(s.x as i32, s.y as i32, 1, 1, DARK);
        }
    }
    // The MAD trail: where the bird was towed, blue quiet to red at the mark threshold.
    // A gradient: each leg in steps, the reading blended from one point's to the next.
    const STEPS: usize = 4;
    for w in mad.trail.iter().collect::<Vec<_>>().windows(2) {
        if w[1].t - w[0].t > 2.0 {
            continue;
        }
        let (a, b) = (at(w[0].at), at(w[1].at));
        let (va, vb) = ((w[0].mad as f32).abs(), (w[1].mad as f32).abs());
        for k in 0..STEPS {
            let (f0, f1) = (k as f32 / STEPS as f32, (k + 1) as f32 / STEPS as f32);
            let v = va + (vb - va) * (f0 + f1) / 2.0;
            cv.stroke(a.lerp(b, f0), a.lerp(b, f1), 2.0, super::mad::trail_colour(v));
        }
    }
    // The operator's marks: a bearing line from each marked sensor, its letter on it.
    for (i, m) in ui.marks.iter().enumerate() {
        let Some(from) = sensor_at(sim, crew, owner, m.sensor) else { continue };
        // Fading from when its trace was last seen.
        let Some(col) = marks::faded_colour(CYAN, ui.t - m.seen, marks::FADE) else { continue };
        let col = if ui.sel_mark == Some(i) { YELLOW } else { col };
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
            .filter(|m| m.id == id && !m.faded(ui.t))
            .filter_map(|m| sensor_at(sim, crew, owner, m.sensor).map(|p| (p, m.bearing)))
            .collect();
        let Some((p, err)) = marks::estimate(&lines) else { continue };
        let s = at(p);
        cv.ring(s, (err as f32 / k).max(4.0), 1.4, YELLOW);
        cv.stroke(s - Vec2::new(3.0, 0.0), s + Vec2::new(3.0, 0.0), 1.2, YELLOW);
        cv.stroke(s - Vec2::new(0.0, 3.0), s + Vec2::new(0.0, 3.0), 1.2, YELLOW);
        let t = format!("{} {}X {:.1}KM", marks::letter(id), lines.len(), err / 1000.0);
        // Below its ring (a track's label is above).
        let below = (err as f32 / k).max(4.0) + 9.0;
        stroke_text_at(cv, s.x, s.y + below, &t, 6.0, 1.0, YELLOW, Align::Centre);
    }
    // The tracks: a diamond, its course for two minutes, its error ring.
    for t in &crew.tracks {
        let Some(col) = marks::faded_colour(YELLOW, (now - t.last) as f32, stealth_sim::asw::TRACK_DROP as f32)
        else {
            continue;
        };
        let s = at(t.at);
        let d = 6.0;
        cv.polyline(
            &[s + Vec2::new(0.0, -d), s + Vec2::new(d, 0.0), s + Vec2::new(0.0, d), s + Vec2::new(-d, 0.0), s + Vec2::new(0.0, -d)],
            1.6,
            col,
        );
        let ahead = at([t.at[0] + t.vel[0] * 120.0, t.at[1] + t.vel[1] * 120.0]);
        cv.stroke(s, ahead, 1.2, col);
        cv.ring(s, (t.err / k).max(3.0), 0.7, col);
        let kt = t.vel[0].hypot(t.vel[1]) / 0.514;
        let course = (t.vel[0].atan2(t.vel[1]).to_degrees().rem_euclid(360.0)) as i32;
        stroke_text_at(cv, s.x + 10.0, s.y - 10.0, &format!("S{} {course:03}/{kt:.0}", t.n), 6.0, 1.0, col, Align::Left);
    }
    // The torpedoes.
    for t in sim.asw().torpedoes.iter().filter(|t| t.owner == owner) {
        let s = at([t.p[0], t.p[1]]);
        let col = if t.phase == TorpPhase::Home { RED } else { WHITE };
        cv.ring(s, 3.0, 1.4, col);
        cv.stroke(s, ray(s, t.heading as f32, 9.0), 1.2, col);
    }
    // The mission's own waypoints (flown after the route), grey.
    let mission: Vec<Vec2> = ui.mission.iter().map(|&w| at(wp_metres(w))).collect();
    for w in mission.windows(2) {
        cv.dashed(w[0], w[1], 6.0, 4.0, 0.8, GREY);
    }
    if let (Some(last), Some(first)) = (ui.route.last(), mission.first()) {
        cv.dashed(at(wp_metres(*last)), *first, 6.0, 4.0, 0.8, GREY);
    }
    let n = ui.route.len();
    for (i, s) in mission.iter().enumerate() {
        let col = if ui.sel_wp == Some(n + i) { YELLOW } else { GREY };
        let land = i + 1 == mission.len();
        cv.rect(s.x as i32 - 3, s.y as i32 - 3, 6, 6, col);
        let label = if land { "LAND".to_string() } else { format!("M{}", i + 1) };
        stroke_text_at(cv, s.x + 7.0, s.y + 8.0, &label, 5.0, 1.0, col, Align::Left);
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
        let label = format!("{}", i + 1);
        stroke_text_at(cv, s.x + 8.0, s.y + 8.0, &label, 6.0, 1.0, col, Align::Left);
        // A buoy dropped there: its letter in a box round the point.
        if let Some(Some((kind, _))) = ui.drops.get(i) {
            cv.rect(s.x as i32 - 8, s.y as i32 - 8, 16, 16, YELLOW);
            stroke_text_at(
                cv,
                s.x + 8.0,
                s.y - 7.0,
                key_letter(*kind),
                6.0,
                1.0,
                YELLOW,
                Align::Left,
            );
        }
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
    // The bird out: its trace in a window in the corner.
    if mad.out {
        let (x0, y0, x1, y1) = (6.0, BOTTOM - 96.0, 206.0, BOTTOM - 4.0);
        cv.fill_rect(x0 as i32, y0 as i32, (x1 - x0) as i32, (y1 - y0) as i32, BLACK);
        cv.rect(x0 as i32, y0 as i32, (x1 - x0) as i32, (y1 - y0) as i32, GREY);
        let full = super::mad::full_scale(gain);
        let now = mad.samples.back().map_or(0.0, |&v| v as f32 / 10.0);
        stroke_text_at(cv, x0 + 6.0, y0 + 9.0, "MAD", 6.0, 1.1, GREEN, Align::Left);
        let col = super::mad::trail_colour(now * 10.0);
        stroke_text_at(cv, x0 + 40.0, y0 + 9.0, &format!("{now:+.1}"), 6.0, 1.1, col, Align::Left);
        stroke_text_at(cv, x1 - 6.0, y0 + 9.0, &format!("+/-{full} NT"), 6.0, 1.1, CYAN, Align::Right);
        super::mad::plot(cv, mad, [x0 + 14.0, y0 + 17.0, x1 - 14.0, y1 - 17.0], full, 5.0, 10.0);
    }
    cv.reset_clip();
    cv.line(0, TOP as i32, 640, TOP as i32, DARK);
    cv.line(0, BOTTOM as i32, 640, BOTTOM as i32, DARK);
}
