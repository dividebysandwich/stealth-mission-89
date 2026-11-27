//! The F-14's AI pilot bombs for the player in the RIO's seat: while the LANTIRN's picture is
//! up on the TID and the RIO has a point designated with it near the mission's primary or
//! secondary target, the pilot leaves the route, flies over the point and releases a bomb where
//! it falls onto it (a laser-guided one steers to the designated point on its way down, the
//! simulation's). He comes round for another pass while the designation stays and bombs are
//! left; the picture switched away (or the designation dropped), he flies on along the route.

use bevy::math::DVec2;
use bevy::prelude::*;

use super::{SteerMode, TomcatPanel};
use crate::aircraft::AircraftKind;
use crate::cockpit3d::layout::PanelStyle;
use crate::flight::{Aircraft, FlightSystems, Player};
use crate::input::{Action, ActionEvent};
use crate::world::{WU, WorldPos, bevy_to_wu};

pub(super) fn build(app: &mut App) {
    app.init_resource::<Attack>().add_systems(
        Update,
        fly.after(crate::input::InputSystemsSet)
            .after(crate::script::ScriptSet)
            .before(FlightSystems)
            .run_if(in_state(crate::settings::AppState::Flight)),
    );
}

/// The weapon types the pilot bombs with: the GBU-12, the Mk 82, 83 and 84.
const BOMBS: [i32; 4] = [LGB, 9, crate::loadout::MK83 as i32, crate::loadout::MK84 as i32];
/// The GBU-12 (laser-guided: it steers to the designated point).
const LGB: i32 = 6;
/// A designated point this close to the primary or the secondary target is his to bomb (m).
const TARGET_AREA: f64 = 6_000.0;
/// Closer than this and not lined up, he flies out first (m).
const RUN_LEAST: f64 = 7_000.0;
/// ... lined up meaning the point within this angle of the nose (rad).
const LINED_UP: f64 = 0.35;
/// On the run in the autopilot's heading error is the point's angle off the nose times this.
const LINE_UP_GAIN: f64 = 4.0;
/// He flies out this far beyond the point before turning in again (m).
const EXTEND: f64 = 9_000.0;
/// He releases only when the bomb would fall this close beside the point (m): a free-fall
/// bomb, and a laser-guided one (its fins make up the rest).
const BESIDE_FREE: f64 = 30.0;
const BESIDE_GUIDED: f64 = 250.0;
/// ... and with the wings this level (rad).
const LEVEL: f32 = 0.35;
/// The release button is held from this long before the release point (s), and this long
/// after (a ripple's bombs follow while it is held).
const HOLD_BEFORE: f64 = 1.5;
const HOLD_AFTER: f32 = 2.0;
/// He bombs from at least this high over the ground (altitude units, ~4000 ft).
const HEIGHT_LEAST: f32 = 4000.0;
/// Seconds between his presses of the weapon select key until a bomb is selected.
const SELECT_WAIT: f32 = 0.5;

/// Where the pilot is in his attack.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    /// Towards the point, to release on the way over it.
    RunIn,
    /// Out on this heading (rad) to turn in again.
    Extend(f32),
}

/// The AI pilot's attack.
#[derive(Resource, Debug, Default)]
pub struct Attack {
    pub phase: Option<Phase>,
    /// The distance along the track still to go to the release point (ft), last frame.
    along: Option<f64>,
    /// The release button stays held this long yet (s).
    hold: f32,
    select_wait: f32,
    /// The steering mode before the attack (put back after it).
    steer: Option<SteerMode>,
    /// Bombs released (the test aid's count).
    pub releases: u32,
    log: f32,
}

/// The designated point (wu east, north) is in the primary's or the secondary's target area.
pub fn in_target_area(point: (f64, f64), waypoints: &crate::hud::Waypoints) -> bool {
    [1usize, 2].into_iter().any(|k| {
        let i = waypoints.anchors.get(k).copied().unwrap_or(k);
        waypoints
            .points
            .get(i)
            .is_some_and(|wp| (wp.east - point.0).hypot(wp.north - point.1) * WU < TARGET_AREA)
    })
}

/// The next phase: `d` metres from the point, which is `off` rad off the nose, flying
/// `heading` (rad).
pub fn next_phase(phase: Option<Phase>, d: f64, off: f64, heading: f32) -> Phase {
    match phase {
        // A new attack: straight in from far enough or lined up, else out first.
        None if d >= RUN_LEAST || off.abs() < LINED_UP => Phase::RunIn,
        None => Phase::Extend(heading),
        Some(Phase::Extend(_)) if d >= EXTEND && off.abs() > std::f64::consts::FRAC_PI_2 => {
            Phase::RunIn
        }
        Some(p) => p,
    }
}

/// A bomb released now from `own` (wu east, north) `height` ft over the ground with velocity
/// `v` (ft/s east, south, up): how far its impact point still is from the point `target` along
/// the track, less `half_stick` (ft, <= 0 at and past the release point), how far beside it
/// the bomb passes (ft), and the ground speed (ft/s).
pub fn solution(
    own: (f64, f64),
    height: f64,
    v: [f64; 3],
    target: (f64, f64),
    half_stick: f64,
    drag: f64,
) -> Option<(f64, f64, f64)> {
    let vh = v[0].hypot(v[1]);
    if vh < 1.0 {
        return None;
    }
    let (d, _) = stealth_sim::ballistics::impact(
        v,
        drag,
        height - stealth_sim::ballistics::RELEASE_DROP,
        1.0 / crate::combat::FS as f64,
    )?;
    // The impact's offset is east and south, the track's east and north.
    let rel = DVec2::new(target.0 - (own.0 + d[0]), target.1 - (own.1 - d[1]));
    let track = DVec2::new(v[0], -v[1]) / vh;
    Some((rel.dot(track) - half_stick, rel.perp_dot(track), vh))
}

#[allow(clippy::too_many_arguments)]
fn fly(
    time: Res<Time>,
    settings: Res<crate::settings::Settings>,
    manning: Option<Res<crate::station::Manning>>,
    contacts: Res<crate::mfd::Contacts>,
    waypoints: Res<crate::hud::Waypoints>,
    mfd: Res<crate::mfd::MfdState>,
    combat: Option<Res<crate::combat::Combat>>,
    mut panel: ResMut<TomcatPanel>,
    mut controls: ResMut<crate::input::FlightControls>,
    mut actions: MessageWriter<ActionEvent>,
    mut player: Query<(&mut Aircraft, &WorldPos), With<Player>>,
    mut attack: ResMut<Attack>,
) {
    let dt = time.delta_secs();
    let Ok((mut a, pos)) = player.single_mut() else { return };
    // The player in his own aircraft's back seat, its picture the LANTIRN's.
    let rio = settings.aircraft == AircraftKind::F14
        && manning.as_deref().is_some_and(|m| m.gunner() && !m.remote_gunner())
        && !a.on_ground
        && a.ended.is_none();
    let lantirn = panel.lts_lantirn && super::avionics::tv_shown(&panel, PanelStyle::F14Rio);
    let sim = combat.as_deref().map(|c| &c.sim);
    let bombs = sim.is_some_and(|s| s.bays().0.iter().any(|b| b.1 > 0 && BOMBS.contains(&b.0)));
    let target = contacts
        .ground_lock
        .filter(|_| rio && lantirn && bombs)
        // (Destroyed, it is done: the next designation is the next target.)
        .filter(|_| sim.is_some_and(|s| !s.targeting().target_killed))
        .filter(|p| in_target_area(*p, &waypoints));
    let (Some(target), Some(sim)) = (target, sim) else {
        // Back to the route, the steering as it was.
        if attack.phase.take().is_some() {
            a.orbit = None;
            if let Some(s) = attack.steer.take()
                && panel.steer == SteerMode::AirToGround
            {
                panel.steer = s;
            }
        }
        attack.along = None;
        attack.hold = 0.0;
        return;
    };
    // The autopilot steers the pilot's heading, not the route's (`hud::advance_waypoints`),
    // high enough over the ground for the bomb.
    a.orbit = Some(crate::world::wu_to_bevy(target.0, target.1, a.ground_altitude as f64));
    let least = crate::flight::view_z(a.ground_altitude + HEIGHT_LEAST);
    if a.autopilot() && a.autopilot_alt < least {
        a.autopilot_alt = least;
    }
    // The HUD in A/G, a bomb selected.
    if attack.phase.is_none() {
        attack.steer = Some(panel.steer);
    }
    if panel.steer != SteerMode::AirToGround {
        panel.steer = SteerMode::AirToGround;
    }
    let (bays, cur) = sim.bays();
    let weapon = if sim.gun_selected() { -1 } else { bays.get(cur).map_or(-1, |b| b.0) };
    // (All the stations of a kind are one selection: the selected one may be the empty one.)
    let selected = BOMBS.contains(&weapon) && bays.iter().any(|b| b.0 == weapon && b.1 > 0);
    attack.select_wait -= dt;
    if !selected && attack.select_wait <= 0.0 {
        attack.select_wait = SELECT_WAIT;
        actions.write(ActionEvent(Action::SelectWeapon));
    }

    let (east, north, _) = bevy_to_wu(pos.0);
    let (de, dn) = (target.0 - east, target.1 - north);
    let d = de.hypot(dn) * WU;
    let bearing = de.atan2(dn);
    let (heading, _, roll) = a.euler();
    let off = (bearing - heading as f64 + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
        - std::f64::consts::PI;
    let mut phase = next_phase(attack.phase, d, off, heading);

    // The release: where the bomb falls now against the point.
    let drag = stealth_sim::ballistics::drag(30, sim.opts.fix_classic_bugs);
    let v = stealth_sim::ballistics::release_velocity(
        crate::flight::to_bam(heading) as i32,
        a.velocity as i32,
        a.climb_rate as i32,
    );
    let (n, spacing) = mfd.ripple();
    let half_stick = (n.max(1) - 1) as f64 * spacing as f64 / 2.0;
    let height = (a.altitude - a.ground_altitude) as f64;
    let solved = drag
        .filter(|_| selected && phase == Phase::RunIn)
        .and_then(|k| solution((east, north), height, v, target, half_stick, k));
    let mut along_now = None;
    if let Some((along, beside, vh)) = solved {
        along_now = Some(along);
        let most = if weapon == LGB { BESIDE_GUIDED } else { BESIDE_FREE };
        let good = beside.abs() * WU < most && roll.abs() < LEVEL;
        // The button held through the release point (the bomb goes there by itself in CCRP).
        if good && along > 0.0 && along / vh < HOLD_BEFORE {
            attack.hold = attack.hold.max(dt * 2.0);
        }
        if attack.along.is_some_and(|last| last > 0.0) && along <= 0.0 {
            if good {
                actions.write(ActionEvent(Action::FireWeapon));
                attack.hold = HOLD_AFTER;
                attack.releases += 1;
            }
            // Over the point and out, to come in again.
            phase = Phase::Extend(heading);
        }
    } else if phase == Phase::RunIn && d < 500.0 {
        // Over the point with nothing to drop yet.
        phase = Phase::Extend(heading);
    }
    attack.along = along_now;
    if attack.hold > 0.0 {
        attack.hold -= dt;
        controls.fire = true;
    }
    let steer = match phase {
        // (The autopilot banks gently on a small error: it is steered past the point's bearing
        // to be lined up in time.)
        Phase::RunIn => heading + (off * LINE_UP_GAIN).clamp(-1.0, 1.0) as f32,
        Phase::Extend(h) => h,
    };
    a.waypoint_bearing = crate::flight::to_bam(steer).rem_euclid(65536.0);
    attack.phase = Some(phase);

    // Test aid `SM89_F14_LOG`.
    attack.log += dt;
    if attack.log >= 1.0 && std::env::var_os("SM89_F14_LOG").is_some() {
        attack.log = 0.0;
        eprintln!(
            "f14 attack {:?} d={:.0} off={:.2} alt={:.0} kt={:.0} weapon={} solved={:?} releases={}",
            phase,
            d,
            off,
            a.altitude,
            a.knots(),
            weapon,
            solved.map(|s| (s.0.round(), s.1.round())),
            attack.releases
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hud::{Waypoint, Waypoints};

    #[test]
    fn only_points_near_the_targets_are_bombed() {
        let at = |east: f64, north: f64| Waypoint { east, north };
        let km = 1000.0 / WU;
        let w = Waypoints::planned(
            vec![at(0.0, 0.0), at(100.0 * km, 0.0), at(0.0, 100.0 * km), at(0.0, 0.0)],
            vec![0, 1, 2, 3],
        );
        assert!(in_target_area((100.0 * km, 3.0 * km), &w));
        assert!(in_target_area((-2.0 * km, 100.0 * km), &w));
        // The route's first point and the base are no targets.
        assert!(!in_target_area((1.0 * km, 1.0 * km), &w));
        assert!(!in_target_area((100.0 * km, 10.0 * km), &w));
    }

    #[test]
    fn he_runs_in_from_afar_and_extends_when_close_and_off() {
        assert_eq!(next_phase(None, 20_000.0, 2.0, 0.5), Phase::RunIn);
        assert_eq!(next_phase(None, 3_000.0, 0.1, 0.5), Phase::RunIn);
        assert_eq!(next_phase(None, 3_000.0, 1.0, 0.5), Phase::Extend(0.5));
        // Out until far enough with the point behind.
        let out = Some(Phase::Extend(0.5));
        assert_eq!(next_phase(out, 5_000.0, 3.0, 1.0), Phase::Extend(0.5));
        assert_eq!(next_phase(out, 10_000.0, 1.0, 1.0), Phase::Extend(0.5));
        assert_eq!(next_phase(out, 10_000.0, 3.0, 1.0), Phase::RunIn);
        assert_eq!(next_phase(Some(Phase::RunIn), 100.0, 3.0, 1.0), Phase::RunIn);
    }

    #[test]
    fn the_release_point_is_where_the_bomb_falls_onto_the_target() {
        let drag = stealth_sim::ballistics::drag(30, true).unwrap();
        // Northward at 700 ft/s, 8000 ft up.
        let v = [0.0, -700.0, 0.0];
        let (d, _) = stealth_sim::ballistics::impact(
            v,
            drag,
            8000.0 - stealth_sim::ballistics::RELEASE_DROP,
            1.0 / crate::combat::FS as f64,
        )
        .unwrap();
        let throw = -d[1];
        assert!(throw > 5000.0, "{throw}");
        // Short of the release point, at it, past it; a target off to the east is beside.
        let s = |north: f64, east: f64| {
            solution((0.0, 0.0), 8000.0, v, (east, north), 0.0, drag).unwrap()
        };
        assert!(s(throw + 1000.0, 0.0).0 > 999.0);
        assert!(s(throw, 0.0).0.abs() < 1e-6);
        assert!(s(throw - 500.0, 0.0).0 < -499.0);
        assert!(s(throw, 0.0).1.abs() < 1e-6);
        assert!((s(throw, 300.0).1.abs() - 300.0).abs() < 1e-6);
        // A ripple starts half its stick early.
        let half = solution((0.0, 0.0), 8000.0, v, (0.0, throw), 200.0, drag).unwrap();
        assert!((half.0 + 200.0).abs() < 1e-6);
    }
}
