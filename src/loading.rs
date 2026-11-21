//! The loading screen on the desktop: entering a flight covers the view with the mission's title
//! and a bar of the shader pipelines built, until no new pipeline has come for `QUIET_FRAMES`
//! frames (at most `MAX_WAIT`). A flight's first frames build tens of pipelines, stalling for
//! seconds; under the screen they are not seen, and a single mission's clock waits.
//!
//! Not in VR (the headset shows its runtime's own until the first frame), not in the browser
//! (its page's loading screen, `web_loading.rs`), nor in test runs that take screenshots or
//! follow a script (their frames and clocks would be the screen's; `--loading-screen` shows it
//! there too).
//!
//! The render world's pipeline count (`Pipelines`) is the browser loading screen's too.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use bevy::prelude::*;
use bevy::render::render_resource::{CachedPipelineState, PipelineCache};
use stealth_i18n::t;

use crate::settings::{AppState, Settings, SetupSet};

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        let pipelines = Pipelines::default();
        app.insert_resource(pipelines.clone());
        if let Some(render) = app.get_sub_app_mut(bevy::render::RenderApp) {
            render.insert_resource(pipelines).add_systems(
                bevy::render::Render,
                count_pipelines.in_set(bevy::render::RenderSystems::Cleanup),
            );
        }
        if !cover_wanted() {
            return;
        }
        app.init_resource::<Cover>()
            .add_systems(OnEnter(AppState::Flight), cover.after(SetupSet::Spawn))
            .add_systems(Update, lift.run_if(in_state(AppState::Flight)))
            .add_systems(Update, cover_benchmark.run_if(in_state(AppState::Menu)))
            .add_systems(
                bevy_egui::EguiPrimaryContextPass,
                draw_benchmark_cover.run_if(in_state(AppState::Menu)),
            );
    }
}

/// The render world's pipelines, for the main world: how many there are and how many of them
/// are not built yet.
#[derive(Resource, Clone, Default)]
pub struct Pipelines {
    total: Arc<AtomicU32>,
    pending: Arc<AtomicU32>,
}

impl Pipelines {
    /// (all, not built yet).
    pub fn counts(&self) -> (u32, u32) {
        (self.total.load(Ordering::Relaxed), self.pending.load(Ordering::Relaxed))
    }
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

/// The flight is drawn with all its pipelines once none has come for this many frames.
const QUIET_FRAMES: u32 = 8;
/// The screen goes after this long anyway (s).
const MAX_WAIT: f32 = 30.0;

/// The desktop outside VR, not a test run.
fn cover_wanted() -> bool {
    if cfg!(target_arch = "wasm32") {
        return false;
    }
    // `--loading-screen`: also in a test run (screenshots of it).
    if std::env::args().any(|a| a == "--loading-screen") {
        return true;
    }
    let off = ["--vr", "--no-loading-screen"];
    !test_run() && !std::env::args().any(|a| off.contains(&a.as_str()))
}

/// A test run: it takes screenshots or follows a script (no screens over it, no benchmark).
pub fn test_run() -> bool {
    const TESTS: [&str; 4] = ["--screenshot", "--script", "--shots", "--shot-time"];
    std::env::args().any(|a| TESTS.contains(&a.as_str()))
}

/// The screen's state while it is up.
#[derive(Resource, Default)]
struct Cover {
    up: bool,
    /// It paused the clock (a single mission), to start again when it goes.
    paused: bool,
    waited: f32,
    quiet: u32,
    pipelines: u32,
    /// The most pipelines seen pending at once (the bar's whole).
    most_pending: u32,
}

#[derive(Component)]
struct CoverRoot;

#[derive(Component)]
struct CoverBar;

#[derive(Component)]
struct CoverDetail;

/// The screen's colours: the menus' dark green, their bright and dim greens.
const BG: Color = Color::srgb(0.02, 0.05, 0.04);
const BRIGHT: Color = Color::srgb(0.45, 1.0, 0.7);
const DIM: Color = Color::srgb(0.2, 0.55, 0.38);

/// OnEnter(Flight), once the aircraft is in place: the screen over the view.
#[allow(clippy::too_many_arguments)]
fn cover(
    mut commands: Commands,
    mut state: ResMut<Cover>,
    settings: Res<Settings>,
    font: Res<crate::i18n::UiFont>,
    mission: Option<Res<crate::mission::ActiveMission>>,
    campaign: Option<Res<crate::campaign::ActiveCampaign>>,
    link: Option<Res<crate::net::NetLink>>,
    mut time: ResMut<Time<Virtual>>,
) {
    *state = Cover::default();
    if settings.vr {
        return;
    }
    state.up = true;
    // A single mission waits; a campaign's or a server's world goes on.
    if campaign.is_none() && link.is_none() && !time.is_paused() {
        time.pause();
        state.paused = true;
    }
    let title = mission
        .map(|m| crate::mission::clean_text(&m.0.briefing.title).to_uppercase())
        .unwrap_or_default();
    spawn_screen(&mut commands, &font, title, t!("loading-title"), AppState::Flight);
}

/// The screen: a title over the heading, the bar and a line under it.
fn spawn_screen(
    commands: &mut Commands,
    font: &crate::i18n::UiFont,
    title: String,
    heading: &str,
    state: AppState,
) {
    commands
        .spawn((
            CoverRoot,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(14),
                ..default()
            },
            BackgroundColor(BG),
            GlobalZIndex(100),
            DespawnOnExit(state),
        ))
        .with_children(|p| {
            p.spawn((Text::new(title), font.at(22.0), TextColor(DIM)));
            p.spawn((Text::new(heading), font.at(40.0), TextColor(BRIGHT)));
            p.spawn((
                Node { width: px(420), height: px(10), border: UiRect::all(px(1)), ..default() },
                BorderColor::all(DIM),
            ))
            .with_children(|p| {
                p.spawn((
                    CoverBar,
                    Node { width: percent(0), height: percent(100), ..default() },
                    BackgroundColor(BRIGHT),
                ));
            });
            p.spawn((CoverDetail, Text::new(""), font.at(16.0), TextColor(DIM)));
        });
}

/// The bar follows the pipelines built; the screen goes once none has come for a while.
#[allow(clippy::too_many_arguments)]
fn lift(
    mut commands: Commands,
    mut state: ResMut<Cover>,
    pipelines: Res<Pipelines>,
    real: Res<Time<Real>>,
    mut time: ResMut<Time<Virtual>>,
    menu: Res<crate::flight_menu::FlightMenu>,
    roots: Query<Entity, With<CoverRoot>>,
    mut bar: Query<&mut Node, With<CoverBar>>,
    mut detail: Query<&mut Text, With<CoverDetail>>,
) {
    if !state.up {
        return;
    }
    let (total, pending) = pipelines.counts();
    state.waited += real.delta_secs();
    if total != state.pipelines || pending > 0 {
        state.pipelines = total;
        state.quiet = 0;
    } else {
        state.quiet += 1;
    }
    state.most_pending = state.most_pending.max(pending);
    let built = state.most_pending.saturating_sub(pending);
    let fraction = if state.most_pending == 0 {
        if state.quiet > 0 { state.quiet as f32 / QUIET_FRAMES as f32 } else { 0.0 }
    } else {
        built as f32 / state.most_pending as f32
    };
    if let Ok(mut node) = bar.single_mut() {
        node.width = percent((fraction * 100.0).clamp(0.0, 100.0));
    }
    if let Ok(mut text) = detail.single_mut() {
        let line = if pending > 0 {
            t!("loading-shaders", done = built, total = state.most_pending).to_string()
        } else {
            String::new()
        };
        if text.0 != line {
            text.0 = line;
        }
    }
    if state.quiet < QUIET_FRAMES && state.waited < MAX_WAIT {
        return;
    }
    info!("loading screen: up {:.1} s ({total} pipelines)", state.waited);
    state.up = false;
    for e in &roots {
        commands.entity(e).despawn();
    }
    // The clock goes on, unless the in-flight menu holds it (a loaded save starts so).
    if std::mem::take(&mut state.paused) && !menu.open {
        time.unpause();
    }
}

/// The graphics benchmark running in the menu (`detail_benchmark.rs`; a new player's first
/// start, DETECT): it hears when no pipeline is being built (`PipelinesQuiet`), and the menu
/// waits under a screen (`draw_benchmark_cover`).
fn cover_benchmark(
    mut commands: Commands,
    bench: Option<Res<crate::detail_benchmark::Bench>>,
    pipelines: Res<Pipelines>,
    settings: Res<Settings>,
    mut state: ResMut<Cover>,
) {
    if bench.is_none() || settings.vr {
        if state.up {
            state.up = false;
            commands.remove_resource::<crate::detail_benchmark::PipelinesQuiet>();
            commands.remove_resource::<crate::detail_benchmark::Covered>();
        }
        return;
    }
    if !state.up {
        *state = Cover { up: true, ..default() };
        commands.insert_resource(crate::detail_benchmark::Covered);
    }
    let (total, pending) = pipelines.counts();
    if total != state.pipelines || pending > 0 {
        state.pipelines = total;
        state.quiet = 0;
    } else {
        state.quiet += 1;
    }
    commands.insert_resource(crate::detail_benchmark::PipelinesQuiet(state.quiet >= QUIET_FRAMES));
}

/// The benchmark's screen over the menu (egui, which draws over Bevy's interface): the
/// heading, a bar of its progress and the seconds; it takes the pointer.
fn draw_benchmark_cover(
    mut contexts: bevy_egui::EguiContexts,
    bench: Option<Res<crate::detail_benchmark::Bench>>,
    settings: Res<Settings>,
) -> Result {
    use bevy_egui::egui;
    let Some(bench) = bench.filter(|_| !settings.vr) else { return Ok(()) };
    let ctx = contexts.ctx_mut()?;
    let screen = ctx.content_rect();
    let [r, g, b, _] = BG.to_srgba().to_u8_array();
    let colour = |c: Color| {
        let [r, g, b, _] = c.to_srgba().to_u8_array();
        egui::Color32::from_rgb(r, g, b)
    };
    egui::Area::new(egui::Id::new("benchmark cover"))
        .order(egui::Order::Tooltip)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(screen.size(), egui::Sense::click_and_drag());
            let p = ui.painter();
            p.rect_filled(rect, 0.0, egui::Color32::from_rgb(r, g, b));
            let mid = rect.center();
            let font = |size: f32| egui::FontId::monospace(size);
            p.text(
                mid - egui::vec2(0.0, 28.0),
                egui::Align2::CENTER_CENTER,
                t!("loading-detect"),
                font(34.0),
                colour(BRIGHT),
            );
            let bar =
                egui::Rect::from_center_size(mid + egui::vec2(0.0, 10.0), egui::vec2(420.0, 10.0));
            p.rect_stroke(bar, 0.0, egui::Stroke::new(1.0, colour(DIM)), egui::StrokeKind::Inside);
            let t = bench.running_for();
            let mut fill = bar;
            fill.set_width(bar.width() * bench.progress());
            p.rect_filled(fill, 0.0, colour(BRIGHT));
            p.text(
                mid + egui::vec2(0.0, 36.0),
                egui::Align2::CENTER_CENTER,
                t!("loading-detect-time", seconds = t as u32),
                font(15.0),
                colour(DIM),
            );
        });
    ctx.request_repaint();
    Ok(())
}
