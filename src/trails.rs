//! Missile smoke trails and contrails. Each trail is a ribbon mesh of
//! segments along the emitter's path, rebuilt every frame; the trail shader
//! (assets/shaders/trail.wgsl) turns every segment to face the view drawing it. Segments widen
//! with age and start fading after `fade_after` seconds; a trail whose emitter is gone stays
//! until its last segment has faded.
//!
//! * Missiles (all projectiles but bombs and dropped equipment) trail smoke.
//! * Aircraft (AI and the player) above `CONTRAIL_VIEW_Z` leave two contrails.
//! * The damaged player aircraft trails smoke: grey, black when badly hit or on fire. It fades
//!   in behind the smoke puffs of [`crate::explosions`] and hangs in the air for a minute.
//! * So do the other pilots' (coop) missiles and damaged aircraft.
//! * Pulling hard (`Effects::vapour`) the player's wingtips trail short vortices of vapour,
//!   sooner in moist air than over the deserts.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::math::DVec3;
use bevy::mesh::{Indices, MeshVertexAttribute, MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    RenderPipelineDescriptor, SpecializedMeshPipelineError, VertexFormat,
};
use bevy::render::storage::ShaderBuffer;
use bevy::shader::ShaderRef;
use stealth_sim::fflags;

use crate::combat::{Combat, CombatSystems, unit_position};
use crate::flight::{Aircraft, Player};
use crate::settings::AppState;
use crate::sim_render::attitude;
use crate::world::{WorldPos, wu_to_bevy};

const SHADER: &str = "shaders/trail.wgsl";

/// Contrails form above this altitude (viewZ, the sim's compressed altitude; below 0x2000 it
/// equals feet). Patrolling fighters fly at the player's viewZ + 1000 (at least 5000), so they
/// leave contrails whenever the player is above 5000; interceptors chasing a high player and
/// aircraft climbing back to base from far away do too.
pub const CONTRAIL_VIEW_Z: i32 = 6000;

/// Trail direction at a point (per vertex).
pub(crate) const ATTRIBUTE_TANGENT: MeshVertexAttribute =
    MeshVertexAttribute::new("TrailTangent", 0x7A11_0001, VertexFormat::Float32x3);
/// Signed half width (metres) and alpha (per vertex).
pub(crate) const ATTRIBUTE_SIDE: MeshVertexAttribute =
    MeshVertexAttribute::new("TrailSide", 0x7A11_0002, VertexFormat::Float32x2);

pub struct TrailsPlugin;

impl Plugin for TrailsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<TrailMaterial>::default())
            .init_resource::<Trails>()
            .add_systems(OnExit(AppState::Flight), cleanup)
            .add_systems(OnExit(AppState::Menu), cleanup)
            .add_systems(
                Update,
                update_trails
                    .run_if(resource_exists::<Combat>)
                    .after(CombatSystems)
                    .run_if(in_state(AppState::Flight)),
            )
            .add_systems(
                Update,
                scene_contrails
                    .run_if(resource_exists::<crate::palette::Palette>)
                    .run_if(in_state(AppState::Menu)),
            );
    }
}

#[derive(Asset, TypePath, Clone, Debug)]
pub struct TrailMaterial {
    /// x: palette index, y: opacity.
    pub(crate) params: Vec4,
    globals: Handle<ShaderBuffer>,
}

crate::palette_material::globals_bind_group!(TrailMaterial, params);

impl Material for TrailMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            ATTRIBUTE_TANGENT.at_shader_location(1),
            ATTRIBUTE_SIDE.at_shader_location(2),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

/// Look of a kind of trail.
struct Style {
    /// Palette colour and opacity.
    colour: u8,
    opacity: f32,
    /// Half width at birth and when fully grown (metres), and the seconds it takes to grow.
    width: (f32, f32),
    grow: f32,
    /// Seconds before a segment starts fading, and how long the fade takes.
    fade_after: f32,
    fade: f32,
    /// A new segment starts every this many metres.
    spacing: f64,
    /// Alpha at the emitter (contrails form a little behind the engines).
    head_alpha: f32,
    /// Seconds a segment takes to fade in (0: born opaque).
    fade_in: f32,
}

const SMOKE: Style = Style {
    colour: 7,
    opacity: 0.75,
    width: (0.8, 6.0),
    grow: 20.0,
    fade_after: 0.0,
    fade: 20.0,
    spacing: 12.0,
    head_alpha: 1.0,
    fade_in: 0.0,
};

/// A strike missile's rocket booster (the Flamingo's off its ramp): a thick white column.
const BOOSTER_SMOKE: Style = Style {
    colour: 7,
    opacity: 0.85,
    width: (1.5, 16.0),
    grow: 25.0,
    fade_after: 15.0,
    fade: 45.0,
    spacing: 10.0,
    head_alpha: 1.0,
    fade_in: 0.0,
};

const CONTRAIL: Style = Style {
    colour: 15,
    opacity: 0.6,
    width: (1.0, 14.0),
    grow: 60.0,
    fade_after: 10.0,
    fade: 60.0,
    spacing: 30.0,
    head_alpha: 0.0,
    fade_in: 0.0,
};

/// Smoke of the damaged player aircraft: light and heavy damage (or fire).
const DAMAGE_SMOKE: Style = Style {
    colour: 7,
    opacity: 0.45,
    width: (1.5, 22.0),
    grow: 40.0,
    fade_after: 20.0,
    fade: 40.0,
    spacing: 20.0,
    head_alpha: 0.0,
    fade_in: 4.0,
};

const HEAVY_DAMAGE_SMOKE: Style = Style {
    colour: 8,
    opacity: 0.7,
    width: (2.5, 30.0),
    grow: 40.0,
    fade_after: 30.0,
    fade: 40.0,
    spacing: 20.0,
    head_alpha: 0.0,
    fade_in: 4.0,
};

/// Wingtip vortices: thin, short lived.
const VORTEX: Style = Style {
    colour: 15,
    opacity: 0.35,
    width: (0.2, 0.8),
    grow: 1.0,
    fade_after: 0.1,
    fade: 0.4,
    spacing: 8.0,
    head_alpha: 0.0,
    fade_in: 0.0,
};
/// G load (g) from which the wingtips trail vortices: moist air, the deserts' dry air.
const VORTEX_G: (f32, f32) = (4.5, 6.5);

/// The wingtips (aircraft frame, metres) of the flown jet (none for helicopters); a swing
/// wing's at its sweep `sweep` (0 forward .. 1 back).
fn wingtips(kind: crate::aircraft::AircraftKind, sweep: f32) -> Option<[Vec3; 2]> {
    use crate::aircraft::AircraftKind::*;
    // The C-5's high wing is some 6 m above its wheels.
    if kind == C5 {
        return Some([Vec3::new(-34.0, 6.4, 3.6), Vec3::new(34.0, 6.4, 3.6)]);
    }
    // The AC-130's straight high wing, some 5 m above its wheels.
    if kind == AC130 {
        return Some([Vec3::new(-20.3, 5.4, -1.8), Vec3::new(20.3, 5.4, -1.8)]);
    }
    let (x, z) = match kind {
        F117 => (6.6, 3.5),
        F16 => (4.7, 1.5),
        F22 => (6.8, 3.7),
        FA18E | EA18G => (6.6, 3.2),
        A10 => (8.7, 0.5),
        AV8B => (4.5, 1.6),
        Tornado => (6.9 - 2.6 * sweep, 1.0 + 1.5 * sweep),
        UH60 | MH60R | AH1Z | AH64 | RAH66 | OH58 | CH47 | C5 | AC130 => return None,
    };
    Some([Vec3::new(-x, 0.0, z), Vec3::new(x, 0.0, z)])
}

/// The AI's contrail emitters: left and right of the exhaust (aircraft frame, metres).
const ENGINES: [Vec3; 2] = [Vec3::new(-3.0, 0.0, 0.0), Vec3::new(3.0, 0.0, 0.0)];
/// The AI aircraft's exhaust: behind the aircraft centre (aircraft frame, metres).
const AI_EXHAUST: Vec3 = Vec3::new(0.0, 0.0, 8.0);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Source {
    /// Projectile slot and spec.
    Missile(usize, i32),
    /// Flight unit index and engine.
    Aircraft(usize, usize),
    /// Player engine.
    Player(usize),
    /// Player wingtip vortex.
    Vortex(usize),
    /// Damaged player aircraft (heavy smoke).
    PlayerSmoke(bool),
    /// Another pilot's missile: (pilot, slot) and spec.
    RemoteMissile((u8, u8), i32),
    /// Another pilot's damaged aircraft (heavy smoke).
    RemoteSmoke(u8, bool),
    /// An aircraft of the menu's scene (`Contrails`) and its engine.
    Scene(Entity, usize),
    /// A 70 mm rocket (its id) while its motor burns.
    Rocket(u16),
    /// A strike missile (flight unit) while its motor or booster burns.
    StrikeMissile(usize),
}

/// On an aircraft of the menu's scene: it trails contrails from these points (its frame,
/// metres; its `Transform`'s rotation turns them, not its scale).
#[derive(Component)]
pub struct Contrails(pub Vec<Vec3>);

/// A menu scene's missile or rocket: smoke from these points (model frame, m), as a missile's
/// in flight.
#[derive(Component)]
pub struct SceneSmoke(pub Vec<Vec3>);

/// The menu scene's contrails: wider than in flight, as they are only ever seen from far below,
/// and lasting longer.
const SCENE_CONTRAIL: Style = Style {
    colour: 15,
    opacity: 0.55,
    width: (3.0, 45.0),
    grow: 60.0,
    fade_after: 40.0,
    fade: 80.0,
    spacing: 150.0,
    head_alpha: 0.0,
    fade_in: 0.0,
};

/// The contrails of the menu scene's aircraft that have `Contrails`.
#[allow(clippy::too_many_arguments)]
fn scene_contrails(
    mut commands: Commands,
    mut trails: ResMut<Trails>,
    time: Res<Time>,
    palette: Res<crate::palette::Palette>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TrailMaterial>>,
    mut visibility: Query<&mut Visibility>,
    aircraft: Query<(Entity, &WorldPos, &Transform, &Contrails)>,
    smoke: Query<(Entity, &WorldPos, &Transform, &SceneSmoke)>,
) {
    // A scene's smoke keys after its contrails (`SMOKE_KEYS` on).
    const SMOKE_KEYS: usize = 64;
    let emitters = aircraft
        .iter()
        .flat_map(|(e, pos, tf, c)| {
            c.0.iter().enumerate().map(move |(k, &o)| {
                (Source::Scene(e, k), &SCENE_CONTRAIL, pos.0 + (tf.rotation * o).as_dvec3())
            })
        })
        .chain(smoke.iter().flat_map(|(e, pos, tf, c)| {
            c.0.iter().enumerate().map(move |(k, &o)| {
                (Source::Scene(e, SMOKE_KEYS + k), &SMOKE, pos.0 + (tf.rotation * o).as_dvec3())
            })
        }))
        .collect();
    let mut ctx = Ribbons {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        globals: &palette.globals,
        sim_objects: false,
    };
    for (e, drawn) in drive(&mut trails, emitters, time.elapsed_secs(), &mut ctx) {
        show(&mut visibility, e, drawn);
    }
}

/// What `drive` builds the ribbons with.
struct Ribbons<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    meshes: &'a mut Assets<Mesh>,
    materials: &'a mut Assets<TrailMaterial>,
    globals: &'a Handle<ShaderBuffer>,
    /// The ribbons are flight's (`sim_render::SimObject`).
    sim_objects: bool,
}

struct Trail {
    entity: Entity,
    mesh: Handle<Mesh>,
    style: &'static Style,
    /// World position of the mesh origin.
    anchor: DVec3,
    /// Committed points: (world position, birth time).
    points: Vec<(DVec3, f32)>,
    /// The emitter's position this frame (None once detached).
    head: Option<DVec3>,
}

impl Trail {
    fn alive(&self, now: f32) -> bool {
        self.head.is_some()
            || self
                .points
                .last()
                .is_some_and(|&(_, t)| now - t < self.style.fade_after + self.style.fade)
    }

    /// Drops faded points from the tail (keeping the last faded one as the ribbon's end).
    fn prune(&mut self, now: f32) {
        let life = self.style.fade_after + self.style.fade;
        let dead = self.points.iter().take_while(|&&(_, t)| now - t >= life).count();
        if dead > 1 {
            self.points.drain(..dead - 1);
        }
    }

    /// At least one segment to draw (an empty mesh is neither uploaded nor drawn).
    fn drawable(&self) -> bool {
        self.points.len() + self.head.is_some() as usize >= 2
    }

    fn rebuild(&self, mesh: &mut Mesh, now: f32) {
        let s = self.style;
        let mut pts: Vec<(DVec3, f32, f32)> = self
            .points
            .iter()
            .map(|&(p, t)| {
                let age = now - t;
                let mut alpha = 1.0 - ((age - s.fade_after) / s.fade).clamp(0.0, 1.0);
                if s.fade_in > 0.0 {
                    alpha *= (age / s.fade_in).clamp(0.0, 1.0);
                }
                let w = s.width.0 + (s.width.1 - s.width.0) * (age / s.grow).clamp(0.0, 1.0).sqrt();
                (p, w, alpha)
            })
            .collect();
        if let Some(h) = self.head {
            pts.push((h, s.width.0, s.head_alpha));
        }
        let n = pts.len();
        let mut pos = Vec::with_capacity(2 * n);
        let mut tan = Vec::with_capacity(2 * n);
        let mut side = Vec::with_capacity(2 * n);
        let mut idx = Vec::with_capacity(6 * n.saturating_sub(1));
        for i in 0..n {
            let (p, w, a) = pts[i];
            let d = (pts[(i + 1).min(n - 1)].0 - pts[i.saturating_sub(1)].0).as_vec3();
            let t = d.try_normalize().unwrap_or(Vec3::Z);
            let local = (p - self.anchor).as_vec3();
            pos.extend([local, local]);
            tan.extend([t, t]);
            side.extend([[-w, a], [w, a]]);
        }
        for i in 0..n as u32 - 1 {
            let k = 2 * i;
            idx.extend([k, k + 1, k + 2, k + 1, k + 3, k + 2]);
        }
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
        mesh.insert_attribute(ATTRIBUTE_TANGENT, tan);
        mesh.insert_attribute(ATTRIBUTE_SIDE, side);
        mesh.insert_indices(Indices::U32(idx));
    }
}

#[derive(Resource, Default)]
pub struct Trails {
    /// Trails with a live emitter.
    active: HashMap<Source, Trail>,
    /// Trails left behind, fading out.
    detached: Vec<Trail>,
    /// By colour and opacity.
    materials: Vec<((u8, u32), Handle<TrailMaterial>)>,
}

/// Missiles trail smoke; bombs (28..30, the CBU-105, the JSOW), the MALD's small jet and
/// dropped equipment (38) do not.
pub fn is_missile(spec: i32) -> bool {
    !matches!(
        spec,
        28..=30 | 38 | stealth_sim::CBU105_SPEC | stealth_sim::JSOW_SPEC | stealth_sim::MALD_SPEC
    )
}

fn cleanup(mut commands: Commands, mut trails: ResMut<Trails>) {
    trails.clear(&mut commands);
}

impl Trails {
    /// Every trail goes (a menu scene's at its end).
    pub fn clear(&mut self, commands: &mut Commands) {
        for t in self.active.values().chain(&self.detached) {
            commands.entity(t.entity).try_despawn();
        }
        *self = Trails::default();
    }
}

#[allow(clippy::too_many_arguments)]
fn update_trails(
    mut commands: Commands,
    combat: Res<Combat>,
    mut trails: ResMut<Trails>,
    time: Res<Time>,
    palette: Res<crate::palette::Palette>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TrailMaterial>>,
    player: Query<(&Aircraft, &WorldPos), With<Player>>,
    settings: Res<crate::settings::Settings>,
    mut visibility: Query<&mut Visibility>,
    theater: Option<Res<crate::world::Theater>>,
) {
    let trails = &mut *trails;
    let sim = &combat.sim;
    let alpha = combat.alpha;
    let now = time.elapsed_secs();

    // Emitter positions this frame.
    let mut emitters: Vec<(Source, &'static Style, DVec3)> = Vec::new();
    // Wingtip vortices pulling hard (sooner in moist air).
    if let Ok((a, pos)) = player.single()
        && settings.effects.vapour
        && !a.on_ground
        && a.ended.is_none()
        && let Some(tips) = wingtips(settings.aircraft, a.wing_sweep)
    {
        let dry = theater.as_ref().is_some_and(|t| crate::weather::dry_theater(&t.id));
        let from = if dry { VORTEX_G.1 } else { VORTEX_G.0 };
        if a.gees / 16.0 >= from {
            for (k, &o) in tips.iter().enumerate() {
                emitters.push((Source::Vortex(k), &VORTEX, pos.0 + (a.orientation * o).as_dvec3()));
            }
        }
    }
    let projs = sim.projectiles();
    for (s, &m) in projs.iter().enumerate() {
        // Smoke while the motor burns, from where the missile is drawn.
        if m.ttl == 0 || !is_missile(m.spec) || !m.motor_visible() {
            continue;
        }
        if let Some((pos, _)) = crate::sim_render::projectile_pose(&combat, s) {
            emitters.push((Source::Missile(s, m.spec), &SMOKE, pos));
        }
    }
    // The 70 mm rockets while their motors burn, where they are drawn.
    for r in sim.rockets().rockets.iter().filter(|r| r.burning()) {
        emitters.push((Source::Rocket(r.id), &SMOKE, crate::rockets::drawn_at(&combat, r)));
    }
    // Strike missiles with their motor (or the Flamingo's booster) burning.
    for (i, u, _) in sim.active_aircraft() {
        let Some((true, booster)) = sim.strike_missile_motor(i) else { continue };
        let prev = combat.prev_flights.get(i);
        let (east, north, alt) = unit_position(prev, &u, alpha);
        let style = if booster { &BOOSTER_SMOKE } else { &SMOKE };
        emitters.push((Source::StrikeMissile(i), style, wu_to_bevy(east, north, alt)));
    }
    for (i, u, _) in sim.active_aircraft() {
        if u.flags & fflags::DESTROYED != 0 || (u.wx == 0 && u.wy == 0) || u.alt < CONTRAIL_VIEW_Z {
            continue;
        }
        let prev = combat.prev_flights.get(i);
        let (east, north, alt) = unit_position(prev, &u, alpha);
        let rot = attitude(u.heading, u.pitch, u.bank);
        for (k, &o) in ENGINES.iter().enumerate() {
            emitters.push((
                Source::Aircraft(i, k),
                &CONTRAIL,
                wu_to_bevy(east, north, alt) + (rot * (AI_EXHAUST + o)).as_dvec3(),
            ));
        }
    }
    let exhaust = settings.aircraft.spec().exhaust;
    // The AC-130's turboprops only leave them high up, far above its orbits.
    let contrails_from =
        if settings.aircraft.spectre() { 2 * CONTRAIL_VIEW_Z } else { CONTRAIL_VIEW_Z };
    if let Ok((a, pos)) = player.single()
        && !a.on_ground
        && sim.player().view_z >= contrails_from
    {
        for (k, o) in crate::aircraft::contrail_emitters(settings.aircraft).into_iter().enumerate()
        {
            let at = pos.0 + (a.orientation * o).as_dvec3();
            emitters.push((Source::Player(k), &CONTRAIL, at));
        }
    }
    let d = sim.damage_effects();
    if let Ok((a, pos)) = player.single()
        && d.bits != 0
        && a.ended.is_none()
    {
        let heavy = d.bits & stealth_sim::dmg::FIRE != 0 || d.hits >= 6;
        emitters.push((
            Source::PlayerSmoke(heavy),
            if heavy { &HEAVY_DAMAGE_SMOKE } else { &DAMAGE_SMOKE },
            pos.0 + (a.orientation * exhaust).as_dvec3(),
        ));
    }

    // The other pilots' missiles and damaged aircraft.
    let me = sim.active_pilot();
    for p in sim.remote_pilots().iter().filter(|p| p.id != me) {
        for &(slot, m) in &p.missiles {
            if is_missile(m.spec) && m.motor_visible() {
                let key = (p.id, slot);
                let (pos, _) = crate::sim_render::remote_missile_pose(&combat, key, &m);
                emitters.push((Source::RemoteMissile(key, m.spec), &SMOKE, pos));
            }
        }
        if p.smoke > 0 && p.chute.is_none() {
            let heavy = p.smoke >= 2;
            let (pos, rot) = crate::sim_render::remote_pose(&combat, p);
            emitters.push((
                Source::RemoteSmoke(p.id, heavy),
                if heavy { &HEAVY_DAMAGE_SMOKE } else { &DAMAGE_SMOKE },
                pos + (rot * crate::aircraft::AircraftKind::of_remote(p).spec().exhaust).as_dvec3(),
            ));
        }
    }

    let mut ctx = Ribbons {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        globals: &palette.globals,
        sim_objects: true,
    };
    for (e, drawn) in drive(trails, emitters, now, &mut ctx) {
        show(&mut visibility, e, drawn);
    }
}

/// Shows a ribbon once it has a segment (empty meshes upset some GPU drivers).
fn show(visibility: &mut Query<&mut Visibility>, e: Entity, drawn: bool) {
    if let Ok(mut v) = visibility.get_mut(e) {
        v.set_if_neq(if drawn { Visibility::Inherited } else { Visibility::Hidden });
    }
}

/// Grows the trails of this frame's emitters, detaches those whose emitter is gone and
/// rebuilds the ribbons: each ribbon and whether it is drawn.
fn drive(
    trails: &mut Trails,
    emitters: Vec<(Source, &'static Style, DVec3)>,
    now: f32,
    ctx: &mut Ribbons,
) -> Vec<(Entity, bool)> {
    // Detach the trails whose emitter is gone.
    let live: Vec<Source> = emitters.iter().map(|e| e.0).collect();
    let gone: Vec<Source> = trails.active.keys().filter(|k| !live.contains(k)).copied().collect();
    for k in gone {
        let mut t = trails.active.remove(&k).unwrap();
        if let Some(h) = t.head.take() {
            t.points.push((h, now));
        }
        trails.detached.push(t);
    }

    for (src, style, p) in emitters {
        if !trails.active.contains_key(&src) {
            let key = (style.colour, style.opacity.to_bits());
            let material = match trails.materials.iter().find(|(k, _)| *k == key) {
                Some((_, m)) => m.clone(),
                None => {
                    let m = ctx.materials.add(TrailMaterial {
                        params: Vec4::new(style.colour as f32, style.opacity, 0.0, 0.0),
                        globals: ctx.globals.clone(),
                    });
                    trails.materials.push((key, m.clone()));
                    m
                }
            };
            let mesh = ctx
                .meshes
                .add(Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default()));
            let mut entity = ctx.commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material),
                WorldPos(p),
                Transform::default(),
                // Shown once the ribbon has a segment (empty meshes upset some GPU drivers).
                Visibility::Hidden,
                NoFrustumCulling,
                bevy::light::NotShadowCaster,
            ));
            if ctx.sim_objects {
                entity.insert(crate::sim_render::SimObject);
            }
            let entity = entity.id();
            trails.active.insert(
                src,
                Trail { entity, mesh, style, anchor: p, points: vec![(p, now)], head: None },
            );
        }
        let t = trails.active.get_mut(&src).unwrap();
        if t.points.last().is_none_or(|&(q, _)| (p - q).length() >= style.spacing) {
            t.points.push((p, now));
        }
        t.head = Some(p);
    }

    trails.detached.retain(|t| {
        let keep = t.alive(now);
        if !keep {
            ctx.commands.entity(t.entity).try_despawn();
        }
        keep
    });
    let mut shown = Vec::new();
    for t in trails.active.values_mut().chain(trails.detached.iter_mut()) {
        t.prune(now);
        let drawn = t.drawable()
            && ctx.meshes.get_mut(&t.mesh).map(|mut mesh| t.rebuild(&mut mesh, now)).is_some();
        shown.push((t.entity, drawn));
    }
    shown
}
