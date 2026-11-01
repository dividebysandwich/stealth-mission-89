//! A Piper (VITS) voice run through `rten`: phoneme ids in, a whole utterance of samples out
//! at the voice's rate.

use std::path::Path;
use std::sync::{Arc, OnceLock};

use rten::{Model, RunOptions, ThreadPool};
use rten_tensor::NdTensor;
use rten_tensor::prelude::*;

use super::voice::VoiceConfig;

pub struct Synth {
    model: Model,
    pub config: VoiceConfig,
}

/// Threads the voices run on: a few, not one per core (rten's default), so a call on the radio
/// doesn't take the CPU from the frame (the Steam Frame most of all). A 5 s call still takes
/// well under a second.
const THREADS: usize = 2;

fn options() -> RunOptions {
    static POOL: OnceLock<Arc<ThreadPool>> = OnceLock::new();
    let pool = POOL.get_or_init(|| Arc::new(ThreadPool::with_num_threads(THREADS)));
    RunOptions::default().with_thread_pool(Some(pool.clone()))
}

impl Synth {
    /// Maps the model rather than reading it. Unsafe because rten reads the weights straight
    /// out of the mapping: the file must not change under it, which is why a voice is
    /// unloaded before the voice dialog deletes it.
    pub fn load(model: &Path) -> Result<Self, String> {
        let config = VoiceConfig::load(model)?;
        let model =
            unsafe { Model::load_mmap(model) }.map_err(|e| format!("{}: {e}", model.display()))?;
        Ok(Self { model, config })
    }

    pub fn sample_rate(&self) -> u32 {
        self.config.audio.sample_rate
    }

    /// `^`, each phoneme's ids followed by 0, `$`; symbols the voice lacks are left out.
    fn encode(&self, phonemes: &str) -> Vec<i32> {
        let map = &self.config.phoneme_id_map;
        let mut ids = map.get(&'^').cloned().unwrap_or_default();
        for c in phonemes.chars() {
            if let Some(v) = map.get(&c) {
                ids.extend_from_slice(v);
                ids.push(0);
            }
        }
        ids.extend(map.get(&'$').cloned().unwrap_or_default());
        ids
    }

    /// One utterance. `tempo` multiplies the voice's speed (1.1: 10 % faster); `speaker`
    /// picks one of a multi-speaker voice's.
    pub fn speak(&self, phonemes: &str, speaker: u32, tempo: f32) -> Result<Vec<f32>, String> {
        let ids = self.encode(phonemes);
        // Nothing between `^` and `$`: the model would only click.
        if ids.len() <= 2 {
            return Ok(Vec::new());
        }
        let inf = &self.config.inference;
        let length_scale = inf.length_scale / tempo.clamp(0.5, 2.0);
        let phoneme_ids: NdTensor<i32, 1> = ids.into();
        let input_lengths = NdTensor::from([phoneme_ids.size(0) as i32]);
        let phoneme_ids = phoneme_ids.with_new_axis(0);
        let scales = NdTensor::from([inf.noise_scale, length_scale, inf.noise_w]);
        let node = |name: &str| self.model.node_id(name).map_err(|e| format!("`{name}`: {e}"));
        let mut inputs = vec![
            (node("input")?, phoneme_ids.into()),
            (node("input_lengths")?, input_lengths.into()),
            (node("scales")?, scales.into()),
        ];
        if self.config.num_speakers > 1 {
            let sid = speaker.min(self.config.num_speakers - 1) as i32;
            inputs.push((node("sid")?, NdTensor::from([sid]).into()));
        }
        let [out] = self
            .model
            .run_n(inputs, [node("output")?], Some(options()))
            .map_err(|e| e.to_string())?;
        // (batch, time, 1, samples)
        let samples: NdTensor<f32, 4> = out.try_into().map_err(|e| format!("output: {e:?}"))?;
        Ok(samples.iter().copied().collect())
    }
}
