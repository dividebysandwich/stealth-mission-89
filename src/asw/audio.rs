//! The sensor operator's headset: one sensor at a time (the buoy or the dipping sonar whose
//! waterfall the listening cursor was last put on), made live on the audio thread from its
//! reports.
//!
//! Off any line the cursor hears only the sea: a faint brown noise. On a contact's line (within
//! `CURSOR_BAND` of one of its tones) that contact comes up:
//! - a submarine: the hum of its machinery's tone and harmonic (raised four octaves into
//!   hearing, as the operator's audio processor does) and a slow beat of its screw;
//! - a surface ship: its screws, the recorded `ship-big1_loop` (the carrier) or
//!   `ship-med1_loop` (the others; made to loop by tools/gen_ship_loops.py), played faster or
//!   slower with its blade rate, under its machinery's hum.
//!
//! Another sonar's ping heard by the sensor (`Sample::heard`) plays `sonarbuoy.flac`; the
//! sensor's own ping a 3.5 kHz tone with its echo after the range's two-way travel time.

use std::num::NonZero;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use bevy::audio::{AddAudioSource, AudioSource, ChannelCount, Decodable, SampleRate, Source, Volume};
use bevy::prelude::*;

use super::AswUi;
use crate::settings::AppState;

/// The synthesiser's rate: the recordings' (48 kHz).
const RATE: u32 = 48_000;
/// The tonals are heard this many times higher (four octaves).
const TRANSPOSE: f32 = 16.0;
const PING_HZ: f32 = 3500.0;
const PING_SECS: f32 = 0.35;
/// Sound in sea water (m/s).
const SOUND_SPEED: f32 = 1500.0;
/// Contacts heard at once (those whose line the cursor is on).
const SOURCES: usize = 4;
/// The cursor's band: a tone this far off it (Hz) is down to 1/e; past `CURSOR_EDGE` bands
/// nothing of it is heard.
const CURSOR_BAND: f32 = 4.0;
const CURSOR_EDGE: f32 = 2.5;
/// The sea's brown noise: how loud, a little more with the sea state.
const SEA: f32 = 0.05;
/// A ship's screws at this blade rate (Hz) play the recording as it is.
const RECORDED_BLADE: f32 = 12.0;
/// Ships turning their screws slower than this are the big ones (the carrier's 11 Hz).
const BIG_BLADE: u16 = 13;

pub struct AswAudioPlugin;

impl Plugin for AswAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_source::<SonarAudio>()
            .init_resource::<SonarLink>()
            .add_systems(Startup, load_clips)
            .add_systems(
                Update,
                (decode_clips, follow).chain().after(super::sonar::record).run_if(in_state(AppState::Flight)),
            );
    }
}

/// The recordings: mono samples at `RATE`.
#[derive(Default)]
struct Clips {
    ship_big: Vec<f32>,
    ship_med: Vec<f32>,
    ping: Vec<f32>,
}

/// One contact as the audio thread plays it (f32 bits): its tone (Hz, as heard), loudness,
/// blade rate (Hz), and 1.0 for a ship.
#[derive(Default)]
struct Voice {
    tone: AtomicU32,
    gain: AtomicU32,
    blade: AtomicU32,
    ship: AtomicU32,
}

/// What the audio thread plays, set from the reports.
#[derive(Default)]
struct Params {
    sea: AtomicU32,
    voices: [Voice; SOURCES],
    /// Count up with each own ping and each other sonar's ping heard; the echo's delay (s) and
    /// loudness.
    pings: AtomicU32,
    heard: AtomicU32,
    echo_delay: AtomicU32,
    echo_gain: AtomicU32,
}

fn set(a: &AtomicU32, v: f32) {
    a.store(v.to_bits(), Ordering::Relaxed);
}

fn get(a: &AtomicU32) -> f32 {
    f32::from_bits(a.load(Ordering::Relaxed))
}

#[derive(Resource, Default)]
struct SonarLink {
    params: Arc<Params>,
    player: Option<Entity>,
    /// The sensor heard, and its last report played.
    sensor: Option<u16>,
    seq: u16,
    files: Vec<Handle<AudioSource>>,
    clips: Option<Arc<Clips>>,
}

/// The sonar's sound.
#[derive(Asset, TypePath)]
pub struct SonarAudio {
    params: Arc<Params>,
    clips: Arc<Clips>,
}

impl Decodable for SonarAudio {
    type Decoder = Synth;

    fn decoder(&self) -> Synth {
        Synth::new(self.params.clone(), self.clips.clone())
    }
}

fn load_clips(server: Res<AssetServer>, mut link: ResMut<SonarLink>) {
    link.files = ["sounds/ship-big1_loop.flac", "sounds/ship-med1_loop.flac", "sounds/sonarbuoy.flac"]
        .into_iter()
        .map(|f| server.load(f))
        .collect();
}

/// A recording's samples, its channels mixed down, at `RATE` (resampled if it isn't).
fn mono(source: &AudioSource) -> Vec<f32> {
    let d = source.decoder();
    let (channels, rate) = (d.channels().get() as usize, d.sample_rate().get());
    let all: Vec<f32> = d.collect();
    let m: Vec<f32> = all.chunks(channels).map(|c| c.iter().sum::<f32>() / channels as f32).collect();
    if rate == RATE {
        return m;
    }
    let k = rate as f64 / RATE as f64;
    (0..(m.len() as f64 / k) as usize).map(|i| m[((i as f64 * k) as usize).min(m.len() - 1)]).collect()
}

/// Decodes the recordings once they have loaded.
fn decode_clips(mut link: ResMut<SonarLink>, sources: Res<Assets<AudioSource>>) {
    if link.clips.is_some() || link.files.is_empty() {
        return;
    }
    let got: Vec<&AudioSource> = link.files.iter().filter_map(|h| sources.get(h)).collect();
    if got.len() != link.files.len() {
        return;
    }
    let clips = Clips { ship_big: mono(got[0]), ship_med: mono(got[1]), ping: mono(got[2]) };
    link.clips = Some(Arc::new(clips));
}

/// Plays the sensor in the headset.
fn follow(
    mut commands: Commands,
    ui: Res<AswUi>,
    manning: Option<Res<crate::station::Manning>>,
    settings: Res<crate::settings::Settings>,
    mut link: ResMut<SonarLink>,
    mut sources: ResMut<Assets<SonarAudio>>,
) {
    let on = manning.as_deref().is_some_and(|m| m.asw()) && ui.audio.is_some_and(|id| ui.falls.contains_key(&id));
    let Some(clips) = link.clips.clone().filter(|_| on) else {
        if let Some(e) = link.player.take() {
            commands.entity(e).despawn();
        }
        return;
    };
    let Some(id) = ui.audio else { return };
    let Some(w) = ui.falls.get(&id) else { return };
    // Another sensor: its own reports from now on (no stale ping carried over).
    if link.sensor != Some(id) {
        link.sensor = Some(id);
        link.seq = w.seq;
    }
    let cursor = ui.cursor.get(&id).copied();
    let p = &link.params;
    let s = &w.last;
    set(&p.sea, SEA * (1.0 + (s.noise as f32 - 45.0).max(0.0) * 0.05));
    let mut k = 0;
    for l in &s.lines {
        let Some(hz) = cursor else { break };
        // Only the contacts whose line the cursor is on.
        let off = l.freqs.iter().map(|&f| (f as f32 - hz).abs()).fold(f32::MAX, f32::min);
        if off > CURSOR_BAND * CURSOR_EDGE || k >= SOURCES {
            continue;
        }
        let g = ((l.snr as f32 + 6.0) / 30.0).clamp(0.0, 0.7) * (-(off / CURSOR_BAND).powi(2)).exp();
        let v = &p.voices[k];
        set(&v.tone, l.freqs[0] as f32 * TRANSPOSE);
        set(&v.gain, g);
        set(&v.blade, l.freqs[1] as f32);
        set(&v.ship, if !l.ship { 0.0 } else if l.freqs[1] < BIG_BLADE { 2.0 } else { 1.0 });
        k += 1;
    }
    for v in &p.voices[k..] {
        set(&v.gain, 0.0);
    }
    if w.seq != link.seq {
        if s.ping {
            let echo = s.lines.iter().find_map(|l| l.range.map(|r| (r, l.snr)));
            set(&p.echo_delay, echo.map_or(-1.0, |(r, _)| 2.0 * r as f32 / SOUND_SPEED));
            set(&p.echo_gain, echo.map_or(0.0, |(_, snr)| ((snr as f32) / 40.0).clamp(0.05, 0.5)));
            p.pings.fetch_add(1, Ordering::Relaxed);
        }
        if s.heard {
            p.heard.fetch_add(1, Ordering::Relaxed);
        }
    }
    link.seq = w.seq;
    if link.player.is_none() {
        let handle = sources.add(SonarAudio { params: link.params.clone(), clips });
        let volume = settings.master_volume * settings.effects_volume * 0.8;
        let e = commands.spawn((AudioPlayer(handle), PlaybackSettings::LOOP.with_volume(Volume::Linear(volume)))).id();
        link.player = Some(e);
    }
}

/// A contact's state on the audio thread: oscillator phases, the screw's beat, the recording's
/// position.
#[derive(Default, Clone, Copy)]
struct VoiceState {
    phase: [f32; 2],
    beat: f32,
    lp: f32,
    pos: f64,
}

/// The synthesiser on the audio thread.
pub struct Synth {
    p: Arc<Params>,
    clips: Arc<Clips>,
    voices: [VoiceState; SOURCES],
    brown: f32,
    rng: u32,
    pings: u32,
    heard: u32,
    /// Seconds since the last own ping (None: none ringing), the echo's delay and loudness.
    ping_t: Option<f32>,
    echo: (f32, f32),
    /// Where another sonar's ping recording is (None: not playing).
    heard_pos: Option<usize>,
}

impl Synth {
    fn new(p: Arc<Params>, clips: Arc<Clips>) -> Self {
        let pings = p.pings.load(Ordering::Relaxed);
        let heard = p.heard.load(Ordering::Relaxed);
        Synth {
            p,
            clips,
            voices: [VoiceState::default(); SOURCES],
            brown: 0.0,
            rng: 0x2545_F491,
            pings,
            heard,
            ping_t: None,
            echo: (-1.0, 0.0),
            heard_pos: None,
        }
    }

    fn white(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    /// A recording looped, read at `speed` from `pos` (interpolated).
    fn looped(clip: &[f32], pos: &mut f64, speed: f64) -> f32 {
        if clip.is_empty() {
            return 0.0;
        }
        let n = clip.len();
        let i = *pos as usize % n;
        let t = (*pos - pos.floor()) as f32;
        let (a, b) = (clip[i], clip[(i + 1) % n]);
        *pos = (*pos + speed) % n as f64;
        a + (b - a) * t
    }

    /// One contact's sample.
    fn voice(&mut self, k: usize, dt: f32) -> f32 {
        let v = &self.p.voices[k];
        let g = get(&v.gain);
        if g <= 0.0 {
            return 0.0;
        }
        let (tone, blade, ship) = (get(&v.tone), get(&v.blade), get(&v.ship));
        let w = self.white();
        let tau = std::f32::consts::TAU;
        let clips = self.clips.clone();
        let st = &mut self.voices[k];
        st.phase[0] = (st.phase[0] + tone * dt).fract();
        st.phase[1] = (st.phase[1] + 2.0 * tone * dt).fract();
        let hum = (st.phase[0] * tau).sin() * 0.5 + (st.phase[1] * tau).sin() * 0.2;
        if ship > 0.5 {
            // The screws as recorded (the big ship's or a smaller one's), at its blade rate, the
            // machinery's hum under them.
            let clip = if ship > 1.5 { &clips.ship_big } else { &clips.ship_med };
            let speed = (blade / RECORDED_BLADE).clamp(0.6, 1.6) as f64;
            let screws = Self::looped(clip, &mut st.pos, speed);
            g * (screws * 4.0 + hum * 0.25)
        } else {
            // The machinery's hum, a slow soft beat of the screw.
            st.beat = (st.beat + blade / 6.0 * dt).fract();
            let beat = 0.75 + 0.25 * (st.beat * tau).sin();
            st.lp += (w - st.lp) * 0.02;
            g * beat * (hum + st.lp * 0.6)
        }
    }
}

impl Iterator for Synth {
    type Item = bevy::audio::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let dt = 1.0 / RATE as f32;
        // The sea: brown noise (integrated white noise, leaking back to zero).
        let w = self.white();
        self.brown = (self.brown * 0.998 + w * 0.02).clamp(-1.0, 1.0);
        let mut y = self.brown * get(&self.p.sea) * 3.0;
        for k in 0..SOURCES {
            y += self.voice(k, dt);
        }
        // Another sonar's ping heard: its recording from the start.
        let heard = self.p.heard.load(Ordering::Relaxed);
        if heard != self.heard {
            self.heard = heard;
            self.heard_pos = Some(0);
        }
        if let Some(i) = self.heard_pos {
            match self.clips.ping.get(i) {
                Some(&v) => {
                    y += v * 0.8;
                    self.heard_pos = Some(i + 1);
                }
                None => self.heard_pos = None,
            }
        }
        // An own ping, and its echo.
        let pings = self.p.pings.load(Ordering::Relaxed);
        if pings != self.pings {
            self.pings = pings;
            self.ping_t = Some(0.0);
            self.echo = (get(&self.p.echo_delay), get(&self.p.echo_gain));
        }
        if let Some(t) = self.ping_t.as_mut() {
            *t += dt;
            let tone = |s: f32| {
                if (0.0..PING_SECS).contains(&s) {
                    (s * PING_HZ * std::f32::consts::TAU).sin() * (1.0 - s / PING_SECS)
                } else {
                    0.0
                }
            };
            y += tone(*t) * 0.6;
            if self.echo.0 > 0.0 {
                y += tone(*t - self.echo.0) * self.echo.1;
            }
            if *t > PING_SECS + self.echo.0.max(0.0) + 0.5 {
                self.ping_t = None;
            }
        }
        Some((y * 0.8).tanh())
    }
}

impl Source for Synth {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        NonZero::new(1).expect("mono")
    }

    fn sample_rate(&self) -> SampleRate {
        NonZero::new(RATE).expect("rate")
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rms(v: &[f32]) -> f32 {
        (v.iter().map(|x| x * x).sum::<f32>() / v.len() as f32).sqrt()
    }

    /// Off every line only the faint sea; a ship's line brings up its screws (the recording),
    /// a submarine's its hum; another sonar's ping plays its recording.
    #[test]
    fn the_cursor_picks_what_is_heard() {
        let screws: Vec<f32> = (0..RATE as usize).map(|i| (i as f32 * 0.37).sin() * 0.1).collect();
        let clips = Arc::new(Clips {
            ship_big: screws.clone(),
            ship_med: screws,
            ping: vec![0.5; RATE as usize / 2],
        });
        let play = |tone: f32, ship: f32, gain: f32| {
            let p = Arc::new(Params::default());
            set(&p.sea, SEA);
            set(&p.voices[0].tone, tone);
            set(&p.voices[0].gain, gain);
            set(&p.voices[0].blade, 12.0);
            set(&p.voices[0].ship, ship);
            let s = Synth::new(p, clips.clone());
            let v: Vec<f32> = s.take(RATE as usize).collect();
            rms(&v)
        };
        let sea = play(0.0, 0.0, 0.0);
        assert!(sea > 0.001 && sea < 0.05, "the sea alone faint: {sea}");
        assert!(play(800.0, 0.0, 0.5) > 4.0 * sea, "a submarine's hum");
        assert!(play(960.0, 2.0, 0.5) > 4.0 * sea, "a ship's screws");
        // Another sonar's ping.
        let p = Arc::new(Params::default());
        let mut s = Synth::new(p.clone(), clips.clone());
        let quiet: Vec<f32> = (&mut s).take(1000).collect();
        p.heard.fetch_add(1, Ordering::Relaxed);
        let loud: Vec<f32> = s.take(1000).collect();
        assert!(rms(&loud) > 10.0 * rms(&quiet).max(1e-4));
    }
}
