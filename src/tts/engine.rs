//! The speech thread: lines in, the voice through the radio's loudspeaker out. It loads a voice
//! the first time it speaks with it (two kept: the controllers' and the pilot's), says one line
//! at a time and waits for the radio to fall silent before the next, so the calls never talk
//! over each other.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bevy::log::{info, warn};
use stealth_net::voice::Downsampler;

use super::model::Synth;
use super::phonemes::Phonemes;
use super::{ATC_TALKER, PILOT_TALKER, voice};
use crate::voice::RadioMix;

/// A line waiting longer than this is not said (the moment has passed).
const STALE: Duration = Duration::from_secs(8);
/// Voices kept loaded.
const LOADED: usize = 2;
/// The voice's loudest sample, before the radio's drive and clipping.
const PEAK: f32 = 0.6;

/// One line to say.
pub struct Line {
    pub talker: u8,
    /// The voice's name (its files' stem).
    pub voice: String,
    pub speaker: u32,
    /// Speed (1.0: the voice's own).
    pub tempo: f32,
    pub text: String,
}

enum Msg {
    Say(Line, Instant, u64),
    Unload(String, Sender<()>),
}

#[derive(Default)]
struct Shared {
    /// Lines sent and not yet in the loudspeaker (or dropped).
    pending: AtomicUsize,
    /// Lines sent before the last `stop` are dropped.
    generation: AtomicU64,
    /// The last failure (a voice that would not load).
    error: Mutex<Option<String>>,
}

pub struct Engine {
    tx: Sender<Msg>,
    shared: Arc<Shared>,
    mix: RadioMix,
}

impl Engine {
    pub fn start(mix: RadioMix) -> Self {
        let (tx, rx) = channel();
        let shared = Arc::new(Shared::default());
        let (s, m) = (shared.clone(), mix.clone());
        if let Err(e) = std::thread::Builder::new().name("tts".into()).spawn(move || run(rx, s, m))
        {
            warn!("tts: no thread: {e}");
        }
        Self { tx, shared, mix }
    }

    pub fn say(&self, line: Line) {
        self.shared.pending.fetch_add(1, Ordering::SeqCst);
        let generation = self.shared.generation.load(Ordering::SeqCst);
        if self.tx.send(Msg::Say(line, Instant::now(), generation)).is_err() {
            self.shared.pending.fetch_sub(1, Ordering::SeqCst);
        }
    }

    /// Drops the lines waiting and cuts the one being said.
    pub fn stop(&self) {
        self.shared.generation.fetch_add(1, Ordering::SeqCst);
        self.mix.silence(ATC_TALKER);
        self.mix.silence(PILOT_TALKER);
    }

    /// A line is waiting or being said.
    pub fn busy(&self) -> bool {
        self.shared.pending.load(Ordering::SeqCst) > 0
            || self.mix.playing(ATC_TALKER)
            || self.mix.playing(PILOT_TALKER)
    }

    /// Lets go of a voice's files (before they are deleted); waits for the thread a moment.
    pub fn unload(&self, name: &str) {
        let (done, wait) = channel();
        if self.tx.send(Msg::Unload(name.to_string(), done)).is_ok() {
            let _ = wait.recv_timeout(Duration::from_secs(5));
        }
    }

    pub fn error(&self) -> Option<String> {
        self.shared.error.lock().ok().and_then(|e| e.clone())
    }
}

fn run(rx: Receiver<Msg>, shared: Arc<Shared>, mix: RadioMix) {
    let mut g2p: Option<Phonemes> = None;
    // Most recently used last.
    let mut loaded: Vec<(String, Synth)> = Vec::new();
    while let Ok(msg) = rx.recv() {
        let (line, at, generation) = match msg {
            Msg::Unload(name, done) => {
                loaded.retain(|(n, _)| *n != name);
                let _ = done.send(());
                continue;
            }
            Msg::Say(line, at, generation) => (line, at, generation),
        };
        let current = || shared.generation.load(Ordering::SeqCst) == generation;
        if current() && at.elapsed() < STALE {
            match say(&line, &mut g2p, &mut loaded) {
                Ok(pcm) if !pcm.is_empty() => {
                    // The radio is one channel: the line before goes out first.
                    while current() && (mix.playing(ATC_TALKER) || mix.playing(PILOT_TALKER)) {
                        std::thread::sleep(Duration::from_millis(20));
                    }
                    if current() {
                        mix.push_pcm(line.talker, &pcm);
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    warn!("tts: {e}");
                    if let Ok(mut err) = shared.error.lock() {
                        *err = Some(e);
                    }
                }
            }
        }
        shared.pending.fetch_sub(1, Ordering::SeqCst);
    }
}

/// The line at the radio's rate.
fn say(
    line: &Line,
    g2p: &mut Option<Phonemes>,
    loaded: &mut Vec<(String, Synth)>,
) -> Result<Vec<f32>, String> {
    let k = match loaded.iter().position(|(n, _)| *n == line.voice) {
        Some(k) => k,
        None => {
            let v = voice::find(&line.voice).ok_or_else(|| format!("no voice {}", line.voice))?;
            let t = Instant::now();
            let synth = Synth::load(&v.model)?;
            info!("tts: {} loaded in {:.2} s", line.voice, t.elapsed().as_secs_f32());
            if loaded.len() >= LOADED {
                loaded.remove(0);
            }
            loaded.push((line.voice.clone(), synth));
            loaded.len() - 1
        }
    };
    let entry = loaded.remove(k);
    loaded.push(entry);
    let synth = &loaded.last().expect("just pushed").1;
    let g2p = g2p.get_or_insert_with(Phonemes::new);
    let t = Instant::now();
    let ipa = g2p.to_ipa(&line.text);
    let pcm = synth.speak(&ipa, line.speaker, line.tempo)?;
    let rate = synth.sample_rate();
    info!(
        "tts: {:.1} s of speech in {:.2} s",
        pcm.len() as f32 / rate as f32,
        t.elapsed().as_secs_f32()
    );
    let peak = pcm.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    let gain = if peak > 0.0 { PEAK / peak } else { 1.0 };
    let pcm: Vec<f32> = pcm.iter().map(|s| s * gain).collect();
    let mut out = Vec::with_capacity(pcm.len() * 8000 / rate.max(1) as usize + 1);
    Downsampler::new(rate).push(&pcm, &mut out);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tts::{catalog, download};

    /// `SM89_VOICE_DIR=<dir> cargo test real_voice -- --ignored --nocapture`: downloads Joe
    /// (or `SM89_VOICE=<id>[:speaker]`) there if not yet, says ATC lines, writes
    /// `<dir>/<id>.wav` (the radio's rate) and prints how fast it went.
    #[test]
    #[ignore]
    fn real_voice() {
        let dir =
            std::path::PathBuf::from(std::env::var("SM89_VOICE_DIR").expect("SM89_VOICE_DIR"));
        let want = std::env::var("SM89_VOICE").unwrap_or("en_US-joe-medium".into());
        let (id, speaker) = want.split_once(':').unwrap_or((&want, "0"));
        let speaker: u32 = speaker.parse().unwrap();
        let v = catalog::find(id).unwrap();
        let model = dir.join(v.files[0].name);
        if !model.exists() {
            let d = download::Download::start(v, dir.clone());
            while d.state() == download::State::Running {
                std::thread::sleep(Duration::from_millis(200));
            }
            assert_eq!(d.state(), download::State::Done);
        }
        let t = Instant::now();
        let synth = Synth::load(&model).unwrap();
        let g2p = Phonemes::new();
        println!("loaded in {:.2} s", t.elapsed().as_secs_f32());
        let mut all = Vec::new();
        for text in [
            "Gutersloh Tower, Viper 1-1, ready for departure, runway 27",
            "Viper 1-1, runway 27, wind calm, cleared for takeoff",
            "Viper 1-1, 3 miles from touchdown, slightly left of course, turn heading 265, slightly above glidepath, check wheels down, cleared to land runway 27",
            "Viper 1-1, missed approach, fly runway heading, climb and maintain 2,000",
        ] {
            let spoken = crate::tts::phrase::spoken(text);
            let ipa = g2p.to_ipa(&spoken);
            let t = Instant::now();
            let pcm = synth.speak(&ipa, speaker, crate::atc::RADIO_TEMPO).unwrap();
            let secs = pcm.len() as f32 / synth.sample_rate() as f32;
            let took = t.elapsed().as_secs_f32();
            println!(
                "{secs:.1} s of speech in {took:.2} s (RTF {:.2}): {spoken}\n  {ipa}",
                took / secs
            );
            let mut out = Vec::new();
            Downsampler::new(synth.sample_rate()).push(&pcm, &mut out);
            all.extend(out);
            all.extend(std::iter::repeat_n(0.0, 4000));
        }
        // 16-bit mono WAV at 8 kHz.
        let n = all.len() as u32 * 2;
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + n).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&[1, 0, 1, 0]);
        wav.extend_from_slice(&8000u32.to_le_bytes());
        wav.extend_from_slice(&16000u32.to_le_bytes());
        wav.extend_from_slice(&[2, 0, 16, 0]);
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&n.to_le_bytes());
        for s in all {
            wav.extend_from_slice(&((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
        }
        std::fs::write(dir.join(format!("{id}-{speaker}.wav")), wav).unwrap();
    }
}
