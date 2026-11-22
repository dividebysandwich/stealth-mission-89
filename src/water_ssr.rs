//! Screen space reflections on calm water (shader: assets/shaders/water_ssr.wgsl): with the
//! water's reflections on (`palette_material::water`), on the desktop, the main view's water
//! mirrors what stands on it or flies over it (ships, the shore, aircraft), wobbled by fine
//! ripples, not only the sky. Only when the sea is calm and the eye looks across the water at a
//! flat angle: the palette shader writes the share of the picture the reflection takes into the
//! water's alpha (`Globals::reflections` zw: the main view's size, 0 when off), and this pass,
//! after the main passes and before the tonemapping, marches each such pixel's mirrored ray
//! across the depth buffer and mixes in what it meets. It sets every pixel's alpha back to 1.

use bevy::core_pipeline::FullscreenShader;
use bevy::core_pipeline::schedule::{Core3d, Core3dSystems};
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_resource::binding_types::{
    texture_2d, texture_depth_2d, texture_depth_2d_multisampled, uniform_buffer,
};
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderContext, ViewQuery};
use bevy::render::view::{ExtractedView, ViewDepthTexture, ViewTarget};
use bevy::render::{GpuResourceAppExt, Render, RenderApp, RenderStartup, RenderSystems};

const SHADER: &str = "shaders/water_ssr.wgsl";

/// On a main view's camera: its water mirrors the picture (when `active`).
#[derive(Component, Clone, Copy, Default, ExtractComponent)]
pub struct WaterMirror;

/// What the shader's ripples need from the main world: the floating origin's x, z (wrapped by
/// the noise period, so the ripples keep their place) and the clock.
#[derive(Resource, Clone, Copy, Default, ExtractResource)]
pub struct MirrorFrame {
    on: bool,
    origin: Vec2,
    time: f32,
    /// The sea's level in the render world (m).
    sea: f32,
}

/// The ripples' pattern repeats every so many metres (the floating origin wraps by it).
const RIPPLE_PERIOD: f64 = 4096.0;

/// Whether the water mirrors the picture: with the water's reflections, on the desktop.
pub fn active(settings: &crate::settings::Settings) -> bool {
    crate::palette_material::water(settings) && !settings.vr && !crate::WEBGL2
}

pub struct WaterSsrPlugin;

impl Plugin for WaterSsrPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<WaterMirror>::default(),
            ExtractResourcePlugin::<MirrorFrame>::default(),
        ))
        .init_resource::<MirrorFrame>()
        .add_systems(PostUpdate, frame);
        // WebGL2 has no pass (its GLSL can't load from a depth texture: building the pipeline
        // there stops the game).
        if crate::WEBGL2 {
            return;
        }
        app.add_systems(PostUpdate, depth_readable);
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .init_gpu_resource::<SpecializedRenderPipelines<MirrorPipeline>>()
            .add_systems(RenderStartup, init_pipeline)
            .add_systems(Render, prepare_pipelines.in_set(RenderSystems::Prepare))
            .add_systems(
                Core3d,
                mirror
                    .before(bevy::core_pipeline::tonemapping::tonemapping)
                    .in_set(Core3dSystems::PostProcess),
            );
    }
}

/// The pass reads the view's depth.
fn depth_readable(mut cams: Query<&mut Camera3d, Added<WaterMirror>>) {
    for mut c in &mut cams {
        let usages = TextureUsages::from(c.depth_texture_usages) | TextureUsages::TEXTURE_BINDING;
        c.depth_texture_usages = usages.into();
    }
}

fn frame(
    settings: Res<crate::settings::Settings>,
    origin: Res<crate::world::FloatingOrigin>,
    time: Res<Time>,
    mut f: ResMut<MirrorFrame>,
) {
    f.on = active(&settings);
    let o = origin.origin;
    f.origin =
        Vec2::new(o.x.rem_euclid(RIPPLE_PERIOD) as f32, o.z.rem_euclid(RIPPLE_PERIOD) as f32);
    f.time = (time.elapsed_secs_f64() % 3600.0) as f32;
    f.sea = -o.y as f32;
}

/// The view's matrices (from positions relative to the eye) and the ripples (water_ssr.wgsl
/// `Params`).
#[derive(Clone, Copy, ShaderType)]
struct Params {
    clip_from_eye: Mat4,
    eye_from_clip: Mat4,
    eye: Vec4,
    forward: Vec4,
    ripple: Vec4,
    size: Vec4,
}

#[derive(Resource)]
struct MirrorPipeline {
    layouts: [BindGroupLayoutDescriptor; 2],
    fullscreen: FullscreenShader,
    shader: Handle<Shader>,
}

fn layout(multisampled: bool) -> BindGroupLayoutDescriptor {
    let depth = if multisampled { texture_depth_2d_multisampled() } else { texture_depth_2d() };
    BindGroupLayoutDescriptor::new(
        "water_ssr",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: false }),
                depth,
                uniform_buffer::<Params>(false),
            ),
        ),
    )
}

fn init_pipeline(
    mut commands: Commands,
    fullscreen: Res<FullscreenShader>,
    server: Res<AssetServer>,
) {
    commands.insert_resource(MirrorPipeline {
        layouts: [layout(false), layout(true)],
        fullscreen: fullscreen.clone(),
        shader: server.load(SHADER),
    });
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
struct MirrorKey {
    format: TextureFormat,
    multisampled: bool,
}

impl SpecializedRenderPipeline for MirrorPipeline {
    type Key = MirrorKey;

    fn specialize(&self, key: MirrorKey) -> RenderPipelineDescriptor {
        let mut defs = Vec::new();
        if key.multisampled {
            defs.push("MULTISAMPLED".into());
        }
        RenderPipelineDescriptor {
            label: Some("water_ssr".into()),
            layout: vec![self.layouts[key.multisampled as usize].clone()],
            vertex: self.fullscreen.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: defs,
                targets: vec![Some(ColorTargetState {
                    format: key.format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            ..default()
        }
    }
}

/// A view's pass: its pipeline, and whether its depth is multisampled.
#[derive(Component)]
struct ViewMirror {
    pipeline: CachedRenderPipelineId,
    multisampled: bool,
}

fn prepare_pipelines(
    mut commands: Commands,
    cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<MirrorPipeline>>,
    pipeline: Res<MirrorPipeline>,
    views: Query<(Entity, &ExtractedView, &Msaa), With<WaterMirror>>,
) {
    // By the view's MSAA: its depth texture is made after this (`PrepareResources`), so on the
    // frame MSAA is switched the one here is still the last frame's.
    for (e, view, msaa) in &views {
        let multisampled = msaa.samples() > 1;
        let key = MirrorKey { format: view.target_format, multisampled };
        let id = pipelines.specialize(&cache, &pipeline, key);
        commands.entity(e).insert(ViewMirror { pipeline: id, multisampled });
    }
}

fn mirror(
    view: ViewQuery<(&ViewTarget, &ViewDepthTexture, &ExtractedView, &ViewMirror)>,
    pipeline: Res<MirrorPipeline>,
    cache: Res<PipelineCache>,
    frame: Res<MirrorFrame>,
    mut ctx: RenderContext,
) {
    let (target, depth, extracted, m) = view.into_inner();
    if !frame.on {
        return;
    }
    // A frame of a switch the pipeline did not see yet.
    if (depth.texture.sample_count() > 1) != m.multisampled {
        return;
    }
    let Some(render_pipeline) = cache.get_render_pipeline(m.pipeline) else { return };
    let size = extracted.viewport.zw().as_vec2();
    // Relative to the eye, multiplied out in f64: kilometres from the floating origin the f32
    // world matrices lose the pixels' rays to rounding (metres off on the far water, in bands
    // that jump from pixel column to column).
    let eye = extracted.world_from_view.translation();
    let clip_from_world = extracted.clip_from_world.map(|m| m.as_dmat4()).unwrap_or_else(|| {
        extracted.clip_from_view.as_dmat4()
            * Mat4::from(extracted.world_from_view.affine()).as_dmat4().inverse()
    });
    let clip_from_eye = clip_from_world * bevy::math::DMat4::from_translation(eye.as_dvec3());
    let params = Params {
        clip_from_eye: clip_from_eye.as_mat4(),
        eye_from_clip: clip_from_eye.inverse().as_mat4(),
        eye: eye.extend(extracted.clip_from_view.w_axis.z),
        forward: extracted.world_from_view.forward().extend(0.0),
        ripple: Vec4::new(frame.origin.x, frame.origin.y, frame.time, size.x),
        size: size.extend(extracted.clip_from_view.y_axis.y).extend(frame.sea),
    };
    let mut bytes = encase::UniformBuffer::new(Vec::<u8>::new());
    if bytes.write(&params).is_err() {
        return;
    }
    let device = ctx.render_device().clone();
    let buffer = device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("water_ssr"),
        contents: bytes.as_ref(),
        usage: BufferUsages::UNIFORM,
    });
    let post = target.post_process_write();
    let bind_group = device.create_bind_group(
        Some("water_ssr"),
        &cache.get_bind_group_layout(&pipeline.layouts[m.multisampled as usize]),
        &BindGroupEntries::sequential((post.source, depth.view(), buffer.as_entire_binding())),
    );
    let pass = RenderPassDescriptor {
        label: Some("water_ssr"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: post.destination,
            depth_slice: None,
            resolve_target: None,
            ops: Operations::default(),
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    };
    let mut render_pass = ctx.command_encoder().begin_render_pass(&pass);
    render_pass.set_pipeline(render_pipeline);
    render_pass.set_bind_group(0, &bind_group, &[]);
    render_pass.draw(0..3, 0..1);
}
