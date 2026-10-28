//! Camera shake and airframe buffet (`Effects::shake`), all on the CPU: the gun firing, blasts
//! near the aircraft (flak, bombs, kills, hits), the buffet approaching the stall, touchdowns and
//! the rumble of the wheels on the ground. Short events add "trauma" that dies away; the gun,
//! the buffet and the ground roll hold a level while they last.
//!
//! On the desktop the head (cockpit views) or the camera (external views) jitters. In VR the
//! head never moves (that makes people sick): the cockpit model jitters around it instead.

use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::transform::TransformSystems;

use crate::camera::{EyeCamera, HeadAnchor, RigSystems, View, ViewState};
use crate::cockpit3d::CockpitRoot;
use crate::flight::{Aircraft, Player};
use crate::settings::{AppState, Settings};
use crate::world::WorldPos;

pub struct ShakePlugin;

impl Plugin for ShakePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Shake>().add_systems(OnEnter(AppState::Flight), reset).add_systems(
            PostUpdate,
            shake
                .after(RigSystems)
                .before(TransformSystems::Propagate)
                .run_if(in_state(AppState::Flight)),
        );
    }
}

/// Trauma lost per second (it starts at up to 1).
const DECAY: f32 = 1.4;
/// Rotation at full shake (radians: pitch, yaw, roll) and the head's travel (m).
const ANGLES: Vec3 = Vec3::new(0.02, 0.012, 0.015);
const TRAVEL: f32 = 0.006;
/// External views shake this much less.
const EXTERNAL: f32 = 0.5;
/// VR: the cockpit's rotation and travel at full shake (a vibration, not a lurch).
const VR_ANGLE: f32 = 0.004;
const VR_TRAVEL: f32 = 0.004;

/// Held levels: the gun firing, the stall buffet at its worst, the ground roll at speed.
const GUN: f32 = 0.35;
const BUFFET: f32 = 0.22;
const GROUND: f32 = 0.09;
/// The buffet starts this far above the stall speed (ratio) and is full at it.
const BUFFET_FROM: f32 = 1.2;
/// Knots at which the ground roll's rumble is full.
const GROUND_KNOTS: f32 = 150.0;
/// A touchdown's trauma: at least, at most and per ft/s of sink.
const TOUCH: (f32, f32, f32) = (0.15, 0.8, 1.0 / 15.0);

#[derive(Resource, Default)]
struct Shake {
    trauma: f32,
    /// Seconds of shake time (runs only while the flight does).
    t: f32,
    /// Last frame: on the ground, the climb rate (wu/s).
    on_ground: bool,
    climb: f32,
}

fn reset(mut shake: ResMut<Shake>) {
    *shake = Shake { on_ground: true, ..default() };
}

/// How deep into the stall buffet the aircraft is: 0 at `BUFFET_FROM` times the stall speed, 1
/// at the stall speed and on past it below (the VR stick's rumble keeps growing; the view's
/// buffet is full at 1). 0 on the ground or without a stall speed (helicopters).
pub fn stall_buffet(a: &Aircraft) -> f32 {
    if a.on_ground || a.stall_speed <= 0.0 {
        return 0.0;
    }
    let r = a.velocity / a.stall_speed;
    ((BUFFET_FROM - r) / (BUFFET_FROM - 1.0)).max(0.0)
}

/// A blast's trauma at `d` metres: its strength, halved at its reach.
fn blast_trauma(kind: i32, d: f64) -> f32 {
    let (strength, reach) = match kind {
        crate::explosions::FLAK => (0.6, 250.0),
        // A gun round's hit: only close by (on the aircraft itself).
        -1 => (0.5, 40.0),
        -6 => (0.8, 600.0),
        0x10 => (0.5, 400.0),
        _ => (0.6, 500.0),
    };
    strength / (1.0 + (d / reach).powi(2)) as f32
}

/// Smooth noise in -1..1: incommensurate sines (Hz) offset by `seed`.
fn noise(t: f32, seed: f32, hz: [f32; 3]) -> f32 {
    use std::f32::consts::TAU;
    (t * hz[0] * TAU + seed).sin() * 0.5
        + (t * hz[1] * TAU + 2.0 * seed).sin() * 0.3
        + (t * hz[2] * TAU + 3.0 * seed).sin() * 0.2
}

#[allow(clippy::too_many_arguments)]
fn shake(
    mut shake: ResMut<Shake>,
    time: Res<Time>,
    settings: Res<Settings>,
    view: Res<ViewState>,
    combat: Option<Res<crate::combat::Combat>>,
    manning: Option<Res<crate::station::Manning>>,
    aircraft: Query<(&Aircraft, &WorldPos), With<Player>>,
    mut head: Query<&mut Transform, (With<HeadAnchor>, Without<EyeCamera>)>,
    mut cam: Query<&mut Transform, (With<EyeCamera>, Without<HeadAnchor>)>,
    mut roots: Query<&mut Transform, (With<CockpitRoot>, Without<HeadAnchor>, Without<EyeCamera>)>,
) {
    let dt = time.delta_secs();
    // The cockpit's own rotation is none: undo last frame's jolt (follow_eye resets its place).
    for mut tf in &mut roots {
        if tf.rotation != Quat::IDENTITY {
            tf.rotation = Quat::IDENTITY;
        }
    }
    let Ok((a, pos)) = aircraft.single() else { return };
    if dt <= 0.0 || !settings.effects.shake || manning.is_some() || a.ended.is_some() {
        return;
    }
    let s = &mut *shake;
    s.t += dt;
    s.trauma = (s.trauma - DECAY * dt).max(0.0);
    let mut held = 0.0f32;
    if let Some(c) = combat.as_deref() {
        let me: DVec3 = pos.0;
        for &(x, y, alt, kind) in &c.explosions {
            let p = crate::sim_render::map_point(x as f64 + 0.5, y as f64 + 0.5, alt.max(0) as f64);
            s.trauma += blast_trauma(kind, p.distance(me));
        }
        if c.sim.gun_firing() && c.end.is_none() && !c.sim.player_absent() {
            held = held.max(GUN);
        }
    }
    if a.on_ground {
        if !s.on_ground && a.has_flown {
            let sink = (-s.climb).max(0.0);
            s.trauma += (sink * TOUCH.2).clamp(TOUCH.0, TOUCH.1);
        }
        let kts = a.velocity / 27.0;
        held = held.max(GROUND * (kts / GROUND_KNOTS).min(1.0));
    } else {
        let b = stall_buffet(a).min(1.0);
        held = held.max(BUFFET * b * b);
    }
    s.on_ground = a.on_ground;
    s.climb = a.climb_rate;
    s.trauma = s.trauma.min(1.0);
    // Trauma squared: big blasts jolt, small ones barely show.
    let level = (s.trauma * s.trauma + held).min(1.0);
    if level < 0.001 {
        return;
    }
    let t = s.t;
    let n = Vec3::new(
        noise(t, 0.0, [9.0, 15.0, 23.0]),
        noise(t, 1.7, [8.0, 13.0, 21.0]),
        noise(t, 3.1, [10.0, 17.0, 25.0]),
    );
    let jolt = |angles: Vec3| {
        let a = n * angles * level;
        Quat::from_rotation_x(a.x) * Quat::from_rotation_y(a.y) * Quat::from_rotation_z(a.z)
    };
    let travel = Vec3::new(n.y, n.x, n.z) * level;
    if settings.vr {
        // The cockpit vibrates around the head.
        if view.first_person() {
            for mut tf in &mut roots {
                tf.translation += travel * VR_TRAVEL;
                tf.rotation = jolt(Vec3::splat(VR_ANGLE));
            }
        }
        return;
    }
    match view.view {
        View::Cockpit | View::LookLeft | View::LookRight | View::LookRear => {
            if let Ok(mut head) = head.single_mut() {
                head.rotation *= jolt(ANGLES);
                head.translation += travel * TRAVEL;
            }
        }
        View::Chase | View::Side | View::Tactical => {
            if let Ok(mut cam) = cam.single_mut() {
                cam.rotation *= jolt(ANGLES * EXTERNAL);
            }
        }
        View::Pilot | View::Weapon | View::Free => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blasts_fade_with_distance() {
        let near = blast_trauma(crate::explosions::FLAK, 50.0);
        let far = blast_trauma(crate::explosions::FLAK, 2000.0);
        assert!(near > 0.5 && far < 0.01, "{near} {far}");
        // A gun hit only counts on the aircraft.
        assert!(blast_trauma(-1, 300.0) < 0.01);
    }
}
