//! Sunbeams (`Effects::sunbeams`): shafts of sunlight through the gaps of a broken stratus deck,
//! seen from below it. The small gaps round the viewpoint are found in the deck's coverage
//! field (`Weather::stratus_field`, the one the palette shader draws the deck from); each gets a
//! long closed cylinder from the deck's base down along the sunlight to the ground, drawn
//! additive with the light it scatters along the view ray through it (shader:
//! assets/shaders/sunbeam.wgsl). Beams fade in and out as the gaps come and go.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::math::DVec3;
use bevy::mesh::{Indices, MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, RenderPipelineDescriptor,
    SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

use crate::lighting::LightingState;
use crate::settings::AppState;
use crate::weather::WeatherState;
use crate::world::{FloatingOrigin, WU, WorldPos, wu_to_bevy};

const SHADER: &str = "shaders/sunbeam.wgsl";

/// The deck is searched for gaps on a lattice this many map units apart (about 400 m), this far
/// round the viewpoint (map units, 64 per km); none closer than `NEAR`.
const STEP: i32 = 26;
const REACH: i32 = 14 * 64;
const NEAR: i32 = 3 * 64 / 2;
/// Samples on each side of a gap's centre counted for its size.
const WINDOW: i32 = 3;
/// At most this many beams, this far apart (map units).
const MAX_BEAMS: usize = 12;
const SPACING: i32 = 100;
/// A beam's radius (m): from its gap's size, within these.
const RADIUS: (f32, f32) = (200.0, 900.0);
/// Longest beam (m; a low sun).
const MAX_LENGTH: f32 = 25_000.0;
/// The deck is searched again once the viewpoint has moved this far (map units).
const RESCAN: i32 = 64;
/// Seconds a beam takes to fade in or out.
const FADE_SECS: f32 = 2.0;
const SIDES: usize = 16;

pub struct SunbeamsPlugin;

impl Plugin for SunbeamsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<SunbeamMaterial>::default())
            .init_resource::<Sunbeams>()
            .add_systems(OnExit(AppState::Flight), clear)
            .add_systems(
                PostUpdate,
                sunbeams
                    .before(bevy::transform::TransformSystems::Propagate)
                    .run_if(in_state(AppState::Flight)),
            );
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct SunbeamMaterial {
    /// x: brightness 0..1, y: haze distance (m).
    #[uniform(0)]
    pub params: Vec4,
    /// Linear colour of the sunlight.
    #[uniform(0)]
    pub colour: Vec4,
}

impl Material for SunbeamMaterial {
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
        let vertex_layout =
            layout.0.get_layout(&[Mesh::ATTRIBUTE_POSITION.at_shader_location(0)])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        // Seen from inside and out (the shader picks the faces), additive.
        descriptor.primitive.cull_mode = None;
        if let Some(fragment) = descriptor.fragment.as_mut() {
            for target in fragment.targets.iter_mut().flatten() {
                target.blend = Some(BlendState {
                    color: BlendComponent {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                    alpha: BlendComponent::OVER,
                });
            }
        }
        Ok(())
    }
}

/// A gap's beam: where it hangs from the deck (world), its radius (m), and how far faded in.
struct Beam {
    entity: Entity,
    material: Handle<SunbeamMaterial>,
    apex: DVec3,
    radius: f32,
    fade: f32,
    wanted: bool,
}

#[derive(Resource, Default)]
pub struct Sunbeams {
    mesh: Option<Handle<Mesh>>,
    /// By the gap's lattice point.
    beams: HashMap<(i32, i32), Beam>,
    /// Where (map units), with which deck (seed, threshold) the deck was last searched.
    scanned: Option<((i32, i32), u32, f32)>,
    /// All beams' brightness, eased.
    strength: f32,
}

#[derive(Component)]
struct Sunbeam;

fn clear(mut commands: Commands, mut beams: ResMut<Sunbeams>) {
    for b in beams.beams.values() {
        commands.entity(b.entity).try_despawn();
    }
    *beams = Sunbeams { mesh: beams.mesh.take(), ..default() };
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The small gaps of the deck round map position (x, y): lattice point, centre (map units) and
/// radius (m), the most enclosed and nearest first, at most `MAX_BEAMS` and `SPACING` apart.
fn gaps(w: &stealth_mission::Weather, threshold: f32, x: i32, y: i32) -> Vec<((i32, i32), f32)> {
    let n = REACH / STEP;
    let side = (2 * n + 1) as usize;
    let (cx, cy) = (x.div_euclid(STEP), y.div_euclid(STEP));
    let field: Vec<f32> = (-n..=n)
        .flat_map(|j| (-n..=n).map(move |i| (cx + i, cy + j)))
        .map(|(gx, gy)| w.stratus_field((gx * STEP) as f32, (gy * STEP) as f32))
        .collect();
    let at = |i: i32, j: i32| field[(j + n) as usize * side + (i + n) as usize];
    // Clear sky: below the threshold, with a margin for the deck's soft edge.
    let clear = threshold - 0.01;
    let mut found = Vec::new();
    let inner = n - WINDOW;
    for j in -inner..=inner {
        for i in -inner..=inner {
            let f = at(i, j);
            let (dx, dy) = ((cx + i) * STEP - x, (cy + j) * STEP - y);
            let d2 = dx * dx + dy * dy;
            if f >= clear || !(NEAR * NEAR..=REACH * REACH).contains(&d2) {
                continue;
            }
            // A gap's centre: the clearest of its neighbours.
            let lowest = (-1..=1).all(|b| (-1..=1).all(|a| at(i + a, j + b) >= f));
            if !lowest {
                continue;
            }
            let open = (-WINDOW..=WINDOW)
                .flat_map(|b| (-WINDOW..=WINDOW).map(move |a| (a, b)))
                .filter(|&(a, b)| at(i + a, j + b) < clear)
                .count();
            let share = open as f32 / ((2 * WINDOW + 1) * (2 * WINDOW + 1)) as f32;
            // A hole in the deck, not open sky.
            if share > 0.5 {
                continue;
            }
            let step_m = STEP as f32 * 32.0 * WU as f32;
            let radius = (open as f32 / std::f32::consts::PI).sqrt() * step_m;
            let score = share + 0.5 * (d2 as f32).sqrt() / REACH as f32;
            found.push((score, (cx + i, cy + j), radius.clamp(RADIUS.0, RADIUS.1)));
        }
    }
    found.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut picked: Vec<((i32, i32), f32)> = Vec::new();
    for (_, key, radius) in found {
        let far = |&(k, _): &((i32, i32), f32)| {
            let (dx, dy) = ((k.0 - key.0) * STEP, (k.1 - key.1) * STEP);
            dx * dx + dy * dy >= SPACING * SPACING
        };
        if picked.iter().all(far) {
            picked.push((key, radius));
            if picked.len() == MAX_BEAMS {
                break;
            }
        }
    }
    picked
}

/// Finds the gaps, spawns and fades the beams, and points them along the sunlight.
#[allow(clippy::too_many_arguments)]
fn sunbeams(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<crate::settings::Settings>,
    weather: Res<WeatherState>,
    lighting: Res<LightingState>,
    origin: Res<FloatingOrigin>,
    mut beams: ResMut<Sunbeams>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SunbeamMaterial>>,
    mut transforms: Query<&mut Transform, With<Sunbeam>>,
) {
    let w = weather.weather;
    let (cover, threshold) = weather.drawn_deck();
    let sun = lighting.sun.truncate().normalize_or(Vec3::Y);
    let below = weather.eye_ft < w.stratus.base_ft as f32;
    // A broken deck (not open sky, not a full overcast) over the eye, the sun up.
    let target = if settings.effects.sunbeams && settings.modern_shading && below {
        smoothstep(30.0, 45.0, cover)
            * (1.0 - smoothstep(92.0, 98.0, cover))
            * smoothstep(0.08, 0.2, sun.y)
            * lighting.daylight
            * (1.0 - weather.whiteout)
            * (1.0 - 0.6 * weather.storm())
    } else {
        0.0
    };
    let dt = time.delta_secs();
    beams.strength += (target - beams.strength) * (dt / FADE_SECS).min(1.0);
    let wanted = target > 0.0;
    let (x, y) = weather.eye_map;
    let seed = w.layer_seed(stealth_mission::CloudKind::Stratus);
    let rescan = wanted
        && beams.scanned.is_none_or(|((sx, sy), s, t)| {
            (sx - x).abs() > RESCAN
                || (sy - y).abs() > RESCAN
                || s != seed
                || (t - threshold).abs() > 0.002
        });
    if rescan {
        beams.scanned = Some(((x, y), seed, threshold));
        let found = gaps(&w, threshold, x, y);
        for b in beams.beams.values_mut() {
            b.wanted = false;
        }
        let mesh = beams.mesh.get_or_insert_with(|| meshes.add(cylinder())).clone();
        for (key, radius) in found {
            if let Some(b) = beams.beams.get_mut(&key) {
                b.wanted = true;
                continue;
            }
            let (mx, my) = (key.0 * STEP, key.1 * STEP);
            let apex =
                wu_to_bevy(mx as f64 * 32.0, (0x8000 - my) as f64 * 32.0, w.stratus.base_ft as f64);
            let material = materials.add(SunbeamMaterial { params: Vec4::ZERO, colour: Vec4::ONE });
            let entity = commands
                .spawn((
                    Sunbeam,
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    WorldPos(apex),
                    Transform::from_translation((apex - origin.origin).as_vec3()),
                    bevy::light::NotShadowCaster,
                ))
                .id();
            beams
                .beams
                .insert(key, Beam { entity, material, apex, radius, fade: 0.0, wanted: true });
        }
    } else if !wanted {
        for b in beams.beams.values_mut() {
            b.wanted = false;
        }
    }
    // Low sun: golden.
    let colour =
        Vec3::new(1.0, 0.8, 0.55).lerp(Vec3::new(1.0, 0.96, 0.88), smoothstep(0.1, 0.45, sun.y));
    let haze = (w.vis_km as f32 * 1000.0).max(4000.0);
    let ground = weather.ground_m() as f64;
    let down = -sun;
    let up = if down.y.abs() > 0.99 { Vec3::X } else { Vec3::Y };
    let k = dt / FADE_SECS;
    let strength = beams.strength;
    beams.beams.retain(|_, b| {
        b.fade = (b.fade + if b.wanted { k } else { -k }).clamp(0.0, 1.0);
        if b.fade <= 0.0 && !b.wanted {
            commands.entity(b.entity).try_despawn();
            return false;
        }
        let (params, colour) = (Vec4::new(strength * b.fade, haze, 0.0, 0.0), colour.extend(1.0));
        if materials.get(&b.material).is_some_and(|m| m.params != params || m.colour != colour)
            && let Some(mut m) = materials.get_mut(&b.material)
        {
            (m.params, m.colour) = (params, colour);
        }
        if let Ok(mut tf) = transforms.get_mut(b.entity) {
            let length =
                (((b.apex.y - ground) as f32).max(100.0) / sun.y.max(0.05)).min(MAX_LENGTH);
            tf.rotation = Transform::IDENTITY.looking_to(down, up).rotation;
            tf.scale = Vec3::new(b.radius, b.radius, length);
        }
        true
    });
    if !wanted && beams.beams.is_empty() {
        beams.scanned = None;
    }
}

/// A closed cylinder of radius 1 from the origin to z = -1, wound counterclockwise seen from
/// outside.
fn cylinder() -> Mesh {
    let mut pos = Vec::new();
    for z in [0.0, -1.0] {
        for s in 0..SIDES {
            let a = s as f32 / SIDES as f32 * std::f32::consts::TAU;
            pos.push([a.cos(), a.sin(), z]);
        }
    }
    let n = SIDES as u32;
    let mut idx = Vec::new();
    for s in 0..n {
        let (a, b) = (s, (s + 1) % n);
        idx.extend_from_slice(&[a, a + n, b, b, a + n, b + n]);
    }
    // The ends: a fan round their centres.
    for (centre, z, ring) in [(2 * n, 0.0, 0), (2 * n + 1, -1.0, n)] {
        pos.push([0.0, 0.0, z]);
        for s in 0..n {
            let (a, b) = (ring + s, ring + (s + 1) % n);
            idx.extend_from_slice(&if z == 0.0 { [centre, a, b] } else { [centre, b, a] });
        }
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_indices(Indices::U32(idx))
}
