//! Scorch marks and craters left on the ground: a blackened patch where a vehicle or a ground
//! target blew up, a crater with a rim of thrown earth and scorched ground round it where a bomb
//! or a missile struck or an aircraft crashed, a small one under a 105 mm shell. They stay for
//! the rest of the flight (the oldest go once there are `MAX_MARKS`).
//!
//! Each mark is one small mesh of several palette colours that follows the rolling hills,
//! lifted over the theater's flat overlays (over the runways and roads on concrete).

use std::collections::VecDeque;

use bevy::asset::RenderAssetUsages;
use bevy::math::{DVec2, DVec3};
use bevy::mesh::{MeshTag, PrimitiveTopology};
use bevy::prelude::*;

use crate::palette::Palette;
use crate::palette_material::{ATTRIBUTE_PALETTE_WORD, WORD_GROUND, mesh_tag};
use crate::settings::AppState;
use crate::world::WorldPos;

pub struct ScorchPlugin;

impl Plugin for ScorchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Scorch>().add_systems(OnExit(AppState::Flight), cleanup).add_systems(
            Update,
            spawn_marks.run_if(resource_exists::<Palette>).run_if(in_state(AppState::Flight)),
        );
    }
}

/// What made a mark.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mark {
    /// A vehicle or a ground target blowing up: a scorched patch.
    Burn,
    /// A bomb or a missile striking the ground: a crater in scorched ground.
    Crater,
    /// An aircraft crashing: a wide burnt patch round a shallow crater.
    Crash,
    /// A heavy shell: a small crater.
    Shell,
}

impl Mark {
    /// Radius of the scorched ground and of the crater (metres; no crater: 0).
    fn size(self, r: &mut Rng) -> (f32, f32) {
        match self {
            Mark::Burn => (r.range(4.0, 6.0), 0.0),
            Mark::Crater => (r.range(9.0, 12.0), r.range(3.0, 4.5)),
            Mark::Crash => (r.range(12.0, 16.0), r.range(2.5, 3.5)),
            Mark::Shell => (r.range(2.5, 3.5), r.range(1.0, 1.4)),
        }
    }
}

/// Marks kept at once.
const MAX_MARKS: usize = 200;
/// Height of a mark over the ground (metres): over the fields on land, over the runways,
/// taxiways, roads and their markings on concrete. Each mark is a little higher than the last,
/// so overlapping ones don't fight.
const LAND_Y: f32 = 0.2;
const CONCRETE_Y: f32 = 1.05;
const STEP_Y: f32 = 0.015;
const STEPS: usize = 8;
/// Colours: scorched ground (the asphalt's darkest grey), the blackened middle and the crater
/// (dark blue-greys), thrown earth (sand brown) and broken concrete (grey); all darken with the
/// day.
const SCORCHED: u8 = 0xEF;
const BLACK: u8 = 0x4E;
const PIT: u8 = 0x4F;
const EARTH: u8 = 0x12;
const RUBBLE: u8 = 0xEB;

/// Marks to make (Bevy world positions on the ground) and the marks made, oldest first.
#[derive(Resource, Default)]
pub struct Scorch {
    pending: Vec<(DVec3, Mark)>,
    marks: VecDeque<(Entity, DVec3, f32)>,
    made: usize,
    rng: Rng,
}

impl Scorch {
    /// A mark at `pos` (Bevy world, on the ground).
    pub fn add(&mut self, pos: DVec3, mark: Mark) {
        self.pending.push((pos, mark));
    }
}

#[derive(Default)]
struct Rng(u32);

impl Rng {
    fn f(&mut self) -> f32 {
        if self.0 == 0 {
            self.0 = 0x2545_F491;
        }
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 >> 8) as f32 / (1u32 << 24) as f32
    }

    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.f()
    }
}

fn cleanup(mut commands: Commands, mut scorch: ResMut<Scorch>) {
    for &(e, _, _) in &scorch.marks {
        commands.entity(e).try_despawn();
    }
    *scorch = Scorch::default();
}

fn spawn_marks(
    mut commands: Commands,
    mut scorch: ResMut<Scorch>,
    palette: Res<Palette>,
    mut meshes: ResMut<Assets<Mesh>>,
    scenery: Option<Res<crate::scenery::Scenery>>,
    theater: Option<Res<crate::world::Theater>>,
) {
    if scorch.pending.is_empty() {
        return;
    }
    let s = &mut *scorch;
    for (pos, mark) in std::mem::take(&mut s.pending) {
        let surface = scenery
            .as_ref()
            .map_or(crate::scenery::Surface::Land, |sc| sc.surface(DVec2::new(pos.x, -pos.z)));
        let concrete = match surface {
            crate::scenery::Surface::Water => continue,
            crate::scenery::Surface::Concrete => true,
            crate::scenery::Surface::Land => false,
        };
        let (radius, crater) = mark.size(&mut s.rng);
        // A mark already there as big covers this one.
        if s.marks.iter().any(|&(_, p, r)| r >= radius && p.distance(pos) < (r * 0.5) as f64) {
            continue;
        }
        let ground =
            |x: f64, z: f64| theater.as_ref().map_or(0.0, |t| t.hills.ground_m(x, -z) as f64);
        let lift = if concrete { CONCRETE_Y } else { LAND_Y } + STEP_Y * (s.made % STEPS) as f32;
        s.made += 1;
        let floor = ground(pos.x, pos.z);
        let centre = DVec3::new(pos.x, floor + lift as f64, pos.z);
        // A point of the mark (metres from its centre, `up` over the ground), following the hills.
        let at = |x: f32, z: f32, up: f32| {
            let h = ground(centre.x + x as f64, centre.z + z as f64) - floor;
            Vec3::new(x, h as f32 + up, z)
        };
        let mut tris: Vec<(u8, [Vec3; 3])> = Vec::new();
        let rng = &mut s.rng;
        // The scorched ground and its blackened middle.
        disc(&mut tris, rng, &at, radius, 14, 0.0, SCORCHED);
        disc(&mut tris, rng, &at, radius * 0.55, 10, 0.03, BLACK);
        if crater > 0.0 {
            disc(&mut tris, rng, &at, crater, 10, 0.06, PIT);
            rim(&mut tris, rng, &at, crater, if concrete { RUBBLE } else { EARTH });
            // Clods and chunks thrown round it.
            for _ in 0..(crater * 3.0) as usize {
                let a = rng.range(0.0, std::f32::consts::TAU);
                let d = crater * rng.range(1.5, 3.0);
                let (x, z) = (a.cos() * d, a.sin() * d);
                let s = rng.range(0.2, 0.5) * crater.sqrt();
                let colour = if concrete { RUBBLE } else { EARTH };
                tris.push((
                    colour,
                    [at(x - s, z - s, 0.09), at(x, z + s, s * 0.6), at(x + s, z - s * 0.4, 0.09)],
                ));
            }
        }
        let mesh = mark_mesh(&palette, tris);
        let e = commands
            .spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(palette.per_vertex.clone()),
                // Hazes like the scenery's fields.
                MeshTag(mesh_tag(3, 4, false)),
                WorldPos(centre),
                Transform::default(),
                bevy::light::NotShadowCaster,
            ))
            .id();
        s.marks.push_back((e, pos, radius));
        while s.marks.len() > MAX_MARKS {
            if let Some((old, _, _)) = s.marks.pop_front() {
                commands.entity(old).try_despawn();
            }
        }
    }
}

/// An irregular disc of `n` corners round the centre, `up` metres over the mark's ground.
fn disc(
    tris: &mut Vec<(u8, [Vec3; 3])>,
    rng: &mut Rng,
    at: &impl Fn(f32, f32, f32) -> Vec3,
    radius: f32,
    n: usize,
    up: f32,
    colour: u8,
) {
    let phase = rng.range(0.0, std::f32::consts::TAU);
    let ring: Vec<Vec3> = (0..n)
        .map(|k| {
            let a = phase + k as f32 / n as f32 * std::f32::consts::TAU;
            let r = radius * rng.range(0.75, 1.2);
            at(a.cos() * r, a.sin() * r, up)
        })
        .collect();
    let c = at(0.0, 0.0, up);
    for k in 0..n {
        tris.push((colour, [c, ring[k], ring[(k + 1) % n]]));
    }
}

/// A crater's rim: a broken ring of thrown earth (or concrete slabs) round the pit, its crest
/// a little outside the pit's edge.
fn rim(
    tris: &mut Vec<(u8, [Vec3; 3])>,
    rng: &mut Rng,
    at: &impl Fn(f32, f32, f32) -> Vec3,
    crater: f32,
    colour: u8,
) {
    let n = 9;
    let phase = rng.range(0.0, std::f32::consts::TAU);
    for k in 0..n {
        if rng.f() < 0.15 {
            continue;
        }
        let a0 = phase + k as f32 / n as f32 * std::f32::consts::TAU;
        let a1 = a0 + std::f32::consts::TAU / n as f32;
        let am = (a0 + a1) * 0.5;
        let h = crater * rng.range(0.2, 0.35);
        let (inner, crest, outer) = (crater * 0.95, crater * 1.2, crater * rng.range(1.6, 1.9));
        let p = |a: f32, r: f32, up: f32| at(a.cos() * r, a.sin() * r, up);
        let (i0, i1) = (p(a0, inner, 0.06), p(a1, inner, 0.06));
        let (o0, o1) = (p(a0, outer, 0.04), p(a1, outer, 0.04));
        let top = p(am, crest, h);
        // Its inside blackened.
        tris.push((BLACK, [i0, i1, top]));
        tris.push((colour, [i1, o1, top]));
        tris.push((colour, [o1, o0, top]));
        tris.push((BLACK, [o0, i0, top]));
    }
}

/// The mark's triangles as one mesh of several colours (drawn with `Palette::per_vertex`), each
/// wound to face up or out (back faces are culled).
fn mark_mesh(palette: &Palette, tris: Vec<(u8, [Vec3; 3])>) -> Mesh {
    let mut pos = Vec::with_capacity(tris.len() * 3);
    let mut words = Vec::with_capacity(tris.len() * 3);
    for (colour, mut t) in tris {
        let n = (t[1] - t[0]).cross(t[2] - t[0]);
        let mid = (t[0] + t[1] + t[2]) / 3.0;
        // Up, or away from the mark's middle for the rim's steep faces.
        let out = Vec3::new(mid.x, 0.0, mid.z).normalize_or_zero() * 0.3 + Vec3::Y;
        if n.dot(out) < 0.0 {
            t.swap(1, 2);
        }
        pos.extend(t);
        let word = palette.word(colour) as u32 | WORD_GROUND;
        words.extend([word; 3]);
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(ATTRIBUTE_PALETTE_WORD, words)
}
