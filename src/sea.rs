//! Ships ride the sea: each tile ship heaves, pitches and rolls with the sea state
//! (`WeatherState::sea`), the more the smaller it is (`stealth_mission::wind::ship_motion`).
//! A carrier moves only a little, and its deck moves the same way for the flight model
//! (`Deck::surface`), so the deck the pilot sees is the deck the wheels meet.
//!
//! The ships' clock is the player's flight time (`Aircraft::time`), which the flight model's
//! sub-steps also go by.

use bevy::math::DVec3;
use bevy::prelude::*;
use stealth_mission::wind::{ShipMotion, ship_motion};

use crate::flight::{Aircraft, Player};
use crate::settings::AppState;
use crate::weather::WeatherState;
use crate::world::{FloatingOrigin, TileObject, WU, WorldPos};

pub struct SeaPlugin;

impl Plugin for SeaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SeaClock>()
            .add_systems(
                Update,
                tick.after(crate::flight::FlightSystems).run_if(in_state(AppState::Flight)),
            )
            .add_systems(
                PostUpdate,
                rock_ships
                    .after(crate::world::apply_world_positions)
                    .before(bevy::transform::TransformSystems::Propagate)
                    .run_if(in_state(AppState::Flight)),
            );
    }
}

/// A carrier's length (metres): its motion is worked out for this hull.
pub const CARRIER_LEN_M: f32 = 330.0;
/// Deck height of a carrier (world units; the flight model's and the simulation's).
pub const DECK_WU: f32 = 128.0;
/// A ship this close to a carrier's world object (map units) is that carrier.
const CARRIER_MATCH: f64 = 64.0;
/// An aircraft resting on a sinking deck stays on it while it is this close above it (wu).
pub const DECK_STICK_WU: f32 = 6.0;

/// The ships' clock (seconds).
#[derive(Resource, Default)]
pub struct SeaClock(pub f64);

/// A tile ship riding the sea.
#[derive(Component, Debug, Clone, Copy)]
pub struct Ship {
    pub len: f32,
    pub seed: u32,
}

/// A carrier's deck: the world object (map units) and the ship's pivot (wu east, north).
#[derive(Debug, Clone, Copy)]
pub struct Deck {
    pub x: i32,
    pub y: i32,
    pub east: f64,
    pub north: f64,
    pub seed: u32,
}

/// Seed of a ship's motion from a position (wu, rounded so both sides agree).
pub fn seed_at(east: f64, north: f64) -> u32 {
    let k = ((east / 64.0).round() as i64 as u64) << 32 ^ (north / 64.0).round() as i64 as u64;
    (stealth_mission::wind::mix(k) >> 32) as u32
}

impl Deck {
    /// The carriers' decks of a theater: each carrier world object with the tile ship drawn
    /// for it (its pivot), else the object itself.
    pub fn find(t: &crate::world::Theater) -> Vec<Deck> {
        let ships = t.ships();
        t.world
            .objects
            .iter()
            .filter(|o| o.target_flags as u16 & 0x200 != 0)
            .map(|o| {
                let near = ships
                    .iter()
                    .map(|s| {
                        let (x, y) = s.map_xy();
                        (s, ((x - o.x as i32) as f64).hypot((y - o.y as i32) as f64))
                    })
                    .filter(|&(_, d)| d <= CARRIER_MATCH)
                    .min_by(|a, b| a.1.total_cmp(&b.1));
                let (east, north) = match near {
                    Some((s, _)) => (s.east, s.north),
                    None => (o.x as f64 * 32.0, (0x8000 as f64 - o.y as f64) * 32.0),
                };
                Deck { x: o.x as i32, y: o.y as i32, east, north, seed: seed_at(east, north) }
            })
            .collect()
    }

    pub fn motion(&self, sea: f32, t: f64) -> ShipMotion {
        ship_motion(sea, CARRIER_LEN_M, self.seed, t)
    }

    /// The deck at a point (wu east, north): its height (wu), how fast it rises (wu/s) and its
    /// tilt there as (pitch up northwards, roll up eastwards) (rad).
    pub fn surface(&self, sea: f32, t: f64, east: f64, north: f64) -> (f32, f32, (f32, f32)) {
        let m = self.motion(sea, t);
        let (de, dn) = ((east - self.east) as f32, (north - self.north) as f32);
        let h = DECK_WU + m.heave / WU as f32 + dn * m.pitch.sin() + de * m.roll.sin();
        (h, m.heave_rate / WU as f32, (m.pitch, m.roll))
    }

    /// The deck's tilt for a model on it: pitch about east (bow north up), roll about north.
    pub fn tilt(&self, sea: f32, t: f64) -> Quat {
        let m = self.motion(sea, t);
        rotation(&m)
    }
}

/// The rotation of a ship's model for its motion (north end up by `pitch`, east side up by
/// `roll`).
fn rotation(m: &ShipMotion) -> Quat {
    Quat::from_rotation_x(m.pitch) * Quat::from_rotation_z(m.roll)
}

/// The deck under a point (wu east, north), if a carrier's: within its landing box.
pub fn deck_at(decks: &[Deck], east: f64, north: f64) -> Option<&Deck> {
    let (mx, my) = (east / 32.0, 32768.0 - north / 32.0);
    decks.iter().find(|d| (mx - d.x as f64).abs() <= 12.0 && (my - d.y as f64).abs() <= 38.0)
}

/// An aircraft's pitch and roll (rad) standing on a deck tilted (pitch northwards, roll
/// eastwards), for its heading (rad, clockwise from north): along the nose and right wing down.
pub fn slope_for_heading(tilt: (f32, f32), heading: f32) -> (f32, f32) {
    let (pitch, roll) = tilt;
    let fwd = (heading.sin(), heading.cos());
    let right = (fwd.1, -fwd.0);
    let along = roll * fwd.0 + pitch * fwd.1;
    let across = roll * right.0 + pitch * right.1;
    (along, -across)
}

fn tick(time: Res<Time>, mut clock: ResMut<SeaClock>, q: Query<&Aircraft, With<Player>>) {
    clock.0 = match q.single() {
        Ok(a) => a.time as f64,
        Err(_) => clock.0 + time.delta_secs_f64(),
    };
}

fn rock_ships(
    clock: Res<SeaClock>,
    weather: Res<WeatherState>,
    origin: Res<FloatingOrigin>,
    mut q: Query<(&WorldPos, &mut Transform, &Ship, &TileObject)>,
) {
    for (pos, mut tf, ship, tile) in &mut q {
        // A wreck lies still (on the bottom, or burning low in the water).
        let m = if tile.destroyed {
            ShipMotion::default()
        } else {
            ship_motion(weather.sea, ship.len, ship.seed, clock.0)
        };
        tf.translation = (pos.0 - origin.origin).as_vec3() + Vec3::Y * m.heave;
        tf.rotation = rotation(&m);
    }
}

/// Where a model standing on a carrier's deck at `p` (Bevy metres) is drawn: lifted with the
/// deck and tilted with it (none away from the decks).
pub fn on_deck(decks: &[Deck], sea: f32, t: f64, p: DVec3) -> Option<(f64, Quat)> {
    let (east, north, alt) = crate::world::bevy_to_wu(p);
    if !(DECK_WU as f64 - 4.0..DECK_WU as f64 + 40.0).contains(&alt) {
        return None;
    }
    let d = deck_at(decks, east, north)?;
    let (h, _, _) = d.surface(sea, t, east, north);
    Some(((h - DECK_WU) as f64 * WU, d.tilt(sea, t)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deck_rises_where_it_tilts_up() {
        let d = Deck { x: 0, y: 0, east: 0.0, north: 0.0, seed: 3 };
        for i in 0..50 {
            let t = i as f64 * 0.7;
            let (h0, _, (pitch, roll)) = d.surface(7.0, t, 0.0, 0.0);
            let (hn, ..) = d.surface(7.0, t, 0.0, 100.0);
            let (he, ..) = d.surface(7.0, t, 100.0, 0.0);
            assert!(((hn - h0) - 100.0 * pitch.sin()).abs() < 1e-3);
            assert!(((he - h0) - 100.0 * roll.sin()).abs() < 1e-3);
            // A carrier's deck stays within a few feet of its height.
            assert!((h0 - DECK_WU).abs() < 1.6 / WU as f32);
        }
        assert_eq!(d.surface(0.0, 3.0, 50.0, 50.0).0, DECK_WU);
    }

    #[test]
    fn slope_follows_heading() {
        // Deck up northwards: facing north the nose is up, facing east the right wing is down
        // (rolled right: up on the left, north).
        let (p, r) = slope_for_heading((0.01, 0.0), 0.0);
        assert!((p - 0.01).abs() < 1e-6 && r.abs() < 1e-6);
        let (p, r) = slope_for_heading((0.01, 0.0), std::f32::consts::FRAC_PI_2);
        assert!(p.abs() < 1e-6 && (r - 0.01).abs() < 1e-6, "{p} {r}");
    }
}
