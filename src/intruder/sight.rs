//! The pilot's optical sight: no symbols but reticles on the reflector plate. The fixed
//! reticle marks the boresight (a cross with a dot, a dashed 50 mil circle, dots every 10
//! mils); the movable reticle (25 and 50 mil arcs round a pipper) goes where the computer
//! puts it: on the impact point of the bombs or rockets, or on the target in an attack the
//! computer releases. An arc round it closes as the release comes, and the breakaway bars
//! blink for the pull-up.

use bevy::prelude::*;

use super::computer::Computer;
use super::{AttackMode, IntruderPanel};
use crate::canvas::Canvas;
use crate::hud::modern::{CX, CY, project};
use crate::hud::{HudFrame, HudMode};

const INK: u8 = 15;
const PEN: f32 = 2.2;
/// The movable reticle's travel from the boresight (mils): up, down and aside.
pub const TRAVEL: (f32, f32, f32) = (88.0, 350.0, 120.0);
/// Seconds to the ground at the sink rate under which the breakaway bars blink.
const BREAKAWAY_SECS: f32 = 4.0;

/// An arc of a circle from `a0` to `a1` (rad, clockwise from up).
fn arc(c: &mut Canvas, mid: Vec2, r: f32, a0: f32, a1: f32, w: f32) {
    let n = (((a1 - a0).abs() * r / 6.0).ceil() as usize).max(2);
    let pts: Vec<Vec2> = (0..=n)
        .map(|i| {
            let a = a0 + (a1 - a0) * i as f32 / n as f32;
            mid + Vec2::new(a.sin(), -a.cos()) * r
        })
        .collect();
    c.polyline(&pts, w, INK);
}

/// Where the movable reticle is, in mils right of and below the boresight: `dir` (aircraft
/// frame from the eye) held inside its travel; at the boresight without one.
pub fn movable(dir: Option<Vec3>) -> Vec2 {
    let Some(d) = dir.filter(|d| d.z < -1e-3) else { return Vec2::ZERO };
    let mils = Vec2::new(d.x / -d.z, -d.y / -d.z) * 1000.0;
    Vec2::new(mils.x.clamp(-TRAVEL.2, TRAVEL.2), mils.y.clamp(-TRAVEL.0, TRAVEL.1))
}

pub fn draw(c: &mut Canvas, f: &HudFrame, panel: &IntruderPanel, computer: &Computer) {
    let mil = f.scale * 0.001;
    let bore = project(Vec3::NEG_Z, f.scale).unwrap_or(Vec2::new(CX, CY));
    // The fixed reticle.
    for d in [Vec2::X, Vec2::Y, Vec2::NEG_X, Vec2::NEG_Y] {
        c.stroke(bore + d * 5.0 * mil, bore + d * 18.0 * mil, PEN, INK);
        for k in 3..=4 {
            super::disc(c, bore + d * k as f32 * 10.0 * mil, 1.6, INK);
        }
    }
    super::disc(c, bore, 2.2, INK);
    for k in 0..12 {
        let a = k as f32 * std::f32::consts::TAU / 12.0;
        arc(c, bore, 50.0 * mil, a - 0.14, a + 0.14, PEN);
    }
    // The movable reticle: on the target in an attack the computer releases, else on the
    // impact point with the weapons up.
    let ground = f.state.mode == HudMode::Gnd;
    let dir = match panel.attack {
        Some(m) if m != AttackMode::Rocket => computer.target_dir.filter(|_| ground),
        _ => computer.impact.filter(|_| ground),
    };
    let at = bore + movable(dir) * mil;
    super::disc(c, at, 2.6, INK);
    for (r, half) in [(25.0, 0.55), (50.0, 0.4)] {
        for k in 0..4 {
            let a = k as f32 * std::f32::consts::FRAC_PI_2 + std::f32::consts::FRAC_PI_4;
            arc(c, at, r * mil, a - half, a + half, PEN);
        }
    }
    // The in range arc closes round the reticle as the release comes.
    if computer.in_range {
        let share = computer.release.map_or(1.0, |s| (1.0 - s / 20.0).clamp(0.05, 1.0));
        arc(c, at, 62.0 * mil, 0.0, share * std::f32::consts::TAU, PEN * 1.3);
    }
    // The breakaway bars.
    let sink = -f.climb as f32 * crate::world::WU as f32;
    let low = sink > 5.0 && f.radar_alt * 0.3048 / sink < BREAKAWAY_SECS && !f.gear_down;
    if (computer.pull_up || low) && f.blink {
        for side in [-1.0, 1.0] {
            let x = bore.x + side * 72.0 * mil;
            c.stroke(
                Vec2::new(x, bore.y - 30.0 * mil),
                Vec2::new(x, bore.y + 30.0 * mil),
                PEN * 2.5,
                INK,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The movable reticle follows a direction, and stops at its travel.
    #[test]
    fn movable_reticle_travel() {
        assert_eq!(movable(None), Vec2::ZERO);
        let down = movable(Some(Vec3::new(0.0, -0.1, -1.0)));
        assert!((down.y - 100.0).abs() < 0.5 && down.x.abs() < 0.01);
        assert_eq!(movable(Some(Vec3::new(0.0, -1.0, -1.0))).y, TRAVEL.1);
        assert_eq!(movable(Some(Vec3::new(0.0, 1.0, -1.0))).y, -TRAVEL.0);
        assert_eq!(movable(Some(Vec3::new(1.0, 0.0, -1.0))).x, TRAVEL.2);
        assert_eq!(movable(Some(Vec3::new(0.0, 0.0, 1.0))), Vec2::ZERO);
    }
}
