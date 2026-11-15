//! Billboard clouds (`CloudQuality::Billboard`, the Steam Frame's default): the cumulus and
//! cumulonimbus drawn as camera-facing puffs, two for each sphere of their volumes, in one
//! particle batch (particle.wgsl / gl/effects.glsl in cloud mode) instead of being ray-marched.
//! The streamed clouds stay as they are (their shadows, fading and the whiteout use them); the
//! volumetric renderer only skips them, hidden.
//!
//! Each puff is shaded once on the CPU from the live cloud ramp (`CloudShading`, which follows
//! the time of day, the night and the goggles): bright on top and towards the sun, grey
//! underneath. The shader fades the puffs into the haze far out and away right in front of
//! the eye. The batch is sorted back to front and rebuilt when the clouds or their light
//! change, or the eye has moved on.

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::math::DVec3;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy_fast_clouds::{CloudShading, CloudVolume, WaveletVolume};

use super::{CloudEntity, CloudShade, Eyes, WeatherState, eye_position};
use crate::explosions::{ATTRIBUTE_COLOUR, ATTRIBUTE_CORNER, ATTRIBUTE_LOOK, ParticleMaterial};
use crate::world::{FloatingOrigin, WorldPos};

/// The eye moves this far (metres) before the puffs are sorted again.
const RESORT: f64 = 400.0;
/// Seconds between checks of the clouds' light (the ramp moves slowly with the time of day).
const RELIGHT: f32 = 0.5;

/// The batch and what it was built from.
#[derive(Resource, Default)]
pub(super) struct Billboards {
    batch: Option<(Entity, Handle<Mesh>)>,
    material: Option<Handle<ParticleMaterial>>,
    /// The eye, the clouds (entity, fill, darkening) and the ramp the batch was built for.
    eye: DVec3,
    clouds: Vec<(Entity, f32, f32)>,
    ramp: [Vec4; 16],
    since: f32,
}

/// The billboard batch's entity.
#[derive(Component)]
pub(super) struct BillboardBatch;

/// Hides the streamed clouds from the volumetric renderer while billboards draw them (and
/// shows them again when the volumetric clouds are chosen).
pub(super) fn hide_volumes(
    settings: Res<crate::settings::Settings>,
    mut q: Query<&mut Visibility, With<CloudShade>>,
) {
    let want =
        if settings.cloud_quality.billboard() { Visibility::Hidden } else { Visibility::Inherited };
    for mut v in &mut q {
        v.set_if_neq(want);
    }
}

/// Builds or updates the billboard batch.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_billboards(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<crate::settings::Settings>,
    palette: Option<Res<crate::palette::Palette>>,
    origin: Res<FloatingOrigin>,
    state: Res<WeatherState>,
    shading: Res<CloudShading>,
    clouds: Query<(Entity, &WorldPos, &CloudVolume), With<CloudShade>>,
    volumes: Res<Assets<WaveletVolume>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ParticleMaterial>>,
    mut b: ResMut<Billboards>,
    mut sets: ParamSet<(
        Eyes,
        Query<
            (&mut WorldPos, &mut Transform, &mut GlobalTransform, &mut Visibility),
            (With<BillboardBatch>, Without<CloudShade>),
        >,
    )>,
) {
    let b = &mut *b;
    let on = settings.cloud_quality.billboard() && super::clouds_enabled();
    let Some(palette) = palette else { return };
    let Some(eye) = eye_position(&sets.p0(), &origin) else { return };
    let mut batches = sets.p1();
    // The material: the haze distance, the lightning's flash.
    let params = Vec4::new(1.0, shading.haze_distance * 1.6, state.flash.min(1.5), 0.0);
    match &b.material {
        Some(m) => {
            if let Some(mat) = materials.get(m)
                && mat.params != params
                && let Some(mut mat) = materials.get_mut(m)
            {
                mat.params = params;
            }
        }
        None => b.material = Some(materials.add(ParticleMaterial::new(params, &palette.globals))),
    }
    let mut list: Vec<(Entity, f32, f32)> = clouds
        .iter()
        .filter(|(.., v)| v.fill > 0.0)
        .map(|(e, _, v)| (e, v.fill, v.darken))
        .collect();
    list.sort_by_key(|c| c.0);
    if !on || list.is_empty() {
        if let Some((e, _)) = b.batch
            && let Ok((.., mut v)) = batches.get_mut(e)
        {
            v.set_if_neq(Visibility::Hidden);
        }
        b.clouds.clear();
        return;
    }
    b.since += time.delta_secs();
    let relight = b.since >= RELIGHT && ramp_moved(&b.ramp, &shading.ramp);
    if b.batch.is_some() && list == b.clouds && eye.distance(b.eye) < RESORT && !relight {
        if let Some((e, _)) = b.batch
            && let Ok((.., mut v)) = batches.get_mut(e)
        {
            v.set_if_neq(Visibility::Inherited);
        }
        return;
    }
    if b.since >= RELIGHT {
        b.since = 0.0;
    }
    b.ramp = shading.ramp;
    b.eye = eye;
    b.clouds = list;

    // The puffs: centre (world metres), radius, colour, alpha, seed.
    let sun = shading.sun_dir.normalize_or(Vec3::Y);
    let mut puffs: Vec<(DVec3, f32, Vec3, f32, f32)> = Vec::new();
    for &(e, fill, darken) in &b.clouds {
        let Ok((_, pos, v)) = clouds.get(e) else { continue };
        let Some(vol) = volumes.get(&v.volume) else { continue };
        let fill = fill.min(1.0);
        let top = vol.aabb_max.y.max(1.0);
        let mid = (vol.aabb_min + vol.aabb_max) * 0.5;
        for (i, s) in vol.spheres.iter().enumerate() {
            // Lit from above and from the sun's side, grey underneath.
            let up = ((s.center.y + 0.5 * s.radius) / top).clamp(0.0, 1.0);
            let side = (s.center - mid).normalize_or_zero().dot(sun);
            for (k, (offset, size, alpha)) in
                [(Vec3::ZERO, 1.15, 0.8), (Vec3::new(0.3, 0.35, -0.25), 0.8, 0.7)]
                    .into_iter()
                    .enumerate()
            {
                let seed = hash(e.to_bits() * 31 + i as u64 * 2 + k as u64);
                // Turn the offset round the axis by the seed.
                let a = seed * std::f32::consts::TAU;
                let o = Vec3::new(
                    offset.x * a.cos() - offset.z * a.sin(),
                    offset.y,
                    offset.x * a.sin() + offset.z * a.cos(),
                ) * s.radius;
                let lift = 0.15 * o.y.signum() * k as f32;
                let lit = (0.3 + 0.6 * up + 0.2 * side + lift).clamp(0.0, 1.0);
                let shade = 7.0 * (1.0 - lit).powf(1.5);
                let c = ramp(&shading.ramp, shade) * (1.0 - 0.65 * darken);
                let centre = pos.0 + (s.center + o).as_dvec3();
                let r = s.radius * size * (0.6 + 0.4 * fill);
                puffs.push((centre, r, c, alpha * fill, seed));
            }
        }
    }
    // Back to front from the eye.
    puffs.sort_by(|a, b| b.0.distance_squared(eye).total_cmp(&a.0.distance_squared(eye)));
    let n = puffs.len();
    let mut pos = Vec::with_capacity(4 * n);
    let mut corner = Vec::with_capacity(4 * n);
    let mut colour = Vec::with_capacity(4 * n);
    let mut look = Vec::with_capacity(4 * n);
    let mut idx = Vec::with_capacity(6 * n);
    for (p, r, c, a, seed) in puffs {
        let local = (p - eye).as_vec3();
        let k = pos.len() as u32;
        for (cx, cy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            pos.push(local);
            corner.push([cx, cy, r, seed * 6.3]);
            colour.push([c.x, c.y, c.z, a]);
            look.push([0.0, seed]);
        }
        idx.extend([k, k + 1, k + 2, k, k + 2, k + 3]);
    }
    let fill = |m: &mut Mesh| {
        m.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
        m.insert_attribute(ATTRIBUTE_CORNER, corner);
        m.insert_attribute(ATTRIBUTE_COLOUR, colour);
        m.insert_attribute(ATTRIBUTE_LOOK, look);
        m.insert_indices(Indices::U32(idx));
    };
    match &b.batch {
        Some((e, mesh)) => {
            if let Some(mut m) = meshes.get_mut(mesh) {
                fill(&mut m);
            }
            // Moved now: the transforms were already propagated this frame.
            if let Ok((mut p, mut tf, mut gtf, mut v)) = batches.get_mut(*e) {
                p.0 = eye;
                tf.translation = (eye - origin.origin).as_vec3();
                *gtf = GlobalTransform::from(*tf);
                v.set_if_neq(Visibility::Inherited);
            }
        }
        None => {
            let mut m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
            fill(&mut m);
            let mesh = meshes.add(m);
            let tf = Transform::from_translation((eye - origin.origin).as_vec3());
            let e = commands
                .spawn((
                    CloudEntity,
                    BillboardBatch,
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(b.material.clone().unwrap()),
                    WorldPos(eye),
                    tf,
                    GlobalTransform::from(tf),
                    Visibility::Inherited,
                    NoFrustumCulling,
                    bevy::light::NotShadowCaster,
                ))
                .id();
            b.batch = Some((e, mesh));
        }
    }
}

/// Forgets the batch (the flight's end despawns it with the other cloud entities).
pub(super) fn reset(mut b: ResMut<Billboards>) {
    *b = Billboards::default();
}

/// The cloud ramp at a fractional step (0 brightest .. 15 darkest).
fn ramp(r: &[Vec4; 16], shade: f32) -> Vec3 {
    let i = shade.clamp(0.0, 15.0);
    let (a, f) = (i.floor() as usize, i.fract());
    r[a].truncate().lerp(r[(a + 1).min(15)].truncate(), f)
}

/// The ramp changed enough to show (about a step of 8-bit colour).
fn ramp_moved(a: &[Vec4; 16], b: &[Vec4; 16]) -> bool {
    a.iter().zip(b).any(|(x, y)| (*x - *y).abs().max_element() > 0.004)
}

/// 0..1 from a number.
fn hash(mut z: u64) -> f32 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((z ^ (z >> 31)) & 0xFFFF) as f32 / 65535.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ramp_interpolates_and_clamps() {
        let r: [Vec4; 16] = std::array::from_fn(|i| Vec4::splat(i as f32));
        assert_eq!(ramp(&r, 2.5), Vec3::splat(2.5));
        assert_eq!(ramp(&r, 40.0), Vec3::splat(15.0));
        assert_eq!(ramp(&r, -1.0), Vec3::ZERO);
        assert!(!ramp_moved(&r, &r));
    }
}
