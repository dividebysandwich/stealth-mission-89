//! The graphics benchmark: it starts at the LOW preset and climbs (`LEVELS`: LOW, MEDIUM,
//! HIGH) while the menu's scene keeps the target frame rate, measured for a few seconds behind
//! the menu at each level; at the first level that is too slow it goes back to the last one that
//! kept up (LOW if none did). The menu's preferences keep the result, and a notice says if it
//! is below HIGH (OPTIONS > GRAPHICS raises it again). Starting low keeps a slow device (a
//! phone) from building and drawing the costliest scene first, which took minutes there.
//!
//! In the browser it runs on the first visit (`DONE_FILE`), behind the loading screen
//! (web_loading.rs), which waits for it: each level builds the scene's pipelines again (new
//! shader variants), seconds of stalls that are better hidden. It settles only once no pipeline
//! is being built (`PipelinesQuiet`). It stops if the player changes the graphics meanwhile, or
//! leaves the menu (then it runs again on the next visit).
//!
//! On the desktop it runs on a new player's first start into the menu (no preferences saved
//! yet), behind a screen of its own (`loading.rs`), and again from OPTIONS > GRAPHICS' DETECT.
//!
//! `--detail-benchmark` runs it on the desktop too (tests; with `--novsync` and
//! `SM89_DETAIL_BENCHMARK_FPS=<fps>` to make a fast machine stop lower).

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::settings::{AppState, Settings};
use crate::weather::CloudQuality;

/// `config/<this>`: the benchmark's version once it has run (raise `VERSION` to run it again
/// for everyone).
const DONE_FILE: &str = "detail-benchmark.json";
const VERSION: u32 = 1;
/// Phones and tablets run it again from this version (before it, one that timed out kept HIGH,
/// minutes a frame there).
const MOBILE_VERSION: u32 = 2;

/// The benchmark has not run in this browser (or on this computer) yet, at this version.
fn not_run() -> bool {
    let version = if mobile() { MOBILE_VERSION } else { VERSION };
    crate::prefs::load_json::<u32>(DONE_FILE).is_none_or(|v| v < version)
}

/// A phone or tablet's browser (its user agent says so, or an iPad's "desktop" one with a touch
/// screen): the HIGH preset's first frames take minutes there.
fn mobile() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(navigator) = web_sys::window().map(|w| w.navigator()) else { return false };
        let ua_data_mobile = js_sys::Reflect::get(&navigator, &"userAgentData".into())
            .ok()
            .and_then(|d| js_sys::Reflect::get(&d, &"mobile".into()).ok())
            .and_then(|m| m.as_bool());
        let agent = navigator.user_agent().unwrap_or_default();
        mobile_agent(ua_data_mobile, &agent, navigator.max_touch_points())
    }
    #[cfg(not(target_arch = "wasm32"))]
    false
}

/// `mobile()` from the browser's answers: `navigator.userAgentData.mobile` (Chromium), the user
/// agent string, the touch points.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn mobile_agent(ua_data_mobile: Option<bool>, agent: &str, touch_points: i32) -> bool {
    const PHONES: [&str; 6] = ["Mobi", "Android", "iPhone", "iPad", "iPod", "Silk"];
    ua_data_mobile == Some(true)
        || PHONES.iter().any(|p| agent.contains(p))
        // iPadOS' Safari says it is a Mac.
        || (agent.contains("Macintosh") && touch_points > 1)
}

/// The benchmark runs from this start into the menu (`first_settings` started it at LOW).
static STARTS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Called on a start into the menu, before anything is built: if the benchmark runs (a
/// browser's first visit, a new player on the desktop, `--detail-benchmark`), the settings start
/// at LOW, its first level.
pub fn first_settings(s: &mut Settings) {
    let forced = std::env::args().any(|a| a == "--detail-benchmark");
    let not_run = not_run();
    let web_first_visit = cfg!(target_arch = "wasm32") && not_run;
    // A new player on the desktop: no preferences yet (a player's own settings are kept).
    let desktop_first_run = crate::fullscreen::DISPLAY_OPTIONS
        && not_run
        && crate::prefs::load_json::<serde_json::Value>("prefs.json").is_none()
        && !crate::loading::test_run();
    if forced || web_first_visit || desktop_first_run {
        LEVELS[0].apply(s);
        STARTS.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

/// The presets climbed, lowest first.
const LEVELS: [Preset; 3] = [Preset::Low, Preset::Medium, Preset::High];

/// The frame rate (median frame time) a level must keep to climb to the next.
const TARGET_FPS: f32 = 40.0;
/// Left out after the scene is up or a change (loading stutter; shader compiles, which can
/// hold single frames for seconds): this long and this many frames.
const SETTLE: f32 = 1.0;
const SETTLE_FRAMES: usize = 10;
/// Then measured for this long and this many frames, or at most `MEASURE_MAX` seconds.
const MEASURE: f32 = 2.0;
const MEASURE_FRAMES: usize = 30;
const MEASURE_MAX: f32 = 8.0;

pub struct DetailBenchmarkPlugin;

impl Plugin for DetailBenchmarkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (start_requested, measure).chain().run_if(in_state(AppState::Menu)),
        )
        .add_systems(OnExit(AppState::Menu), abandon)
        .add_systems(
            EguiPrimaryContextPass,
            notice.run_if(in_state(AppState::Menu)).run_if(resource_exists::<Notice>),
        );
        if STARTS.load(std::sync::atomic::Ordering::Relaxed) {
            if mobile() {
                info!("detail benchmark: a phone or tablet");
            }
            app.insert_resource(Bench::new());
        }
    }
}

/// OPTIONS > GRAPHICS' DETECT was pressed (`request`).
static REQUESTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Runs the benchmark again from LOW (OPTIONS > GRAPHICS' DETECT).
pub fn request() {
    REQUESTED.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// DETECT pressed: LOW, then the benchmark climbs from there.
fn start_requested(
    mut commands: Commands,
    bench: Option<Res<Bench>>,
    mut settings: ResMut<Settings>,
) {
    if !REQUESTED.swap(false, std::sync::atomic::Ordering::Relaxed) || bench.is_some() {
        return;
    }
    LEVELS[0].apply(&mut settings);
    commands.remove_resource::<Notice>();
    commands.insert_resource(Bench::new());
}

/// The graphics presets of OPTIONS > GRAPHICS: HIGH is a new player's, MEDIUM and LOW are the
/// benchmark's steps down from it, ULTRA turns every costly part on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Preset {
    Low,
    Medium,
    High,
    Ultra,
}

impl Preset {
    pub const ALL: [Preset; 4] = [Preset::Low, Preset::Medium, Preset::High, Preset::Ultra];

    /// Its name's message.
    pub fn name_id(self) -> &'static str {
        match self {
            Preset::Low => "options-preset-low",
            Preset::Medium => "options-preset-medium",
            Preset::High => "options-preset-high",
            Preset::Ultra => "options-preset-ultra",
        }
    }

    /// The preset's graphics and scenery distances.
    fn graphics(self) -> (Graphics, f32) {
        let defaults = crate::prefs::Prefs::default();
        let mut g = Graphics {
            modern_shading: true,
            msaa: defaults.msaa,
            shadows: defaults.shadows,
            local_lights: defaults.local_lights,
            cloud_quality: defaults.cloud_quality,
            effects: crate::effects::Effects::default(),
        };
        let distance = crate::settings::DEFAULT_SCENERY_DISTANCE;
        match self {
            Preset::High => (g, distance),
            Preset::Medium | Preset::Low => {
                let steps = if self == Preset::Medium { 1 } else { STEPS };
                for step in 0..steps {
                    lower(&mut g, step);
                }
                (g, distance)
            }
            Preset::Ultra => {
                (g.msaa, g.shadows, g.local_lights) = (true, true, true);
                g.cloud_quality = CloudQuality::Ultra;
                set_heavy(&mut g.effects, true);
                (g, ULTRA_DISTANCE)
            }
        }
    }

    pub fn apply(self, s: &mut Settings) {
        let (g, distance) = self.graphics();
        let effects = s.effects;
        g.write(s);
        // Only the costly effects are the preset's; the looks (retro, shake...) stay.
        s.effects = effects;
        for (on, set) in heavy(&g.effects).into_iter().zip(heavy_mut(&mut s.effects)) {
            *set = on;
        }
        s.scenery_distance = distance;
        s.city_light_distance = distance;
    }

    /// The preset the settings are, if any (else they are the player's own).
    pub fn of(s: &Settings) -> Option<Preset> {
        Preset::ALL.into_iter().find(|p| {
            let (g, distance) = p.graphics();
            let now = Graphics::of(s);
            (now.modern_shading, now.msaa, now.shadows, now.local_lights, now.cloud_quality)
                == (g.modern_shading, g.msaa, g.shadows, g.local_lights, g.cloud_quality)
                && heavy(&now.effects) == heavy(&g.effects)
                && s.scenery_distance == distance
                && s.city_light_distance == distance
        })
    }
}

/// The ULTRA preset's scenery and city-light distances (m).
const ULTRA_DISTANCE: f32 = 15_000.0;

/// The costly effects (the ones the benchmark turns off).
fn heavy(e: &crate::effects::Effects) -> [bool; 8] {
    [e.mist, e.cloud_shadows, e.ocean, e.glass, e.water, e.ground_detail, e.sunbeams, e.vapour]
}

fn heavy_mut(e: &mut crate::effects::Effects) -> [&mut bool; 8] {
    [
        &mut e.mist,
        &mut e.cloud_shadows,
        &mut e.ocean,
        &mut e.glass,
        &mut e.water,
        &mut e.ground_detail,
        &mut e.sunbeams,
        &mut e.vapour,
    ]
}

fn set_heavy(e: &mut crate::effects::Effects, on: bool) {
    for f in heavy_mut(e) {
        *f = on;
    }
}

/// Whether no pipeline is being built (kept by the browser's loading screen; none: quiet).
#[derive(Resource)]
pub struct PipelinesQuiet(pub bool);

/// The browser's loading screen covers the game: the notice waits.
#[derive(Resource)]
pub struct Covered;

/// The benchmark gives up after this long (s; the scene up and measuring), at the last level
/// that kept up.
pub const MAX_TIME: f32 = 45.0;

/// Running while present (the browser's loading screen waits for it, the desktop's shows while
/// it runs).
#[derive(Resource, Default)]
pub struct Bench {
    target: f32,
    /// The level being measured, and the highest that kept up (`LEVELS`).
    level: usize,
    good: Option<usize>,
    /// Seconds and frames since the scene was up or the last change, then since measuring
    /// began.
    since: f32,
    settle_frames: usize,
    measuring: bool,
    frames: Vec<f32>,
    /// The graphics as last set: anything else means the player changed them.
    set: Option<Graphics>,
    /// Seconds since the scene was up.
    total: f32,
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

/// A median frame time keeps up with the target frame rate.
fn keeps_up(median: f32, target_fps: f32) -> bool {
    1.0 / median.max(1e-6) >= target_fps
}

fn measure(
    mut commands: Commands,
    bench: Option<ResMut<Bench>>,
    mut settings: ResMut<Settings>,
    director: Option<Res<crate::menu_scene::director::Director>>,
    time: Res<Time<Real>>,
    quiet: Option<Res<PipelinesQuiet>>,
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
    bench.set = Some(now);
    if !page_visible() {
        bench.restart();
        return;
    }
    let dt = time.delta_secs();
    bench.total += dt;
    if bench.total > MAX_TIME {
        // This level never settled (pipelines still coming, frames of seconds): the last that
        // kept up.
        info!("detail benchmark: out of time at {:?}", LEVELS[bench.level]);
        bench.settle(&mut settings);
        return finish(&mut commands, &bench, true);
    }
    let Some((m, frames)) = bench.tick(dt, quiet.is_none_or(|q| q.0)) else { return };
    // A frame rate limit below the target is not the computer's slowness.
    let target = match settings.fps_cap {
        0 => bench.target,
        cap => bench.target.min(f32::from(cap) * 0.9),
    };
    let level = LEVELS[bench.level];
    info!(
        "detail benchmark: {level:?}, {frames} frames, median {:.1} ms ({:.0} fps)",
        m * 1000.0,
        1.0 / m
    );
    if !keeps_up(m, target) {
        bench.settle(&mut settings);
        return finish(&mut commands, &bench, true);
    }
    bench.good = Some(bench.level);
    if bench.level + 1 == LEVELS.len() {
        return finish(&mut commands, &bench, true);
    }
    bench.level += 1;
    let next = LEVELS[bench.level];
    info!("detail benchmark: up to {next:?}");
    next.apply(&mut settings);
    bench.set = Some(Graphics::of(&settings));
    bench.restart();
}

impl Bench {
    fn new() -> Self {
        let target = std::env::var("SM89_DETAIL_BENCHMARK_FPS")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(TARGET_FPS);
        Self { target, ..default() }
    }

    /// Back to the last level that kept up (LOW if none did), if not there.
    fn settle(&mut self, settings: &mut Settings) {
        let good = self.good.unwrap_or(0);
        if good != self.level {
            self.level = good;
            LEVELS[good].apply(settings);
            self.set = Some(Graphics::of(settings));
        }
        info!("detail benchmark: settled at {:?}", LEVELS[good]);
    }

    /// Seconds since the scene was up and the benchmark ran (the desktop's screen shows it).
    pub fn running_for(&self) -> f32 {
        self.total
    }

    /// One frame of `dt` seconds: settling (while the pipelines are being built, then
    /// `SETTLE` seconds and `SETTLE_FRAMES` frames), then measuring; once measured, the median
    /// frame time and the frames measured.
    fn tick(&mut self, dt: f32, pipelines_quiet: bool) -> Option<(f32, usize)> {
        if !self.measuring {
            // Settled once the pipelines are built (a stall of seconds each in the browser).
            if !pipelines_quiet {
                self.restart();
                return None;
            }
            self.since += dt;
            self.settle_frames += 1;
            if self.settle_frames >= SETTLE_FRAMES && self.since >= SETTLE {
                // Measured from the next frame.
                self.measuring = true;
                self.since = 0.0;
            }
            return None;
        }
        self.since += dt;
        self.frames.push(dt);
        let n = self.frames.len();
        if (self.since < MEASURE || n < MEASURE_FRAMES) && self.since < MEASURE_MAX {
            return None;
        }
        let m = median(&mut self.frames);
        self.restart();
        Some((m, n))
    }

    /// Settles and measures again.
    fn restart(&mut self) {
        self.since = 0.0;
        self.settle_frames = 0;
        self.measuring = false;
        self.frames.clear();
    }
}

/// Done: remembered; a notice if it is below HIGH (`noticed`: not stopped by the player).
fn finish(commands: &mut Commands, bench: &Bench, noticed: bool) {
    let version = if mobile() { MOBILE_VERSION } else { VERSION };
    if let Err(e) = crate::prefs::save_json(DONE_FILE, &version) {
        warn!("saving {DONE_FILE}: {e}");
    }
    if noticed && bench.level + 1 < LEVELS.len() {
        commands.insert_resource(Notice { shown_for: 0.0 });
    }
    commands.remove_resource::<Bench>();
}

/// Left the menu before it was done: the last level that kept up (if the settings are still
/// the benchmark's), and it runs again on the next visit.
fn abandon(mut commands: Commands, bench: Option<ResMut<Bench>>, mut settings: ResMut<Settings>) {
    if let Some(mut bench) = bench {
        info!("detail benchmark: left the menu, not finished");
        if bench.set.as_ref().is_none_or(|g| *g == Graphics::of(&settings)) {
            bench.settle(&mut settings);
        }
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
    covered: Option<Res<Covered>>,
) -> Result {
    use crate::menu::{BRIGHT, big_button, menu_window};
    if covered.is_some() {
        return Ok(());
    }
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

    /// Frames of `dt` until a measurement comes: the measured frames and the seconds taken.
    fn run(bench: &mut Bench, dt: f32) -> (usize, f32) {
        let mut t = 0.0;
        loop {
            t += dt;
            if let Some((_, n)) = bench.tick(dt, true) {
                return (n, t);
            }
        }
    }

    #[test]
    fn measures_after_settling() {
        // 60 fps: `SETTLE` seconds settling, `MEASURE` measured.
        let (n, t) = run(&mut Bench::default(), 1.0 / 60.0);
        let expected = (MEASURE * 60.0) as usize;
        assert!((expected - 1..=expected + 1).contains(&n), "{n}");
        assert!((t - (SETTLE + MEASURE)).abs() < 0.05, "{t}");
        // 4 fps: the frame counts hold it longer, up to `MEASURE_MAX`.
        let (n, t) = run(&mut Bench::default(), 0.25);
        assert!(n == MEASURE_FRAMES && t > SETTLE + MEASURE, "{n} {t}");
        let (n, _) = run(&mut Bench::default(), 1.0);
        assert!(n < MEASURE_FRAMES && n as f32 >= MEASURE_MAX - 1.0, "{n}");
        // Pipelines being built: settling starts again.
        let mut b = Bench::default();
        for _ in 0..200 {
            assert!(b.tick(1.0 / 60.0, false).is_none());
        }
        assert_eq!((b.settle_frames, b.measuring), (0, false));
    }

    #[test]
    fn phones_and_tablets() {
        let pixel = "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like \
                     Gecko) Chrome/130.0.0.0 Mobile Safari/537.36";
        let iphone = "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15";
        let mac = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15";
        let linux = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/130.0.0.0";
        assert!(mobile_agent(None, pixel, 5));
        assert!(mobile_agent(None, iphone, 5));
        assert!(mobile_agent(Some(true), linux, 0));
        // iPadOS' Safari: a Mac with a touch screen.
        assert!(mobile_agent(None, mac, 5));
        assert!(!mobile_agent(None, mac, 0));
        assert!(!mobile_agent(Some(false), linux, 0));
    }

    #[test]
    fn keeps_up_with_the_target() {
        assert!(keeps_up(1.0 / 60.0, TARGET_FPS));
        assert!(!keeps_up(1.0 / 30.0, TARGET_FPS));
    }

    /// Back to the last level that kept up; LOW if none did.
    #[test]
    fn settles_at_the_last_good_level() {
        let mut s = Settings::from_args();
        let mut b = Bench { level: 2, good: Some(1), ..default() };
        Preset::High.apply(&mut s);
        b.settle(&mut s);
        assert_eq!((b.level, Preset::of(&s)), (1, Some(Preset::Medium)));
        let mut b = Bench { level: 0, ..default() };
        b.settle(&mut s);
        assert_eq!(b.level, 0);
        let mut b = Bench { level: 1, ..default() };
        b.settle(&mut s);
        assert_eq!(Preset::of(&s), Some(Preset::Low));
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

    /// A new player's graphics are HIGH; each preset applied is itself, MEDIUM and LOW are the
    /// benchmark's steps down from HIGH; the looks (retro, shake) are not the presets'.
    #[test]
    fn presets() {
        let mut s = Settings::from_args();
        crate::prefs::Prefs::default().apply(&mut crate::menu::MenuState::default(), &mut s);
        assert_eq!(Preset::of(&s), Some(Preset::High));
        s.effects.scanlines = true;
        for p in Preset::ALL {
            p.apply(&mut s);
            assert_eq!(Preset::of(&s), Some(p), "{p:?}");
            assert!(s.effects.scanlines);
        }
        Preset::High.apply(&mut s);
        let mut g = Graphics::of(&s);
        lower(&mut g, 0);
        g.write(&mut s);
        assert_eq!(Preset::of(&s), Some(Preset::Medium));
        s.shadows = true;
        assert_eq!(Preset::of(&s), None);
    }
}
