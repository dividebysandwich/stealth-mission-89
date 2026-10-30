//! Campaign: the drones of a drone command center, flown from its console. A center launches
//! up to [`MAX_AIRBORNE`] at a time off its rail: **Stork-100** surveillance drones, whose
//! sensor finds every enemy air defence unit and SAM site within [`STORK_SENSOR`] of them,
//! **FP-2** attack drones, which dive into a ground point and blow up there, and
//! **interceptors**, small quadrotors as fast as 600 km/h with a battery for minutes, which
//! lock on to an aircraft or drone in front of them within [`LOCK_RANGE`] (whoever's: they
//! can't tell friend from foe) and fly it down while the pilot opens the throttle. Into the
//! ground, an interceptor's warhead goes off too, wrecking whatever stands there, friendly or
//! not.
//!
//! * A drone flies its route by itself (**auto**): through its nav points, round a **loiter**
//!   point on a wide circle, or into a **terminate** point (FP-2 only) in a steep dive. A route
//!   without either ends in a circle round its last point.
//! * Taken over (**manual**), it follows the console's stick and throttle: the stick rolls and
//!   pitches it (released, it keeps its attitude), the throttle sets its speed. An interceptor
//!   turns its nose almost at once; its flight path swings round after it with at most
//!   [`TURN_G`].
//! * Its battery runs down with the throttle; empty, it glides down (an interceptor, without
//!   wings, drops) and is lost. An interceptor under [`Perf::hover`] throttle can't hold its
//!   height. The enemy's
//!   air defences (guns, MANPADS teams, SAM sites) find a drone only within [`DETECT_RANGE`]:
//!   its radar cross section is tiny.
//!
//! The drone's own state is kept in metres and radians ([`Uav::pos`]) and written to its flight
//! unit each frame, which draws it, sends it to the clients and lets the guns aim at it.

use serde::{Deserialize, Serialize};

use crate::Sim;
use crate::aaa::{GroundKind, M_PER_MAP};
use crate::campaign::{CampaignFlight, M_PER_WU, Role, add, dot, flight_metres, len, mul, sub};
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
/// `aaa_damage` does a bomb's), an interceptor's smaller one about 30 m.
const BLAST: f64 = 4.0;
const QUAD_BLAST: f64 = 2.0;
/// A battery this low (fraction) glides down.
const EMPTY: f32 = 0.0;
/// An interceptor locks on to an enemy aircraft this close (m) and this far off its nose (rad,
/// its camera's view), and lets go of it further than `LOCK_BREAK` (m).
pub const LOCK_RANGE: f64 = 500.0;
pub const LOCK_CONE: f64 = 0.35;
const LOCK_BREAK: f64 = 1500.0;
/// An interceptor's flight path follows its nose this quickly (1/s), with this many g at most;
/// with too little throttle it sinks up to `SINK` m/s.
pub const TURN_G: f64 = 20.0;
const RESPONSE: f64 = 2.5;
const SINK: f64 = 30.0;
/// The stick's roll and pitch rates and the rudder's yaw rate at full deflection (rad/s): a
/// winged drone's, an interceptor's (which also turns its nose this fast at a locked target).
const ROLL_RATE: f64 = 0.8;
const PITCH_RATE: f64 = 0.4;
const YAW_RATE: f64 = 0.15;
const QUAD_ROLL_RATE: f64 = 4.0;
const QUAD_PITCH_RATE: f64 = 3.0;
const QUAD_YAW_RATE: f64 = 3.0;
/// The steepest climb or dive an interceptor's nose holds (rad).
const QUAD_PITCH: f64 = 1.45;
/// An interceptor's warhead goes off this close to its target (m).
const INTERCEPT_FUSE: f64 = 6.0;
/// An interceptor with a flat battery falls at this speed (m/s).
const FALL: f64 = 45.0;

/// The aircraft types (assets/sim/units.ron).
pub const FP2_SPEC: i32 = 33;
pub const STORK_SPEC: i32 = 34;
pub const INTERCEPTOR_SPEC: i32 = 35;

/// What a drone is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UavKind {
    /// Stork-100: slow, long endurance, the sensor.
    #[default]
    Stork,
    /// FP-2: fast, short endurance, a warhead.
    Fp2,
    /// A quadrotor interceptor: very fast, minutes of battery, a warhead against aircraft.
    Interceptor,
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
    /// Seconds of flight at cruise on a full battery (an interceptor: at `hover` throttle).
    pub endurance: f64,
    /// The circle round a loiter point.
    pub loiter_radius: f64,
    /// The height a new waypoint gets (m over the ground).
    pub altitude: f64,
    /// How quickly it banks, pitches and speeds up (1: a winged drone); the steepest pitch
    /// flown by hand (rad).
    pub agility: f64,
    pub pitch: f64,
    /// An interceptor: the throttle that just holds it up (it sinks under it), its battery
    /// drains with the throttle. 0: a winged drone, its battery drains with its speed.
    pub hover: f64,
}

impl Perf {
    /// The speed a throttle (0..1) flies.
    pub fn throttle_speed(&self, t: f64) -> f64 {
        self.stall * 1.1 + (self.max - self.stall * 1.1) * t
    }

    /// The throttle that flies a speed.
    pub fn speed_throttle(&self, v: f64) -> f64 {
        ((v - self.stall * 1.1) / (self.max - self.stall * 1.1)).clamp(0.0, 1.0)
    }
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
                agility: 1.0,
                pitch: 0.45,
                hover: 0.0,
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
                agility: 1.0,
                pitch: 0.45,
                hover: 0.0,
            },
            // Its cruise is the speed of its hover throttle (`Perf::throttle_speed`).
            UavKind::Interceptor => Perf {
                cruise: 55.3,
                max: 600.0 / 3.6,
                stall: 25.0,
                dive: 170.0,
                climb: 25.0,
                bank: 1.2,
                endurance: 8.0 * 60.0,
                loiter_radius: 400.0,
                altitude: 500.0,
                agility: 4.0,
                pitch: 1.0,
                hover: 0.2,
            },
        }
    }

    /// "STORK", "FP-2".
    pub fn name(self) -> &'static str {
        match self {
            UavKind::Stork => "STORK",
            UavKind::Fp2 => "FP-2",
            UavKind::Interceptor => "INTERCEPTOR",
        }
    }

    /// "STORK", "FP-2", "INT" (where space is short).
    pub fn short(self) -> &'static str {
        match self {
            UavKind::Interceptor => "INT",
            k => k.name(),
        }
    }

    pub fn spec(self) -> i32 {
        match self {
            UavKind::Stork => STORK_SPEC,
            UavKind::Fp2 => FP2_SPEC,
            UavKind::Interceptor => INTERCEPTOR_SPEC,
        }
    }

    /// Its stockpile at a center (`DefenceState::stock`, `stock2`, `stock3`).
    pub fn stock_index(self) -> usize {
        match self {
            UavKind::Stork => 0,
            UavKind::Fp2 => 1,
            UavKind::Interceptor => 2,
        }
    }

    /// It has a TGP (an interceptor only looks ahead).
    pub fn tgp(self) -> bool {
        self != UavKind::Interceptor
    }

    /// The camera sees the enemy's air defences this far (an interceptor's never looks for
    /// them).
    pub fn sensor(self) -> f64 {
        match self {
            UavKind::Stork => STORK_SENSOR,
            UavKind::Fp2 => FP2_SENSOR,
            UavKind::Interceptor => 0.0,
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
    /// Manual: the rudder (nose right, -1..1).
    #[serde(default)]
    pub yaw: f32,
    /// Seconds left of the climb out after the rail.
    pub climb_out: f32,
    /// An interceptor: the enemy aircraft (flight unit) it is locked on to.
    #[serde(default)]
    pub lock: Option<u16>,
    /// An interceptor: its velocity (m/s: east, north, up), which follows its nose (`heading`,
    /// `pitch` and `bank` are its attitude).
    #[serde(default)]
    pub vel: [f64; 3],
}

impl Uav {
    /// "STK-2", "FP2-5".
    pub fn label(&self) -> String {
        match self.kind {
            UavKind::Stork => format!("STK-{}", self.id),
            UavKind::Fp2 => format!("FP2-{}", self.id),
            UavKind::Interceptor => format!("INT-{}", self.id),
        }
    }

    /// The waypoint it flies to.
    pub fn target(&self) -> Option<&UavWp> {
        self.route.get(self.next as usize)
    }

    /// Battery drain per second at the speed it flies (full battery: 1).
    pub fn drain(&self) -> f64 {
        let p = self.kind.perf();
        if p.hover > 0.0 {
            // The motors' power follows the throttle (the hover throttle's lasts `endurance`).
            return self.throttle().max(0.05) / p.hover / p.endurance;
        }
        // Power grows with the cube of the speed over the cruise's, a third of it to spare.
        let r = (self.speed / p.cruise).max(0.5);
        (0.35 + 0.65 * r * r * r) / p.endurance
    }

    /// The throttle it flies at (0..1): the stick's flown by hand, else the one for its speed
    /// (an interceptor's at least its hover throttle).
    pub fn throttle(&self) -> f64 {
        let p = self.kind.perf();
        match self.mode {
            UavMode::Manual => self.ctrl[2] as f64,
            _ => p.speed_throttle(self.speed).max(p.hover),
        }
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
    if kind != UavKind::Fp2 && route.iter().any(|w| w.kind == WpKind::Terminate) {
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

/// Where an interceptor at `pos` flying `speed` meets a target at `tp` moving `tv` (m, m/s);
/// the target itself when it can't catch it.
fn intercept(pos: [f64; 3], speed: f64, tp: [f64; 3], tv: [f64; 3]) -> [f64; 3] {
    let r = sub(tp, pos);
    // |r + tv t| = speed t
    let (a, b, c) = (dot(tv, tv) - speed * speed, 2.0 * dot(r, tv), dot(r, r));
    let t = if a.abs() < 1e-6 {
        (b < 0.0).then(|| -c / b)
    } else {
        let disc = b * b - 4.0 * a * c;
        (disc >= 0.0)
            .then(|| {
                let q = disc.sqrt();
                [(-b - q) / (2.0 * a), (-b + q) / (2.0 * a)]
                    .into_iter()
                    .filter(|t| *t > 0.0)
                    .fold(f64::INFINITY, f64::min)
            })
            .filter(|t| t.is_finite())
    };
    match t {
        Some(t) => add(tp, mul(tv, t.min(20.0))),
        None => tp,
    }
}

/// A drone's nose, right wing and top (unit vectors: east, north, up) from its heading, pitch
/// and bank.
fn axes(u: &Uav) -> ([f64; 3], [f64; 3], [f64; 3]) {
    let (sh, ch, sp, cp) = (u.heading.sin(), u.heading.cos(), u.pitch.sin(), u.pitch.cos());
    let f = [sh * cp, ch * cp, sp];
    let (r0, u0) = ([ch, -sh, 0.0], [-sh * sp, -ch * sp, cp]);
    let (sb, cb) = (u.bank.sin(), u.bank.cos());
    (f, sub(mul(r0, cb), mul(u0, sb)), add(mul(u0, cb), mul(r0, sb)))
}

/// A drone's heading, pitch and bank from its nose and top (its pitch at most `QUAD_PITCH`).
fn set_attitude(u: &mut Uav, f: [f64; 3], up: [f64; 3]) {
    u.heading = f[0].atan2(f[1]);
    u.pitch = f[2].clamp(-1.0, 1.0).asin().clamp(-QUAD_PITCH, QUAD_PITCH);
    let (sh, ch, sp, cp) = (u.heading.sin(), u.heading.cos(), u.pitch.sin(), u.pitch.cos());
    u.bank = dot(up, [ch, -sh, 0.0]).atan2(dot(up, [-sh * sp, -ch * sp, cp]));
}

/// An interceptor's stick and rudder: rolled about its nose, pitched about its wings and yawed
/// about its top at these rates (rad/s), the nose drifting round with the bank as the X wings
/// lift.
fn body_rates(u: &mut Uav, roll: f64, pitch: f64, yaw: f64, dt: f64) {
    let (f, _, up) = axes(u);
    let (sa, ca) = ((pitch * dt).sin(), (pitch * dt).cos());
    let (f, up) = (add(mul(f, ca), mul(up, sa)), sub(mul(up, ca), mul(f, sa)));
    // The right wing: nose cross top.
    let right = |f: [f64; 3]| {
        [f[1] * up[2] - f[2] * up[1], f[2] * up[0] - f[0] * up[2], f[0] * up[1] - f[1] * up[0]]
    };
    let (sy, cy) = ((yaw * dt).sin(), (yaw * dt).cos());
    let f = add(mul(f, cy), mul(right(f), sy));
    let r = right(f);
    let (sb, cb) = ((roll * dt).sin(), (roll * dt).cos());
    set_attitude(u, f, add(mul(up, cb), mul(r, sb)));
    u.heading = wrap(u.heading + 9.81 * u.bank.sin() / u.speed.max(20.0) * dt);
}

/// An interceptor turns its nose towards direction `want` (m: east, north, up) by at most
/// `max` (rad), banked into the turn.
fn steer_along(u: &mut Uav, want: [f64; 3], max: f64, dt: f64) {
    let l = len(want);
    if l < 1e-6 {
        return;
    }
    let w = want.map(|c| c / l);
    let d = [u.heading.sin() * u.pitch.cos(), u.heading.cos() * u.pitch.cos(), u.pitch.sin()];
    let cos = dot(d, w).clamp(-1.0, 1.0);
    let angle = cos.acos();
    let new = if angle <= max {
        w
    } else {
        // Rotated `max` from d towards w.
        let perp = sub(w, mul(d, cos));
        let pl = len(perp).max(1e-9);
        let (c, s) = (max.cos(), max.sin());
        [0, 1, 2].map(|k| d[k] * c + perp[k] / pl * s)
    };
    let heading = new[0].atan2(new[1]);
    let turn = wrap(heading - u.heading) / dt;
    let bank = (turn * u.speed / 9.81).atan().clamp(-1.4, 1.4);
    u.bank += (bank - u.bank).clamp(-4.0 * dt, 4.0 * dt);
    u.heading = heading;
    u.pitch = new[2].clamp(-1.0, 1.0).asin();
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
        let u = self.units[center];
        let h = crate::aaa::unit_heading(center) as f64;
        let p = u.metres();
        ([p[0] + h.sin() * RAMP_FORWARD, p[1] + h.cos() * RAMP_FORWARD, RAMP_UP], h)
    }

    /// Seconds until a center's rail is ready again.
    pub fn uav_rail_wait(&self, center: usize) -> i32 {
        self.units.get(center).map_or(0, |u| (u.cooldown + self.fs.max(1) - 1) / self.fs.max(1))
    }

    /// A drone center's drones in the air.
    pub fn uavs_of(&self, center: usize) -> impl Iterator<Item = &Uav> {
        self.uavs().iter().filter(move |u| u.center as usize == center)
    }

    /// A center's drones in stock: surveillance, attack, interceptors.
    pub fn uav_stock(&self, center: usize) -> [i32; 3] {
        let Some(d) = self.defence() else { return [0; 3] };
        [&d.stock, &d.stock2, &d.stock3].map(|s| s.get(center).copied().unwrap_or(0))
    }

    /// Why station `st` can't launch a `kind` drone on `route` now (None: it can).
    pub fn launch_refusal(&self, st: &Station, kind: UavKind, route: &[UavWp]) -> Option<String> {
        let k = st.unit as usize;
        if self.units.get(k).is_none_or(|u| u.kind != GroundKind::DroneCenter) {
            return Some("No drones here".into());
        }
        if self.uavs_of(k).count() >= MAX_AIRBORNE {
            return Some(format!("{MAX_AIRBORNE} drones are in the air already"));
        }
        let stock = self.uav_stock(k)[kind.stock_index()];
        if stock <= 0 {
            return Some(format!("No {} left: request resupply", kind.name()));
        }
        if self.units[k].cooldown > 0 {
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
                        let t = u.kind.perf().speed_throttle(u.speed);
                        u.ctrl = [0.0, 0.0, t as f32];
                        u.yaw = 0.0;
                    }
                    (_, false) => u.mode = UavMode::Auto,
                }
            }
            C::Fly { uav, roll, pitch, yaw, throttle } => {
                let k = self.station_uav(st, uav)?;
                let u = &mut self.defence_mut().uavs[k];
                if u.mode != UavMode::Manual {
                    return Err("Not in manual".into());
                }
                u.ctrl = [roll.clamp(-1.0, 1.0), pitch.clamp(-1.0, 1.0), throttle.clamp(0.0, 1.0)];
                u.yaw = yaw.clamp(-1.0, 1.0);
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
        self.units[k].cooldown = LAUNCH_INTERVAL * fs;
        let c = self.campaign.as_mut().expect("campaign");
        c.flights[i] = Some(CampaignFlight {
            tag: 0,
            role: Role::Uav,
            blue: true,
            goal: (self.units[k].x, self.units[k].y),
            target: None,
            cooldown: 0,
            loadout: Default::default(),
            task: Default::default(),
            home: 0,
            anchor: (self.units[k].x, self.units[k].y),
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
        let stock = match kind {
            UavKind::Stork => &mut d.stock,
            UavKind::Fp2 => &mut d.stock2,
            UavKind::Interceptor => &mut d.stock3,
        };
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
            yaw: 0.0,
            climb_out: CLIMB_OUT,
            lock: None,
            vel: [0.0; 3],
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
        let ag = p.agility;
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
        // An interceptor locked on: its target's position and velocity (m, m/s).
        let chase = u
            .lock
            .filter(|_| u.battery > EMPTY && u.mode != UavMode::Launch)
            .map(|j| self.air_target(j as usize));
        let from = u.pos;
        let quad = p.hover > 0.0;
        if quad && len(u.vel) < 1e-6 {
            u.vel = mul(axes(&u).0, u.speed);
        }
        // Flown along a direction (heading and pitch set directly), not by its bank.
        let mut direct = false;
        // An interceptor's speed along its nose and its throttle (None: its motors stopped).
        let mut thrust: Option<(f64, f64)> = None;
        if u.battery <= EMPTY && quad {
            // No wings: it tumbles and drops.
            u.bank = wrap(u.bank + 1.5 * dt);
            u.pitch += (-1.3 - u.pitch).clamp(-dt, dt);
            direct = true;
        } else if u.battery <= EMPTY {
            // Gliding down, wings level.
            u.bank -= u.bank * (2.0 * dt).min(1.0);
            u.pitch = (-0.08_f64).max(u.pitch - 0.2 * dt);
            u.speed += ((p.stall * 1.1) - u.speed) * (0.3 * dt).min(1.0);
        } else if let Some((tp, tv)) = chase {
            // The nose at where the target will be, leading the flight path round (past it by
            // as much as the path still has to swing).
            let aim = intercept(u.pos, u.speed, tp, tv);
            let to = sub(aim, u.pos);
            let w = mul(to, 1.0 / len(to).max(1e-6));
            let want = sub(mul(w, 2.0 * u.speed.max(20.0)), u.vel);
            steer_along(&mut u, want, QUAD_ROLL_RATE * dt, dt);
            direct = true;
            // At the throttle's speed (auto: its cruise, holding itself up).
            let t = match u.mode {
                UavMode::Manual => u.ctrl[2] as f64,
                _ => p.hover,
            };
            thrust = Some((p.throttle_speed(t), t));
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
            u.bank += (bank - u.bank).clamp(-0.6 * ag * dt, 0.6 * ag * dt);
            let climb = ((want_alt - u.pos[2]) * 0.15).clamp(-p.climb, p.climb);
            let pitch = (climb / u.speed.max(1.0)).clamp(-0.5, 0.5).asin();
            u.pitch += (pitch - u.pitch).clamp(-0.3 * ag * dt, 0.3 * ag * dt);
            if quad {
                thrust = Some((want_v, p.speed_throttle(want_v).max(p.hover)));
            } else {
                u.speed += (want_v - u.speed).clamp(-3.0 * dt, 3.0 * dt);
            }
        } else if quad {
            // The stick rolls and pitches it and the rudder yaws it (held where they are let
            // go), the throttle drives it along its nose.
            let [roll, pitch, throttle] = u.ctrl.map(|c| c as f64);
            let yaw = u.yaw as f64 * QUAD_YAW_RATE;
            body_rates(&mut u, roll * QUAD_ROLL_RATE, pitch * QUAD_PITCH_RATE, yaw, dt);
            direct = true;
            thrust = Some((p.throttle_speed(throttle), throttle));
        } else {
            // The stick rolls and pitches it (held where it is let go), the rudder skids the
            // nose round, the throttle sets the speed.
            let [roll, pitch, throttle] = u.ctrl.map(|c| c as f64);
            u.heading = wrap(u.heading + u.yaw as f64 * YAW_RATE * dt);
            let max_bank = p.bank * 1.2;
            u.bank = (u.bank + roll * ROLL_RATE * dt).clamp(-max_bank, max_bank);
            u.pitch = (u.pitch + pitch * PITCH_RATE * dt).clamp(-p.pitch, p.pitch);
            let want_v = p.throttle_speed(throttle);
            // Nose down picks up speed, nose up loses it.
            let v = want_v - u.pitch.sin() * 40.0;
            u.speed += (v.clamp(p.stall, p.dive) - u.speed).clamp(-4.0 * dt, 4.0 * dt);
        }
        if !direct {
            // A coordinated turn: the bank turns the heading.
            u.heading = wrap(u.heading + 9.81 * u.bank.tan() / u.speed.max(1.0) * dt);
        }
        if quad {
            // Its flight path follows the nose (with too little throttle, sinking), as hard as
            // `TURN_G` lets it; with the motors stopped it falls.
            let acc = match thrust {
                Some((v, t)) => {
                    let mut want = mul(axes(&u).0, v);
                    if t < p.hover {
                        want[2] = want[2].min(-SINK * (p.hover - t) / p.hover);
                    }
                    let a = mul(sub(want, u.vel), RESPONSE);
                    let max = TURN_G * 9.81;
                    mul(a, (max / len(a).max(1e-9)).min(1.0))
                }
                None => sub([0.0, 0.0, -9.81], mul(u.vel, len(u.vel) * 9.81 / (FALL * FALL))),
            };
            u.vel = add(u.vel, mul(acc, dt));
            u.pos = add(u.pos, mul(u.vel, dt));
            u.speed = len(u.vel);
        } else {
            let h = u.speed * u.pitch.cos() * dt;
            u.pos[0] += u.heading.sin() * h;
            u.pos[1] += u.heading.cos() * h;
            u.pos[2] += u.speed * u.pitch.sin() * dt;
        }
        let on_target = dive_at.is_some_and(|t| {
            let d = [t[0] - u.pos[0], t[1] - u.pos[1], t[2] - u.pos[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() < FUSE
        });
        // The interceptor's warhead: where its way and the target's pass closest this frame.
        let hit = chase.and_then(|(tp, tv)| {
            let step = sub(sub(u.pos, from), mul(tv, dt));
            (crate::aaa::closest(sub(from, tp), step) < INTERCEPT_FUSE).then_some(u.lock?)
        });
        let crashed = u.pos[2] <= 0.0 || u.pos[2] < ground - 50.0;
        self.defence_mut().uavs[n] = u;
        if let Some(j) = hit {
            self.uav_intercept(n, j as usize);
        } else if on_target || crashed {
            self.uav_impact(n);
        } else {
            self.write_uav_flight(n);
        }
    }

    /// Flight unit `j` as an interceptor's target: where it is and how it moves (m, m/s: east,
    /// north, up).
    fn air_target(&self, j: usize) -> ([f64; 3], [f64; 3]) {
        let v = self.flight_velocity(j);
        (flight_metres(&self.flights[j]), [v[0] * M_PER_WU, -v[1] * M_PER_WU, v[2] * M_PER_WU])
    }

    /// The enemy aircraft and drones in the air (flight units).
    pub fn enemy_aircraft(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.flights.len()).filter(|&j| self.ai_side(j) == Some(false))
    }

    /// What an interceptor locks on to: the aircraft and drones in the air of either side (not
    /// the command centers' own drones).
    pub fn interceptor_targets(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.flights.len())
            .filter(|&j| self.ai_side(j).is_some() && self.uav_of_flight(j).is_none())
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

    /// Drone `n` hit the ground: an FP-2 blows up (the enemy's guns and SAM sites there with
    /// it), an interceptor too (whatever stands there, friendly or not), a Stork breaks up.
    /// Either way it is gone.
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
            self.uav_blast_units(x, y, BLAST);
            let o = self.find_object_at(x, y);
            if o > 2 {
                let ob = self.obj(o);
                let close = crate::math::approx(ob.x - x, ob.y - y) < 4;
                if ob.flags & oflags::DESTROYED == 0 && ob.flags & oflags::FRIENDLY == 0 && close {
                    self.destroy_ground(o);
                }
            }
            self.msg(format!("{} impact", u.label()));
        } else if u.kind == UavKind::Interceptor {
            self.emit(SimEvent::Explosion { x, y, alt: 0, kind: -6 });
            self.uav_blast_units(x, y, QUAD_BLAST);
            // The building, vehicle or site there (the nearest tile object, as a bomb's).
            let o = self.find_object_at(x, y);
            if o != -1 && self.obj(o).flags & oflags::DESTROYED == 0 {
                let t = self.pl.last_tile;
                let (tx, ty) = t.map_or((self.obj(o).x, self.obj(o).y), |t| t.map_pos());
                if self.tile_is_intact(&t)
                    && (crate::math::approx(tx - x, ty - y) as f64) < QUAD_BLAST
                {
                    let name = self.place_name(o as usize);
                    self.destroy_ground(o);
                    self.msg(format!("{name} destroyed by {}", u.label()));
                }
            }
            self.msg(format!("{} impact", u.label()));
        } else {
            self.emit(SimEvent::Explosion { x, y, alt: 0, kind: -1 });
            self.msg(format!("{} lost", u.label()));
        }
    }

    /// A warhead at map point (x, y): the air defence units of either side within `r` map units
    /// (also with nobody flying, when `aaa_damage` sleeps).
    fn uav_blast_units(&mut self, x: i32, y: i32, r: f64) {
        for k in 0..self.units.len() {
            let a = self.units[k];
            if !a.destroyed && ((a.x - x) as f64).hypot((a.y - y) as f64) < r {
                self.units[k].destroyed = true;
                self.units[k].burst = 0;
                self.msg(format!("{} destroyed", a.kind.name_on(self.aaa_blue(k))));
                self.emit(SimEvent::Explosion { x: a.x, y: a.y, alt: 0, kind: 0x10 });
                self.emit(SimEvent::AaaDestroyed { unit: k, by: None });
            }
        }
    }

    /// Interceptor `n` hit flight unit `j`: both are gone.
    fn uav_intercept(&mut self, n: usize, j: usize) {
        let u = self.defence_mut().uavs.remove(n);
        let i = u.flight as usize;
        if let Some(Some(f)) = self.campaign.as_mut().and_then(|c| c.flights.get_mut(i)) {
            f.reported = true;
        }
        self.flights[i].flags = 0;
        let (x, y) = u.map();
        let alt = (u.pos[2] / M_PER_WU) as i32;
        self.emit(SimEvent::Explosion { x, y, alt, kind: -6 });
        let name = self.ac_type(self.flights[j].spec).name.clone();
        self.msg(format!("{} hit {name}", u.label()));
        self.ai_kill(j);
    }

    /// The interceptors' locks: one without finds the nearest aircraft or drone (of either side)
    /// within `LOCK_RANGE` and `LOCK_CONE` of its nose; a lock goes when its target is gone or
    /// further than `LOCK_BREAK`.
    fn uav_locks(&mut self) {
        let targets: Vec<(usize, [f64; 3])> =
            self.interceptor_targets().map(|j| (j, flight_metres(&self.flights[j]))).collect();
        let mut news = Vec::new();
        for (n, u) in self.uavs().iter().enumerate() {
            if u.kind != UavKind::Interceptor {
                continue;
            }
            let lock = if u.battery <= EMPTY || u.mode == UavMode::Launch {
                None
            } else if let Some(j) = u.lock {
                targets
                    .iter()
                    .find(|e| e.0 == j as usize)
                    .filter(|e| len(sub(e.1, u.pos)) < LOCK_BREAK)
                    .map(|_| j)
            } else {
                let nose = [
                    u.heading.sin() * u.pitch.cos(),
                    u.heading.cos() * u.pitch.cos(),
                    u.pitch.sin(),
                ];
                targets
                    .iter()
                    .map(|&(j, p)| (j, sub(p, u.pos)))
                    .map(|(j, r)| (j, len(r), dot(r, nose)))
                    .filter(|&(_, d, along)| d < LOCK_RANGE && along > d * LOCK_CONE.cos())
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .map(|(j, ..)| j as u16)
            };
            if lock != u.lock {
                news.push((n, lock));
            }
        }
        for (n, lock) in news {
            let label = self.uavs()[n].label();
            match lock {
                Some(j) => {
                    let name = self.ac_type(self.flights[j as usize].spec).name.clone();
                    self.msg(format!("{label} locked on {name}"));
                }
                None if self.uavs()[n].battery > EMPTY => self.msg(format!("{label} lost lock")),
                None => {}
            }
            self.defence_mut().uavs[n].lock = lock;
        }
    }

    /// Once a frame: drones shot down or whose center is gone are lost, the batteries run down,
    /// the interceptors lock on, and once a second the drones look for the enemy's air
    /// defences.
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
        self.uav_locks();
        if self.frame % self.fs.max(1) == 0 {
            self.uav_look();
        }
    }

    /// The drones' sensors: what they see now joins what they have found; what is destroyed
    /// is forgotten.
    fn uav_look(&mut self) {
        let mut known = self.defence().map(|d| d.uav_known.clone()).unwrap_or_default();
        known.retain(|s| match *s {
            Seen::Aaa(k) => self.units.get(k as usize).is_some_and(|u| !u.destroyed),
            Seen::Object(o) => self
                .objects
                .get(o as usize)
                .is_some_and(|o| o.flags & oflags::DESTROYED == 0 && o.unit_type != 0),
        });
        for u in self.uavs() {
            let r = u.kind.sensor();
            let near = |x: f64, y: f64| (x - u.pos[0]).hypot(y - u.pos[1]) < r;
            for (k, a) in self.units.iter().enumerate() {
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
            Seen::Aaa(k) => self.units.get(k as usize).map(|u| (u.x, u.y)),
            Seen::Object(o) => self.objects.get(o as usize).map(|o| (o.x, o.y)),
        }
    }
}
