//! The browser's loading screen after the wasm has started (web/index.html): the game loads its
//! models (a bar as each file comes), then lays out the menu scene's scenery (on the page's
//! main thread: a busy bar, shown before the layout blocks the page), and says when the first
//! scene is up and drawn with all its pipelines, which takes the screen away.
//!
//! The browser builds each pipeline on the page's thread (Bevy composes and validates the
//! shader in the wasm, the browser compiles it at its first draw), a few hundred milliseconds to
//! seconds each: the scene's first frames would stutter through them. The screen stays until
//! no new pipeline has come for `QUIET_FRAMES` frames (at most `MAX_PIPELINE_WAIT`).

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use bevy::prelude::*;
use bevy::render::render_resource::{CachedPipelineState, PipelineCache};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use crate::settings::AppState;

pub struct WebLoadingPlugin;

impl Plugin for WebLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(crate::scenery::LayoutHold(true))
            .insert_resource(crate::detail_benchmark::Covered)
            .insert_resource(crate::detail_benchmark::PipelinesQuiet(false))
            .insert_resource(crate::sound::MusicHold(true))
            .init_resource::<Loading>()
            .add_systems(Update, report.run_if(in_state(AppState::Menu)))
            // Away from the menu before it was up: the flight lays out its own scenery.
            .add_systems(OnExit(AppState::Menu), finish);
        let pipelines = Pipelines::default();
        app.insert_resource(pipelines.clone());
        if let Some(render) = app.get_sub_app_mut(bevy::render::RenderApp) {
            render.insert_resource(pipelines).add_systems(
                bevy::render::Render,
                count_pipelines.in_set(bevy::render::RenderSystems::Cleanup),
            );
        }
    }
}

/// The render world's pipelines, for the main world: how many there are and how many of them
/// are not built yet.
#[derive(Resource, Clone, Default)]
struct Pipelines {
    total: Arc<AtomicU32>,
    pending: Arc<AtomicU32>,
}

fn count_pipelines(cache: Res<PipelineCache>, counts: Res<Pipelines>) {
    let (mut total, mut pending) = (0, 0);
    for p in cache.pipelines() {
        total += 1;
        pending += matches!(p.state, CachedPipelineState::Queued | CachedPipelineState::Creating(_))
            as u32;
    }
    pending += cache.waiting_pipelines().count() as u32;
    counts.total.store(total, Ordering::Relaxed);
    counts.pending.store(pending, Ordering::Relaxed);
}

/// The scene is drawn with all its pipelines once none has come for this many frames.
const QUIET_FRAMES: u32 = 8;
/// The screen goes after this long anyway (s, from the scene's spawn; the detail benchmark gives
/// up after `detail_benchmark::MAX_TIME`).
const MAX_PIPELINE_WAIT: f32 = 40.0;

/// The model files the menu's scenes draw from (besides the theater's own).
const FILES: [&str; 4] = ["aircraft", "drones", "aaa", "special_targets"];
/// Frames the scenery's words are up before the layout begins (painted before the page
/// blocks).
const SHOWN_FRAMES: u32 = 3;

#[derive(Resource, Default)]
struct Loading {
    /// The last report (the phase, the files done of all).
    last: Option<(&'static str, usize, usize)>,
    scenery_frames: u32,
    /// After the scene is spawned: seconds, frames without a new pipeline, the last count.
    waited: f32,
    quiet: u32,
    pipelines: u32,
    done: bool,
    /// Seconds since the game was up (the music waits `MUSIC_DELAY` more).
    up_for: f32,
}

/// The music starts this long after the game is up (s): the first seconds still stutter.
const MUSIC_DELAY: f32 = 3.0;

fn finish(
    mut commands: Commands,
    mut loading: ResMut<Loading>,
    mut hold: ResMut<crate::scenery::LayoutHold>,
    mut music: ResMut<crate::sound::MusicHold>,
) {
    (hold.0, music.0) = (false, false);
    if !loading.done {
        loading.done = true;
        up(&mut commands);
    }
}

/// The screen goes: the game is seen.
fn up(commands: &mut Commands) {
    commands.remove_resource::<crate::detail_benchmark::Covered>();
    commands.remove_resource::<crate::detail_benchmark::PipelinesQuiet>();
    call("SM89_READY", &[]);
}

/// Calls the page's function `name` with `args`.
fn call(name: &str, args: &[JsValue]) {
    let Some(window) = web_sys::window() else { return };
    let Ok(f) = js_sys::Reflect::get(&window, &JsValue::from_str(name)) else { return };
    let Some(f) = f.dyn_ref::<js_sys::Function>() else { return };
    let array: js_sys::Array = args.iter().collect();
    let _ = f.apply(&JsValue::NULL, &array);
}

#[allow(clippy::too_many_arguments)]
fn report(
    mut loading: ResMut<Loading>,
    mut hold: ResMut<crate::scenery::LayoutHold>,
    mut library: ResMut<crate::models::ModelLibrary>,
    server: Res<AssetServer>,
    cockpits: Option<Res<crate::cockpit3d::CockpitModelsPub>>,
    theater: Option<Res<crate::world::Theater>>,
    scenery: Option<Res<crate::scenery::Scenery>>,
    director: Option<Res<crate::menu_scene::director::Director>>,
    mut music: ResMut<crate::sound::MusicHold>,
    time: Res<Time<Real>>,
    pipelines: Res<Pipelines>,
    mut commands: Commands,
    quiet: Option<ResMut<crate::detail_benchmark::PipelinesQuiet>>,
    bench: Option<Res<crate::detail_benchmark::Bench>>,
) {
    if loading.done {
        loading.up_for += time.delta_secs();
        if music.0 && loading.up_for >= MUSIC_DELAY {
            music.0 = false;
        }
        return;
    }
    // The models: the scenes' files, the theater's, the cockpits'.
    let mut handles: Vec<UntypedHandle> =
        FILES.iter().map(|f| library.load(&server, f).untyped()).collect();
    if let Some(t) = &theater {
        handles.push(library.load(&server, &t.id).untyped());
    }
    if let Some(c) = &cockpits {
        handles.extend(c.handles());
    }
    let total = handles.len() + 1;
    let done = handles.iter().filter(|h| server.is_loaded_with_dependencies(h.id())).count()
        + theater.is_some() as usize;
    let report = if done < total {
        ("models", done, total)
    } else if scenery.is_none() {
        // Up for a few frames, then the layout may begin.
        loading.scenery_frames += 1;
        if loading.scenery_frames > SHOWN_FRAMES {
            hold.0 = false;
        }
        ("scenery", 0, 0)
    } else {
        hold.0 = false;
        if !director.is_some_and(|d| d.spawned) {
            return;
        }
        // Drawn: wait for its pipelines.
        let total = pipelines.total.load(Ordering::Relaxed);
        let pending = pipelines.pending.load(Ordering::Relaxed);
        loading.waited += time.delta_secs();
        if total != loading.pipelines || pending > 0 {
            loading.pipelines = total;
            loading.quiet = 0;
        } else {
            loading.quiet += 1;
        }
        let is_quiet = loading.quiet >= QUIET_FRAMES;
        if let Some(mut q) = quiet {
            q.0 = is_quiet;
        }
        // And the detail benchmark done (it may build them again).
        if (is_quiet && bench.is_none()) || loading.waited >= MAX_PIPELINE_WAIT {
            info!(
                "web loading: up after {:.1} s of pipelines and benchmark ({total}, {pending} pending)",
                loading.waited
            );
            loading.done = true;
            up(&mut commands);
        }
        return;
    };
    if loading.last != Some(report) {
        loading.last = Some(report);
        let (phase, done, total) = report;
        call("SM89_LOADING", &[phase.into(), (done as u32).into(), (total as u32).into()]);
    }
}
