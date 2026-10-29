//! The ground control station: the wide map screen on the drone command center's desk. Left
//! the drone list (or the launch plan), right the map: the terrain (the moving map's, north
//! up), the friendly units, the known contacts, the drones and the selected one's route, the
//! picture in picture in its top right corner and a row of buttons along its bottom. Redrawn
//! ten times a second; the terrain only when the view leaves what was drawn of it. The
//! aircraft and drones in the air show with their heights (the enemy's red), for the
//! interceptors.

use bevy::math::DVec2;
use bevy::prelude::*;
use stealth_sim::aaa::{AaaKind, M_PER_MAP};
use stealth_sim::uav::{STORK_SENSOR, Uav, UavKind, UavMode, UavWp, WpKind};
use stealth_sim::{Sim, fflags};

use super::super::scope::{BLACK, CYAN, DARK, GREEN, GREY, RED, WHITE, YELLOW};
use super::{DroneUi, ZOOMS_KM, center_uavs};
use crate::canvas::{Canvas, CanvasTexture, TRANSPARENT};
use crate::cockpit3d::{ConsoleCanvas, ConsoleScreen, MAP_CANVAS};
use crate::combat::Combat;
use crate::font::{Align, stroke_text_at};
use crate::station::Manning;

pub const CANVAS: (usize, usize) = MAP_CANVAS;
/// The list's width; the map right of it; the button row along the map's bottom.
const SIDEBAR: f32 = 168.0;
pub const MAP_X0: f32 = 170.0;
pub const MAP_W: f32 = CANVAS.0 as f32 - MAP_X0;
const BAR_Y: f32 = 222.0;
const BAR_H: f32 = 18.0;
/// The panning cross's buttons.
const PAD: f32 = 18.0;
/// The picture in picture (`tools/models/build_console_drone.py`, the camera's wide shape):
/// x, y, w, h.
pub const PIP: [f32; 4] = [480.0, 6.0, 154.0, 65.0];
/// The list's rows.
const ROW_Y: f32 = 34.0;
const ROW_H: f32 = 34.0;
/// Redraws a second.
const RATE: f32 = 10.0;
/// The terrain drawn around the view (canvas pixels).
const TERRAIN: (usize, usize) = (960, 480);
const BLUE: u8 = CYAN;

/// The map's on-screen buttons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Button {
    ZoomIn,
    ZoomOut,
    Left,
    Right,
    Up,
    Down,
    Centre,
    Main,
    Manual,
    View,
    Type,
    Next,
    Delete,
    AltUp,
    AltDown,
    NewDrone,
    Kind(UavKind),
    Launch,
    Cancel,
}

/// The map's centre on the canvas.
fn map_centre() -> Vec2 {
    Vec2::new(MAP_X0 + MAP_W / 2.0, BAR_Y / 2.0)
}

/// A map point (map units, x east, y south) on the canvas.
pub fn to_screen(ui: &DroneUi, x: f64, y: f64) -> Vec2 {
    let d = (DVec2::new(x, y) - ui.centre) / ui.scale();
    map_centre() + d.as_vec2()
}

/// The map point at a canvas point.
pub fn to_map(ui: &DroneUi, at: Vec2) -> DVec2 {
    ui.centre + (at - map_centre()).as_dvec2() * ui.scale()
}

pub fn in_map(at: Vec2) -> bool {
    at.x >= MAP_X0 && at.y < BAR_Y
}

pub fn in_pip(at: Vec2) -> bool {
    let [x, y, w, h] = PIP;
    at.x >= x && at.x < x + w && at.y >= y && at.y < y + h
}

/// The list's row at a canvas point (drone index in the list).
pub fn row_at(at: Vec2) -> Option<usize> {
    if at.x >= SIDEBAR || at.y < ROW_Y {
        return None;
    }
    let row = ((at.y - ROW_Y) / ROW_H) as usize;
    (row < stealth_sim::uav::MAX_AIRBORNE).then_some(row)
}

/// The buttons shown now: what, where (x, y, w, h), its label, whether it does anything.
pub fn buttons(ui: &DroneUi, uavs: &[Uav]) -> Vec<(Button, [f32; 4], &'static str, bool)> {
    let route = ui.route(uavs).is_some();
    let wp = ui.wp.is_some() && route;
    let flying = ui.plan.is_none() && ui.selected.is_some();
    // The panning cross in the map's bottom left corner (centring in its middle), the zoom
    // beside it.
    let (cx, cy, s) = (MAP_X0 + 30.0, BAR_Y - 34.0, PAD);
    let mut out = vec![
        (Button::Up, [cx - s / 2.0, cy - s * 1.5 - 2.0, s, s], "", true),
        (Button::Left, [cx - s * 1.5 - 2.0, cy - s / 2.0, s, s], "", true),
        (Button::Centre, [cx - s / 2.0, cy - s / 2.0, s, s], "C", true),
        (Button::Right, [cx + s / 2.0 + 2.0, cy - s / 2.0, s, s], "", true),
        (Button::Down, [cx - s / 2.0, cy + s / 2.0 + 2.0, s, s], "", true),
        (Button::ZoomIn, [cx + s * 2.0 + 6.0, cy - s * 1.5 - 2.0, s, s], "+", true),
        (Button::ZoomOut, [cx + s * 2.0 + 6.0, cy + s / 2.0 + 2.0, s, s], "-", true),
    ];
    // The row along the map's bottom: width, then a gap before the next group (None).
    let row = [
        Some((Button::Type, 36.0, "TYPE", wp)),
        Some((Button::Next, 36.0, "NEXT", wp && flying)),
        Some((Button::Delete, 32.0, "DEL", wp)),
        Some((Button::AltUp, 32.0, "ALT+", wp)),
        Some((Button::AltDown, 32.0, "ALT-", wp)),
        None,
        Some((Button::Main, 40.0, "MAIN", flying)),
        Some((Button::Manual, 46.0, "MANUAL", flying)),
        Some((Button::View, 32.0, "FWD", flying)),
    ];
    let mut x = MAP_X0 + 4.0;
    for b in row {
        match b {
            Some((b, w, label, on)) => {
                out.push((b, [x, BAR_Y, w, BAR_H], label, on));
                x += w + 3.0;
            }
            None => x += 6.0,
        }
    }
    match &ui.plan {
        None => out.push((Button::NewDrone, [4.0, 208.0, SIDEBAR - 8.0, 26.0], "NEW DRONE", true)),
        Some(p) => {
            let w = (SIDEBAR - 12.0) / 2.0;
            let kw = (SIDEBAR - 16.0) / 3.0;
            for (k, kind) in
                [UavKind::Stork, UavKind::Fp2, UavKind::Interceptor].into_iter().enumerate()
            {
                let on = p.kind != kind;
                let at = [4.0 + k as f32 * (kw + 4.0), 150.0, kw, 22.0];
                out.push((Button::Kind(kind), at, kind.short(), on));
            }
            let ok = stealth_sim::uav::route_refusal(p.kind, &p.route).is_none();
            out.push((Button::Launch, [4.0, 206.0, w, 28.0], "LAUNCH", ok));
            out.push((Button::Cancel, [8.0 + w, 206.0, w, 28.0], "CANCEL", true));
        }
    }
    out
}

/// The button at a canvas point (only one that does something).
pub fn button_at(ui: &DroneUi, uavs: &[Uav], at: Vec2) -> Option<Button> {
    buttons(ui, uavs).into_iter().find_map(|(b, [x, y, w, h], _, on)| {
        (on && at.x >= x && at.x < x + w && at.y >= y && at.y < y + h).then_some(b)
    })
}

/// The waypoint of `route` at a canvas point.
pub fn waypoint_at(ui: &DroneUi, route: &[UavWp], at: Vec2) -> Option<usize> {
    route
        .iter()
        .enumerate()
        .map(|(i, w)| (i, to_screen(ui, w.x as f64, w.y as f64).distance(at)))
        .filter(|&(_, d)| d < 8.0)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// Where a drone is on the map (map units), from its flight unit.
fn drone_map(sim: &Sim, u: &Uav) -> DVec2 {
    let f = sim.flights().get(u.flight as usize).copied().unwrap_or_default();
    DVec2::new(f.wx as f64 / 32.0, f.wy as f64 / 32.0)
}

/// The drone at a canvas point.
pub fn drone_at(ui: &DroneUi, sim: &Sim, uavs: &[Uav], at: Vec2) -> Option<u16> {
    uavs.iter()
        .map(|u| {
            let p = drone_map(sim, u);
            (u.id, to_screen(ui, p.x, p.y).distance(at))
        })
        .filter(|&(_, d)| d < 10.0)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(id, _)| id)
}

/// A contact on the map: where (map units), what, found by the drones (else known from the
/// campaign's intelligence).
pub struct Contact {
    pub x: i32,
    pub y: i32,
    pub name: String,
    pub drones: bool,
}

/// The enemy air defences known: those the drones found, and the SAM sites and guns the
/// friendly side knows of.
pub fn contacts(sim: &Sim, campaign: Option<&crate::campaign::ActiveCampaign>) -> Vec<Contact> {
    use stealth_sim::uav::Seen;
    let mut out: Vec<Contact> = sim
        .uav_known()
        .iter()
        .filter_map(|&s| {
            let (x, y) = sim.seen_at(s)?;
            let name = match s {
                Seen::Aaa(k) => sim.aaa_units().get(k as usize)?.kind.name().to_string(),
                Seen::Object(o) => sim.threat_name(sim.objects().get(o as usize)?.unit_type).into(),
            };
            Some(Contact { x, y, name, drones: true })
        })
        .collect();
    if let Some(c) = campaign {
        let pic = crate::campaign::picture::picture(c.state(), sim);
        for s in &pic.sams {
            if !out.iter().any(|o| (o.x, o.y) == (s.x, s.y)) {
                out.push(Contact { x: s.x, y: s.y, name: s.name.clone(), drones: false });
            }
        }
        for g in &pic.guns {
            if !out.iter().any(|o| (o.x, o.y) == (g.x, g.y)) {
                out.push(Contact { x: g.x, y: g.y, name: g.kind.name().into(), drones: false });
            }
        }
    }
    out
}

/// The contact at a canvas point (its map point).
pub fn contact_at(ui: &DroneUi, contacts: &[Contact], at: Vec2) -> Option<(i32, i32)> {
    contacts
        .iter()
        .map(|c| (c, to_screen(ui, c.x as f64, c.y as f64).distance(at)))
        .filter(|&(_, d)| d < 8.0)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(c, _)| (c.x, c.y))
}

/// The theater's towns, roads and woods for the map (`scenery::map_scenery`, made once a
/// theater while a drone command center is manned).
#[derive(Resource)]
pub(super) struct MapDetail {
    theater: String,
    scenery: crate::scenery::MapScenery,
}

/// Cells of the woods' grid across the theater.
const FOREST_GRID: usize = 1024;
/// Map colours: towns, streets, roads.
const TOWN: u8 = DARK;
const STREET: u8 = GREY;
const ROAD: u8 = GREY;
/// Contour line steps (metres), their colours.
const CONTOURS: [f32; 4] = [100.0, 200.0, 500.0, 1000.0];
const CONTOUR: u8 = DARK;
const CONTOUR_MAJOR: u8 = GREY;

/// Makes the map's details once the flight's scenery is laid out (it shares the layout).
pub(super) fn detail(
    mut commands: Commands,
    manning: Option<Res<Manning>>,
    theater: Option<Res<crate::world::Theater>>,
    built: Option<Res<crate::scenery::Scenery>>,
    detail: Option<Res<MapDetail>>,
    assets: crate::models::ModelAssets,
    library: Res<crate::models::ModelLibrary>,
    mut layouts: ResMut<crate::scenery::LayoutCache>,
) {
    let (Some(m), Some(theater)) = (manning, theater) else { return };
    if !m.drones() || detail.is_some_and(|d| d.theater == theater.id) {
        return;
    }
    if built.is_none() || layouts.making(&theater.id) {
        return;
    }
    let day = crate::palette::load_theater_palettes(&theater.id).day;
    let scenery = crate::scenery::map_scenery(
        &theater,
        &assets,
        &library.files,
        &day,
        FOREST_GRID,
        &mut layouts,
    );
    commands.insert_resource(MapDetail { theater: theater.id.clone(), scenery });
}

/// The terrain drawn for the map: around which map point, at which scale.
#[derive(Default)]
pub(super) struct Terrain {
    canvas: Option<Canvas>,
    centre: DVec2,
    scale: f64,
    theater: String,
    /// It has the towns, roads and woods.
    detailed: bool,
    /// The theater's relief (`stealth_host::theater::relief`).
    relief: Option<(String, stealth_sim::relief::Relief)>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    c: Option<Res<crate::campaign::ActiveCampaign>>,
    ui: Res<super::DroneUi>,
    pip: Res<crate::station::PipShown>,
    theater: Option<Res<crate::world::Theater>>,
    raster: Res<crate::mfd::MapRaster>,
    palette: Option<Res<crate::palette::Palette>>,
    detail: Option<Res<MapDetail>>,
    time: Res<Time<Real>>,
    mut since: Local<f32>,
    mut terrain: Local<Terrain>,
    mut screens: Query<(&ConsoleCanvas, &mut CanvasTexture)>,
) {
    let (Some(m), Some(combat)) = (manning, combat) else { return };
    if !m.drones() {
        return;
    }
    *since += time.delta_secs();
    if *since < 1.0 / RATE && !ui.is_changed() {
        return;
    }
    *since = 0.0;
    let Some((_, mut tex)) = screens.iter_mut().find(|(s, _)| s.0 == ConsoleScreen::Map) else {
        return;
    };
    tex.dirty = true;
    let cv = &mut tex.canvas;
    cv.reset_clip();
    cv.clear(BLACK);
    let sim = &combat.sim;
    let uavs = center_uavs(&combat, &m);
    // The terrain under the map.
    if let (Some(theater), Some(palette)) = (theater, palette) {
        let detail = detail.as_deref().filter(|d| d.theater == theater.id);
        ground(cv, &ui, &mut terrain, &theater, &raster, &palette, detail);
    }
    cv.set_clip(MAP_X0 as i32, 0, CANVAS.0 as i32 - 1, BAR_Y as i32 - 2);
    let contacts = contacts(sim, c.as_deref());
    // The friendly air defences, this center.
    for (k, u) in sim.aaa_units().iter().enumerate() {
        if u.destroyed || !sim.aaa_blue(k) {
            continue;
        }
        let p = to_screen(&ui, u.x as f64 + 0.5, u.y as f64 + 0.5);
        if u.kind == AaaKind::DroneCenter {
            cv.frame(p - Vec2::splat(4.0), p + Vec2::splat(4.0), 1.2, BLUE);
            cv.stroke(p + Vec2::new(-3.0, -1.5), p + Vec2::new(0.0, 1.5), 1.0, BLUE);
            cv.stroke(p + Vec2::new(0.0, 1.5), p + Vec2::new(3.0, -1.5), 1.0, BLUE);
            if k == m.unit {
                stroke_text_at(cv, p.x, p.y + 11.0, "DCC", 5.0, 0.8, BLUE, Align::Centre);
            }
        } else if u.kind != AaaKind::PatriotLauncher {
            cv.ring(p, 2.5, 1.0, BLUE);
        }
    }
    // The enemy air defences known: a red diamond, its type.
    for k in &contacts {
        let p = to_screen(&ui, k.x as f64 + 0.5, k.y as f64 + 0.5);
        let s = 5.0;
        let pts = [
            p + Vec2::new(0.0, -s),
            p + Vec2::new(s, 0.0),
            p + Vec2::new(0.0, s),
            p + Vec2::new(-s, 0.0),
            p + Vec2::new(0.0, -s),
        ];
        cv.polyline(&pts, 1.2, RED);
        if k.drones {
            cv.circle(p.x, p.y, 1.2, RED);
        }
        if ui.zoom <= 4 {
            stroke_text_at(cv, p.x + 7.0, p.y - 4.0, &k.name, 5.0, 0.8, RED, Align::Left);
        }
    }
    aircraft(cv, sim, &ui);
    // The drones: a Stork's sensor ring, each one's way to its next waypoint, its symbol.
    for u in &uavs {
        let at = drone_map(sim, u);
        let p = to_screen(&ui, at.x, at.y);
        if u.kind == UavKind::Stork {
            let r = (STORK_SENSOR / M_PER_MAP / ui.scale()) as f32;
            if r > 6.0 {
                cv.ring(p, r, 0.6, DARK);
            }
        }
        let chosen = ui.selected == Some(u.id) && ui.plan.is_none();
        if !chosen && let Some(w) = u.target() {
            let q = to_screen(&ui, w.x as f64, w.y as f64);
            cv.dashed(p, q, 3.0, 3.0, 0.8, GREY);
        }
    }
    // The route shown: the plan's from the rail, else the selected drone's from where it is.
    let route = ui.route(&uavs).map(<[UavWp]>::to_vec);
    if let Some(route) = &route {
        let (from, next, kind) = match (&ui.plan, super::selected(&ui, &uavs)) {
            (Some(p), _) => {
                let (r, _) = sim.uav_ramp(m.unit);
                let x = r[0] / M_PER_MAP;
                let y = 0x8000 as f64 - r[1] / M_PER_MAP;
                (to_screen(&ui, x, y), 0, p.kind)
            }
            (None, Some(u)) => {
                let at = drone_map(sim, u);
                (to_screen(&ui, at.x, at.y), u.next as usize, u.kind)
            }
            _ => (Vec2::ZERO, 0, UavKind::Stork),
        };
        let plan = ui.plan.is_some();
        let pts: Vec<Vec2> = route.iter().map(|w| to_screen(&ui, w.x as f64, w.y as f64)).collect();
        // Legs: the one being flown bright.
        let mut last = from;
        for (i, &q) in pts.iter().enumerate() {
            let col = if plan {
                YELLOW
            } else if i == next {
                WHITE
            } else {
                GREEN
            };
            if i >= next || plan {
                cv.stroke(last, q, if i == next { 1.4 } else { 1.0 }, col);
            } else if i > 0 {
                cv.dashed(pts[i - 1], q, 2.0, 3.0, 0.6, GREY);
            }
            last = q;
        }
        for (i, (w, &q)) in route.iter().zip(&pts).enumerate() {
            let col = if plan {
                YELLOW
            } else if i == next {
                WHITE
            } else if i < next {
                GREY
            } else {
                GREEN
            };
            waypoint(cv, &ui, q, w, kind, col);
            let label = match w.kind {
                WpKind::Nav => format!("{} {:.0}M", i + 1, w.alt),
                WpKind::Loiter => format!("{} LOITER {:.0}M", i + 1, w.alt),
                WpKind::Terminate => format!("{} TERM", i + 1),
            };
            stroke_text_at(cv, q.x + 7.0, q.y + 9.0, &label, 5.0, 0.8, col, Align::Left);
            if ui.wp == Some(i) {
                cv.frame(q - Vec2::splat(8.0), q + Vec2::splat(8.0), 1.0, WHITE);
            }
        }
    }
    for u in &uavs {
        let at = drone_map(sim, u);
        let p = to_screen(&ui, at.x, at.y);
        let chosen = ui.selected == Some(u.id);
        let col = if chosen { WHITE } else { BLUE };
        drone(cv, p, u.heading as f32, u.kind, col);
        let label = format!("{} {:.0}M", u.label(), u.pos[2]);
        stroke_text_at(cv, p.x + 8.0, p.y - 6.0, &label, 5.0, 0.8, col, Align::Left);
        // An interceptor's lock: a line to its target.
        if let Some(j) = u.lock
            && let Some(f) = sim.flights().get(j as usize)
        {
            let q = to_screen(&ui, f.wx as f64 / 32.0, f.wy as f64 / 32.0);
            cv.stroke(p, q, 1.2, RED);
        }
        if chosen {
            cv.ring(p, 10.0, 1.0, WHITE);
        }
    }
    // The scale and what the map is doing, top left.
    let km = ZOOMS_KM[ui.zoom];
    let title = match &ui.plan {
        Some(p) => format!("PLAN {} - CLICK: WAYPOINT, ON A CONTACT: END", p.kind.name()),
        None => format!("{km:.0} KM"),
    };
    stroke_text_at(cv, MAP_X0 + 5.0, 9.0, &title, 6.0, 0.9, WHITE, Align::Left);
    if let Some((note, _)) = &ui.note {
        stroke_text_at(cv, MAP_X0 + 5.0, BAR_Y - 8.0, note, 6.0, 0.9, YELLOW, Align::Left);
    }
    cv.reset_clip();
    // The picture in picture: clear where the camera's picture shows through, framed.
    let [x, y, w, h] = PIP;
    let sel = super::selected(&ui, &uavs);
    if pip.0 {
        cv.fill_rect(x as i32, y as i32, w as i32, h as i32, TRANSPARENT);
        if let Some(u) = sel {
            let cam = ui.cam(u.id);
            let text = format!("{} {}", u.label(), crate::mfd::cam_zoom_label(Some(cam.zoom)));
            stroke_text_at(cv, x + 4.0, y + h - 6.0, &text, 5.0, 0.8, WHITE, Align::Left);
        }
        cv.frame(Vec2::new(x - 1.0, y - 1.0), Vec2::new(x + w, y + h), 1.0, WHITE);
    } else if sel.is_some() && ui.on_main {
        cv.fill_rect(x as i32, y as i32, w as i32, 20, BLACK);
        cv.frame(Vec2::new(x - 1.0, y - 1.0), Vec2::new(x + w, y + 20.0), 1.0, GREY);
        stroke_text_at(cv, x + w / 2.0, y + 10.0, "CAMERA ON MAIN", 5.0, 0.8, GREY, Align::Centre);
    }
    sidebar(cv, sim, &m, &ui, &uavs);
    // The buttons.
    for (b, [bx, by, bw, bh], label, on) in buttons(&ui, &uavs) {
        let col = if on { GREEN } else { DARK };
        cv.fill_rect(bx as i32, by as i32, bw as i32, bh as i32, BLACK);
        cv.frame(Vec2::new(bx, by), Vec2::new(bx + bw, by + bh), 1.0, col);
        let lit = ui.plan.as_ref().is_some_and(|p| b == Button::Kind(p.kind));
        let col = if lit { WHITE } else { col };
        stroke_text_at(cv, bx + bw / 2.0, by + bh / 2.0, label, 6.0, 0.9, col, Align::Centre);
        // The panning cross's arrows.
        let c = Vec2::new(bx + bw / 2.0, by + bh / 2.0);
        let d = match b {
            Button::Up => Some(Vec2::NEG_Y),
            Button::Down => Some(Vec2::Y),
            Button::Left => Some(Vec2::NEG_X),
            Button::Right => Some(Vec2::X),
            _ => None,
        };
        if let Some(d) = d {
            let side = Vec2::new(-d.y, d.x);
            cv.triangle([c + d * 5.0, c - d * 3.0 + side * 5.0, c - d * 3.0 - side * 5.0], col);
        }
    }
}

/// The terrain under the map: drawn again when the view leaves it or the zoom changes.
fn ground(
    cv: &mut Canvas,
    ui: &DroneUi,
    t: &mut Terrain,
    theater: &crate::world::Theater,
    raster: &crate::mfd::MapRaster,
    palette: &crate::palette::Palette,
    detail: Option<&MapDetail>,
) {
    if raster.models().is_empty() {
        return;
    }
    let scale = ui.scale();
    let (bw, bh) = (TERRAIN.0 as f64, TERRAIN.1 as f64);
    let inside = t.canvas.is_some()
        && t.detailed == detail.is_some()
        && t.scale == scale
        && t.theater == theater.id
        && ((ui.centre.x - t.centre.x) / scale).abs() < (bw - MAP_W as f64) / 2.0 - 4.0
        && ((ui.centre.y - t.centre.y) / scale).abs() < (bh - BAR_Y as f64) / 2.0 - 4.0;
    if !inside {
        let mut c = Canvas::new(TERRAIN.0, TERRAIN.1);
        let centre = ui.centre;
        let to_px = |east: f64, north: f64| {
            let p = DVec2::new(east / 32.0, 0x8000 as f64 - north / 32.0);
            ((p - centre) / scale + DVec2::new(bw / 2.0, bh / 2.0)).as_vec2()
        };
        let visible = (
            centre.x * 32.0,
            (0x8000 as f64 - centre.y) * 32.0,
            bw / 2.0 * scale * 32.0,
            bh / 2.0 * scale * 32.0,
        );
        // The moving map's detail for this many map units a pixel (`mfd::rasterise_map`).
        let zoom = (11.0 - (scale * 77.0).log2()).round().clamp(1.0, 9.0) as i32;
        let map16: Vec<u32> = palette
            .remaps
            .iter()
            .find(|t| t.name == "map16")
            .map(|t| t.entries.iter().map(|&e| e as u32).collect())
            .unwrap_or_default();
        // The transparent index is the picture in picture's: never the ground's.
        let remap = |c: u8| match map16.get(c as usize).map_or(c, |w| (*w & 0xFF) as u8) {
            TRANSPARENT => BLACK,
            c => c,
        };
        crate::mfd::rasterise_map(&mut c, raster.models(), theater, &to_px, visible, zoom, &remap);
        let relief = match t.relief.take() {
            Some(r) if r.0 == theater.id => r,
            _ => (theater.id.clone(), stealth_host::theater::relief(&theater.id)),
        };
        heights(&mut c, &relief.1, (centre, scale));
        if let Some(d) = detail {
            scenery(&mut c, &d.scenery, &to_px, (centre, scale));
        }
        *t = Terrain {
            canvas: Some(c),
            centre,
            scale,
            theater: theater.id.clone(),
            detailed: detail.is_some(),
            relief: Some(relief),
        };
    }
    let Some(src) = &t.canvas else { return };
    // The buffer's pixel under the map's top left corner.
    let off = (ui.centre - t.centre) / scale;
    let sx = (bw / 2.0 + off.x - MAP_W as f64 / 2.0).round() as i32;
    let sy = (bh / 2.0 + off.y - BAR_Y as f64 / 2.0).round() as i32;
    cv.blit(src, sx, sy, MAP_W as i32, BAR_Y as i32, MAP_X0 as i32, 0, None, None);
}

/// The relief over the terrain: the slopes turned from the light (north west) shaded (ordered
/// dither), contour lines every `CONTOURS` step at least 1.5 pixels of height apart, every fifth
/// brighter.
fn heights(c: &mut Canvas, relief: &stealth_sim::relief::Relief, (centre, scale): (DVec2, f64)) {
    const BAYER: [[f32; 4]; 4] = [
        [0.0, 8.0, 2.0, 10.0],
        [12.0, 4.0, 14.0, 6.0],
        [3.0, 11.0, 1.0, 9.0],
        [15.0, 7.0, 13.0, 5.0],
    ];
    if relief.bytes().iter().all(|&b| b == 0) {
        return;
    }
    let (w, h) = (c.w, c.h);
    let half = DVec2::new(w as f64, h as f64) / 2.0;
    let at = |x: usize, y: usize| {
        let m = centre + (DVec2::new(x as f64, y as f64) - half) * scale;
        relief.smooth_m(m.x, m.y)
    };
    let z: Vec<f32> = (0..w * h).map(|i| at(i % w, i / w)).collect();
    let px_m = (scale * M_PER_MAP) as f32;
    let step =
        CONTOURS.into_iter().find(|&s| s >= px_m * 1.5).unwrap_or(CONTOURS[CONTOURS.len() - 1]);
    let band = |h: f32| (h / step).floor() as i32;
    for y in 0..h - 1 {
        for x in 0..w - 1 {
            let (here, east, south) = (z[y * w + x], z[y * w + x + 1], z[(y + 1) * w + x]);
            let b = band(here);
            if b != band(east) || b != band(south) {
                let top = b.max(band(east)).max(band(south));
                c.set(x as i32, y as i32, if top % 5 == 0 { CONTOUR_MAJOR } else { CONTOUR });
                continue;
            }
            // Slope towards the light (metres a metre): its shade.
            let (gx, gy) = ((east - here) / px_m, (south - here) / px_m);
            let shade = (-(gx + gy) * std::f32::consts::FRAC_1_SQRT_2 * 1.5).min(0.75);
            if shade * 16.0 > BAYER[y % 4][x % 4] + 0.5 {
                c.set(x as i32, y as i32, BLACK);
            }
        }
    }
}

/// The towns, roads and woods over the terrain: woods darkened (every other pixel), towns
/// filled, the streets and roads as lines.
fn scenery(
    c: &mut Canvas,
    s: &crate::scenery::MapScenery,
    to_px: &dyn Fn(f64, f64) -> Vec2,
    (centre, scale): (DVec2, f64),
) {
    let wu = crate::world::WU;
    let at = |p: DVec2| to_px(p.x / wu, p.y / wu);
    let n = FOREST_GRID;
    if s.forest.len() == n * n {
        let cell = 0x8000 as f64 / n as f64;
        let (w, h) = (c.w as i32, c.h as i32);
        for y in 0..h {
            for x in (y & 1..w).step_by(2) {
                let m = centre
                    + (DVec2::new(x as f64, y as f64) - DVec2::new(w as f64, h as f64) / 2.0)
                        * scale;
                let (col, row) = ((m.x / cell) as i64, (m.y / cell) as i64);
                if (0..n as i64).contains(&col)
                    && (0..n as i64).contains(&row)
                    && s.forest[row as usize * n + col as usize]
                {
                    c.set(x, y, BLACK);
                }
            }
        }
    }
    let px_m = scale * M_PER_MAP;
    for &(p, r) in &s.towns {
        let q = at(p);
        let r = (r / px_m) as f32;
        if r >= 1.0 {
            c.circle(q.x, q.y, r, TOWN);
        } else {
            c.set(q.x as i32, q.y as i32, TOWN);
        }
    }
    // Streets only close in: further out they fill the towns.
    if scale < 4.0 {
        for pts in &s.streets {
            let pts: Vec<Vec2> = pts.iter().map(|&p| at(p)).collect();
            for w in pts.windows(2) {
                c.line(w[0].x as i32, w[0].y as i32, w[1].x as i32, w[1].y as i32, STREET);
            }
        }
    }
    for pts in &s.roads {
        let pts: Vec<Vec2> = pts.iter().map(|&p| at(p)).collect();
        for w in pts.windows(2) {
            c.line(w[0].x as i32, w[0].y as i32, w[1].x as i32, w[1].y as i32, ROAD);
        }
    }
}

/// A waypoint: a nav point's ring, a loiter point's ring round a dot and its circle, a
/// terminate point's cross in a diamond.
fn waypoint(cv: &mut Canvas, ui: &DroneUi, q: Vec2, w: &UavWp, kind: UavKind, col: u8) {
    match w.kind {
        WpKind::Nav => cv.ring(q, 4.0, 1.2, col),
        WpKind::Loiter => {
            cv.ring(q, 4.0, 1.2, col);
            cv.circle(q.x, q.y, 1.4, col);
            let r = (kind.perf().loiter_radius / M_PER_MAP / ui.scale()) as f32;
            if r > 6.0 {
                cv.ring(q, r, 0.7, col);
            }
        }
        WpKind::Terminate => {
            let s = 5.0;
            for (a, b) in
                [(Vec2::new(-s, -s), Vec2::new(s, s)), (Vec2::new(-s, s), Vec2::new(s, -s))]
            {
                cv.stroke(q + a, q + b, 1.4, col);
            }
            let d = s + 2.0;
            let pts = [
                q + Vec2::new(0.0, -d),
                q + Vec2::new(d, 0.0),
                q + Vec2::new(0.0, d),
                q + Vec2::new(-d, 0.0),
                q + Vec2::new(0.0, -d),
            ];
            cv.polyline(&pts, 0.8, col);
        }
    }
}

/// A drone: an arrowhead along its heading (a surveillance drone's open, an attack drone's
/// filled, an interceptor's a slim dart with a cross bar).
fn drone(cv: &mut Canvas, p: Vec2, heading: f32, kind: UavKind, col: u8) {
    let f = Vec2::new(heading.sin(), -heading.cos());
    let r = Vec2::new(-f.y, f.x);
    let (a, b, c) = (p + f * 7.0, p - f * 4.0 + r * 4.5, p - f * 4.0 - r * 4.5);
    match kind {
        UavKind::Fp2 => cv.triangle([a, b, c], col),
        UavKind::Stork => cv.polyline(&[a, b, p - f * 1.5, c, a], 1.2, col),
        UavKind::Interceptor => {
            cv.triangle([a, p - f * 4.0 + r * 2.5, p - f * 4.0 - r * 2.5], col);
            cv.stroke(p - f * 2.0 + r * 5.0, p - f * 2.0 - r * 5.0, 1.2, col);
        }
    }
}

/// The aircraft and drones in the air (not the center's own drones): the enemy's red, the
/// friendly side's blue, each along its heading with its height.
fn aircraft(cv: &mut Canvas, sim: &Sim, ui: &DroneUi) {
    for (j, f) in sim.flights().iter().enumerate() {
        let flying = f.flags & (fflags::EXISTS | fflags::AIRBORNE)
            == fflags::EXISTS | fflags::AIRBORNE
            && f.flags & fflags::DESTROYED == 0
            && f.speed > 0;
        if !flying || sim.uav_of_flight(j).is_some() {
            continue;
        }
        let col = if sim.friendly_unit(j, f.spec) { BLUE } else { RED };
        let p = to_screen(ui, f.wx as f64 / 32.0, f.wy as f64 / 32.0);
        if !in_map(p) {
            continue;
        }
        let h = f.heading as f32 / 65536.0 * std::f32::consts::TAU;
        let d = Vec2::new(h.sin(), -h.cos());
        // A diamond with a stalk along its heading.
        let s = 4.0;
        let pts = [
            p + Vec2::new(0.0, -s),
            p + Vec2::new(s, 0.0),
            p + Vec2::new(0.0, s),
            p + Vec2::new(-s, 0.0),
            p + Vec2::new(0.0, -s),
        ];
        cv.polyline(&pts, 1.0, col);
        cv.stroke(p + d * s, p + d * (s + 6.0), 1.0, col);
        let m = (f.alt.max(0) as f64 * crate::world::WU).round();
        let label = if ui.zoom <= 4 {
            format!("{} {m:.0}M", sim.aircraft_name(f.spec).to_uppercase())
        } else {
            format!("{m:.0}M")
        };
        stroke_text_at(cv, p.x + 7.0, p.y + 7.0, &label, 5.0, 0.8, col, Align::Left);
    }
}

/// The list: the drones in the air (or the launch being planned), the stock and the rail.
fn sidebar(cv: &mut Canvas, sim: &Sim, m: &Manning, ui: &DroneUi, uavs: &[Uav]) {
    cv.stroke(Vec2::new(SIDEBAR, 0.0), Vec2::new(SIDEBAR, CANVAS.1 as f32), 1.0, DARK);
    let [stork, fp2, int] = sim.uav_stock(m.unit);
    let n = uavs.len();
    let max = stealth_sim::uav::MAX_AIRBORNE;
    stroke_text_at(cv, 6.0, 9.0, &format!("DRONES {n}/{max}"), 7.0, 1.0, WHITE, Align::Left);
    let wait = sim.uav_rail_wait(m.unit);
    let (rail, col) =
        if wait > 0 { (format!("RAIL {wait} S"), YELLOW) } else { ("RAIL RDY".into(), GREEN) };
    stroke_text_at(cv, SIDEBAR - 6.0, 9.0, &rail, 5.0, 0.8, col, Align::Right);
    let stock = format!("STOCK STK {stork} FP2 {fp2} INT {int}");
    let col = if stork + fp2 + int > 0 { GREY } else { RED };
    stroke_text_at(cv, 6.0, 23.0, &stock, 5.0, 0.8, col, Align::Left);
    if let Some(p) = &ui.plan {
        plan(cv, p);
        return;
    }
    for (i, u) in uavs.iter().enumerate() {
        let y = ROW_Y + i as f32 * ROW_H;
        let chosen = ui.selected == Some(u.id);
        if chosen {
            cv.frame(Vec2::new(2.0, y), Vec2::new(SIDEBAR - 3.0, y + ROW_H - 3.0), 1.0, WHITE);
        }
        let col = if chosen { WHITE } else { BLUE };
        let mode = match u.mode {
            _ if u.lock.is_some() => "LOCK",
            UavMode::Auto => u.target().map_or("AUTO", |w| match w.kind {
                WpKind::Loiter => "LOITER",
                WpKind::Terminate => "ATTACK",
                WpKind::Nav => "AUTO",
            }),
            m => m.name(),
        };
        stroke_text_at(cv, 6.0, y + 8.0, &u.label(), 6.0, 0.9, col, Align::Left);
        let mcol = match u.mode {
            _ if u.lock.is_some() => RED,
            UavMode::Manual => YELLOW,
            UavMode::Dive => RED,
            _ => GREEN,
        };
        stroke_text_at(cv, SIDEBAR - 6.0, y + 8.0, mode, 5.0, 0.8, mcol, Align::Right);
        let bat = (u.battery * 100.0).round() as i32;
        let end = u.endurance() as i32;
        let line = format!(
            "{:>4}M {:>3}KT {:>3}% {}",
            u.pos[2].round() as i32,
            (u.speed / 0.514).round() as i32,
            bat,
            duration(end)
        );
        let bcol = if bat < 20 {
            RED
        } else if bat < 40 {
            YELLOW
        } else {
            GREY
        };
        stroke_text_at(cv, 6.0, y + 21.0, &line, 5.0, 0.8, bcol, Align::Left);
    }
    if uavs.is_empty() {
        stroke_text_at(cv, 6.0, ROW_Y + 10.0, "NONE AIRBORNE", 5.0, 0.8, GREY, Align::Left);
    }
}

/// The launch being planned: the type, its waypoints, what is missing.
fn plan(cv: &mut Canvas, p: &super::Plan) {
    let perf = p.kind.perf();
    stroke_text_at(
        cv,
        6.0,
        ROW_Y + 8.0,
        &format!("NEW {}", p.kind.name()),
        7.0,
        1.0,
        YELLOW,
        Align::Left,
    );
    let time = if perf.endurance < 3600.0 {
        format!("{:.0} MIN", perf.endurance / 60.0)
    } else {
        format!("{:.1} H", perf.endurance / 3600.0)
    };
    let lines = [
        format!("{} KT  {time}", (perf.cruise / 0.514).round()),
        format!("{} WAYPOINTS", p.route.len()),
        match stealth_sim::uav::route_refusal(p.kind, &p.route) {
            Some(why) => why.to_uppercase(),
            None => "READY".into(),
        },
    ];
    for (i, l) in lines.iter().enumerate() {
        let col = if i == 2 && l != "READY" { RED } else { GREY };
        stroke_text_at(cv, 6.0, ROW_Y + 24.0 + i as f32 * 12.0, l, 5.0, 0.8, col, Align::Left);
    }
    let hint = match p.kind {
        UavKind::Stork => ["END ON A LOITER POINT", ""],
        UavKind::Fp2 => ["END ON A TERMINATE POINT", "(OR LOITER: SURVEY)"],
        UavKind::Interceptor => ["LOITER IN THEIR PATH", "AT THEIR HEIGHT (ALT+/-)"],
    };
    for (i, l) in hint.iter().enumerate() {
        stroke_text_at(cv, 6.0, ROW_Y + 72.0 + i as f32 * 12.0, l, 5.0, 0.8, GREY, Align::Left);
    }
}

/// "1:05" (h:mm), "4:30" under an hour (m:ss).
pub fn duration(s: i32) -> String {
    if s < 3600 {
        format!("{}:{:02}", s / 60, s % 60)
    } else {
        format!("{}:{:02}", s / 3600, s / 60 % 60)
    }
}
