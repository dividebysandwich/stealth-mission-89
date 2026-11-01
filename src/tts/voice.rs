//! The voices on disk: `<name>.onnx` with its `<name>.onnx.json` beside it, in the player's
//! voices folder (where the downloads go) or in `voices/` next to the game.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::catalog::{self, CatalogVoice};

/// The parts of a Piper `.onnx.json` used here.
#[derive(Debug, Clone, Deserialize)]
pub struct VoiceConfig {
    pub audio: AudioConfig,
    pub inference: InferenceConfig,
    /// IPA character to the model's input ids.
    pub phoneme_id_map: HashMap<char, Vec<i32>>,
    #[serde(default = "one")]
    pub num_speakers: u32,
    /// Speaker names (the corpus's) to ids, for a multi-speaker voice.
    #[serde(default)]
    pub speaker_id_map: HashMap<String, u32>,
}

fn one() -> u32 {
    1
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct InferenceConfig {
    pub noise_scale: f32,
    /// The voice's tempo: larger is slower.
    pub length_scale: f32,
    pub noise_w: f32,
}

impl VoiceConfig {
    pub fn load(model: &Path) -> Result<Self, String> {
        let path = config_path(model);
        let json =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        serde_json::from_str(&json).map_err(|e| format!("{}: {e}", path.display()))
    }
}

/// `x.onnx` -> `x.onnx.json`.
fn config_path(model: &Path) -> PathBuf {
    let mut p = model.as_os_str().to_owned();
    p.push(".json");
    PathBuf::from(p)
}

/// Where downloaded voices go (None: no player folder).
pub fn download_dir() -> Option<PathBuf> {
    stealth_host::store::path("voices")
}

/// Every folder a voice may be in, the downloads' first.
pub fn voice_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = download_dir().into_iter().collect();
    if let Some(exe) = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf))
    {
        dirs.push(exe.join("voices"));
    }
    if cfg!(debug_assertions) {
        dirs.push(PathBuf::from("assets/voices"));
    }
    dirs
}

/// A voice found on disk.
#[derive(Debug, Clone)]
pub struct InstalledVoice {
    /// The files' stem, e.g. `en_US-joe-medium`.
    pub name: String,
    pub model: PathBuf,
    /// Bytes of its files.
    pub size: u64,
    pub speakers: u32,
    /// Speaker names in id order (empty: a single speaker).
    pub speaker_names: Vec<String>,
    /// In the downloads folder (the dialog may delete it).
    pub removable: bool,
    pub catalog: Option<&'static CatalogVoice>,
}

impl InstalledVoice {
    /// The model and its two companions (those present).
    pub fn files(&self) -> Vec<PathBuf> {
        let dir = self.model.parent().unwrap_or(Path::new("."));
        [
            self.model.clone(),
            config_path(&self.model),
            dir.join(format!("{}.MODEL_CARD", self.name)),
        ]
        .into_iter()
        .filter(|p| p.exists())
        .collect()
    }
}

/// The voices in `voice_dirs`, by name (the first found of a name wins).
pub fn installed() -> Vec<InstalledVoice> {
    let download = download_dir();
    let mut out: Vec<InstalledVoice> = Vec::new();
    for dir in voice_dirs() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        let mut models: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "onnx"))
            .collect();
        models.sort();
        for model in models {
            let Some(name) = model.file_stem().and_then(|s| s.to_str()).map(str::to_string) else {
                continue;
            };
            if out.iter().any(|v| v.name == name) {
                continue;
            }
            let Ok(config) = VoiceConfig::load(&model) else { continue };
            let mut speaker_names = vec![String::new(); config.speaker_id_map.len()];
            for (n, &id) in &config.speaker_id_map {
                if let Some(s) = speaker_names.get_mut(id as usize) {
                    *s = n.clone();
                }
            }
            let mut v = InstalledVoice {
                catalog: catalog::find(&name),
                name,
                model: model.clone(),
                size: 0,
                speakers: config.num_speakers.max(1),
                speaker_names,
                removable: download.as_ref().is_some_and(|d| model.parent() == Some(d.as_path())),
            };
            v.size = v.files().iter().filter_map(|p| p.metadata().ok()).map(|m| m.len()).sum();
            out.push(v);
        }
    }
    out
}

/// The installed voice of that name.
pub fn find(name: &str) -> Option<InstalledVoice> {
    if name.is_empty() {
        return None;
    }
    installed().into_iter().find(|v| v.name == name)
}
