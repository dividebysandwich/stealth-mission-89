//! The WPN page: the stations and their stores (touch a torpedo's station to take it next),
//! the Mk 54's program (its initial heading, the delay before it searches, its search pattern,
//! the depths it searches between), the release's arming, a sketch of the programmed run from
//! the aircraft, and the torpedoes in the water.

use bevy::prelude::*;
use stealth_sim::asw::{AswCmd, AswCrew, MK54_WEAPON, Pattern, TorpPhase, TorpProgram};

use super::{AswUi, owner_at, owner_stores};
use crate::canvas::Canvas;
use crate::font::{Align, stroke_text_at};
use crate::station::scope::{BLACK, CYAN, DARK, GREEN, GREY, RED, WHITE, YELLOW};

/// A button on the page.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Btn {
    Station(u8),
    Heading(i16),
    HeadingAuto,
    Delay(i8),
    Pattern,
    Ceiling(i16),
    Floor(i16),
    Arm,
    Release,
}

const ROW0: f32 = 200.0;
const ROW: f32 = 40.0;

fn buttons(stations: usize) -> Vec<(Btn, [f32; 4], &'static str)> {
    let mut v = Vec::new();
    for s in 0..stations.min(4) {
        v.push((Btn::Station(s as u8), [10.0, 50.0 + s as f32 * 28.0, 290.0, 24.0], ""));
    }
    let row = |k: usize| ROW0 + k as f32 * ROW;
    let pm = |v: &mut Vec<_>, k: usize, minus: Btn, plus: Btn| {
        v.push((minus, [190.0, row(k) - 12.0, 34.0, 24.0], "-"));
        v.push((plus, [230.0, row(k) - 12.0, 34.0, 24.0], "+"));
    };
    pm(&mut v, 0, Btn::Heading(-15), Btn::Heading(15));
    v.push((Btn::HeadingAuto, [270.0, row(0) - 12.0, 40.0, 24.0], "AUTO"));
    pm(&mut v, 1, Btn::Delay(-5), Btn::Delay(5));
    v.push((Btn::Pattern, [190.0, row(2) - 12.0, 120.0, 24.0], "NEXT"));
    pm(&mut v, 3, Btn::Ceiling(-10), Btn::Ceiling(10));
    pm(&mut v, 4, Btn::Floor(-10), Btn::Floor(10));
    v.push((Btn::Arm, [10.0, 410.0, 140.0, 30.0], "ARM"));
    v.push((Btn::Release, [160.0, 410.0, 150.0, 30.0], "RELEASE"));
    v
}

fn inside(r: [f32; 4], p: Vec2) -> bool {
    p.x >= r[0] && p.x <= r[0] + r[2] && p.y >= r[1] && p.y <= r[1] + r[3]
}

/// A touch on the page.
pub fn click(ui: &mut AswUi, p: Vec2, crew: &AswCrew, sim: &stealth_sim::Sim, owner: u8) {
    let stores = owner_stores(sim, owner);
    let Some((b, _, _)) = buttons(stores.len()).into_iter().find(|(_, r, _)| inside(*r, p)) else { return };
    let mut prog = crew.program;
    let here = owner_at(sim, owner).map_or(0.0, |p| p.1.to_degrees());
    match b {
        Btn::Station(s) => {
            if stores.get(s as usize).is_some_and(|&(w, _)| w == MK54_WEAPON) {
                ui.station = s;
            }
            return;
        }
        Btn::Heading(d) => {
            let h = prog.heading.map_or(here.round() as i32, |h| h as i32);
            prog.heading = Some((h + d as i32).rem_euclid(360) as u16);
        }
        Btn::HeadingAuto => prog.heading = None,
        Btn::Delay(d) => prog.delay = (prog.delay as i32 + d as i32).clamp(0, 60) as u8,
        Btn::Pattern => prog.pattern = prog.pattern.next(),
        Btn::Ceiling(d) => prog.ceiling = (prog.ceiling as i32 + d as i32).clamp(10, prog.floor as i32 - 10) as u16,
        Btn::Floor(d) => prog.floor = (prog.floor as i32 + d as i32).clamp(prog.ceiling as i32 + 10, 450) as u16,
        Btn::Arm => {
            ui.outbox.push(AswCmd::Arm(!crew.armed));
            return;
        }
        Btn::Release => {
            super::key(ui, crate::cockpit3d::layout::ConsoleKey::Release, crew, sim, owner);
            return;
        }
    }
    ui.outbox.push(AswCmd::Program(prog));
}

/// Where a torpedo released here on this program runs (m east, north from the release point,
/// at the aircraft's heading `heading`), for the sketch.
pub fn run_path(prog: &TorpProgram, heading: f64, secs: f64) -> Vec<[f64; 2]> {
    let kt = 0.514_444;
    let mut h = prog.heading.map_or(heading, |d| (d as f64).to_radians());
    let base = h;
    let (mut e, mut n) = (0.0, 0.0);
    let mut v = vec![[e, n]];
    let dt = 2.0;
    let mut t = 0.0;
    while t < secs {
        let searching = t >= prog.delay as f64;
        let speed = if searching { 28.0 * kt } else { 40.0 * kt };
        if searching {
            let st = t - prog.delay as f64;
            h = match prog.pattern {
                Pattern::Straight => base,
                Pattern::Circle | Pattern::Helix => base + 28.0 * kt / 350.0 * st,
                Pattern::Snake => base + 45f64.to_radians() * (std::f64::consts::TAU * st / 40.0).sin(),
            };
        }
        e += h.sin() * speed * dt;
        n += h.cos() * speed * dt;
        v.push([e, n]);
        t += dt;
    }
    v
}

pub fn draw(cv: &mut Canvas, ui: &AswUi, crew: &AswCrew, sim: &stealth_sim::Sim, owner: u8) {
    cv.reset_clip();
    cv.clear(BLACK);
    stroke_text_at(cv, 8.0, 13.0, "WEAPONS", 10.0, 1.5, YELLOW, Align::Left);
    let (arm, col) = if crew.armed { ("RELEASE ARMED", RED) } else { ("SAFE", GREEN) };
    stroke_text_at(cv, 300.0, 13.0, arm, 9.0, 1.4, col, Align::Right);
    let stores = owner_stores(sim, owner);
    let names = ["LO", "LI", "RI", "RO"];
    stroke_text_at(cv, 10.0, 40.0, "STATIONS", 7.0, 1.1, CYAN, Align::Left);
    for (b, r, label) in buttons(stores.len()) {
        match b {
            Btn::Station(s) => {
                let (w, n) = stores[s as usize];
                let torp = w == MK54_WEAPON;
                let chosen = torp && s == ui.station;
                cv.rect(r[0] as i32, r[1] as i32, r[2] as i32, r[3] as i32, if chosen { YELLOW } else { DARK });
                let store = match w {
                    MK54_WEAPON => "MK 54 TORPEDO",
                    20 => "AGM-114 HELLFIRE",
                    w if w < 0 => "-",
                    _ => "STORE",
                };
                let t = format!("{} {store}", names[s as usize]);
                stroke_text_at(cv, r[0] + 8.0, r[1] + r[3] / 2.0 + 1.0, &t, 7.0, 1.1, if n > 0 { GREEN } else { GREY }, Align::Left);
                stroke_text_at(cv, r[0] + r[2] - 8.0, r[1] + r[3] / 2.0 + 1.0, &n.max(0).to_string(), 7.0, 1.1, WHITE, Align::Right);
            }
            _ => {
                let lit = matches!(b, Btn::Arm) && crew.armed;
                cv.rect(r[0] as i32, r[1] as i32, r[2] as i32, r[3] as i32, if lit { RED } else { GREY });
                stroke_text_at(cv, r[0] + r[2] / 2.0, r[1] + r[3] / 2.0 + 1.0, label, 7.0, 1.2, WHITE, Align::Centre);
            }
        }
    }
    // The program.
    let p = crew.program;
    stroke_text_at(cv, 10.0, ROW0 - 28.0, "MK 54 PROGRAM", 7.0, 1.1, CYAN, Align::Left);
    let rows = [
        ("HDG", p.heading.map_or("AUTO".to_string(), |h| format!("{h:03}"))),
        ("DELAY", format!("{} S", p.delay)),
        ("PATTERN", p.pattern.name().to_string()),
        ("CEILING", format!("{} M", p.ceiling)),
        ("FLOOR", format!("{} M", p.floor)),
    ];
    for (k, (name, value)) in rows.iter().enumerate() {
        let y = ROW0 + k as f32 * ROW;
        stroke_text_at(cv, 14.0, y, name, 8.0, 1.2, GREEN, Align::Left);
        stroke_text_at(cv, 100.0, y, value, 8.0, 1.2, WHITE, Align::Left);
    }
    // The sketch: the run from here over its first three minutes, north up.
    let (x0, y0, x1, y1) = (330.0, 40.0, 630.0, 340.0);
    cv.rect(x0 as i32, y0 as i32, (x1 - x0) as i32, (y1 - y0) as i32, DARK);
    stroke_text_at(cv, x0 + 6.0, y0 + 10.0, "RUN 3 MIN", 6.0, 1.0, GREY, Align::Left);
    let heading = owner_at(sim, owner).map_or(0.0, |p| p.1);
    let path = run_path(&p, heading, 180.0);
    let reach = path.iter().map(|q| q[0].abs().max(q[1].abs())).fold(500.0, f64::max) * 1.15;
    let c = Vec2::new((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    let k = (x1 - x0) / 2.0 / reach as f32;
    let pts: Vec<Vec2> = path.iter().map(|q| c + Vec2::new(q[0] as f32 * k, -q[1] as f32 * k)).collect();
    cv.polyline(&pts, 1.2, CYAN);
    let enable = (p.delay as usize / 2).min(pts.len() - 1);
    cv.ring(pts[enable], 3.0, 1.0, YELLOW);
    cv.ring(c, 4.0, 1.2, GREEN);
    stroke_text_at(cv, x1 - 6.0, y1 - 8.0, &format!("{:.1} KM", reach / 1000.0), 6.0, 1.0, GREY, Align::Right);
    // The torpedoes in the water.
    stroke_text_at(cv, x0, 360.0, "TORPEDOES", 7.0, 1.1, CYAN, Align::Left);
    let mut row = 0;
    for t in sim.asw().torpedoes.iter().filter(|t| t.owner == owner) {
        let phase = match t.phase {
            TorpPhase::Air => "CHUTE",
            TorpPhase::Run => "RUN",
            TorpPhase::Search => "SEARCH",
            TorpPhase::Home => "HOMING",
        };
        let s = format!("T{} {phase} {:.0}M {:.0}:{:02.0}", t.id % 100, -t.p[2], (t.t / 60.0).floor(), t.t % 60.0);
        let col = if t.phase == TorpPhase::Home { RED } else { WHITE };
        stroke_text_at(cv, x0, 380.0 + row as f32 * 16.0, &s, 7.0, 1.1, col, Align::Left);
        row += 1;
    }
    if row == 0 {
        stroke_text_at(cv, x0, 380.0, "NONE", 7.0, 1.1, DARK, Align::Left);
    }
}
