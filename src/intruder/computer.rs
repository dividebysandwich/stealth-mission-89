//! The A-6's attack-navigation computer: what it steers to (the lock or the pod's ground
//! point, else the coordinates keyed, else the route's point), the steering, release and
//! landing solutions its displays show (`vdi`, `scope`, `sight`), and the panel's hold on the
//! game's own systems: the attack buttons choose the bombing mode, the station switches the
//! weapon, the release switch and the interval counter the stick of bombs, MASTER ARM the
//! trigger, the search radar's power switch the radar.

use bevy::math::DVec2;
use bevy::prelude::*;

use super::{AttackMode, FlightMode, IntruderPanel, Power, ReleaseMode, StationSelect};
use crate::aircraft::AircraftKind;
use crate::flight::{Aircraft, Player};
use crate::hud::{HudCues, HudMode, HudState, Waypoints};
use crate::world::{WU, WorldPos, bevy_to_wu};

/// The steering's bank limit (rad) and the heading error that asks for all of it, cruising and
/// attacking.
pub const BANK_LIMIT: f32 = 26.0 * std::f32::consts::PI / 180.0;
const STEER_FULL: (f32, f32) = (0.52, 0.26);
/// Seconds before the release the IN RANGE light comes on.
const IN_RANGE_SECS: f32 = 20.0;
/// The climb a high loft releases in (rad).
const LOFT_CLIMB: f32 = 0.6;
/// The bays (`loadout::A6_STATIONS`) of the armament panel's five stations: the third is the
/// centre line, where the A-6C's pod hangs.
pub const STATION_BAYS: [Option<usize>; 5] = [Some(0), Some(1), None, Some(2), Some(3)];
/// Weapons that go by the switches' ROCKETS position: the missiles and the rocket pods (the
/// Sidewinder, HARM, Penguin, Harpoon, Maverick, Hydra, APKWS, Shrike and Zuni).
const ROCKET_WEAPONS: [i32; 9] = [0, 2, 3, 4, 5, 31, 32, 35, 36];
/// Seconds between weapon steps while the selection follows the switches.
const STEP_SECS: f32 = 0.3;

/// The landing the pilot's display leads to: a carrier's or a runway's.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Landing {
    /// Right of the centre line and above the glide path (rad).
    pub lineup: f32,
    pub glide: f32,
    /// To the touchdown (m), the approach's course (rad from north) and its glide path (rad).
    pub range: f32,
    pub course: f32,
    pub slope: f32,
}

/// What the computer has worked out this frame.
#[derive(Resource, Debug, Clone, Default)]
pub struct Computer {
    /// An A-6 is flown.
    pub on: bool,
    /// The point it steers to (m east, north), and whether it is a lock or the pod's point
    /// (else the route's).
    pub target: Option<DVec2>,
    pub designated: bool,
    /// The target's bearing right of the nose (rad), its range over the ground (m) and the
    /// seconds to it.
    pub bearing: f32,
    pub range: f32,
    pub time: f32,
    /// The turn the steering asks for (rad, right positive: the target's bearing from the track
    /// over the ground) and the bank it commands.
    pub steer: f32,
    pub bank: f32,
    /// Seconds to the release with an attack on; the weapon would reach.
    pub release: Option<f32>,
    pub in_range: bool,
    /// A high loft's pull-up is due.
    pub pull_up: bool,
    /// Where a bomb or rocket let go now would hit, from the eye (aircraft frame).
    pub impact: Option<Vec3>,
    /// The target from the eye (aircraft frame).
    pub target_dir: Option<Vec3>,
    /// The sim holds a ground lock; it is on a vehicle or gun the pod follows.
    pub lock: bool,
    pub discrete: bool,
    /// Master arm is on and a station with the selected weapon is switched in.
    pub armed: bool,
    /// An attack is on with no station to release.
    pub reselect: bool,
    pub landing: Option<Landing>,
}

/// The panel as it was, and what the weapon selection is being stepped to.
#[derive(Default)]
pub(super) struct Before {
    stations: Option<[StationSelect; 5]>,
    weapon: Option<i32>,
    want: Option<i32>,
    wait: f32,
    attack: Option<Option<AttackMode>>,
    flight: Option<FlightMode>,
    /// The pod's FLIR was switched on for the A-6 (off again in another aircraft).
    flir: bool,
}

/// The smaller angle from `b` to `a` (rad).
pub fn wrap(a: f32) -> f32 {
    (a + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

/// The bank the steering commands for a turn of `steer` (rad).
pub fn bank_command(steer: f32, attacking: bool) -> f32 {
    let full = if attacking { STEER_FULL.1 } else { STEER_FULL.0 };
    (steer / full).clamp(-1.0, 1.0) * BANK_LIMIT
}

/// How far ahead (m over the ground) a high loft's pull-up starts: the arc of a pull at `g` up
/// to `LOFT_CLIMB`, and the bomb's throw from there (no drag: the cue is early, the release is
/// the computer's).
pub fn loft_range(speed: f32, height: f32, g: f32) -> f32 {
    const G: f32 = 9.81;
    let radius = speed * speed / ((g - 1.0).max(0.5) * G);
    let (s, c) = LOFT_CLIMB.sin_cos();
    let h = height + radius * (1.0 - c);
    let (vu, vh) = (speed * s, speed * c);
    let fall = (vu + (vu * vu + 2.0 * G * h.max(0.0)).sqrt()) / G;
    radius * s + vh * fall
}

/// The stations switched in that hold `weapon`, and the stores on them.
fn switched(panel: &IntruderPanel, bays: &[(i32, i32)], weapon: i32) -> (u8, i32) {
    let mut out = (0, 0);
    for (i, bay) in STATION_BAYS.iter().enumerate() {
        if let Some(&(w, n)) = bay.and_then(|b| bays.get(b))
            && w == weapon
            && n > 0
            && panel.stations[i] != StationSelect::Off
        {
            out = (out.0 + 1, out.1 + n);
        }
    }
    out
}

/// The station switches for `weapon`: its stations in, the others off.
fn select(panel: &mut IntruderPanel, bays: &[(i32, i32)], weapon: i32) {
    let on = if ROCKET_WEAPONS.contains(&weapon) {
        StationSelect::Rockets
    } else {
        StationSelect::Bombs
    };
    for (i, bay) in STATION_BAYS.iter().enumerate() {
        let holds = bay.and_then(|b| bays.get(b)).is_some_and(|&(w, n)| w == weapon && n > 0);
        panel.stations[i] = if holds { on } else { StationSelect::Off };
    }
}

/// The panel's hold on the game: see the module's notes.
#[allow(clippy::too_many_arguments)]
pub(super) fn drive(
    time: Res<Time>,
    settings: Res<crate::settings::Settings>,
    mut panel: ResMut<IntruderPanel>,
    mut hud: ResMut<HudState>,
    mut mfd: ResMut<crate::mfd::MfdState>,
    mut combat: Option<ResMut<crate::combat::Combat>>,
    mut waypoints: ResMut<Waypoints>,
    mut before: Local<Before>,
) {
    if settings.aircraft != AircraftKind::A6 {
        if mfd.panel_ripple.is_some() || mfd.master_safe {
            mfd.panel_ripple = None;
            mfd.master_safe = false;
        }
        if before.flir {
            mfd.flir = false;
        }
        *before = Before::default();
        return;
    }
    // The TRIM pod's picture is its FLIR's: it has no TV camera.
    if !mfd.flir {
        mfd.flir = true;
    }
    before.flir = true;
    // The search radar's power switch is the radar's.
    let standby = panel.search_power != Power::On;
    if mfd.av.radar_standby != standby {
        mfd.av.radar_standby = standby;
    }
    // An attack button in: air to ground, the computer releases; out: the sight's own aim.
    if before.attack != Some(panel.attack) {
        if before.attack.is_some() {
            hud.mode = if panel.attack.is_some() { HudMode::Gnd } else { HudMode::Nav };
        }
        before.attack = Some(panel.attack);
    }
    let ccrp = panel.attack.is_some_and(|m| m != AttackMode::Rocket);
    if hud.ccrp != ccrp {
        hud.ccrp = ccrp;
    }
    // LAND brings the landing system up, the other flight modes take it away.
    if before.flight != Some(panel.flight) {
        if before.flight.is_some() {
            mfd.ils = panel.flight == FlightMode::Land;
        }
        before.flight = Some(panel.flight);
    }
    // TARGET in and a number: the route's point of that number is the target.
    if let Some(n) = panel.entry.take()
        && panel.address_in(super::Address::Target)
        && n >= 1
        && (n as usize) <= waypoints.points.len()
    {
        waypoints.current = n as usize - 1;
        panel.keyed = None;
    }
    let Some(combat) = combat.as_deref_mut() else { return };
    let (bays, cur) = combat.sim.bays();
    let bays = bays.to_vec();
    let weapon = bays.get(cur).map_or(-1, |b| b.0);
    // The switches moved: the selection follows them to the weapon of the station switched in.
    if before.stations.is_some_and(|s| s != panel.stations) {
        let was = before.stations.unwrap_or(panel.stations);
        before.want = STATION_BAYS.iter().enumerate().find_map(|(i, bay)| {
            let (w, n) = *bays.get((*bay)?)?;
            (panel.stations[i] != StationSelect::Off && was[i] == StationSelect::Off && n > 0)
                .then_some(w)
        });
    } else if before.weapon.is_some_and(|w| w != weapon) && before.want.is_none() && weapon >= 0 {
        // The weapon was stepped by its key: the switches follow.
        select(&mut panel, &bays, weapon);
    } else if before.stations.is_none() && weapon >= 0 {
        select(&mut panel, &bays, weapon);
    }
    before.wait -= time.delta_secs();
    match before.want {
        Some(w) if w == weapon || !bays.iter().any(|b| b.0 == w && b.1 > 0) => before.want = None,
        Some(_) if before.wait <= 0.0 => {
            combat.step_weapon();
            before.wait = STEP_SECS;
        }
        _ => {}
    }
    before.stations = Some(panel.stations);
    before.weapon = Some(weapon);
    // The stick of bombs: all the stations' at once, in train at the counter's feet, or one.
    let (stations, stores) = switched(&panel, &bays, weapon);
    let ripple = match panel.release {
        ReleaseMode::Step => None,
        ReleaseMode::Salvo => Some((stations.max(1), 1)),
        ReleaseMode::Train => Some((stores.clamp(1, 255) as u8, panel.interval.max(1))),
    };
    if mfd.panel_ripple != ripple {
        mfd.panel_ripple = ripple;
    }
    let safe = !(panel.master_arm && stations > 0);
    if mfd.master_safe != safe {
        mfd.master_safe = safe;
    }
}

/// Works the solutions out for the displays.
#[allow(clippy::too_many_arguments)]
pub(super) fn compute(
    settings: Res<crate::settings::Settings>,
    panel: Res<IntruderPanel>,
    mfd: Res<crate::mfd::MfdState>,
    cues: Res<HudCues>,
    hud: Res<HudState>,
    waypoints: Res<Waypoints>,
    combat: Option<Res<crate::combat::Combat>>,
    papi: Option<Res<crate::scenery::PapiSites>>,
    aircraft: Query<(&Aircraft, &WorldPos), With<Player>>,
    time: Res<Time>,
    mut out: ResMut<Computer>,
    mut logged: Local<f32>,
) {
    let flown = aircraft.single().ok().filter(|_| settings.aircraft == AircraftKind::A6);
    let Some((a, pos)) = flown else {
        if out.on {
            *out = Computer::default();
        }
        return;
    };
    let mut c = Computer { on: true, ..default() };
    let (heading, _, _) = a.euler();
    let own = DVec2::new(pos.0.x, -pos.0.z);
    let (east, north, _) = bevy_to_wu(pos.0);
    let sim = combat.as_deref().map(|c| &c.sim);
    // The target: the lock or the pod's point, else the coordinates keyed, else the route's.
    let aim = sim
        .and_then(|s| s.ccrp_aim())
        .map(|(x, y)| DVec2::new(x * 32.0 * WU, (0x8000 as f64 - y) * 32.0 * WU));
    c.designated = aim.is_some();
    c.target = aim.or_else(|| {
        panel
            .keyed_waypoint()
            .or_else(|| waypoints.target())
            .filter(|_| !a.damage.autopilot_failed)
            .map(|w| DVec2::new(w.east * WU, w.north * WU))
    });
    if let Some(t) = sim.map(|s| s.targeting()) {
        c.lock = t.ground.is_some() && t.lock_valid;
    }
    c.discrete = mfd.av.tgp.track.is_some();
    let speed = a.ground_knots().max(a.knots() * 0.2) * 0.5144;
    let path = a.path_dir();
    let track = if path.x.hypot(path.z) > 0.05 { path.x.atan2(-path.z) } else { heading };
    let attacking = panel.attack.is_some() && hud.mode == HudMode::Gnd;
    if let Some(t) = c.target {
        let d = t - own;
        let bearing = (d.x as f32).atan2(d.y as f32);
        c.bearing = wrap(bearing - heading);
        c.range = d.length() as f32;
        c.time = if speed > 1.0 { c.range / speed } else { 0.0 };
        c.steer = wrap(bearing - track);
        c.bank = bank_command(c.steer, attacking);
        let ground = a.ground_altitude as f64 * WU;
        let rel = Vec3::new(d.x as f32, (ground - pos.0.y) as f32, -d.y as f32);
        c.target_dir = Some(a.orientation.inverse() * rel - settings.cockpit_eye());
    }
    // The release: the bombsight's, a rocket's reach or the lock's range.
    if attacking {
        c.release = cues.bomb.filter(|b| b.ccrp).and_then(|b| b.release_secs).filter(|s| *s > -1.0);
        c.in_range = match (c.release, cues.rocket) {
            (Some(secs), _) => secs < IN_RANGE_SECS,
            (None, Some(r)) => r.in_range,
            (None, None) => cues.lock && cues.in_range,
        };
        if panel.attack == Some(AttackMode::HiLoft) && c.target.is_some() && c.release.is_some() {
            let height = ((a.altitude - a.ground_altitude) as f64 * WU) as f32;
            let reach = loft_range(speed, height, panel.pull_g.g());
            c.pull_up = c.range < reach && c.steer.abs() < 0.5;
        }
    }
    c.impact = cues.ccip_dir.or(cues.rocket.map(|r| r.dir));
    if let Some(sim) = sim {
        let me = sim.active_pilot();
        let stations = match sim.crew_view(me).filter(|v| v.owner != me) {
            // In another's right seat: his stores, any station switched in that holds some.
            Some(v) => STATION_BAYS
                .iter()
                .enumerate()
                .filter(|(i, bay)| {
                    panel.stations[*i] != StationSelect::Off
                        && bay.and_then(|b| v.stores.get(b)).is_some_and(|s| s.0 >= 0 && s.1 > 0)
                })
                .count() as u8,
            None => {
                let (bays, cur) = sim.bays();
                switched(&panel, bays, bays.get(cur).map_or(-1, |b| b.0)).0
            }
        };
        c.armed = panel.master_arm && stations > 0;
        c.reselect = panel.attack.is_some() && stations == 0;
    }
    // The landing: the carrier a few miles ahead, else the runway the landing system has.
    if panel.flight == FlightMode::Land {
        let carrier = sim.and_then(|s| crate::lso::approaching(s, east, north)).map(|(_, pose)| {
            let (out, right) = pose.out_and_right(east, north);
            let (out, right) = ((out * WU).max(1.0), right * WU);
            let slope = stealth_sim::catapult::GLIDE_DEG.to_radians();
            let height = pos.0.y - stealth_sim::catapult::DECK_M;
            Landing {
                lineup: right.atan2(out) as f32,
                glide: (height.atan2(out) - slope) as f32,
                range: out as f32,
                course: pose.heading as f32,
                slope: slope as f32,
            }
        });
        c.landing = carrier.or_else(|| {
            let (site, along, lateral) =
                crate::scenery::approach_to(&papi.as_ref()?.0, own, heading)?;
            let slope = crate::scenery::GLIDE_PATH_DEG.to_radians();
            let height = (a.altitude - a.ground_altitude) as f64 * WU;
            Some(Landing {
                lineup: lateral.atan2(along) as f32,
                glide: (height.max(0.0).atan2(along) - slope) as f32,
                range: along as f32,
                course: (-site.toward.x as f32).atan2(-site.toward.y as f32),
                slope: slope as f32,
            })
        });
    }
    // Test aid `SM89_A6_LOG`: the solution once a second.
    *logged += time.delta_secs();
    if *logged >= 1.0 && std::env::var_os("SM89_A6_LOG").is_some() {
        *logged = 0.0;
        let stores = sim.map(|s| s.bays().0.to_vec());
        println!(
            "a6 t={:.0} panel {:02x?} mode {:?} ccrp {} armed {} range {:.0} steer {:.2} release {:?} in_range {} \
             pull_up {} alt {:.0} stores {stores:?}",
            time.elapsed_secs(),
            &panel.pack()[..10],
            hud.mode,
            hud.ccrp,
            c.armed,
            c.range,
            c.steer,
            c.release,
            c.in_range,
            c.pull_up,
            pos.0.y,
        );
    }
    *out = c;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The steering banks into the turn, all of its 26 degrees from a quarter radian's error
    /// in an attack and from half a radian's en route, and never more.
    #[test]
    fn steering_commands_a_limited_bank() {
        assert_eq!(bank_command(0.0, false), 0.0);
        assert!(bank_command(0.1, true) > bank_command(0.1, false));
        assert!((bank_command(0.26, true) - BANK_LIMIT).abs() < 1e-4);
        assert!((bank_command(-2.0, false) + BANK_LIMIT).abs() < 1e-4);
        assert!(bank_command(0.2, false) < BANK_LIMIT);
        assert!((wrap(3.5) - (3.5 - std::f32::consts::TAU)).abs() < 1e-5);
    }

    /// A loft throws farther the faster the run in, and a harder pull starts later.
    #[test]
    fn loft_pull_up_range() {
        let slow = loft_range(200.0, 150.0, 4.0);
        let fast = loft_range(250.0, 150.0, 4.0);
        assert!(fast > slow);
        assert!((4000.0..9000.0).contains(&slow), "{slow}");
        assert!(loft_range(250.0, 150.0, 5.0) < fast);
    }

    /// Only stations switched in that hold the selected weapon count; the weapon's key sets
    /// the switches.
    #[test]
    fn switches_and_weapon() {
        let bays = [(33, 2), (34, 1), (34, 1), (36, 4)];
        let mut p = IntruderPanel::default();
        assert_eq!(switched(&p, &bays, 34), (0, 0));
        select(&mut p, &bays, 34);
        assert_eq!(p.stations[1], StationSelect::Bombs);
        assert_eq!(p.stations[3], StationSelect::Bombs);
        assert_eq!(p.stations[0], StationSelect::Off);
        assert_eq!(p.stations[2], StationSelect::Off);
        assert_eq!(switched(&p, &bays, 34), (2, 2));
        select(&mut p, &bays, 36);
        assert_eq!(p.stations[4], StationSelect::Rockets);
        assert_eq!(switched(&p, &bays, 36), (1, 4));
    }
}
