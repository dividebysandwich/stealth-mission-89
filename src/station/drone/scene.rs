//! The drone command center's screens as the menu's scenes show them (`menu_scene::consoles`):
//! the selected drone's camera overlay and a ground control map, from a made-up drone and what
//! it hunts instead of the simulation's.

use bevy::math::DVec2;
use bevy::prelude::*;
use stealth_sim::uav::{Uav, UavKind, UavMode, UavWp, WpKind};

use super::super::scope::{BLACK, CYAN, DARK, GREEN, GREY, RED, WHITE, YELLOW};
use crate::canvas::{Canvas, TRANSPARENT};
use crate::cockpit3d::{FEED_CANVAS, MAP_CANVAS};
use crate::font::{Align, stroke_text_at};

/// A drone of a scene: its kind, number, mode, place (m east, north, up), heading, pitch and
/// bank (rad), speed (m/s), throttle (0..1), its route (m east, north; the last a terminate
/// point if it dives).
#[allow(clippy::too_many_arguments)]
pub fn uav(
    kind: UavKind,
    id: u16,
    mode: UavMode,
    pos: [f64; 3],
    attitude: [f64; 3],
    speed: f64,
    throttle: f32,
    route: &[(DVec2, WpKind)],
) -> Uav {
    let m = stealth_sim::aaa::M_PER_MAP;
    Uav {
        id,
        center: 0,
        kind,
        flight: 0,
        route: route
            .iter()
            .map(|&(p, k)| UavWp {
                x: (p.x / m) as i32,
                y: 0x8000 - (p.y / m) as i32,
                alt: pos[2] as f32,
                kind: k,
            })
            .collect(),
        next: 0,
        mode,
        battery: 0.78,
        pos,
        heading: attitude[0],
        pitch: attitude[1],
        bank: attitude[2],
        speed,
        ctrl: [0.0, 0.0, throttle],
        yaw: 0.0,
        climb_out: 0.0,
        lock: None,
        vel: [speed * attitude[0].sin(), speed * attitude[0].cos(), 0.0],
        rate: [0.0; 3],
    }
}

/// The camera overlay over the drone's forward picture: its name, the mode, the battery, the
/// lock or the dive, a box on its quarry (canvas units), the ladders, the throttle, the
/// heading tape and the waypoint.
pub fn feed(cv: &mut Canvas, u: &Uav, lock: Option<&str>, quarry: Option<Vec2>) {
    cv.reset_clip();
    cv.clear(TRANSPARENT);
    let (w, h) = (FEED_CANVAS.0 as f32, FEED_CANVAS.1 as f32);
    // The camera in FLIR, white hot (as `feed.rs` with FLIR on).
    let col = WHITE;
    stroke_text_at(cv, 46.0, 9.0, &format!("{} FWD FLIR", u.label()), 6.0, 0.9, col, Align::Left);
    stroke_text_at(cv, w - 46.0, 9.0, u.mode.name(), 6.0, 0.9, col, Align::Right);
    let tank = if u.kind.jet() { "FUEL" } else { "BAT" };
    let battery = format!("{tank} {:.0}%", u.battery * 100.0);
    stroke_text_at(cv, 46.0, 20.0, &battery, 5.0, 0.8, col, Align::Left);
    if u.mode == UavMode::Dive {
        stroke_text_at(cv, w / 2.0, 32.0, "TERMINAL DIVE", 7.0, 1.0, RED, Align::Centre);
    }
    if let Some(name) = lock {
        stroke_text_at(cv, w / 2.0, 58.0, &format!("LOCK {name}"), 7.0, 1.0, RED, Align::Centre);
    }
    if let Some(q) = quarry {
        super::feed::bracket(cv, q, 9.0, RED);
    }
    super::feed::ladders(cv, u, col);
    super::feed::throttle_gauge(cv, u, u.ctrl[2], col);
    super::feed::heading_tape(cv, u, col);
    super::feed::waypoint_line(cv, u);
    let _ = h;
}

/// Something on the scene's map: where (m east, north of the map's middle), its heading
/// (rad), what (a drone's kind, else a vehicle), its colour, its label.
pub struct Mark<'a> {
    pub at: Vec2,
    pub heading: f32,
    pub drone: Option<UavKind>,
    pub col: u8,
    pub label: &'a str,
}

/// The ground control map: a grid, the roads, the marks, the drones' routes; the drones'
/// list on the side.
pub fn map(cv: &mut Canvas, km: f32, marks: &[Mark], route: &[Vec2], list: &[(&str, &str)]) {
    cv.reset_clip();
    cv.clear(BLACK);
    let (w, h) = (MAP_CANVAS.0 as f32, MAP_CANVAS.1 as f32);
    let side = 168.0;
    let mid = Vec2::new(side + (w - side) / 2.0, h / 2.0 - 8.0);
    // Pixels a metre: the map's width is `km` across.
    let k = (w - side) / (km * 1000.0);
    let at = |p: Vec2| mid + Vec2::new(p.x, -p.y) * k;
    // A kilometre grid, a couple of roads and a river.
    let step = 1000.0 * k;
    let mut x = mid.x - (((mid.x - side) / step).floor()) * step;
    while x < w {
        cv.stroke(Vec2::new(x, 0.0), Vec2::new(x, h - 20.0), 0.5, DARK);
        x += step;
    }
    let mut y = mid.y - ((mid.y / step).floor()) * step;
    while y < h - 20.0 {
        cv.stroke(Vec2::new(side, y), Vec2::new(w, y), 0.5, DARK);
        y += step;
    }
    let roads = [
        [
            Vec2::new(-9000.0, -2500.0),
            Vec2::new(-1500.0, -600.0),
            Vec2::new(3000.0, 400.0),
            Vec2::new(9000.0, 2600.0),
        ],
        [
            Vec2::new(-2000.0, 6000.0),
            Vec2::new(-800.0, 1500.0),
            Vec2::new(200.0, -2000.0),
            Vec2::new(900.0, -6000.0),
        ],
    ];
    for r in roads {
        let pts: Vec<Vec2> = r.iter().map(|&p| at(p)).collect();
        cv.polyline(&pts, 1.0, GREY);
    }
    let river: Vec<Vec2> = (0..12)
        .map(|i| {
            at(Vec2::new(-9000.0 + i as f32 * 1600.0, 3500.0 + (i as f32 * 0.9).sin() * 900.0))
        })
        .collect();
    cv.polyline(&river, 1.4, CYAN);
    cv.set_clip(side as i32, 0, w as i32 - 1, h as i32 - 21);
    if route.len() > 1 {
        let pts: Vec<Vec2> = route.iter().map(|&p| at(p)).collect();
        cv.polyline(&pts, 1.0, YELLOW);
        for &p in &pts {
            cv.ring(p, 4.0, 1.2, YELLOW);
        }
    }
    for m in marks {
        let p = at(m.at);
        match m.drone {
            Some(kind) => super::map::drone(cv, p, m.heading, kind, m.col),
            None => cv.frame(p - Vec2::splat(3.0), p + Vec2::splat(3.0), 1.2, m.col),
        }
        stroke_text_at(cv, p.x + 8.0, p.y - 6.0, m.label, 5.0, 0.8, m.col, Align::Left);
    }
    cv.reset_clip();
    // The side: the center's drones.
    cv.stroke(Vec2::new(side, 0.0), Vec2::new(side, h), 1.0, GREEN);
    stroke_text_at(cv, 8.0, 12.0, "DRONES", 7.0, 1.0, GREEN, Align::Left);
    for (i, (name, mode)) in list.iter().enumerate() {
        let y = 34.0 + i as f32 * 34.0;
        let col = if i == 0 { WHITE } else { GREEN };
        if i == 0 {
            cv.frame(Vec2::new(4.0, y - 12.0), Vec2::new(side - 4.0, y + 14.0), 1.0, WHITE);
        }
        stroke_text_at(cv, 10.0, y, name, 7.0, 1.0, col, Align::Left);
        stroke_text_at(cv, 10.0, y + 10.0, mode, 5.0, 0.8, GREY, Align::Left);
    }
    // The scale.
    let bar = 1000.0 * k;
    let y = h - 10.0;
    cv.stroke(Vec2::new(w - 20.0 - bar, y), Vec2::new(w - 20.0, y), 1.0, GREEN);
    stroke_text_at(cv, w - 24.0 - bar, y, "1 KM", 5.0, 0.8, GREEN, Align::Right);
}
