//! Night barrage (`Effects::barrage`): in a conventional war (ROE 2) after dark the guns of the
//! towns and air defence sites near the aircraft fire blind into the sky, streams of tracer
//! arcing up and burning out, and searchlights sweep the sky over the towns. A town's guns only
//! fire with the other side's aircraft overhead (the player or friendly aircraft over an enemy
//! town, enemy aircraft over a friendly one; a neutral town's never), and the air defence guns
//! likewise by their side. Cosmetic only: the rounds hit nothing and the searchlights find
//! nothing.
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
use stealth_sim::aaa::AaaKind;
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
const MAX_GUNS: usize = 8;
const GUN_RANGE: f64 = 15_000.0;
/// A town's guns fire at aircraft of the other side within this distance (m, horizontal).
const THREAT_RANGE: f64 = 8_000.0;
/// Searchlights at once at most, and how far from the eye they shine (m).
const MAX_LIGHTS: usize = 6;
const LIGHT_RANGE: f64 = 20_000.0;
/// Rounds in the air at most.
const MAX_ROUNDS: usize = 600;
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
const BEAM_LENGTH: f32 = 3500.0;
const BEAM_ANGLE: f32 = 1.2;
const LAMP: f32 = 1.0;
const BEAM_BRIGHTNESS: f32 = 0.6;

/// A place guns fire from: towns (with a searchlight or two) and air defence units.
struct Site {
    pos: DVec3,
    /// The simulation's ground object nearest the town: its side is the town's.
    object: Option<usize>,
    lights: u8,
    gun: Gun,
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

#[derive(Resource, Default)]
pub struct Barrage {
    /// None until the scenery is there; empty when there is no barrage this flight.
    sites: Option<Vec<Site>>,
    /// The air defence units' guns (`Sim::aaa_units` order).
    units: Vec<Gun>,
    rounds: Vec<Round>,
    rng: u32,
    /// The tracer mesh's entity and mesh; the searchlights' entities, their cone and material.
    tracers: Option<(Entity, Handle<Mesh>)>,
    lights: Vec<Entity>,
    beam: Option<(Handle<Mesh>, Handle<BeamMaterial>)>,
    /// Seconds of barrage (the searchlights' sweep).
    clock: f32,
}

impl Barrage {
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
    let war = mission.as_ref().is_some_and(|m| m.0.settings.roe == 2);
    if !war {
        barrage.sites = Some(Vec::new());
        return;
    }
    let (Some(scenery), Some(theater), Some(combat)) = (scenery, theater, combat) else { return };
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
            sites.push(Site { pos, object, lights, gun });
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

/// Steps a gun: bursts and pauses, its traverse, the rounds it fires from `at`.
fn step_gun(b: &mut Barrage, gun: &mut Gun, at: DVec3, dt: f32) {
    gun.left -= dt;
    if gun.left <= 0.0 {
        gun.firing = !gun.firing;
        gun.left = b.range(if gun.firing { BURST } else { PAUSE });
        if gun.firing {
            gun.el = b.range(ELEVATION);
            gun.turn = if b.rand() < 0.5 { -TRAVERSE } else { TRAVERSE } * b.range((0.3, 1.0));
        }
    }
    if !gun.firing {
        return;
    }
    gun.az += gun.turn * dt;
    gun.next -= dt;
    while gun.next <= 0.0 && b.rounds.len() < MAX_ROUNDS {
        gun.next += b.range(RATE);
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
        r.vel.y -= 9.81 * dt as f64;
        r.vel *= 1.0 - 0.08 * dt as f64;
        r.pos += r.vel * dt as f64;
        r.age += dt;
    }
    b.rounds.retain(|r| r.age < r.burn);
    let Some(mut sites) = b.sites.take() else { return };
    let Ok(eye) = anchor.single().map(|p| p.0) else {
        b.sites = Some(sites);
        return;
    };
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
        let objects = combat.as_deref().map_or(&[][..], |c| c.sim.objects());
        // Aircraft of the other side than `friendly` near `at`.
        let threat = |at: DVec3, friendly: bool| {
            air.iter().any(|&(p, blue)| blue != friendly && horizontal(p, at) < THREAT_RANGE)
        };
        let threatened = |s: &Site| {
            let Some(o) = s.object.and_then(|o| objects.get(o)) else { return false };
            if o.flags & oflags::NEUTRAL != 0 && o.flags & oflags::FRIENDLY == 0 {
                return false;
            }
            threat(s.pos, o.flags & oflags::FRIENDLY != 0)
        };
        // The nearest towns' guns and the air defence units' (the guns not destroyed), with the
        // other side's aircraft overhead.
        let mut guns: Vec<(f64, usize)> = sites
            .iter()
            .enumerate()
            .map(|(i, s)| (s.pos.distance(eye), i))
            .filter(|&(d, i)| d < GUN_RANGE && threatened(&sites[i]))
            .collect();
        let units = combat.as_deref().map_or(&[][..], |c| c.sim.aaa_units());
        if !sites.is_empty() {
            b.units.resize(units.len(), Gun::default());
            for (i, u) in units.iter().enumerate() {
                let p = crate::sim_render::map_point(u.x as f64 + 0.5, u.y as f64 + 0.5, 0.0);
                let gun = matches!(u.kind, AaaKind::Zsu | AaaKind::S60 | AaaKind::Gepard);
                let blue = combat.as_deref().is_some_and(|c| c.sim.aaa_blue(i));
                if gun && !u.destroyed && p.distance(eye) < GUN_RANGE && threat(p, blue) {
                    guns.push((p.distance(eye), sites.len() + i));
                }
            }
        }
        guns.sort_by(|a, b| a.0.total_cmp(&b.0));
        for &(_, i) in guns.iter().take(MAX_GUNS) {
            if let Some(s) = sites.get_mut(i) {
                step_gun(b, &mut s.gun, s.pos, dt);
            } else {
                let k = i - sites.len();
                let u = &units[k];
                let at = crate::sim_render::map_point(u.x as f64 + 0.5, u.y as f64 + 0.5, 0.0);
                let mut gun = b.units[k];
                step_gun(b, &mut gun, at, dt);
                b.units[k] = gun;
            }
        }
    }
    b.sites = Some(sites);
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
    let mut lit: Vec<(f64, DVec3, u8, usize)> = sites
        .iter()
        .enumerate()
        .filter(|(_, s)| s.lights > 0)
        .map(|(i, s)| (s.pos.distance(eye), s.pos, s.lights, i))
        .filter(|&(d, ..)| d < LIGHT_RANGE)
        .collect();
    lit.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut want: Vec<(DVec3, f32)> = Vec::new();
    for (_, p, n, i) in lit {
        for k in 0..n {
            want.push((p + DVec3::new(k as f64 * 40.0, 0.0, 0.0), i as f32 * 1.7 + k as f32 * 2.3));
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
    let t = b.clock;
    for (k, &e) in b.lights.iter().enumerate() {
        let Ok((mut pos, mut tf, mut vis)) = q.get_mut(e) else { continue };
        let Some(&(p, seed)) = want.get(k) else {
            vis.set_if_neq(Visibility::Hidden);
            continue;
        };
        let (az, el) = sweep(t, seed);
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
    fn searchlights_point_up() {
        for k in 0..20 {
            let (_, el) = sweep(k as f32 * 3.7, k as f32);
            assert!((30.0..80.0).contains(&el), "{el}");
        }
    }
}
