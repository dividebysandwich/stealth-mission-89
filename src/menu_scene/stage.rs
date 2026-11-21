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
    scenery: Option<Res<'w, crate::scenery::Scenery>>,
    papi: Option<Res<'w, crate::scenery::PapiSites>>,
}

/// A scene waits this long (s) for its theater's scenery (the ground its things stand on)
/// before it starts without it.
const SCENERY_WAIT: f32 = 1.5;

/// How long a scene waits for the scenery: on one theater (the browser's) longer, its layout
/// is made once, slowly, and every scene after has it.
fn scenery_wait() -> f32 {
    if super::director::one_theater().is_some() { 12.0 } else { SCENERY_WAIT }
}
/// The clearing a scene on the ground wants round its place (m).
const CLEARING: f64 = 70.0;

impl Stage<'_, '_> {
    /// The model files are loaded (and the theater): the scene can be spawned at once.
    pub fn ready(&mut self, files: &[&str]) -> bool {
        self.theater.is_some()
            && (self.scenery.is_some() || self.director.t > scenery_wait())
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
        // Its clock from the start (it may have waited for the ground).
        self.director.restart();
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

    /// Spawns a cargo parachute's canopy (its origin at its load), hidden until it opens.
    pub fn canopy(&mut self) -> Entity {
        let e = crate::sim_render::spawn_canopy(&mut self.commands, &self.assets);
        self.commands.entity(e).insert((ShotEntity, Transform::from_scale(Vec3::splat(0.01))));
        e
    }

    /// Spawns a flyable aircraft's model, lit, its gear up and its lights on.
    pub fn aircraft(&mut self, kind: AircraftKind) -> Option<Entity> {
        let spec = kind.spec();
        self.model(AIRCRAFT, spec.model, spec.model_lift, flying())
    }

    /// The ground's height (m) under a point (Bevy metres).
    pub fn ground(&self, p: DVec3) -> f64 {
        ground_at(self.theater.as_deref(), p)
    }

    /// Whether nothing stands in the way from `a` to `b` (Bevy metres): the hills stay `margin`
    /// metres under the line all along it, and where the line runs low no buildings or trees
    /// stand under it (the last few metres at `b`, where the target stands, left out).
    pub fn sight_clear(&self, a: DVec3, b: DVec3, margin: f64) -> bool {
        let len = a.distance(b);
        let n = (len / 20.0).ceil().max(2.0) as usize;
        (1..n).all(|i| {
            let p = a.lerp(b, i as f64 / n as f64);
            let agl = p.y - self.ground(p);
            if p.distance(b) < 30.0 {
                return agl > -1.0;
            }
            agl > margin
                && (agl > 40.0
                    || self
                        .scenery
                        .as_ref()
                        .is_none_or(|s| s.unit_clear(DVec2::new(p.x, -p.z), 8.0)))
        })
    }

    /// Whether a place (Bevy metres) is open ground: dry, no buildings or trees within `r`
    /// metres (true without the scenery).
    pub fn clear(&self, p: DVec3, r: f64) -> bool {
        self.scenery.as_ref().is_none_or(|s| s.unit_clear(DVec2::new(p.x, -p.z), r))
    }

    /// The runway ends of the scenery's airfields (aim point, unit direction towards the
    /// approaching aircraft; metres east, north).
    pub fn runway_ends(&self) -> Vec<(DVec2, DVec2)> {
        self.papi.as_ref().map_or(Vec::new(), |p| p.0.iter().map(|s| (s.aim, s.toward)).collect())
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
            // With the scenery there: a clearing (no water, buildings or trees round it).
            let flat = t.hills.is_flat();
            let good = |p: DVec2| {
                let open = flat || t.hills.mask(p.x, p.y) > 0.3;
                open && self.scenery.as_ref().is_none_or(|s| s.unit_clear(p, CLEARING))
            };
            for i in 0..160 {
                let b = if i < 80 { first } else { base((next() * n as f64) as usize) };
                let (a, d) = (next() * std::f64::consts::TAU, 3000.0 + next() * 9000.0);
                let p = b + DVec2::new(a.sin(), a.cos()) * d;
                if good(p) {
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
    /// The carriers' decks, and their world objects (map units).
    pub decks: Vec<DVec2>,
    pub deck_objects: Vec<(i32, i32)>,
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
        p.deck_objects = t.decks.iter().map(|d| (d.x, d.y)).collect();
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
pub fn put<F: bevy::ecs::query::QueryFilter>(
    q: &mut Query<(&mut WorldPos, &mut Transform), F>,
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

/// A pilot's maneuvering: level flight, rolling into a bank of a random side and depth,
/// holding the turn while pulling, now and then reversing into the other bank, rolling out
/// and flying level again. The bank follows its targets at the roll rate the aircraft has,
/// eased in and out; the heading turns as the bank makes it, the nose comes up with the pull.
#[derive(Clone, Debug)]
pub struct Maneuver {
    pub start: DVec3,
    pub heading: f32,
    pub speed: f32,
    /// The bank wanted from each time on (s, rad, right wing down positive).
    plan: Vec<(f32, f32)>,
    /// The fastest roll (rad/s).
    roll_rate: f32,
}

impl Maneuver {
    /// A random sequence `seed`'s, `long` seconds or more; `agile`: a fighter's banks and roll
    /// rate, else a heavier aircraft's or a helicopter's.
    pub fn random(
        seed: u32,
        start: DVec3,
        heading: f32,
        speed: f32,
        agile: bool,
        long: f32,
    ) -> Self {
        let mut h = seed as u64 | 1;
        let mut next = || {
            h = h.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((h >> 33) as f32) / (1u64 << 31) as f32
        };
        let range = |r: &mut dyn FnMut() -> f32, a: f32, b: f32| a + (b - a) * r();
        let (deep, roll_rate) = if agile { (75f32, 100f32) } else { (40f32, 35f32) };
        let mut plan = Vec::new();
        let mut t = 0.0;
        // Sometimes already in a turn at the start.
        let mut bank = if next() < 0.3 { range(&mut next, -0.5, 0.5) } else { 0.0 };
        plan.push((0.0, bank));
        t += range(&mut next, 0.4, 1.4);
        while t < long {
            let level = bank.abs() > 0.1;
            let r = next();
            bank = if level && r < 0.55 {
                // Roll out.
                0.0
            } else if level && r < 0.8 {
                // Reverse into the other side.
                -bank.signum() * range(&mut next, 25.0, deep).to_radians()
            } else {
                // Into a bank (or deeper in this one).
                let side = if next() < 0.5 { -1.0 } else { 1.0 };
                side * range(&mut next, 25.0, deep).to_radians()
            };
            plan.push((t, bank));
            t += if bank == 0.0 { range(&mut next, 0.6, 1.5) } else { range(&mut next, 1.0, 2.2) };
        }
        Maneuver { start, heading, speed, plan, roll_rate: roll_rate.to_radians() }
    }

    /// The bank wanted at `t`.
    fn wanted(&self, t: f32) -> f32 {
        self.plan.iter().rev().find(|(at, _)| *at <= t).map_or(0.0, |p| p.1)
    }

    /// Where it is `t` seconds in (Bevy metres), its attitude and velocity: flown from the
    /// start in small steps.
    pub fn at(&self, t: f32) -> (DVec3, Quat, Vec3) {
        const DT: f32 = 1.0 / 120.0;
        let (mut pos, mut heading) = (self.start, self.heading);
        let mut bank = self.wanted(0.0);
        let mut rate = 0.0f32;
        let mut s = 0.0;
        while s < t {
            let dt = DT.min(t - s);
            // The roll rate eases towards what closes on the bank wanted, no faster than the
            // aircraft rolls, as a pilot's stick.
            let want = ((self.wanted(s) - bank) * 3.0).clamp(-self.roll_rate, self.roll_rate);
            rate += (want - rate) * (dt * 6.0).min(1.0);
            bank += rate * dt;
            heading += 9.81 * bank.tan() / self.speed.max(1.0) * dt;
            let f = forward(heading) * self.speed * dt;
            pos += enu(f.x as f64, 0.0, f.y as f64);
            s += dt;
        }
        // The pull holding the turn level lifts the nose a little; a touch of buffet.
        let g = 1.0 / bank.cos().max(0.25);
        let pitch = (0.03 + 0.015 * (g - 1.0)) + (t * 7.0).sin() * 0.002 * (g - 1.0);
        let f = forward(heading) * self.speed;
        // The model's bank is left wing down positive (`attitude`).
        (pos, attitude(heading, pitch, -bank), Vec3::new(f.x, 0.0, -f.y))
    }
}

/// The ground's height (m) under a point (Bevy metres): the hills' (the scenery's skin draws
/// them).
pub fn ground_at(theater: Option<&Theater>, p: DVec3) -> f64 {
    theater.map_or(0.0, |t| t.hills.ground_m(p.x, -p.z) as f64)
}
