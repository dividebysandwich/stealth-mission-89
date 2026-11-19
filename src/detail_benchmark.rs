//! The browser build's first-visit benchmark: once the menu's first scene is up, the frame times
//! behind the menu are measured for a few seconds; while the browser can't keep up, the graphics
//! detail steps down (`STEPS`, the costliest effects for the least look first) and is measured
//! again. Settings are only ever lowered, and the menu's preferences keep them; a notice says what
//! changed (OPTIONS > GRAPHICS raises them again). It runs once per browser (`DONE_FILE`), and
//! stops if the player changes the graphics meanwhile or leaves the menu (then it runs again on
//! the next visit).
//!
//! `--detail-benchmark` runs it on the desktop too (tests; with `--novsync` and
//! `SM89_DETAIL_BENCHMARK_FPS=<fps>` to make a fast machine step down).

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::settings::{AppState, Settings};
use crate::weather::CloudQuality;

/// `config/<this>`: the benchmark's version once it has run (raise `VERSION` to run it again
/// for everyone).
const DONE_FILE: &str = "detail-benchmark.json";
const VERSION: u32 = 1;

/// The frame rate (median frame time) below which the detail steps down; very slow (under
/// `VERY_SLOW_FPS`) steps two at once.
const TARGET_FPS: f32 = 40.0;
const VERY_SLOW_FPS: f32 = 20.0;
/// Left out after the scene is up or a change (loading stutter; shader compiles, which can
/// hold single frames for seconds): this long and this many frames.
const SETTLE: f32 = 2.0;
const SETTLE_FRAMES: usize = 10;
/// Then measured for this long and this many frames, or at most `MEASURE_MAX` seconds.
const MEASURE: f32 = 2.5;
const MEASURE_FRAMES: usize = 30;
const MEASURE_MAX: f32 = 8.0;

pub struct DetailBenchmarkPlugin;

impl Plugin for DetailBenchmarkPlugin {
    fn build(&self, app: &mut App) {
        let forced = std::env::args().any(|a| a == "--detail-benchmark");
        let settings = app.world().resource::<Settings>();
        let web_first_visit = cfg!(target_arch = "wasm32")
            && crate::prefs::load_json::<u32>(DONE_FILE).is_none_or(|v| v < VERSION);
        if !forced && !(web_first_visit && settings.initial_state == AppState::Menu && !settings.vr)
        {
            return;
        }
        let target = std::env::var("SM89_DETAIL_BENCHMARK_FPS")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(TARGET_FPS);
        app.insert_resource(Bench { target, ..default() })
            .add_systems(Update, measure.run_if(in_state(AppState::Menu)))
            .add_systems(OnExit(AppState::Menu), abandon)
            .add_systems(
                EguiPrimaryContextPass,
                notice.run_if(in_state(AppState::Menu)).run_if(resource_exists::<Notice>),
            );
    }
}

#[derive(Resource, Default)]
struct Bench {
    target: f32,
    /// Steps taken (`STEPS`).
    step: usize,
    /// Seconds and frames since the scene was up or the last change, then since measuring
    /// began.
    since: f32,
    settle_frames: usize,
    frames: Vec<f32>,
    /// The graphics as last set: anything else means the player changed them.
    set: Option<Graphics>,
}

/// The settings the benchmark changes.
#[derive(Clone, PartialEq, Debug)]
struct Graphics {
    modern_shading: bool,
    msaa: bool,
    shadows: bool,
    local_lights: bool,
    cloud_quality: CloudQuality,
    effects: crate::effects::Effects,
}

impl Graphics {
    fn of(s: &Settings) -> Self {
        Self {
            modern_shading: s.modern_shading,
            msaa: s.msaa,
            shadows: s.shadows,
            local_lights: s.local_lights,
            cloud_quality: s.cloud_quality,
            effects: s.effects,
        }
    }

    fn write(&self, s: &mut Settings) {
        s.modern_shading = self.modern_shading;
        s.msaa = self.msaa;
        s.shadows = self.shadows;
        s.local_lights = self.local_lights;
        s.cloud_quality = self.cloud_quality;
        s.effects = self.effects;
    }
}

/// The steps down, each on top of the ones before. Only ever lowers what is set.
const STEPS: usize = 3;

fn lower(g: &mut Graphics, step: usize) {
    let clouds = |g: &mut Graphics, q: CloudQuality| {
        let rank = |q: CloudQuality| match q {
            CloudQuality::Billboard => 0,
            CloudQuality::Low => 1,
            CloudQuality::Medium => 2,
            CloudQuality::High | CloudQuality::Auto => 3,
            CloudQuality::Ultra => 4,
        };
        if rank(q) < rank(g.cloud_quality) {
            g.cloud_quality = q;
        }
    };
    match step {
        // Full-screen costs that are hardly seen behind a menu or at a glance.
        0 => {
            clouds(g, CloudQuality::Medium);
            let e = &mut g.effects;
            g.msaa = false;
            g.shadows = false;
            g.local_lights = false;
            e.mist = false;
            e.cloud_shadows = false;
            e.ocean = false;
        }
        1 => {
            clouds(g, CloudQuality::Low);
            let e = &mut g.effects;
            e.glass = false;
            e.water = false;
            e.ground_detail = false;
            e.sunbeams = false;
            e.vapour = false;
        }
        _ => {
            clouds(g, CloudQuality::Billboard);
            g.modern_shading = false;
        }
    }
}

/// The browser shows the page (a hidden tab gets few or no frames).
fn page_visible() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        let hidden = web_sys::window()
            .and_then(|w| js_sys::Reflect::get(&w, &"document".into()).ok())
            .and_then(|d| js_sys::Reflect::get(&d, &"hidden".into()).ok())
            .and_then(|h| h.as_bool());
        hidden != Some(true)
    }
    #[cfg(not(target_arch = "wasm32"))]
    true
}

fn median(frames: &mut [f32]) -> f32 {
    frames.sort_by(f32::total_cmp);
    frames[frames.len() / 2]
}

/// How many steps down a median frame time asks for.
fn steps_for(median: f32, target_fps: f32) -> usize {
    let fps = 1.0 / median.max(1e-6);
    if fps >= target_fps {
        0
    } else if fps < VERY_SLOW_FPS.min(target_fps / 2.0) {
        2
    } else {
        1
    }
}

fn measure(
    mut commands: Commands,
    bench: Option<ResMut<Bench>>,
    mut settings: ResMut<Settings>,
    director: Option<Res<crate::menu_scene::director::Director>>,
    time: Res<Time<Real>>,
) {
    let Some(mut bench) = bench else { return };
    // Measured behind the menu's scene, once it is up.
    if !director.is_some_and(|d| d.spawned) {
        return;
    }
    let now = Graphics::of(&settings);
    if bench.set.as_ref().is_some_and(|g| *g != now) {
        info!("detail benchmark: graphics changed by the player, stopped");
        return finish(&mut commands, &bench, false);
    }
    bench.set = Some(now.clone());
    if !page_visible() {
        bench.restart();
        return;
    }
    let dt = time.delta_secs();
    bench.since += dt;
    if bench.settle_frames < SETTLE_FRAMES || bench.since < SETTLE {
        bench.settle_frames += 1;
        if bench.settle_frames >= SETTLE_FRAMES && bench.since >= SETTLE {
            bench.since = 0.0;
        }
        return;
    }
    bench.frames.push(dt);
    let enough = bench.since >= MEASURE && bench.frames.len() >= MEASURE_FRAMES;
    if !enough && bench.since < MEASURE_MAX {
        return;
    }
    let m = median(&mut bench.frames);
    let n = steps_for(m, bench.target);
    info!(
        "detail benchmark: step {}, {} frames, median {:.1} ms ({:.0} fps)",
        bench.step,
        bench.frames.len(),
        m * 1000.0,
        1.0 / m
    );
    if n == 0 || bench.step >= STEPS {
        return finish(&mut commands, &bench, true);
    }
    let mut g = now;
    for _ in 0..n.min(STEPS - bench.step) {
        lower(&mut g, bench.step);
        bench.step += 1;
    }
    info!("detail benchmark: lowered to step {}: {g:?}", bench.step);
    g.write(&mut settings);
    bench.set = Some(g);
    bench.restart();
}

impl Bench {
    /// Settles and measures again.
    fn restart(&mut self) {
        self.since = 0.0;
        self.settle_frames = 0;
        self.frames.clear();
    }
}

/// Done: remembered; a notice if the detail was lowered.
fn finish(commands: &mut Commands, bench: &Bench, completed: bool) {
    if completed {
        info!("detail benchmark: done at step {}", bench.step);
    }
    if let Err(e) = crate::prefs::save_json(DONE_FILE, &VERSION) {
        warn!("saving {DONE_FILE}: {e}");
    }
    if bench.step > 0 && completed {
        commands.insert_resource(Notice { shown_for: 0.0 });
    }
    commands.remove_resource::<Bench>();
}

/// Left the menu before it was done: it runs again on the next visit.
fn abandon(mut commands: Commands, bench: Option<Res<Bench>>) {
    if bench.is_some() {
        info!("detail benchmark: left the menu, not finished");
        commands.remove_resource::<Bench>();
    }
    commands.remove_resource::<Notice>();
}

#[derive(Resource)]
struct Notice {
    shown_for: f32,
}

/// Shown for this long unless closed (s).
const NOTICE_TIME: f32 = 20.0;

fn notice(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut shown: ResMut<Notice>,
    time: Res<Time<Real>>,
) -> Result {
    use crate::menu::{BRIGHT, big_button, menu_window};
    shown.shown_for += time.delta_secs();
    let ctx = contexts.ctx_mut()?;
    let mut close = shown.shown_for > NOTICE_TIME;
    menu_window(ctx, "detail-benchmark", 420.0)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -24.0))
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new(stealth_i18n::t!("detail-benchmark-lowered")).color(BRIGHT),
            );
            ui.add_space(8.0);
            close |= ui.add(big_button(stealth_i18n::t!("ok"))).clicked();
        });
    if close {
        commands.remove_resource::<Notice>();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_follow_the_frame_rate() {
        assert_eq!(steps_for(1.0 / 60.0, TARGET_FPS), 0);
        assert_eq!(steps_for(1.0 / 30.0, TARGET_FPS), 1);
        assert_eq!(steps_for(1.0 / 12.0, TARGET_FPS), 2);
    }

    #[test]
    fn steps_only_lower() {
        let s = Settings::from_args();
        let mut g = Graphics::of(&s);
        g.cloud_quality = CloudQuality::Billboard;
        g.effects.water = false;
        lower(&mut g, 0);
        assert_eq!(g.cloud_quality, CloudQuality::Billboard);
        lower(&mut g, 1);
        assert!(!g.effects.water && !g.msaa && !g.effects.mist);
        lower(&mut g, 2);
        assert!(!g.modern_shading);
        // Never turns anything on.
        let mut low = Graphics::of(&s);
        low.modern_shading = false;
        low.effects.glass = false;
        for step in 0..STEPS {
            lower(&mut low, step);
        }
        assert!(!low.effects.glass && !low.modern_shading && !low.shadows);
    }
}
