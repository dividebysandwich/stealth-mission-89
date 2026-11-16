//! The open sea's waves (`Effects::ocean`, with the water reflections and modern shading): an
//! FFT ocean of the wind's spectrum (`spectrum`), computed on a worker thread (`worker`) for
//! each frame. Its cascades' displacement, foam and slopes are textures the world shader
//! samples: round the viewpoint the waves rise in a mesh (`grid`), kept to the open water by a
//! mask of the sea (`mask`); beyond it the flat sea polygons take their slopes and foam.
//! Ships ride the same surface (`sea`).
//!
//! The sea is the one the weather's sea state has raised (`WeatherState::sea`: in a campaign
//! it builds and calms down over hours), running with the wind.

pub mod fft;
pub mod grid;
pub mod mask;
pub mod spectrum;
pub mod worker;

use std::sync::Arc;
use std::sync::mpsc::Receiver;

use bevy::asset::{RenderAssetUsages, uuid_handle};
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::math::{DVec2, DVec3};
use bevy::mesh::MeshTag;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
};

use self::mask::Mask;
use self::spectrum::{Frame, SeaParams};
use self::worker::{Done, MIPS, N, Worker};
use crate::palette_material::{FLAG_OCEAN, FLAG_WATER, PaletteMaterial, PaletteParams, mesh_tag};
use crate::settings::AppState;
use crate::weather::WeatherState;
use crate::world::{FloatingOrigin, WorldAnchor, WorldPos};

/// The cascades' displacement and foam (`ocean_disp` in palette.wgsl): RGBA16F, 3 layers.
pub const OCEAN_DISP: Handle<Image> = uuid_handle!("8f3d2a61-0c4b-4e7a-9b15-6d2e7f1a3c58");
/// Their derivatives (`ocean_deriv`).
pub const OCEAN_DERIV: Handle<Image> = uuid_handle!("2c7a9e14-5b3f-4d86-a0e2-91f4c3b7d602");
/// The open water round the viewpoint (`ocean_mask`, `mask::Mask`).
pub const OCEAN_MASK: Handle<Image> = uuid_handle!("d41e8b70-3a2c-4f95-8e6d-07b5a9c1f3e4");

/// The waves' mesh rises fully below this height of the viewpoint (m), fading out up to
/// `GRID_TOP` (and no mesh above it: from there the slopes on the flat sea are all that shows).
const GRID_FULL: f64 = 1200.0;
const GRID_TOP: f64 = 3000.0;

pub struct OceanPlugin;

impl Plugin for OceanPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Ocean>()
            .init_resource::<OceanWrites>()
            .add_systems(Startup, add_images)
            .add_systems(OnExit(AppState::Flight), reset)
            .add_systems(
                Update,
                drive
                    .after(crate::sea::tick)
                    .before(crate::sea::SeaSystems)
                    .run_if(in_state(AppState::Flight)),
            );
        if let Some(render) = app.get_sub_app_mut(bevy::render::RenderApp) {
            render
                .init_resource::<gpu::Pending>()
                .add_systems(bevy::render::ExtractSchedule, gpu::extract)
                .add_systems(
                    bevy::render::Render,
                    gpu::write.in_set(bevy::render::RenderSystems::PrepareResources),
                );
        }
    }
}

/// The ocean now.
#[derive(Resource, Default)]
pub struct Ocean {
    /// The waves are simulated (the effect is on and there is a theater).
    pub on: bool,
    /// The latest surface.
    pub frame: Option<Arc<Frame>>,
    /// The open water round the viewpoint.
    pub mask: Option<Arc<Mask>>,
    /// The waves' mesh is drawn, and how high its waves rise (0..1, by the viewpoint's height).
    pub grid_on: bool,
    pub amp: f32,
    /// The viewpoint (absolute Bevy x, z) the levels are round.
    eye: DVec2,
    params: SeaParams,
    /// The wind (m/s) the gain was worked out for, and the gain (`spectrum::calibrated_gain`).
    gain: (f32, f32),
    /// The latest surface's texture bytes (`--gl-vr` uploads them from here).
    pub textures: Option<Arc<OceanTextures>>,
    worker: Option<Worker>,
    mask_job: Option<std::sync::Mutex<Receiver<Mask>>>,
    grid: Option<(Entity, Handle<PaletteMaterial>)>,
}

/// A surface's texture bytes (`worker::Done`).
pub struct OceanTextures {
    pub disp: Vec<u8>,
    pub deriv: Vec<u8>,
}

/// Texture bytes for the render world.
#[derive(Resource, Default)]
pub struct OceanWrites {
    pub textures: Option<Arc<OceanTextures>>,
    pub mask: Option<Arc<Mask>>,
}

impl Ocean {
    /// `Globals::ocean`: on (x), the waves' rise (y); `Globals::ocean_mask`: the mask's
    /// corner (x, z, from the floating origin) and span (z); `Globals::ocean_levels`: each
    /// level's centre (from the floating origin), two a vec4.
    pub fn globals(&self, origin: DVec3) -> (Vec4, Vec4, [Vec4; 4]) {
        let on = self.on && self.frame.is_some();
        let ocean = Vec4::new(
            on as u32 as f32,
            if self.grid_on { self.amp } else { 0.0 },
            self.grid_on as u32 as f32,
            0.0,
        );
        let mask = match &self.mask {
            Some(m) if on => Vec4::new(
                (m.centre.x - mask::SPAN / 2.0 - origin.x) as f32,
                (-m.centre.y - mask::SPAN / 2.0 - origin.z) as f32,
                mask::SPAN as f32,
                1.0,
            ),
            _ => Vec4::new(0.0, 0.0, mask::SPAN as f32, 0.0),
        };
        let c = grid::level_centres(self.eye);
        let rel = |l: usize| ((c[l].x - origin.x) as f32, (c[l].y - origin.z) as f32);
        let levels = std::array::from_fn(|i| {
            let (a, b) = (rel(2 * i), rel(2 * i + 1));
            Vec4::new(a.0, a.1, b.0, b.1)
        });
        (ocean, mask, levels)
    }

    /// The water's height (m) at a world point (absolute Bevy x, z): the simulated waves
    /// (the cascades `weights` given), let down near the shore; 0 without them.
    pub fn height(&self, x: f64, z: f64, weights: [f32; 3]) -> f32 {
        let Some(f) = self.frame.as_ref().filter(|_| self.on) else { return 0.0 };
        let rise = self.mask.as_ref().map_or(1.0, |m| m.sample(x, z).1);
        if rise <= 0.0 {
            return 0.0;
        }
        f.height(x, z, weights) * rise
    }
}

/// Sea depth (m) and fetch (m) of a theater ([`stealth_mission::THEATER_NAMES`] order):
/// shallow gulfs and the Baltic keep the long waves down.
fn depth_and_fetch(theater: usize) -> (f32, f32) {
    match theater {
        1 | 6 => (35.0, 150_000.0), // the Persian Gulf
        10 => (40.0, 100_000.0),    // the Gulf of Finland
        11 => (55.0, 200_000.0),    // the Baltic
        9 => (60.0, 250_000.0),     // the Black Sea and the Sea of Azov
        _ => (200.0, 400_000.0),
    }
}

fn image(size: u32, layers: u32, format: TextureFormat, mips: u32, repeat: bool) -> Image {
    let texel = format.block_copy_size(None).unwrap_or(4) as usize;
    let mut img = Image::new_fill(
        Extent3d { width: size, height: size, depth_or_array_layers: layers },
        TextureDimension::D2,
        &vec![0u8; texel],
        format,
        RenderAssetUsages::RENDER_WORLD,
    );
    // Written by `gpu::write` (every mip level).
    img.data = None;
    img.texture_descriptor.mip_level_count = mips;
    if layers > 1 {
        img.texture_view_descriptor = Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::D2Array),
            ..default()
        });
    }
    let address = if repeat { ImageAddressMode::Repeat } else { ImageAddressMode::ClampToEdge };
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: address,
        address_mode_v: address,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    });
    img
}

fn add_images(mut images: ResMut<Assets<Image>>) {
    let layers = spectrum::CASCADE_LENGTHS.len() as u32;
    for h in [OCEAN_DISP, OCEAN_DERIV] {
        let _ = images
            .insert(h.id(), image(N as u32, layers, TextureFormat::Rgba16Float, MIPS as u32, true));
    }
    let _ = images
        .insert(OCEAN_MASK.id(), image(mask::SIZE as u32, 1, TextureFormat::Rgba8Unorm, 1, false));
}

fn reset(mut commands: Commands, mut ocean: ResMut<Ocean>) {
    if let Some((e, _)) = ocean.grid.take() {
        commands.entity(e).try_despawn();
    }
    *ocean = Ocean::default();
}

/// The sea's spectrum from the weather: the sea state's wind (rounded, so the spectrum is made
/// again only when it changes visibly) towards where the wind blows.
fn sea_params(weather: &WeatherState, theater: usize) -> SeaParams {
    let kt = stealth_mission::wind::sea_wind_kt(weather.sea);
    let wind_ms = ((kt * crate::wind::KNOT_MS) * 10.0).round() / 10.0;
    let (e, n) = weather.wind.velocity();
    // Bevy x = east, z = -north.
    let dir = if weather.wind.kt > 0.5 { (-n).atan2(e) } else { 0.6 };
    let dir = (dir.to_degrees().round()).to_radians();
    let (depth_m, fetch_m) = depth_and_fetch(theater);
    SeaParams { wind_ms: wind_ms.max(0.5), wind_dir: dir, fetch_m, depth_m, ..default() }
}

#[allow(clippy::too_many_arguments)]
fn drive(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<crate::settings::Settings>,
    weather: Res<WeatherState>,
    theater: Option<Res<crate::world::Theater>>,
    scenery: Option<Res<crate::scenery::Scenery>>,
    palette: Option<Res<crate::palette::Palette>>,
    clock: Res<crate::sea::SeaClock>,
    anchor: Query<&WorldPos, With<WorldAnchor>>,
    origin: Res<FloatingOrigin>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PaletteMaterial>>,
    mut ocean: ResMut<Ocean>,
    mut writes: ResMut<OceanWrites>,
) {
    let ocean = &mut *ocean;
    let on = crate::palette_material::ocean(&settings) && theater.is_some();
    if !on {
        if ocean.on {
            if let Some((e, _)) = ocean.grid.take() {
                commands.entity(e).try_despawn();
            }
            ocean.worker = None;
            ocean.frame = None;
        }
        ocean.on = false;
        return;
    }
    ocean.on = true;
    let theater_index =
        theater.as_ref().and_then(|t| crate::mission::theater_index(&t.id)).unwrap_or(3);
    let mut params = sea_params(&weather, theater_index);
    if ocean.gain.0 != params.wind_ms {
        ocean.gain = (params.wind_ms, spectrum::calibrated_gain(params.wind_ms));
    }
    params.gain = ocean.gain.1;
    ocean.params = params;

    // --- the surface
    let seed = weather.weather.seed ^ 0x0CEA_0000;
    let worker = ocean.worker.get_or_insert_with(|| Worker::new(params, seed));
    if let Some(Done { frame, disp, deriv }) = worker.poll() {
        ocean.frame = Some(frame);
        let t = Arc::new(OceanTextures { disp, deriv });
        writes.textures = Some(t.clone());
        ocean.textures = Some(t);
    }
    // Ahead by a frame: shown when it is done.
    worker.request(clock.0 + time.delta_secs_f64(), params);

    // --- the viewpoint, the mask round it
    let eye = anchor.single().map_or(origin.origin, |p| p.0);
    ocean.eye = DVec2::new(eye.x, eye.z);
    let height = eye.y;
    ocean.grid_on = height < GRID_TOP;
    ocean.amp = (1.0 - ((height - GRID_FULL) / (GRID_TOP - GRID_FULL)).clamp(0.0, 1.0)) as f32;
    if let Some(job) = ocean.mask_job.as_mut()
        && let Ok(m) = job.get_mut().map_err(|_| ()).and_then(|rx| rx.try_recv().map_err(|_| ()))
    {
        let m = Arc::new(m);
        writes.mask = Some(m.clone());
        ocean.mask = Some(m);
        ocean.mask_job = None;
    }
    // East, north (metres).
    let here = DVec2::new(eye.x, -eye.z);
    let stale =
        ocean.mask.as_ref().is_none_or(|m| (m.centre - here).abs().max_element() > mask::RECENTRE);
    if stale
        && ocean.mask_job.is_none()
        && let Some(scenery) = &scenery
    {
        let centre = (here / 256.0).round() * 256.0;
        let scenery = (**scenery).clone();
        let (tx, rx) = std::sync::mpsc::channel();
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || {
            let _ = tx.send(Mask::build(centre, |p| scenery.is_water(p)));
        });
        #[cfg(target_arch = "wasm32")]
        let _ = tx.send(Mask::build(centre, |p| scenery.is_water(p)));
        ocean.mask_job = Some(std::sync::Mutex::new(rx));
    }

    // --- the mesh, in the sea's colour
    let Some(palette) = palette else { return };
    // The sea polygons' colour as the palette draws it now (its remap: the detail level).
    let word = palette.word(SEA_COLOUR) as u32;
    match &ocean.grid {
        Some((e, material)) => {
            if let Some(m) = materials.get(material)
                && m.params.word != word
                && let Some(mut m) = materials.get_mut(material)
            {
                m.params.word = word;
            }
            let shown = if ocean.grid_on { Visibility::Inherited } else { Visibility::Hidden };
            commands.entity(*e).insert(shown);
        }
        None => {
            let material = materials.add(PaletteMaterial {
                params: PaletteParams { word, flags: FLAG_WATER | FLAG_OCEAN, ..default() },
                globals: palette.globals.clone(),
            });
            let e = commands
                .spawn((
                    Name::new("ocean waves"),
                    Mesh3d(meshes.add(grid::mesh())),
                    MeshMaterial3d(material.clone()),
                    // Hazed by each vertex's distance (it spans kilometres).
                    MeshTag(mesh_tag(1, 0, false) | crate::palette_material::TAG_VERTEX_HAZE),
                    Transform::IDENTITY,
                    Visibility::default(),
                    bevy::camera::visibility::NoFrustumCulling,
                    bevy::light::NotShadowCaster,
                ))
                .id();
            ocean.grid = Some((e, material));
        }
    }
}

/// The sea's model colour (`scenery::ground::Surface`: OR with the sky's bands at high detail).
const SEA_COLOUR: u8 = 49;

/// Render world: the ocean's textures written in place (their bind groups stay valid).
mod gpu {
    use bevy::prelude::*;
    use bevy::render::MainWorld;
    use bevy::render::render_asset::RenderAssets;
    use bevy::render::render_resource::*;
    use bevy::render::renderer::RenderQueue;
    use bevy::render::texture::GpuImage;
    use std::sync::Arc;

    use super::{OCEAN_DERIV, OCEAN_DISP, OCEAN_MASK, OceanWrites};

    #[derive(Resource, Default)]
    pub struct Pending {
        textures: Option<Arc<super::OceanTextures>>,
        mask: Option<Arc<super::Mask>>,
    }

    pub fn extract(mut main: ResMut<MainWorld>, mut pending: ResMut<Pending>) {
        let Some(mut w) = main.get_resource_mut::<OceanWrites>() else { return };
        if let Some(t) = w.textures.take() {
            pending.textures = Some(t);
        }
        if let Some(m) = w.mask.take() {
            pending.mask = Some(m);
        }
    }

    /// A texture array's mip levels, each level's layers in turn (`worker::pack`).
    fn write_levels(queue: &RenderQueue, gpu: &GpuImage, bytes: &[u8]) {
        let size = gpu.texture.width();
        let layers = gpu.texture.depth_or_array_layers();
        let mut at = 0;
        for level in 0..gpu.texture.mip_level_count() {
            let s = (size >> level).max(1);
            let len = (s * s * 8 * layers) as usize;
            let Some(data) = bytes.get(at..at + len) else { return };
            queue.write_texture(
                TexelCopyTextureInfo {
                    texture: &gpu.texture,
                    mip_level: level,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                data,
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(s * 8),
                    rows_per_image: Some(s),
                },
                Extent3d { width: s, height: s, depth_or_array_layers: layers },
            );
            at += len;
        }
    }

    pub fn write(
        mut pending: ResMut<Pending>,
        images: Res<RenderAssets<GpuImage>>,
        queue: Res<RenderQueue>,
    ) {
        if let (Some(disp), Some(deriv)) = (images.get(&OCEAN_DISP), images.get(&OCEAN_DERIV))
            && let Some(t) = pending.textures.take()
        {
            write_levels(&queue, disp, &t.disp);
            write_levels(&queue, deriv, &t.deriv);
        }
        if let Some(gpu) = images.get(&OCEAN_MASK)
            && let Some(m) = pending.mask.take()
        {
            let s = super::mask::SIZE as u32;
            queue.write_texture(
                gpu.texture.as_image_copy(),
                &m.rgba,
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(s * 4),
                    rows_per_image: Some(s),
                },
                Extent3d { width: s, height: s, depth_or_array_layers: 1 },
            );
        }
    }
}
