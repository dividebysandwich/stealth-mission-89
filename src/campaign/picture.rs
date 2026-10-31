//! The air picture the friendly side has, for the war map and the briefing map: its own
//! aircraft and air defences (the Patriot batteries and Gepards a player can man), the enemy
//! fighters its radars see, and the enemy air defences it knows of (from recon, or because they
//! gave themselves away by painting the player or firing).

use bevy_egui::egui;
use stealth_campaign::{CampaignState, Side, SiteKind};
use stealth_sim::aaa::GroundKind;
use stealth_sim::{AircraftKind, Sim, fflags, oflags};

use crate::app6::{Affiliation, Dimension, Echelon, Icon, Symbol};
use crate::map_draw::{self, FRIENDLY_BLUE, Layer, MISSILE_RED, MapLayers, MapTips, MapView, app6};
use crate::menu::{BRIGHT, GREEN};

/// Radar cover: friendly places (ground radars), friendly aircraft, the player (map units).
const GROUND_RADAR: i32 = 0x1400;
const AIR_RADAR: i32 = 0xA00;
const PLAYER_RADAR: i32 = 0x1400;
/// Guns this close to a reconned place are known (6 km).
const RECON_GUNS: i32 = 0x180;

#[derive(Clone)]
pub struct Aircraft {
    /// Flight unit and its type (the unit view checks the slot still holds it).
    pub unit: usize,
    pub spec: i32,
    pub x: i32,
    pub y: i32,
    pub heading: i32,
    pub helicopter: bool,
    /// Its type's name.
    pub name: String,
}

pub struct SamSite {
    pub object: usize,
    pub x: i32,
    pub y: i32,
    pub name: String,
    /// Missile reach and how far it launches at the friendly aircraft (map units).
    pub reach: f32,
    pub engage: f32,
    pub reconned: bool,
}

pub struct Gun {
    /// AAA unit index.
    pub index: usize,
    pub x: i32,
    pub y: i32,
    pub kind: GroundKind,
}

/// A friendly air defence unit: a Patriot battery (its radar), a Gepard, a Stinger team.
pub struct Defence {
    /// AAA unit index.
    pub index: usize,
    pub x: i32,
    pub y: i32,
    pub kind: GroundKind,
    /// The player at its console.
    pub manned: Option<u8>,
}

/// A tank or an infantry fighting vehicle, friendly or seen.
#[derive(Clone, Copy)]
pub struct Vehicle {
    /// AAA unit index.
    pub index: usize,
    pub x: i32,
    pub y: i32,
    pub kind: GroundKind,
    pub blue: bool,
    /// Its heading while its convoy drives (radians, clockwise from north).
    pub heading: Option<f32>,
}

/// Patriot reach on the map (map units).
const PATRIOT_REACH: f32 = stealth_sim::campaign::PATRIOT_REACH as f32;

#[derive(Default)]
pub struct Picture {
    pub friendly: Vec<Aircraft>,
    pub fighters: Vec<Aircraft>,
    pub sams: Vec<SamSite>,
    pub guns: Vec<Gun>,
    pub defences: Vec<Defence>,
    /// Armour: the friendly side's, and the enemy's its ground units or drones see now.
    pub armour: Vec<Vehicle>,
    /// The friendly convoys' ways still to drive (map units), to the place they attack.
    pub columns: Vec<Vec<(i32, i32)>>,
    /// The player in the air.
    pub player: Option<(i32, i32)>,
}

fn near(ax: i32, ay: i32, bx: i32, by: i32, r: i32) -> bool {
    stealth_campaign::dist(ax - bx, ay - by) < r
}

/// What the friendly side knows now.
pub fn picture(state: &CampaignState, sim: &Sim) -> Picture {
    let mut pic = Picture::default();
    let mut eyes: Vec<(i32, i32, i32)> = state
        .places
        .iter()
        .filter(|p| p.owner == Side::Blue)
        .map(|p| (p.x, p.y, GROUND_RADAR))
        .collect();
    if !sim.player_absent() {
        let p = sim.player();
        pic.player = Some((p.mx, p.my));
        eyes.push((p.mx, p.my, PLAYER_RADAR));
    }
    let mut enemies = Vec::new();
    for (i, u) in sim.flights().iter().enumerate() {
        if u.flags & (fflags::EXISTS | fflags::AIRBORNE) != fflags::EXISTS | fflags::AIRBORNE
            || u.flags & fflags::DESTROYED != 0
            || u.speed == 0
        {
            continue;
        }
        let blue = sim.friendly_unit(i, u.spec);
        let a = Aircraft {
            unit: i,
            spec: u.spec,
            x: u.x,
            y: u.y,
            heading: u.heading,
            helicopter: sim.aircraft_kind(u.spec) == AircraftKind::Helicopter,
            name: sim.aircraft_name(u.spec),
        };
        if blue {
            eyes.push((u.x, u.y, AIR_RADAR));
            pic.friendly.push(a);
        } else {
            enemies.push(a);
        }
    }
    pic.fighters = enemies
        .into_iter()
        .filter(|a| eyes.iter().any(|&(x, y, r)| near(a.x, a.y, x, y, r)))
        .collect();
    // SAM sites: reconned, or once detected by a friendly aircraft (`CampaignState` spotted).
    let objects = sim.objects();
    for s in state.sites.iter().filter(|s| s.kind == SiteKind::Sam && s.alive()) {
        let i = s.object as usize;
        let Some(o) = objects.get(i) else { continue };
        if o.unit_type == 0 || o.flags & (oflags::FRIENDLY | oflags::NEUTRAL) != 0 {
            continue;
        }
        let reconned = sim.has_intel(i);
        if reconned || s.spotted || sim.site_engaging(i) {
            pic.sams.push(SamSite {
                object: i,
                x: o.x,
                y: o.y,
                name: sim.threat_name(o.unit_type).to_string(),
                reach: (sim.site_lock_range(o.unit_type) * 64) as f32,
                engage: sim.site_engage_reach(i) as f32,
                reconned,
            });
        }
    }
    // Guns and MANPADS: around reconned places, engaging, or found by a command center's drones.
    let reconned: Vec<(i32, i32)> =
        state.places.iter().filter(|p| p.intel(state.clock)).map(|p| (p.x, p.y)).collect();
    let found = |k: usize| sim.uav_known().contains(&stealth_sim::uav::Seen::Aaa(k as u16));
    for (k, u) in sim.units().iter().enumerate() {
        if u.destroyed {
            continue;
        }
        // Armour, and the guns with a convoy: the enemy's only while seen.
        let moving = sim.unit_moving(k);
        if u.kind.armour() || moving && u.kind == GroundKind::Zsu {
            let blue = sim.aaa_blue(k);
            if blue || sim.unit_seen_by(k, true) {
                let heading = moving.then(|| u.fine.map(|f| f.heading)).flatten();
                if u.kind.armour() {
                    pic.armour.push(Vehicle {
                        index: k,
                        x: u.x,
                        y: u.y,
                        kind: u.kind,
                        blue,
                        heading,
                    });
                } else {
                    pic.guns.push(Gun { index: k, x: u.x, y: u.y, kind: u.kind });
                }
            }
            continue;
        }
        if sim.aaa_blue(k) {
            if matches!(
                u.kind,
                GroundKind::PatriotRadar
                    | GroundKind::Gepard
                    | GroundKind::Manpads
                    | GroundKind::DroneCenter
            ) {
                let manned = sim.manned_by(k);
                pic.defences.push(Defence { index: k, x: u.x, y: u.y, kind: u.kind, manned });
            }
            continue;
        }
        if sim.aaa_engaging(k)
            || found(k)
            || reconned.iter().any(|&(x, y)| near(u.x, u.y, x, y, RECON_GUNS))
        {
            pic.guns.push(Gun { index: k, x: u.x, y: u.y, kind: u.kind });
        }
    }
    for v in state.convoys.iter().filter(|v| v.side == Side::Blue && v.active()) {
        pic.columns.push(v.route_left());
    }
    pic
}

/// A tank or an infantry fighting vehicle: a filled box (crossed for the infantry), its
/// heading a tick when it drives; APP-6: an armour or mechanised infantry unit, `k` its size.
pub(super) fn vehicle(painter: &egui::Painter, p: egui::Pos2, v: &Vehicle, k: f32) {
    if app6() {
        let side = if v.blue { Affiliation::Friend } else { Affiliation::Hostile };
        let icon = if v.kind.tank() { Icon::Armour } else { Icon::MechInfantry };
        let mut sym = Symbol::new(side, Dimension::Ground, icon);
        if let Some(h) = v.heading {
            sym = sym.heading(h);
        }
        sym.draw(painter, p, k, 1.0);
        return;
    }
    let col = if v.blue { FRIENDLY_BLUE } else { MISSILE_RED };
    let stroke = egui::Stroke::new(1.3, col);
    let r = egui::Rect::from_center_size(p, egui::vec2(7.0, 5.0));
    if v.kind.tank() {
        painter.rect_filled(r, 1.0, col);
    } else {
        painter.rect_stroke(r, 0.0, stroke, egui::StrokeKind::Middle);
        painter.line_segment([r.left_top(), r.right_bottom()], stroke);
        painter.line_segment([r.left_bottom(), r.right_top()], stroke);
    }
    if let Some(h) = v.heading {
        let fwd = egui::vec2(h.sin(), -h.cos());
        painter.line_segment([p + fwd * 4.0, p + fwd * 9.0], stroke);
    }
}

/// An aircraft symbol: a dot for a helicopter, an arrowhead along the heading for a plane (APP-6:
/// a fixed or rotary wing air track, heading `None` for the legend's).
pub(super) fn aircraft(painter: &egui::Painter, p: egui::Pos2, a: &Aircraft, col: egui::Color32) {
    if app6() {
        let side = if col == MISSILE_RED { Affiliation::Hostile } else { Affiliation::Friend };
        let icon = if a.helicopter { Icon::RotaryWing } else { Icon::FixedWing };
        let mut sym = Symbol::new(side, Dimension::Air, icon);
        if a.unit != usize::MAX {
            sym = sym.heading(a.heading as f32 / 65536.0 * std::f32::consts::TAU);
        }
        // Dimmed colours (the air defence map's friendly aircraft) dim the symbol.
        sym.draw(painter, p, 0.8, col.a() as f32 / 255.0);
        return;
    }
    if a.helicopter {
        painter.circle_filled(p, 3.0, col);
        painter.circle_stroke(p, 5.0, egui::Stroke::new(1.0, col));
        return;
    }
    let h = a.heading as f32 / 65536.0 * std::f32::consts::TAU;
    let fwd = egui::vec2(h.sin(), -h.cos());
    let side = egui::vec2(fwd.y, -fwd.x);
    let pts = vec![p + fwd * 6.0, p - fwd * 4.0 + side * 4.0, p - fwd * 4.0 - side * 4.0];
    painter.add(egui::Shape::convex_polygon(pts, col, egui::Stroke::NONE));
}

/// A gun: a small cross (a ring for MANPADS); APP-6: a hostile air defence gun or missile
/// unit, `k` its size.
fn gun(painter: &egui::Painter, p: egui::Pos2, kind: GroundKind, k: f32) {
    if app6() {
        let icon =
            if kind == GroundKind::Manpads { Icon::AirDefenceMissile } else { Icon::AirDefenceGun };
        let mut sym = Symbol::new(Affiliation::Hostile, Dimension::Ground, icon);
        if kind == GroundKind::Manpads {
            sym = sym.echelon(Echelon::Team);
        }
        sym.draw(painter, p, k, 1.0);
        return;
    }
    let stroke = egui::Stroke::new(1.5, MISSILE_RED);
    if kind == GroundKind::Manpads {
        painter.circle_stroke(p, 2.5, stroke);
    } else {
        painter.line_segment([p + egui::vec2(-3.0, -3.0), p + egui::vec2(3.0, 3.0)], stroke);
        painter.line_segment([p + egui::vec2(-3.0, 3.0), p + egui::vec2(3.0, -3.0)], stroke);
    }
}

/// Draws the picture's shown `layers`: friendly air defences (a battery with its missile
/// reach), known enemy defences (with their missile reach), enemy fighters, friendly aircraft,
/// the player; and their `tips`. The symbols `k` times their size.
pub fn draw(
    painter: &egui::Painter,
    view: &MapView,
    pic: &Picture,
    layers: &MapLayers,
    tips: &mut MapTips,
    k: f32,
) {
    let scaled =
        |c, draw: &dyn Fn(&egui::Painter, egui::Pos2)| map_draw::scaled(painter, c, k, draw);
    let at = |x: i32, y: i32| view.to_screen(x as f32, y as f32);
    for d in &pic.defences {
        let c = at(d.x, d.y);
        let patriot = d.kind == GroundKind::PatriotRadar;
        if patriot && layers.shows(Layer::PatriotReach) {
            painter.extend(reach_circle(c, PATRIOT_REACH / view.scale));
        }
        if !layers.shows(if patriot { Layer::Patriots } else { Layer::Gepards }) {
            continue;
        }
        let kind = d.kind;
        scaled(c, &|p, c| defence(p, c, kind, 0.8));
        let tip = tips.add(c, defence_name(kind), "", FRIENDLY_BLUE);
        tip.symbol(move |p, c| defence(p, c, kind, 0.8));
        if patriot {
            tip.line(format!("Missile range {:.0} km", PATRIOT_REACH / 64.0));
        }
        if d.manned.is_some() {
            painter.circle_stroke(c, map_draw::halo(9.0) * k, egui::Stroke::new(1.5, GREEN));
            tip.line("Manned by a player");
        }
    }
    for s in pic.sams.iter().filter(|s| layers.shows_type(&s.name)) {
        let c = at(s.x, s.y);
        if layers.shows(Layer::MissileRange) {
            painter.extend(map_draw::missile_circle(c, s.reach / view.scale, 0.7));
        }
        if layers.shows(Layer::EngagementRange) {
            painter.add(map_draw::engagement_circle(c, s.engage / view.scale, 0.8));
        }
        if !layers.shows(Layer::Sams) {
            continue;
        }
        map_draw::site(painter, view, c, &s.name, 1.0, k);
        let tip = tips.add(c, "Enemy SAM site", &s.name, MISSILE_RED);
        tip.symbol(|p, c| map_draw::site_symbol(p, c, 1.0));
        tip.line(format!("Missile range {:.0} km", s.reach / 64.0));
        if s.reconned {
            painter.circle_stroke(c, map_draw::halo(6.0) * k, egui::Stroke::new(1.0, BRIGHT));
            tip.line("Reconned");
        }
    }
    if layers.shows(Layer::Armour) {
        // Our columns' way to the place they attack, an arrowhead at its end.
        let faint = egui::Stroke::new(1.2, FRIENDLY_BLUE.gamma_multiply(0.6));
        for route in &pic.columns {
            let pts: Vec<egui::Pos2> = route.iter().map(|&(x, y)| at(x, y)).collect();
            painter.extend(egui::Shape::dashed_line(&pts, faint, 4.0, 3.0));
            if let [.., a, b] = pts[..] {
                let fwd = (b - a).normalized();
                let side = egui::vec2(-fwd.y, fwd.x);
                let tip = vec![b, b - fwd * 7.0 + side * 4.0, b - fwd * 7.0 - side * 4.0];
                painter.add(egui::Shape::convex_polygon(tip, faint.color, egui::Stroke::NONE));
            }
        }
        for v in &pic.armour {
            let p = at(v.x, v.y);
            scaled(p, &|pa, p| vehicle(pa, p, v, 0.7));
            let kind = match (v.blue, v.kind.tank()) {
                (true, true) => "Friendly tank",
                (true, false) => "Friendly infantry fighting vehicle",
                (false, true) => "Enemy tank",
                (false, false) => "Enemy infantry fighting vehicle",
            };
            let colour = if v.blue { FRIENDLY_BLUE } else { MISSILE_RED };
            let tip = tips.add(p, kind, v.kind.name(), colour);
            let v = *v;
            tip.symbol(move |p, c| vehicle(p, c, &v, 0.7));
            if v.heading.is_some() {
                tip.line("On the move");
            }
            if !v.blue {
                tip.line("In sight");
            }
        }
    }
    for g in &pic.guns {
        let c = at(g.x, g.y);
        if layers.shows(Layer::EngagementRange) {
            let reach = stealth_sim::campaign::gun_reach(g.kind);
            painter.add(map_draw::engagement_circle(c, reach / view.scale, 0.6));
        }
        if layers.shows(Layer::Guns) {
            scaled(c, &|p, c| gun(p, c, g.kind, 0.7));
            let kind = if g.kind == GroundKind::Manpads { "Enemy MANPADS" } else { "Enemy gun" };
            let reach = stealth_sim::campaign::gun_reach(g.kind);
            let what = g.kind;
            tips.add(c, kind, g.kind.name(), MISSILE_RED)
                .line(format!("Engagement range {:.1} km", reach / 64.0))
                .symbol(move |p, c| gun(p, c, what, 0.7));
        }
    }
    if layers.shows(Layer::EnemyFighters) {
        for a in &pic.fighters {
            let p = at(a.x, a.y);
            scaled(p, &|pa, p| aircraft(pa, p, a, MISSILE_RED));
            aircraft_tip(tips, p, a, false);
        }
    }
    for a in &pic.friendly {
        let layer = if a.helicopter { Layer::FriendlyHelicopters } else { Layer::FriendlyAircraft };
        if layers.shows(layer) {
            let p = at(a.x, a.y);
            scaled(p, &|pa, p| aircraft(pa, p, a, FRIENDLY_BLUE));
            aircraft_tip(tips, p, a, true);
        }
    }
    if let Some((x, y)) = pic.player {
        painter.circle_stroke(at(x, y), 6.0 * k, egui::Stroke::new(2.0, GREEN));
        tips.add(at(x, y), "You", "", GREEN).symbol(|p, c| {
            p.circle_stroke(c, 6.0, egui::Stroke::new(2.0, GREEN));
        });
    }
}

/// An aircraft's tooltip: its side, kind and type, its heading.
pub(super) fn aircraft_tip(tips: &mut MapTips, p: egui::Pos2, a: &Aircraft, blue: bool) {
    let kind = match (blue, a.helicopter) {
        (true, false) => "Friendly aircraft",
        (true, true) => "Friendly helicopter",
        (false, false) => "Enemy aircraft (on radar)",
        (false, true) => "Enemy helicopter (on radar)",
    };
    let colour = if blue { FRIENDLY_BLUE } else { MISSILE_RED };
    let heading = (a.heading as u16 as f32 / 65536.0 * 360.0).round() as i32 % 360;
    let a = a.clone();
    tips.add(p, kind, &a.name, colour)
        .line(format!("Heading {heading:03}"))
        .symbol(move |p, c| aircraft(p, c, &a, colour));
}

/// A friendly air defence unit's name on the maps.
pub(super) fn defence_name(kind: GroundKind) -> &'static str {
    match kind {
        GroundKind::PatriotRadar => "Patriot battery",
        k => k.name_on(true),
    }
}

/// A friendly air defence unit: a Patriot battery a square with its launcher rails, a Gepard
/// a square with its two barrels, a Stinger team a ring. APP-6: a friendly air defence missile
/// battery, a tracked air defence gun, a missile team; `k` its size.
pub(super) fn defence(painter: &egui::Painter, p: egui::Pos2, kind: GroundKind, k: f32) {
    if app6() {
        let sym = |icon| Symbol::new(Affiliation::Friend, Dimension::Ground, icon);
        let sym = match kind {
            GroundKind::PatriotRadar => sym(Icon::AirDefenceMissile).echelon(Echelon::Battery),
            GroundKind::Gepard => sym(Icon::AirDefenceGun).tracked(),
            GroundKind::DroneCenter => {
                Symbol::new(Affiliation::Friend, Dimension::Installation, Icon::Uav)
            }
            _ => sym(Icon::AirDefenceMissile).echelon(Echelon::Team),
        };
        sym.draw(painter, p, k, 1.0);
        return;
    }
    let stroke = egui::Stroke::new(1.5, FRIENDLY_BLUE);
    match kind {
        GroundKind::PatriotRadar => {
            let r = egui::Rect::from_center_size(p, egui::vec2(9.0, 9.0));
            painter.rect_filled(r, 0.0, FRIENDLY_BLUE.gamma_multiply(0.35));
            painter.rect_stroke(r, 0.0, stroke, egui::StrokeKind::Middle);
            for dx in [-2.0, 2.0] {
                painter.line_segment([p + egui::vec2(dx, 3.0), p + egui::vec2(dx, -3.0)], stroke);
            }
        }
        GroundKind::Gepard => {
            let r = egui::Rect::from_center_size(p, egui::vec2(7.0, 7.0));
            painter.rect_stroke(r, 0.0, stroke, egui::StrokeKind::Middle);
            for dx in [-1.5, 1.5] {
                painter.line_segment([p + egui::vec2(dx, -3.5), p + egui::vec2(dx, -7.5)], stroke);
            }
        }
        // A drone command center: a square with a drone's wings in it.
        GroundKind::DroneCenter => {
            let r = egui::Rect::from_center_size(p, egui::vec2(9.0, 9.0));
            painter.rect_stroke(r, 0.0, stroke, egui::StrokeKind::Middle);
            painter.line_segment([p + egui::vec2(-3.5, -1.5), p + egui::vec2(0.0, 1.5)], stroke);
            painter.line_segment([p + egui::vec2(0.0, 1.5), p + egui::vec2(3.5, -1.5)], stroke);
        }
        _ => {
            painter.circle_stroke(p, 2.5, stroke);
        }
    }
}

/// A friendly battery's missile reach: a faint dashed circle.
fn reach_circle(c: egui::Pos2, r: f32) -> Vec<egui::Shape> {
    let n = ((r * std::f32::consts::TAU / 2.0) as usize).clamp(24, 720);
    let pts: Vec<egui::Pos2> = (0..=n)
        .map(|k| c + egui::Vec2::angled(k as f32 / n as f32 * std::f32::consts::TAU) * r)
        .collect();
    let stroke = egui::Stroke::new(1.2, FRIENDLY_BLUE.gamma_multiply(0.3));
    egui::Shape::dashed_line(&pts, stroke, 3.0, 5.0)
}

/// The legend lines of the picture, with their layers' checkboxes; the SAM sites' (and a
/// checkbox per SAM type) only with `pic`.
pub fn legend(ui: &mut egui::Ui, layers: &mut MapLayers, pic: Option<&Picture>) {
    // (No unit: no heading staff on the APP-6 symbol.)
    let a = |helicopter| Aircraft {
        unit: usize::MAX,
        spec: 0,
        x: 0,
        y: 0,
        heading: 0,
        helicopter,
        name: String::new(),
    };
    layers.legend(ui, Layer::FriendlyAircraft, "Friendly aircraft", |p, c0| {
        aircraft(p, c0, &a(false), FRIENDLY_BLUE);
    });
    layers.legend(ui, Layer::FriendlyHelicopters, "Friendly helicopter", |p, c0| {
        aircraft(p, c0, &a(true), FRIENDLY_BLUE);
    });
    layers.legend(ui, Layer::EnemyFighters, "Enemy fighter (on radar)", |p, c0| {
        aircraft(p, c0, &a(false), MISSILE_RED);
    });
    layers.legend(ui, Layer::Patriots, "Patriot battery", |p, c0| {
        defence(p, c0, GroundKind::PatriotRadar, 0.8)
    });
    layers.legend(ui, Layer::PatriotReach, "Patriot reach", |p, c0| {
        p.extend(reach_circle(c0, 6.0));
    });
    layers.legend(ui, Layer::Gepards, "Gepard / Stinger", |p, c0| {
        if app6() {
            defence(p, c0 - egui::vec2(7.5, 0.0), GroundKind::Gepard, 0.65);
            defence(p, c0 + egui::vec2(7.5, 1.0), GroundKind::Manpads, 0.65);
        } else {
            defence(p, c0 - egui::vec2(4.0, -2.0), GroundKind::Gepard, 1.0);
            defence(p, c0 + egui::vec2(6.0, 0.0), GroundKind::Manpads, 1.0);
        }
    });
    if let Some(pic) = pic {
        layers.legend(ui, Layer::Sams, "SAM site (known)", |p, c0| {
            map_draw::site_symbol(p, c0, 1.0);
        });
        layers.types(ui, "SAM types", pic.sams.iter().map(|s| s.name.as_str()).collect());
        layers.legend(ui, Layer::MissileRange, "SAM missile range", |p, c0| {
            p.extend(map_draw::missile_circle(c0, 6.0, 1.0));
        });
        // (The route editor has its own line for the engagement range.)
        layers.legend(ui, Layer::EngagementRange, "Engagement range", |p, c0| {
            p.add(map_draw::engagement_circle(c0, 6.0, 1.0));
        });
    }
    let tank = |blue| Vehicle { index: 0, x: 0, y: 0, kind: GroundKind::T90, blue, heading: None };
    let ifv = |blue| Vehicle { kind: GroundKind::Bmp2, ..tank(blue) };
    layers.legend(ui, Layer::Armour, "Tank / IFV (enemy: in sight)", |p, c0| {
        let (dx, k) = if app6() { (7.5, 0.65) } else { (5.0, 1.0) };
        vehicle(p, c0 - egui::vec2(dx, 0.0), &tank(true), k);
        vehicle(p, c0 + egui::vec2(dx, 0.0), &ifv(false), k);
    });
    layers.legend(ui, Layer::Guns, "Guns / MANPADS (known)", |p, c0| {
        if app6() {
            gun(p, c0 - egui::vec2(7.5, 0.0), GroundKind::Zsu, 0.65);
            gun(p, c0 + egui::vec2(7.5, 1.0), GroundKind::Manpads, 0.65);
        } else {
            gun(p, c0 - egui::vec2(4.0, 0.0), GroundKind::Zsu, 1.0);
            gun(p, c0 + egui::vec2(5.0, 0.0), GroundKind::Manpads, 1.0);
        }
    });
    // Not layers: lined up with the lines with a checkbox.
    let unchecked = |ui: &mut egui::Ui, text, draw: &dyn Fn(&egui::Painter, egui::Pos2)| {
        ui.horizontal(|ui| {
            ui.add_space(ui.spacing().icon_width);
            map_draw::legend(ui, text, draw);
        });
    };
    unchecked(ui, "Reconned", &|p, c0| {
        p.circle_stroke(c0, if app6() { 7.5 } else { 6.0 }, egui::Stroke::new(1.0, BRIGHT));
    });
    unchecked(ui, "You", &|p, c0| {
        p.circle_stroke(c0, 5.0, egui::Stroke::new(2.0, GREEN));
    });
}

/// The unit nearest a screen point on the map (within a few points, `k` times as many for
/// symbols drawn `k` times their size), and its name.
pub fn pick(
    view: &MapView,
    pic: &Picture,
    at: egui::Pos2,
    k: f32,
) -> Option<(super::unit_view::UnitRef, String)> {
    use super::unit_view::UnitRef;
    let screen = |x: i32, y: i32| view.to_screen(x as f32, y as f32);
    let mut best: Option<(f32, UnitRef, String)> = None;
    let mut consider = |p: egui::Pos2, r: UnitRef, name: String| {
        let d = p.distance(at);
        if d < 12.0 * k && best.as_ref().is_none_or(|b| d < b.0) {
            best = Some((d, r, name));
        }
    };
    for a in pic.friendly.iter().chain(&pic.fighters) {
        consider(screen(a.x, a.y), UnitRef::Flight { index: a.unit, spec: a.spec }, a.name.clone());
    }
    for s in &pic.sams {
        consider(screen(s.x, s.y), UnitRef::Site(s.object), format!("{} site", s.name));
    }
    for g in &pic.guns {
        consider(screen(g.x, g.y), UnitRef::Aaa(g.index), g.kind.name().to_string());
    }
    for v in &pic.armour {
        consider(screen(v.x, v.y), UnitRef::Aaa(v.index), v.kind.name().to_string());
    }
    for d in &pic.defences {
        consider(screen(d.x, d.y), UnitRef::Aaa(d.index), defence_name(d.kind).to_string());
    }
    best.map(|b| (b.1, b.2))
}
