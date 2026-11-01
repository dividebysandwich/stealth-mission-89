//! Spoken ATC: the controllers' and the pilot's calls said by Piper voices (text to speech in
//! pure Rust, `rten`), through the radio's loudspeaker (`crate::voice`). No voice ships with
//! the game: the voice dialog (`voice_dialog.rs`) downloads freely licensed ones from a
//! catalogue (`catalog.rs`), and voices dropped into `voices/` next to the game are found too.
//! Without a voice the calls are text, as before. Not in the browser build.

#[cfg(not(target_arch = "wasm32"))]
pub mod catalog;
#[cfg(not(target_arch = "wasm32"))]
pub mod download;
#[cfg(not(target_arch = "wasm32"))]
mod engine;
#[cfg(not(target_arch = "wasm32"))]
mod model;
#[cfg(not(target_arch = "wasm32"))]
mod phonemes;
#[cfg(not(target_arch = "wasm32"))]
pub mod phrase;
#[cfg(not(target_arch = "wasm32"))]
pub mod voice;

use bevy::prelude::*;

/// The radio talkers of the voices (beside the coop players' ids).
pub const ATC_TALKER: u8 = 254;
pub const PILOT_TALKER: u8 = 253;

pub struct TtsPlugin;

impl Plugin for TtsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::voice::Voice>().init_resource::<Tts>();
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Startup, say_flag).add_systems(
            OnExit(crate::settings::AppState::Flight),
            // The flight is over: so is its radio call.
            |tts: Res<Tts>| tts.stop(),
        );
    }
}

/// The speech: started the first time something is said.
#[derive(Resource)]
pub struct Tts {
    #[cfg(not(target_arch = "wasm32"))]
    engine: Option<engine::Engine>,
    #[cfg(not(target_arch = "wasm32"))]
    mix: crate::voice::RadioMix,
    /// The voices on disk (`refresh` after a download or a delete).
    #[cfg(not(target_arch = "wasm32"))]
    pub installed: Vec<voice::InstalledVoice>,
}

impl FromWorld for Tts {
    fn from_world(world: &mut World) -> Self {
        let _mix = world.resource::<crate::voice::Voice>().mix().clone();
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            engine: None,
            #[cfg(not(target_arch = "wasm32"))]
            mix: _mix,
            #[cfg(not(target_arch = "wasm32"))]
            installed: voice::installed(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Tts {
    fn engine(&mut self) -> &engine::Engine {
        self.engine.get_or_insert_with(|| engine::Engine::start(self.mix.clone()))
    }

    pub fn refresh(&mut self) {
        self.installed = voice::installed();
    }

    pub fn has(&self, name: &str) -> bool {
        !name.is_empty() && self.installed.iter().any(|v| v.name == name)
    }

    /// Speakers of a voice (1 unless it is a multi-speaker one).
    pub fn speakers(&self, name: &str) -> u32 {
        self.installed.iter().find(|v| v.name == name).map_or(1, |v| v.speakers)
    }

    /// Says `text` (a radio call, numbers as written) as `talker` in a voice; false if that
    /// voice isn't there.
    pub fn say(&mut self, talker: u8, voice: &str, speaker: u32, tempo: f32, text: &str) -> bool {
        if !self.has(voice) {
            return false;
        }
        let line = engine::Line {
            talker,
            voice: voice.to_string(),
            speaker,
            tempo,
            text: phrase::spoken(text),
        };
        self.engine().say(line);
        true
    }

    /// Cuts what is being said and says this instead (the voice dialog's preview).
    pub fn preview(&mut self, talker: u8, voice: &str, speaker: u32, text: &str) {
        self.stop();
        self.say(talker, voice, speaker, crate::atc::RADIO_TEMPO, text);
    }

    pub fn stop(&self) {
        if let Some(e) = &self.engine {
            e.stop();
        }
    }

    /// A line is waiting or on the radio.
    pub fn busy(&self) -> bool {
        self.engine.as_ref().is_some_and(engine::Engine::busy)
    }

    /// The speech has started (the radio's loudspeaker is wanted).
    pub fn started(&self) -> bool {
        self.engine.is_some()
    }

    /// Lets go of a voice's files.
    pub fn unload(&self, name: &str) {
        if let Some(e) = &self.engine {
            e.unload(name);
        }
    }

    pub fn error(&self) -> Option<String> {
        self.engine.as_ref().and_then(engine::Engine::error)
    }
}

/// The browser build: never speaks.
#[cfg(target_arch = "wasm32")]
impl Tts {
    pub fn say(&mut self, _: u8, _: &str, _: u32, _: f32, _: &str) -> bool {
        false
    }

    pub fn busy(&self) -> bool {
        false
    }

    pub fn speakers(&self, _name: &str) -> u32 {
        1
    }

    pub fn started(&self) -> bool {
        false
    }
}

/// `--say "<text>"` (testing a voice): says it as the controller in the chosen ATC voice.
#[cfg(not(target_arch = "wasm32"))]
fn say_flag(mut tts: ResMut<Tts>, settings: Res<crate::settings::Settings>) {
    let args: Vec<String> = std::env::args().collect();
    if let Some(text) = args.windows(2).find(|w| w[0] == "--say").map(|w| w[1].clone())
        && !tts.say(ATC_TALKER, &settings.atc_voice, settings.atc_speaker, 1.0, &text)
    {
        warn!("--say: no ATC voice (chosen: {:?})", settings.atc_voice);
    }
}
