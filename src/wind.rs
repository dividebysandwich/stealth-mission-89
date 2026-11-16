//! The air the player flies through: the mean wind (`WeatherState::wind`) at the aircraft's
//! height, its gusts, the outflow of storm cells and turbulence. Written to
//! `Aircraft::wind` before the flight models run, which fly through it.
//!
//! Only the player's own flight uses it (and the rain, smoke and sea draw it); the simulation's
//! AI aircraft and weapons fly in still air.

use bevy::prelude::*;
use stealth_mission::weather::CloudKind;

use crate::flight::{Aircraft, KNOT_WU, Player};
use crate::settings::AppState;
use crate::weather::WeatherState;
use crate::world::{WorldPos, bevy_to_wu};

pub struct WindPlugin;

impl Plugin for WindPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_air.before(crate::flight::FlightSystems).run_if(in_state(AppState::Flight)),
        );
    }
}

/// One knot in metres per second.
pub const KNOT_MS: f32 = 0.514_444;
/// Storm cells' outflow reaches this far past their edge (map units, 64 per km).
const OUTFLOW_REACH: i32 = 4 * 64;
/// Outflow at a cell's edge (knots): this plus up to 15 by cell.
const OUTFLOW_KT: f32 = 30.0;
/// Turbulence's horizontal and vertical gusts at strength 1 (knots).
const TURB_KT: (f32, f32) = (12.0, 9.0);

/// Smooth noise in -1..1: incommensurate sines (Hz) offset by `seed`.
fn noise(t: f32, seed: f32, hz: [f32; 3]) -> f32 {
    use std::f32::consts::TAU;
    (t * hz[0] * TAU + seed).sin() * 0.5
        + (t * hz[1] * TAU + 2.0 * seed).sin() * 0.3
        + (t * hz[2] * TAU + 3.0 * seed).sin() * 0.2
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Share of the mean wind (given at 2000 ft) blowing at a height above the ground (feet):
/// slowed by the ground's friction below, a little stronger high up.
pub fn height_profile(agl_ft: f32) -> f32 {
    let h = agl_ft.max(0.0);
    let low = if h < 30.0 {
        0.55 + 0.05 * h / 30.0
    } else {
        (0.6 + 0.4 * (h / 30.0).ln() / (2000.0f32 / 30.0).ln()).min(1.0)
    };
    low + 0.2 * smoothstep(10_000.0, 25_000.0, h)
}

impl WeatherState {
    /// The mean wind (no gusts) at a height above the ground (m/s, Bevy frame).
    pub fn mean_air(&self, agl_ft: f32) -> Vec3 {
        let (e, n) = self.wind.velocity();
        Vec3::new(e, 0.0, -n) * KNOT_MS * height_profile(agl_ft)
    }
}

/// Storm cells near a map position (map units, feet): their outflow (knots, east and north),
/// downdraft (knots, up) and the turbulence of the gust front, scaled by how built up the
/// storms are.
fn storm_outflow(
    w: &stealth_mission::Weather,
    x: i32,
    y: i32,
    ft: f32,
    built: f32,
) -> (Vec2, f32, f32) {
    let (mut out, mut down, mut turb) = (Vec2::ZERO, 0.0, 0.0f32);
    if built <= 0.0 || !w.has_storms() {
        return (out, down, turb);
    }
    for c in w.clouds_near(x, y, stealth_mission::weather::STORM_CELL + OUTFLOW_REACH) {
        if c.kind != CloudKind::Cumulonimbus {
            continue;
        }
        // East and north from the cell's centre (map y grows southwards).
        let d = Vec2::new((x - c.x) as f32, (c.y - y) as f32);
        let dist = d.length();
        let r = c.r.max(1) as f32;
        if dist > r + OUTFLOW_REACH as f32 {
            continue;
        }
        let peak = OUTFLOW_KT
            + 15.0 * stealth_mission::wind::unit(c.gx as u64 ^ (c.gy as u64) << 32, w.seed as u64);
        // Rising from the calm centre to the edge, dying away past it.
        let radial =
            if dist < r { dist / r } else { 1.0 - smoothstep(0.0, OUTFLOW_REACH as f32, dist - r) };
        // The outflow spreads along the ground, under the cloud base.
        let low = 1.0 - smoothstep(c.base_ft as f32, c.base_ft as f32 * 2.5 + 1500.0, ft);
        let dir = if dist > 1.0 { d / dist } else { Vec2::ZERO };
        out += dir * peak * radial * low * built;
        // The downdraft under the core.
        if dist < r && ft < c.base_ft as f32 {
            down += 15.0 * (1.0 - dist / r) * built;
        }
        // Rough at the gust front (round the cell's edge) and inside.
        let front = 1.0 - smoothstep(0.0, 0.6 * OUTFLOW_REACH as f32, (dist - r).abs());
        turb = turb.max(0.6 * front * low * built);
    }
    (out, down, turb)
}

/// The air at the player: mean wind at its height, gusts, storm outflow and turbulence.
fn update_air(
    mut state: ResMut<WeatherState>,
    mut q: Query<(&mut Aircraft, &WorldPos), With<Player>>,
) {
    let Ok((mut a, pos)) = q.single_mut() else { return };
    let w = state.wind;
    let agl = (a.altitude - a.ground_altitude).max(0.0);
    let t = a.time;
    let seed = (w.seed % 1000) as f32 * 0.37;
    // Gusts: now and then the wind picks up towards its gust speed.
    let g = 0.5 + 0.5 * noise(t, seed, [0.031, 0.083, 0.19]);
    let gust = (w.gust_kt - w.kt).max(0.0) * ((g * 1.6 - 0.6).max(0.0) / 1.0).min(1.0);
    let profile = height_profile(agl);
    let mean_kt = (w.kt + gust) * profile;
    let (e, n) = w.velocity();
    let dir = if w.kt > 0.01 { Vec2::new(e, n) / w.kt } else { Vec2::ZERO };
    let mut horiz = dir * mean_kt;
    // Storm outflow and downdrafts.
    let (east, north, _) = bevy_to_wu(pos.0);
    let map = ((east / 32.0) as i32, (0x8000 as f64 - north / 32.0) as i32);
    let (out, down, front) =
        storm_outflow(&state.weather, map.0, map.1, a.altitude, state.storm_built());
    horiz += out;
    state.outflow = out * KNOT_MS;
    // Turbulence: the background, the wind tumbling over the ground below a few thousand feet,
    // storm cells (inside the cloud, and weaker under it) and their gust fronts.
    let mechanical =
        0.5 * smoothstep(8.0, 40.0, w.kt + out.length()) * (1.0 - smoothstep(500.0, 3000.0, agl));
    let storm = if state.under_storm { if state.whiteout > 0.5 { 1.0 } else { 0.35 } } else { 0.0 };
    let turb = (w.turbulence + mechanical + storm + front).min(1.5);
    state.turb = turb;
    let s = seed + 11.0;
    let jolt = Vec3::new(
        noise(t, s, [0.37, 0.91, 2.3]) * TURB_KT.0,
        noise(t, s + 5.0, [0.29, 0.83, 2.9]) * TURB_KT.1,
        noise(t, s + 9.0, [0.41, 1.07, 2.6]) * TURB_KT.0,
    ) * turb;
    // Vertical gusts die out at the ground.
    let vertical = (jolt.y - down) * smoothstep(0.0, 300.0, agl);
    let kt = Vec3::new(horiz.x + jolt.x, vertical, -(horiz.y + jolt.z));
    state.air = kt * KNOT_MS;
    a.wind = kt * KNOT_WU;
    a.sea = state.sea;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wind_slows_near_the_ground() {
        assert!((height_profile(2000.0) - 1.0).abs() < 1e-3);
        assert!(height_profile(0.0) < 0.6);
        assert!(height_profile(30.0) < height_profile(500.0));
        assert!(height_profile(30_000.0) > 1.15);
    }

    #[test]
    fn storm_cells_blow_outwards() {
        use stealth_mission::{Sky, Weather};
        let w = Weather::from_sky(Sky::Storm, 77, [500; 8]);
        // Find a storm cell and stand at its edge, south of it.
        let c = (0..64)
            .flat_map(|i| w.clouds_near(i * 512, 16384, 4096).collect::<Vec<_>>())
            .find(|c| c.kind == CloudKind::Cumulonimbus)
            .expect("a storm cell");
        let (out, down, turb) = storm_outflow(&w, c.x, c.y + c.r, 200.0, 1.0);
        assert!(out.y < -OUTFLOW_KT * 0.9, "{out:?}"); // blowing south
        assert!(turb > 0.3, "{turb}");
        assert_eq!(down, 0.0);
        let (calm, ..) = storm_outflow(&w, c.x, c.y + c.r, 200.0, 0.0);
        assert_eq!(calm, Vec2::ZERO);
    }
}
