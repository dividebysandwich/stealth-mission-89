//! Ships ride the sea. With the simulated ocean (`ocean`) each tile ship's hull follows the
//! waves under it: the water's height at points along its length and across its beam,
//! averaged (a long hull rides over the short waves), pulls its heave, pitch and roll like a
//! damped spring of the hull's own periods. Without it (the lighter option) each ship heaves,
//! pitches and rolls with the sea state by its length (`stealth_mission::wind::ship_motion`).
//! A carrier moves only a little either way, and its deck moves the same for the flight model
//! (`Decks`, `Deck::surface`), so the deck the pilot sees is the deck the wheels meet.
//!
//! The ships' clock is the player's flight time (`Aircraft::time`).

use bevy::math::DVec3;
use bevy::prelude::*;
use stealth_mission::wind::{ShipMotion, ship_motion};

use crate::flight::{Aircraft, Player};
use crate::ocean::Ocean;
use crate::settings::AppState;
use crate::weather::WeatherState;
use crate::world::{FloatingOrigin, TileObject, WU, WorldPos};

pub struct SeaPlugin;

/// The decks' motion for the frame (after the ocean's surface, before the flight models of
/// the next frame use it).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SeaSystems;

impl Plugin for SeaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SeaClock>()
            .init_resource::<Decks>()
            .add_systems(OnExit(AppState::Flight), |mut d: ResMut<Decks>| *d = Decks::default())
            .add_systems(
                Update,
                (tick.after(crate::flight::FlightSystems), ride_decks.in_set(SeaSystems))
                    .chain()
                    .run_if(in_state(AppState::Flight)),
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
/// A carrier moves at most about this much (heave m, pitch and roll rad), the limits easing in.
const CARRIER_CAP: (f32, f32, f32) = (1.5, 0.0175, 0.035);

/// The ships' clock (seconds).
#[derive(Resource, Default)]
pub struct SeaClock(pub f64);

/// Each carrier deck's (`Theater::decks`) motion now, and their hulls.
#[derive(Resource, Default)]
pub struct Decks {
    pub motion: Vec<ShipMotion>,
    hulls: Vec<Hull>,
}

/// A tile ship riding the sea.
#[derive(Component, Debug, Clone, Copy)]
pub struct Ship {
    pub len: f32,
    pub seed: u32,
    /// The carrier deck it is (`Theater::decks` index).
    pub deck: Option<usize>,
    pub hull: Hull,
}

impl Ship {
    pub fn new(len: f32, seed: u32, deck: Option<usize>) -> Self {
        Self { len, seed, deck, hull: Hull::default() }
    }
}

/// A hull's heave (m), pitch and roll (rad) and their rates, following the waves.
#[derive(Debug, Clone, Copy, Default)]
pub struct Hull {
    heave: f32,
    heave_v: f32,
    pitch: f32,
    pitch_v: f32,
    roll: f32,
    roll_v: f32,
}

impl Hull {
    /// Natural periods (s) of heave and pitch, and of roll, of a hull `len` metres long.
    fn periods(len: f32) -> (f32, f32) {
        let r = len.max(5.0).sqrt();
        ((0.6 * r).clamp(2.0, 12.0), (0.7 * r).clamp(2.5, 14.0))
    }

    /// Follows the waves' pull (`target`: heave, pitch, roll) for `dt` seconds.
    fn step(&mut self, target: (f32, f32, f32), len: f32, dt: f32) -> ShipMotion {
        let (t_hp, t_roll) = Self::periods(len);
        let (w_hp, w_roll) = (std::f32::consts::TAU / t_hp, std::f32::consts::TAU / t_roll);
        let steps = (dt / (1.0 / 60.0)).ceil().clamp(1.0, 8.0) as usize;
        let h = dt.min(0.2) / steps as f32;
        let spring = |x: &mut f32, v: &mut f32, to: f32, w: f32, zeta: f32| {
            for _ in 0..steps {
                *v += (w * w * (to - *x) - 2.0 * zeta * w * *v) * h;
                *x += *v * h;
            }
        };
        spring(&mut self.heave, &mut self.heave_v, target.0, w_hp, 0.6);
        spring(&mut self.pitch, &mut self.pitch_v, target.1, w_hp, 0.6);
        spring(&mut self.roll, &mut self.roll_v, target.2, w_roll, 0.2);
        ShipMotion {
            heave: self.heave,
            pitch: self.pitch,
            roll: self.roll,
            heave_rate: self.heave_v,
        }
    }
}

/// The waves' pull on a hull at (x, z) (absolute Bevy metres, the bow north): the mean height
/// over it, its slope along the length (north end up) and across the beam (east side up).
fn wave_pull(ocean: &Ocean, x: f64, z: f64, len: f32) -> (f32, f32, f32) {
    let beam = (len * 0.14).max(3.0);
    // A big hull doesn't feel the 10 m cascade's ripples.
    let w = [1.0, 1.0, if len < 60.0 { 1.0 } else { 0.0 }];
    let h = |dx: f32, dn: f32| ocean.height(x + dx as f64, z - dn as f64, w);
    let (l, b) = (len * 0.4, beam * 0.5);
    let (bow, stern) = (h(0.0, l), h(0.0, -l));
    let (east, west) = (h(b, 0.0), h(-b, 0.0));
    let mid = (h(0.0, 0.0) + h(0.0, l * 0.5) + h(0.0, -l * 0.5)) / 3.0;
    let heave = (2.0 * mid + bow + stern + east + west) / 6.0;
    ((heave), ((bow - stern) / (2.0 * l)).atan(), ((east - west) / (2.0 * b)).atan())
}

/// `x` eased into ±`cap`.
fn soft(x: f32, cap: f32) -> f32 {
    cap * (x / cap).tanh()
}

/// Seed of a ship's motion from a position (wu, rounded so both sides agree).
pub fn seed_at(east: f64, north: f64) -> u32 {
    let k = ((east / 64.0).round() as i64 as u64) << 32 ^ (north / 64.0).round() as i64 as u64;
    (stealth_mission::wind::mix(k) >> 32) as u32
}

/// A carrier's deck: the world object (map units), the ship's pivot (wu east, north) and the
/// tile object drawn for it (LOD, cell column, row, index), if there is one.
#[derive(Debug, Clone, Copy)]
pub struct Deck {
    pub x: i32,
    pub y: i32,
    pub east: f64,
    pub north: f64,
    pub seed: u32,
    pub tile: Option<(usize, i32, i32, u16)>,
}

impl Deck {
    /// The carriers' decks of a theater: each carrier world object with the tile object drawn
    /// for it (its pivot: the nearest of the finer levels' objects, whatever its model's class
    /// — the theaters' carriers are not all of the ship classes), else the object itself.
    pub fn find(t: &crate::world::Theater) -> Vec<Deck> {
        use stealth_formats::terrain::{GRID_DIM, LEVEL_UNIT};
        t.world
            .objects
            .iter()
            .filter(|o| o.target_flags as u16 & 0x200 != 0)
            .map(|o| {
                let mut best: Option<(f64, (usize, i32, i32, u16), f64, f64)> = None;
                for lod in 1..=2usize {
                    let unit = LEVEL_UNIT[lod];
                    let cell = 4096.0 * unit;
                    let centre = |c: i32| (c as f64 * 4096.0 + 2048.0) * unit;
                    // The cells round the object (it may sit near a cell's edge).
                    let (oe, on) = (o.x as f64 * 32.0, (0x8000 as f64 - o.y as f64) * 32.0);
                    let (cx, cy) = ((oe / cell).floor() as i32, (on / cell).floor() as i32);
                    for row in cy - 1..=cy + 1 {
                        for col in cx - 1..=cx + 1 {
                            if col < 0 || row < 0 || col >= GRID_DIM[lod] || row >= GRID_DIM[lod] {
                                continue;
                            }
                            let tile = t.grid.tile(lod, col, row) as usize;
                            let Some(objs) = t.tiles.tiles.get(lod).and_then(|l| l.get(tile))
                            else {
                                continue;
                            };
                            for (k, obj) in objs.iter().enumerate() {
                                let east = centre(col) + obj.x as f64 * unit;
                                let north = centre(row) + obj.y as f64 * unit;
                                let d = (east / 32.0 - o.x as f64)
                                    .hypot(0x8000 as f64 - north / 32.0 - o.y as f64);
                                if d <= CARRIER_MATCH && best.is_none_or(|b| d < b.0) {
                                    best = Some((d, (lod, col, row, k as u16), east, north));
                                }
                            }
                        }
                    }
                }
                let (east, north, tile) = match best {
                    Some((_, key, e, n)) => (e, n, Some(key)),
                    None => (o.x as f64 * 32.0, (0x8000 as f64 - o.y as f64) * 32.0, None),
                };
                Deck { x: o.x as i32, y: o.y as i32, east, north, seed: seed_at(east, north), tile }
            })
            .collect()
    }

    /// The deck at a point (wu east, north) for its motion `m`: its height (wu), how fast it
    /// rises (wu/s) and its tilt there as (pitch up northwards, roll up eastwards) (rad).
    pub fn surface(&self, m: &ShipMotion, east: f64, north: f64) -> (f32, f32, (f32, f32)) {
        let (de, dn) = ((east - self.east) as f32, (north - self.north) as f32);
        let h = DECK_WU + m.heave / WU as f32 + dn * m.pitch.sin() + de * m.roll.sin();
        (h, m.heave_rate / WU as f32, (m.pitch, m.roll))
    }
}

/// The rotation of a ship's model for its motion (north end up by `pitch`, east side up by
/// `roll`).
fn rotation(m: &ShipMotion) -> Quat {
    Quat::from_rotation_x(m.pitch) * Quat::from_rotation_z(m.roll)
}

/// The deck under a point (wu east, north), if a carrier's (within its landing box), and its
/// index.
pub fn deck_at(decks: &[Deck], east: f64, north: f64) -> Option<(usize, &Deck)> {
    let (mx, my) = (east / 32.0, 32768.0 - north / 32.0);
    decks
        .iter()
        .enumerate()
        .find(|(_, d)| (mx - d.x as f64).abs() <= 12.0 && (my - d.y as f64).abs() <= 38.0)
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

pub(crate) fn tick(
    time: Res<Time>,
    mut clock: ResMut<SeaClock>,
    q: Query<&Aircraft, With<Player>>,
) {
    clock.0 = match q.single() {
        Ok(a) => a.time as f64,
        Err(_) => clock.0 + time.delta_secs_f64(),
    };
}

/// The carriers' decks this frame: on the simulated waves, else by the sea state. The player's
/// aircraft gets them for its deck (`flight::ground_contact`).
fn ride_decks(
    time: Res<Time>,
    clock: Res<SeaClock>,
    weather: Res<WeatherState>,
    ocean: Res<Ocean>,
    theater: Option<Res<crate::world::Theater>>,
    mut decks: ResMut<Decks>,
    mut player: Query<&mut Aircraft, With<Player>>,
) {
    let Some(t) = theater else { return };
    let decks = &mut *decks;
    decks.hulls.resize(t.decks.len(), Hull::default());
    let dt = time.delta_secs();
    let live = ocean.on && ocean.frame.is_some();
    decks.motion = t
        .decks
        .iter()
        .zip(decks.hulls.iter_mut())
        .map(|(d, hull)| {
            if !live {
                return ship_motion(weather.sea, CARRIER_LEN_M, d.seed, clock.0);
            }
            let (x, z) = (d.east * WU, -d.north * WU);
            let pull = wave_pull(&ocean, x, z, CARRIER_LEN_M);
            let m = hull.step(pull, CARRIER_LEN_M, dt);
            let (h, p, r) = CARRIER_CAP;
            ShipMotion {
                heave: soft(m.heave, h),
                pitch: soft(m.pitch, p),
                roll: soft(m.roll, r),
                heave_rate: m.heave_rate * (1.0 - (m.heave / h).tanh().powi(2)),
            }
        })
        .collect();
    for mut a in &mut player {
        a.decks.clone_from(&decks.motion);
    }
}

fn rock_ships(
    time: Res<Time>,
    clock: Res<SeaClock>,
    weather: Res<WeatherState>,
    ocean: Res<Ocean>,
    decks: Res<Decks>,
    origin: Res<FloatingOrigin>,
    mut q: Query<(&WorldPos, &mut Transform, &mut Ship, &TileObject)>,
) {
    let live = ocean.on && ocean.frame.is_some();
    let dt = time.delta_secs();
    for (pos, mut tf, mut ship, tile) in &mut q {
        // A wreck lies still (on the bottom, or burning low in the water).
        let m = if tile.destroyed {
            ShipMotion::default()
        } else if let Some(m) = ship.deck.and_then(|i| decks.motion.get(i)) {
            *m
        } else if live {
            let len = ship.len;
            let pull = wave_pull(&ocean, pos.0.x, pos.0.z, len);
            ship.hull.step(pull, len, dt)
        } else {
            ship_motion(weather.sea, ship.len, ship.seed, clock.0)
        };
        let rot = rotation(&m);
        // A carrier turns about its flight deck, so the deck stays under what stands on it
        // (`on_deck`, the player's aircraft); other ships about their waterline.
        let pivot = if ship.deck.is_some() { Vec3::Y * DECK_WU * WU as f32 } else { Vec3::ZERO };
        tf.translation =
            (pos.0 - origin.origin).as_vec3() + Vec3::Y * m.heave + pivot - rot * pivot;
        tf.rotation = rot;
    }
}

/// Where a model standing on a carrier's deck at `p` (Bevy metres) is drawn: lifted with the
/// deck and tilted with it (none away from the decks).
pub fn on_deck(decks: &[Deck], motion: &[ShipMotion], p: DVec3) -> Option<(f64, Quat)> {
    let (east, north, alt) = crate::world::bevy_to_wu(p);
    if !(DECK_WU as f64 - 4.0..DECK_WU as f64 + 40.0).contains(&alt) {
        return None;
    }
    let (i, d) = deck_at(decks, east, north)?;
    let m = motion.get(i)?;
    let (h, _, _) = d.surface(m, east, north);
    Some(((h - DECK_WU) as f64 * WU, rotation(m)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deck_rises_where_it_tilts_up() {
        let d = Deck { x: 0, y: 0, east: 0.0, north: 0.0, seed: 3, tile: None };
        for i in 0..50 {
            let t = i as f64 * 0.7;
            let m = ship_motion(7.0, CARRIER_LEN_M, d.seed, t);
            let (h0, _, (pitch, roll)) = d.surface(&m, 0.0, 0.0);
            let (hn, ..) = d.surface(&m, 0.0, 100.0);
            let (he, ..) = d.surface(&m, 100.0, 0.0);
            assert!(((hn - h0) - 100.0 * pitch.sin()).abs() < 1e-3);
            assert!(((he - h0) - 100.0 * roll.sin()).abs() < 1e-3);
            // A carrier's deck stays within a few feet of its height.
            assert!((h0 - DECK_WU).abs() < 1.6 / WU as f32);
        }
        assert_eq!(d.surface(&ShipMotion::default(), 50.0, 50.0).0, DECK_WU);
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

    /// A hull settles on a steady pull, a small one faster than a big one.
    #[test]
    fn hull_follows_the_waves() {
        let (mut small, mut big) = (Hull::default(), Hull::default());
        let (mut s, mut b) = (ShipMotion::default(), ShipMotion::default());
        for _ in 0..120 {
            s = small.step((1.0, 0.05, 0.1), 30.0, 1.0 / 60.0);
            b = big.step((1.0, 0.05, 0.1), 330.0, 1.0 / 60.0);
        }
        assert!(s.heave > b.heave && s.heave > 0.5, "{s:?} {b:?}");
        for _ in 0..6000 {
            b = big.step((1.0, 0.05, 0.1), 330.0, 1.0 / 60.0);
        }
        assert!((b.heave - 1.0).abs() < 0.01 && (b.roll - 0.1).abs() < 0.01, "{b:?}");
        assert!(soft(10.0, 1.5) < 1.5 && (soft(0.1, 1.5) - 0.1).abs() < 1e-3);
    }
}

#[cfg(test)]
mod theater_tests {
    /// Every theater's carriers are drawn by a tile object, found whatever its model's class.
    #[test]
    fn carriers_have_their_tile_objects() {
        for id in ["persian_gulf", "cuba"] {
            let t = crate::world::Theater::load(id);
            assert!(!t.decks.is_empty(), "{id}");
            for d in &t.decks {
                assert!(d.tile.is_some(), "{id}: deck {d:?} has no tile object");
            }
        }
    }
}
