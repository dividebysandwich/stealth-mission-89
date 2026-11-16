//! The sensor operator's headset: the selected sensor's sound, made live on the audio thread
//! from its reports. The sea's noise (band-limited, louder with the sea state), a heard
//! submarine's tonal lines (its machinery's tone and harmonic, raised four octaves into hearing
//! as the operator's audio processor does) at its signal, the throb of its screw at the blade
//! rate in the noise, and an active sensor's ping (a 3.5 kHz tone) with its echo coming back
//! after the range's two-way travel time.

use std::num::NonZero;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use bevy::audio::{AddAudioSource, ChannelCount, Decodable, SampleRate, Source, Volume};
use bevy::prelude::*;

use super::AswUi;
use crate::settings::AppState;

const RATE: u32 = 22_050;
/// The tonals are heard this many times higher (four octaves).
const TRANSPOSE: f32 = 16.0;
const PING_HZ: f32 = 3500.0;
const PING_SECS: f32 = 0.35;
/// Sound in sea water (m/s).
const SOUND_SPEED: f32 = 1500.0;

pub struct AswAudioPlugin;

impl Plugin for AswAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_source::<SonarAudio>()
            .init_resource::<SonarLink>()
            .add_systems(Update, follow.after(super::sonar::record).run_if(in_state(AppState::Flight)));
    }
}

/// What the audio thread plays, set from the reports (f32 bits).
#[derive(Default)]
struct Params {
    noise: AtomicU32,
    tones: [AtomicU32; 2],
    tone_gain: AtomicU32,
    blade_hz: AtomicU32,
    /// Counts up with each ping; the echo's delay (s) and loudness.
    pings: AtomicU32,
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
    seq: u16,
}

/// The sonar's sound.
#[derive(Asset, TypePath)]
pub struct SonarAudio {
    params: Arc<Params>,
}

impl Decodable for SonarAudio {
    type Decoder = Synth;

    fn decoder(&self) -> Synth {
        Synth { p: self.params.clone(), ..Synth::new() }
    }
}

/// Plays the selected sensor while AUDIO is on.
fn follow(
    mut commands: Commands,
    ui: Res<AswUi>,
    manning: Option<Res<crate::station::Manning>>,
    settings: Res<crate::settings::Settings>,
    mut link: ResMut<SonarLink>,
    mut sources: ResMut<Assets<SonarAudio>>,
) {
    let on = manning.as_deref().is_some_and(|m| m.asw()) && ui.audio.is_some_and(|id| ui.falls.contains_key(&id));
    if !on {
        if let Some(e) = link.player.take() {
            commands.entity(e).despawn();
        }
        return;
    }
    let Some(w) = ui.audio.and_then(|id| ui.falls.get(&id)) else { return };
    let p = &link.params;
    let s = &w.last;
    set(&p.noise, 0.05 + (s.noise as f32 - 45.0).max(0.0) * 0.01);
    match s.lines.first() {
        Some(l) => {
            let g = ((l.snr as f32 + 6.0) / 30.0).clamp(0.0, 0.5);
            set(&p.tones[0], l.freqs[0] as f32 * TRANSPOSE);
            set(&p.tones[1], l.freqs[0] as f32 * 2.0 * TRANSPOSE);
            set(&p.tone_gain, g);
            set(&p.blade_hz, l.freqs[1] as f32 / 4.0);
        }
        None => {
            set(&p.tone_gain, 0.0);
            set(&p.blade_hz, 0.0);
        }
    }
    if s.ping && w.seq != link.seq {
        let echo = s.lines.iter().find_map(|l| l.range.map(|r| (r, l.snr)));
        set(&p.echo_delay, echo.map_or(-1.0, |(r, _)| 2.0 * r as f32 / SOUND_SPEED));
        set(&p.echo_gain, echo.map_or(0.0, |(_, snr)| ((snr as f32) / 40.0).clamp(0.05, 0.5)));
        p.pings.fetch_add(1, Ordering::Relaxed);
    }
    link.seq = w.seq;
    if link.player.is_none() {
        let handle = sources.add(SonarAudio { params: link.params.clone() });
        let volume = settings.master_volume * settings.effects_volume * 0.7;
        let e = commands.spawn((AudioPlayer(handle), PlaybackSettings::LOOP.with_volume(Volume::Linear(volume)))).id();
        link.player = Some(e);
    }
}

/// The synthesiser on the audio thread.
pub struct Synth {
    p: Arc<Params>,
    t: f32,
    phase: [f32; 3],
    lp: [f32; 2],
    rng: u32,
    pings: u32,
    /// Seconds since the last ping (None: none ringing), the echo's delay and loudness.
    ping_t: Option<f32>,
    echo: (f32, f32),
}

impl Synth {
    fn new() -> Self {
        Synth {
            p: Arc::new(Params::default()),
            t: 0.0,
            phase: [0.0; 3],
            lp: [0.0; 2],
            rng: 0x2545_F491,
            pings: 0,
            ping_t: None,
            echo: (-1.0, 0.0),
        }
    }

    fn white(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

impl Iterator for Synth {
    type Item = bevy::audio::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let dt = 1.0 / RATE as f32;
        self.t += dt;
        // The sea: white noise through two one-pole low-passes (a dull hiss).
        let w = self.white();
        self.lp[0] += (w - self.lp[0]) * 0.08;
        self.lp[1] += (self.lp[0] - self.lp[1]) * 0.15;
        let blade = get(&self.p.blade_hz);
        self.phase[2] = (self.phase[2] + blade * dt).fract();
        let throb = if blade > 0.0 { 1.0 + 0.6 * (self.phase[2] * std::f32::consts::TAU).sin() } else { 1.0 };
        let mut y = self.lp[1] * get(&self.p.noise) * 6.0 * throb;
        // The tonals.
        let g = get(&self.p.tone_gain);
        if g > 0.0 {
            for k in 0..2 {
                let f = get(&self.p.tones[k]);
                self.phase[k] = (self.phase[k] + f * dt).fract();
                y += (self.phase[k] * std::f32::consts::TAU).sin() * g * if k == 0 { 0.35 } else { 0.15 };
            }
        }
        // A new ping, and its echo.
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
        Some(y.clamp(-1.0, 1.0))
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
