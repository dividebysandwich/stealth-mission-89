//! The layered clouds' render world: noise textures, the maps around the eye (rebuilt when the
//! clouds or the map's place change), and per view the march, the temporal resolve into the
//! view's history and the composite (drawn between the opaque and the transparent pass, like
//! the hull march).

use std::collections::HashMap;

use bevy::{
    core_pipeline::{
        Core3dSystems, FullscreenShader,
        core_3d::CORE_3D_DEPTH_FORMAT,
        core_3d::{main_opaque_pass_3d, main_transparent_pass_3d},
        schedule::Core3d,
    },
    math::DVec3,
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        camera::ExtractedCamera,
        extract_resource::ExtractResourcePlugin,
        render_resource::{
            binding_types::{
                sampler, texture_2d, texture_3d, texture_depth_2d, texture_depth_2d_multisampled,
                uniform_buffer,
            },
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery},
        texture::{CachedTexture, TextureCache},
        view::{
            ExtractedView, Msaa, RetainedViewEntity, ViewDepthTexture, ViewTarget, ViewUniform,
            ViewUniformOffset, ViewUniforms,
        },
    },
    shader::ShaderDefVal,
};

use super::{LayeredClouds, LayeredFrame, LayeredGlobals, MAP_SIZE, noise};
use crate::render::{CloudCamera, PREMULTIPLIED_OVER, storage_buffers};

const SHADER_DIR: &str = "embedded://bevy_fast_clouds/shaders/";
const COLOUR_FORMAT: TextureFormat = TextureFormat::Rgba16Float;
const WEATHER_FORMAT: TextureFormat = TextureFormat::Rgba16Float;
const DECK_FORMAT: TextureFormat = TextureFormat::R16Float;
const MAP_DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;
/// A history not drawn for this many frames is dropped.
const HISTORY_KEEP: u64 = 120;

#[derive(Clone, Copy, ShaderType, Default)]
struct MapUniform {
    anchor: Vec4,
    field: Vec4,
    cells: Vec4,
    weights: Vec4,
}

#[derive(Clone, Copy, ShaderType)]
struct Instances {
    c: [Vec4; 2 * super::MAX_CONVECTIVE],
}

impl Default for Instances {
    fn default() -> Self {
        Self { c: [Vec4::ZERO; 2 * super::MAX_CONVECTIVE] }
    }
}

#[derive(Clone, Copy, ShaderType, Default)]
struct LayeredView {
    prev_clip_from_world: Mat4,
    shift: Vec4,
}

pub(crate) fn build_render(app: &mut App) {
    app.init_resource::<LayeredClouds>()
        .init_resource::<LayeredFrame>()
        .add_plugins(ExtractResourcePlugin::<LayeredFrame>::default())
        .add_systems(
            PostUpdate,
            (super::upload_layered_frame, clear_hulls)
                .chain()
                .in_set(crate::CloudSystems)
                .after(crate::render::upload_cloud_frame),
        );
    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app
        .init_resource::<Histories>()
        .add_systems(RenderStartup, init_layered)
        .add_systems(
            Render,
            (prepare_layered, prepare_layered_views)
                .chain()
                .in_set(RenderSystems::PrepareResources),
        )
        .add_systems(
            Core3d,
            draw_layered_clouds
                .after(main_opaque_pass_3d)
                .before(main_transparent_pass_3d)
                .in_set(Core3dSystems::MainPass),
        );
}

/// While the layered clouds draw, the hull march draws nothing.
fn clear_hulls(layered: Res<LayeredFrame>, mut frame: ResMut<crate::render::CloudFrame>) {
    if layered.on {
        frame.volumes.clear();
        frame.spheres.clear();
        frame.batches = [UVec2::ZERO; 8];
    }
}

#[derive(Resource)]
struct LayeredPipelines {
    /// Held so the shaders' import stays loaded.
    _common: Handle<Shader>,
    maps_shader: Handle<Shader>,
    march_shader: Handle<Shader>,
    resolve_shader: Handle<Shader>,
    composite_shader: Handle<Shader>,
    fullscreen: FullscreenShader,
    map_layout: BindGroupLayoutDescriptor,
    /// Single sampled and multisampled scene depth.
    march_layouts: [BindGroupLayoutDescriptor; 2],
    resolve_layout: BindGroupLayoutDescriptor,
    composite_layouts: [BindGroupLayoutDescriptor; 2],
    composite_depth_layout: BindGroupLayoutDescriptor,
    webgl2: bool,
    weather: Option<CachedRenderPipelineId>,
    coverage: Option<CachedRenderPipelineId>,
    deck: Option<CachedRenderPipelineId>,
    resolve: Option<CachedRenderPipelineId>,
    march: HashMap<(bool, u32, u32), CachedRenderPipelineId>,
    composite: HashMap<(TextureFormat, u32), [CachedRenderPipelineId; 2]>,
    clamp_linear: Sampler,
    repeat_linear: Sampler,
    nearest: Sampler,
}

/// The maps, the noise and the uniforms.
#[derive(Resource)]
struct LayeredMaps {
    weather: TextureView,
    weather_depth: TextureView,
    /// The convective clouds' coverage (max of their soft footprints), highest top, lowest
    /// base (negated), storm coverage.
    coverage: TextureView,
    deck: TextureView,
    noise: Option<(TextureView, TextureView)>,
    weather_gen: u64,
    deck_gen: u64,
    globals: UniformBuffer<LayeredGlobals>,
    map: UniformBuffer<MapUniform>,
    instances: UniformBuffer<Instances>,
    views: DynamicUniformBuffer<LayeredView>,
}

/// A view's history: two textures (read one, write the other), and what the last frame was
/// drawn with.
struct History {
    textures: [TextureView; 2],
    size: UVec2,
    current: usize,
    prev: Option<(Mat4, DVec3)>,
    seen: u64,
}

#[derive(Resource, Default)]
struct Histories {
    views: HashMap<RetainedViewEntity, History>,
    frame: u64,
}

/// The view's pipelines, textures and uniform offset this frame.
#[derive(Component)]
struct ViewLayered {
    march: CachedRenderPipelineId,
    composite: [CachedRenderPipelineId; 2],
    offset: u32,
    colour: CachedTexture,
    depths: CachedTexture,
    history_read: TextureView,
    history_write: TextureView,
}

fn map_texture(device: &RenderDevice, label: &'static str, format: TextureFormat) -> TextureView {
    device
        .create_texture(&TextureDescriptor {
            label: Some(label),
            size: Extent3d { width: MAP_SIZE, height: MAP_SIZE, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&TextureViewDescriptor::default())
}

fn init_layered(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    fullscreen: Res<FullscreenShader>,
    device: Res<RenderDevice>,
) {
    let webgl2 = !storage_buffers(&device);
    let scene_depth = |multisampled: bool| {
        if multisampled {
            texture_depth_2d_multisampled()
        } else if webgl2 {
            texture_2d(TextureSampleType::Float { filterable: false })
        } else {
            texture_depth_2d()
        }
    };
    let float = TextureSampleType::Float { filterable: true };
    let unfiltered = TextureSampleType::Float { filterable: false };
    let march_layout = |label: &'static str, ms: bool| {
        BindGroupLayoutDescriptor::new(
            label,
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    scene_depth(ms),
                    uniform_buffer::<ViewUniform>(true),
                    uniform_buffer::<LayeredGlobals>(false),
                    texture_2d(float),
                    texture_2d(float),
                    sampler(SamplerBindingType::Filtering),
                    texture_3d(float),
                    texture_3d(float),
                    sampler(SamplerBindingType::Filtering),
                    sampler(SamplerBindingType::NonFiltering),
                    texture_2d(float),
                ),
            ),
        )
    };
    let composite_layout = |label: &'static str, ms: bool| {
        BindGroupLayoutDescriptor::new(
            label,
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(float),
                    sampler(SamplerBindingType::Filtering),
                    texture_2d(unfiltered),
                    uniform_buffer::<ViewUniform>(true),
                    uniform_buffer::<LayeredGlobals>(false),
                    scene_depth(ms),
                    sampler(SamplerBindingType::NonFiltering),
                ),
            ),
        )
    };
    let shader = |name: &str| asset_server.load(format!("{SHADER_DIR}{name}"));
    let linear = |label, mode| {
        device.create_sampler(&SamplerDescriptor {
            label: Some(label),
            address_mode_u: mode,
            address_mode_v: mode,
            address_mode_w: mode,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..default()
        })
    };
    commands.insert_resource(LayeredPipelines {
        _common: shader("layered_common.wgsl"),
        maps_shader: shader("layered_maps.wgsl"),
        march_shader: shader("layered_cloud.wgsl"),
        resolve_shader: shader("layered_resolve.wgsl"),
        composite_shader: shader("layered_composite.wgsl"),
        fullscreen: fullscreen.clone(),
        map_layout: BindGroupLayoutDescriptor::new(
            "layered_map_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::VERTEX_FRAGMENT,
                (uniform_buffer::<MapUniform>(false), uniform_buffer::<Instances>(false)),
            ),
        ),
        march_layouts: [
            march_layout("layered_march_layout", false),
            march_layout("layered_march_layout_msaa", true),
        ],
        resolve_layout: BindGroupLayoutDescriptor::new(
            "layered_resolve_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(unfiltered),
                    texture_2d(unfiltered),
                    texture_2d(float),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<ViewUniform>(true),
                    uniform_buffer::<LayeredView>(true),
                    uniform_buffer::<LayeredGlobals>(false),
                ),
            ),
        ),
        composite_layouts: [
            composite_layout("layered_composite_layout", false),
            composite_layout("layered_composite_layout_msaa", true),
        ],
        composite_depth_layout: BindGroupLayoutDescriptor::new(
            "layered_composite_depth_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(float),
                    sampler(SamplerBindingType::Filtering),
                    texture_2d(unfiltered),
                    uniform_buffer::<ViewUniform>(true),
                    uniform_buffer::<LayeredGlobals>(false),
                ),
            ),
        ),
        webgl2,
        weather: None,
        coverage: None,
        deck: None,
        resolve: None,
        march: HashMap::new(),
        composite: HashMap::new(),
        clamp_linear: linear("layered_clamp", AddressMode::ClampToEdge),
        repeat_linear: linear("layered_repeat", AddressMode::Repeat),
        nearest: device
            .create_sampler(&SamplerDescriptor { label: Some("layered_nearest"), ..default() }),
    });
    commands.insert_resource(LayeredMaps {
        weather: map_texture(&device, "layered_weather", WEATHER_FORMAT),
        weather_depth: map_texture(&device, "layered_weather_depth", MAP_DEPTH_FORMAT),
        coverage: map_texture(&device, "layered_coverage", WEATHER_FORMAT),
        deck: map_texture(&device, "layered_deck", DECK_FORMAT),
        noise: None,
        weather_gen: 0,
        deck_gen: 0,
        globals: UniformBuffer::default(),
        map: UniformBuffer::default(),
        instances: UniformBuffer::default(),
        views: DynamicUniformBuffer::default(),
    });
}

fn depth_defs(pipelines: &LayeredPipelines, multisampled: bool) -> Vec<ShaderDefVal> {
    let mut defs = Vec::new();
    if multisampled {
        defs.push("MULTISAMPLED".into());
    } else if pipelines.webgl2 {
        defs.push("WEBGL2".into());
    }
    defs
}

#[allow(clippy::too_many_arguments)]
fn fullscreen_pipeline(
    pipelines: &LayeredPipelines,
    label: &'static str,
    layout: BindGroupLayoutDescriptor,
    shader: Handle<Shader>,
    entry: &'static str,
    defs: Vec<ShaderDefVal>,
    targets: Vec<Option<ColorTargetState>>,
    depth_stencil: Option<DepthStencilState>,
    samples: u32,
) -> RenderPipelineDescriptor {
    RenderPipelineDescriptor {
        label: Some(label.into()),
        layout: vec![layout],
        vertex: pipelines.fullscreen.to_vertex_state(),
        depth_stencil,
        multisample: MultisampleState { count: samples, ..default() },
        fragment: Some(FragmentState {
            shader,
            shader_defs: defs,
            entry_point: Some(entry.into()),
            targets,
        }),
        ..default()
    }
}

fn target(format: TextureFormat) -> Option<ColorTargetState> {
    Some(ColorTargetState { format, blend: None, write_mask: ColorWrites::ALL })
}

/// Pipelines that don't depend on the view, the noise textures, the uniforms, and the maps
/// when they changed.
#[allow(clippy::too_many_arguments)]
fn prepare_layered(
    frame: Res<LayeredFrame>,
    mut pipelines: ResMut<LayeredPipelines>,
    mut maps: ResMut<LayeredMaps>,
    pipeline_cache: Res<PipelineCache>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    if !frame.on {
        return;
    }
    let p = &mut *pipelines;
    if p.weather.is_none() {
        p.weather = Some(pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("layered_weather_map".into()),
            layout: vec![p.map_layout.clone()],
            vertex: VertexState {
                shader: p.maps_shader.clone(),
                entry_point: Some("quad".into()),
                buffers: vec![],
                ..default()
            },
            depth_stencil: Some(DepthStencilState {
                format: MAP_DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(CompareFunction::Less),
                stencil: default(),
                bias: default(),
            }),
            fragment: Some(FragmentState {
                shader: p.maps_shader.clone(),
                entry_point: Some("quad_fragment".into()),
                targets: vec![target(WEATHER_FORMAT)],
                ..default()
            }),
            ..default()
        }));
        // The coverage: the same squares, each texel keeping the largest of each channel.
        let max = BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Max,
        };
        p.coverage = Some(pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("layered_coverage_map".into()),
            layout: vec![p.map_layout.clone()],
            vertex: VertexState {
                shader: p.maps_shader.clone(),
                entry_point: Some("quad".into()),
                buffers: vec![],
                ..default()
            },
            fragment: Some(FragmentState {
                shader: p.maps_shader.clone(),
                entry_point: Some("coverage_fragment".into()),
                targets: vec![Some(ColorTargetState {
                    format: WEATHER_FORMAT,
                    blend: Some(BlendState { color: max, alpha: max }),
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            ..default()
        }));
        let deck = fullscreen_pipeline(
            p,
            "layered_deck_map",
            p.map_layout.clone(),
            p.maps_shader.clone(),
            "deck_field",
            vec![],
            vec![target(DECK_FORMAT)],
            None,
            1,
        );
        p.deck = Some(pipeline_cache.queue_render_pipeline(deck));
        let resolve = fullscreen_pipeline(
            p,
            "layered_resolve",
            p.resolve_layout.clone(),
            p.resolve_shader.clone(),
            "fragment",
            vec![],
            vec![target(COLOUR_FORMAT)],
            None,
            1,
        );
        p.resolve = Some(pipeline_cache.queue_render_pipeline(resolve));
    }

    // The noise, once made.
    if maps.noise.is_none()
        && let Some(set) = noise::get()
    {
        let texture3d = |label, n: usize, data: &[u8]| {
            let size =
                Extent3d { width: n as u32, height: n as u32, depth_or_array_layers: n as u32 };
            let texture = device.create_texture(&TextureDescriptor {
                label: Some(label),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D3,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            });
            queue.write_texture(
                texture.as_image_copy(),
                data,
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(n as u32 * 4),
                    rows_per_image: Some(n as u32),
                },
                size,
            );
            texture.create_view(&TextureViewDescriptor::default())
        };
        maps.noise = Some((
            texture3d("layered_shape_noise", noise::SHAPE, &set.shape),
            texture3d("layered_detail_noise", noise::DETAIL, &set.detail),
        ));
    }

    maps.globals.set(frame.globals);
    maps.globals.write_buffer(&device, &queue);

    // The maps, when they changed (and their pipelines are ready).
    let weather_dirty = maps.weather_gen != frame.weather_gen;
    let deck_dirty = maps.deck_gen != frame.deck_gen;
    if !(weather_dirty || deck_dirty) {
        return;
    }
    let (Some(weather), Some(coverage), Some(deck)) = (
        p.weather.and_then(|id| pipeline_cache.get_render_pipeline(id)),
        p.coverage.and_then(|id| pipeline_cache.get_render_pipeline(id)),
        p.deck.and_then(|id| pipeline_cache.get_render_pipeline(id)),
    ) else {
        return;
    };
    let (seed, unit, oct) = frame.deck_key;
    let f = |bits: u32| f32::from_bits(bits);
    maps.map.set(MapUniform {
        anchor: Vec4::new(frame.anchor.x as f32, frame.anchor.y as f32, super::MAP_HALF, 0.0),
        field: Vec4::new((seed & 0xFFFF) as f32, f(unit), 0.0, 0.0),
        cells: Vec4::new(f(oct[0]), f(oct[2]), f(oct[4]), 1.0),
        weights: Vec4::new(f(oct[1]), f(oct[3]), f(oct[5]), 0.0),
    });
    maps.map.write_buffer(&device, &queue);
    let mut instances = Instances::default();
    for (i, c) in frame.instances.iter().enumerate() {
        instances.c[2 * i] = c[0];
        instances.c[2 * i + 1] = c[1];
    }
    maps.instances.set(instances);
    maps.instances.write_buffer(&device, &queue);
    let (Some(map_binding), Some(inst_binding)) = (maps.map.binding(), maps.instances.binding())
    else {
        return;
    };
    let bind_group = device.create_bind_group(
        "layered_map_bind_group",
        &pipeline_cache.get_bind_group_layout(&p.map_layout),
        &BindGroupEntries::sequential((map_binding, inst_binding)),
    );
    let mut encoder =
        device.create_command_encoder(&CommandEncoderDescriptor { label: Some("layered_maps") });
    if weather_dirty {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("layered_weather_map"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &maps.weather,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(LinearRgba::new(8.0, 0.0, 0.0, 0.0).into()),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &maps.weather_depth,
                depth_ops: Some(Operations { load: LoadOp::Clear(1.0), store: StoreOp::Store }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(weather);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..6, 0..frame.instances.len() as u32);
        drop(pass);
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("layered_coverage_map"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &maps.coverage,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    // No cover, no top, no base (-base: the lowest base wins the max).
                    load: LoadOp::Clear(LinearRgba::new(0.0, -60000.0, -60000.0, 0.0).into()),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(coverage);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..6, 0..frame.instances.len() as u32);
    }
    if deck_dirty {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("layered_deck_map"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &maps.deck,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(LinearRgba::NONE.into()),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(deck);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
    queue.submit([encoder.finish()]);
    maps.weather_gen = frame.weather_gen;
    maps.deck_gen = frame.deck_gen;
}

/// Per view: the march and composite pipelines, the march's textures, the history and the
/// reprojection uniform.
#[allow(clippy::too_many_arguments)]
fn prepare_layered_views(
    mut commands: Commands,
    frame: Res<LayeredFrame>,
    views: Query<(Entity, &ExtractedView, &ViewTarget, &Msaa), With<CloudCamera>>,
    mut pipelines: ResMut<LayeredPipelines>,
    mut maps: ResMut<LayeredMaps>,
    mut histories: ResMut<Histories>,
    mut cache: ResMut<TextureCache>,
    pipeline_cache: Res<PipelineCache>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    histories.frame += 1;
    let now = histories.frame;
    histories.views.retain(|_, h| now - h.seen < HISTORY_KEEP);
    if !frame.on {
        for (entity, ..) in &views {
            commands.entity(entity).remove::<ViewLayered>();
        }
        return;
    }
    let scale = frame.resolution_scale.max(1);
    maps.views.clear();
    let p = &mut *pipelines;
    for (entity, view, view_target, msaa) in &views {
        let ms = msaa.samples() > 1;
        let key = (ms, frame.steps, scale);
        let march = if let Some(id) = p.march.get(&key) {
            *id
        } else {
            let mut defs = depth_defs(p, ms);
            defs.push(ShaderDefVal::Int("MARCH_STEPS".into(), frame.steps as i32));
            defs.push(ShaderDefVal::Int("RES_SCALE".into(), scale as i32));
            let id = pipeline_cache.queue_render_pipeline(fullscreen_pipeline(
                p,
                "layered_march",
                p.march_layouts[ms as usize].clone(),
                p.march_shader.clone(),
                "fragment",
                defs,
                vec![target(COLOUR_FORMAT), target(COLOUR_FORMAT)],
                None,
                1,
            ));
            p.march.insert(key, id);
            id
        };
        let ckey = (view_target.main_texture_format(), msaa.samples());
        let composite = if let Some(ids) = p.composite.get(&ckey) {
            *ids
        } else {
            let colour = fullscreen_pipeline(
                p,
                "layered_composite",
                p.composite_layouts[ms as usize].clone(),
                p.composite_shader.clone(),
                "fragment",
                depth_defs(p, ms),
                vec![Some(ColorTargetState {
                    format: ckey.0,
                    blend: Some(PREMULTIPLIED_OVER),
                    write_mask: ColorWrites::ALL,
                })],
                None,
                ckey.1,
            );
            let depth = fullscreen_pipeline(
                p,
                "layered_depth",
                p.composite_depth_layout.clone(),
                p.composite_shader.clone(),
                "depth",
                depth_defs(p, ms),
                vec![Some(ColorTargetState {
                    format: ckey.0,
                    blend: None,
                    write_mask: ColorWrites::empty(),
                })],
                Some(DepthStencilState {
                    format: CORE_3D_DEPTH_FORMAT,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(CompareFunction::GreaterEqual),
                    stencil: default(),
                    bias: default(),
                }),
                ckey.1,
            );
            let ids = [colour, depth].map(|d| pipeline_cache.queue_render_pipeline(d));
            p.composite.insert(ckey, ids);
            ids
        };

        let size = ((view.viewport.zw() + (scale - 1)) / scale).max(UVec2::ONE);
        let mut texture = |label| {
            cache.get(
                &device,
                TextureDescriptor {
                    label: Some(label),
                    size: Extent3d { width: size.x, height: size.y, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: COLOUR_FORMAT,
                    usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                },
            )
        };
        let (colour, depths) = (texture("layered_colour"), texture("layered_depths"));

        // The history: kept while the view keeps its size and is drawn every frame.
        let clip_from_world = view
            .clip_from_world
            .unwrap_or_else(|| view.clip_from_view * view.world_from_view.affine().inverse());
        let key = view.retained_view_entity;
        let fresh = |device: &RenderDevice| History {
            textures: std::array::from_fn(|_| {
                device
                    .create_texture(&TextureDescriptor {
                        label: Some("layered_history"),
                        size: Extent3d { width: size.x, height: size.y, depth_or_array_layers: 1 },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: TextureDimension::D2,
                        format: COLOUR_FORMAT,
                        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                    })
                    .create_view(&TextureViewDescriptor::default())
            }),
            size,
            current: 0,
            prev: None,
            seen: 0,
        };
        let h = histories.views.entry(key).or_insert_with(|| fresh(&device));
        if h.size != size {
            *h = fresh(&device);
        }
        // `SM89_CLOUD_TAA=0`: no accumulation over frames (debugging).
        let taa = std::env::var("SM89_CLOUD_TAA").map_or(true, |v| v != "0");
        let valid = taa && h.seen + 1 == now;
        let uniform = match (h.prev, valid) {
            (Some((m, origin)), true) => LayeredView {
                prev_clip_from_world: m,
                shift: (frame.origin - origin).as_vec3().extend(1.0),
            },
            _ => LayeredView::default(),
        };
        h.prev = Some((clip_from_world, frame.origin));
        h.seen = now;
        h.current ^= 1;
        let (read, write) = (h.textures[h.current ^ 1].clone(), h.textures[h.current].clone());
        let offset = maps.views.push(&uniform);
        commands.entity(entity).insert(ViewLayered {
            march,
            composite,
            offset,
            colour,
            depths,
            history_read: read,
            history_write: write,
        });
    }
    maps.views.write_buffer(&device, &queue);
}

#[allow(clippy::too_many_arguments)]
fn draw_layered_clouds(
    view: ViewQuery<(
        &ExtractedCamera,
        &ViewTarget,
        &ViewDepthTexture,
        &ViewUniformOffset,
        &ViewLayered,
        &Msaa,
    )>,
    frame: Res<LayeredFrame>,
    pipelines: Option<Res<LayeredPipelines>>,
    maps: Option<Res<LayeredMaps>>,
    pipeline_cache: Res<PipelineCache>,
    view_uniforms: Res<ViewUniforms>,
    mut ctx: RenderContext,
) {
    let (Some(p), Some(maps)) = (pipelines, maps) else {
        return;
    };
    if !frame.on || maps.weather_gen != frame.weather_gen || maps.deck_gen != frame.deck_gen {
        return;
    }
    let Some((shape, detail)) = maps.noise.as_ref() else {
        return;
    };
    let (camera, target, depth, view_offset, layered, msaa) = view.into_inner();
    let get = |id: CachedRenderPipelineId| pipeline_cache.get_render_pipeline(id);
    let (Some(march), Some(resolve), Some(composite), Some(depth_write)) = (
        get(layered.march),
        p.resolve.and_then(get),
        get(layered.composite[0]),
        get(layered.composite[1]),
    ) else {
        return;
    };
    let (Some(view_binding), Some(globals), Some(views)) =
        (view_uniforms.uniforms.binding(), maps.globals.binding(), maps.views.binding())
    else {
        return;
    };
    let ms = msaa.samples() > 1;
    let device = ctx.render_device().clone();
    let layout = |l: &BindGroupLayoutDescriptor| pipeline_cache.get_bind_group_layout(l);
    let cleared = |view| RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: Operations { load: LoadOp::Clear(LinearRgba::NONE.into()), store: StoreOp::Store },
    };

    // March.
    let bind_group = device.create_bind_group(
        "layered_march_bind_group",
        &layout(&p.march_layouts[ms as usize]),
        &BindGroupEntries::sequential((
            depth.view(),
            view_binding.clone(),
            globals.clone(),
            &maps.weather,
            &maps.deck,
            &p.clamp_linear,
            shape,
            detail,
            &p.repeat_linear,
            &p.nearest,
            &maps.coverage,
        )),
    );
    {
        let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("layered_march"),
            color_attachments: &[
                Some(cleared(&layered.colour.default_view)),
                Some(cleared(&layered.depths.default_view)),
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_render_pipeline(march);
        pass.set_bind_group(0, &bind_group, &[view_offset.offset]);
        pass.draw(0..3, 0..1);
    }

    // Temporal resolve into the history.
    let bind_group = device.create_bind_group(
        "layered_resolve_bind_group",
        &layout(&p.resolve_layout),
        &BindGroupEntries::sequential((
            &layered.colour.default_view,
            &layered.depths.default_view,
            &layered.history_read,
            &p.clamp_linear,
            view_binding.clone(),
            views,
            globals.clone(),
        )),
    );
    {
        let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("layered_resolve"),
            color_attachments: &[Some(cleared(&layered.history_write))],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_render_pipeline(resolve);
        pass.set_bind_group(0, &bind_group, &[view_offset.offset, layered.offset]);
        pass.draw(0..3, 0..1);
    }

    // Over the view, then into its depth.
    let bind_group = device.create_bind_group(
        "layered_composite_bind_group",
        &layout(&p.composite_layouts[ms as usize]),
        &BindGroupEntries::sequential((
            &layered.history_write,
            &p.clamp_linear,
            &layered.depths.default_view,
            view_binding.clone(),
            globals.clone(),
            depth.view(),
            &p.nearest,
        )),
    );
    {
        let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("layered_composite"),
            color_attachments: &[Some(target.get_color_attachment())],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        if let Some(viewport) = camera.viewport.as_ref() {
            pass.set_camera_viewport(viewport);
        }
        pass.set_render_pipeline(composite);
        pass.set_bind_group(0, &bind_group, &[view_offset.offset]);
        pass.draw(0..3, 0..1);
    }
    let bind_group = device.create_bind_group(
        "layered_depth_bind_group",
        &layout(&p.composite_depth_layout),
        &BindGroupEntries::sequential((
            &layered.history_write,
            &p.clamp_linear,
            &layered.depths.default_view,
            view_binding,
            globals,
        )),
    );
    let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("layered_depth"),
        color_attachments: &[Some(target.get_color_attachment())],
        depth_stencil_attachment: Some(depth.get_attachment(StoreOp::Store)),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    if let Some(viewport) = camera.viewport.as_ref() {
        pass.set_camera_viewport(viewport);
    }
    pass.set_render_pipeline(depth_write);
    pass.set_bind_group(0, &bind_group, &[view_offset.offset]);
    pass.draw(0..3, 0..1);
}
