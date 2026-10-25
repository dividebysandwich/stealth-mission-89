//! The world around the aircraft (`ambient_*.flac`), looped through the cockpit's low-pass
//! (`muffle`), at sound level 3:
//!
//! - Night crickets in the late evening and at night (21:00 to 03:00) near the ground.
//!   No crickets at all, night or desert, while the weather has storms.
//! - Light rain while it rains around the eye, heavy rain on top of it under a storm cell (not
//!   inside the clouds).
//! - Summer by day in fair weather in Central Europe and Ukraine, fading out from 500 ft to
//!   1000 ft and as the eye moves from 50 kt to 100 kt.
//! - Crickets around noon in the desert theaters, outside the aircraft only.
//! - Wind out of the aircraft above 300 ft, the high-altitude wind on top above 4000 ft.
//!
//! A sound whose time, weather or place comes or goes fades over `FADE` (real seconds); moving
//! the view in or out of the aircraft fades the outside-only sounds over `VIEW_FADE`. Altitude
//! takes no time: each altitude band's volume follows the height alone, crossfading across a
//! span of height (`*_BAND`), so climbing through it never cuts; the same for the eye's speed.
//! A sound gone quiet pauses, and goes on from where it was when it is heard again.

use bevy::audio::{PlaybackMode, PlaybackSettings, Volume};
use bevy::math::DVec3;
use bevy::prelude::*;
use stealth_mission::weather::Sky;

use super::muffle::{self, Muffler};
use super::{Channel, SoundLevel};
use crate::camera::{EyeCamera, View, ViewState};
use crate::flight::{Aircraft, Player};
use crate::weather::WeatherState;
use crate::world::FloatingOrigin;

/// Fade of a sound coming into or out of its time (real seconds).
const FADE: f32 = 60.0;
/// Fade of the outside-only sounds when the view moves in or out of the aircraft.
const VIEW_FADE: f32 = 0.3;

/// The sounds: file and volume (they are recorded at different levels).
const BEDS: [(&str, f32); 7] = [
    ("ambient_cricketsnight.flac", 0.7),
    ("ambient_crickets.flac", 0.5),
    ("ambient_rainlight.flac", 0.9),
    ("ambient_rain.flac", 0.6),
    ("ambient_summer.flac", 0.5),
    ("ambient_wind.flac", 0.8),
    ("ambient_windhighalt.flac", 0.5),
];
const NIGHT_CRICKETS: usize = 0;
const CRICKETS: usize = 1;
const RAIN_LIGHT: usize = 2;
const RAIN: usize = 3;
const SUMMER: usize = 4;
const WIND: usize = 5;
const WIND_HIGH: usize = 6;

/// Altitude bands (ft above the ground): each sound fades in or out across its band.
const CRICKETS_BAND: (f32, f32) = (60.0, 140.0);
const SUMMER_BAND: (f32, f32) = (500.0, 1000.0);
const WIND_BAND: (f32, f32) = (200.0, 400.0);
const WIND_HIGH_BAND: (f32, f32) = (3500.0, 4500.0);
/// The summer fades out as the eye moves faster across this band (knots).
const SUMMER_SPEED: (f32, f32) = (50.0, 100.0);
/// Time constant (s) smoothing the eye's speed, and the speed (m/s) beyond which a move is a
/// jump of the view, not flight.
const SPEED_TAU: f32 = 0.5;
const JUMP_SPEED: f32 = 2000.0;
const KNOTS_PER_MS: f32 = 1.943_844;

/// The night crickets' hours (local time, around midnight), the summer's daylight and the
/// desert crickets' noon.
const NIGHT: (f32, f32) = (21.0, 3.0);
const DAY: (f32, f32) = (6.5, 18.0);
const NOON: (f32, f32) = (11.0, 14.0);

const SUMMER_THEATERS: [&str; 2] = ["central_europe", "ukraine"];
const DESERT_THEATERS: [&str; 4] = ["libya", "persian_gulf", "middle_east", "desert_storm"];

/// The loaded sounds and how far each has faded in (by time and place, and by view).
#[derive(Resource, Default)]
pub(super) struct Ambience {
    beds: [Option<Handle<AudioSource>>; 7],
    fade: [f32; 7],
    view: [f32; 7],
    /// The flight has just started: the sounds start where they are.
    fresh: bool,
    /// The eye last frame (world position and view), and its speed (knots, smoothed).
    eye: Option<(DVec3, View)>,
    knots: f32,
}

/// An ambient loop (index into `BEDS`), playing or paused while it is not heard.
#[derive(Component)]
pub(super) struct AmbientLoop(usize);

pub(super) fn load(mut ambience: ResMut<Ambience>, server: Res<AssetServer>) {
    for (bed, (file, _)) in ambience.beds.iter_mut().zip(BEDS) {
        let path = format!("sounds/{file}");
        *bed = crate::asset_path::exists(&path).then(|| server.load(path));
    }
}

pub(super) fn reset(mut ambience: ResMut<Ambience>) {
    ambience.fresh = true;
    ambience.eye = None;
    ambience.knots = 0.0;
}

/// What the ambient sounds follow.
struct Around<'a> {
    /// Local time (hours, 0..24).
    hours: f32,
    /// Height of the eye above the ground (ft), and its speed (knots).
    agl: f32,
    knots: f32,
    /// The view is out of the aircraft.
    outside: bool,
    theater: &'a str,
    raining: bool,
    /// Under a storm cell.
    storm: bool,
    /// The weather has storms (somewhere).
    stormy: bool,
    /// Fair weather (clear or scattered, not in a cloud's shade).
    sunny: bool,
}

/// Each sound's due: in its time and place, heard from this view, and its gain by the eye's
/// altitude (and speed).
fn due(a: &Around) -> [(bool, bool, f32); 7] {
    let within = |(from, to): (f32, f32)| {
        if from <= to { (from..to).contains(&a.hours) } else { a.hours >= from || a.hours < to }
    };
    let below = |(lo, hi): (f32, f32)| 1.0 - smoothstep(lo, hi, a.agl);
    let above = |(lo, hi): (f32, f32)| smoothstep(lo, hi, a.agl);
    let mut d = [(false, true, 1.0); 7];
    d[NIGHT_CRICKETS] = (within(NIGHT) && !a.stormy, true, below(CRICKETS_BAND));
    d[CRICKETS] =
        (within(NOON) && !a.stormy && DESERT_THEATERS.contains(&a.theater), a.outside, 1.0);
    d[RAIN_LIGHT] = (a.raining, true, 1.0);
    d[RAIN] = (a.raining && a.storm, true, 1.0);
    let slow = 1.0 - smoothstep(SUMMER_SPEED.0, SUMMER_SPEED.1, a.knots);
    d[SUMMER] = (
        within(DAY) && a.sunny && SUMMER_THEATERS.contains(&a.theater),
        true,
        below(SUMMER_BAND) * slow,
    );
    d[WIND] = (true, a.outside, above(WIND_BAND));
    d[WIND_HIGH] = (true, a.outside, above(WIND_HIGH_BAND));
    d
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Moves `v` towards 1 (`on`) or 0 at `1 / secs` a second.
fn approach(v: f32, on: bool, dt: f32, secs: f32) -> f32 {
    let step = dt / secs;
    if on { (v + step).min(1.0) } else { (v - step).max(0.0) }
}

/// The eye's speed (knots, smoothed) from its move since last frame; a change of view or a jump
/// is not counted.
fn eye_speed(ambience: &mut Ambience, eye: Option<(DVec3, View)>, dt: f32) -> f32 {
    let last = std::mem::replace(&mut ambience.eye, eye);
    if let (Some((p, view)), Some((q, last_view))) = (eye, last)
        && view == last_view
        && dt > 0.0
    {
        let speed = (p.distance(q) as f32) / dt;
        if speed < JUMP_SPEED {
            let k = 1.0 - (-dt / SPEED_TAU).exp();
            ambience.knots += (speed * KNOTS_PER_MS - ambience.knots) * k;
        }
    }
    ambience.knots
}

/// Fades the ambient sounds in and out, starting their loops and pausing them while silent.
#[allow(clippy::too_many_arguments)]
pub(super) fn ambience(
    mut commands: Commands,
    mut ambience: ResMut<Ambience>,
    level: Res<SoundLevel>,
    view: Option<Res<ViewState>>,
    weather: Res<WeatherState>,
    theater: Option<Res<crate::world::Theater>>,
    clock: Res<crate::mfd::MissionClock>,
    player: Query<&Aircraft, With<Player>>,
    real: Res<Time<Real>>,
    virt: Res<Time<Virtual>>,
    origin: Res<FloatingOrigin>,
    eye: Query<&GlobalTransform, With<EyeCamera>>,
    settings: Res<crate::settings::Settings>,
    mut loops: Query<(&AmbientLoop, Option<&mut AudioSink>)>,
    mut muffler: Muffler,
) {
    let a = player.single().ok();
    let at = eye
        .single()
        .ok()
        .zip(view.as_deref())
        .map(|(tf, v)| (origin.origin + tf.translation().as_dvec3(), v.view));
    let knots = eye_speed(&mut ambience, at, virt.delta_secs());
    let seconds = clock.start_seconds + clock.elapsed.unwrap_or(a.map_or(0.0, |a| a.time));
    let around = Around {
        hours: (seconds / 3600.0).rem_euclid(24.0),
        agl: weather.eye_ft - a.map_or(0.0, |a| a.ground_altitude),
        knots,
        outside: !muffle::inside(view.as_deref()),
        theater: theater.as_ref().map_or("", |t| t.id.as_str()),
        // Under the clouds, not in them.
        raining: weather.raining() && weather.whiteout < 0.5,
        storm: weather.under_storm,
        stormy: weather.weather.has_storms(),
        sunny: matches!(weather.weather.sky, Sky::Clear | Sky::Scattered)
            && weather.shade < 0.5
            && !weather.raining(),
    };
    let dt = real.delta_secs();
    let fresh = std::mem::take(&mut ambience.fresh);
    let gain = if level.0 == 3 { Channel::Effects.gain(&settings) } else { 0.0 };
    let mut volumes = [0.0; 7];
    for (i, (timely, heard, band)) in due(&around).into_iter().enumerate() {
        let (fade, view) = if fresh {
            (timely as u8 as f32, heard as u8 as f32)
        } else {
            (
                approach(ambience.fade[i], timely, dt, FADE),
                approach(ambience.view[i], heard, dt, VIEW_FADE),
            )
        };
        ambience.fade[i] = fade;
        ambience.view[i] = view;
        volumes[i] = BEDS[i].1 * fade * view * band * gain;
    }
    // Silent loops pause where they are (rodio reads no samples of a paused sink).
    let mut have = [false; 7];
    for (l, sink) in &mut loops {
        have[l.0] = true;
        let Some(mut sink) = sink else { continue };
        let v = volumes[l.0];
        if v <= 1e-4 {
            if !sink.is_paused() {
                sink.pause();
            }
            continue;
        }
        sink.set_volume(Volume::Linear(v));
        if sink.is_paused() {
            sink.play();
        }
    }
    for (i, &v) in volumes.iter().enumerate() {
        if have[i] || v <= 1e-4 {
            continue;
        }
        let Some(handle) = &ambience.beds[i] else { continue };
        let Some((handle, pitch)) = muffler.looped(handle, 1.0) else { continue };
        debug!("ambient {}", BEDS[i].0);
        commands.spawn((
            AudioPlayer(handle),
            PlaybackSettings {
                // The filtered sound loops itself.
                mode: PlaybackMode::Once,
                volume: Volume::Linear(v),
                ..default()
            },
            pitch,
            AmbientLoop(i),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn around() -> Around<'static> {
        Around {
            hours: 12.0,
            agl: 0.0,
            knots: 0.0,
            outside: true,
            theater: "central_europe",
            raining: false,
            storm: false,
            stormy: false,
            sunny: true,
        }
    }

    /// Storms silence the crickets, the night's and the desert's.
    #[test]
    fn no_crickets_in_storms() {
        let night = Around { hours: 23.0, stormy: true, ..around() };
        assert!(!due(&night)[NIGHT_CRICKETS].0);
        let desert = Around { theater: "libya", stormy: true, ..around() };
        assert!(!due(&desert)[CRICKETS].0);
        assert!(due(&Around { stormy: false, ..desert })[CRICKETS].0);
    }

    /// The night crickets sing from the late evening to about 3 am near the ground, fading out
    /// on the way up.
    #[test]
    fn night_crickets() {
        let at = |hours, agl| due(&Around { hours, agl, ..around() })[NIGHT_CRICKETS];
        assert!(at(23.0, 0.0).0 && at(2.5, 0.0).0 && at(21.5, 0.0).0);
        assert!(!at(3.5, 0.0).0 && !at(12.0, 0.0).0 && !at(20.0, 0.0).0);
        assert_eq!(at(23.0, 50.0).2, 1.0);
        let mid = at(23.0, 100.0).2;
        assert!(mid > 0.3 && mid < 0.7, "{mid}");
        assert_eq!(at(23.0, 200.0).2, 0.0);
    }

    /// The wind is heard out of the aircraft only, crossfading in with height; the high wind
    /// joins it above 4000 ft.
    #[test]
    fn wind_by_altitude_and_view() {
        let at = |agl, outside| due(&Around { agl, outside, ..around() });
        assert_eq!(at(100.0, true)[WIND].2, 0.0);
        assert_eq!(at(1000.0, true)[WIND].2, 1.0);
        assert!(!at(1000.0, false)[WIND].1);
        assert_eq!(at(1000.0, true)[WIND_HIGH].2, 0.0);
        assert_eq!(at(6000.0, true)[WIND_HIGH].2, 1.0);
        let w = at(4000.0, true)[WIND_HIGH].2;
        assert!(w > 0.3 && w < 0.7, "{w}");
    }

    /// The summer fades out as the eye speeds up from 50 to 100 kt.
    #[test]
    fn summer_by_speed() {
        let summer = |knots| due(&Around { knots, ..around() })[SUMMER].2;
        assert_eq!(summer(40.0), 1.0);
        assert!(summer(75.0) > 0.3 && summer(75.0) < 0.7);
        assert_eq!(summer(100.0), 0.0);
    }

    /// The eye's speed follows its moves, but not a change of view or a jump.
    #[test]
    fn eye_speed_skips_jumps() {
        let mut a = Ambience::default();
        let mut knots = 0.0;
        for i in 0..1000 {
            let p = DVec3::X * (i as f64 * 0.5);
            knots = eye_speed(&mut a, Some((p, View::Chase)), 0.01);
        }
        assert!((knots - 50.0 * KNOTS_PER_MS).abs() < 1.0, "{knots}");
        let before = knots;
        let jump = eye_speed(&mut a, Some((DVec3::X * 5000.0, View::Chase)), 0.01);
        let switch = eye_speed(&mut a, Some((DVec3::X * 5040.0, View::Cockpit)), 0.01);
        assert_eq!((jump, switch), (before, before));
    }

    /// Summer in fair daylight in Central Europe and Ukraine; desert crickets at noon, outside.
    #[test]
    fn places() {
        let d = |theater, hours, sunny| due(&Around { theater, hours, sunny, ..around() });
        assert!(d("ukraine", 10.0, true)[SUMMER].0);
        assert!(!d("ukraine", 10.0, false)[SUMMER].0);
        assert!(!d("ukraine", 20.0, true)[SUMMER].0);
        assert!(!d("libya", 10.0, true)[SUMMER].0);
        let summer = |agl| due(&Around { agl, ..around() })[SUMMER].2;
        assert_eq!(summer(500.0), 1.0);
        assert!(summer(750.0) > 0.3 && summer(750.0) < 0.7);
        assert_eq!(summer(1000.0), 0.0);
        assert!(d("libya", 12.0, true)[CRICKETS].0);
        assert!(!d("libya", 16.0, true)[CRICKETS].0);
        assert!(!d("korea", 12.0, true)[CRICKETS].0);
        let inside = due(&Around { theater: "libya", outside: false, ..around() });
        assert!(!inside[CRICKETS].1);
    }

    /// A minute to fade in, the outside-only sounds a moment on changing the view.
    #[test]
    fn fades() {
        let mut v = 0.0;
        for _ in 0..30 {
            v = approach(v, true, 1.0, FADE);
        }
        assert!((v - 0.5).abs() < 1e-5);
        assert_eq!(approach(0.0, true, 0.3, VIEW_FADE), 1.0);
        assert_eq!(approach(0.5, false, 1.0, FADE), 0.5 - 1.0 / 60.0);
    }
}
