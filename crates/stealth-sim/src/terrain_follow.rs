//! A terrain following radar's control law (the Tornado's, `src/flight/terrain_follow.rs` in the
//! game, and the AI's low flyers, [`Sim::noe_gamma`]): it compares the terrain ahead with a
//! `template` ("ski toe"): `clearance` under the flight path, level for the reaction distance
//! and then curving up as a pull-up of `PULL_G`. Terrain above the template asks for a climb,
//! terrain all below it lets the aircraft descend; `command` turns the largest difference into
//! a flight path angle. Everything in metres and metres a second.

use crate::Sim;
use crate::types::{FlightUnit, WU_METRES, fflags};

/// Extra load factor the pull-up template assumes (the control law may pull more).
const PULL_G: f32 = 1.5;
/// Reaction time (s): obstacles count as this much flight time nearer.
const REACTION: f32 = 1.5;
/// Time (s) over which the height is brought to the clearance; more than `MARGIN` of the
/// clearance below it twice as fast.
const CAPTURE: f32 = 3.0;
const MARGIN: f32 = 0.3;
/// Steepest commanded dive and climb (rad).
pub const MAX_DIVE: f32 = 0.3;
pub const MAX_CLIMB: f32 = 0.6;
const G: f32 = 9.81;

/// The template's height `d` metres ahead at `speed` (m/s), relative to a level flight path
/// through the aircraft (metres): `clearance` under it for the reaction distance, then rising
/// as the pull-up.
pub fn template(d: f32, speed: f32, clearance: f32) -> f32 {
    let v = speed.max(50.0);
    let x = (d - v * REACTION).max(0.0);
    x * x * G * PULL_G / (2.0 * v * v) - clearance
}

/// The flight path angle (rad) to fly at `speed` (m/s) over a profile of (distance ahead,
/// obstacle height relative to the aircraft) in metres: the climb that puts the template over
/// the highest obstacle above it, or the descent towards the nearest under it. Past the
/// reaction distance the template's own curve makes up the climb; nearer obstacles are closed
/// in on over `CAPTURE` seconds (smoothly holding the clearance; far under it faster, which
/// keeps a sudden obstacle from being cleared late).
pub fn command(profile: &[(f32, f32)], speed: f32, clearance: f32) -> f32 {
    let v = speed.max(50.0);
    let lag = v * REACTION;
    profile
        .iter()
        .map(|&(d, h)| {
            let x = (d - lag).max(0.0);
            let above = h - template(d, v, clearance);
            let urgent = (above - clearance * MARGIN).max(0.0) * (1.0 - x / (v * CAPTURE)).max(0.0);
            ((above + urgent) / x.max(v * CAPTURE)).atan()
        })
        .fold(-MAX_DIVE, f32::max)
        .min(MAX_CLIMB)
}

/// The AI's low flyers: their height over the ground (altitude units, 100 ft).
pub const NOE_CLEARANCE: i32 = 100;
/// Look-ahead: this much flight time, at least `MIN_AHEAD` metres; sample spacing (metres) at
/// the aircraft, growing with the distance; the scanned corridor's half width (metres) at the
/// aircraft, growing with the distance.
const AHEAD_SECS: f32 = 20.0;
const MIN_AHEAD: f32 = 3000.0;
const SPACING: f32 = 30.0;
const SPACING_GROWTH: f32 = 0.03;
const HALF_WIDTH: f32 = 30.0;
const WIDTH_GROWTH: f32 = 0.05;
/// A low flyer on its way home climbs for the landing this close to its base (map units, 30 km:
/// well before its approach point, 10 km out).
const NOE_HOME: i32 = 0x780;

impl Sim {
    /// Flight unit `i` flies nap of the earth: an F-117A, F-22A or Tornado of the campaign's
    /// on a strike or recon run, in and out (not taking off, landing or falling).
    pub(crate) fn noe(&self, i: usize, u: &FlightUnit) -> bool {
        use crate::ai_missions::{F22, F117, TORNADO};
        use crate::campaign::Role;
        let busy = fflags::TAKEOFF | fflags::FINAL | fflags::ROLLOUT | fflags::DESTROYED;
        if !matches!(u.spec, F117 | F22 | TORNADO) || u.flags & busy != 0 {
            return false;
        }
        let Some(f) = self.campaign_flight(i) else { return false };
        if !matches!(f.role, Role::Strike | Role::Recon) || f.kamikaze {
            return false;
        }
        if f.strike.is_some() {
            return true;
        }
        let home = self.objects.get(f.home.max(0) as usize).map_or(f.anchor, |o| (o.x, o.y));
        u.flags & fflags::RTB != 0 && crate::math::approx(u.x - home.0, u.y - home.1) > NOE_HOME
    }

    /// The flight path angle (rad) a low flyer's terrain following commands at (wu east, wu
    /// south, altitude units), heading `psi` (rad), speed `v` (wu/s), turning at `turn` (rad/s)
    /// for at most `to_go` (rad) more: the mountains and rolling hills along the track ahead
    /// (curving with the turn) under `clearance` (altitude units, [`NOE_CLEARANCE`] for the
    /// low flyers).
    pub(crate) fn noe_gamma(
        &self,
        pos: [f64; 3],
        psi: f64,
        v: f64,
        turn: f64,
        to_go: f64,
        clearance: i32,
    ) -> f64 {
        let Some(t) = self.tiles.as_ref() else { return 0.0 };
        let m = WU_METRES as f32;
        let speed = (v as f32 * m).max(50.0);
        let ahead = (speed * AHEAD_SECS).max(MIN_AHEAD);
        let ground = |x: f64, y: f64| {
            let (mx, my) = ((x as i32 >> 5) & 0xFFFF, (y as i32 >> 5) & 0xFFFF);
            t.relief.height(mx, my).max(t.hills.ground_alt(mx, my)) as f32 * m
        };
        let here = pos[2] as f32 * m;
        let mut profile = Vec::with_capacity(128);
        let (mut d, mut p, mut dir, mut turned) = (0.0f32, (pos[0], pos[1]), psi, 0.0f64);
        while d <= ahead {
            let step = SPACING + SPACING_GROWTH * d;
            let half = ((HALF_WIDTH + WIDTH_GROWTH * d) / m) as f64;
            let (s, c) = dir.sin_cos();
            let top = [0.0, -half, half]
                .into_iter()
                .map(|o| ground(p.0 + c * o, p.1 + s * o))
                .fold(0.0, f32::max);
            profile.push((d, top - here));
            let left = to_go.abs() - turned;
            let dt = (turn * (step / speed) as f64).clamp(-left.max(0.0), left.max(0.0));
            turned += dt.abs();
            dir += dt;
            let w = (step / m) as f64;
            p = (p.0 + dir.sin() * w, p.1 - dir.cos() * w);
            d += step;
        }
        command(&profile, speed, clearance as f32 * m) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLEARANCE: f32 = 20.0;

    #[test]
    fn command_follows_the_profile() {
        let v = 250.0;
        // Level at the clearance over flat ground: hold it.
        let flat: Vec<_> = (0..200).map(|i| (i as f32 * 25.0, -CLEARANCE)).collect();
        assert!(command(&flat, v, CLEARANCE).abs() < 0.01);
        // High above it: the steepest dive.
        let high: Vec<_> = flat.iter().map(|&(d, _)| (d, -1500.0)).collect();
        assert_eq!(command(&high, v, CLEARANCE), -MAX_DIVE);
        // A 300 m wall 1.5 km ahead: climbing already, a far one not yet.
        let wall = |at: f32| {
            flat.iter().map(move |&(d, h)| (d, if d > at { 300.0 - CLEARANCE } else { h }))
        };
        assert!(command(&wall(1500.0).collect::<Vec<_>>(), v, CLEARANCE) > 0.05);
        assert!(command(&wall(4900.0).collect::<Vec<_>>(), v, CLEARANCE).abs() < 0.01);
        // Right in front: a steep climb.
        assert!(command(&wall(100.0).collect::<Vec<_>>(), v, CLEARANCE) > 0.3);
    }
}
