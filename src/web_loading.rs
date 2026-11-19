//! The browser's loading screen after the wasm has started (web/index.html): the game loads its
//! models (a bar as each file comes), then lays out the menu scene's scenery (on the page's
//! main thread: a busy bar, shown before the layout blocks the page), and says when the first
//! scene is up, which takes the screen away.

use bevy::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use crate::settings::AppState;

pub struct WebLoadingPlugin;

impl Plugin for WebLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(crate::scenery::LayoutHold(true))
            .insert_resource(crate::sound::MusicHold(true))
            .init_resource::<Loading>()
            .add_systems(Update, report.run_if(in_state(AppState::Menu)))
            // Away from the menu before it was up: the flight lays out its own scenery.
            .add_systems(OnExit(AppState::Menu), finish);
    }
}

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
    done: bool,
    /// Seconds since the game was up (the music waits `MUSIC_DELAY` more).
    up_for: f32,
}

/// The music starts this long after the game is up (s): the first seconds still stutter.
const MUSIC_DELAY: f32 = 5.0;

fn finish(
    mut loading: ResMut<Loading>,
    mut hold: ResMut<crate::scenery::LayoutHold>,
    mut music: ResMut<crate::sound::MusicHold>,
) {
    (hold.0, music.0) = (false, false);
    if !loading.done {
        loading.done = true;
        call("SM89_READY", &[]);
    }
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
        if director.is_some_and(|d| d.spawned) {
            loading.done = true;
            call("SM89_READY", &[]);
        }
        return;
    };
    if loading.last != Some(report) {
        loading.last = Some(report);
        let (phase, done, total) = report;
        call("SM89_LOADING", &[phase.into(), (done as u32).into(), (total as u32).into()]);
    }
}
