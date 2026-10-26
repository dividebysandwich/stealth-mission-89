//! The flown jet's afterburner plume (shader: assets/shaders/flame.wgsl): translucent nested
//! cones behind the nozzle (`AircraftSpec::exhaust`), drawn additively while the afterburner
//! burns. A yellow-white core with shock diamonds (fading with altitude), a sheath blue-violet
//! at the nozzle turning orange, and the nozzle's glowing exit; brighter and longer with the
//! afterburner's setting, longer in thinner air.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, RenderPipelineDescriptor,
    SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

use crate::flight::{Aircraft, FlightConstants, Player};

const SHADER: &str = "shaders/flame.wgsl";

/// Sides of the cones and rings along them.
const SIDES: usize = 16;
const RINGS: usize = 12;
/// The core's and the sheath's length (m) and radius at the nozzle, the nozzle exit's radius
/// (the F-16's: tools/models/aircraft.py).
const CORE: (f32, f32) = (2.8, 0.4);
const SHEATH: (f32, f32) = (6.0, 0.46);
const EXIT: f32 = 0.45;
/// The plume's length at the lowest setting (share of the full one), and how much longer it
/// grows by 40000 ft.
const MIN_LENGTH: f32 = 0.65;
const ALTITUDE_LENGTH: f32 = 0.4;
/// The shock diamonds fade out by this altitude (ft).
const DIAMONDS_ALT: f32 = 30000.0;

pub struct AfterburnerFlamePlugin;

impl Plugin for AfterburnerFlamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<FlameMaterial>::default())
            .add_systems(Update, (attach, update).chain());
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct FlameMaterial {
    /// x: intensity 0..1, y: seconds (flicker), z: shock diamonds 0..1.
    #[uniform(0)]
    pub params: Vec4,
}

impl Material for FlameMaterial {
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
            Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        // Seen from inside and out: both walls add up like a glowing volume.
        descriptor.primitive.cull_mode = None;
        // Additive, like the lightning.
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

/// The plume (a child of the player's aircraft).
#[derive(Component)]
struct Flame;

/// The plume's mesh in the aircraft frame from the nozzle exit, along +z (aft): uv x along
/// 0..1, y the layer (0 core, 1 sheath, 2 nozzle exit).
fn flame_mesh() -> Mesh {
    let mut pos: Vec<[f32; 3]> = Vec::new();
    let mut uv: Vec<[f32; 2]> = Vec::new();
    let mut idx: Vec<u32> = Vec::new();
    let core = |t: f32| CORE.1 * (1.0 - t).powf(0.8);
    let sheath = |t: f32| SHEATH.1 * (1.0 + 0.25 * t) * (1.0 - t * t * t).max(0.0).sqrt();
    for (layer, length, radius) in
        [(0.0, CORE.0, &core as &dyn Fn(f32) -> f32), (1.0, SHEATH.0, &sheath)]
    {
        let v0 = pos.len() as u32;
        for i in 0..=RINGS {
            let t = i as f32 / RINGS as f32;
            let r = radius(t);
            for j in 0..SIDES {
                let a = j as f32 / SIDES as f32 * std::f32::consts::TAU;
                pos.push([r * a.cos(), r * a.sin(), t * length]);
                uv.push([t, layer]);
            }
        }
        for i in 0..RINGS as u32 {
            for j in 0..SIDES as u32 {
                let (a, b) = (v0 + i * SIDES as u32, v0 + (i + 1) * SIDES as u32);
                let k = (j + 1) % SIDES as u32;
                idx.extend([a + j, b + j, b + k, a + j, b + k, a + k]);
            }
        }
    }
    // The nozzle exit: a fan.
    let c = pos.len() as u32;
    pos.push([0.0, 0.0, 0.0]);
    uv.push([0.0, 2.0]);
    for j in 0..SIDES {
        let a = j as f32 / SIDES as f32 * std::f32::consts::TAU;
        pos.push([EXIT * a.cos(), EXIT * a.sin(), 0.0]);
        uv.push([0.0, 2.0]);
    }
    for j in 0..SIDES as u32 {
        idx.extend([c, c + 1 + j, c + 1 + (j + 1) % SIDES as u32]);
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
        .with_inserted_indices(Indices::U32(idx))
}

/// The plume on a jet with an afterburner; none on any other.
fn attach(
    mut commands: Commands,
    player: Query<Entity, With<Player>>,
    flames: Query<Entity, With<Flame>>,
    k: Option<Res<FlightConstants>>,
    settings: Res<crate::settings::Settings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FlameMaterial>>,
) {
    let wanted = k.is_some_and(|k| k.afterburner());
    if !wanted {
        for e in &flames {
            commands.entity(e).despawn();
        }
        return;
    }
    let Ok(player) = player.single() else { return };
    if !flames.is_empty() {
        return;
    }
    let flame = commands
        .spawn((
            Flame,
            Mesh3d(meshes.add(flame_mesh())),
            MeshMaterial3d(materials.add(FlameMaterial { params: Vec4::ZERO })),
            Transform::from_translation(settings.aircraft.spec().exhaust),
            Visibility::Hidden,
        ))
        .id();
    commands.entity(player).add_child(flame);
}

/// The plume's brightness, length and shock diamonds with the afterburner, shown while it
/// burns (not with the own aircraft hidden or after the flight's end).
fn update(
    time: Res<Time>,
    player: Query<&Aircraft, With<Player>>,
    hide: Option<Res<crate::camera::HideOwnAircraft>>,
    settings: Res<crate::settings::Settings>,
    mut flames: Query<
        (&mut Transform, &mut Visibility, &MeshMaterial3d<FlameMaterial>),
        With<Flame>,
    >,
    mut materials: ResMut<Assets<FlameMaterial>>,
) {
    let Ok(a) = player.single() else { return };
    let ab = a.afterburner.clamp(0.0, 1.0);
    let on = ab > 0.0 && a.ended.is_none() && !hide.is_some_and(|h| h.0);
    for (mut tf, mut vis, material) in &mut flames {
        vis.set_if_neq(if on { Visibility::Inherited } else { Visibility::Hidden });
        if !on {
            continue;
        }
        let high = (a.altitude / 40000.0).clamp(0.0, 1.0);
        let length = (MIN_LENGTH + (1.0 - MIN_LENGTH) * ab) * (1.0 + ALTITUDE_LENGTH * high);
        tf.translation = settings.aircraft.spec().exhaust;
        tf.scale = Vec3::new(1.0, 1.0, length);
        if let Some(mut m) = materials.get_mut(&material.0) {
            let diamonds = (1.0 - a.altitude / DIAMONDS_ALT).clamp(0.0, 1.0);
            m.params =
                Vec4::new(0.4 + 0.6 * ab.sqrt(), time.elapsed_secs() % 1000.0, diamonds, 0.0);
        }
    }
}
