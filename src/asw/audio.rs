//! The sensor operator's headset: the selected sensor's sound, made live on the audio thread
//! from its reports.
//!
//! The sea: band-limited noise, louder with the sea state. Each contact the sensor hears (up to
//! four, at their signal over the noise):
//! - a submarine: the hum of its machinery's tone and harmonic (raised four octaves into
//!   hearing, as the operator's audio processor does), a slow, soft beat of its screw at the
//!   blade rate in the noise;
//! - a surface ship: louder and rougher, the thrash of its screws beating at the shaft rate,
//!   their cavitation crackling, its diesels' harmonics.
//!
//! The listening cursor on the waterfall narrows the band heard: the sea quieter, a contact's
//! tone loud when the cursor sits on its line and fading off it. An active sensor's ping (a
//! 3.5 kHz tone) and its echo after the range's two-way travel time sound over it.

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
/// Contacts heard at once.
const SOURCES: usize = 4;
/// The cursor's band: a tone this far off it (Hz) is down to 1/e.
const CURSOR_BAND: f32 = 6.0;

pub struct AswAudioPlugin;

impl Plugin for AswAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_source::<SonarAudio>()
            .init_resource::<SonarLink>()
            .add_systems(Update, follow.after(super::sonar::record).run_if(in_state(AppState::Flight)));
    }
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
    noise: AtomicU32,
    voices: [Voice; SOURCES],
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
        Synth::new(self.params.clone())
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
    let Some(id) = ui.audio else { return };
    let Some(w) = ui.falls.get(&id) else { return };
    let cursor = ui.cursor.get(&id).copied();
    let p = &link.params;
    let s = &w.last;
    let sea = 0.05 + (s.noise as f32 - 45.0).max(0.0) * 0.01;
    set(&p.noise, if cursor.is_some() { sea * 0.4 } else { sea });
    for (k, v) in p.voices.iter().enumerate() {
        match s.lines.get(k) {
            Some(l) => {
                let mut g = ((l.snr as f32 + 6.0) / 30.0).clamp(0.0, 0.6);
                // The cursor's band: the line under it loud, the others faint.
                if let Some(hz) = cursor {
                    let off = l.freqs.iter().map(|&f| (f as f32 - hz).abs()).fold(f32::MAX, f32::min);
                    g *= 0.15 + 1.6 * (-(off / CURSOR_BAND).powi(2)).exp();
                }
                set(&v.tone, l.freqs[0] as f32 * TRANSPOSE);
                set(&v.gain, g);
                set(&v.blade, l.freqs[1] as f32);
                set(&v.ship, if l.ship { 1.0 } else { 0.0 });
            }
            None => set(&v.gain, 0.0),
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

/// A contact's state on the audio thread: oscillator phases, its own noise filter, the crackle's
/// envelope.
#[derive(Default, Clone, Copy)]
struct VoiceState {
    phase: [f32; 3],
    beat: f32,
    lp: f32,
    crackle: f32,
}

/// The synthesiser on the audio thread.
pub struct Synth {
    p: Arc<Params>,
    voices: [VoiceState; SOURCES],
    sea: [f32; 2],
    rng: u32,
    pings: u32,
    /// Seconds since the last ping (None: none ringing), the echo's delay and loudness.
    ping_t: Option<f32>,
    echo: (f32, f32),
}

impl Synth {
    fn new(p: Arc<Params>) -> Self {
        Synth {
            p,
            voices: [VoiceState::default(); SOURCES],
            sea: [0.0; 2],
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

    /// One contact's sample.
    fn voice(&mut self, k: usize, dt: f32) -> f32 {
        let v = &self.p.voices[k];
        let g = get(&v.gain);
        if g <= 0.0 {
            return 0.0;
        }
        let (tone, blade, ship) = (get(&v.tone), get(&v.blade), get(&v.ship) > 0.5);
        let w = self.white();
        let c = self.white();
        let st = &mut self.voices[k];
        let tau = std::f32::consts::TAU;
        st.phase[0] = (st.phase[0] + tone * dt).fract();
        st.phase[1] = (st.phase[1] + 2.0 * tone * dt).fract();
        if ship {
            // The shaft's beat (a four-bladed screw: a quarter of the blade rate), deep and
            // hard; the thrash of the screws in band-limited noise under it; cavitation's
            // crackle; the diesels' harmonics.
            st.beat = (st.beat + blade / 4.0 * dt).fract();
            let pulse = (1.0 - st.beat).powi(6);
            st.lp += (w - st.lp) * 0.25;
            if c > 0.995 {
                st.crackle = 1.0;
            }
            st.crackle *= 0.985;
            st.phase[2] = (st.phase[2] + 3.0 * tone * dt).fract();
            let engine = 0.25 * (st.phase[0] * tau).sin()
                + 0.18 * (st.phase[1] * tau).sin()
                + 0.12 * (st.phase[2] * tau).sin();
            g * (st.lp * (0.25 + 2.6 * pulse) + st.crackle * c * 0.4 + engine * 0.5)
        } else {
            // The machinery's hum, a slow soft beat of the screw.
            st.beat = (st.beat + blade / 6.0 * dt).fract();
            let beat = 0.75 + 0.25 * (st.beat * tau).sin();
            st.lp += (w - st.lp) * 0.04;
            let hum = 0.55 * (st.phase[0] * tau).sin() + 0.25 * (st.phase[1] * tau).sin();
            g * beat * (hum + st.lp * 0.8)
        }
    }
}

impl Iterator for Synth {
    type Item = bevy::audio::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let dt = 1.0 / RATE as f32;
        // The sea: white noise through two one-pole low-passes (a dull hiss).
        let w = self.white();
        self.sea[0] += (w - self.sea[0]) * 0.08;
        self.sea[1] += (self.sea[0] - self.sea[1]) * 0.15;
        let mut y = self.sea[1] * get(&self.p.noise) * 6.0;
        for k in 0..SOURCES {
            y += self.voice(k, dt);
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

    /// A ship and a submarine sound different: the ship louder, beating hard at its shaft rate;
    /// the cursor on the submarine's line brings its hum up over the sea.
    #[test]
    fn ships_and_submarines_sound_apart() {
        let level = |ship: bool| {
            let p = Arc::new(Params::default());
            set(&p.noise, 0.0);
            set(&p.voices[0].tone, 50.0 * TRANSPOSE);
            set(&p.voices[0].gain, 0.4);
            set(&p.voices[0].blade, 12.0);
            set(&p.voices[0].ship, if ship { 1.0 } else { 0.0 });
            let s = Synth::new(p);
            let v: Vec<f32> = s.take(RATE as usize * 2).collect();
            // Loudness over windows of 1/20 s: its spread is the beat.
            let rms: Vec<f32> = v
                .chunks(RATE as usize / 20)
                .map(|c| (c.iter().map(|x| x * x).sum::<f32>() / c.len() as f32).sqrt())
                .collect();
            let mean = rms.iter().sum::<f32>() / rms.len() as f32;
            let spread = rms.iter().map(|r| (r - mean).abs()).sum::<f32>() / rms.len() as f32;
            (mean, spread / mean)
        };
        let (ship, ship_beat) = level(true);
        let (sub, sub_beat) = level(false);
        assert!(ship_beat > sub_beat * 1.5, "beats: ship {ship_beat:.3}, sub {sub_beat:.3}");
        assert!(ship > 0.0 && sub > 0.0);
    }
}
