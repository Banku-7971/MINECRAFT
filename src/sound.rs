#![cfg_attr(not(windows), allow(dead_code))]
//! Procedural sound: a software mixer with Minecraft-style material footsteps,
//! jump/land, mob sounds, combat SFX, and a calm looping soundtrack.
//! Pure Rust, zero external assets, works on Windows (WinMM) and Web (WebAudio queue).

use std::collections::VecDeque;
use std::sync::{OnceLock, Mutex};

pub const SAMPLE_RATE: u32 = 22050;

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
    pos: usize,
    volume: f32,
}

pub struct Audio {
    pub music_on: bool,
    pub sound_on: bool,
    music: Vec<i16>,
    music_pos: usize,
    voices: Vec<Voice>,
    sfx: Vec<(Sfx, Vec<i16>)>,
    master: f32,
}

impl Audio {
    pub fn new() -> Self {
        let sfx = vec![
            (Sfx::Click, synth_click()),
            (Sfx::Place, synth_thud(185.0, 0.09, 0.50)),
            (Sfx::Break, synth_break()),
            (Sfx::Step, synth_step_grass()),
            (Sfx::StepGrass, synth_step_grass()),
            (Sfx::StepStone, synth_step_stone()),
            (Sfx::StepWood, synth_step_wood()),
            (Sfx::StepSand, synth_step_sand()),
            (Sfx::StepWater, synth_noise(0.16, 0.24, 700.0)),
            (Sfx::Jump, synth_jump()),
            (Sfx::Land, synth_thud(95.0, 0.12, 0.42)),
            (Sfx::Hurt, synth_hurt()),
            (Sfx::MobHurt, synth_mob_hurt()),
            (Sfx::MobDie, synth_mob_die()),
            (Sfx::Pickup, synth_blip(880.0, 0.07)),
            (Sfx::Craft, synth_blip(660.0, 0.09)),
            (Sfx::Furnace, synth_noise(0.25, 0.20, 900.0)),
            (Sfx::Explode, synth_explode()),
            (Sfx::Splash, synth_noise(0.30, 0.25, 500.0)),
            (Sfx::Zombie, synth_growl(105.0, 0.55)),
            (Sfx::Skeleton, synth_rattle()),
            (Sfx::Cow, synth_cow()),
            (Sfx::Sheep, synth_sheep()),
            (Sfx::Pig, synth_pig()),
            (Sfx::Chicken, synth_chicken()),
            (Sfx::Spider, synth_spider()),
            (Sfx::Creeper, synth_noise(0.45, 0.34, 1400.0)),
            (Sfx::Enderman, synth_enderman()),
            (Sfx::Door, synth_thud(90.0, 0.18, 0.40)),
            (Sfx::LevelUp, synth_arp()),
            (Sfx::ShieldBlock, synth_thud(310.0, 0.14, 0.52)),
            (Sfx::Totem, synth_chime_chord(&[0, 4, 7, 12, 16], 1.1)),
            (Sfx::Beacon, synth_chime_chord(&[-12, 0, 7, 12], 1.4)),
            (Sfx::Wither, synth_growl(68.0, 0.95)),
            (Sfx::JukeboxCat, synth_jukebox(false)),
            (Sfx::JukeboxPigstep, synth_jukebox(true)),
        ];
        Audio {
            music_on: true,
            sound_on: true,
            music: render_music(),
            music_pos: 0,
            voices: Vec::new(),
            sfx,
            master: 0.85,
        }
    }

    pub fn play(&mut self, s: Sfx) {
        if !self.sound_on { return; }
        if let Some(custom) = crate::packs::custom_sound_bytes(s as u8) {
            if let Some(pcm) = decode_wav_pcm(&custom) {
                if self.voices.len() < 16 {
                    self.voices.push(Voice { data: pcm, pos: 0, volume: 0.72 });
                }
                return;
            }
        }
        if let Some((_, data)) = self.sfx.iter().find(|(k, _)| *k == s) {
            if self.voices.len() < 16 {
                self.voices.push(Voice { data: data.clone(), pos: 0, volume: 0.58 });
            }
        }
    }

    /// Fill an output buffer (mono i16) with mixed music + effects.
    pub fn render(&mut self, out: &mut [i16]) {
        let n = out.len();
        for i in 0..n {
            let mut v = 0.0f32;
            if self.music_on && !self.music.is_empty() {
                v += self.music[self.music_pos] as f32 / 32768.0 * 0.58;
                self.music_pos += 1;
                if self.music_pos >= self.music.len() { self.music_pos = 0; }
            }
            for vc in self.voices.iter_mut() {
                if vc.pos < vc.data.len() {
                    v += vc.data[vc.pos] as f32 / 32768.0 * vc.volume;
                    vc.pos += 1;
                }
            }
            self.voices.retain(|v| v.pos < v.data.len());
            let s = (v * self.master).clamp(-1.0, 1.0);
            out[i] = (s * 32000.0) as i16;
        }
    }
}

fn env(t: f32, attack: f32, decay: f32) -> f32 {
    if t < attack { t / attack } else { (-(t - attack) / decay).exp() }
}

fn synth_thud(freq: f32, dur: f32, vol: f32) -> Vec<i16> {
    let n = (dur * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut noise_state = 12345u32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        noise_state = noise_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((noise_state >> 16) as f32 / 32768.0) - 1.0;
        let body = (t * freq * 6.283 * (1.0 - t * 3.0).max(0.2)).sin();
        let s = (body * 0.7 + noise * 0.3) * env(t, 0.004, dur * 0.5) * vol;
        out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
    }
    out
}

fn synth_step_grass() -> Vec<i16> {
    let n = (0.085 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut st = 5151u32;
    let mut prev = 0.0f32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        prev = prev * 0.55 + noise * 0.45;
        let thump = (t * 105.0 * 6.283).sin() * 0.35;
        let s = (prev * 0.8 + thump) * env(t, 0.004, 0.032) * 0.34;
        out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
    }
    out
}

fn synth_step_stone() -> Vec<i16> {
    let n = (0.070 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut st = 9191u32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        let tap = (t * 420.0 * 6.283).sin() * 0.55 + (t * 890.0 * 6.283).sin() * 0.25;
        let s = (tap + noise * 0.35) * env(t, 0.001, 0.018) * 0.38;
        out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
    }
    out
}

fn synth_step_wood() -> Vec<i16> {
    let n = (0.080 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let knock = (t * 215.0 * 6.283).sin() * 0.72 + (t * 430.0 * 6.283).sin() * 0.25;
        let s = knock * env(t, 0.002, 0.025) * 0.40;
        out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
    }
    out
}

fn synth_step_sand() -> Vec<i16> {
    let n = (0.105 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut st = 31337u32;
    let mut last = 0.0f32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        let hp = noise - last * 0.65;
        last = noise;
        let s = hp * env(t, 0.006, 0.042) * 0.30;
        out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
    }
    out
}

fn synth_jump() -> Vec<i16> {
    let n = (0.11 * SAMPLE_RATE as f32) as usize;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let f = 140.0 + t * 1400.0;
        let s = (t * f * 6.283).sin() * env(t, 0.005, 0.045) * 0.26;
        (s.clamp(-1.0, 1.0) * 32000.0) as i16
    }).collect()
}

fn synth_click() -> Vec<i16> {
    let n = (0.045 * SAMPLE_RATE as f32) as usize;
    let mut st = 1093u32;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        let tone = (t * 980.0 * 6.283).sin() * 0.65 + (t * 1950.0 * 6.283).sin() * 0.25 + noise * 0.20;
        let s = tone * env(t, 0.001, 0.008) * 0.34;
        (s.clamp(-1.0, 1.0) * 32000.0) as i16
    }).collect()
}

fn synth_break() -> Vec<i16> {
    let n = (0.19 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut st = 987654321u32;
    let mut lp = 0.0f32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1103515245).wrapping_add(12345);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        lp = lp * 0.45 + noise * 0.55;
        let crack = (t * 190.0 * 6.283 * (1.0 - t * 2.5).max(0.3)).sin() * 0.42;
        let grit = lp * 0.65 + (noise - lp) * 0.35;
        let s = (grit + crack) * env(t, 0.002, 0.055) * 0.48;
        out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
    }
    out
}

fn synth_noise(dur: f32, vol: f32, _tone: f32) -> Vec<i16> {
    let n = (dur * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut st = 24680u32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        let s = noise * env(t, 0.01, dur * 0.4) * vol;
        out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
    }
    out
}

fn synth_hurt() -> Vec<i16> {
    let n = (0.17 * SAMPLE_RATE as f32) as usize;
    let mut st = 8080u32;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        let f = 185.0 - t * 420.0;
        let thud = (t * f.max(70.0) * 6.283).sin() * 0.55 + noise * 0.45;
        let s = thud * env(t, 0.002, 0.05) * 0.52;
        (s.clamp(-1.0, 1.0) * 32000.0) as i16
    }).collect()
}

fn synth_mob_hurt() -> Vec<i16> {
    let n = (0.16 * SAMPLE_RATE as f32) as usize;
    let mut st = 4141u32;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        let f = 260.0 - t * 650.0;
        let smack = (t * f.max(90.0) * 6.283).sin() * 0.65 + noise * 0.35;
        let s = smack * env(t, 0.002, 0.055) * 0.52;
        (s.clamp(-1.0, 1.0) * 32000.0) as i16
    }).collect()
}

fn synth_mob_die() -> Vec<i16> {
    let n = (0.28 * SAMPLE_RATE as f32) as usize;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let f = 360.0 * (-t * 4.5).exp() + 70.0;
        let s = (t * f * 6.283).sin() * env(t, 0.004, 0.11) * 0.45;
        (s.clamp(-1.0, 1.0) * 32000.0) as i16
    }).collect()
}

fn synth_blip(freq: f32, dur: f32) -> Vec<i16> {
    // Crisp Minecraft-style item pop / chime
    let n = (dur * SAMPLE_RATE as f32) as usize;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let f = freq * (1.0 + t * 5.0);
        let s = ((t * f * 6.283).sin() * 0.78 + (t * f * 2.0 * 6.283).sin() * 0.22)
            * env(t, 0.001, dur * 0.42) * 0.34;
        (s.clamp(-1.0, 1.0) * 32000.0) as i16
    }).collect()
}

fn synth_growl(base: f32, dur: f32) -> Vec<i16> {
    let n = (dur * SAMPLE_RATE as f32) as usize;
    let mut st = 777u32;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        let wob = (t * 22.0 * 6.283).sin() * 12.0;
        let tone = ((base + wob) * t * 6.283).sin();
        let s = (tone * 0.7 + noise * 0.3) * env(t, 0.03, dur * 0.45) * 0.35;
        (s.clamp(-1.0, 1.0) * 32000.0) as i16
    }).collect()
}

fn synth_cow() -> Vec<i16> {
    let n = (0.52 * SAMPLE_RATE as f32) as usize;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let f = 148.0 - (t * 3.0).sin() * 18.0 - t * 25.0;
        let tone = (t * f * 6.283).sin() * 0.65 + (t * f * 2.0 * 6.283).sin() * 0.25;
        let s = tone * env(t, 0.05, 0.22) * 0.36;
        (s.clamp(-1.0, 1.0) * 32000.0) as i16
    }).collect()
}

fn synth_sheep() -> Vec<i16> {
    let n = (0.38 * SAMPLE_RATE as f32) as usize;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let vib = (t * 28.0 * 6.283).sin() * 24.0;
        let f = 290.0 + vib;
        let tone = (t * f * 6.283).sin() * 0.6 + (t * f * 2.0 * 6.283).sin() * 0.25;
        let s = tone * env(t, 0.025, 0.16) * 0.32;
        (s.clamp(-1.0, 1.0) * 32000.0) as i16
    }).collect()
}

fn synth_pig() -> Vec<i16> {
    let n = (0.24 * SAMPLE_RATE as f32) as usize;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let f = 195.0 + (t * 45.0 * 6.283).sin() * 35.0;
        let tone = (t * f * 6.283).sin() * 0.7 + (t * f * 1.5 * 6.283).sin() * 0.2;
        let s = tone * env(t, 0.015, 0.09) * 0.35;
        (s.clamp(-1.0, 1.0) * 32000.0) as i16
    }).collect()
}

fn synth_chicken() -> Vec<i16> {
    let n = (0.18 * SAMPLE_RATE as f32) as usize;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let f = if t < 0.08 { 680.0 + t * 1400.0 } else { 540.0 + (t - 0.08) * 1100.0 };
        let s = (t * f * 6.283).sin() * env(t, 0.008, 0.05) * 0.28;
        (s.clamp(-1.0, 1.0) * 32000.0) as i16
    }).collect()
}

fn synth_rattle() -> Vec<i16> {
    let n = (0.20 * SAMPLE_RATE as f32) as usize;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let pulse = if ((t * 36.0) as i32) % 2 == 0 { 1.0 } else { 0.15 };
        let s = (t * 620.0 * 6.283).sin() * pulse * env(t, 0.005, 0.07) * 0.30;
        (s.clamp(-1.0, 1.0) * 32000.0) as i16
    }).collect()
}

fn synth_spider() -> Vec<i16> {
    let n = (0.22 * SAMPLE_RATE as f32) as usize;
    let mut st = 54321u32;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        let modl = (t * 50.0 * 6.283).sin() * 0.5 + 0.5;
        let s = (noise * 0.5 + (t * 920.0 * 6.283).sin() * 0.3) * modl * env(t, 0.01, 0.09) * 0.30;
        (s.clamp(-1.0, 1.0) * 32000.0) as i16
    }).collect()
}

fn synth_enderman() -> Vec<i16> {
    let n = (0.36 * SAMPLE_RATE as f32) as usize;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let f1 = 220.0 + (t * 14.0 * 6.283).sin() * 90.0;
        let f2 = 445.0 - t * 240.0;
        let s = ((t * f1 * 6.283).sin() * 0.5 + (t * f2 * 6.283).sin() * 0.4) * env(t, 0.02, 0.15) * 0.32;
        (s.clamp(-1.0, 1.0) * 32000.0) as i16
    }).collect()
}

fn synth_explode() -> Vec<i16> {
    let n = (0.8 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut st = 4242u32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1103515245).wrapping_add(12345);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        let low = (t * 60.0 * 6.283).sin() * (-t * 4.0).exp();
        let s = (noise * 0.6 + low * 0.4) * env(t, 0.005, 0.25) * 0.7;
        out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
    }
    out
}

fn synth_arp() -> Vec<i16> {
    let notes = [659.25, 830.61, 987.77, 1318.51];
    let per = (0.08 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::new();
    for (k, f) in notes.iter().enumerate() {
        for i in 0..per {
            let t = i as f32 / SAMPLE_RATE as f32;
            let tone = (t * f * 6.283).sin() * 0.75 + (t * f * 2.0 * 6.283).sin() * 0.25;
            let tail = if k == 3 { 0.14 } else { 0.07 };
            let s = tone * env(t, 0.002, tail) * 0.30;
            out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
        }
    }
    out
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
            if fmt != 1 { return None; }
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
    if mono.is_empty() { return None; }
    if rate == SAMPLE_RATE { return Some(mono); }
    let out_len = ((mono.len() as f64) * (SAMPLE_RATE as f64 / rate as f64)).round().max(1.0) as usize;
    let mut resampled = Vec::with_capacity(out_len);
    for idx in 0..out_len {
        let src = (idx as f64 * rate as f64 / SAMPLE_RATE as f64) as usize;
        resampled.push(mono[src.min(mono.len() - 1)]);
    }
    Some(resampled)
}

fn note_hz(semi: i32) -> f32 {
    440.0 * 2f32.powf((semi as f32 - 9.0) / 12.0)
}

fn synth_chime_chord(semis: &[i32], dur: f32) -> Vec<i16> {
    let n = (dur * SAMPLE_RATE as f32) as usize;
    let mut out = vec![0i16; n];
    for (i, s) in out.iter_mut().enumerate() {
        let t = i as f32 / SAMPLE_RATE as f32;
        let a = env(t, 0.02, dur * 0.45);
        let mut acc = 0.0f32;
        for (k, &semi) in semis.iter().enumerate() {
            let delay = k as f32 * 0.04;
            if t >= delay {
                let f = note_hz(semi);
                acc += ((t - delay) * f * 6.283).sin() * 0.28 + ((t - delay) * f * 2.0 * 6.283).sin() * 0.08;
            }
        }
        *s = (acc * a * 18000.0).clamp(-32000.0, 32000.0) as i16;
    }
    out
}

fn synth_jukebox(pigstep: bool) -> Vec<i16> {
    let dur = 7.2f32;
    let n = (dur * SAMPLE_RATE as f32) as usize;
    let mut out = vec![0.0f32; n];
    let notes_cat: [i32; 24] = [0, 4, 7, 12, 11, 7, 4, 0, 2, 5, 9, 14, 12, 9, 5, 2, 4, 7, 11, 16, 14, 12, 9, 7];
    let notes_pig: [i32; 24] = [-12, -5, 0, -5, 3, -5, 0, -5, -10, -3, 2, -3, 5, 2, -3, 0, -8, -1, 4, -1, 7, 4, 0, -5];
    let seq = if pigstep { &notes_pig } else { &notes_cat };
    let step = dur / seq.len() as f32;
    for (idx, &semi) in seq.iter().enumerate() {
        let start = (idx as f32 * step * SAMPLE_RATE as f32) as usize;
        let len = (step * 1.15 * SAMPLE_RATE as f32) as usize;
        let f = note_hz(semi);
        let bass = note_hz(semi - 12);
        for j in 0..len {
            if start + j >= n { break; }
            let t = j as f32 / SAMPLE_RATE as f32;
            let a = env(t, 0.01, step * 0.55);
            let lead = if pigstep {
                (t * f * 6.283).sin().signum() * 0.35 + (t * bass * 6.283).sin() * 0.45
            } else {
                (t * f * 6.283).sin() * 0.55 + (t * f * 2.0 * 6.283).sin() * 0.18
            };
            out[start + j] += lead * a * 0.32;
        }
    }
    out.iter().map(|v| (v.clamp(-1.0, 1.0) * 26000.0) as i16).collect()
}

/// Calm C418-inspired looping ambient soundtrack (24-bar felt piano + sub-pad + shimmer + hall reverb).
fn render_music() -> Vec<i16> {
    const BPM: f32 = 58.0;
    let beat = 60.0 / BPM;
    let step = beat;
    let bars = 24usize;
    let steps = bars * 4;
    let total = (steps as f32 * step * SAMPLE_RATE as f32) as usize;
    let mut buf = vec![0.0f32; total];

    let chords: [[i32; 3]; 8] = [
        [-7, -3, 0],
        [-12, -5, 0],
        [-3, 0, 4],
        [-7, -3, 2],
        [-10, -3, 0],
        [-5, -1, 2],
        [-8, -3, 0],
        [-5, 0, 4],
    ];
    // Sparse contemplative melody (-99 = rest)
    let melody: [i32; 32] = [
        12, 7, -99, 4, 9, -99, 7, -99,
        16, 12, -99, 9, 7, -99, 4, 0,
        7, 9, -99, 12, 7, -99, 4, -99,
        14, 12, 9, -99, 7, 4, 0, -99,
    ];

    let add_tone = |buf: &mut Vec<f32>, start: usize, freq: f32, dur: f32, vol: f32, bright: f32| {
        let n = (dur * SAMPLE_RATE as f32) as usize;
        for i in 0..n {
            let idx = start + i;
            if idx >= buf.len() { break; }
            let t = i as f32 / SAMPLE_RATE as f32;
            let a = env(t, 0.040, dur * 0.45);
            let s = (t * freq * 6.283).sin() * 0.74
                + (t * freq * 2.0 * 6.283).sin() * 0.14 * bright
                + (t * freq * 0.5 * 6.283).sin() * 0.12;
            buf[idx] += s * a * vol;
        }
    };

    for s in 0..steps {
        let start = (s as f32 * step * SAMPLE_RATE as f32) as usize;
        let chord = chords[(s / 8) % chords.len()];
        if s % 8 == 0 {
            for (ci, c) in chord.iter().enumerate() {
                add_tone(&mut buf, start + ci * 950, note_hz(c - 12), 4.2, 0.16, 0.25);
            }
        }
        if s % 2 == 0 {
            let m = melody[(s / 2) % melody.len()];
            if m != -99 {
                add_tone(&mut buf, start, note_hz(m), 2.8, 0.18, 0.50);
            }
        }
    }

    // Simple warm multi-tap room reverb + lowpass felt filter
    let taps = [(1900usize, 0.26f32), (4300, 0.18), (7900, 0.12), (12500, 0.07)];
    let dry = buf.clone();
    for (delay, gain) in taps {
        for i in delay..buf.len() {
            buf[i] += dry[i - delay] * gain;
        }
    }

    let mut prev = 0.0f32;
    for v in buf.iter_mut() {
        prev = prev * 0.70 + *v * 0.30;
        *v = prev;
    }
    buf.iter().map(|v| (v.clamp(-1.0, 1.0) * 24000.0) as i16).collect()
}
