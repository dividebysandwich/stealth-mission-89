//! Combat flashes (`Effects::flashes`): flak bursts, explosions, SAM launches and the player's own
//! missile launches light what is around them for a moment, the clouds and the ground most of
//! all at night. The brightest burst as seen from the eye is the one lit: a point light in the
//! world shader (`Globals::burst`, palette.wgsl `burst_glow`), the volumetric clouds' line light
//! while lightning leaves it free (weather.rs), and the cockpit's ambient light (lighting.rs).
//! The gun's muzzle flash lights nothing.
//!
//! `--flak <m>` (testing): a flak burst every 1.5 s that far ahead of the aircraft.

use bevy::math::DVec3;
use bevy::prelude::*;

use crate::combat::{Combat, CombatSystems};
use crate::flight::{Aircraft, Player};
use crate::settings::{AppState, Settings};
use crate::world::WorldPos;

pub struct FlashesPlugin;

impl Plugin for FlashesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Bursts>()
            .add_systems(OnExit(AppState::Flight), clear)
            .add_systems(Update, update.after(CombatSystems).run_if(in_state(AppState::Flight)));
    }
}

/// One light: peak intensity (1 = a lightning stroke), seconds it lasts, reach (m), linear colour.
#[derive(Clone, Copy)]
struct Look {
    peak: f32,
    life: f32,
    reach: f32,
    colour: Vec3,
}

/// Orange-yellow of burning explosive, the whiter flash of a rocket motor lighting.
const FIRE: Vec3 = Vec3::new(1.0, 0.55, 0.18);
const MOTOR: Vec3 = Vec3::new(1.0, 0.8, 0.5);

const FLAK: Look = Look { peak: 1.0, life: 0.3, reach: 700.0, colour: FIRE };
const BOMB: Look = Look { peak: 1.6, life: 0.6, reach: 1500.0, colour: FIRE };
const KILL: Look = Look { peak: 1.2, life: 0.45, reach: 1000.0, colour: FIRE };
const SAM: Look = Look { peak: 0.9, life: 1.2, reach: 600.0, colour: MOTOR };
/// The player's missile leaving the rail, just below and ahead of the cockpit.
const OWN: Look = Look { peak: 1.0, life: 0.35, reach: 60.0, colour: MOTOR };
const OWN_OFFSET: Vec3 = Vec3::new(0.0, -1.5, -3.0);

/// The bursts alight and the one handed to the shaders.
#[derive(Resource, Default)]
pub struct Bursts {
    live: Vec<(DVec3, Look, f32)>,
    /// The brightest: position (Bevy world), intensity, colour and reach; None if all are out.
    pub light: Option<(DVec3, f32, Vec3, f32)>,
    /// Its light at the eye (0 .. about 2): the cockpit's share.
    pub cockpit: f32,
}

impl Bursts {
    /// `Globals::burst` / `burst_colour` (render space: relative to the floating origin).
    pub fn globals(&self, origin: DVec3) -> (Vec4, Vec4) {
        match self.light {
            Some((p, i, c, reach)) => ((p - origin).as_vec3().extend(i), c.extend(reach)),
            None => (Vec4::ZERO, Vec4::ZERO),
        }
    }
}

fn clear(mut bursts: ResMut<Bursts>) {
    *bursts = Bursts::default();
}

/// `--flak <m>`: test bursts this far ahead, every `TEST_EVERY` seconds, offset in turn.
fn test_flak() -> Option<f32> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == "--flak").and_then(|i| args.get(i + 1)?.parse().ok())
}
const TEST_EVERY: f32 = 1.5;
const TEST_OFFSETS: [Vec3; 4] = [
    Vec3::new(-0.15, 0.05, -1.0),
    Vec3::new(0.2, -0.1, -1.0),
    Vec3::new(0.05, 0.15, -1.0),
    Vec3::new(-0.25, -0.05, -1.0),
];

/// A burst's intensity `age` seconds in: a sharp flash dying away.
fn intensity(look: &Look, age: f32) -> f32 {
    let k = (1.0 - age / look.life).max(0.0);
    look.peak * k * k
}

/// The light a burst puts at `d` metres (as the shader's halo falls off).
fn light_at(look: &Look, i: f32, d: f64) -> f32 {
    let halo = d as f32 / (look.reach * 0.3);
    i / (1.0 + halo * halo)
}

/// The player's projectile slots (the enemy's are below).
const PLAYER_SLOTS: std::ops::Range<usize> = 8..stealth_sim::N_PROJ;

fn update(
    mut bursts: ResMut<Bursts>,
    mut flying: Local<[bool; stealth_sim::N_PROJ]>,
    mut test: Local<Option<(Option<f32>, f32, usize)>>,
    mut explosions: ResMut<crate::explosions::Explosions>,
    time: Res<Time>,
    settings: Res<Settings>,
    combat: Option<Res<Combat>>,
    player: Query<(&Aircraft, &WorldPos), With<Player>>,
) {
    let b = &mut *bursts;
    let dt = time.delta_secs();
    for (_, _, age) in &mut b.live {
        *age += dt;
    }
    b.live.retain(|(_, look, age)| *age < look.life);
    if let Some(c) = combat.as_deref()
        && settings.effects.flashes
    {
        for &(x, y, alt, kind) in &c.explosions {
            let look = match kind {
                crate::explosions::FLAK => FLAK,
                -6 => BOMB,
                // Gun rounds hitting: too small to light anything.
                -1 => continue,
                _ => KILL,
            };
            let p = crate::sim_render::map_point(x as f64 + 0.5, y as f64 + 0.5, alt.max(0) as f64);
            b.live.push((p, look, 0.0));
        }
        for &p in &c.sam_launches {
            b.live.push((p, SAM, 0.0));
        }
        // A missile in a player's slot that was free: just launched.
        let projs = c.sim.projectiles();
        let launched = PLAYER_SLOTS.clone().any(|s| {
            let m = projs[s];
            m.ttl != 0 && !flying[s] && crate::trails::is_missile(m.spec)
        });
        if launched && let Ok((a, pos)) = player.single() {
            b.live.push((pos.0 + (a.orientation * OWN_OFFSET).as_dvec3(), OWN, 0.0));
        }
    }
    if let Some(c) = combat.as_deref() {
        for (s, f) in flying.iter_mut().enumerate() {
            *f = c.sim.projectiles()[s].ttl != 0;
        }
    }
    let (test_m, clock, n) = test.get_or_insert_with(|| (test_flak(), 0.0, 0));
    if let Some(m) = *test_m
        && let Ok((a, pos)) = player.single()
    {
        *clock += dt;
        if *clock >= TEST_EVERY {
            *clock = 0.0;
            *n += 1;
            let p = pos.0 + (a.orientation * TEST_OFFSETS[*n % 4] * m).as_dvec3();
            explosions.pending.push((p, crate::explosions::Blast::Flak));
            b.live.push((p, FLAK, 0.0));
        }
    }
    let eye = player.single().map(|(_, p)| p.0).unwrap_or_default();

    // The brightest as seen from the eye: its light where the eye is, but distant ones still
    // count for the clouds and the ground they light.
    let seen = |(p, look, age): &(DVec3, Look, f32)| {
        let i = intensity(look, *age);
        i / (1.0 + (p.distance(eye) as f32 / (look.reach * 4.0)).powi(2))
    };
    let best = b.live.iter().max_by(|x, y| seen(x).total_cmp(&seen(y)));
    b.light = best.map(|&(p, look, age)| (p, intensity(&look, age), look.colour, look.reach));
    b.cockpit =
        best.map_or(0.0, |(p, look, age)| light_at(look, intensity(look, *age), p.distance(eye)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bursts_die_away() {
        assert_eq!(intensity(&FLAK, 0.0), FLAK.peak);
        assert!(intensity(&FLAK, FLAK.life * 0.5) < FLAK.peak * 0.3);
        assert_eq!(intensity(&FLAK, FLAK.life), 0.0);
        // Its light falls off well within its reach.
        assert!(light_at(&FLAK, 1.0, FLAK.reach as f64) < 0.1);
    }
}
