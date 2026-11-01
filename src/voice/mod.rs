//! Push-to-talk radio in coop: hold the talk key (CapsLock by default, or a HOTAS or VR button
//! mapped to it) and everybody in the game hears you, on one channel, through a radio's
//! loudspeaker. The voice goes as 8 kHz ADPCM (`stealth_net::voice`) through the server.
//!
//! The microphone opens the first time the key is pressed and stays open for the game; the
//! cockpit's notices panel shows who is talking, the Ops screen's player list too, where a
//! player can be muted.

mod capture;
mod playback;

use std::collections::HashSet;

use bevy::audio::AddAudioSource;
use bevy::prelude::*;
use stealth_net::voice::{VoiceEncoder, VoicePacket};

use crate::input::FlightControls;
use crate::net::NetLink;
use crate::sound::{Channel, Mixed};

pub use capture::devices;
#[cfg_attr(target_arch = "wasm32", allow(unused_imports))]
pub use playback::RadioMix;

pub struct VoicePlugin;

impl Plugin for VoicePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Voice>()
            .add_audio_source::<playback::Radio>()
            .add_systems(Startup, test_file)
            .add_systems(
                Update,
                (transmit, receive, indicators)
                    .chain()
                    .run_if(resource_exists::<NetLink>)
                    .after(crate::net::NetSystems)
                    // A test script holds the talk button there.
                    .after(crate::script::ScriptSet),
            )
            .add_systems(Update, speaker.run_if(radio_wanted))
            .add_systems(Update, leave.run_if(not(resource_exists::<NetLink>)));
    }
}

/// The radio: the microphone and the transmission going out, the voices coming in.
#[derive(Resource, Default)]
pub struct Voice {
    mic: Option<capture::Mic>,
    /// `--voice-file`: a recording (mono samples, rate) talking instead of the microphone,
    /// and where it has got to.
    file: Option<(Vec<f32>, u32, usize)>,
    encoder: Option<(u32, VoiceEncoder)>,
    /// The talk key is held (a transmission is going out).
    pub transmitting: bool,
    mix: playback::RadioMix,
    /// Players not heard (pilot ids).
    pub muted: HashSet<u8>,
    /// The players talking now (pilot ids), as the speaker plays them.
    pub talking: Vec<u8>,
    /// Why the microphone could not be opened.
    pub error: Option<String>,
}

impl Voice {
    /// The loudspeaker's mix (the spoken ATC talks into it too).
    pub fn mix(&self) -> &playback::RadioMix {
        &self.mix
    }

    /// The microphone's (or the recording's) samples since the last frame and their rate.
    fn input(&mut self, dt: f32) -> (Vec<f32>, Option<u32>) {
        if let Some((samples, rate, pos)) = &mut self.file {
            let n = ((*rate as f32 * dt) as usize).min(samples.len());
            let out = (0..n).map(|k| samples[(*pos + k) % samples.len()]).collect();
            *pos = (*pos + n) % samples.len().max(1);
            return (out, Some(*rate));
        }
        match &self.mic {
            Some(m) => m.take(),
            None => (Vec::new(), None),
        }
    }
}

/// `--voice-file <wav>` (test automation): a 16-bit PCM recording stands in for the
/// microphone, looped.
fn test_file(mut voice: ResMut<Voice>) {
    let args: Vec<String> = std::env::args().collect();
    let Some(path) = args.windows(2).find(|w| w[0] == "--voice-file").map(|w| w[1].clone()) else {
        return;
    };
    match std::fs::read(&path).map_err(|e| e.to_string()).and_then(|b| wav(&b)) {
        Ok((samples, rate)) => {
            info!("voice: {path}, {} s at {rate} Hz", samples.len() as u32 / rate.max(1));
            voice.file = Some((samples, rate, 0));
        }
        Err(e) => warn!("voice: {path}: {e}"),
    }
}

/// A WAV file's 16-bit PCM samples (the channels mixed) and rate.
fn wav(bytes: &[u8]) -> Result<(Vec<f32>, u32), String> {
    let u16_at = |i: usize| bytes.get(i..i + 2).map(|b| u16::from_le_bytes([b[0], b[1]]));
    let u32_at =
        |i: usize| bytes.get(i..i + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    if bytes.get(..4) != Some(b"RIFF") || bytes.get(8..12) != Some(b"WAVE") {
        return Err("not a WAV file".into());
    }
    let (mut channels, mut rate, mut bits) = (0, 0, 0);
    let mut i = 12;
    while let (Some(id), Some(len)) = (bytes.get(i..i + 4), u32_at(i + 4)) {
        let body = i + 8;
        match id {
            b"fmt " => {
                channels = u16_at(body + 2).unwrap_or(0) as usize;
                rate = u32_at(body + 4).unwrap_or(0);
                bits = u16_at(body + 14).unwrap_or(0);
            }
            b"data" if channels > 0 && bits == 16 => {
                let data =
                    bytes.get(body..body + len as usize).unwrap_or(&bytes[body.min(bytes.len())..]);
                let samples = data
                    .chunks_exact(2 * channels)
                    .map(|f| {
                        f.chunks_exact(2)
                            .map(|s| i16::from_le_bytes([s[0], s[1]]) as f32)
                            .sum::<f32>()
                            / (channels as f32 * 32768.0)
                    })
                    .collect();
                return Ok((samples, rate));
            }
            _ => {}
        }
        i = body + len as usize + (len as usize & 1);
    }
    Err("no 16-bit PCM data".into())
}

fn send(link: &mut NetLink, packet: VoicePacket) {
    match link {
        NetLink::Client(c) => c.voice(packet),
        NetLink::Host(s) => s.voice(packet),
    }
}

/// While the talk key is held, the microphone's voice goes out; let go, the transmission ends.
fn transmit(
    settings: Res<crate::settings::Settings>,
    controls: Res<FlightControls>,
    real: Res<Time<Real>>,
    mut voice: ResMut<Voice>,
    mut link: ResMut<NetLink>,
) {
    let talk = controls.talk && settings.voice;
    if talk && voice.mic.is_none() && voice.file.is_none() {
        voice.mic = Some(capture::Mic::open(&settings.mic_device));
    }
    voice.error = voice.mic.as_ref().and_then(capture::Mic::error);
    let (mut samples, rate) = voice.input(real.delta_secs());
    for s in &mut samples {
        *s = (*s * settings.mic_gain).clamp(-1.0, 1.0);
    }
    let Some(rate) = rate else { return };
    if talk {
        if voice.encoder.as_ref().is_none_or(|(r, _)| *r != rate) {
            voice.encoder = Some((rate, VoiceEncoder::new(rate)));
        }
        let (_, enc) = voice.encoder.as_mut().expect("just made");
        for p in enc.push(&samples) {
            send(&mut link, p);
        }
        voice.transmitting = true;
    } else if voice.transmitting {
        voice.transmitting = false;
        if let Some((_, enc)) = voice.encoder.as_mut() {
            let mut packets = enc.push(&samples);
            packets.push(enc.finish());
            for p in packets {
                send(&mut link, p);
            }
        }
    }
}

/// The others' transmissions into the speaker: from the server (a client), or the players'
/// (a listen server).
fn receive(
    mut voice: ResMut<Voice>,
    mut link: ResMut<NetLink>,
    mut inbox: ResMut<crate::net::NetInbox>,
) {
    let came = match &mut *link {
        NetLink::Client(_) => std::mem::take(&mut inbox.voice),
        NetLink::Host(server) => server.take_voice(),
    };
    for (from, packet) in came {
        if !voice.muted.contains(&from.0) {
            voice.mix.add(from.0, &packet);
        }
    }
    // Players only (not the spoken ATC).
    voice.talking =
        voice.mix.talking().into_iter().filter(|&k| k < crate::tts::PILOT_TALKER).collect();
}

/// The loudspeaker plays in coop, and once the spoken ATC has said something.
fn radio_wanted(link: Option<Res<NetLink>>, tts: Option<Res<crate::tts::Tts>>) -> bool {
    link.is_some() || tts.is_some_and(|t| t.started())
}

/// The radio's loudspeaker plays for as long as the game goes on (spawned again if a screen
/// change took it away).
#[derive(Component)]
struct Speaker;

fn speaker(
    mut commands: Commands,
    settings: Res<crate::settings::Settings>,
    voice: Res<Voice>,
    speakers: Query<Entity, With<Speaker>>,
    mut radios: ResMut<Assets<playback::Radio>>,
) {
    if !speakers.is_empty() {
        return;
    }
    let mix = Mixed(Channel::Voice, 1.0);
    // Endless already: Bevy's looping would buffer it in 4-second blocks (rodio's repeat).
    let playback = mix.settings(&settings, PlaybackSettings::ONCE);
    let radio = radios.add(playback::Radio(voice.mix.clone()));
    commands.spawn((AudioPlayer(radio), playback, mix, Speaker));
}

/// Who is talking, on the cockpit's notices panel.
fn indicators(
    voice: Res<Voice>,
    campaign: Option<Res<crate::campaign::ActiveCampaign>>,
    mut notices: ResMut<crate::player_notices::PlayerNotices>,
) {
    let name = |id: u8| {
        campaign
            .as_ref()
            .and_then(|c| c.state().player_name(stealth_campaign::PlayerId(id)).map(str::to_string))
            .unwrap_or_else(|| format!("Pilot {id}"))
    };
    let mut lines: Vec<String> =
        voice.talking.iter().map(|&k| format!("{} on the radio", name(k))).collect();
    if voice.transmitting {
        lines.insert(0, "Transmitting".to_string());
    } else if let Some(e) = &voice.error {
        lines.insert(0, format!("Microphone: {e}"));
    }
    let radio = (!lines.is_empty()).then(|| lines.join(", "));
    if notices.radio != radio {
        notices.radio = radio;
    }
}

/// Out of the game: the microphone closes, the speaker stops, the voices are forgotten.
fn leave(
    mut commands: Commands,
    mut voice: ResMut<Voice>,
    speakers: Query<Entity, With<Speaker>>,
    mut notices: ResMut<crate::player_notices::PlayerNotices>,
    tts: Option<Res<crate::tts::Tts>>,
) {
    if voice.mic.is_some() || voice.transmitting || !voice.talking.is_empty() {
        voice.mic = None;
        voice.encoder = None;
        voice.transmitting = false;
        voice.talking.clear();
        voice.error = None;
        voice.mix.clear();
        notices.radio = None;
    }
    // The spoken ATC keeps it.
    if tts.is_some_and(|t| t.started()) {
        return;
    }
    for e in &speakers {
        commands.entity(e).despawn();
    }
}

/// Mutes or unmutes a player (the Ops screen's player list).
pub fn toggle_mute(voice: &mut Voice, id: u8) {
    if !voice.muted.remove(&id) {
        voice.muted.insert(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_wav_file() {
        let mut b = Vec::new();
        b.extend(b"RIFF");
        b.extend(0u32.to_le_bytes());
        b.extend(b"WAVEfmt ");
        b.extend(16u32.to_le_bytes());
        b.extend(1u16.to_le_bytes());
        b.extend(2u16.to_le_bytes());
        b.extend(16000u32.to_le_bytes());
        b.extend(64000u32.to_le_bytes());
        b.extend(4u16.to_le_bytes());
        b.extend(16u16.to_le_bytes());
        b.extend(b"data");
        b.extend(8u32.to_le_bytes());
        for s in [16384i16, 0, -16384, -16384] {
            b.extend(s.to_le_bytes());
        }
        let (samples, rate) = wav(&b).unwrap();
        assert_eq!(rate, 16000);
        assert_eq!(samples, [0.25, -0.5]);
        assert!(wav(b"nonsense").is_err());
    }
}
