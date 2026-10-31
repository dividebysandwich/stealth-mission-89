//! `--gl-vr`: the headset drawn with our own OpenGL renderer instead of Bevy's (a test of
//! foveated rendering on the Steam Frame, see docs at `foveation.rs`). Bevy runs without its
//! renderer and window. A render thread owns the OpenGL context and the headset's frames (as
//! rust-dos's headset thread): it waits for each frame, tells the main thread to update the
//! game, and meanwhile draws the last update's draw list (`extract.rs`) with the eyes' poses
//! of this frame, so updating and drawing overlap as with Bevy's pipelined rendering.
//! Only the palette world is drawn so far (no cockpit, clouds, dots or effects).

mod controllers;
mod egl;
mod extract;
mod foveation;
mod headset;
mod render;

use bevy::app::AppExit;
use bevy::asset::{AssetEvent, AssetId};
use bevy::prelude::*;
use extract::{DrawList, Extractor, Eyes};
use render::AssetMsg;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Asks for foveation before anything starts (the environment is read when Zink makes its
/// Vulkan instance): the line to log.
pub fn before_start() -> String {
    foveation::enable()
}

/// Meshes and images changed or gone since the renderer last looked.
#[derive(Resource, Default)]
struct DirtyMeshes(Vec<AssetId<Mesh>>, Vec<(AssetId<Image>, bool)>);

pub struct GlVrPlugin(pub String);

impl Plugin for GlVrPlugin {
    fn build(&self, app: &mut App) {
        info!("glvr: {}", self.0);
        app.insert_resource(crate::recon::ReadbackByImage)
            .init_resource::<DirtyMeshes>()
            .init_resource::<controllers::XrInput>()
            .init_resource::<controllers::Recenter>()
            .add_systems(Last, collect_dirty)
            .add_systems(
                PreUpdate,
                controllers::read_hands
                    .in_set(crate::cockpit3d::HandsUpdate)
                    .after(crate::input::InputSystemsSet),
            )
            .add_systems(Update, controllers::recenter)
            .add_systems(
                OnEnter(crate::settings::AppState::Flight),
                |mut r: ResMut<controllers::Recenter>| r.0 = Some(0),
            )
            .add_systems(
                OnExit(crate::settings::AppState::Flight),
                crate::xr::detach_root.before(crate::combat::FlightTeardown),
            )
            .set_runner(runner);
    }

    /// The tracking root, as bevy_mod_openxr's (`crate::camera::XrRoot`): the rig and the menu
    /// scene put it at the eye point, the eyes are drawn from it. Its camera draws nothing (Bevy
    /// renders nothing here); it is the main view the models' detail levels are picked for.
    fn finish(&self, app: &mut App) {
        let world = app.world_mut();
        let root = world
            .spawn((
                Camera3d::default(),
                // The clouds' detail is picked for this camera, about the headset's field of
                // view (bevy_fast_clouds takes 1.7 for XR eyes).
                Projection::Perspective(PerspectiveProjection { fov: 1.7, ..default() }),
                bevy_fast_clouds::CloudCamera,
                Transform::IDENTITY,
                Visibility::default(),
                bevy::camera::visibility::RenderLayers::from_layers(&[
                    0,
                    crate::camera::MAIN_BACKGROUND_LAYER,
                    crate::camera::MAIN_LOD_LAYER,
                ]),
            ))
            .id();
        world.spawn((
            crate::camera::XrHead,
            Transform::IDENTITY,
            Visibility::default(),
            ChildOf(root),
        ));
        world.insert_resource(crate::camera::XrRoot(Some(root)));
    }
}

fn collect_dirty(
    mut events: MessageReader<AssetEvent<Mesh>>,
    mut images: MessageReader<AssetEvent<Image>>,
    mut dirty: ResMut<DirtyMeshes>,
) {
    for e in images.read() {
        match e {
            AssetEvent::Modified { id } => dirty.1.push((*id, false)),
            AssetEvent::Removed { id } | AssetEvent::Unused { id } => dirty.1.push((*id, true)),
            _ => {}
        }
    }
    for e in events.read() {
        match e {
            AssetEvent::Modified { id }
            | AssetEvent::Removed { id }
            | AssetEvent::Unused { id } => dirty.0.push(*id),
            _ => {}
        }
    }
}

/// `SM89_GL_MSAA`: samples a pixel (0, 2 or 4; default 2).
fn samples() -> i32 {
    std::env::var("SM89_GL_MSAA").ok().and_then(|v| v.parse().ok()).unwrap_or(2)
}

fn runner(mut app: App) -> AppExit {
    match run(&mut app) {
        Ok(exit) => exit,
        Err(e) => {
            error!("glvr: {e}");
            AppExit::error()
        }
    }
}

fn run(app: &mut App) -> Result<AppExit, String> {
    // As Bevy's own runners do: plugins finish (the glTF loader is registered then).
    if app.plugins_state() != bevy::app::PluginsState::Cleaned {
        while app.plugins_state() == bevy::app::PluginsState::Adding {
            bevy::tasks::tick_global_task_pools_on_main_thread();
        }
        app.finish();
        app.cleanup();
    }
    let (tick_tx, tick_rx) = mpsc::sync_channel::<Eyes>(1);
    let (read_tx, read_rx) = mpsc::channel::<(Entity, Vec<u8>)>();
    let (mesh_tx, mesh_rx) = mpsc::channel::<AssetMsg>();
    let slot = Arc::new(Mutex::new(None::<DrawList>));
    let quit = Arc::new(AtomicBool::new(false));
    let thread = {
        let (slot, quit) = (slot.clone(), quit.clone());
        std::thread::Builder::new()
            .name("glvr-render".into())
            .spawn(move || render_thread(tick_tx, mesh_rx, read_tx, slot, quit))
            .map_err(|e| e.to_string())?
    };
    let mut extractor = Extractor::new(mesh_tx);
    let mut eyes = None;
    let mut last_log = Instant::now();
    let (mut updates, mut update_time, mut extract_time) = (0u32, Duration::ZERO, Duration::ZERO);
    let stop = |exit: AppExit| {
        quit.store(true, Ordering::Relaxed);
        thread.join().map_err(|_| "the render thread panicked".to_string())??;
        Ok(exit)
    };
    loop {
        // Paced by the headset's frames; idle (no session running) at 50 Hz.
        match tick_rx.recv_timeout(Duration::from_millis(20)) {
            Ok(e) => {
                let mut input = app.world_mut().resource_mut::<controllers::XrInput>();
                input.controllers = e.controllers;
                input.eyes = e
                    .views
                    .iter()
                    .map(|v| {
                        let (p, q) = render::pose(v);
                        Transform::from_translation(p).with_rotation(q)
                    })
                    .collect();
                let heads = input.eyes.clone();
                set_head(app.world_mut(), &heads);
                app.world_mut().resource_mut::<crate::camera::HeadPoses>().0 = heads;
                eyes = Some(e);
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return stop(AppExit::Success),
        }
        // Images read back (recon.rs's photos) arrive as Bevy's readbacks do.
        while let Ok((entity, data)) = read_rx.try_recv() {
            app.world_mut().trigger(bevy::render::gpu_readback::ReadbackComplete { entity, data });
        }
        let t0 = Instant::now();
        app.update();
        if let Some(exit) = app.should_exit() {
            return stop(exit);
        }
        let t1 = Instant::now();
        let dirty = std::mem::take(&mut *app.world_mut().resource_mut::<DirtyMeshes>());
        extractor.forget(dirty.0.into_iter());
        extractor.forget_images(dirty.1.into_iter());
        let list = extractor.extract(app.world_mut(), eyes.as_ref());
        *slot.lock().map_err(|_| "the draw list's lock")? = Some(list);
        updates += 1;
        update_time += t1 - t0;
        extract_time += t1.elapsed();
        if last_log.elapsed() >= Duration::from_secs(5) {
            info!(
                "glvr: {:.1} updates/s, update {:.2} ms, extract {:.2} ms",
                updates as f32 / last_log.elapsed().as_secs_f32(),
                update_time.as_secs_f32() * 1000.0 / updates.max(1) as f32,
                extract_time.as_secs_f32() * 1000.0 / updates.max(1) as f32,
            );
            last_log = Instant::now();
            (updates, update_time, extract_time) = (0, Duration::ZERO, Duration::ZERO);
        }
    }
}

/// Puts the head (`XrHead`) between the eyes `eyes`, facing as the first.
fn set_head(world: &mut World, eyes: &[Transform]) {
    let Some(first) = eyes.first() else { return };
    let middle = eyes.iter().map(|e| e.translation).sum::<Vec3>() / eyes.len() as f32;
    let mut heads = world.query_filtered::<&mut Transform, With<crate::camera::XrHead>>();
    for mut head in heads.iter_mut(world) {
        *head = Transform::from_translation(middle).with_rotation(first.rotation);
    }
}

/// Ends `frame` showing the image drawn last (`shown`: its eyes' poses and fence; the oldest
/// image acquired), once the GPU is done with it; or nothing.
fn show(
    headset: &mut headset::Headset,
    renderer: &render::Renderer,
    frame: &headset::Frame,
    shown: Option<(usize, Vec<openxr::View>, Option<glow::Fence>)>,
) -> Result<(), String> {
    match shown {
        Some((chain, views, fence)) => {
            if let Some(fence) = fence {
                renderer.wait(fence);
            }
            headset.present(frame, chain, &views)
        }
        None => headset.end_empty(frame),
    }
}

/// The render thread: the OpenGL context, the session and its frames.
fn render_thread(
    tick: SyncSender<Eyes>,
    meshes: Receiver<AssetMsg>,
    readouts: mpsc::Sender<(Entity, Vec<u8>)>,
    slot: Arc<Mutex<Option<DrawList>>>,
    quit: Arc<AtomicBool>,
) -> Result<(), String> {
    let context = egl::Context::new()?;
    let mut renderer = render::Renderer::new(
        context.gl(),
        |name| {
            context
                .egl
                .get_proc_address(name)
                .map_or(std::ptr::null(), |f| f as *const std::ffi::c_void)
        },
        samples(),
    )?;
    // Each frame's image is drawn with its own eyes and shown at once (waited for), into one
    // swapchain. SM89_GL_DELAYED=1: shown a frame later instead, the next one drawn meanwhile
    // into a second swapchain (more time for the GPU, but SteamVR on the Steam Frame then shows
    // older images between new ones: a judder whenever the head turns).
    let direct = !std::env::var("SM89_GL_DELAYED").is_ok_and(|v| v == "1");
    let mut headset = headset::Headset::new(&context, if direct { 1 } else { 2 })?;
    info!("glvr: eyes {}x{}, sRGB {}", headset.size.0, headset.size.1, headset.srgb);
    // SM89_XR_SHOT=<secs>: both eyes' images saved that long into the run.
    let started = Instant::now();
    let mut shot = std::env::var("SM89_XR_SHOT").ok().and_then(|v| v.parse::<f32>().ok());
    if !direct {
        info!("glvr: images shown a frame after they are drawn");
    }
    let mut list: Option<DrawList> = None;
    // The image drawn, its eyes' poses and the fence it is done at, to show.
    let mut drawn: Option<(usize, Vec<openxr::View>, Option<glow::Fence>)> = None;
    let mut last_log = Instant::now();
    let (mut frames, mut draw_time, mut draws, mut culled) = (0u32, Duration::ZERO, 0u64, 0u64);
    let (mut gpu, mut gpu_frames, mut distinct) = (0.0f32, 0u32, 0u64);
    while !quit.load(Ordering::Relaxed) {
        if !headset.poll()? {
            return Ok(());
        }
        if !headset.running() {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }
        let Some(frame) = headset.begin(!direct)? else { continue };
        // The main thread updates the game for the next frame meanwhile.
        let _ = tick.try_send(Eyes { views: frame.views.clone(), controllers: frame.controllers });
        let t0 = Instant::now();
        if let Some(new) = slot.lock().map_err(|_| "the draw list's lock")?.take() {
            for (i, &amount) in new.haptics.iter().enumerate() {
                if amount > 0.0 {
                    headset.pulse(i, amount);
                }
            }
            list = Some(new);
        }
        // (The meshes of the list taken were sent before it.)
        while let Ok(msg) = meshes.try_recv() {
            renderer.apply(msg);
        }
        // Delayed: the image drawn last frame is shown in this one, with the poses it was drawn
        // with, before this frame's is drawn (waiting for it right after drawing it held up the next
        // frame's drawing; released after the next image's drawing was queued, it was only
        // ready once that was done too).
        if !direct {
            show(&mut headset, &renderer, &frame, drawn.take())?;
        }
        let Some(list) = &list else {
            if direct {
                headset.end_empty(&frame)?;
            }
            continue;
        };
        let (chain, image) = headset.acquire()?;
        renderer.draw(list, &frame.views, image, headset.size, headset.srgb)?;
        if shot.is_some_and(|at| started.elapsed().as_secs_f32() >= at) {
            shot = None;
            if let Err(e) = renderer.save(image, headset.size) {
                warn!("glvr: {e}");
            }
        }
        let fence = renderer.fence();
        drawn = Some((chain, frame.views.clone(), fence));
        if direct {
            show(&mut headset, &renderer, &frame, drawn.take())?;
        }
        for r in renderer.readouts.drain(..) {
            let _ = readouts.send(r);
        }
        frames += 1;
        draw_time += t0.elapsed();
        draws += renderer.stats.draws as u64;
        culled += renderer.stats.culled as u64;
        distinct += renderer.stats.meshes as u64;
        if let Some(ms) = renderer.stats.gpu_ms {
            gpu += ms;
            gpu_frames += 1;
        }
        if last_log.elapsed() >= Duration::from_secs(5) {
            let n = frames.max(1);
            // (GPU times only with SM89_GL_TIMER=1.)
            let gpu_text = if gpu_frames > 0 {
                format!(", GPU {:.2} ms", gpu / gpu_frames as f32)
            } else {
                String::new()
            };
            let (lo, hi, ahead) = headset.pacing();
            info!(
                "glvr: {:.1} fps, draw {:.2} ms{gpu_text}, {} draws ({} culled, {} distinct world meshes) a frame, runtime period {lo:.1}-{hi:.1} ms, eyes {ahead:.1} ms ahead",
                frames as f32 / last_log.elapsed().as_secs_f32(),
                draw_time.as_secs_f32() * 1000.0 / n as f32,
                draws / n as u64,
                culled / n as u64,
                distinct / n as u64,
            );
            last_log = Instant::now();
            (frames, draw_time, draws, culled) = (0, Duration::ZERO, 0, 0);
            (gpu, gpu_frames, distinct) = (0.0, 0, 0);
        }
    }
    Ok(())
}
