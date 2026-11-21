//! Fast-turning rotors drawn as a blurred disc (shader: assets/shaders/rotor.wgsl) instead of
//! their blades. The disc is a child of the rotor's articulated part, so it turns with the drawn
//! rotor angle; the shader layers the optical effects of a rotor too fast to follow: the haze of
//! the blades' sweep (denser towards the root), motion-blurred ghost blades turning with the
//! drawn angle, the blade roots, the tip path, faint bands along the span, a glint where the
//! blades catch the sun, and a disc that thickens seen edge on.
//!
//! The hub stays solid. The blades fade into the disc with the rotor speed (`RotorBlur` on a
//! model root: the own helicopter's spin-up and spin-down); other rotors are always blurred.

use bevy::mesh::{Indices, MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::render::storage::ShaderBuffer;
use bevy::shader::ShaderRef;

const SHADER: &str = "shaders/rotor.wgsl";

/// Blade chord as a fraction of the rotor radius (UH-60: 0.53 m of 8.2 m).
const CHORD: f32 = 0.065;
/// Sides of the disc polygon (its rim is softened in the shader).
const SEGMENTS: usize = 96;
/// Rotor speeds (1 = 100%) over which the blades fade into the disc.
const BLUR_SPEED: (f32, f32) = (0.2, 0.55);
/// Blur at which the blades are hidden.
const BLADES_HIDDEN: f32 = 0.5;

pub struct RotorBlurPlugin;

impl Plugin for RotorBlurPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<RotorMaterial>::default())
            .init_resource::<Downwash>()
            .add_systems(Update, (own_rotor_blur, apply_rotor_blur, downwash).chain());
    }
}

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct RotorParams {
    /// Blade palette index.
    pub(crate) colour: u32,
    pub(crate) blades: u32,
    /// Hub radius and blade chord (fractions of the rotor radius).
    pub(crate) hub: f32,
    pub(crate) chord: f32,
    /// 0 (blades) .. 1 (disc).
    pub(crate) blur: f32,
    /// The part's axis (`models::ArticulatedPart`: 0 = y, 1 = x, 2 = z).
    pub(crate) axis: u32,
    _pad: Vec2,
}

#[derive(Asset, TypePath, Clone, Debug)]
pub struct RotorMaterial {
    pub(crate) params: RotorParams,
    pub(crate) globals: Handle<ShaderBuffer>,
}

crate::palette_material::globals_bind_group!(RotorMaterial, params);

impl Material for RotorMaterial {
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
        // Seen from above and below.
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

/// A rotor found among a part's primitives (model units, in the part's frame).
pub struct RotorShape {
    pub radius: f32,
    /// Hub radius as a fraction of `radius`.
    pub hub: f32,
    /// Height of the blades along the axis.
    pub offset: f32,
    pub blades: u32,
    /// Palette colour of the blades.
    pub colour: u8,
    /// Which primitives are blades (the others are the hub).
    pub blade_prims: Vec<bool>,
}

/// Plane coordinates (u, v) and axial height of a point of a part turning about `axis`
/// (`models::ArticulatedPart`: 0 = y, 1 = x, 2 = z), with u, v ordered so the drawn rotor
/// turns towards increasing atan2(v, u).
fn plane(axis: u8, p: Vec3) -> (Vec2, f32) {
    match axis {
        0 => (Vec2::new(p.x, p.z), p.y),
        1 => (Vec2::new(p.z, p.y), p.x),
        _ => (Vec2::new(p.y, p.x), p.z),
    }
}

fn from_plane(axis: u8, uv: Vec2, h: f32) -> Vec3 {
    match axis {
        0 => Vec3::new(uv.x, h, uv.y),
        1 => Vec3::new(h, uv.y, uv.x),
        _ => Vec3::new(uv.y, uv.x, h),
    }
}

/// Recognises a rotor from its primitives' (colour, triangles): blades reach well out from the
/// axis in a few separate directions. A part sweeping all round (a rotodome) is not one.
pub fn rotor_shape(axis: u8, prims: &[(u8, Vec<[Vec3; 3]>)]) -> Option<RotorShape> {
    // Points spread over each primitive's faces (a long blade may be one thin triangle).
    const STEPS: usize = 6;
    let points: Vec<Vec<Vec3>> = prims
        .iter()
        .map(|(_, tris)| {
            let mut out = Vec::new();
            for t in tris {
                for i in 0..=STEPS {
                    for j in 0..=STEPS - i {
                        let (a, b) = (i as f32 / STEPS as f32, j as f32 / STEPS as f32);
                        out.push(t[0] + (t[1] - t[0]) * a + (t[2] - t[0]) * b);
                    }
                }
            }
            out
        })
        .collect();
    let reach = |p: &[Vec3]| p.iter().map(|&v| plane(axis, v).0.length()).fold(0.0, f32::max);
    let radius = points.iter().map(|p| reach(p)).fold(0.0, f32::max);
    if radius <= 0.0 {
        return None;
    }
    let blade_prims: Vec<bool> = points.iter().map(|p| reach(p) > 0.5 * radius).collect();
    let blade_points =
        || points.iter().zip(&blade_prims).filter(|(_, b)| **b).flat_map(|(p, _)| p.iter());
    // Directions the outer half of the blades point in, in 5 degree bins.
    const BINS: usize = 72;
    let mut bins = [false; BINS];
    for &v in blade_points() {
        let (uv, _) = plane(axis, v);
        if uv.length() > 0.5 * radius {
            let a = uv.y.atan2(uv.x).rem_euclid(std::f32::consts::TAU);
            bins[(a / std::f32::consts::TAU * BINS as f32) as usize % BINS] = true;
        }
    }
    let filled = bins.iter().filter(|&&b| b).count();
    let blades = (0..BINS).filter(|&i| bins[i] && !bins[(i + BINS - 1) % BINS]).count() as u32;
    if !(2..=8).contains(&blades) || filled > BINS / 2 {
        return None;
    }
    let (sum, n) = blade_points().fold((0.0, 0), |(s, n), &v| (s + plane(axis, v).1, n + 1));
    let hub = points
        .iter()
        .zip(&blade_prims)
        .filter(|(_, b)| !**b)
        .map(|(p, _)| reach(p))
        .fold(0.05 * radius, f32::max);
    let colour = prims
        .iter()
        .zip(&blade_prims)
        .filter(|(_, b)| **b)
        .max_by_key(|((_, t), _)| t.len())
        .map_or(15, |((c, _), _)| *c);
    Some(RotorShape {
        radius,
        hub: hub / radius,
        offset: sum / n.max(1) as f32,
        blades,
        colour,
        blade_prims,
    })
}

/// A blade mesh of a rotor, shown only while its model's rotor turns slowly.
#[derive(Component)]
pub struct RotorBlade {
    pub root: Entity,
}

/// A rotor's blurred disc.
#[derive(Component)]
pub struct RotorDisc {
    pub root: Entity,
    material: Handle<RotorMaterial>,
    blur: f32,
    /// The hub, the axis and a point on the tips' circle (the part's frame).
    hub: Vec3,
    axis: Vec3,
    tip: Vec3,
}

/// On a model root: how far its rotors are blurred, 0 (blades) .. 1 (disc). Without it, 1.
#[derive(Component, Clone, Copy)]
pub struct RotorBlur(pub f32);

/// Spawns the disc of a rotor as a child of its part (`parent`); returns it.
pub fn spawn_disc(
    commands: &mut Commands,
    parent: Entity,
    root: Entity,
    axis: u8,
    shape: &RotorShape,
    globals: Handle<ShaderBuffer>,
) -> Entity {
    let mut pos = Vec::with_capacity(2 * SEGMENTS);
    let mut uv = Vec::with_capacity(2 * SEGMENTS);
    let mut idx = Vec::with_capacity(6 * SEGMENTS);
    // A ring from inside the hub to just past the tips (the shader fades both edges).
    let (inner, outer) = (shape.hub * 0.5, 1.02);
    for i in 0..SEGMENTS {
        let a = i as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
        let d = Vec2::new(a.cos(), a.sin());
        for r in [inner, outer] {
            pos.push(from_plane(axis, d * r * shape.radius, shape.offset).to_array());
            uv.push((d * r).to_array());
        }
        let (j, k) = (2 * i as u32, 2 * ((i + 1) % SEGMENTS) as u32);
        idx.extend([j, j + 1, k + 1, j, k + 1, k]);
    }
    let mut mesh =
        Mesh::new(PrimitiveTopology::TriangleList, bevy::asset::RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
    mesh.insert_indices(Indices::U32(idx));
    let material = RotorMaterial {
        params: RotorParams {
            colour: shape.colour as u32,
            blades: shape.blades,
            hub: shape.hub,
            chord: CHORD,
            blur: 1.0,
            axis: axis as u32,
            _pad: Vec2::ZERO,
        },
        globals,
    };
    let hub = from_plane(axis, Vec2::ZERO, shape.offset);
    let tip = from_plane(axis, Vec2::X * shape.radius, shape.offset);
    let axis_dir = from_plane(axis, Vec2::ZERO, 1.0) - from_plane(axis, Vec2::ZERO, 0.0);
    let disc = commands
        .spawn((Transform::default(), Visibility::Inherited, bevy::light::NotShadowCaster))
        .id();
    commands.entity(parent).add_child(disc);
    commands.queue(move |world: &mut World| {
        let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
        let material = world.resource_mut::<Assets<RotorMaterial>>().add(material);
        if let Ok(mut e) = world.get_entity_mut(disc) {
            e.insert((
                Mesh3d(mesh),
                MeshMaterial3d(material.clone()),
                RotorDisc { root, material, blur: 1.0, hub, axis: axis_dir, tip },
            ));
        }
    });
    disc
}

/// The own helicopter's rotors blur with its rotor speed.
#[allow(clippy::type_complexity)]
fn own_rotor_blur(
    mut commands: Commands,
    player: Query<&crate::flight::Aircraft, With<crate::flight::Player>>,
    flight: Option<Res<crate::flight::FlightConstants>>,
    own: Query<
        (Entity, Option<&RotorBlur>),
        Or<(With<crate::player::OwnModel>, With<crate::player::OwnPart>)>,
    >,
) {
    if !flight.is_some_and(|k| k.heli.is_some()) {
        return;
    }
    let Ok(a) = player.single() else { return };
    let (lo, hi) = BLUR_SPEED;
    let t = ((a.rotor_speed() - lo) / (hi - lo)).clamp(0.0, 1.0);
    let blur = t * t * (3.0 - 2.0 * t);
    for (e, was) in &own {
        if was.is_none_or(|b| (b.0 - blur).abs() > 0.005) {
            // The model may be going this frame (another aircraft type).
            commands.entity(e).try_insert(RotorBlur(blur));
        }
    }
}

fn apply_rotor_blur(
    roots: Query<&RotorBlur>,
    mut discs: Query<&mut RotorDisc>,
    mut blades: Query<(&RotorBlade, &mut Visibility)>,
    mut materials: ResMut<Assets<RotorMaterial>>,
) {
    let blur = |root: Entity| roots.get(root).map_or(1.0, |b| b.0);
    for mut d in &mut discs {
        let b = blur(d.root);
        if d.blur != b {
            d.blur = b;
            if let Some(mut m) = materials.get_mut(&d.material) {
                m.params.blur = b;
            }
        }
    }
    for (blade, mut vis) in &mut blades {
        let show = blur(blade.root) < BLADES_HIDDEN;
        vis.set_if_neq(if show { Visibility::Inherited } else { Visibility::Hidden });
    }
}

/// On a fixed-wing aircraft's model root (the own, the AI's, the other pilots', the menu
/// scenes'): low over the water at speed it leaves a wake on it, and spray close down
/// (`Downwash`). The radius (m) stands for its half span: how wide the air it pushes down.
#[derive(Component, Clone, Copy)]
pub struct WaterWake(pub f32);

impl WaterWake {
    /// A fixed-wing type's: about its half span (m); a helicopter's rotors stir the water instead.
    pub fn of(kind: crate::aircraft::AircraftKind) -> Option<Self> {
        use crate::aircraft::AircraftKind as K;
        if kind.spec().helicopter {
            return None;
        }
        Some(WaterWake(match kind {
            K::C5 => 34.0,
            K::AC130 => 20.0,
            K::A10 => 8.8,
            K::AV8B => 4.6,
            _ => 6.0,
        }))
    }
}

/// Something stirring the water under it: a helicopter's main rotor, or a jet low over it.
#[derive(Clone, Copy)]
pub struct WashSource {
    /// The rotor's hub or the aircraft's centre (world metres, from the map's origin).
    pub at: bevy::math::DVec3,
    /// The rotor's radius or the jet's half span (m).
    pub radius: f32,
    /// Its velocity (m/s).
    pub vel: Vec3,
    /// A rotor (beating the water round under it even in a hover), else a jet's wash.
    pub rotor: bool,
    /// The own aircraft's (its rotor wash already raises spray and dust: `explosions::downwash`).
    pub own: bool,
}

/// The aircraft stirring the water: those low over the sea (`sources`, for the spray), and for
/// `Globals::downwash` / `wash_vel` (with the ocean-waves effect) the two lowest for their size,
/// each its hub or centre (xyz, the render world) and radius (w; 0: none), and its velocity
/// (xyz, m/s; w 1 for a rotor).
#[derive(Resource, Default)]
pub struct Downwash {
    pub slots: [Vec4; 2],
    pub vel: [Vec4; 2],
    pub sources: Vec<WashSource>,
}

/// A rotor's wash reaches the water from this many radii up (palette.wgsl `WASH_REACH`); a
/// jet's from as many half spans.
pub const WASH_REACH: f32 = 5.0;

#[allow(clippy::too_many_arguments)]
fn downwash(
    settings: Res<crate::settings::Settings>,
    origin: Res<crate::world::FloatingOrigin>,
    time: Res<Time>,
    scenery: Option<Res<crate::scenery::Scenery>>,
    discs: Query<(Entity, &RotorDisc, &GlobalTransform)>,
    jets: Query<(Entity, &WaterWake, &GlobalTransform, &InheritedVisibility)>,
    own: Query<(), With<crate::player::OwnModel>>,
    mut out: ResMut<Downwash>,
    // Each source's last place (world) and smoothed velocity.
    mut last: Local<std::collections::HashMap<Entity, (bevy::math::DVec3, Vec3)>>,
) {
    use bevy::math::{DVec2, DVec3};
    let dt = time.delta_secs();
    let o = origin.origin;
    let mut found: Vec<(Entity, DVec3, Vec3, f32, bool, bool)> = Vec::new();
    // Main rotors turning (not a tail rotor's upright disc).
    for (e, d, gt) in &discs {
        if d.blur <= 0.5 {
            continue;
        }
        let hub = gt.transform_point(d.hub);
        let axis = gt.affine().transform_vector3(d.axis).normalize_or_zero();
        let radius = (gt.transform_point(d.tip) - hub).length();
        if axis.y.abs() > 0.7 && radius > 0.5 {
            found.push((e, hub.as_dvec3() + o, hub, radius, true, own.contains(d.root)));
        }
    }
    for (e, w, gt, vis) in &jets {
        // The own model is hidden in the cockpit view, but still flies.
        if vis.get() || own.contains(e) {
            let c = gt.translation();
            found.push((e, c.as_dvec3() + o, c, w.0, false, own.contains(e)));
        }
    }
    let mut seen = std::collections::HashMap::with_capacity(found.len());
    let mut sources = Vec::new();
    let mut low: Vec<(f32, Vec4, Vec4)> = Vec::new();
    for (e, at, render, radius, rotor, is_own) in found {
        let vel = match last.get(&e) {
            Some(&(prev, v)) if dt > 0.0 => {
                let now = ((at - prev) / dt as f64).as_vec3();
                // A jump (a teleport, the floating origin's shift is already out): start again.
                if now.length() > 800.0 { Vec3::ZERO } else { v.lerp(now, (dt * 8.0).min(1.0)) }
            }
            Some(&(_, v)) => v,
            None => Vec3::ZERO,
        };
        seen.insert(e, (at, vel));
        // The sea is at 0 m.
        let height = at.y as f32;
        if height <= 0.0 || height >= WASH_REACH * radius {
            continue;
        }
        let water = scenery.as_ref().is_none_or(|s| s.is_water(DVec2::new(at.x, -at.z)));
        if !water {
            continue;
        }
        sources.push(WashSource { at, radius, vel, rotor, own: is_own });
        low.push((height / radius, render.extend(radius), vel.extend(rotor as u8 as f32)));
    }
    *last = seen;
    let mut slots = [Vec4::ZERO; 2];
    let mut vels = [Vec4::ZERO; 2];
    if crate::palette_material::ocean(&settings) {
        low.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (k, (_, s, v)) in low.into_iter().take(2).enumerate() {
            (slots[k], vels[k]) = (s, v);
        }
    }
    if out.slots != slots || out.vel != vels {
        out.slots = slots;
        out.vel = vels;
    }
    out.sources = sources;
}
