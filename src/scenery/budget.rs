//! Automatic scenery detail in VR: a governor watches the frame times against the headset's
//! frame rate and steps a scenery budget level, which shortens the scenery's streaming ranges,
//! thins the distant towns and shortens the models' detail level ranges. Graphics settings stay
//! as chosen; the scenery gives way so they run at the headset's rate.
//!
//! The rate is learnt from the frames (the best two-second average, at least `MIN_HZ`):
//! SteamVR on the Steam Frame reports a 90 Hz display period while it runs the app at 72 Hz.
//!
//! Against oscillation: short dips don't count (it takes two bad two-second windows, or one
//! very bad one, which steps down two levels), frames that build scenery or settle a change are left out, raising detail
//! waits for 15 good seconds, and a level that failed soon after being raised to is not tried
//! again for a minute, then two, four... (at most ten).

use bevy::prelude::*;

use super::content::Tier;
use crate::settings::{AppState, Settings};

/// Budget levels: 0 is full detail, `MAX` the shortest ranges.
pub const MAX: u8 = 8;

/// Streaming ranges (metres) and the models' detail level range factor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ranges {
    main: f64,
    town: f64,
    detail: f64,
    far: f64,
    distant: f64,
    fields: f64,
    pub lod: f32,
    /// Share of the houses the town tier draws.
    pub houses: f64,
}

impl Ranges {
    pub fn radius(&self, tier: Tier) -> f64 {
        match tier {
            Tier::Main => self.main,
            Tier::Town => self.town,
            Tier::Detail => self.detail,
            Tier::Far => self.far,
            Tier::Distant => self.distant,
            Tier::Fields => self.fields,
        }
    }
}

/// Level 0 (full detail, also outside VR) and level `MAX`; the levels between are spaced
/// evenly in ratio.
const FULL: Ranges = Ranges {
    main: 2500.0,
    town: 7000.0,
    detail: 1000.0,
    far: 2000.0,
    distant: 4000.0,
    fields: 24_000.0,
    lod: 1.0,
    houses: 1.0,
};
const LEAST: Ranges = Ranges {
    main: 500.0,
    town: 3000.0,
    detail: 400.0,
    far: 800.0,
    distant: 1600.0,
    fields: 10_000.0,
    lod: 0.4,
    houses: 0.3,
};

pub fn ranges(level: u8) -> Ranges {
    let t = level.min(MAX) as f64 / MAX as f64;
    let mix = |a: f64, b: f64| a * (b / a).powf(t);
    Ranges {
        main: mix(FULL.main, LEAST.main),
        town: mix(FULL.town, LEAST.town),
        detail: mix(FULL.detail, LEAST.detail),
        far: mix(FULL.far, LEAST.far),
        distant: mix(FULL.distant, LEAST.distant),
        fields: mix(FULL.fields, LEAST.fields),
        lod: mix(FULL.lod as f64, LEAST.lod as f64) as f32,
        houses: mix(FULL.houses, LEAST.houses),
    }
}

/// The scenery budget level in use (0 = full detail).
#[derive(Resource, Default)]
pub struct SceneryBudget {
    pub level: u8,
}

impl SceneryBudget {
    pub fn ranges(&self) -> Ranges {
        ranges(self.level)
    }
}

/// Scenery blocks built this frame.
#[derive(Resource, Default)]
pub struct BlocksBuilt(pub u32);

/// The lowest frame rate aimed for (the Steam Frame's default), and the highest.
const MIN_HZ: f32 = 72.0;
const MAX_HZ: f32 = 144.0;
/// A frame is late beyond this many frame periods.
const LATE: f32 = 1.3;
/// Frames longer than this are hitches (loading, streaming), not a measure of the load.
const HITCH: f32 = 0.25;
/// Seconds of frames measured per verdict.
const WINDOW: f32 = 2.0;
/// Share of late frames in a window: bad (two in a row step down), very bad (one steps down),
/// good (15 seconds of them step up).
const BAD: f32 = 0.15;
const VERY_BAD: f32 = 0.5;
const GOOD: f32 = 0.02;
const RAISE_AFTER: f32 = 15.0;
/// Seconds after a change or a hitch before frames count again, and at the start of a flight
/// (whose first seconds are slow anyway).
const SETTLE: f32 = 1.5;
const START_SETTLE: f32 = 8.0;
/// A level dropped within this many seconds of being raised to failed: it waits out a
/// cooldown, `COOLDOWN` the first time, doubled each time after, at most `COOLDOWN_MAX`.
const FAILED_WITHIN: f32 = 20.0;
const COOLDOWN: f32 = 60.0;
const COOLDOWN_MAX: f32 = 600.0;

/// The level stepping (independent of Bevy, for the tests).
#[derive(Debug)]
pub struct Governor {
    pub level: u8,
    /// The frame period aimed for (seconds).
    pace: f32,
    frames: u32,
    late: u32,
    measured: f32,
    prev_bad: bool,
    /// Seconds before frames count (`SETTLE`).
    settle: f32,
    good_for: f32,
    /// Seconds since detail was raised (to `level`).
    since_raise: Option<f32>,
    /// Seconds each level must still wait before it may be raised to, and its next cooldown.
    cooldown: [f32; MAX as usize + 1],
    penalty: [f32; MAX as usize + 1],
}

impl Default for Governor {
    fn default() -> Self {
        Self {
            level: 0,
            pace: 1.0 / MIN_HZ,
            frames: 0,
            late: 0,
            measured: 0.0,
            prev_bad: false,
            settle: START_SETTLE,
            good_for: 0.0,
            since_raise: None,
            cooldown: [0.0; MAX as usize + 1],
            penalty: [COOLDOWN / 2.0; MAX as usize + 1],
        }
    }
}

impl Governor {
    /// One frame of `dt` seconds; `skip`: a frame not to judge (it built scenery). The new
    /// level, when it changes.
    pub fn frame(&mut self, dt: f32, skip: bool) -> Option<u8> {
        if let Some(t) = &mut self.since_raise {
            *t += dt;
        }
        for c in &mut self.cooldown {
            *c = (*c - dt).max(0.0);
        }
        if dt > HITCH {
            self.settle = self.settle.max(SETTLE);
            return None;
        }
        if self.settle > 0.0 {
            self.settle -= dt;
            return None;
        }
        if skip {
            return None;
        }
        self.frames += 1;
        self.late += (dt > self.pace * LATE) as u32;
        self.measured += dt;
        if self.measured < WINDOW {
            return None;
        }
        let share = self.late as f32 / self.frames as f32;
        let window = self.measured;
        self.pace = self.pace.min(window / self.frames as f32).clamp(1.0 / MAX_HZ, 1.0 / MIN_HZ);
        (self.frames, self.late, self.measured) = (0, 0, 0.0);
        let bad = share > BAD;
        let drop = (share > VERY_BAD || (bad && self.prev_bad)) && self.level < MAX;
        self.prev_bad = bad;
        if drop {
            let failed = self.level as usize;
            if self.since_raise.is_some_and(|t| t < FAILED_WITHIN) {
                self.penalty[failed] = (self.penalty[failed] * 2.0).min(COOLDOWN_MAX);
                self.cooldown[failed] = self.penalty[failed];
            }
            // Far behind (SteamVR halving the frame rate): two levels at once.
            let by = if share > VERY_BAD { 2 } else { 1 };
            return Some(self.change((self.level + by).min(MAX)));
        }
        if share >= GOOD {
            self.good_for = 0.0;
            return None;
        }
        self.good_for += window;
        if self.good_for >= RAISE_AFTER
            && self.level > 0
            && self.cooldown[self.level as usize - 1] <= 0.0
        {
            let level = self.change(self.level - 1);
            self.since_raise = Some(0.0);
            return Some(level);
        }
        None
    }

    fn change(&mut self, level: u8) -> u8 {
        self.level = level;
        self.settle = SETTLE;
        self.good_for = 0.0;
        self.prev_bad = false;
        self.since_raise = None;
        (self.frames, self.late, self.measured) = (0, 0, 0.0);
        level
    }
}

/// `SM89_AUTO_DETAIL`: `0` turns the governor off (full detail), `fixed:N` holds level N.
fn env_override() -> Option<u8> {
    let v = std::env::var("SM89_AUTO_DETAIL").ok()?;
    if v == "0" {
        return Some(0);
    }
    v.strip_prefix("fixed:").and_then(|n| n.parse().ok()).map(|n: u8| n.min(MAX))
}

pub struct BudgetPlugin;

impl Plugin for BudgetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneryBudget>()
            .init_resource::<BlocksBuilt>()
            .add_systems(OnEnter(AppState::Flight), reset)
            .add_systems(Last, govern.run_if(in_state(AppState::Flight)));
    }
}

fn reset(mut commands: Commands, mut budget: ResMut<SceneryBudget>) {
    commands.insert_resource(GovernorState(Governor::default()));
    budget.level = env_override().unwrap_or(0);
}

#[derive(Resource)]
struct GovernorState(Governor);

fn govern(
    time: Res<Time<Real>>,
    settings: Res<Settings>,
    built: Res<BlocksBuilt>,
    mut last_built: Local<bool>,
    governor: Option<ResMut<GovernorState>>,
    mut budget: ResMut<SceneryBudget>,
) {
    let Some(mut governor) = governor else { return };
    if let Some(level) = env_override() {
        budget.level = level;
        return;
    }
    if !(settings.vr && settings.vr_auto_detail) {
        if budget.level != 0 {
            budget.level = 0;
            governor.0 = Governor::default();
        }
        return;
    }
    // A build's frame and the next one (pipelined rendering) don't count.
    let skip = built.0 > 0 || *last_built;
    *last_built = built.0 > 0;
    let from = governor.0.level;
    if let Some(level) = governor.0.frame(time.delta_secs(), skip) {
        let hz = 1.0 / governor.0.pace;
        info!("auto detail: scenery level {from} -> {level} (aiming for {hz:.0} fps)");
        budget.level = level;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const P: f32 = 1.0 / 72.0;

    /// A governor past its start.
    fn governor() -> Governor {
        let mut g = Governor::default();
        assert!(run(&mut g, START_SETTLE + 1.0, 0.0).is_empty());
        g
    }

    /// Runs `secs` of frames, `late_share` of them two periods long; the level changes.
    fn run(g: &mut Governor, secs: f32, late_share: f32) -> Vec<u8> {
        let mut changes = Vec::new();
        let (mut t, mut acc) = (0.0, 0.0);
        while t < secs {
            acc += late_share;
            let dt = if acc >= 1.0 {
                acc -= 1.0;
                2.0 * P
            } else {
                P
            };
            t += dt;
            changes.extend(g.frame(dt, false));
        }
        changes
    }

    #[test]
    fn slow_start_doesnt_count() {
        let mut g = Governor::default();
        assert!(run(&mut g, START_SETTLE - 0.5, 1.0).is_empty());
    }

    #[test]
    fn short_dip_keeps_detail() {
        let mut g = governor();
        run(&mut g, 10.0, 0.0);
        assert!(run(&mut g, 0.6, 1.0).is_empty());
        assert!(run(&mut g, 10.0, 0.0).is_empty());
        assert_eq!(g.level, 0);
    }

    #[test]
    fn sustained_misses_step_down() {
        let mut g = governor();
        run(&mut g, 10.0, 0.25);
        assert!(g.level >= 1);
        let mut g = governor();
        // Halved frame rate: down a level about every 3.5 s.
        run(&mut g, 10.0, 1.0);
        assert!(g.level >= 2);
    }

    #[test]
    fn steps_up_after_quiet() {
        let mut g = governor();
        run(&mut g, 6.0, 0.25);
        let level = g.level;
        assert!(level >= 1);
        assert!(run(&mut g, 12.0, 0.0).is_empty());
        run(&mut g, 8.0, 0.0);
        assert_eq!(g.level, level - 1);
    }

    #[test]
    fn skipped_frames_dont_count() {
        let mut g = governor();
        for _ in 0..72 * 20 {
            assert_eq!(g.frame(3.0 * P, true), None);
        }
        for _ in 0..20 {
            assert_eq!(g.frame(0.5, false), None);
        }
        assert_eq!(g.level, 0);
    }

    #[test]
    fn learns_a_faster_headset() {
        // 90 Hz: a frame missed now and then (72 fps on average) is late once the governor
        // has seen 90.
        let mut g = governor();
        let mut changes = Vec::new();
        for _ in 0..90 * 5 {
            changes.extend(g.frame(1.0 / 90.0, false));
        }
        assert!(changes.is_empty());
        for i in 0..72 * 8 {
            changes.extend(g.frame(if i % 4 == 0 { 2.0 } else { 1.0 } / 90.0, false));
        }
        assert!(!changes.is_empty());
    }

    #[test]
    fn no_ping_pong() {
        // Level 2 holds, level 1 doesn't: after the first failures it rarely tries 1 again.
        let mut g = governor();
        let mut changes = 0;
        let mut t = 0.0;
        while t < 600.0 {
            let late = if g.level >= 2 { 0.0 } else { 0.3 };
            changes += run(&mut g, 1.0, late).len();
            t += 1.0;
        }
        assert_eq!(g.level, 2);
        // Down to 2, then a few probes of 1 at 15 s, 1 min, 2 min, 4 min...
        assert!(changes <= 12, "{changes} changes");
    }
}
