//! What the scenes share: spawning their models, finding their places on the look's theater
//! (airfields, SAM sites, a carrier), flight paths that are a function of the scene's time (so
//! a scene can start part way in) and the camera's moves.

use bevy::ecs::system::SystemParam;
use bevy::math::{DVec2, DVec3};
use bevy::prelude::*;

use super::director::Director;
use super::{SceneCamera, ShotEntity};
use crate::aircraft::AircraftKind;
use crate::models::{ModelAssets, ModelLibrary, ModelState};
use crate::palette_material::LEVEL_AIRCRAFT;
use crate::world::{Theater, WorldPos};

/// The model files the scenes draw from.
pub const AIRCRAFT: &str = "aircraft";
pub const DRONES: &str = "drones";
pub const AAA: &str = "aaa";

/// A model's root: where it is (`WorldPos`) and how it is turned; the model is its child,
/// lifted by `model_lift` and scaled to the world.
#[derive(Component)]
pub struct Model(pub Entity);

/// Spawning and placing the scenes' things.
#[derive(SystemParam)]
pub struct Stage<'w, 's> {
    pub commands: Commands<'w, 's>,
    library: ResMut<'w, ModelLibrary>,
    server: Res<'w, AssetServer>,
    assets: ModelAssets<'w>,
    pub theater: Option<Res<'w, Theater>>,
    pub director: ResMut<'w, Director>,
}

impl Stage<'_, '_> {
    /// The model files are loaded (and the theater): the scene can be spawned at once.
    pub fn ready(&mut self, files: &[&str]) -> bool {
        self.theater.is_some()
            && files.iter().all(|f| {
                let h = self.library.load(&self.server, f);
                self.assets.gltfs.get(&h).is_some()
            })
    }

    /// The scene's things are spawned once: true the first time it may spawn them (marks them
    /// spawned).
    pub fn begin(&mut self, files: &[&str]) -> bool {
        if self.director.spawned || !self.ready(files) {
            return false;
        }
        self.director.spawned = true;
        true
    }

    /// Spawns model `name` of `file` under a root of its own (the scene's), lifted `lift`
    /// metres; None if it isn't there.
    pub fn model(
        &mut self,
        file: &str,
        name: &str,
        lift: f32,
        state: ModelState,
    ) -> Option<Entity> {
        let h = self.library.load(&self.server, file);
        let model = self.assets.spawn(&mut self.commands, &h, name, LEVEL_AIRCRAFT, true)?;
        let scale = crate::player::AIRCRAFT_MODEL_SCALE as f32;
        self.commands
            .entity(model)
            .insert((Transform::from_xyz(0.0, lift, 0.0).with_scale(Vec3::splat(scale)), state));
        let root = self
            .commands
            .spawn((
                ShotEntity,
                Model(model),
                WorldPos::default(),
                Transform::default(),
                Visibility::default(),
            ))
            .add_child(model)
            .id();
        Some(root)
    }

    /// Spawns a flyable aircraft's model, lit, its gear up and its lights on.
    pub fn aircraft(&mut self, kind: AircraftKind) -> Option<Entity> {
        let spec = kind.spec();
        self.model(AIRCRAFT, spec.model, spec.model_lift, flying())
    }

    /// The ground's height (m) under a point (Bevy metres).
    pub fn ground(&self, p: DVec3) -> f64 {
        self.theater.as_ref().map_or(0.0, |t| t.hills.ground_m(p.x, -p.z) as f64)
    }

    /// The look's theater's places.
    pub fn places(&self) -> Places {
        self.theater.as_deref().map(Places::of).unwrap_or_default()
    }

    /// A point (Bevy metres, on the ground) for a scene over land: in open country (where
    /// the hills may rise, away from water, airfields and towns) a few kilometres from one of
    /// the theater's airfields, the scene's own pick; else by the airfield.
    pub fn land(&self) -> DVec3 {
        let places = self.places();
        let n = places.airfields.len().max(1);
        let mut h = self.director.roll as u64 | 1;
        let mut next = || {
            h = h.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((h >> 33) as f64) / (1u64 << 31) as f64
        };
        let base =
            |i: usize| places.airfields.get(i % n).copied().unwrap_or(DVec2::splat(256_000.0));
        let first = base(self.director.pick(n));
        // Open ground near the airfield, if nothing better.
        let a = next() * std::f64::consts::TAU;
        let mut found = first + DVec2::new(a.sin(), a.cos()) * 4000.0;
        if let Some(t) = &self.theater {
            for i in 0..64 {
                let b = if i < 32 { first } else { base((next() * n as f64) as usize) };
                let (a, d) = (next() * std::f64::consts::TAU, 3000.0 + next() * 9000.0);
                let p = b + DVec2::new(a.sin(), a.cos()) * d;
                if t.hills.mask(p.x, p.y) > 0.3 {
                    found = p;
                    break;
                }
            }
        }
        let at = DVec3::new(found.x, 0.0, -found.y);
        DVec3::new(at.x, self.ground(at), at.z)
    }
}

/// A flying aircraft's state: gear up (slot 13), navigation lights on (12).
pub fn flying() -> ModelState {
    let mut s = ModelState::default();
    (s.0[12], s.0[13]) = (1, 1);
    s
}

/// The places of a theater (metres east, north).
#[derive(Default, Clone)]
pub struct Places {
    /// Airfields on land.
    pub airfields: Vec<DVec2>,
    /// The enemy's SAM sites and their shapes.
    pub sams: Vec<(DVec2, usize)>,
    /// The carriers' decks.
    pub decks: Vec<DVec2>,
}

impl Places {
    pub fn of(t: &Theater) -> Self {
        use stealth_mission::world::flags as F;
        let mut p = Places::default();
        let at = |o: &stealth_formats::wld::WorldObject| {
            let (e, n) = crate::world::map_to_wu(o.x, o.y);
            DVec2::new(e, n) * crate::world::WU
        };
        for o in &t.world.objects {
            let f = o.target_flags;
            if f & (F::CARRIER | F::SHIP) != 0 {
                continue;
            }
            if f & F::AIRFIELD != 0 {
                p.airfields.push(at(o));
            } else if o.unit_type != 0 && f & (F::FRIENDLY | F::NEUTRAL | F::DISABLED) == 0 {
                p.sams.push((at(o), (o.object_idx & 0x7F) as usize));
            }
        }
        p.decks = t.decks.iter().map(|d| DVec2::new(d.east, d.north) * crate::world::WU).collect();
        p
    }
}

/// Bevy metres from metres east, up, north.
pub fn enu(e: f64, u: f64, n: f64) -> DVec3 {
    DVec3::new(e, u, -n)
}

/// A model's rotation flying `heading` (radians clockwise from north), pitched up `pitch` and
/// banked `bank` (left wing down positive, as `Transform`'s z).
pub fn attitude(heading: f32, pitch: f32, bank: f32) -> Quat {
    Quat::from_rotation_y(-heading) * Quat::from_rotation_x(pitch) * Quat::from_rotation_z(bank)
}

/// The flat forward (east, north) of a heading, and its right.
pub fn forward(heading: f32) -> Vec2 {
    Vec2::new(heading.sin(), heading.cos())
}

pub fn right(heading: f32) -> Vec2 {
    Vec2::new(heading.cos(), -heading.sin())
}

/// A weaving flight: along `heading` at `speed` (m/s) from `start` (Bevy metres), swinging
/// `amp` metres to either side every `period` seconds (`phase` radians into the swing),
/// climbing `climb` m/s.
#[derive(Clone, Copy, Debug)]
pub struct Weave {
    pub start: DVec3,
    pub heading: f32,
    pub speed: f32,
    pub amp: f32,
    pub period: f32,
    pub phase: f32,
    pub climb: f32,
}

impl Weave {
    /// The swing's amplitude (m) that banks `bank_deg` at its widest.
    pub fn amp_for(period: f32, bank_deg: f32) -> f32 {
        let w = std::f32::consts::TAU / period.max(0.1);
        9.81 * bank_deg.to_radians().tan() / (w * w)
    }

    /// Where it is `t` seconds in (Bevy metres), its attitude (banked into the swing) and
    /// velocity (Bevy m/s).
    pub fn at(&self, t: f32) -> (DVec3, Quat, Vec3) {
        let w = std::f32::consts::TAU / self.period.max(0.1);
        let a = w * t + self.phase;
        let (f, r) = (forward(self.heading), right(self.heading));
        let side = self.amp * (a.sin() - self.phase.sin());
        let flat = f * self.speed * t + r * side;
        let vel = f * self.speed + r * self.amp * w * a.cos();
        let accel = -self.amp * w * w * a.sin();
        let heading = vel.x.atan2(vel.y);
        let bank = -(accel / 9.81).atan();
        let pitch = (self.climb / self.speed.max(1.0)).atan();
        let pos = self.start + enu(flat.x as f64, (self.climb * t) as f64, flat.y as f64);
        (pos, attitude(heading, pitch, bank), Vec3::new(vel.x, self.climb, -vel.y))
    }
}

/// Puts a model's root at a place and attitude.
pub fn put(
    q: &mut Query<(&mut WorldPos, &mut Transform), Without<SceneCamera>>,
    e: Entity,
    p: DVec3,
    r: Quat,
) {
    if let Ok((mut pos, mut tf)) = q.get_mut(e) {
        pos.0 = p;
        tf.rotation = r;
    }
}

/// The scene camera.
pub type Camera<'w, 's> =
    Query<'w, 's, (&'static mut WorldPos, &'static mut Transform), With<SceneCamera>>;

/// The camera at `eye` looking at `at` (Bevy metres), the horizon level.
pub fn look(cam: &mut Camera, eye: DVec3, at: DVec3) {
    if let Ok((mut p, mut tf)) = cam.single_mut() {
        p.0 = eye;
        let dir = (at - eye).as_vec3().normalize_or(Vec3::NEG_Z);
        tf.rotation = Transform::IDENTITY.looking_to(dir, Vec3::Y).rotation;
    }
}

/// The camera at `eye` turned `rot`.
pub fn place(cam: &mut Camera, eye: DVec3, rot: Quat) {
    if let Ok((mut p, mut tf)) = cam.single_mut() {
        p.0 = eye;
        tf.rotation = rot;
    }
}

/// A point `back` metres behind, `side` to the right and `up` over `p` for something flying
/// `heading`.
pub fn behind(p: DVec3, heading: f32, back: f32, side: f32, up: f32) -> DVec3 {
    let (f, r) = (forward(heading), right(heading));
    let o = -f * back + r * side;
    p + enu(o.x as f64, up as f64, o.y as f64)
}

/// The rough length of an aircraft (m), for framing it.
pub fn length(kind: AircraftKind) -> f32 {
    use AircraftKind as K;
    match kind {
        K::C5 => 75.0,
        K::AC130 => 30.0,
        K::CH47 => 30.0,
        K::UH60 | K::MH60R => 20.0,
        K::OH58 | K::RAH66 | K::AV8B => 14.0,
        _ => 18.0,
    }
}

/// A sensible cruising speed (m/s) for a scene.
pub fn cruise(kind: AircraftKind) -> f32 {
    use AircraftKind as K;
    match kind {
        K::C5 => 120.0,
        K::AC130 => 100.0,
        _ if kind.spec().helicopter => 55.0,
        K::A10 => 140.0,
        _ => 190.0,
    }
}
