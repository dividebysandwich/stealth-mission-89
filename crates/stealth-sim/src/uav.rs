//! Campaign: the drones of a drone command center, flown from its console. A center launches
//! up to [`MAX_AIRBORNE`] at a time off its rail: **Stork-100** surveillance drones, whose
//! sensor finds every enemy air defence unit and SAM site within [`STORK_SENSOR`] of them, and
//! **FP-2** attack drones, which dive into a ground point and blow up there.
//!
//! * A drone flies its route by itself (**auto**): through its nav points, round a **loiter**
//!   point on a wide circle, or into a **terminate** point (FP-2 only) in a steep dive. A route
//!   without either ends in a circle round its last point.
//! * Taken over (**manual**), it follows the console's stick and throttle: bank and pitch as the
//!   stick is held, speed by the throttle.
//! * Its battery runs down with the throttle; empty, it glides down and is lost. The enemy's
//!   air defences (guns, MANPADS teams, SAM sites) find a drone only within [`DETECT_RANGE`]:
//!   its radar cross section is tiny.
//!
//! The drone's own state is kept in metres and radians ([`Uav::pos`]) and written to its flight
//! unit each frame, which draws it, sends it to the clients and lets the guns aim at it.

use serde::{Deserialize, Serialize};

use crate::Sim;
use crate::aaa::{AaaKind, M_PER_MAP};
use crate::campaign::{CampaignFlight, M_PER_WU, Role};
use crate::station::Station;
use crate::types::*;

/// The enemy's air defences find a drone this close (m), and only then shoot at it.
pub const DETECT_RANGE: f64 = 2000.0;
/// Of the shells and missiles that would bring down an aircraft, this many per mille bring down
/// a drone (small, a little engine for the infrared seekers).
pub const SHELL_HIT: i32 = 15;
pub const MISSILE_HIT: i32 = 300;
/// Drones of a center in the air at once.
pub const MAX_AIRBORNE: usize = 5;
/// Waypoints a route may have.
pub const MAX_WAYPOINTS: usize = 12;
/// A surveillance drone sees the enemy's air defences this far (m, over the ground), an attack
/// drone's camera this far.
pub const STORK_SENSOR: f64 = 6000.0;
pub const FP2_SENSOR: f64 = 2000.0;
/// The launch rail (tools/models/air_defence.py): its rear end forward of the bunker's centre
/// and up (m), its length (m) and its rise (rad).
pub const RAMP_FORWARD: f64 = 7.0;
pub const RAMP_UP: f64 = 1.3;
pub const RAMP_LENGTH: f64 = 7.0;
pub const RAMP_RISE: f64 = 0.21;
/// Seconds between two launches off the rail.
pub const LAUNCH_INTERVAL: i32 = 15;
/// Seconds a drone climbs straight out after the rail.
const CLIMB_OUT: f32 = 4.0;
/// A nav point is passed this close (m).
const WP_REACHED: f64 = 150.0;
/// Metres over the mountains on the way.
const CLEARANCE: f64 = 120.0;
/// An FP-2's warhead goes off this close to its terminate point (m).
const FUSE: f64 = 10.0;
/// An FP-2's warhead destroys the air defence units this close (map units, about 60 m; as
/// `aaa_damage` does a bomb's).
const BLAST: f64 = 4.0;
/// A battery this low (fraction) glides down.
const EMPTY: f32 = 0.0;

/// The aircraft types (assets/sim/units.ron).
pub const FP2_SPEC: i32 = 33;
pub const STORK_SPEC: i32 = 34;

/// What a drone is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UavKind {
    /// Stork-100: slow, long endurance, the sensor.
    #[default]
    Stork,
    /// FP-2: fast, short endurance, a warhead.
    Fp2,
}

/// How a drone flies (m, m/s, rad, s).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Perf {
    pub cruise: f64,
    pub max: f64,
    pub stall: f64,
    pub dive: f64,
    /// Fastest climb and sink (m/s), steepest bank.
    pub climb: f64,
    pub bank: f64,
    /// Seconds of flight at cruise on a full battery.
    pub endurance: f64,
    /// The circle round a loiter point.
    pub loiter_radius: f64,
    /// The height a new waypoint gets (m over the ground).
    pub altitude: f64,
}

impl UavKind {
    pub fn perf(self) -> Perf {
        match self {
            UavKind::Stork => Perf {
                cruise: 25.0,
                max: 33.0,
                stall: 14.0,
                dive: 33.0,
                climb: 3.0,
                bank: 0.5,
                endurance: 2.5 * 3600.0,
                loiter_radius: 900.0,
                altitude: 800.0,
            },
            UavKind::Fp2 => Perf {
                cruise: 50.0,
                max: 60.0,
                stall: 24.0,
                dive: 85.0,
                climb: 5.0,
                bank: 0.6,
                endurance: 1.2 * 3600.0,
                loiter_radius: 600.0,
                altitude: 300.0,
            },
        }
    }

    /// "STORK", "FP-2".
    pub fn name(self) -> &'static str {
        match self {
            UavKind::Stork => "STORK",
            UavKind::Fp2 => "FP-2",
        }
    }

    pub fn spec(self) -> i32 {
        match self {
            UavKind::Stork => STORK_SPEC,
            UavKind::Fp2 => FP2_SPEC,
        }
    }

    /// The camera sees the enemy's air defences this far.
    pub fn sensor(self) -> f64 {
        match self {
            UavKind::Stork => STORK_SENSOR,
            UavKind::Fp2 => FP2_SENSOR,
        }
    }
}

/// A waypoint's kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WpKind {
    /// Fly through it.
    #[default]
    Nav,
    /// Circle it (the route ends there).
    Loiter,
    /// FP-2: dive into it (the route ends there).
    Terminate,
}

impl WpKind {
    pub fn name(self) -> &'static str {
        match self {
            WpKind::Nav => "NAV",
            WpKind::Loiter => "LOITER",
            WpKind::Terminate => "TERMINATE",
        }
    }

    /// The route ends here.
    pub fn last(self) -> bool {
        self != WpKind::Nav
    }
}

/// A waypoint: a map point (x east, y south), the height over the ground (m), what to do there.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UavWp {
    pub x: i32,
    pub y: i32,
    pub alt: f32,
    pub kind: WpKind,
}

impl UavWp {
    /// Metres east, north (the middle of its map unit, where a unit there stands).
    pub fn metres(&self) -> [f64; 2] {
        [(self.x as f64 + 0.5) * M_PER_MAP, (0x8000 as f64 - self.y as f64 - 0.5) * M_PER_MAP]
    }
}

/// How a drone is flown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UavMode {
    /// Off the rail and climbing out.
    #[default]
    Launch,
    Auto,
    Manual,
    /// FP-2 in its final dive at its terminate point.
    Dive,
}

impl UavMode {
    pub fn name(self) -> &'static str {
        match self {
            UavMode::Launch => "LAUNCH",
            UavMode::Auto => "AUTO",
            UavMode::Manual => "MANUAL",
            UavMode::Dive => "DIVE",
        }
    }
}

/// A drone in the air.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Uav {
    /// Its number at its center (shown as STK-n / FP2-n).
    pub id: u16,
    /// The drone command center (AAA unit) flying it.
    pub center: u16,
    pub kind: UavKind,
    /// Its flight unit.
    pub flight: u16,
    pub route: Vec<UavWp>,
    /// The waypoint it flies to.
    pub next: u16,
    pub mode: UavMode,
    /// Charge left (1 full).
    pub battery: f32,
    /// Metres east, north, up; heading (clockwise from north), pitch (up) and bank (right) in
    /// radians; speed (m/s).
    pub pos: [f64; 3],
    pub heading: f64,
    pub pitch: f64,
    pub bank: f64,
    pub speed: f64,
    /// Manual: the stick (roll right, pitch up, -1..1) and the throttle (0..1).
    pub ctrl: [f32; 3],
    /// Seconds left of the climb out after the rail.
    pub climb_out: f32,
}

impl Uav {
    /// "STK-2", "FP2-5".
    pub fn label(&self) -> String {
        match self.kind {
            UavKind::Stork => format!("STK-{}", self.id),
            UavKind::Fp2 => format!("FP2-{}", self.id),
        }
    }

    /// The waypoint it flies to.
    pub fn target(&self) -> Option<&UavWp> {
        self.route.get(self.next as usize)
    }

    /// Battery drain per second at the speed it flies (full battery: 1).
    pub fn drain(&self) -> f64 {
        let p = self.kind.perf();
        // Power grows with the cube of the speed over the cruise's, a third of it to spare.
        let r = (self.speed / p.cruise).max(0.5);
        (0.35 + 0.65 * r * r * r) / p.endurance
    }

    /// Seconds of flight left at this drain.
    pub fn endurance(&self) -> f64 {
        self.battery as f64 / self.drain().max(1e-9)
    }

    /// Map point (x east, y south).
    pub fn map(&self) -> (i32, i32) {
        ((self.pos[0] / M_PER_MAP) as i32, 0x8000 - (self.pos[1] / M_PER_MAP) as i32)
    }
}

/// Something the drones have found: an air defence unit, a SAM site (world object).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Seen {
    Aaa(u16),
    Object(u16),
}

/// What is wrong with a route (None: it may be flown).
pub fn route_refusal(kind: UavKind, route: &[UavWp]) -> Option<&'static str> {
    if route.is_empty() {
        return Some("No waypoints");
    }
    if route.len() > MAX_WAYPOINTS {
        return Some("Too many waypoints");
    }
    if route.iter().any(|w| !(0..0x8000).contains(&w.x) || !(0..0x8000).contains(&w.y)) {
        return Some("Waypoint off the map");
    }
    if kind == UavKind::Stork && route.iter().any(|w| w.kind == WpKind::Terminate) {
        return Some("Only attack drones terminate");
    }
    if route[..route.len() - 1].iter().any(|w| w.kind.last()) {
        return Some("The route ends at a loiter or terminate point");
    }
    None
}

/// A heading (rad, clockwise from north) as a 16-bit angle.
fn bam(a: f64) -> i32 {
    (a / std::f64::consts::TAU * 65536.0).round() as i32 & 0xFFFF
}

/// A signed 16-bit angle.
fn sbam(a: f64) -> i32 {
    (a / std::f64::consts::TAU * 65536.0).round().clamp(-32768.0, 32767.0) as i32
}

/// An angle wrapped into -pi..pi.
fn wrap(a: f64) -> f64 {
    use std::f64::consts::{PI, TAU};
    (a + PI).rem_euclid(TAU) - PI
}

impl Sim {
    /// The drones in the air (a campaign's).
    pub fn uavs(&self) -> &[Uav] {
        self.defence().map_or(&[], |d| &d.uavs)
    }

    /// What the drones have found (and is still there).
    pub fn uav_known(&self) -> &[Seen] {
        self.defence().map_or(&[], |d| &d.uav_known)
    }

    /// The drone flying as flight unit `i`.
    pub fn uav_of_flight(&self, i: usize) -> Option<&Uav> {
        self.uavs().iter().find(|u| u.flight as usize == i)
    }

    /// A drone center's launch rail: where a drone sits on it (m east, north, up) and the way
    /// it points (rad, clockwise from north).
    pub fn uav_ramp(&self, center: usize) -> ([f64; 3], f64) {
        let u = self.aaa[center];
        let h = crate::aaa::unit_heading(center) as f64;
        let p = u.metres();
        ([p[0] + h.sin() * RAMP_FORWARD, p[1] + h.cos() * RAMP_FORWARD, RAMP_UP], h)
    }

    /// Seconds until a center's rail is ready again.
    pub fn uav_rail_wait(&self, center: usize) -> i32 {
        self.aaa.get(center).map_or(0, |u| (u.cooldown + self.fs.max(1) - 1) / self.fs.max(1))
    }

    /// A drone center's drones in the air.
    pub fn uavs_of(&self, center: usize) -> impl Iterator<Item = &Uav> {
        self.uavs().iter().filter(move |u| u.center as usize == center)
    }

    /// A center's drones in stock: surveillance, attack.
    pub fn uav_stock(&self, center: usize) -> [i32; 2] {
        let Some(d) = self.defence() else { return [0; 2] };
        [d.stock.get(center).copied().unwrap_or(0), d.stock2.get(center).copied().unwrap_or(0)]
    }

    /// Why station `st` can't launch a `kind` drone on `route` now (None: it can).
    pub fn launch_refusal(&self, st: &Station, kind: UavKind, route: &[UavWp]) -> Option<String> {
        let k = st.unit as usize;
        if self.aaa.get(k).is_none_or(|u| u.kind != AaaKind::DroneCenter) {
            return Some("No drones here".into());
        }
        if self.uavs_of(k).count() >= MAX_AIRBORNE {
            return Some(format!("{MAX_AIRBORNE} drones are in the air already"));
        }
        let stock = self.uav_stock(k)[(kind == UavKind::Fp2) as usize];
        if stock <= 0 {
            return Some(format!("No {} left: request resupply", kind.name()));
        }
        if self.aaa[k].cooldown > 0 {
            return Some(format!("Rail ready in {} s", self.uav_rail_wait(k)));
        }
        route_refusal(kind, route).map(String::from)
    }

    /// The drone with id `id` of station `st`'s center (its index).
    fn station_uav(&self, st: &Station, id: u16) -> Result<usize, String> {
        self.uavs()
            .iter()
            .position(|u| u.center == st.unit && u.id == id)
            .ok_or_else(|| "No such drone".into())
    }

    /// A drone command of the player at station `st`.
    pub(crate) fn uav_command(
        &mut self,
        st: &Station,
        cmd: crate::station::StationCmd,
    ) -> Result<(), String> {
        use crate::station::StationCmd as C;
        match cmd {
            C::Launch { kind, route } => {
                if let Some(why) = self.launch_refusal(st, kind, &route) {
                    return Err(why);
                }
                self.launch_uav(st.unit as usize, kind, route);
            }
            C::Route { uav, route } => {
                let k = self.station_uav(st, uav)?;
                let kind = self.uavs()[k].kind;
                if let Some(why) = route_refusal(kind, &route) {
                    return Err(why.into());
                }
                let d = self.defence_mut();
                let u = &mut d.uavs[k];
                if u.mode == UavMode::Dive {
                    return Err("Diving".into());
                }
                u.next = u.next.min(route.len() as u16 - 1);
                u.route = route;
            }
            C::Next { uav, wp } => {
                let k = self.station_uav(st, uav)?;
                let u = &mut self.defence_mut().uavs[k];
                if wp as usize >= u.route.len() {
                    return Err("No such waypoint".into());
                }
                if u.mode == UavMode::Dive {
                    return Err("Diving".into());
                }
                u.next = wp;
            }
            C::Manual { uav, on } => {
                let k = self.station_uav(st, uav)?;
                let u = &mut self.defence_mut().uavs[k];
                match (u.mode, on) {
                    (UavMode::Dive | UavMode::Launch, _) => return Err("Not now".into()),
                    (_, true) => {
                        u.mode = UavMode::Manual;
                        let p = u.kind.perf();
                        let t = ((u.speed - p.stall) / (p.max - p.stall)).clamp(0.0, 1.0);
                        u.ctrl = [0.0, 0.0, t as f32];
                    }
                    (_, false) => u.mode = UavMode::Auto,
                }
            }
            C::Fly { uav, roll, pitch, throttle } => {
                let k = self.station_uav(st, uav)?;
                let u = &mut self.defence_mut().uavs[k];
                if u.mode != UavMode::Manual {
                    return Err("Not in manual".into());
                }
                u.ctrl = [roll.clamp(-1.0, 1.0), pitch.clamp(-1.0, 1.0), throttle.clamp(0.0, 1.0)];
            }
            _ => return Err("Not a drone command".into()),
        }
        Ok(())
    }

    /// A drone off center `k`'s rail.
    fn launch_uav(&mut self, k: usize, kind: UavKind, route: Vec<UavWp>) {
        let (pos, heading) = self.uav_ramp(k);
        let i = self.campaign_slot();
        self.flights[i] = FlightUnit { spec: kind.spec(), ..Default::default() };
        let fs = self.fs.max(1);
        self.aaa[k].cooldown = LAUNCH_INTERVAL * fs;
        let c = self.campaign.as_mut().expect("campaign");
        c.flights[i] = Some(CampaignFlight {
            tag: 0,
            role: Role::Uav,
            blue: true,
            goal: (self.aaa[k].x, self.aaa[k].y),
            target: None,
            cooldown: 0,
            loadout: Default::default(),
            task: Default::default(),
            home: 0,
            anchor: (self.aaa[k].x, self.aaa[k].y),
            quiet: 0,
            burst: 0,
            killed_by_ai: false,
            reported: false,
            killed_by: None,
            strike: None,
            kamikaze: false,
            supply: None,
        });
        let d = &mut c.defence;
        d.uav_serial += 1;
        let stock = if kind == UavKind::Fp2 { &mut d.stock2 } else { &mut d.stock };
        stock[k] -= 1;
        let p = kind.perf();
        let uav = Uav {
            id: d.uav_serial,
            center: k as u16,
            kind,
            flight: i as u16,
            route,
            next: 0,
            mode: UavMode::Launch,
            battery: 1.0,
            pos,
            heading,
            pitch: RAMP_RISE,
            bank: 0.0,
            speed: p.stall * 1.3,
            ctrl: [0.0; 3],
            climb_out: CLIMB_OUT,
        };
        d.uavs.push(uav);
        let n = d.uavs.len() - 1;
        self.write_uav_flight(n);
    }

    /// The drone's flight unit follows it.
    fn write_uav_flight(&mut self, n: usize) {
        let u = self.defence().expect("campaign").uavs[n].clone();
        let f = &mut self.flights[u.flight as usize];
        f.wx = (u.pos[0] / M_PER_WU) as i32;
        f.wy = 0x10_0000 - (u.pos[1] / M_PER_WU) as i32;
        f.x = (f.wx >> 5) & 0xFFFF;
        f.y = (f.wy >> 5) & 0xFFFF;
        f.alt = (u.pos[2] / M_PER_WU).max(1.0) as i32;
        f.heading = bam(u.heading);
        f.pitch = sbam(u.pitch);
        f.bank = sbam(u.bank);
        f.speed = ((u.speed / M_PER_WU) as i32).max(1);
        f.flags = (f.flags | fflags::EXISTS | fflags::AIRBORNE) & !fflags::TAKEOFF;
        f.wp = -1;
        f.timer = 0x7FFF;
    }

    /// The highest ground (m) under a point and ahead of it on a heading.
    fn uav_ground(&self, pos: [f64; 3], heading: f64) -> f64 {
        let Some(relief) = self.tiles.as_ref().map(|t| &t.relief) else { return 0.0 };
        let cell = 2000.0;
        (0..4)
            .map(|k| {
                let d = k as f64 * cell;
                let (e, n) = (pos[0] + heading.sin() * d, pos[1] + heading.cos() * d);
                relief.height((e / M_PER_MAP) as i32, 0x8000 - (n / M_PER_MAP) as i32) as f64
                    * M_PER_WU
            })
            .fold(0.0, f64::max)
    }

    /// One frame of flight unit `i`, a drone.
    pub(crate) fn fly_uav(&mut self, i: usize) {
        let Some(n) = self.uavs().iter().position(|u| u.flight as usize == i) else { return };
        let dt = 1.0 / self.fs.max(1) as f64;
        let mut u = self.defence().expect("campaign").uavs[n].clone();
        let p = u.kind.perf();
        let ground = self.uav_ground(u.pos, u.heading);
        // What it steers for: a heading, a height, a speed (or the stick's bank and pitch).
        let mut steer: Option<(f64, f64, f64)> = None;
        let mut dive_at: Option<[f64; 3]> = None;
        match u.mode {
            UavMode::Launch => {
                u.climb_out -= dt as f32;
                if u.climb_out <= 0.0 {
                    u.mode = UavMode::Auto;
                }
                let climb = p.climb.min(u.speed * RAMP_RISE.sin());
                steer = Some((u.heading, u.pos[2] + climb * 10.0, p.cruise));
            }
            UavMode::Auto => {
                steer = Some(self.uav_auto(&mut u, ground));
                if u.mode == UavMode::Dive {
                    dive_at = u.target().map(|w| {
                        let m = w.metres();
                        [m[0], m[1], 0.0]
                    });
                }
            }
            UavMode::Dive => {
                dive_at = u.target().map(|w| {
                    let m = w.metres();
                    [m[0], m[1], 0.0]
                });
            }
            UavMode::Manual => {}
        }
        if u.battery <= EMPTY {
            // Gliding down, wings level.
            u.bank -= u.bank * (2.0 * dt).min(1.0);
            u.pitch = (-0.08_f64).max(u.pitch - 0.2 * dt);
            u.speed += ((p.stall * 1.1) - u.speed) * (0.3 * dt).min(1.0);
        } else if let Some(t) = dive_at {
            // Straight at the point, speeding up.
            let d = [t[0] - u.pos[0], t[1] - u.pos[1], t[2] - u.pos[2]];
            let want_h = d[0].atan2(d[1]);
            let want_p = d[2].atan2(d[0].hypot(d[1]));
            let turn = wrap(want_h - u.heading);
            u.heading = wrap(u.heading + turn.clamp(-1.0 * dt, 1.0 * dt));
            u.bank = (turn * 2.0).clamp(-p.bank, p.bank);
            u.pitch += (want_p - u.pitch).clamp(-0.8 * dt, 0.8 * dt);
            u.speed += (p.dive - u.speed).min(15.0 * dt);
        } else if let Some((want_h, want_alt, want_v)) = steer {
            let turn = wrap(want_h - u.heading);
            let bank = (turn * 1.5).clamp(-p.bank, p.bank);
            u.bank += (bank - u.bank).clamp(-0.6 * dt, 0.6 * dt);
            let climb = ((want_alt - u.pos[2]) * 0.15).clamp(-p.climb, p.climb);
            let pitch = (climb / u.speed.max(1.0)).clamp(-0.5, 0.5).asin();
            u.pitch += (pitch - u.pitch).clamp(-0.3 * dt, 0.3 * dt);
            u.speed += (want_v - u.speed).clamp(-3.0 * dt, 3.0 * dt);
        } else {
            // The stick sets bank and pitch, the throttle the speed.
            let [roll, pitch, throttle] = u.ctrl.map(|c| c as f64);
            u.bank += (roll * p.bank * 1.2 - u.bank).clamp(-1.0 * dt, 1.0 * dt);
            u.pitch += (pitch * 0.45 - u.pitch).clamp(-0.6 * dt, 0.6 * dt);
            let want_v = p.stall * 1.1 + (p.max - p.stall * 1.1) * throttle;
            // Nose down picks up speed, nose up loses it.
            let v = want_v - u.pitch.sin() * 40.0;
            u.speed += (v.clamp(p.stall, p.dive) - u.speed).clamp(-4.0 * dt, 4.0 * dt);
        }
        // A coordinated turn: the bank turns the heading.
        u.heading = wrap(u.heading + 9.81 * u.bank.tan() / u.speed.max(1.0) * dt);
        let h = u.speed * u.pitch.cos() * dt;
        u.pos[0] += u.heading.sin() * h;
        u.pos[1] += u.heading.cos() * h;
        u.pos[2] += u.speed * u.pitch.sin() * dt;
        let on_target = dive_at.is_some_and(|t| {
            let d = [t[0] - u.pos[0], t[1] - u.pos[1], t[2] - u.pos[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() < FUSE
        });
        let crashed = u.pos[2] <= 0.0 || u.pos[2] < ground - 50.0;
        self.defence_mut().uavs[n] = u;
        if on_target || crashed {
            self.uav_impact(n);
        } else {
            self.write_uav_flight(n);
        }
    }

    /// Auto: the heading, height and speed to its waypoint (and on to the next one past it);
    /// an FP-2 near its terminate point starts its dive.
    fn uav_auto(&self, u: &mut Uav, ground: f64) -> (f64, f64, f64) {
        let p = u.kind.perf();
        let floor = ground + CLEARANCE;
        let Some(w) = u.target().copied() else {
            return (u.heading, u.pos[2].max(floor), p.cruise);
        };
        let m = w.metres();
        let (de, dn) = (m[0] - u.pos[0], m[1] - u.pos[1]);
        let d = de.hypot(dn);
        let brg = de.atan2(dn);
        let alt = (w.alt as f64).max(floor);
        let last = u.next as usize + 1 >= u.route.len();
        match w.kind {
            WpKind::Terminate => {
                // A dive of about 30 degrees from where it is.
                if d < (u.pos[2] * 1.8).max(250.0) {
                    u.mode = UavMode::Dive;
                }
                (brg, alt, p.cruise)
            }
            WpKind::Nav if !last => {
                if d < WP_REACHED {
                    u.next += 1;
                }
                (brg, alt, p.cruise)
            }
            // Round the point clockwise: towards it from outside the circle, out from inside.
            _ => {
                let r = p.loiter_radius;
                if d > 2.0 * r {
                    return (brg, alt, p.cruise);
                }
                let off = ((d - r) / r).clamp(-1.0, 1.0) * 1.0;
                (wrap(brg - std::f64::consts::FRAC_PI_2 + off), alt, p.cruise * 0.9)
            }
        }
    }

    /// Drone `n` hit the ground: an FP-2 blows up (the guns and SAM sites there with it), a
    /// Stork breaks up. Either way it is gone.
    fn uav_impact(&mut self, n: usize) {
        let u = self.defence_mut().uavs.remove(n);
        let i = u.flight as usize;
        let (x, y) = u.map();
        if let Some(Some(f)) = self.campaign.as_mut().and_then(|c| c.flights.get_mut(i)) {
            f.reported = true;
        }
        self.flights[i].flags = 0;
        if u.kind == UavKind::Fp2 {
            self.emit(SimEvent::Explosion { x, y, alt: 0, kind: -6 });
            // The guns and launchers there (also with nobody flying, when `aaa_damage` sleeps).
            for k in 0..self.aaa.len() {
                let a = self.aaa[k];
                if !a.destroyed && ((a.x - x) as f64).hypot((a.y - y) as f64) < BLAST {
                    self.aaa[k].destroyed = true;
                    self.aaa[k].burst = 0;
                    self.msg(format!("{} destroyed", a.kind.name_on(self.aaa_blue(k))));
                    self.emit(SimEvent::Explosion { x: a.x, y: a.y, alt: 0, kind: 0x10 });
                    self.emit(SimEvent::AaaDestroyed { unit: k, by: None });
                }
            }
            let o = self.find_object_at(x, y);
            if o > 2 {
                let ob = self.obj(o);
                let close = crate::math::approx(ob.x - x, ob.y - y) < 4;
                if ob.flags & oflags::DESTROYED == 0 && ob.flags & oflags::FRIENDLY == 0 && close {
                    self.destroy_ground(o);
                }
            }
            self.msg(format!("{} impact", u.label()));
        } else {
            self.emit(SimEvent::Explosion { x, y, alt: 0, kind: -1 });
            self.msg(format!("{} lost", u.label()));
        }
    }

    /// Once a frame: drones shot down or whose center is gone are lost, the batteries run down,
    /// and once a second the drones look for the enemy's air defences.
    pub(crate) fn update_uavs(&mut self) {
        let Some(d) = self.defence() else { return };
        if d.uavs.is_empty() && d.uav_known.is_empty() {
            return;
        }
        let dt = 1.0 / self.fs.max(1) as f64;
        let mut n = 0;
        while n < self.uavs().len() {
            let u = &self.uavs()[n];
            let f = self.flights.get(u.flight as usize).copied().unwrap_or_default();
            let ours = self.campaign_flight(u.flight as usize).is_some_and(|c| c.role == Role::Uav);
            if f.flags & fflags::EXISTS == 0 || f.flags & fflags::DESTROYED != 0 || !ours {
                let label = u.label();
                self.defence_mut().uavs.remove(n);
                self.msg(format!("{label} shot down"));
                continue;
            }
            let drain = u.drain() * dt;
            let u = &mut self.defence_mut().uavs[n];
            u.battery = (u.battery as f64 - drain).max(0.0) as f32;
            n += 1;
        }
        if self.frame % self.fs.max(1) == 0 {
            self.uav_look();
        }
    }

    /// The drones' sensors: what they see now joins what they have found; what is destroyed
    /// is forgotten.
    fn uav_look(&mut self) {
        let mut known = self.defence().map(|d| d.uav_known.clone()).unwrap_or_default();
        known.retain(|s| match *s {
            Seen::Aaa(k) => self.aaa.get(k as usize).is_some_and(|u| !u.destroyed),
            Seen::Object(o) => self
                .objects
                .get(o as usize)
                .is_some_and(|o| o.flags & oflags::DESTROYED == 0 && o.unit_type != 0),
        });
        for u in self.uavs() {
            let r = u.kind.sensor();
            let near = |x: f64, y: f64| (x - u.pos[0]).hypot(y - u.pos[1]) < r;
            for (k, a) in self.aaa.iter().enumerate() {
                let m = a.metres();
                if !a.destroyed && !self.aaa_blue(k) && near(m[0], m[1]) {
                    let s = Seen::Aaa(k as u16);
                    if !known.contains(&s) {
                        known.push(s);
                    }
                }
            }
            for (i, o) in self.objects.iter().enumerate().take(self.ground_units) {
                let hostile =
                    o.flags & (oflags::DESTROYED | oflags::FRIENDLY | oflags::NEUTRAL) == 0;
                let (e, n) = (o.x as f64 * M_PER_MAP, (0x8000 - o.y) as f64 * M_PER_MAP);
                if o.unit_type != 0 && hostile && near(e, n) {
                    let s = Seen::Object(i as u16);
                    if !known.contains(&s) {
                        known.push(s);
                    }
                }
            }
        }
        if let Some(c) = self.campaign.as_mut() {
            c.defence.uav_known = known;
        }
    }

    /// Test aid: drone `n`'s battery.
    pub fn set_uav_battery(&mut self, n: usize, charge: f32) {
        if let Some(u) = self.campaign.as_mut().and_then(|c| c.defence.uavs.get_mut(n)) {
            u.battery = charge;
        }
    }

    /// Where something the drones found is (map units).
    pub fn seen_at(&self, s: Seen) -> Option<(i32, i32)> {
        match s {
            Seen::Aaa(k) => self.aaa.get(k as usize).map(|u| (u.x, u.y)),
            Seen::Object(o) => self.objects.get(o as usize).map(|o| (o.x, o.y)),
        }
    }
}
