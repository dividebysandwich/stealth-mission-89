//! The radio's loudspeaker: the other players' transmissions mixed on the audio thread, with
//! a radio's sound (a narrow band, a little clipping, a hiss while the channel is open) and the
//! squelch's click at the start and end of each transmission.
//!
//! Each talker's voice waits in a jitter buffer until about 100 ms have come; a gap of lost
//! packets is filled with silence, and a buffer grown past `MAX_BUFFER` drops its oldest
//! samples (the delay would only grow).

use std::collections::{HashMap, VecDeque};
use std::num::NonZero;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bevy::audio::{ChannelCount, Decodable, SampleRate, Source};
use bevy::prelude::*;
use stealth_net::voice::{FRAME, FRAMES_PER_PACKET, RATE, VoicePacket};

/// Samples (at `RATE`) a talker's buffer fills before it plays, holds at most, and the
/// silence after which a transmission without its end packet is over.
const START_BUFFER: usize = RATE as usize / 10;
const MAX_BUFFER: usize = RATE as usize * 2 / 5;
const SILENT_END: usize = RATE as usize * 3 / 10;
/// Lost packets filled with silence at most (a longer gap: the voice just goes on).
const MAX_GAP: u16 = 3;
/// Samples of the squelch click and its level, the hiss's level while a channel is open.
const CLICK: usize = RATE as usize / 25;
const CLICK_LEVEL: f32 = 0.25;
const HISS_LEVEL: f32 = 0.012;
/// Voice gain before the clipping.
const DRIVE: f32 = 1.6;

/// One talker's transmission.
#[derive(Default)]
struct Talker {
    samples: VecDeque<f32>,
    /// Playing (the start buffer has filled), its end packet came.
    playing: bool,
    ended: bool,
    /// The last packet's number.
    seq: Option<u16>,
    /// Samples played out of an empty buffer in a row.
    starved: usize,
}

/// The talkers' voices, shared by the game (which adds packets) and the audio thread.
#[derive(Default)]
struct Mix {
    talkers: HashMap<u8, Talker>,
    /// Squelch clicks to play (samples left of each).
    clicks: Vec<usize>,
}

/// The radio's shared state.
#[derive(Clone, Default)]
pub struct RadioMix(Arc<Mutex<Mix>>);

impl RadioMix {
    /// A packet of `from`'s transmission arrived.
    pub fn add(&self, from: u8, packet: &VoicePacket) {
        let voice = stealth_net::voice::decode(packet);
        let Ok(mut mix) = self.0.lock() else { return };
        let mix = &mut *mix;
        let t = mix.talkers.entry(from).or_default();
        if t.ended && t.samples.is_empty() && !t.playing {
            // A new transmission (its numbers go on from the last one's, or start again
            // after a reconnection).
            *t = Talker::default();
        }
        if let Some(last) = t.seq {
            let gap = packet.seq.wrapping_sub(last).wrapping_sub(1);
            // Late (or repeated) packets are of no use.
            if gap > u16::MAX / 2 {
                return;
            }
            if t.playing || !t.samples.is_empty() {
                let lost = gap.min(MAX_GAP) as usize;
                t.samples.extend(std::iter::repeat_n(0.0, lost * FRAME * FRAMES_PER_PACKET));
            }
        }
        t.seq = Some(packet.seq);
        t.ended |= packet.end;
        t.samples.extend(voice);
        let extra = t.samples.len().saturating_sub(MAX_BUFFER);
        t.samples.drain(..extra);
    }

    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    /// A whole transmission of `from`'s at `RATE` (the spoken ATC): played at once, after the
    /// one still going if any; no buffer limit.
    pub fn push_pcm(&self, from: u8, samples: &[f32]) {
        let Ok(mut mix) = self.0.lock() else { return };
        let t = mix.talkers.entry(from).or_default();
        if t.ended && t.samples.is_empty() && !t.playing {
            *t = Talker::default();
        }
        t.samples.extend(samples);
        t.ended = true;
    }

    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    /// `from` is on the radio or about to be.
    pub fn playing(&self, from: u8) -> bool {
        self.0
            .lock()
            .is_ok_and(|m| m.talkers.get(&from).is_some_and(|t| t.playing || !t.samples.is_empty()))
    }

    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    /// Cuts `from`'s transmission short.
    pub fn silence(&self, from: u8) {
        if let Ok(mut m) = self.0.lock()
            && let Some(t) = m.talkers.get_mut(&from)
        {
            t.samples.clear();
            t.ended = true;
        }
    }

    /// Talkers heard right now.
    pub fn talking(&self) -> Vec<u8> {
        self.0
            .lock()
            .map(|m| m.talkers.iter().filter(|(_, t)| t.playing).map(|(&k, _)| k).collect())
            .unwrap_or_default()
    }

    /// Forgets everybody (the game left).
    pub fn clear(&self) {
        if let Ok(mut m) = self.0.lock() {
            *m = Mix::default();
        }
    }

    /// The next sample of the mix (before the radio's filter).
    fn next(&self, rng: &mut u32) -> (f32, bool) {
        let Ok(mut mix) = self.0.lock() else { return (0.0, false) };
        let mix = &mut *mix;
        let mut v = 0.0;
        let mut open = false;
        let mut done = Vec::new();
        let mut clicks = 0;
        for (&k, t) in &mut mix.talkers {
            if !t.playing {
                if t.samples.len() >= START_BUFFER || (t.ended && !t.samples.is_empty()) {
                    t.playing = true;
                    clicks += 1;
                } else {
                    continue;
                }
            }
            open = true;
            match t.samples.pop_front() {
                Some(s) => {
                    v += s;
                    t.starved = 0;
                }
                None => t.starved += 1,
            }
            if (t.ended && t.samples.is_empty()) || t.starved > SILENT_END {
                done.push(k);
                clicks += 1;
            }
        }
        for k in done {
            if let Some(t) = mix.talkers.get_mut(&k) {
                t.playing = false;
                t.samples.clear();
                if !t.ended {
                    // Its end packet was lost: the next packet starts a new transmission.
                    t.ended = true;
                }
            }
        }
        mix.clicks.extend(std::iter::repeat_n(CLICK, clicks));
        let mut click = 0.0;
        for c in &mut mix.clicks {
            *c -= 1;
            click += noise(rng) * CLICK_LEVEL * (*c as f32 / CLICK as f32);
        }
        mix.clicks.retain(|&c| c > 0);
        (v + click, open)
    }
}

/// White noise -1..1 (xorshift).
fn noise(state: &mut u32) -> f32 {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    (*state as f32 / u32::MAX as f32) * 2.0 - 1.0
}

/// The radio as a sound to play: endless, silent when nobody talks.
#[derive(Asset, TypePath)]
pub struct Radio(pub RadioMix);

impl Decodable for Radio {
    type Decoder = Speaker;

    fn decoder(&self) -> Speaker {
        Speaker { mix: self.0.clone(), rng: 0x1234_5678, hp: [0.0; 2], lp: [0.0; 2] }
    }
}

/// The radio's sound: the mix through a band-pass (two one-pole high-passes at 300 Hz and
/// low-passes at 3 kHz) and a soft clip, the hiss under it while a channel is open.
pub struct Speaker {
    mix: RadioMix,
    rng: u32,
    hp: [f32; 2],
    lp: [f32; 2],
}

impl Speaker {
    fn filter(&mut self, x: f32) -> f32 {
        let hp_k = (-std::f32::consts::TAU * 300.0 / RATE as f32).exp();
        let lp_k = (-std::f32::consts::TAU * 3000.0 / RATE as f32).exp();
        // High-pass: the input minus its low-passed self.
        let mut v = x;
        for s in &mut self.hp {
            *s = v + (*s - v) * hp_k;
            v -= *s;
        }
        for s in &mut self.lp {
            *s = v + (*s - v) * lp_k;
            v = *s;
        }
        v
    }
}

impl Iterator for Speaker {
    type Item = bevy::audio::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let (v, open) = self.mix.next(&mut self.rng);
        let hiss = if open { noise(&mut self.rng) * HISS_LEVEL } else { 0.0 };
        let v = self.filter(v * DRIVE + hiss);
        Some(v.tanh())
    }
}

impl Source for Speaker {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        NonZero::new(1).expect("one")
    }

    fn sample_rate(&self) -> SampleRate {
        NonZero::new(RATE).expect("8 kHz")
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stealth_net::voice::VoiceEncoder;

    fn packets(n: usize) -> Vec<VoicePacket> {
        let tone: Vec<f32> = (0..n * 480 * 6).map(|i| 0.3 * (i as f32 * 0.07).sin()).collect();
        let mut enc = VoiceEncoder::new(48_000);
        let mut p = enc.push(&tone);
        p.push(enc.finish());
        p
    }

    /// Samples of the mix until it falls silent: (voice samples, clicks heard).
    fn play(mix: &RadioMix, max: usize) -> (usize, usize) {
        let mut rng = 1;
        let (mut voiced, mut was_open, mut edges) = (0, false, 0);
        for _ in 0..max {
            let (v, open) = mix.next(&mut rng);
            if open {
                voiced += (v.abs() > 0.0) as usize;
            }
            edges += (open != was_open) as usize;
            was_open = open;
        }
        (voiced, edges)
    }

    #[test]
    fn a_transmission_plays_after_its_buffer_and_closes_at_its_end() {
        let mix = RadioMix::default();
        let p = packets(4);
        // One packet (60 ms) is under the start buffer: nothing yet.
        mix.add(2, &p[0]);
        assert_eq!(play(&mix, 100).1, 0);
        assert!(mix.talking().is_empty());
        for q in &p[1..] {
            mix.add(2, q);
        }
        let (voiced, edges) = play(&mix, 20_000);
        assert!(voiced > 1500, "{voiced}");
        assert_eq!(edges, 2, "opens and closes once");
        assert!(mix.talking().is_empty());
        // The next transmission starts afresh.
        for q in &packets(2) {
            mix.add(2, q);
        }
        assert_eq!(play(&mix, 20_000).1, 2);
    }

    #[test]
    fn lost_packets_become_silence_and_late_ones_are_dropped() {
        let mix = RadioMix::default();
        let p = packets(6);
        mix.add(1, &p[0]);
        mix.add(1, &p[1]);
        mix.add(1, &p[3]);
        let len = |m: &RadioMix| m.0.lock().unwrap().talkers[&1].samples.len();
        assert_eq!(len(&mix), 4 * FRAME * FRAMES_PER_PACKET);
        mix.add(1, &p[2]);
        assert_eq!(len(&mix), 4 * FRAME * FRAMES_PER_PACKET, "late packet dropped");
    }

    #[test]
    fn the_speaker_band_limits() {
        let mix = RadioMix::default();
        let mut s = Radio(mix).decoder();
        let level = |s: &mut Speaker, hz: f32| {
            let mut peak: f32 = 0.0;
            for i in 0..8000 {
                let x = 0.1 * (std::f32::consts::TAU * hz * i as f32 / RATE as f32).sin();
                let y = s.filter(x);
                if i > 2000 {
                    peak = peak.max(y.abs());
                }
            }
            peak / 0.1
        };
        assert!(level(&mut s, 1000.0) > 0.6);
        assert!(level(&mut s, 60.0) < 0.1);
    }

    /// A spoken line plays at once and whole (no buffer limit), clicks at both ends; one
    /// after it on the same talker follows on; cut short, it closes.
    #[test]
    fn spoken_lines_play_whole() {
        let mix = RadioMix::default();
        let line = vec![0.3; RATE as usize * 2];
        mix.push_pcm(254, &line);
        assert!(mix.playing(254));
        let (voiced, edges) = play(&mix, RATE as usize * 3);
        assert_eq!(voiced, line.len());
        assert_eq!(edges, 2);
        assert!(!mix.playing(254));
        mix.push_pcm(254, &line);
        play(&mix, 100);
        mix.silence(254);
        let (voiced, _) = play(&mix, RATE as usize);
        assert!(voiced <= 1, "{voiced}");
        assert!(!mix.playing(254));
    }
}
