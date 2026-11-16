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

/// (knots, sea state) of the fully developed sea (Douglas scale).
const SEA: [(f32, f32); 9] = [
    (0.0, 0.0),
    (3.0, 1.0),
    (8.0, 2.0),
    (13.0, 3.0),
    (19.0, 4.0),
    (25.0, 5.0),
    (31.0, 6.0),
    (38.0, 7.0),
    (45.0, 8.0),
];

/// Sea state 0..8 (continuous) a steady wind of `kt` raises.
pub fn sea_state(kt: f32) -> f32 {
    let kt = kt.max(0.0);
    for w in SEA.windows(2) {
        let ((k0, s0), (k1, s1)) = (w[0], w[1]);
        if kt <= k1 {
            return s0 + (s1 - s0) * (kt - k0) / (k1 - k0);
        }
    }
    8.0
}

/// The steady wind (knots) that raises sea state `sea` ([`sea_state`] inverted): what a sea
/// still building up or calming down looks like it was raised by.
pub fn sea_wind_kt(sea: f32) -> f32 {
    let sea = sea.clamp(0.0, 8.0);
    for w in SEA.windows(2) {
        let ((k0, s0), (k1, s1)) = (w[0], w[1]);
        if sea <= s1 {
            return k0 + (k1 - k0) * (sea - s0) / (s1 - s0);
        }
    }
    SEA[SEA.len() - 1].0
}

fn table(sea: f32, t: [f32; 9]) -> f32 {
    let s = sea.clamp(0.0, 8.0);
    let i = (s.floor() as usize).min(7);
    t[i] + (t[i + 1] - t[i]) * (s - i as f32)
}

/// Significant wave height (metres) at a sea state.
pub fn wave_height_m(sea: f32) -> f32 {
    table(sea, [0.0, 0.1, 0.3, 0.9, 1.9, 3.2, 5.0, 7.5, 11.0])
}

/// Dominant wave period (seconds) at a sea state.
pub fn wave_period_s(sea: f32) -> f32 {
    table(sea, [2.0, 2.5, 3.5, 5.0, 6.5, 8.0, 9.5, 11.0, 12.5])
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
/// The response falls with length: a carrier at sea state 7 moves about ±1.5 m, ±1° pitch,
/// ±2° roll; a frigate several degrees; small boats a lot.
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
    let ratio = (wave_len / len).min(1.5);
    let heave_amp = (0.5 * h * ratio.min(1.0)).min(if len > 250.0 { 1.5 } else { 8.0 });
    let pitch_amp = (h / len * 0.9 * ratio).min(if len > 250.0 { 1f32.to_radians() } else { 0.2 });
    let roll_amp = (h / len.sqrt() * 0.12).min(if len > 250.0 { 2f32.to_radians() } else { 0.35 });
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
    let roll_f = (period as f64 / (4.0 + (len as f64).sqrt() * 0.6)).clamp(0.4, 1.2);
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
        for kt in [0.0, 5.0, 17.5, 33.0, 45.0] {
            assert!((sea_wind_kt(sea_state(kt)) - kt).abs() < 1e-3, "{kt}");
        }
        assert_eq!(sea_state(0.0), 0.0);
        assert!((sea_state(13.0) - 3.0).abs() < 1e-5);
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
        assert!(max.heave <= 1.5 && max.heave > 0.3, "{max:?}");
        assert!(max.pitch <= 1f32.to_radians() + 1e-6, "{max:?}");
        assert!(max.roll <= 2f32.to_radians() + 1e-6 && max.roll > 0.2f32.to_radians(), "{max:?}");
        let small = ship_motion(7.0, 30.0, 1, 1.3);
        assert!(small.roll.abs() > 0.0);
        assert_eq!(ship_motion(0.0, 330.0, 1, 3.0), ShipMotion::default());
    }
}
