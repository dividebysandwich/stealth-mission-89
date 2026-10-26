//! Landing lights: a lamp under the nose shining forwards and a little down. With the local
//! lights on it is a spotlight (`lighting::Spotlight`) lighting the runway ahead; with them off
//! its beam is drawn as a translucent cone (shader: assets/shaders/beam.wgsl), never the own
//! aircraft's from the cockpit. On while the gear is down and the aircraft moves: the own
//! aircraft, the other players' and the AI aircraft taking off and landing
//! (`stealth_sim::lights::gear_down`); a helicopter's (no gear to go by) near an airfield of its
//! side, with its position lights. Never on a parked aircraft; dimmer by day.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, RenderPipelineDescriptor,
    SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

use crate::camera::{View, ViewState};
use crate::flight::{Aircraft, FlightConstants, Player};
use crate::lighting::{LightEmitter, LightingState, Spotlight};

const SHADER: &str = "shaders/beam.wgsl";

/// The lamp: ahead of and below the aircraft's origin (m), tilted down (degrees; a helicopter's
/// more, to light where it sets down).
const AHEAD: f32 = 4.0;
const BELOW: f32 = 1.0;
const TILT: f32 = 3.0;
const HELICOPTER_TILT: f32 = 20.0;
/// Colour at night and range (m) of the spotlight.
const COLOUR: Vec3 = Vec3::new(4.0, 3.8, 3.4);
const RANGE: f32 = 450.0;
/// Share of the brightness taken away in full daylight: the light's, the beam's.
const DAY_DIM: f32 = 0.7;
const BEAM_DAY_DIM: f32 = 0.9;
/// The beam cone's length (m), half angle (degrees, about the spotlight's), the lamp's radius
/// (m), its sides and rings along it.
const BEAM_LENGTH: f32 = 50.0;
const BEAM_ANGLE: f32 = 12.0;
const LAMP_RADIUS: f32 = 0.15;
const SIDES: usize = 16;
const RINGS: usize = 8;

pub struct LandingLightsPlugin;

impl Plugin for LandingLightsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<BeamMaterial>::default())
            .init_resource::<LandingAssets>()
            .add_systems(Update, (attach_own, update_own, beams, dim).chain());
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct BeamMaterial {
    /// x: intensity 0..1, y: tangent of the half angle, z: the lamp's radius, w: length (m).
    #[uniform(0)]
    pub params: Vec4,
}

impl Material for BeamMaterial {
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
        // Seen from inside and out, additive like the afterburner's plume.
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

/// The beam's mesh and material, shared by all landing lights.
#[derive(Resource)]
pub struct LandingAssets {
    mesh: Handle<Mesh>,
    material: Handle<BeamMaterial>,
}

impl FromWorld for LandingAssets {
    fn from_world(world: &mut World) -> Self {
        let mesh = world.resource_mut::<Assets<Mesh>>().add(beam_mesh());
        let params = Vec4::new(1.0, BEAM_ANGLE.to_radians().tan(), LAMP_RADIUS, BEAM_LENGTH);
        let material = world.resource_mut::<Assets<BeamMaterial>>().add(BeamMaterial { params });
        Self { mesh, material }
    }
}

/// A landing light's lamp (shown while it is on).
#[derive(Component)]
pub struct LandingLight;

/// The lamp's beam cone (shown with the local lights off).
#[derive(Component)]
struct Beam;

/// The own aircraft's lamp.
#[derive(Component)]
struct OwnLandingLight;

/// The beam cone from the lamp along -z, closed at both ends: uv x 0 (lamp) .. 1 (far end).
fn beam_mesh() -> Mesh {
    let end = BEAM_LENGTH * BEAM_ANGLE.to_radians().tan();
    let mut pos = Vec::new();
    let mut uv = Vec::new();
    for r in 0..=RINGS {
        let t = r as f32 / RINGS as f32;
        let radius = LAMP_RADIUS + (end - LAMP_RADIUS) * t;
        for s in 0..=SIDES {
            let a = s as f32 / SIDES as f32 * std::f32::consts::TAU;
            pos.push([radius * a.cos(), radius * a.sin(), -BEAM_LENGTH * t]);
            uv.push([t, 0.0]);
        }
    }
    let mut idx = Vec::new();
    let row = SIDES as u32 + 1;
    for r in 0..RINGS as u32 {
        for s in 0..SIDES as u32 {
            let a = r * row + s;
            idx.extend_from_slice(&[a, a + row, a + 1, a + 1, a + row, a + row + 1]);
        }
    }
    // Closed at both ends, all faces wound counterclockwise seen from outside (the shader draws
    // the back faces, where the view leaves it).
    for (ring, z) in [(0, 0.0), (RINGS as u32, -BEAM_LENGTH)] {
        let centre = pos.len() as u32;
        pos.push([0.0, 0.0, z]);
        uv.push([-z / BEAM_LENGTH, 0.0]);
        for s in 0..SIDES as u32 {
            let (a, b) = (ring * row + s, ring * row + s + 1);
            idx.extend_from_slice(&if z == 0.0 { [centre, a, b] } else { [centre, b, a] });
        }
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
        .with_inserted_indices(Indices::U32(idx))
}

/// A landing light, off, on aircraft entity `parent` (scaled by `scale`); its lamp.
pub fn spawn(
    commands: &mut Commands,
    assets: &LandingAssets,
    parent: Entity,
    scale: f32,
    helicopter: bool,
) -> Entity {
    let tilt = if helicopter { HELICOPTER_TILT } else { TILT };
    let lamp = commands
        .spawn((
            LandingLight,
            LightEmitter { color: COLOUR, range: RANGE },
            Spotlight,
            Transform {
                translation: Vec3::new(0.0, -BELOW, -AHEAD) / scale,
                rotation: Quat::from_rotation_x(-tilt.to_radians()),
                scale: Vec3::splat(1.0 / scale),
            },
            Visibility::Hidden,
        ))
        .with_child((
            Beam,
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.material.clone()),
            Transform::IDENTITY,
            Visibility::Hidden,
            bevy::light::NotShadowCaster,
        ))
        .id();
    commands.entity(parent).add_child(lamp);
    lamp
}

/// The landing lights' visibility (apart from the sim objects they hang on).
pub type Lights<'w, 's> = Query<
    'w,
    's,
    &'static mut Visibility,
    (With<LandingLight>, Without<crate::sim_render::SimObject>),
>;

/// Turns landing light `e` on or off.
pub fn set(lights: &mut Lights, e: Entity, on: bool) {
    if let Ok(mut v) = lights.get_mut(e) {
        v.set_if_neq(if on { Visibility::Inherited } else { Visibility::Hidden });
    }
}

fn attach_own(
    mut commands: Commands,
    assets: Res<LandingAssets>,
    settings: Res<crate::settings::Settings>,
    player: Query<Entity, With<Player>>,
    own: Query<(), With<OwnLandingLight>>,
) {
    let Ok(player) = player.single() else { return };
    if !own.is_empty() {
        return;
    }
    let helicopter = settings.aircraft.spec().helicopter;
    let lamp = spawn(&mut commands, &assets, player, 1.0, helicopter);
    commands.entity(lamp).insert(OwnLandingLight);
}

fn update_own(
    player: Query<&Aircraft, With<Player>>,
    k: Option<Res<FlightConstants>>,
    combat: Option<Res<crate::combat::Combat>>,
    own: Query<Entity, With<OwnLandingLight>>,
    mut lights: Lights,
) {
    let Ok(a) = player.single() else { return };
    let parked = a.on_ground && a.velocity == 0.0;
    // A helicopter's go with its position lights.
    let wanted = if k.is_some_and(|k| k.heli.is_some()) {
        combat.is_some_and(|c| c.sim.player_lights())
    } else {
        a.gear_down
    };
    for e in &own {
        set(&mut lights, e, wanted && !parked && a.ended.is_none());
    }
}

/// The beams show with the local lights off, the own one in the external views only.
fn beams(
    settings: Res<crate::settings::Settings>,
    view: Res<ViewState>,
    mut q: Query<(Entity, &mut Visibility), With<Beam>>,
    own: Query<&Children, With<OwnLandingLight>>,
) {
    let cockpit =
        matches!(view.view, View::Cockpit | View::LookLeft | View::LookRight | View::LookRear);
    let own = |e: Entity| own.iter().any(|c| c.contains(&e));
    for (e, mut v) in &mut q {
        let shown = !settings.local_lights && !(cockpit && own(e));
        v.set_if_neq(if shown { Visibility::Inherited } else { Visibility::Hidden });
    }
}

/// Dims them by day.
fn dim(
    state: Res<LightingState>,
    assets: Res<LandingAssets>,
    mut lights: Query<&mut LightEmitter, With<LandingLight>>,
    mut materials: ResMut<Assets<BeamMaterial>>,
) {
    let colour = COLOUR * (1.0 - DAY_DIM * state.daylight);
    for mut l in &mut lights {
        if l.color != colour {
            l.color = colour;
        }
    }
    let beam = 1.0 - BEAM_DAY_DIM * state.daylight;
    if materials.get(&assets.material).is_some_and(|m| m.params.x != beam)
        && let Some(mut m) = materials.get_mut(&assets.material)
    {
        m.params.x = beam;
    }
}
