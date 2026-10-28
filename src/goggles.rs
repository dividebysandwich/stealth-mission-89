//! The night vision goggles' tube (`Effects::nvg`, desktop cockpit views): the goggles see a
//! round field of view, black around it. A black ring just in front of the eye with a soft
//! inner edge, as tall as the view; opaque beyond its feather, so the scene behind it is never
//! shaded. The headset's own field of view is round enough: none in VR.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::camera::{EyeCamera, ViewState};
use crate::settings::{AppState, Settings};

pub struct GogglesPlugin;

impl Plugin for GogglesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            tube.after(crate::camera::RigSystems)
                .before(bevy::transform::TransformSystems::Propagate)
                .run_if(in_state(AppState::Flight)),
        );
    }
}

/// The ring's distance in front of the eye (m; beyond the near plane, before the cockpit).
const DISTANCE: f32 = 0.08;
/// The tube's radius as a share of the view's half height, its soft edge's width (share of the
/// radius) and the ring's outer radius (the view's corners).
const RADIUS: f32 = 1.0;
const FEATHER: f32 = 0.08;
const OUTER: f32 = 8.0;
const SIDES: usize = 64;

#[derive(Component)]
struct Tube;

/// A unit ring: transparent at radius 1, black from 1 + `FEATHER` out to `OUTER`.
fn ring_mesh() -> Mesh {
    let mut pos = Vec::new();
    let mut colour = Vec::new();
    for (r, a) in [(1.0, 0.0), (1.0 + FEATHER, 1.0), (OUTER, 1.0)] {
        for s in 0..SIDES {
            let t = s as f32 / SIDES as f32 * std::f32::consts::TAU;
            pos.push([r * t.cos(), r * t.sin(), 0.0]);
            colour.push([0.0, 0.0, 0.0, a]);
        }
    }
    let mut idx = Vec::new();
    let n = SIDES as u32;
    for ring in 0..2 {
        for s in 0..n {
            let (a, b) = (ring * n + s, ring * n + (s + 1) % n);
            idx.extend_from_slice(&[a, b + n, b, a, a + n, b + n]);
        }
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colour)
        .with_inserted_indices(Indices::U32(idx))
}

#[allow(clippy::too_many_arguments)]
fn tube(
    mut commands: Commands,
    settings: Res<Settings>,
    view: Res<ViewState>,
    hud: Option<Res<crate::hud::HudState>>,
    cams: Query<(Entity, &Projection), With<EyeCamera>>,
    mut tubes: Query<(&mut Transform, &mut Visibility), With<Tube>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let on = settings.effects.nvg
        && !settings.vr
        && view.first_person()
        && view.view != crate::camera::View::Pilot
        && hud.is_some_and(|h| h.nvg);
    let Ok((cam, projection)) = cams.single() else { return };
    let Ok((mut tf, mut vis)) = tubes.single_mut() else {
        if on {
            commands.entity(cam).with_child((
                Tube,
                Mesh3d(meshes.add(ring_mesh())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                })),
                Transform::default(),
                Visibility::Hidden,
                bevy::light::NotShadowCaster,
                bevy::camera::visibility::NoFrustumCulling,
                bevy::camera::visibility::RenderLayers::layer(crate::camera::MAIN_BACKGROUND_LAYER),
            ));
        }
        return;
    };
    vis.set_if_neq(if on { Visibility::Inherited } else { Visibility::Hidden });
    if !on {
        return;
    }
    // As tall as the view (it follows the zoom).
    let fov = match projection {
        Projection::Perspective(p) => p.fov,
        _ => 1.0,
    };
    let r = DISTANCE * (fov / 2.0).tan() * RADIUS;
    *tf = Transform::from_xyz(0.0, 0.0, -DISTANCE).with_scale(Vec3::new(r, r, 1.0));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_is_closed() {
        let m = ring_mesh();
        let n = m.count_vertices();
        assert_eq!(n, 3 * SIDES);
        let Some(Indices::U32(idx)) = m.indices() else { panic!() };
        assert!(idx.iter().all(|&i| (i as usize) < n));
        assert_eq!(idx.len(), 2 * SIDES * 6);
    }
}
