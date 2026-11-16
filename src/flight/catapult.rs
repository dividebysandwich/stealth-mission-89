//! The player's launch from a carrier's catapult (`stealth_sim::catapult`): held back on it
//! while the engines run up (the shooter beside him circles two fingers: more power), at full
//! power the shooter's signal, a second later the shuttle throws the aircraft down the track
//! and off it, flaps down, faster than it stalls. Both fixed-wing flight models ask
//! [`Aircraft::catapult_speed`] for their ground roll's speed while it holds them.

use serde::{Deserialize, Serialize};
use stealth_sim::catapult::{Phase, SHOOT_S, STROKE_S, report};

use super::{Aircraft, FlightConstants};

/// The stroke ends this much faster than the stall speed (flaps down).
pub const END_SPEED: f32 = 1.25;
/// Thrust (percent) that counts as full power.
const FULL_POWER: f32 = 99.5;

/// Where the launch is.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CatPhase {
    /// Queued behind the catapult for it to be free (`taxi_up` moves him up).
    Wait,
    /// Held back on it, the engines running up.
    Tension,
    /// At full power: the shooter's signal; seconds since.
    Shoot(f32),
    /// The shuttle runs: seconds since it fired, and the speed it ends at (velocity units).
    Stroke(f32, f32),
}

/// A launch from catapult `cat` (0..4) of the carrier at world object `carrier`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CatLaunch {
    pub cat: u8,
    pub carrier: usize,
    pub phase: CatPhase,
}

impl CatLaunch {
    /// The launch as the pilot's report carries it (`PlayerState::catapult`).
    pub fn report(&self) -> u8 {
        let phase = match self.phase {
            CatPhase::Wait => Phase::Wait,
            CatPhase::Tension => Phase::Tension,
            CatPhase::Shoot(_) => Phase::Shoot,
            CatPhase::Stroke(..) => Phase::Stroke,
        };
        report(self.cat as usize, phase)
    }
}

/// The launch a start state names (`PlayerState::catapult`) on carrier object `carrier`, for
/// an aircraft that takes a catapult: not a helicopter, nor a V/STOL type (which takes off
/// short).
pub fn launch_from(report: u8, carrier: usize, k: &FlightConstants) -> Option<CatLaunch> {
    if k.heli.is_some() || k.aero.vtol.is_some() {
        return None;
    }
    let (cat, phase) = stealth_sim::catapult::from_report(report)?;
    let phase = if phase == Phase::Wait { CatPhase::Wait } else { CatPhase::Tension };
    Some(CatLaunch { cat: cat as u8, carrier, phase })
}

/// The carrier world object nearest a point (wu east, north) within its landing box, if any.
pub fn carrier_at(world: &stealth_formats::wld::World, east: f64, north: f64) -> Option<usize> {
    let (mx, my) = (east / 32.0, 32768.0 - north / 32.0);
    world.objects.iter().position(|o| {
        o.target_flags as u16 & 0x200 != 0
            && (mx - o.x as f64).abs() <= 12.0
            && (my - o.y as f64).abs() <= 38.0
    })
}

impl Aircraft {
    /// The catapult's hold on this step's ground roll: the speed along the nose (velocity
    /// units) while it holds the aircraft or throws it, else None (and the launch over).
    /// The flaps stay down and the gear locked, nose-wheel steering and rotation are the
    /// catapult's.
    pub fn catapult_speed(&mut self, dt: f32) -> Option<f32> {
        let mut c = self.catapult?;
        if !self.on_ground || self.ended.is_some() {
            self.catapult = None;
            return None;
        }
        self.flaps = true;
        self.gear_down = true;
        self.brakes = false;
        let full = self.set_thrust >= 100.0 && self.thrust >= FULL_POWER;
        let speed = match c.phase {
            CatPhase::Wait => 0.0,
            CatPhase::Tension => {
                if full {
                    c.phase = CatPhase::Shoot(0.0);
                }
                0.0
            }
            CatPhase::Shoot(_) if !full => {
                c.phase = CatPhase::Tension;
                0.0
            }
            CatPhase::Shoot(t) => {
                let t = t + dt;
                c.phase = if t >= SHOOT_S {
                    CatPhase::Stroke(0.0, self.stall_speed.max(1.0) * END_SPEED)
                } else {
                    CatPhase::Shoot(t)
                };
                0.0
            }
            CatPhase::Stroke(t, end) => {
                let t = t + dt;
                if t >= STROKE_S {
                    self.catapult = None;
                    return Some(end);
                }
                c.phase = CatPhase::Stroke(t, end);
                end * t / STROKE_S
            }
        };
        self.catapult = Some(c);
        Some(speed)
    }

    /// On a catapult, the launch's phase as the pilot's report carries it (0 none).
    pub fn catapult_report(&self) -> u8 {
        self.catapult.map_or(0, |c| c.report())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on_cat() -> Aircraft {
        let mut a = Aircraft::new_on_runway(5000.0);
        a.stall_speed = 120.0 * 27.0;
        a.catapult = Some(CatLaunch { cat: 1, carrier: 0, phase: CatPhase::Tension });
        a
    }

    #[test]
    fn held_until_full_power_then_shot_after_a_second() {
        let mut a = on_cat();
        let dt = 1.0 / 120.0;
        a.set_thrust = 80.0;
        a.thrust = 80.0;
        for _ in 0..1200 {
            assert_eq!(a.catapult_speed(dt), Some(0.0));
        }
        assert_eq!(a.catapult.unwrap().phase, CatPhase::Tension);
        assert!(a.flaps && a.gear_down);
        a.set_thrust = 100.0;
        a.thrust = 100.0;
        // The signal: held a second more.
        let mut held = 0;
        while a.catapult_speed(dt) == Some(0.0) {
            held += 1;
            assert!(held < 1000);
        }
        assert!((held as f32 * dt - SHOOT_S).abs() < 0.02, "{held}");
        // The stroke: over STROKE_S to past the stall speed, then free.
        let mut last = 0.0;
        let mut n = 0;
        while let Some(v) = a.catapult_speed(dt) {
            assert!(v >= last);
            last = v;
            n += 1;
        }
        assert!((n as f32 * dt - STROKE_S).abs() < 0.02, "{n}");
        assert!(last > a.stall_speed * 1.2, "{last}");
        assert!(a.catapult.is_none());
    }

    #[test]
    fn jets_take_the_catapult_not_the_harrier_nor_helicopters() {
        use crate::aircraft::AircraftKind;
        let on = stealth_sim::catapult::report(2, Phase::Tension);
        let k = |a| crate::flight::constants_for(a, false);
        assert!(
            launch_from(on, 7, &k(AircraftKind::F16)).is_some_and(|c| c.cat == 2 && c.carrier == 7)
        );
        assert!(launch_from(on, 7, &k(AircraftKind::F117)).is_some());
        assert!(launch_from(on, 7, &k(AircraftKind::AV8B)).is_none());
        assert!(launch_from(on, 7, &k(AircraftKind::UH60)).is_none());
        assert!(launch_from(0, 7, &k(AircraftKind::F16)).is_none());
    }

    #[test]
    fn throttling_back_before_the_shot_holds_on() {
        let mut a = on_cat();
        a.set_thrust = 100.0;
        a.thrust = 100.0;
        a.catapult_speed(0.5);
        assert!(matches!(a.catapult.unwrap().phase, CatPhase::Shoot(_)));
        a.set_thrust = 70.0;
        a.thrust = 70.0;
        assert_eq!(a.catapult_speed(0.6), Some(0.0));
        assert_eq!(a.catapult.unwrap().phase, CatPhase::Tension);
    }
}
