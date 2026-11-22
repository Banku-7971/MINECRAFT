#![cfg_attr(not(windows), allow(dead_code))]
//! Real Recorded Audio & Music Engine:
//! Plays real open-source Foley sound effects, mob vocalizations, jukebox discs,
//! and dimension piano/ambient soundtracks packed in `assets/sounds.bin` (IMA-ADPCM)
//! on Windows/native, and streams real `.ogg` assets on WebAssembly (`web/sounds/`).

use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

pub const SAMPLE_RATE: u32 = 22050;

#[cfg(not(target_arch = "wasm32"))]
static SOUNDS_BIN: &[u8] = include_bytes!("../assets/sounds.bin");

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Sfx {
    Click = 0,
    Place = 1,
    Break = 2,
    Step = 3,
    StepGrass = 4,
    StepStone = 5,
    StepWood = 6,
    StepSand = 7,
    StepWater = 8,
    Jump = 9,
    Land = 10,
    Hurt = 11,
    MobHurt = 12,
    MobDie = 13,
    Pickup = 14,
    Craft = 15,
    Furnace = 16,
    Explode = 17,
    Splash = 18,
    Zombie = 19,
    Skeleton = 20,
    Cow = 21,
    Sheep = 22,
    Pig = 23,
    Chicken = 24,
    Spider = 25,
    Creeper = 26,
    Enderman = 27,
    Door = 28,
    LevelUp = 29,
    ShieldBlock = 30,
    Totem = 31,
    Beacon = 32,
    Wither = 33,
    JukeboxCat = 34,
    JukeboxPigstep = 35,
    Villager = 36,
    Wolf = 37,
    Bee = 38,
    Warden = 39,
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
static WEB_SFX_QUEUE: OnceLock<Mutex<VecDeque<u8>>> = OnceLock::new();

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn push_web_sfx(s: Sfx) {
    let q = WEB_SFX_QUEUE.get_or_init(|| Mutex::new(VecDeque::with_capacity(32)));
    if let Ok(mut g) = q.lock() {
        if g.len() < 32 {
            g.push_back(s as u8);
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn pop_web_sfx() -> i32 {
    let q = WEB_SFX_QUEUE.get_or_init(|| Mutex::new(VecDeque::with_capacity(32)));
    if let Ok(mut g) = q.lock() {
        if let Some(id) = g.pop_front() {
            return id as i32;
        }
    }
    -1
}

struct Voice {
    data: Vec<i16>,
    /// 16.16 fixed-point sample cursor for crisp pitch-varied playback
    pos_fp: u64,
    step_fp: u64,
    volume: f32,
}

pub struct Audio {
    pub music_on: bool,
    pub sound_on: bool,
    pub music_vol: f32,
    pub sound_vol: f32,
    music_tracks: [Vec<i16>; 3],
    music_dim: usize,
    music_pos: usize,
    voices: Vec<Voice>,
    sfx: Vec<(u8, Vec<i16>)>,
    master: f32,
    rng: u32,
}

const INDEX_TABLE: [i32; 16] = [-1, -1, -1, -1, 2, 4, 6, 8, -1, -1, -1, -1, 2, 4, 6, 8];
const STEP_TABLE: [i32; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45,
    50, 55, 60, 66, 73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307,
    337, 371, 408, 449, 494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552,
    1707, 1878, 2066, 2272, 2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484,
    7132, 7845, 8630, 9493, 10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385,
    24623, 27086, 29794, 32767,
];

fn decode_ima_adpcm(data: &[u8], num_samples: usize, src_rate: u32) -> Vec<i16> {
    let mut pcm = Vec::with_capacity(num_samples);
    let mut pred: i32 = 0;
    let mut step_idx: i32 = 0;
    for &byte in data {
        for nib in [byte & 0x0F, (byte >> 4) & 0x0F] {
            if pcm.len() >= num_samples {
                break;
            }
            let step = STEP_TABLE[step_idx as usize];
            let mut vpdiff = step >> 3;
            if (nib & 4) != 0 { vpdiff += step; }
            if (nib & 2) != 0 { vpdiff += step >> 1; }
            if (nib & 1) != 0 { vpdiff += step >> 2; }
            if (nib & 8) != 0 { pred -= vpdiff; } else { pred += vpdiff; }
            pred = pred.clamp(-32768, 32767);
            step_idx = (step_idx + INDEX_TABLE[nib as usize]).clamp(0, 88);
            pcm.push(pred as i16);
        }
    }
    if src_rate == SAMPLE_RATE || pcm.is_empty() {
        return pcm;
    }
    let out_len = ((pcm.len() as f64) * (SAMPLE_RATE as f64 / src_rate as f64))
        .round()
        .max(1.0) as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let pos = i as f64 * (src_rate as f64 / SAMPLE_RATE as f64);
        let idx = (pos as usize).min(pcm.len() - 1);
        let frac = (pos - idx as f64) as f32;
        let s0 = pcm[idx] as f32;
        let s1 = pcm[(idx + 1).min(pcm.len() - 1)] as f32;
        out.push((s0 + (s1 - s0) * frac) as i16);
    }
    out
}

#[cfg(not(target_arch = "wasm32"))]
fn load_sound_bank() -> (Vec<(u8, Vec<i16>)>, [Vec<i16>; 3]) {
    let mut sfx = Vec::new();
    let mut tracks: [Vec<i16>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    if SOUNDS_BIN.len() < 6 || &SOUNDS_BIN[0..4] != b"SNDB" {
        return (sfx, tracks);
    }
    let count = u16::from_le_bytes([SOUNDS_BIN[4], SOUNDS_BIN[5]]) as usize;
    let header_end = 6 + count * 13;
    if SOUNDS_BIN.len() < header_end {
        return (sfx, tracks);
    }
    let mut off = header_end;
    for i in 0..count {
        let h = 6 + i * 13;
        let id = SOUNDS_BIN[h];
        let rate = u32::from_le_bytes([SOUNDS_BIN[h + 1], SOUNDS_BIN[h + 2], SOUNDS_BIN[h + 3], SOUNDS_BIN[h + 4]]);
        let nsamp = u32::from_le_bytes([SOUNDS_BIN[h + 5], SOUNDS_BIN[h + 6], SOUNDS_BIN[h + 7], SOUNDS_BIN[h + 8]]) as usize;
        let dlen = u32::from_le_bytes([SOUNDS_BIN[h + 9], SOUNDS_BIN[h + 10], SOUNDS_BIN[h + 11], SOUNDS_BIN[h + 12]]) as usize;
        let end = off.saturating_add(dlen).min(SOUNDS_BIN.len());
        let decoded = decode_ima_adpcm(&SOUNDS_BIN[off..end], nsamp, rate);
        off = end;
        if (40..=42).contains(&id) {
            tracks[(id - 40) as usize] = decoded;
        } else {
            sfx.push((id, decoded));
        }
    }
    (sfx, tracks)
}

#[cfg(target_arch = "wasm32")]
fn load_sound_bank() -> (Vec<(u8, Vec<i16>)>, [Vec<i16>; 3]) {
    (Vec::new(), [Vec::new(), Vec::new(), Vec::new()])
}

impl Audio {
    pub fn new() -> Self {
        let (sfx, music_tracks) = load_sound_bank();
        Audio {
            music_on: true,
            sound_on: true,
            music_vol: 0.50,
            sound_vol: 0.85,
            music_tracks,
            music_dim: 0,
            music_pos: 0,
            voices: Vec::new(),
            sfx,
            master: 0.90,
            rng: 0x5EED_CAFE,
        }
    }

    pub fn set_dimension(&mut self, dim: u8) {
        let d = (dim as usize).min(2);
        if d != self.music_dim {
            self.music_dim = d;
            self.music_pos = 0;
        }
    }

    fn next_rand(&mut self) -> f32 {
        self.rng = self.rng.wrapping_mul(1664525).wrapping_add(1013904223);
        ((self.rng >> 8) as f32) / 16777216.0
    }

    fn pitch_step(&mut self, s: Sfx) -> u64 {
        let pitch = match s {
            Sfx::Click | Sfx::LevelUp | Sfx::Totem | Sfx::Beacon | Sfx::JukeboxCat | Sfx::JukeboxPigstep => 1.0,
            Sfx::Pickup => 0.94 + self.next_rand() * 0.18,
            Sfx::Step | Sfx::StepGrass | Sfx::StepStone | Sfx::StepWood | Sfx::StepSand | Sfx::StepWater => {
                0.88 + self.next_rand() * 0.24
            }
            _ => 0.92 + self.next_rand() * 0.16,
        };
        ((pitch * 65536.0) as u64).max(32768)
    }

    pub fn play(&mut self, s: Sfx) {
        if !self.sound_on || self.sound_vol <= 0.01 {
            return;
        }
        let step_fp = self.pitch_step(s);
        if let Some(custom) = crate::packs::custom_sound_bytes(s as u8) {
            if let Some(pcm) = decode_wav_pcm(&custom) {
                if self.voices.len() < 18 {
                    self.voices.push(Voice {
                        data: pcm,
                        pos_fp: 0,
                        step_fp,
                        volume: 0.72 * self.sound_vol,
                    });
                }
                return;
            }
        }
        let sid = s as u8;
        if let Some((_, data)) = self.sfx.iter().find(|(k, _)| *k == sid) {
            if self.voices.len() < 18 {
                let vol = match s {
                    Sfx::JukeboxCat | Sfx::JukeboxPigstep => 0.72,
                    Sfx::Explode => 0.72,
                    Sfx::Step | Sfx::StepGrass | Sfx::StepStone | Sfx::StepWood | Sfx::StepSand | Sfx::StepWater => 0.36,
                    Sfx::Click => 0.45,
                    _ => 0.60,
                } * self.sound_vol;
                self.voices.push(Voice {
                    data: data.clone(),
                    pos_fp: 0,
                    step_fp,
                    volume: vol,
                });
            }
        }
    }

    /// Fill an output buffer (mono i16) with mixed music + pitch-interpolated effects.
    pub fn render(&mut self, out: &mut [i16]) {
        let track = &self.music_tracks[self.music_dim];
        let tlen = track.len();
        let m_gain = 0.38 * self.music_vol;
        for sample_out in out.iter_mut() {
            let mut v = 0.0f32;
            if self.music_on && self.music_vol > 0.01 && tlen > 0 {
                v += (track[self.music_pos] as f32 / 32768.0) * m_gain;
                self.music_pos += 1;
                if self.music_pos >= tlen {
                    self.music_pos = 0;
                }
            }
            for vc in self.voices.iter_mut() {
                let idx = (vc.pos_fp >> 16) as usize;
                if idx < vc.data.len() {
                    let frac = (vc.pos_fp & 0xFFFF) as f32 / 65536.0;
                    let s0 = vc.data[idx] as f32;
                    let s1 = if idx + 1 < vc.data.len() {
                        vc.data[idx + 1] as f32
                    } else {
                        0.0
                    };
                    let interp = s0 + (s1 - s0) * frac;
                    v += (interp / 32768.0) * vc.volume;
                    vc.pos_fp += vc.step_fp;
                }
            }
            self.voices.retain(|vc| ((vc.pos_fp >> 16) as usize) < vc.data.len());
            let scaled = v * self.master;
            let s = if scaled.abs() > 0.85 {
                scaled.signum() * (0.85 + 0.15 * ((scaled.abs() - 0.85) / 0.5).tanh())
            } else {
                scaled
            };
            *sample_out = (s.clamp(-1.0, 1.0) * 32000.0) as i16;
        }
    }
}

fn decode_wav_pcm(bytes: &[u8]) -> Option<Vec<i16>> {
    if bytes.len() < 44 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return None;
    }
    let mut i = 12usize;
    let mut channels = 1u16;
    let mut rate = SAMPLE_RATE;
    let mut bits = 16u16;
    let mut data_slice: Option<&[u8]> = None;
    while i + 8 <= bytes.len() {
        let id = &bytes[i..i + 4];
        let sz = u32::from_le_bytes([bytes[i + 4], bytes[i + 5], bytes[i + 6], bytes[i + 7]]) as usize;
        let start = i + 8;
        let end = start.saturating_add(sz).min(bytes.len());
        if id == b"fmt " && sz >= 16 {
            let fmt = u16::from_le_bytes([bytes[start], bytes[start + 1]]);
            if fmt != 1 {
                return None;
            }
            channels = u16::from_le_bytes([bytes[start + 2], bytes[start + 3]]).max(1);
            rate = u32::from_le_bytes([bytes[start + 4], bytes[start + 5], bytes[start + 6], bytes[start + 7]]).max(4000);
            bits = u16::from_le_bytes([bytes[start + 14], bytes[start + 15]]);
        } else if id == b"data" {
            data_slice = Some(&bytes[start..end]);
            break;
        }
        i = end + (sz & 1);
    }
    let raw = data_slice?;
    let mut mono = Vec::new();
    let ch = channels as usize;
    if bits == 16 {
        let step = 2 * ch;
        let mut p = 0usize;
        while p + step <= raw.len() {
            let mut acc = 0i32;
            for c in 0..ch {
                acc += i16::from_le_bytes([raw[p + c * 2], raw[p + c * 2 + 1]]) as i32;
            }
            mono.push((acc / ch as i32) as i16);
            p += step;
        }
    } else if bits == 8 {
        let mut p = 0usize;
        while p + ch <= raw.len() {
            let mut acc = 0i32;
            for c in 0..ch {
                acc += ((raw[p + c] as i32) - 128) * 256;
            }
            mono.push((acc / ch as i32) as i16);
            p += ch;
        }
    } else {
        return None;
    }
    if mono.is_empty() {
        return None;
    }
    if rate == SAMPLE_RATE {
        return Some(mono);
    }
    let out_len = ((mono.len() as f64) * (SAMPLE_RATE as f64 / rate as f64))
        .round()
        .max(1.0) as usize;
    let mut resampled = Vec::with_capacity(out_len);
    for idx in 0..out_len {
        let src = (idx as f64 * rate as f64 / SAMPLE_RATE as f64) as usize;
        resampled.push(mono[src.min(mono.len() - 1)]);
    }
    Some(resampled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_audio_and_tracks_render_cleanly() {
        let mut audio = Audio::new();
        for dim in 0..=2u8 {
            audio.set_dimension(dim);
            let mut buf = vec![0i16; 2048];
            audio.render(&mut buf);
            assert!(buf.iter().any(|&s| s != 0), "music track {dim} should produce audio");
        }
        audio.music_on = false;
        audio.play(Sfx::StepWater);
        audio.play(Sfx::Creeper);
        audio.play(Sfx::JukeboxCat);
        audio.play(Sfx::JukeboxPigstep);
        let mut sfx_buf = vec![0i16; 2048];
        audio.render(&mut sfx_buf);
        assert!(sfx_buf.iter().any(|&s| s != 0), "SFX voices should render audio");
    }
}
