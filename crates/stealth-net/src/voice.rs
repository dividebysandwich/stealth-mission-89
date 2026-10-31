//! Push-to-talk radio voice: 8 kHz IMA ADPCM, three 20 ms frames to a packet. Pure Rust, the
//! same in the browser build; a radio's bandwidth is all it needs.
//!
//! Each frame starts with the coder's state (predictor and step index), so a lost packet costs
//! only its own 60 ms. The server relays the packets as they are and never decodes them.

use serde::{Deserialize, Serialize};

/// Sample rate of the coded voice (Hz), samples in a frame (20 ms), frames in a packet.
pub const RATE: u32 = 8000;
pub const FRAME: usize = 160;
pub const FRAMES_PER_PACKET: usize = 3;
/// Bytes of a coded frame: the state (3) and two samples to a byte.
pub const FRAME_BYTES: usize = 3 + FRAME / 2;
/// A packet's voice data at most (bytes); the server drops longer ones.
pub const MAX_DATA: usize = FRAME_BYTES * FRAMES_PER_PACKET;

/// A piece of one player's transmission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoicePacket {
    /// Counts up through a transmission (gaps are lost packets).
    pub seq: u16,
    /// The last packet: the transmit button was let go.
    pub end: bool,
    /// Coded frames (`FRAME_BYTES` each).
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

const STEPS: [i32; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66,
    73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449,
    494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272,
    2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630, 9493,
    10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794, 32767,
];
const INDEX_STEP: [i32; 8] = [-1, -1, -1, -1, 2, 4, 6, 8];

/// The IMA ADPCM coder's state, the same on both ends.
#[derive(Debug, Clone, Copy, Default)]
struct Adpcm {
    predictor: i32,
    index: i32,
}

impl Adpcm {
    /// Applies a code: the next sample.
    fn decode(&mut self, code: u8) -> i16 {
        let step = STEPS[self.index as usize];
        let mut diff = step >> 3;
        if code & 4 != 0 {
            diff += step;
        }
        if code & 2 != 0 {
            diff += step >> 1;
        }
        if code & 1 != 0 {
            diff += step >> 2;
        }
        if code & 8 != 0 {
            diff = -diff;
        }
        self.predictor = (self.predictor + diff).clamp(-32768, 32767);
        self.index = (self.index + INDEX_STEP[(code & 7) as usize]).clamp(0, 88);
        self.predictor as i16
    }

    /// The code nearest the sample (and the state moves on as the decoder's will).
    fn encode(&mut self, sample: i16) -> u8 {
        let step = STEPS[self.index as usize];
        let mut diff = sample as i32 - self.predictor;
        let mut code = 0u8;
        if diff < 0 {
            code = 8;
            diff = -diff;
        }
        let mut s = step;
        for bit in [4u8, 2, 1] {
            if diff >= s {
                code |= bit;
                diff -= s;
            }
            s >>= 1;
        }
        self.decode(code);
        code
    }
}

/// Codes one frame of `FRAME` samples onto `out`.
fn encode_frame(state: &mut Adpcm, pcm: &[i16], out: &mut Vec<u8>) {
    out.extend_from_slice(&(state.predictor as i16).to_le_bytes());
    out.push(state.index as u8);
    for pair in pcm.chunks(2) {
        let lo = state.encode(pair[0]);
        let hi = pair.get(1).map_or(0, |&s| state.encode(s));
        out.push(lo | hi << 4);
    }
}

/// Decodes one coded frame onto `out` (None: too short).
fn decode_frame(data: &[u8], out: &mut Vec<f32>) -> Option<()> {
    if data.len() < FRAME_BYTES {
        return None;
    }
    let mut state = Adpcm {
        predictor: i16::from_le_bytes([data[0], data[1]]) as i32,
        index: (data[2] as i32).clamp(0, 88),
    };
    for &b in &data[3..FRAME_BYTES] {
        out.push(state.decode(b & 15) as f32 / 32768.0);
        out.push(state.decode(b >> 4) as f32 / 32768.0);
    }
    Some(())
}

/// A packet's voice at `RATE` (-1..1); empty if it is malformed.
pub fn decode(packet: &VoicePacket) -> Vec<f32> {
    let mut out = Vec::with_capacity(packet.data.len() / FRAME_BYTES * FRAME);
    for frame in packet.data.chunks(FRAME_BYTES).take(FRAMES_PER_PACKET) {
        if decode_frame(frame, &mut out).is_none() {
            return Vec::new();
        }
    }
    out
}

/// A biquad low-pass (RBJ cookbook), for the decimation.
#[derive(Debug, Clone, Copy)]
struct LowPass {
    b: [f32; 3],
    a: [f32; 2],
    x: [f32; 2],
    y: [f32; 2],
}

impl LowPass {
    fn new(rate: f32, cutoff: f32, q: f32) -> Self {
        let w = std::f32::consts::TAU * cutoff / rate;
        let (sin, cos) = w.sin_cos();
        let alpha = sin / (2.0 * q);
        let a0 = 1.0 + alpha;
        let b1 = (1.0 - cos) / a0;
        Self {
            b: [b1 / 2.0, b1, b1 / 2.0],
            a: [-2.0 * cos / a0, (1.0 - alpha) / a0],
            x: [0.0; 2],
            y: [0.0; 2],
        }
    }

    fn run(&mut self, v: f32) -> f32 {
        let y = self.b[0] * v + self.b[1] * self.x[0] + self.b[2] * self.x[1]
            - self.a[0] * self.y[0]
            - self.a[1] * self.y[1];
        self.x = [v, self.x[0]];
        self.y = [y, self.y[0]];
        y
    }
}

/// The microphone's rate down to `RATE`: a steep low-pass under 4 kHz, then samples picked
/// between (linear).
#[derive(Debug, Clone)]
pub struct Downsampler {
    step: f64,
    pos: f64,
    filters: [LowPass; 4],
    last: f32,
}

impl Downsampler {
    pub fn new(input_rate: u32) -> Self {
        let rate = input_rate.max(RATE) as f32;
        // Butterworth eighth order as four biquads: a 6 kHz tone (aliasing to 2 kHz) is 40 dB
        // down.
        let filter = |q| LowPass::new(rate, 3400.0, q);
        Self {
            step: input_rate as f64 / RATE as f64,
            pos: 0.0,
            filters: [filter(0.5098), filter(0.6013), filter(0.9000), filter(2.5629)],
            last: 0.0,
        }
    }

    /// Takes in samples at the input rate, puts out those at `RATE`.
    pub fn push(&mut self, input: &[f32], out: &mut Vec<f32>) {
        for &v in input {
            let v = self.filters.iter_mut().fold(v, |v, f| f.run(v));
            // Output samples falling between the last input and this one.
            while self.pos < 1.0 {
                out.push(self.last + (v - self.last) * self.pos as f32);
                self.pos += self.step;
            }
            self.pos -= 1.0;
            self.last = v;
        }
    }
}

/// One player's transmissions: microphone samples in, packets out.
#[derive(Debug, Clone)]
pub struct VoiceEncoder {
    down: Downsampler,
    pending: Vec<f32>,
    state: Adpcm,
    seq: u16,
    scratch: Vec<f32>,
}

impl VoiceEncoder {
    pub fn new(input_rate: u32) -> Self {
        Self {
            down: Downsampler::new(input_rate),
            pending: Vec::new(),
            state: Adpcm::default(),
            seq: 0,
            scratch: Vec::new(),
        }
    }

    /// Microphone samples (-1..1, mono): the packets they complete.
    pub fn push(&mut self, input: &[f32]) -> Vec<VoicePacket> {
        self.scratch.clear();
        self.down.push(input, &mut self.scratch);
        self.pending.extend_from_slice(&self.scratch);
        let mut out = Vec::new();
        while self.pending.len() >= FRAME * FRAMES_PER_PACKET {
            let samples: Vec<f32> = self.pending.drain(..FRAME * FRAMES_PER_PACKET).collect();
            out.push(self.packet(&samples, false));
        }
        out
    }

    /// The transmission ends: what is left, in a last packet.
    pub fn finish(&mut self) -> VoicePacket {
        let mut samples = std::mem::take(&mut self.pending);
        let frames = samples.len().div_ceil(FRAME).max(1);
        samples.resize(frames * FRAME, 0.0);
        let p = self.packet(&samples, true);
        self.state = Adpcm::default();
        p
    }

    fn packet(&mut self, samples: &[f32], end: bool) -> VoicePacket {
        let pcm: Vec<i16> =
            samples.iter().map(|v| (v.clamp(-1.0, 1.0) * 32767.0).round() as i16).collect();
        let mut data = Vec::with_capacity(MAX_DATA);
        for frame in pcm.chunks(FRAME) {
            encode_frame(&mut self.state, frame, &mut data);
        }
        let seq = self.seq;
        self.seq = self.seq.wrapping_add(1);
        VoicePacket { seq, end, data }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(rate: u32, hz: f32, secs: f32, amp: f32) -> Vec<f32> {
        (0..(rate as f32 * secs) as usize)
            .map(|i| amp * (std::f32::consts::TAU * hz * i as f32 / rate as f32).sin())
            .collect()
    }

    fn rms(x: &[f32]) -> f32 {
        (x.iter().map(|v| v * v).sum::<f32>() / x.len().max(1) as f32).sqrt()
    }

    #[test]
    fn adpcm_round_trip_is_close() {
        let input = tone(RATE, 700.0, 0.06, 0.5);
        let pcm: Vec<i16> = input.iter().map(|v| (v * 32767.0) as i16).collect();
        let mut state = Adpcm::default();
        let mut data = Vec::new();
        for f in pcm.chunks(FRAME) {
            encode_frame(&mut state, f, &mut data);
        }
        assert_eq!(data.len(), MAX_DATA);
        let out = decode(&VoicePacket { seq: 0, end: false, data });
        assert_eq!(out.len(), input.len());
        // After the first few samples' settling: an SNR over 20 dB.
        let err: Vec<f32> = input.iter().zip(&out).skip(40).map(|(a, b)| a - b).collect();
        let snr = 20.0 * (rms(&input[40..]) / rms(&err)).log10();
        assert!(snr > 20.0, "SNR {snr} dB");
    }

    #[test]
    fn voice_band_passes_and_high_tones_are_cut() {
        let level = |hz: f32| {
            let mut d = Downsampler::new(48_000);
            let mut out = Vec::new();
            d.push(&tone(48_000, hz, 0.5, 0.5), &mut out);
            assert!((out.len() as i32 - 4000).abs() <= 1, "{}", out.len());
            rms(&out[1000..]) / (0.5 / 2f32.sqrt())
        };
        assert!(level(1000.0) > 0.9, "{}", level(1000.0));
        assert!(level(6000.0) < 0.02, "{}", level(6000.0));
        assert!(Downsampler::new(44_100).step > 5.5);
    }

    #[test]
    fn encoder_packets_and_finish() {
        let mut enc = VoiceEncoder::new(48_000);
        let packets = enc.push(&tone(48_000, 500.0, 0.2, 0.3));
        // 0.2 s at 8 kHz is 1600 samples: three packets of 480, 160 left over.
        assert_eq!(packets.len(), 3);
        assert_eq!(packets.iter().map(|p| p.seq).collect::<Vec<_>>(), [0, 1, 2]);
        assert!(packets.iter().all(|p| !p.end && p.data.len() == MAX_DATA));
        let last = enc.finish();
        assert!(last.end && last.seq == 3 && last.data.len() == FRAME_BYTES);
        assert_eq!(decode(&last).len(), FRAME);
        assert!(decode(&VoicePacket { seq: 0, end: false, data: vec![1, 2, 3] }).is_empty());
    }

    #[test]
    fn packet_is_small_on_the_wire() {
        let mut enc = VoiceEncoder::new(48_000);
        let p = enc.push(&tone(48_000, 500.0, 0.07, 0.3)).remove(0);
        let to_server = crate::protocol::encode(&crate::protocol::ToServer::Voice(p.clone()));
        let to_client = crate::protocol::encode(&crate::protocol::ToClient::Voice {
            from: stealth_campaign::PlayerId(3),
            packet: p.clone(),
        });
        assert!(
            to_server.len() < 300 && to_client.len() < 300,
            "{} {}",
            to_server.len(),
            to_client.len()
        );
        let back: crate::protocol::ToClient = crate::protocol::decode(&to_client).unwrap();
        assert!(matches!(back, crate::protocol::ToClient::Voice { packet, .. } if packet == p));
    }
}
