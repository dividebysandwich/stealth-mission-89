//! Wind: a mean wind (direction, speed), its gusts and the air's turbulence, and the sea
//! state it raises. Picked from hashes of the weather's seed rather than from an RNG stream,
//! so missions and the weather roll exactly as before.
//!
//! Only the player's flight model, the rain, the smoke, the sea and the displays use it; the
//! simulation (AI flight, bombs) flies in still air.

use serde::{Deserialize, Serialize};

use crate::weather::{Sky, Weather};

/// Wind strength preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum WindLevel {
    #[default]
    Calm,
    Light,
    Moderate,
    Strong,
    Gale,
}

impl WindLevel {
    pub const ALL: [WindLevel; 5] =
        [Self::Calm, Self::Light, Self::Moderate, Self::Strong, Self::Gale];

    pub fn name(self) -> &'static str {
        match self {
            Self::Calm => "Calm",
            Self::Light => "Light",
            Self::Moderate => "Moderate",
            Self::Strong => "Strong",
            Self::Gale => "Gale",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.name().eq_ignore_ascii_case(s))
    }

    /// Mean speed range (knots).
    pub fn knots(self) -> (f32, f32) {
        match self {
            Self::Calm => (0.0, 3.0),
            Self::Light => (5.0, 10.0),
            Self::Moderate => (12.0, 18.0),
            Self::Strong => (20.0, 30.0),
            Self::Gale => (32.0, 45.0),
        }
    }

    /// Gusts above the mean (knots range).
    fn gusts(self) -> (f32, f32) {
        match self {
            Self::Calm => (0.0, 0.0),
            Self::Light => (0.0, 5.0),
            Self::Moderate => (5.0, 10.0),
            Self::Strong => (10.0, 15.0),
            Self::Gale => (12.0, 20.0),
        }
    }

    /// The preset a mean speed belongs to.
    pub fn of_knots(kt: f32) -> Self {
        match kt {
            k if k < 4.0 => Self::Calm,
            k if k < 11.0 => Self::Light,
            k if k < 19.0 => Self::Moderate,
            k if k < 31.0 => Self::Strong,
            _ => Self::Gale,
        }
    }

    /// Odds (percent) of [`WindLevel::ALL`] under a sky.
    pub fn odds(sky: Sky) -> [u8; 5] {
        match sky {
            Sky::Clear => [25, 40, 25, 9, 1],
            Sky::Scattered => [15, 40, 30, 13, 2],
            Sky::Broken => [10, 30, 38, 18, 4],
            Sky::Overcast => [5, 25, 40, 23, 7],
            Sky::Storm => [0, 10, 30, 40, 20],
        }
    }
}

/// Prevailing wind (degrees true it blows from) per theater ([`crate::THEATER_NAMES`] order).
const PREVAILING: [f32; 12] = [
    330.0, // Libya
    320.0, // Persian Gulf: the shamal
    240.0, // North Cape
    260.0, // Central Europe
    45.0,  // Vietnam: north-east monsoon
    290.0, // Middle East
    320.0, // Desert Storm
    80.0,  // Cuba: trade winds
    300.0, // Korea
    270.0, // Ukraine
    240.0, // Gulf of Finland
    260.0, // Suwalki Gap
];

/// The theater's prevailing wind direction (degrees true, from).
pub fn prevailing(theater: usize) -> f32 {
    PREVAILING[theater.min(PREVAILING.len() - 1)]
}

/// The wind of a mission or of a moment of the campaign.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Wind {
    /// Direction it blows from, degrees true.
    pub from_deg: f32,
    /// Mean speed at 2000 ft (knots).
    pub kt: f32,
    /// Peak gusts (knots, at least `kt`).
    pub gust_kt: f32,
    /// Background turbulence 0..1 (storms and low level add their own).
    pub turbulence: f32,
    /// Seed of the gusts and turbulence.
    pub seed: u32,
}

// Never NaN, so equality is total (the mission is `Eq`).
impl Eq for Wind {}

/// 64-bit mixer (splitmix finaliser).
pub fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A value 0..1 from a seed and a salt.
pub fn unit(seed: u64, salt: u64) -> f32 {
    (mix(seed ^ mix(salt)) >> 40) as f32 / (1u64 << 24) as f32
}

impl Wind {
    /// A wind of `level` from `from_deg`, its speed, gusts and turbulence picked by `seed`.
    pub fn of_level(level: WindLevel, from_deg: f32, seed: u64) -> Self {
        let (lo, hi) = level.knots();
        let kt = lo + (hi - lo) * unit(seed, 1);
        let (glo, ghi) = level.gusts();
        let gust = glo + (ghi - glo) * unit(seed, 2);
        let base = match level {
            WindLevel::Calm => 0.0,
            WindLevel::Light => 0.05,
            WindLevel::Moderate => 0.15,
            WindLevel::Strong => 0.35,
            WindLevel::Gale => 0.6,
        };
        Self {
            from_deg: from_deg.rem_euclid(360.0),
            kt,
            gust_kt: kt + gust,
            turbulence: (base * (0.8 + 0.4 * unit(seed, 3))).min(1.0),
            seed: (mix(seed ^ 0x5749_4E44) >> 32) as u32,
        }
    }

    /// A random level fitting the sky, from `seed`.
    pub fn pick_level(sky: Sky, seed: u64) -> WindLevel {
        let mut n = (unit(seed, 11) * 100.0) as i32;
        WindLevel::ALL
            .into_iter()
            .zip(WindLevel::odds(sky))
            .find(|&(_, p)| {
                n -= p as i32;
                n < 0
            })
            .map_or(WindLevel::Light, |(l, _)| l)
    }

    /// A mission's wind: `level` (or one fitting the sky), from about the theater's
    /// prevailing direction (± 90°), all picked from the weather's seed.
    pub fn roll(level: Option<WindLevel>, weather: &Weather, theater: usize) -> Self {
        let seed = mix(weather.seed as u64 ^ 0x5749_4E44_0000_0000);
        let level = level.unwrap_or_else(|| Self::pick_level(weather.sky, seed));
        let from = prevailing(theater) + (unit(seed, 7) - 0.5) * 180.0;
        Self::of_level(level, from, seed)
    }

    /// The preset this wind's mean speed belongs to.
    pub fn level(&self) -> WindLevel {
        WindLevel::of_knots(self.kt)
    }

    /// Where the air moves to, as (east, north) knots.
    pub fn velocity(&self) -> (f32, f32) {
        let to = (self.from_deg + 180.0).to_radians();
        (self.kt * to.sin(), self.kt * to.cos())
    }

    /// Blend from `a` to `b` (direction the short way round).
    pub fn lerp(a: &Wind, b: &Wind, p: f32) -> Wind {
        let d = ((b.from_deg - a.from_deg + 540.0).rem_euclid(360.0)) - 180.0;
        Wind {
            from_deg: (a.from_deg + d * p).rem_euclid(360.0),
            kt: a.kt + (b.kt - a.kt) * p,
            gust_kt: a.gust_kt + (b.gust_kt - a.gust_kt) * p,
            turbulence: a.turbulence + (b.turbulence - a.turbulence) * p,
            seed: if p < 0.5 { a.seed } else { b.seed },
        }
    }

    /// Direction rounded to 10° (360 for north) as the tower reads it.
    pub fn heading_10(&self) -> i32 {
        let d = ((self.from_deg / 10.0).round() as i32 * 10).rem_euclid(360);
        if d == 0 { 360 } else { d }
    }

    /// The briefing line, e.g. "Wind 270° at 15 kt, gusting 25 kt." (player's language).
    pub fn describe(&self) -> String {
        use stealth_i18n::t;
        let kt = self.kt.round() as i32;
        if kt < 3 {
            return t!("wx-wind-calm").to_string();
        }
        let gust = self.gust_kt.round() as i32;
        if gust >= kt + 5 {
            t!("wx-wind-gusts", dir = self.heading_10(), kt = kt, gust = gust)
        } else {
            t!("wx-wind", dir = self.heading_10(), kt = kt)
        }
    }
}

/// Probable wave heights of the open sea by the wind (knots, metres): the WMO Beaufort scale's
/// (as the Met Office's and the Hong Kong Observatory's tables give it), at each force's middle.
const BEAUFORT: [(f32, f32); 13] = [
    (0.0, 0.0),
    (2.0, 0.1),
    (5.0, 0.2),
    (8.5, 0.6),
    (13.5, 1.0),
    (19.0, 2.0),
    (24.5, 3.0),
    (30.5, 4.0),
    (37.0, 5.5),
    (44.0, 7.0),
    (51.5, 9.0),
    (59.5, 11.5),
    (68.0, 14.0),
];

/// (significant wave height in metres, sea state) at the middles of the WMO sea state code's
/// classes (code 3700, NATO STANAG 4194): 3 slight 0.5-1.25 m, 4 moderate 1.25-2.5 m, 5 rough
/// 2.5-4 m, 6 very rough 4-6 m, 7 high 6-9 m, 8 very high 9-14 m.
const CODES: [(f32, f32); 9] = [
    (0.0, 0.0),
    (0.05, 1.0),
    (0.3, 2.0),
    (0.875, 3.0),
    (1.875, 4.0),
    (3.25, 5.0),
    (5.0, 6.0),
    (7.5, 7.0),
    (11.5, 8.0),
];

/// Linear interpolation in a table of increasing (x, y), clamped at its ends.
fn interp(t: &[(f32, f32)], x: f32) -> f32 {
    if x <= t[0].0 {
        return t[0].1;
    }
    for w in t.windows(2) {
        let ((x0, y0), (x1, y1)) = (w[0], w[1]);
        if x <= x1 {
            return y0 + (y1 - y0) * (x - x0) / (x1 - x0);
        }
    }
    t[t.len() - 1].1
}

/// The table read backwards (y to x; y increasing too).
fn interp_back(t: &[(f32, f32)], y: f32) -> f32 {
    if y <= t[0].1 {
        return t[0].0;
    }
    for w in t.windows(2) {
        let ((x0, y0), (x1, y1)) = (w[0], w[1]);
        if y <= y1 {
            return x0 + (x1 - x0) * (y - y0) / (y1 - y0);
        }
    }
    t[t.len() - 1].0
}

/// Probable wave height (metres) of the open sea a steady wind of `kt` raises (Beaufort).
pub fn wind_wave_height_m(kt: f32) -> f32 {
    interp(&BEAUFORT, kt.max(0.0))
}

/// Sea state 0..8 (continuous) a steady wind of `kt` raises.
pub fn sea_state(kt: f32) -> f32 {
    interp(&CODES, wind_wave_height_m(kt))
}

/// The steady wind (knots) that raises sea state `sea` ([`sea_state`] inverted): what a sea
/// still building up or calming down looks like it was raised by.
pub fn sea_wind_kt(sea: f32) -> f32 {
    interp_back(&BEAUFORT, wave_height_m(sea))
}

fn table(sea: f32, t: [f32; 9]) -> f32 {
    let s = sea.clamp(0.0, 8.0);
    let i = (s.floor() as usize).min(7);
    t[i] + (t[i + 1] - t[i]) * (s - i as f32)
}

/// Significant wave height (metres) at a sea state.
pub fn wave_height_m(sea: f32) -> f32 {
    interp_back(&CODES, sea.clamp(0.0, 8.0))
}

/// Dominant wave period (seconds) at a sea state (the wind sea's, rising to the most probable
/// periods of STANAG 4194's open ocean: 9 s at sea state 4, 12 s at 6, 17 s at 8).
pub fn wave_period_s(sea: f32) -> f32 {
    table(sea, [2.0, 2.5, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 17.0])
}

/// A hull's natural periods (seconds) of heave and pitch, and of roll, by its length (metres):
/// pitch by Sandia's fit for warships (SAND2008-1020: 6.5 s at 152 m, 9 s at 300 m), roll from
/// about 5 s for a patrol boat through 10-11 s for a destroyer to the 21 s measured on a Nimitz
/// class carrier (Pattison & Bushway, AGARD CP-509).
pub fn hull_periods(length_m: f32) -> (f32, f32) {
    let l = length_m.clamp(10.0, 350.0);
    (2.44 + 0.032 * l - 0.000_036 * l * l, 3.0 + 0.055 * l)
}

/// A ship's motion on the waves at time `t` (seconds).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ShipMotion {
    /// Up (metres).
    pub heave: f32,
    /// Bow up (radians).
    pub pitch: f32,
    /// Starboard down (radians).
    pub roll: f32,
    /// Heave rate (metres/second).
    pub heave_rate: f32,
}

/// How a ship `length_m` long, with its own `seed`, rides a sea state `sea` at time `t`.
/// The response falls with length: a carrier at sea state 7 moves about ±2.5 m, ±1.5° pitch,
/// ±3° roll; a frigate several degrees; small boats a lot.
pub fn ship_motion(sea: f32, length_m: f32, seed: u32, t: f64) -> ShipMotion {
    let h = wave_height_m(sea);
    if h <= 0.0 {
        return ShipMotion::default();
    }
    let period = wave_period_s(sea);
    let len = length_m.max(10.0);
    // Wave length of the dominant waves (deep water): g T² / 2π.
    let wave_len = 9.81 * period * period / std::f32::consts::TAU;
    // A hull much longer than the waves averages them out.
    let ratio = (wave_len / len).min(1.0);
    let heave_amp = (0.5 * h * ratio).min(if len > 250.0 { 2.5 } else { 8.0 });
    let pitch_amp =
        (h / len * 1.3 * ratio).min(if len > 250.0 { 1.5f32.to_radians() } else { 0.2 });
    // A carrier's long roll period and wide beam: half the roll (about 1° significant amplitude
    // in sea state 4, the NAVSEA deck motion criteria's).
    let roll_k = if len > 250.0 { 0.15 } else { 0.3 };
    let roll_amp =
        (h / len.sqrt() * roll_k).min(if len > 250.0 { 3f32.to_radians() } else { 0.35 });
    let s = seed as f64;
    let ph = |k: f64| (s * 0.618_034 * k).fract() * std::f64::consts::TAU;
    let w = std::f64::consts::TAU / period as f64;
    // Two incommensurate components per motion so it never looks like a metronome.
    let wave = |k: f64, f: f64| {
        ((w * f * t + ph(k)).sin() * 0.7 + (w * f * 1.37 * t + ph(k + 1.0)).sin() * 0.3) as f32
    };
    let dwave = |k: f64, f: f64| {
        (w * f
            * ((w * f * t + ph(k)).cos() * 0.7
                + 1.37 * (w * f * 1.37 * t + ph(k + 1.0)).cos() * 0.3)) as f32
    };
    // Roll at the hull's own (longer) period.
    let roll_f = (period as f64 / hull_periods(len).1 as f64).clamp(0.3, 1.2);
    ShipMotion {
        heave: heave_amp * wave(1.0, 1.0),
        pitch: pitch_amp * wave(3.0, 1.0),
        roll: roll_amp * wave(5.0, roll_f),
        heave_rate: heave_amp * dwave(1.0, 1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_parse_and_classify() {
        for l in WindLevel::ALL {
            assert_eq!(WindLevel::parse(&l.name().to_lowercase()), Some(l));
            let (lo, hi) = l.knots();
            assert_eq!(WindLevel::of_knots((lo + hi) / 2.0), l);
        }
    }

    #[test]
    fn roll_is_deterministic_and_fits_the_sky() {
        let mut storms = 0.0;
        let mut clear = 0.0;
        for seed in 0..400u32 {
            let mut w = Weather { seed, sky: Sky::Storm, ..Default::default() };
            let a = Wind::roll(None, &w, 3);
            assert_eq!(a, Wind::roll(None, &w, 3));
            assert!(a.gust_kt >= a.kt);
            storms += a.kt;
            w.sky = Sky::Clear;
            clear += Wind::roll(None, &w, 3).kt;
        }
        assert!(storms > clear * 2.0, "storms {storms} clear {clear}");
        let w = Weather { seed: 9, ..Default::default() };
        assert_eq!(Wind::roll(Some(WindLevel::Gale), &w, 0).level(), WindLevel::Gale);
    }

    #[test]
    fn lerp_goes_the_short_way() {
        let a = Wind { from_deg: 350.0, ..Default::default() };
        let b = Wind { from_deg: 10.0, ..Default::default() };
        let m = Wind::lerp(&a, &b, 0.5).from_deg;
        assert!(m < 1.0 || m > 359.0, "{m}");
    }

    #[test]
    fn sea_rises_with_wind() {
        for kt in [0.0, 5.0, 17.5, 33.0, 45.0, 55.0] {
            assert!((sea_wind_kt(sea_state(kt)) - kt).abs() < 1e-3, "{kt}");
        }
        assert_eq!(sea_state(0.0), 0.0);
        // Force 5 (19 kt): 2 m, moderate; force 9 (44 kt): 7 m, high.
        assert!((wave_height_m(sea_state(19.0)) - 2.0).abs() < 1e-3);
        assert!((sea_state(19.0) - 4.1).abs() < 0.05, "{}", sea_state(19.0));
        assert!((wave_height_m(sea_state(44.0)) - 7.0).abs() < 1e-3);
        assert_eq!(sea_state(80.0), 8.0);
        assert!(wave_height_m(5.5) > wave_height_m(5.0));
    }

    #[test]
    fn carrier_motion_stays_small() {
        let mut max = ShipMotion::default();
        for i in 0..4000 {
            let m = ship_motion(7.0, 330.0, 1234, i as f64 * 0.05);
            max.heave = max.heave.max(m.heave.abs());
            max.pitch = max.pitch.max(m.pitch.abs());
            max.roll = max.roll.max(m.roll.abs());
        }
        assert!(max.heave <= 2.5 && max.heave > 0.3, "{max:?}");
        assert!(max.pitch <= 1.5f32.to_radians() + 1e-6, "{max:?}");
        assert!(max.roll <= 3f32.to_radians() + 1e-6 && max.roll > 0.2f32.to_radians(), "{max:?}");
        let small = ship_motion(7.0, 30.0, 1, 1.3);
        assert!(small.roll.abs() > 0.0);
        assert_eq!(ship_motion(0.0, 330.0, 1, 3.0), ShipMotion::default());
    }
}
