//! The desktop mirror in a VR flight: the desktop window shows the left eye's picture.
//!
//! The headset's cameras render into the OpenXR swapchain, so nothing renders to the window
//! (in the menus the VR panel is mirrored, `vr_menu`). A 2D camera on the window that draws
//! nothing of its own (`XrMirror`) makes the window present; after its upscaling (`mirror`, in
//! the `Core2d` schedule) the left eye's swapchain image is drawn over it with Bevy's blit
//! pipeline, letterboxed to keep its shape. That costs one fullscreen pass, not another view of the world.
//! The window stops waiting for its vertical sync in VR, so it never holds the headset back.
//! The mouse cursor on the mirror points into the world through the left eye (`MirrorRay`), so
//! the mouse clicks the cockpit, the kneeboard and the menus as on the desktop.

use bevy::camera::visibility::RenderLayers;
use bevy::core_pipeline::blit::{BlitPipeline, BlitPipelineKey};
use bevy::core_pipeline::schedule::Core2d;
use bevy::core_pipeline::upscaling::upscaling;
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_resource::{
    BindGroup, CachedRenderPipelineId, PipelineCache, RenderPassDescriptor,
    SpecializedRenderPipelines, TextureViewId,
};
use bevy::render::renderer::{RenderContext, ViewQuery};
use bevy::render::texture::ManualTextureViews;
use bevy::render::view::{ExtractedView, ViewTarget};
use bevy::render::{Render, RenderApp, RenderSystems};
use bevy::window::{PresentMode, PrimaryWindow};
use bevy_mod_openxr::render::{OxrFrameProgress, XR_TEXTURE_INDEX};
use bevy_mod_openxr::resources::OxrCurrentSessionConfig;
use bevy_mod_xr::camera::XrCamera;
use bevy_mod_xr::session::session_available;

use crate::camera::MirrorRay;
use crate::settings::AppState;

/// Drawn after everything else on the window.
const MIRROR_ORDER: isize = 1000;

/// The window's mirror camera.
#[derive(Component, Clone, Copy, ExtractComponent)]
pub struct XrMirror;

/// Its blit pipeline (render world).
#[derive(Component)]
struct MirrorPipeline(CachedRenderPipelineId, BlitPipelineKey);

pub struct XrMirrorPlugin;

impl Plugin for XrMirrorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<XrMirror>::default())
            .add_systems(OnEnter(AppState::Flight), spawn_mirror.run_if(session_available))
            .add_systems(
                PostUpdate,
                mirror_ray
                    .after(TransformSystems::Propagate)
                    .run_if(in_state(AppState::Flight).and_then(session_available)),
            )
            .add_systems(OnExit(AppState::Flight), |mut ray: ResMut<MirrorRay>| ray.0 = None);
        let Some(render) = app.get_sub_app_mut(RenderApp) else { return };
        render
            .add_systems(
                Render,
                prepare_pipelines.in_set(RenderSystems::Prepare).ambiguous_with_all(),
            )
            .add_systems(Core2d, mirror.after(upscaling));
    }
}

fn spawn_mirror(mut commands: Commands, mut window: Query<&mut Window, With<PrimaryWindow>>) {
    commands.spawn((
        Camera2d,
        Camera {
            order: MIRROR_ORDER,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        RenderLayers::none(),
        XrMirror,
        DespawnOnExit(AppState::Flight),
    ));
    if let Ok(mut w) = window.single_mut() {
        w.present_mode = PresentMode::AutoNoVsync;
    }
}

fn prepare_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<BlitPipeline>>,
    blit: Res<BlitPipeline>,
    views: Query<(Entity, &ViewTarget, Option<&MirrorPipeline>), With<XrMirror>>,
) {
    for (e, target, have) in &views {
        let Some(target_format) = target.out_texture_view_format() else { continue };
        // The eye's sRGB view samples linear.
        let key =
            BlitPipelineKey { target_format, blend_state: None, samples: 1, source_space: None };
        if have.is_none_or(|p| p.1 != key) {
            let id = pipelines.specialize(&pipeline_cache, &blit, key);
            commands.entity(e).insert(MirrorPipeline(id, key));
        }
    }
}

/// The ray through the mouse cursor on the mirrored left eye (`MirrorRay`): from the eye
/// through the point of its picture under the cursor.
fn mirror_ray(
    window: Query<&Window, With<PrimaryWindow>>,
    config: Option<Res<OxrCurrentSessionConfig>>,
    eyes: Query<(&XrCamera, &GlobalTransform, &Projection)>,
    mut ray: ResMut<MirrorRay>,
) {
    ray.0 = (|| {
        let window = window.single().ok()?;
        let (_, eye, projection) = eyes.iter().find(|(c, ..)| c.0 == 0)?;
        eye_ray(
            window.physical_cursor_position()?,
            window.physical_size(),
            config?.resolution,
            eye,
            projection.get_clip_from_view(),
        )
    })();
}

/// The ray from an eye (`eye`, `clip_from_view`) whose `picture` (pixels) is letterboxed in a
/// `window` through the point of it under the `cursor` (window pixels): none beside it.
fn eye_ray(
    cursor: Vec2,
    window: UVec2,
    picture: UVec2,
    eye: &GlobalTransform,
    clip_from_view: Mat4,
) -> Option<Ray3d> {
    let [x, y, w, h] = letterbox(picture, UVec4::new(0, 0, window.x, window.y));
    let uv = (cursor - Vec2::new(x, y)) / Vec2::new(w, h);
    if !(0.0..=1.0).contains(&uv.x) || !(0.0..=1.0).contains(&uv.y) {
        return None;
    }
    // A point in the view under the cursor (any depth in the frustum is in front).
    let ndc = Vec3::new(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, 0.5);
    let at = eye.to_matrix() * clip_from_view.inverse();
    let origin = eye.translation();
    let dir = Dir3::new(at.project_point3(ndc) - origin).ok()?;
    Some(Ray3d::new(origin, dir))
}

/// Where a picture `src` (w, h) goes letterboxed in a view `dst` (x, y, w, h): x, y, w, h.
fn letterbox(src: UVec2, dst: UVec4) -> [f32; 4] {
    let (sw, sh) = (src.x.max(1) as f32, src.y.max(1) as f32);
    let (x, y, w, h) = (dst.x as f32, dst.y as f32, dst.z.max(1) as f32, dst.w.max(1) as f32);
    let k = (w / sw).min(h / sh);
    let (fw, fh) = (sw * k, sh * k);
    [x + (w - fw) / 2.0, y + (h - fh) / 2.0, fw, fh]
}

/// The left eye's picture over the mirror camera's (black) output.
fn mirror(
    view: ViewQuery<(&ViewTarget, &ExtractedView, &MirrorPipeline), With<XrMirror>>,
    progress: Option<Res<OxrFrameProgress>>,
    textures: Res<ManualTextureViews>,
    pipeline_cache: Res<PipelineCache>,
    blit: Res<BlitPipeline>,
    mut cache: Local<Option<(TextureViewId, BindGroup)>>,
    mut ctx: RenderContext,
) {
    let (target, extracted, pipeline) = view.into_inner();
    // Only an eye picture rendered this frame (its image acquired and waited for).
    if !progress.is_some_and(|p| p.waited) {
        return;
    }
    let Some(eye) = textures.get(&bevy::camera::ManualTextureViewHandle(XR_TEXTURE_INDEX)) else {
        return;
    };
    let Some(pipe) = pipeline_cache.get_render_pipeline(pipeline.0) else { return };
    let bind_group = match &mut *cache {
        Some((id, bg)) if eye.texture_view.id() == *id => bg,
        slot => {
            let bg =
                blit.create_bind_group(ctx.render_device(), &eye.texture_view, &pipeline_cache);
            &mut slot.insert((eye.texture_view.id(), bg)).1
        }
    };
    let Some(out) = target.out_texture_color_attachment(Some(LinearRgba::BLACK)) else {
        return;
    };
    let [x, y, w, h] = letterbox(eye.size, extracted.viewport);
    let mut pass = ctx.command_encoder().begin_render_pass(&RenderPassDescriptor {
        label: Some("xr_mirror"),
        color_attachments: &[Some(out)],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_viewport(x, y, w, h, 0.0, 1.0);
    pass.set_pipeline(pipe);
    pass.set_bind_group(0, &*bind_group, &[]);
    pass.draw(0..3, 0..1);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tall eye picture sits pillarboxed in a wide window, a wide one letterboxed.
    #[test]
    fn letterboxed() {
        let tall = letterbox(UVec2::new(3072, 3264), UVec4::new(0, 0, 1280, 768));
        assert_eq!(tall[1], 0.0);
        assert_eq!(tall[3], 768.0);
        assert!((tall[2] - 768.0 * 3072.0 / 3264.0).abs() < 0.01);
        assert!((tall[0] - (1280.0 - tall[2]) / 2.0).abs() < 0.01);
        let wide = letterbox(UVec2::new(2000, 1000), UVec4::new(0, 0, 1000, 1000));
        assert_eq!((wide[0], wide[2], wide[3]), (0.0, 1000.0, 500.0));
        assert_eq!(wide[1], 250.0);
    }

    /// The cursor in the middle of the mirrored eye looks where the eye looks, at the top edge
    /// half its field of view up; beside the pillarboxed picture there is no ray.
    #[test]
    fn cursor_through_the_eye() {
        let clip = Mat4::perspective_infinite_reverse_rh(1.0, 1.0, 0.05);
        let eye =
            GlobalTransform::from(Transform::from_xyz(1.0, 2.0, 3.0).looking_to(Vec3::X, Vec3::Y));
        let (window, picture) = (UVec2::new(1600, 900), UVec2::new(1000, 1000));
        let ray = eye_ray(Vec2::new(800.0, 450.0), window, picture, &eye, clip).unwrap();
        assert!(ray.origin.distance(Vec3::new(1.0, 2.0, 3.0)) < 1e-4);
        assert!(ray.direction.distance(Vec3::X) < 1e-4, "{:?}", ray.direction);
        let up = eye_ray(Vec2::new(800.0, 0.0), window, picture, &eye, clip).unwrap();
        assert!((up.direction.y.atan2(up.direction.x) - 0.5).abs() < 1e-3, "{:?}", up.direction);
        assert!(eye_ray(Vec2::new(100.0, 450.0), window, picture, &eye, clip).is_none());
    }
}
