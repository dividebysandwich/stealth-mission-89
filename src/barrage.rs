//! Night barrage (`Effects::barrage`): in a conventional war (ROE 2) after dark the guns of the
//! towns and air defence sites near the aircraft fire blind into the sky, streams of tracer
//! arcing up and burning out, and searchlights sweep the sky over the towns. A town's guns only
//! fire with the other side's aircraft overhead (the player or friendly aircraft over an enemy
//! town, enemy aircraft over a friendly one; a neutral town's never), and the air defence guns
//! likewise by their side. Cosmetic only: the rounds hit nothing.
//!
//! Drones are the exception to the blind fire: an attack drone of the other side (an enemy
//! Geran over a friendly town, one of the player's side's drones over an enemy one) that comes
//! within `SPOT_RANGE` of a town's searchlight is caught by it, and the light follows it until it
//! is out of reach or explodes; the town's guns then fire at it, heavier than their blind
//! barrage, but unsure how far to lead it (`LEAD`) and always off by some way (`MISS`): their
//! streams hose about round it and never hit it. The drone defense missions have the towns' guns and lights whatever
//! the ROE, a campaign the lights and the guns firing at drones whatever its ROE.
//!
//! All the rounds are one line mesh rebuilt every frame (one draw); the searchlights are beam
//! cones (landing_lights.rs `BeamMaterial` with a negative intensity: lit analytically, not
//! marched).

use bevy::math::{DVec2, DVec3};
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;

use crate::combat::Combat;
use crate::landing_lights::BeamMaterial;
use crate::mfd::ContactKind;
use crate::mission::ActiveMission;
use crate::palette_material::{
    FLAG_LINE, LEVEL_AIRCRAFT, PaletteMaterial, PaletteParams, mesh_tag,
};
use crate::settings::{AppState, Settings};
use crate::world::{Theater, WU, WorldAnchor, WorldPos, map_to_wu, wu_to_bevy};
use stealth_sim::aaa::GroundKind;
use stealth_sim::oflags;

pub struct BarragePlugin;

impl Plugin for BarragePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Barrage>().add_systems(OnExit(AppState::Flight), teardown).add_systems(
            Update,
            (find_sites, fire, draw)
                .chain()
                .after(crate::combat::CombatSystems)
                .run_if(in_state(AppState::Flight)),
        );
    }
}

/// Guns firing at once at most (the nearest), and how far from the eye they fire (m).
const MAX_GUNS: usize = 10;
const GUN_RANGE: f64 = 15_000.0;
/// A town's guns fire at aircraft of the other side within this distance (m, horizontal).
const THREAT_RANGE: f64 = 8_000.0;
/// Searchlights at once at most, and how far from the eye they shine (m).
const MAX_LIGHTS: usize = 6;
const LIGHT_RANGE: f64 = 20_000.0;
/// Rounds in the air at most.
const MAX_ROUNDS: usize = 1200;
/// A burst: seconds firing, then pausing (ranges).
const BURST: (f32, f32) = (1.5, 4.5);
const PAUSE: (f32, f32) = (0.5, 3.0);
/// Seconds between rounds, muzzle velocity (m/s), seconds a tracer burns.
const RATE: (f32, f32) = (0.05, 0.09);
const SPEED: (f32, f32) = (750.0, 950.0);
const BURN: (f32, f32) = (2.5, 3.8);
/// Elevation of the guns (degrees) and how fast they traverse (radians per second).
const ELEVATION: (f32, f32) = (35.0, 75.0);
const TRAVERSE: f32 = 0.25;
/// The streak drawn behind each round: seconds of its travel.
const STREAK: f64 = 0.08;
/// Below this daylight (0 night .. 1 day) it is dark enough; full at a quarter of it.
const DUSK: f32 = 0.35;
/// The searchlights: beam length (m), half angle (degrees), lamp radius (m), brightness.
const BEAM_LENGTH: f32 = 4500.0;
const BEAM_ANGLE: f32 = 1.2;
const LAMP: f32 = 1.0;
const BEAM_BRIGHTNESS: f32 = 0.6;
/// A searchlight catches a drone this close (m, slant), loses it this far.
const SPOT_RANGE: f64 = 4000.0;
const LOSE_RANGE: f64 = 5000.0;
/// How fast a searchlight swings (radians per second, degrees per second): to the drone it
/// caught, or back to its sweep.
const SLEW: f32 = 0.9;
const SLEW_EL: f32 = 50.0;
/// A town's guns fire at a drone this close (m, slant).
const DRONE_GUN_RANGE: f64 = 4000.0;
/// A gun's guess at the lead: a slow wander through this range of the true lead (none .. more
/// than double).
const LEAD: (f64, f64) = (0.0, 2.3);
/// How far its stream passes the drone besides (m): a slow wander across the line of sight.
const MISS: (f64, f64) = (70.0, 160.0);
/// The rounds' drag (a fraction of their speed lost per second).
const DRAG: f64 = 0.08;
/// Firing at a drone: bursts and pauses (ranges, seconds), seconds between rounds.
const DRONE_BURST: (f32, f32) = (2.5, 6.0);
const DRONE_PAUSE: (f32, f32) = (0.2, 0.8);
const DRONE_RATE: (f32, f32) = (0.012, 0.025);
/// The campaign's attack drone types (the Geran-2 and -3).
const GERANS: [i32; 2] =
    [stealth_mission::drones::GERAN2 as i32, stealth_mission::drones::GERAN3 as i32];

/// A place guns fire from: towns (with a searchlight or two) and air defence units.
struct Site {
    pos: DVec3,
    /// The simulation's ground object nearest the town: its side is the town's.
    object: Option<usize>,
    lights: u8,
    gun: Gun,
    /// Each searchlight's azimuth (radians) and elevation (degrees), and the drone it follows
    /// (its flight unit).
    aim: [(f32, f32); 2],
    track: [Option<usize>; 2],
    /// Aimed (and swung) once: a light coming into view starts on its sweep.
    aimed: bool,
}

/// Azimuth (radians) and elevation (degrees).
type Aim = (f32, f32);

/// A drone in the air: its flight unit, position and velocity (Bevy world, m/s), side.
#[derive(Clone, Copy)]
struct Drone {
    unit: usize,
    pos: DVec3,
    vel: DVec3,
    blue: bool,
}

#[derive(Default, Clone, Copy)]
struct Gun {
    /// Firing (else pausing) and seconds left of it; seconds to the next round.
    firing: bool,
    left: f32,
    next: f32,
    /// Azimuth (radians) and elevation (degrees) aimed at, traverse direction.
    az: f32,
    el: f32,
    turn: f32,
}

struct Round {
    pos: DVec3,
    vel: DVec3,
    age: f32,
    burn: f32,
}

impl Round {
    /// Flies on `dt` seconds: falls, slows.
    fn fly(&mut self, dt: f32) {
        self.vel.y -= 9.81 * dt as f64;
        self.vel *= 1.0 - DRAG * dt as f64;
        self.pos += self.vel * dt as f64;
        self.age += dt;
    }
}

#[derive(Resource, Default)]
pub struct Barrage {
    /// None until the scenery is there; empty when there is no barrage this flight.
    sites: Option<Vec<Site>>,
    /// The air defence units' guns (`Sim::units` order).
    units: Vec<Gun>,
    rounds: Vec<Round>,
    rng: u32,
    /// The tracer mesh's entity and mesh; the searchlights' entities, their cone and material.
    tracers: Option<(Entity, Handle<Mesh>)>,
    lights: Vec<Entity>,
    beam: Option<(Handle<Mesh>, Handle<BeamMaterial>)>,
    /// Seconds of barrage (the searchlights' sweep).
    clock: f32,
    /// The drones in the air this frame.
    drones: Vec<Drone>,
    /// The guns fire blind at aircraft (else only at drones).
    blind: bool,
}

impl Barrage {
    /// A drone a searchlight follows (its flight unit).
    pub fn lit_drone(&self) -> Option<usize> {
        self.sites.as_ref()?.iter().flat_map(|s| s.track).flatten().next()
    }

    fn rand(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng >> 8) as f32 / 16_777_216.0
    }

    fn range(&mut self, (lo, hi): (f32, f32)) -> f32 {
        lo + (hi - lo) * self.rand()
    }
}

fn teardown(mut commands: Commands, mut barrage: ResMut<Barrage>) {
    if let Some((e, _)) = barrage.tracers.take() {
        commands.entity(e).try_despawn();
    }
    for e in barrage.lights.drain(..) {
        commands.entity(e).try_despawn();
    }
    *barrage = Barrage::default();
}

/// The towns' gun sites, once the scenery is there: one to three guns in a town or city, some
/// with searchlights; without generated towns, the theater's own.
fn find_sites(
    mut barrage: ResMut<Barrage>,
    mission: Option<Res<ActiveMission>>,
    scenery: Option<Res<crate::scenery::Scenery>>,
    theater: Option<Res<Theater>>,
    combat: Option<Res<Combat>>,
) {
    if barrage.sites.is_some() {
        return;
    }
    let (Some(scenery), Some(theater), Some(combat)) = (scenery, theater, combat) else { return };
    let war = mission.as_ref().is_some_and(|m| m.0.settings.roe == 2);
    barrage.blind = war || combat.sim.drones().is_some();
    if !barrage.blind && combat.sim.campaign().is_none() {
        barrage.sites = Some(Vec::new());
        return;
    }
    let seed = mission.as_ref().map_or(1, |m| m.0.weather.seed);
    barrage.rng = (seed as u32).wrapping_mul(0x9E37_79B9) | 1;
    let mut towns = scenery.towns();
    if towns.is_empty() {
        for o in &theater.world.objects {
            if o.target_flags & 0x100 == 0 && crate::fires::is_town_name(theater.world.name(o.name))
            {
                let (e, n) = map_to_wu(o.x, o.y);
                towns.push((DVec2::new(e, n) * WU, 800.0, true));
            }
        }
    }
    let mut sites = Vec::new();
    for (c, r, city) in towns {
        let guns = if city { 3 } else { 1 };
        for k in 0..guns {
            let off = DVec2::from_angle(barrage.rand() as f64 * std::f64::consts::TAU)
                * (r * 0.8 * barrage.rand() as f64);
            let p = c + off;
            let lights = match (city, k) {
                (true, 0) => 2,
                (true, 1) | (false, _) => (barrage.rand() < 0.5) as u8,
                _ => 0,
            };
            let gun = Gun { az: barrage.rand() * std::f32::consts::TAU, ..default() };
            let pos = wu_to_bevy(p.x / WU, p.y / WU, 0.0);
            let object = nearest_object(&combat, pos);
            let aim = [(0.0, 0.0); 2];
            sites.push(Site { pos, object, lights, gun, aim, track: [None; 2], aimed: false });
        }
    }
    debug!("barrage: {} gun sites", sites.len());
    barrage.sites = Some(sites);
}

/// The ground object nearest `pos` (not the first three, which are no places).
fn nearest_object(combat: &Combat, pos: DVec3) -> Option<usize> {
    let objects = combat.sim.objects().iter().enumerate().skip(3);
    objects
        .map(|(i, o)| {
            let p = crate::sim_render::map_point(o.x as f64 + 0.5, o.y as f64 + 0.5, 0.0);
            (horizontal(p, pos), i)
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, i)| i)
}

fn horizontal(a: DVec3, b: DVec3) -> f64 {
    (a.x - b.x).hypot(a.z - b.z)
}

/// How dark it is for the barrage: 0 (daylight) .. 1.
fn darkness(daylight: f32) -> f32 {
    (1.0 - (daylight - DUSK * 0.25) / (DUSK * 0.75)).clamp(0.0, 1.0)
}

/// Steps a gun: bursts and pauses, its traverse, the rounds it fires from `at`; aimed at
/// `target` (azimuth radians, elevation degrees) when it fires at a drone, else blind.
fn step_gun(b: &mut Barrage, gun: &mut Gun, at: DVec3, target: Option<(f32, f32)>, dt: f32) {
    gun.left -= dt;
    if gun.left <= 0.0 {
        gun.firing = !gun.firing;
        let (burst, pause) =
            if target.is_some() { (DRONE_BURST, DRONE_PAUSE) } else { (BURST, PAUSE) };
        gun.left = b.range(if gun.firing { burst } else { pause });
        if gun.firing {
            gun.el = b.range(ELEVATION);
            gun.turn = if b.rand() < 0.5 { -TRAVERSE } else { TRAVERSE } * b.range((0.3, 1.0));
        }
    }
    if !gun.firing {
        return;
    }
    if let Some((az, el)) = target {
        gun.az = az;
        gun.el = el;
    } else {
        gun.az += gun.turn * dt;
    }
    let rate = if target.is_some() { DRONE_RATE } else { RATE };
    gun.next -= dt;
    while gun.next <= 0.0 && b.rounds.len() < MAX_ROUNDS {
        gun.next += b.range(rate);
        // A little dispersion round the aim.
        let az = gun.az + (b.rand() - 0.5) * 0.04;
        let el = (gun.el + (b.rand() - 0.5) * 2.0).to_radians();
        let dir = Vec3::new(az.sin() * el.cos(), el.sin(), -az.cos() * el.cos());
        let speed = b.range(SPEED);
        let burn = b.range(BURN);
        // Fired a moment ago (between frames): already on its way.
        let age = -gun.next.min(0.0);
        let vel = (dir * speed).as_dvec3();
        b.rounds.push(Round { pos: at + vel * age as f64, vel, age, burn });
    }
}

#[allow(clippy::too_many_arguments)]
fn fire(
    mut barrage: ResMut<Barrage>,
    time: Res<Time>,
    settings: Res<Settings>,
    lighting: Res<crate::lighting::LightingState>,
    combat: Option<Res<Combat>>,
    contacts: Option<Res<crate::mfd::Contacts>>,
    anchor: Query<&WorldPos, With<WorldAnchor>>,
) {
    let b = &mut *barrage;
    let dt = time.delta_secs();
    b.clock += dt;
    // The rounds in the air fly on (and fall), and burn out.
    for r in &mut b.rounds {
        r.fly(dt);
    }
    b.rounds.retain(|r| r.age < r.burn);
    let Some(mut sites) = b.sites.take() else { return };
    let Ok(eye) = anchor.single().map(|p| p.0) else {
        b.sites = Some(sites);
        return;
    };
    if !sites.is_empty() {
        find_drones(b, combat.as_deref(), contacts.as_deref(), dt);
    }
    let objects = combat.as_deref().map_or(&[][..], |c| c.sim.objects());
    // A town's side (friendly: blue), None for a neutral one's.
    let side = |s: &Site| {
        let o = s.object.and_then(|o| objects.get(o))?;
        if o.flags & oflags::NEUTRAL != 0 && o.flags & oflags::FRIENDLY == 0 {
            return None;
        }
        Some(o.flags & oflags::FRIENDLY != 0)
    };
    for (i, s) in sites.iter_mut().enumerate() {
        if s.lights > 0 && s.pos.distance(eye) < LIGHT_RANGE {
            let blue = side(s);
            aim_lights(s, i, &b.drones, blue, b.clock, dt);
        } else {
            s.aimed = false;
            s.track = [None; 2];
        }
    }
    if settings.effects.barrage && darkness(lighting.daylight) > 0.0 && dt > 0.0 {
        // The aircraft in the air (the player's and others', friendly or not).
        let mut air: Vec<(DVec3, bool)> = vec![(eye, true)];
        if let Some(c) = &contacts {
            air.extend(
                c.list
                    .iter()
                    .filter(|c| matches!(c.kind, ContactKind::Aircraft | ContactKind::Drone))
                    .map(|c| (wu_to_bevy(c.east, c.north, 0.0), c.friendly)),
            );
        }
        // Aircraft of the other side than `friendly` near `at`.
        let threat = |at: DVec3, friendly: bool| {
            air.iter().any(|&(p, blue)| blue != friendly && horizontal(p, at) < THREAT_RANGE)
        };
        let threatened = |s: &Site| side(s).is_some_and(|blue| threat(s.pos, blue));
        // The nearest towns' guns and the air defence units' (the guns not destroyed), with the
        // other side's aircraft overhead; those firing at a drone first.
        let mut guns: Vec<(f64, usize, Option<Aim>)> = sites
            .iter()
            .enumerate()
            .map(|(i, s)| (s.pos.distance(eye), i))
            .filter(|&(d, i)| d < GUN_RANGE && threatened(&sites[i]))
            .map(|(d, i)| {
                let target = side(&sites[i]).and_then(|blue| drone_aim(b, &sites[i], blue));
                (if target.is_some() { d - GUN_RANGE } else { d }, i, target)
            })
            .filter(|g| b.blind || g.2.is_some())
            .collect();
        let units = combat.as_deref().map_or(&[][..], |c| c.sim.units());
        if !sites.is_empty() && b.blind {
            b.units.resize(units.len(), Gun::default());
            for (i, u) in units.iter().enumerate() {
                let p = crate::sim_render::map_point(u.x as f64 + 0.5, u.y as f64 + 0.5, 0.0);
                let gun = matches!(u.kind, GroundKind::Zsu | GroundKind::S60 | GroundKind::Gepard);
                let blue = combat.as_deref().is_some_and(|c| c.sim.aaa_blue(i));
                if gun && !u.destroyed && p.distance(eye) < GUN_RANGE && threat(p, blue) {
                    guns.push((p.distance(eye), sites.len() + i, None));
                }
            }
        }
        guns.sort_by(|a, b| a.0.total_cmp(&b.0));
        for &(_, i, target) in guns.iter().take(MAX_GUNS) {
            if let Some(s) = sites.get_mut(i) {
                step_gun(b, &mut s.gun, s.pos, target, dt);
            } else {
                let k = i - sites.len();
                let u = &units[k];
                let at = crate::sim_render::map_point(u.x as f64 + 0.5, u.y as f64 + 0.5, 0.0);
                let mut gun = b.units[k];
                step_gun(b, &mut gun, at, None, dt);
                b.units[k] = gun;
            }
        }
    }
    b.sites = Some(sites);
}

/// The drones in the air (the drone defense mission's and the campaign's attack drones, the
/// drone command centers'), their velocity from where they were last frame.
fn find_drones(
    b: &mut Barrage,
    combat: Option<&Combat>,
    contacts: Option<&crate::mfd::Contacts>,
    dt: f32,
) {
    let (Some(combat), Some(contacts)) = (combat, contacts) else {
        b.drones.clear();
        return;
    };
    let flights = combat.sim.flights();
    let old = std::mem::take(&mut b.drones);
    for c in &contacts.list {
        let Some(crate::mfd::ContactRef::Unit(unit)) = c.target else { continue };
        let drone = c.kind == ContactKind::Drone
            || flights.get(unit).is_some_and(|u| GERANS.contains(&u.spec))
            || combat
                .sim
                .campaign_flight(unit)
                .is_some_and(|f| f.role == stealth_sim::campaign::Role::Uav);
        if !drone {
            continue;
        }
        let pos = wu_to_bevy(c.east, c.north, c.alt as f64);
        let vel = match old.iter().find(|d| d.unit == unit) {
            Some(d) if dt > 0.0 => {
                let v = (pos - d.pos) / dt as f64;
                // A jump (a respawn, a resync) is no speed.
                if v.length() < 400.0 { d.vel.lerp(v, (dt as f64 * 3.0).min(1.0)) } else { d.vel }
            }
            Some(d) => d.vel,
            None => DVec3::ZERO,
        };
        b.drones.push(Drone { unit, pos, vel, blue: c.friendly });
    }
}

/// Azimuth (radians, from north clockwise) and elevation (degrees) from `from` to `to`.
fn look(from: DVec3, to: DVec3) -> (f32, f32) {
    let d = to - from;
    let az = d.x.atan2(-d.z) as f32;
    let el = d.y.atan2(d.x.hypot(d.z)).to_degrees() as f32;
    (az, el)
}

/// Swings a town's searchlights (town `i`, side `blue`): each follows the drone it caught (an
/// enemy's within `SPOT_RANGE`) until it is past `LOSE_RANGE` or gone, else sweeps the sky.
fn aim_lights(s: &mut Site, i: usize, drones: &[Drone], blue: Option<bool>, t: f32, dt: f32) {
    let hostile = |d: &&Drone| blue.is_some_and(|blue| d.blue != blue);
    for k in 0..s.lights.min(2) as usize {
        let at = light_pos(s.pos, k);
        let held = s.track[k]
            .and_then(|u| drones.iter().find(|d| d.unit == u))
            .filter(|d| d.pos.distance(at) < LOSE_RANGE);
        let caught = held.or_else(|| {
            drones
                .iter()
                .filter(hostile)
                .map(|d| (d.pos.distance(at), d))
                .filter(|&(r, _)| r < SPOT_RANGE)
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, d)| d)
        });
        s.track[k] = caught.map(|d| d.unit);
        let want = caught.map_or_else(|| sweep(t, light_seed(i, k)), |d| look(at, d.pos));
        if !s.aimed {
            s.aim[k] = want;
            continue;
        }
        let (az, el) = &mut s.aim[k];
        let turn = (want.0 - *az + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        *az = (*az + turn.clamp(-SLEW * dt, SLEW * dt)).rem_euclid(std::f32::consts::TAU);
        *el += (want.1 - *el).clamp(-SLEW_EL * dt, SLEW_EL * dt);
    }
    s.aimed = true;
}

/// Where a town's guns aim at the nearest enemy drone within `DRONE_GUN_RANGE` (town side
/// `blue`).
fn drone_aim(b: &Barrage, s: &Site, blue: bool) -> Option<Aim> {
    let d = b
        .drones
        .iter()
        .filter(|d| d.blue != blue)
        .filter(|d| d.pos.distance(s.pos) < DRONE_GUN_RANGE)
        .min_by(|a, b| a.pos.distance(s.pos).total_cmp(&b.pos.distance(s.pos)))?;
    // Each gun out of step with the others.
    let t = b.clock as f64 + s.pos.x * 0.013 + s.pos.z * 0.007;
    Some(look(s.pos, miss_point(s.pos, d, t)))
}

/// Where a gun at `at` aims at drone `d`, `t` seconds into its fire: ahead of it by a wandering
/// guess at the lead, and to one side of that.
fn miss_point(at: DVec3, d: &Drone, t: f64) -> DVec3 {
    let r = d.pos.distance(at);
    // The rounds' time of flight (slowed by drag) at the middle of their speed.
    let v = (SPEED.0 + SPEED.1) as f64 / 2.0;
    let fly = -(1.0 - (r * DRAG / v).min(0.9)).ln() / DRAG;
    let s = 0.5 + 0.35 * (t * 0.53).sin() + 0.15 * (t * 1.37 + 2.0).sin();
    let lead = LEAD.0 + (LEAD.1 - LEAD.0) * s;
    // Off across the line of sight, round and round.
    let sight = (d.pos - at).normalize_or(DVec3::Y);
    let u = sight.cross(DVec3::Y).normalize_or(DVec3::X);
    let w = u.cross(sight);
    let a = t * 0.41;
    let miss = MISS.0 + (MISS.1 - MISS.0) * (0.5 + 0.5 * (t * 0.83 + 1.0).sin());
    let side = u * a.cos() + w * a.sin();
    let mut off = d.vel * (fly * (lead - 1.0)) + side * miss;
    // The lead's error must not take the miss back: at least `MISS.0` off to the side.
    let across = off.dot(side);
    if across < MISS.0 {
        off += side * (MISS.0 - across);
    }
    // And raised for the rounds' drop.
    let drop = DVec3::Y * (0.5 * 9.81 * fly * fly);
    d.pos + d.vel * fly + off + drop
}

/// Searchlight `k` of a town at `pos`, and its sweep's seed.
fn light_pos(pos: DVec3, k: usize) -> DVec3 {
    pos + DVec3::new(k as f64 * 40.0, 0.0, 0.0)
}

fn light_seed(i: usize, k: usize) -> f32 {
    i as f32 * 1.7 + k as f32 * 2.3
}

/// The searchlights' sweep `t` seconds in: azimuth (radians) and elevation (degrees), each
/// light (`seed`) its own.
fn sweep(t: f32, seed: f32) -> (f32, f32) {
    let az = seed * 6.0 + 0.6 * (t * 0.11 + seed * 3.0).sin() + 0.25 * (t * 0.27 + seed).sin();
    let el = 55.0 + 15.0 * (t * 0.17 + seed * 5.0).sin() + 6.0 * (t * 0.41 + seed * 2.0).sin();
    (az, el)
}

#[allow(clippy::too_many_arguments)]
fn draw(
    mut commands: Commands,
    mut barrage: ResMut<Barrage>,
    settings: Res<Settings>,
    lighting: Res<crate::lighting::LightingState>,
    palette: Res<crate::palette::Palette>,
    anchor: Query<&WorldPos, (With<WorldAnchor>, Without<BarrageObject>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PaletteMaterial>>,
    mut beams: ResMut<Assets<BeamMaterial>>,
    mut q: Query<(&mut WorldPos, &mut Transform, &mut Visibility), With<BarrageObject>>,
) {
    let b = &mut *barrage;
    let Some(sites) = b.sites.as_ref().filter(|s| !s.is_empty()) else { return };
    let Ok(eye) = anchor.single().map(|p| p.0) else { return };
    let dark = if settings.effects.barrage { darkness(lighting.daylight) } else { 0.0 };

    // The tracers: a streak behind each round, relative to the eye.
    let (tracers, mesh) = b.tracers.get_or_insert_with(|| {
        let mesh = meshes.add(
            Mesh::new(PrimitiveTopology::LineList, bevy::asset::RenderAssetUsages::default())
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![Vec3::ZERO; 2]),
        );
        let material = materials.add(PaletteMaterial {
            params: PaletteParams {
                word: 0xFF00 | crate::sim_render::ENEMY_TRACER as u32,
                flags: FLAG_LINE,
                ..default()
            },
            globals: palette.globals.clone(),
        });
        let e = commands
            .spawn((
                BarrageObject,
                WorldPos::default(),
                Transform::default(),
                Visibility::Hidden,
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material),
                bevy::mesh::MeshTag(mesh_tag(LEVEL_AIRCRAFT, 0, true)),
                bevy::light::NotShadowCaster,
                bevy::camera::visibility::NoFrustumCulling,
            ))
            .id();
        (e, mesh)
    });
    let shown = dark > 0.0 && !b.rounds.is_empty();
    if let Ok((mut pos, _, mut vis)) = q.get_mut(*tracers) {
        pos.0 = eye;
        vis.set_if_neq(if shown { Visibility::Inherited } else { Visibility::Hidden });
    }
    if shown && let Some(mut m) = meshes.get_mut(&*mesh) {
        let mut v: Vec<Vec3> = Vec::with_capacity(b.rounds.len() * 2);
        for r in &b.rounds {
            // Never back past the muzzle (into the ground).
            let streak = STREAK.min(r.age.max(0.0) as f64);
            v.push((r.pos - eye).as_vec3());
            v.push((r.pos - r.vel * streak - eye).as_vec3());
        }
        m.insert_attribute(Mesh::ATTRIBUTE_POSITION, v);
    }

    // The searchlights of the nearest towns that have them.
    // Those following a drone first.
    let mut lit: Vec<(bool, f64, &Site)> = sites
        .iter()
        .filter(|s| s.lights > 0 && s.aimed)
        .map(|s| (s.track.iter().all(Option::is_none), s.pos.distance(eye), s))
        .filter(|&(_, d, _)| d < LIGHT_RANGE)
        .collect();
    lit.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
    let mut want: Vec<(DVec3, (f32, f32))> = Vec::new();
    for (_, _, s) in lit {
        for k in 0..s.lights.min(2) as usize {
            want.push((light_pos(s.pos, k), s.aim[k]));
        }
    }
    want.truncate(MAX_LIGHTS);
    if dark <= 0.0 {
        want.clear();
    }
    let (cone, material) = b.beam.get_or_insert_with(|| {
        let mesh = meshes.add(crate::landing_lights::cone_mesh(BEAM_LENGTH, BEAM_ANGLE, LAMP));
        let tan = BEAM_ANGLE.to_radians().tan();
        let params = Vec4::new(-BEAM_BRIGHTNESS, tan, LAMP, BEAM_LENGTH);
        (mesh, beams.add(BeamMaterial { params }))
    });
    // Dimmer at dusk (in eighths: a change re-uploads the material).
    let brightness = -BEAM_BRIGHTNESS * (dark * 8.0).ceil() / 8.0;
    if beams.get(&*material).is_some_and(|m| m.params.x != brightness)
        && let Some(mut m) = beams.get_mut(&*material)
    {
        m.params.x = brightness;
    }
    let (cone, material) = (cone.clone(), material.clone());
    while b.lights.len() < want.len() {
        let (mesh, material) = (cone.clone(), material.clone());
        let e = commands
            .spawn((
                BarrageObject,
                WorldPos::default(),
                Transform::default(),
                Visibility::Hidden,
                Mesh3d(mesh),
                MeshMaterial3d(material),
                bevy::light::NotShadowCaster,
                bevy::camera::visibility::NoFrustumCulling,
            ))
            .id();
        b.lights.push(e);
    }
    for (k, &e) in b.lights.iter().enumerate() {
        let Ok((mut pos, mut tf, mut vis)) = q.get_mut(e) else { continue };
        let Some(&(p, (az, el))) = want.get(k) else {
            vis.set_if_neq(Visibility::Hidden);
            continue;
        };
        pos.0 = p;
        tf.rotation = Quat::from_rotation_y(-az) * Quat::from_rotation_x(el.to_radians());
        vis.set_if_neq(Visibility::Inherited);
    }
}

/// The barrage's entities (placed from their `WorldPos` like the other world objects).
#[derive(Component)]
pub struct BarrageObject;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_only_after_dusk() {
        assert_eq!(darkness(1.0), 0.0);
        assert_eq!(darkness(DUSK), 0.0);
        assert_eq!(darkness(0.0), 1.0);
    }

    #[test]
    fn searchlights_follow_enemy_drones() {
        let mut s = Site {
            pos: DVec3::ZERO,
            object: None,
            lights: 1,
            gun: Gun::default(),
            aim: [(0.0, 0.0); 2],
            track: [None; 2],
            aimed: false,
        };
        let drone = |unit, x: f64, blue| Drone {
            unit,
            pos: DVec3::new(x, 500.0, -1000.0),
            vel: DVec3::ZERO,
            blue,
        };
        // A friendly drone is left alone, an enemy one caught and followed.
        aim_lights(&mut s, 0, &[drone(1, 0.0, true)], Some(true), 0.0, 0.1);
        assert_eq!(s.track[0], None);
        aim_lights(&mut s, 0, &[drone(1, 0.0, true), drone(2, 0.0, false)], Some(true), 0.0, 0.1);
        assert_eq!(s.track[0], Some(2));
        for _ in 0..100 {
            aim_lights(&mut s, 0, &[drone(2, 0.0, false)], Some(true), 0.0, 0.1);
        }
        let (az, el) = look(DVec3::ZERO, drone(2, 0.0, false).pos);
        assert!((s.aim[0].0 - az.rem_euclid(std::f32::consts::TAU)).abs() < 0.01);
        assert!((s.aim[0].1 - el).abs() < 0.1);
        // Held past the catching range, lost past the losing range, and when it is gone.
        aim_lights(&mut s, 0, &[drone(2, 4000.0, false)], Some(true), 0.0, 0.1);
        assert_eq!(s.track[0], Some(2));
        aim_lights(&mut s, 0, &[drone(2, 6000.0, false)], Some(true), 0.0, 0.1);
        assert_eq!(s.track[0], None);
        aim_lights(&mut s, 0, &[drone(2, 0.0, false)], Some(true), 0.0, 0.1);
        aim_lights(&mut s, 0, &[], Some(true), 0.0, 0.1);
        assert_eq!(s.track[0], None);
        // A neutral town's lights catch nothing.
        aim_lights(&mut s, 0, &[drone(2, 0.0, false)], None, 0.0, 0.1);
        assert_eq!(s.track[0], None);
    }

    #[test]
    fn rounds_miss_drones() {
        let at = DVec3::ZERO;
        for (k, &(x, z, vx)) in
            [(0.0, -2000.0, 50.0), (800.0, -1500.0, -50.0), (3000.0, 0.0, 0.0)].iter().enumerate()
        {
            let d = Drone {
                unit: 0,
                pos: DVec3::new(x, 600.0, z),
                vel: DVec3::new(vx, 0.0, 10.0),
                blue: false,
            };
            for i in 0..200 {
                let t = i as f64 * 0.37 + k as f64 * 11.0;
                let (az, el) = look(at, miss_point(at, &d, t));
                let el = el.to_radians();
                let dir = Vec3::new(az.sin() * el.cos(), el.sin(), -az.cos() * el.cos());
                for speed in [SPEED.0, SPEED.1] {
                    let mut r =
                        Round { pos: at, vel: (dir * speed).as_dvec3(), age: 0.0, burn: 9.0 };
                    let mut closest = f64::MAX;
                    while r.age < BURN.1 {
                        r.fly(1.0 / 120.0);
                        closest = closest.min(r.pos.distance(d.pos + d.vel * r.age as f64));
                    }
                    assert!(closest > 25.0, "t {t} speed {speed}: {closest:.0} m");
                }
            }
        }
    }

    #[test]
    fn searchlights_point_up() {
        for k in 0..20 {
            let (_, el) = sweep(k as f32 * 3.7, k as f32);
            assert!((30.0..80.0).contains(&el), "{el}");
        }
    }
}
