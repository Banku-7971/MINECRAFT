#![cfg_attr(not(windows), allow(dead_code))]
//! 100% Original Procedural Audio & Music Engine:
//! A zero-dependency software mixer with pitch-varied Foley sound effects,
//! bespoke mob vocal formants, original Jukebox grooves, and three original
//! dimension soundtracks (Overworld, Nether, The End).
//! Pure Rust, zero external audio assets, works on Windows (WinMM) and Web (WebAudio).

use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

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
    music_tracks: [Vec<i16>; 3],
    music_dim: usize,
    music_pos: usize,
    voices: Vec<Voice>,
    sfx: Vec<(Sfx, Vec<i16>)>,
    master: f32,
    rng: u32,
}

impl Audio {
    pub fn new() -> Self {
        let sfx = vec![
            (Sfx::Click, synth_click()),
            (Sfx::Place, synth_place()),
            (Sfx::Break, synth_break()),
            (Sfx::Step, synth_step_grass()),
            (Sfx::StepGrass, synth_step_grass()),
            (Sfx::StepStone, synth_step_stone()),
            (Sfx::StepWood, synth_step_wood()),
            (Sfx::StepSand, synth_step_sand()),
            (Sfx::StepWater, synth_step_water()),
            (Sfx::Jump, synth_jump()),
            (Sfx::Land, synth_land()),
            (Sfx::Hurt, synth_hurt()),
            (Sfx::MobHurt, synth_mob_hurt()),
            (Sfx::MobDie, synth_mob_die()),
            (Sfx::Pickup, synth_pickup()),
            (Sfx::Craft, synth_craft()),
            (Sfx::Furnace, synth_furnace()),
            (Sfx::Explode, synth_explode()),
            (Sfx::Splash, synth_splash()),
            (Sfx::Zombie, synth_growl(102.0, 0.58)),
            (Sfx::Skeleton, synth_rattle()),
            (Sfx::Cow, synth_cow()),
            (Sfx::Sheep, synth_sheep()),
            (Sfx::Pig, synth_pig()),
            (Sfx::Chicken, synth_chicken()),
            (Sfx::Spider, synth_spider()),
            (Sfx::Creeper, synth_creeper()),
            (Sfx::Enderman, synth_enderman()),
            (Sfx::Door, synth_door()),
            (Sfx::LevelUp, synth_level_up()),
            (Sfx::ShieldBlock, synth_shield_block()),
            (Sfx::Totem, synth_chime_chord(&[0, 4, 7, 11, 14, 19], 1.25)),
            (Sfx::Beacon, synth_chime_chord(&[-12, 0, 7, 12, 19], 1.55)),
            (Sfx::Wither, synth_growl(64.0, 0.96)),
            (Sfx::JukeboxCat, synth_jukebox_meadow()),
            (Sfx::JukeboxPigstep, synth_jukebox_basalt()),
            (Sfx::Villager, synth_villager()),
            (Sfx::Wolf, synth_wolf()),
            (Sfx::Bee, synth_bee()),
            (Sfx::Warden, synth_warden()),
        ];
        Audio {
            music_on: true,
            sound_on: true,
            music_tracks: [
                render_music_overworld(),
                render_music_nether(),
                render_music_end(),
            ],
            music_dim: 0,
            music_pos: 0,
            voices: Vec::new(),
            sfx,
            master: 0.86,
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
            _ => 0.91 + self.next_rand() * 0.18,
        };
        ((pitch * 65536.0) as u64).max(32768)
    }

    pub fn play(&mut self, s: Sfx) {
        if !self.sound_on {
            return;
        }
        let step_fp = self.pitch_step(s);
        if let Some(custom) = crate::packs::custom_sound_bytes(s as u8) {
            if let Some(pcm) = decode_wav_pcm(&custom) {
                if self.voices.len() < 16 {
                    self.voices.push(Voice {
                        data: pcm,
                        pos_fp: 0,
                        step_fp,
                        volume: 0.72,
                    });
                }
                return;
            }
        }
        if let Some((_, data)) = self.sfx.iter().find(|(k, _)| *k == s) {
            if self.voices.len() < 16 {
                let vol = match s {
                    Sfx::JukeboxCat | Sfx::JukeboxPigstep => 0.72,
                    Sfx::Explode => 0.68,
                    _ => 0.58,
                };
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
        for sample_out in out.iter_mut() {
            let mut v = 0.0f32;
            if self.music_on && tlen > 0 {
                v += track[self.music_pos] as f32 / 32768.0 * 0.60;
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
            // Soft tanh-like saturation prevents clipping when many voices overlap
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

#[inline]
fn env(t: f32, attack: f32, decay: f32) -> f32 {
    if t < attack {
        t / attack.max(1e-5)
    } else {
        (-(t - attack) / decay.max(1e-5)).exp()
    }
}

#[inline]
fn note_hz(semi: i32) -> f32 {
    440.0 * 2f32.powf((semi as f32 - 9.0) / 12.0)
}

fn synth_place() -> Vec<i16> {
    let n = (0.095 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut st = 12345u32;
    let mut lp = 0.0f32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        lp = lp * 0.62 + noise * 0.38;
        let f = 175.0 * (1.0 - t * 4.5).max(0.45);
        let body = (t * f * 6.283).sin() * 0.65 + (t * f * 2.1 * 6.283).sin() * 0.18;
        let click = lp * 0.42 * (-t * 75.0).exp();
        let s = (body * env(t, 0.002, 0.036) + click) * 0.54;
        out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
    }
    out
}

fn synth_land() -> Vec<i16> {
    let n = (0.14 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut st = 54321u32;
    let mut lp = 0.0f32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        lp = lp * 0.72 + noise * 0.28;
        let f = 92.0 * (1.0 - t * 3.0).max(0.5);
        let s = ((t * f * 6.283).sin() * 0.68 + lp * 0.45) * env(t, 0.004, 0.055) * 0.48;
        out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
    }
    out
}

fn synth_step_grass() -> Vec<i16> {
    let n = (0.090 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut st = 5151u32;
    let mut prev = 0.0f32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        prev = prev * 0.58 + noise * 0.42;
        let thump = (t * 108.0 * 6.283).sin() * 0.32;
        let s = (prev * 0.78 + thump) * env(t, 0.005, 0.032) * 0.36;
        out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
    }
    out
}

fn synth_step_stone() -> Vec<i16> {
    let n = (0.072 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut st = 9191u32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        let tap = (t * 410.0 * 6.283).sin() * 0.50 + (t * 920.0 * 6.283).sin() * 0.25;
        let s = (tap + noise * 0.35) * env(t, 0.001, 0.017) * 0.38;
        out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
    }
    out
}

fn synth_step_wood() -> Vec<i16> {
    let n = (0.082 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let knock = (t * 210.0 * 6.283).sin() * 0.70 + (t * 425.0 * 6.283).sin() * 0.26;
        let s = knock * env(t, 0.002, 0.024) * 0.40;
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
        let hp = noise - last * 0.68;
        last = noise;
        let s = hp * env(t, 0.007, 0.040) * 0.32;
        out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
    }
    out
}

fn synth_step_water() -> Vec<i16> {
    let n = (0.18 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut st = 6161u32;
    let mut bp = 0.0f32;
    let mut lp = 0.0f32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        lp = lp * 0.78 + noise * 0.22;
        bp = bp * 0.55 + (noise - lp) * 0.45;
        let bubble = (t * (250.0 + t * 1800.0) * 6.283).sin() * (-t * 28.0).exp() * 0.42;
        let s = (bp * 0.55 + bubble) * env(t, 0.008, 0.065) * 0.44;
        out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
    }
    out
}

fn synth_splash() -> Vec<i16> {
    let n = (0.38 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut st = 88421u32;
    let mut lp = 0.0f32;
    let bubbles = [(0.02f32, 290.0f32), (0.07, 420.0), (0.13, 360.0), (0.19, 540.0)];
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        lp = lp * 0.70 + noise * 0.30;
        let spray = (noise - lp * 0.6) * env(t, 0.01, 0.14) * 0.42;
        let mut bloop = 0.0f32;
        for &(bt, bf) in &bubbles {
            if t >= bt {
                let dt = t - bt;
                if dt < 0.07 {
                    let f = bf + dt * 2100.0;
                    bloop += (dt * f * 6.283).sin() * (-dt * 42.0).exp() * 0.28;
                }
            }
        }
        let s = (spray + bloop) * 0.65;
        out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
    }
    out
}

fn synth_jump() -> Vec<i16> {
    let n = (0.11 * SAMPLE_RATE as f32) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let f = 145.0 + t * 1350.0;
            let s = (t * f * 6.283).sin() * env(t, 0.005, 0.044) * 0.25;
            (s.clamp(-1.0, 1.0) * 32000.0) as i16
        })
        .collect()
}

fn synth_click() -> Vec<i16> {
    let n = (0.042 * SAMPLE_RATE as f32) as usize;
    let mut st = 1093u32;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            st = st.wrapping_mul(1664525).wrapping_add(1013904223);
            let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
            let tone = (t * 960.0 * 6.283).sin() * 0.65 + (t * 1920.0 * 6.283).sin() * 0.24 + noise * 0.18;
            let s = tone * env(t, 0.001, 0.008) * 0.34;
            (s.clamp(-1.0, 1.0) * 32000.0) as i16
        })
        .collect()
}

fn synth_break() -> Vec<i16> {
    let n = (0.20 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut st = 987654321u32;
    let mut lp = 0.0f32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1103515245).wrapping_add(12345);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        lp = lp * 0.45 + noise * 0.55;
        let crack = (t * 185.0 * 6.283 * (1.0 - t * 2.5).max(0.3)).sin() * 0.44;
        let grit = lp * 0.65 + (noise - lp) * 0.35;
        let s = (grit + crack) * env(t, 0.002, 0.056) * 0.50;
        out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
    }
    out
}

fn synth_furnace() -> Vec<i16> {
    let n = (0.34 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut st = 49157u32;
    let mut lp = 0.0f32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1664525).wrapping_add(1013904223);
        let r = (st >> 16) as f32 / 65536.0;
        let noise = r * 2.0 - 1.0;
        lp = lp * 0.88 + noise * 0.12;
        let crackle = if r > 0.985 { (r - 0.985) * 48.0 } else { 0.0 };
        let s = (lp * 0.65 + crackle * 0.45) * env(t, 0.02, 0.15) * 0.42;
        out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
    }
    out
}

fn synth_creeper() -> Vec<i16> {
    // Unmistakable tense rising fuse sizzle ("sssSSS...")
    let n = (0.58 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut st = 24680u32;
    let mut hp_prev = 0.0f32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        let hp = noise - hp_prev * 0.72;
        hp_prev = noise;
        let rise = (0.35 + (t / 0.58) * 0.65).min(1.0);
        let whistle = (t * (1650.0 + t * 1400.0) * 6.283).sin() * 0.14;
        let s = (hp * 0.68 + whistle) * rise * env(t, 0.02, 0.35) * 0.44;
        out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
    }
    out
}

fn synth_hurt() -> Vec<i16> {
    let n = (0.17 * SAMPLE_RATE as f32) as usize;
    let mut st = 8080u32;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            st = st.wrapping_mul(1664525).wrapping_add(1013904223);
            let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
            let f = 185.0 - t * 420.0;
            let thud = (t * f.max(70.0) * 6.283).sin() * 0.55 + noise * 0.45;
            let s = thud * env(t, 0.002, 0.05) * 0.52;
            (s.clamp(-1.0, 1.0) * 32000.0) as i16
        })
        .collect()
}

fn synth_mob_hurt() -> Vec<i16> {
    let n = (0.16 * SAMPLE_RATE as f32) as usize;
    let mut st = 4141u32;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            st = st.wrapping_mul(1664525).wrapping_add(1013904223);
            let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
            let f = 260.0 - t * 650.0;
            let smack = (t * f.max(90.0) * 6.283).sin() * 0.65 + noise * 0.35;
            let s = smack * env(t, 0.002, 0.055) * 0.52;
            (s.clamp(-1.0, 1.0) * 32000.0) as i16
        })
        .collect()
}

fn synth_mob_die() -> Vec<i16> {
    let n = (0.28 * SAMPLE_RATE as f32) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let f = 360.0 * (-t * 4.5).exp() + 70.0;
            let s = (t * f * 6.283).sin() * env(t, 0.004, 0.11) * 0.45;
            (s.clamp(-1.0, 1.0) * 32000.0) as i16
        })
        .collect()
}

fn synth_pickup() -> Vec<i16> {
    let n = (0.075 * SAMPLE_RATE as f32) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let f = 760.0 + t * 5200.0;
            let s = ((t * f * 6.283).sin() * 0.78 + (t * f * 2.0 * 6.283).sin() * 0.22)
                * env(t, 0.001, 0.028)
                * 0.34;
            (s.clamp(-1.0, 1.0) * 32000.0) as i16
        })
        .collect()
}

fn synth_craft() -> Vec<i16> {
    let n = (0.14 * SAMPLE_RATE as f32) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let tap1 = (t * 420.0 * 6.283).sin() * (-t * 45.0).exp() * 0.48;
            let tap2 = if t > 0.045 {
                let dt = t - 0.045;
                ((dt * 680.0 * 6.283).sin() * 0.55 + (dt * 1020.0 * 6.283).sin() * 0.22)
                    * (-dt * 36.0).exp()
            } else {
                0.0
            };
            let s = (tap1 + tap2) * 0.50;
            (s.clamp(-1.0, 1.0) * 32000.0) as i16
        })
        .collect()
}

fn synth_door() -> Vec<i16> {
    let n = (0.18 * SAMPLE_RATE as f32) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let creak = (t * (130.0 + t * 180.0) * 6.283).sin() * 0.55
                + (t * 260.0 * 6.283).sin() * 0.20;
            let latch = if t > 0.09 {
                ((t - 0.09) * 520.0 * 6.283).sin() * (-(t - 0.09) * 65.0).exp() * 0.40
            } else {
                0.0
            };
            let s = (creak * env(t, 0.005, 0.06) + latch) * 0.45;
            (s.clamp(-1.0, 1.0) * 32000.0) as i16
        })
        .collect()
}

fn synth_shield_block() -> Vec<i16> {
    let n = (0.18 * SAMPLE_RATE as f32) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let clang = (t * 320.0 * 6.283).sin() * 0.45
                + (t * 515.0 * 6.283).sin() * 0.35
                + (t * 890.0 * 6.283).sin() * 0.22;
            let s = clang * env(t, 0.001, 0.055) * 0.55;
            (s.clamp(-1.0, 1.0) * 32000.0) as i16
        })
        .collect()
}

fn synth_growl(base: f32, dur: f32) -> Vec<i16> {
    let n = (dur * SAMPLE_RATE as f32) as usize;
    let mut st = 777u32;
    let mut phase = 0.0f32;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            st = st.wrapping_mul(1664525).wrapping_add(1013904223);
            let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
            let f = base * (1.0 - 0.22 * t / dur) + (t * 17.0 * 6.283).sin() * 6.0;
            phase = (phase + f / SAMPLE_RATE as f32).fract();
            let glottal = phase * 2.0 - 1.0;
            let formant = (t * 350.0 * 6.283).sin() * 0.45 + (t * 700.0 * 6.283).sin() * 0.20;
            let s = (glottal * 0.45 + formant * 0.35 + noise * 0.20) * env(t, 0.04, dur * 0.45) * 0.38;
            (s.clamp(-1.0, 1.0) * 32000.0) as i16
        })
        .collect()
}

fn synth_cow() -> Vec<i16> {
    let n = (0.68 * SAMPLE_RATE as f32) as usize;
    let mut phase = 0.0f32;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let f = 138.0 - (t * 2.4).sin() * 14.0 - t * 28.0;
            phase = (phase + f / SAMPLE_RATE as f32).fract();
            let saw = phase * 2.0 - 1.0;
            let f1 = (phase * 2.0 * 6.283).sin() * 0.55 + (phase * 3.0 * 6.283).sin() * 0.30;
            let f2 = (phase * 5.0 * 6.283).sin() * 0.18;
            let s = (saw * 0.25 + f1 * 0.55 + f2 * 0.20) * env(t, 0.06, 0.28) * 0.44;
            (s.clamp(-1.0, 1.0) * 32000.0) as i16
        })
        .collect()
}

fn synth_sheep() -> Vec<i16> {
    let n = (0.46 * SAMPLE_RATE as f32) as usize;
    let mut phase = 0.0f32;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let vib = (t * 28.0 * 6.283).sin() * 22.0;
            let f = 265.0 + vib - t * 35.0;
            phase = (phase + f / SAMPLE_RATE as f32).fract();
            let amp_bleat = 0.78 + 0.22 * (t * 28.0 * 6.283).sin();
            let saw = phase * 2.0 - 1.0;
            let nasal = (phase * 2.0 * 6.283).sin() * 0.42 + (phase * 4.0 * 6.283).sin() * 0.28;
            let s = (saw * 0.35 + nasal * 0.65) * amp_bleat * env(t, 0.03, 0.19) * 0.38;
            (s.clamp(-1.0, 1.0) * 32000.0) as i16
        })
        .collect()
}

fn synth_pig() -> Vec<i16> {
    let n = (0.24 * SAMPLE_RATE as f32) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let f = 195.0 + (t * 45.0 * 6.283).sin() * 35.0;
            let tone = (t * f * 6.283).sin() * 0.7 + (t * f * 1.5 * 6.283).sin() * 0.2;
            let s = tone * env(t, 0.015, 0.09) * 0.35;
            (s.clamp(-1.0, 1.0) * 32000.0) as i16
        })
        .collect()
}

fn synth_chicken() -> Vec<i16> {
    let n = (0.18 * SAMPLE_RATE as f32) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let f = if t < 0.08 {
                680.0 + t * 1400.0
            } else {
                540.0 + (t - 0.08) * 1100.0
            };
            let s = (t * f * 6.283).sin() * env(t, 0.008, 0.05) * 0.28;
            (s.clamp(-1.0, 1.0) * 32000.0) as i16
        })
        .collect()
}

fn synth_rattle() -> Vec<i16> {
    let n = (0.28 * SAMPLE_RATE as f32) as usize;
    let mut out = vec![0i16; n];
    let mut st = 918273u32;
    let hits = [(0.00f32, 1320.0f32), (0.075, 1680.0), (0.16, 1180.0)];
    for &(ht, hf) in &hits {
        let start = (ht * SAMPLE_RATE as f32) as usize;
        let len = (0.045 * SAMPLE_RATE as f32) as usize;
        for j in 0..len {
            if start + j >= n {
                break;
            }
            let t = j as f32 / SAMPLE_RATE as f32;
            st = st.wrapping_mul(1664525).wrapping_add(1013904223);
            let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
            let click = (t * hf * 6.283).sin() * 0.55 + (t * hf * 1.47 * 6.283).sin() * 0.25 + noise * 0.35;
            let val = click * (-t * 78.0).exp() * 0.42;
            out[start + j] = (out[start + j] as i32 + (val * 32000.0) as i32).clamp(-32000, 32000) as i16;
        }
    }
    out
}

fn synth_villager() -> Vec<i16> {
    let n = (0.36 * SAMPLE_RATE as f32) as usize;
    let mut phase = 0.0f32;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let f = 152.0 + (t * 8.0).sin() * 12.0 - t * 22.0;
            phase = (phase + f / SAMPLE_RATE as f32).fract();
            let s = ((phase * 6.283).sin() * 0.55
                + (phase * 2.0 * 6.283).sin() * 0.30
                + (phase * 3.0 * 6.283).sin() * 0.15)
                * env(t, 0.04, 0.15)
                * 0.36;
            (s.clamp(-1.0, 1.0) * 32000.0) as i16
        })
        .collect()
}

fn synth_wolf() -> Vec<i16> {
    let n = (0.20 * SAMPLE_RATE as f32) as usize;
    let mut st = 31337u32;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            st = st.wrapping_mul(1664525).wrapping_add(1013904223);
            let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
            let f = 320.0 - t * 900.0;
            let s = ((t * f.max(130.0) * 6.283).sin() * 0.65 + noise * 0.35) * env(t, 0.008, 0.065) * 0.40;
            (s.clamp(-1.0, 1.0) * 32000.0) as i16
        })
        .collect()
}

fn synth_bee() -> Vec<i16> {
    let n = (0.32 * SAMPLE_RATE as f32) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let f = 195.0 + (t * 14.0 * 6.283).sin() * 14.0;
            let s = ((t * f * 6.283).sin() * 0.55 + (t * f * 2.0 * 6.283).sin() * 0.30)
                * env(t, 0.03, 0.14)
                * 0.24;
            (s.clamp(-1.0, 1.0) * 32000.0) as i16
        })
        .collect()
}

fn synth_warden() -> Vec<i16> {
    let n = (0.72 * SAMPLE_RATE as f32) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let pulse = 0.6 + 0.4 * (t * 6.5 * 6.283).sin();
            let sub = (t * 52.0 * 6.283).sin() * 0.58 + (t * 78.0 * 6.283).sin() * 0.30;
            let sculk = (t * 240.0 * 6.283).sin() * (-t * 5.0).exp() * 0.25;
            let s = (sub * pulse + sculk) * env(t, 0.03, 0.24) * 0.52;
            (s.clamp(-1.0, 1.0) * 32000.0) as i16
        })
        .collect()
}

fn synth_spider() -> Vec<i16> {
    let n = (0.22 * SAMPLE_RATE as f32) as usize;
    let mut st = 54321u32;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            st = st.wrapping_mul(1664525).wrapping_add(1013904223);
            let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
            let modl = (t * 50.0 * 6.283).sin() * 0.5 + 0.5;
            let s = (noise * 0.5 + (t * 920.0 * 6.283).sin() * 0.3) * modl * env(t, 0.01, 0.09) * 0.30;
            (s.clamp(-1.0, 1.0) * 32000.0) as i16
        })
        .collect()
}

fn synth_enderman() -> Vec<i16> {
    let n = (0.36 * SAMPLE_RATE as f32) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let f1 = 220.0 + (t * 14.0 * 6.283).sin() * 90.0;
            let f2 = 445.0 - t * 240.0;
            let s = ((t * f1 * 6.283).sin() * 0.5 + (t * f2 * 6.283).sin() * 0.4)
                * env(t, 0.02, 0.15)
                * 0.32;
            (s.clamp(-1.0, 1.0) * 32000.0) as i16
        })
        .collect()
}

fn synth_explode() -> Vec<i16> {
    let n = (0.88 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut st = 4242u32;
    let mut lp = 0.0f32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1103515245).wrapping_add(12345);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        lp = lp * 0.80 + noise * 0.20;
        let sub = (t * (68.0 - t * 35.0).max(28.0) * 6.283).sin() * (-t * 4.5).exp();
        let s = (lp * 0.62 + noise * 0.18 + sub * 0.48) * env(t, 0.004, 0.28) * 0.72;
        out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
    }
    out
}

fn synth_level_up() -> Vec<i16> {
    // Original sparkling 5-note Major-9th arpeggio
    let notes = [659.25, 830.61, 987.77, 1244.51, 1479.98];
    let per = (0.075 * SAMPLE_RATE as f32) as usize;
    let mut out = vec![0.0f32; per * notes.len() + (0.45 * SAMPLE_RATE as f32) as usize];
    for (k, &f) in notes.iter().enumerate() {
        let start = k * per;
        let dur = if k == notes.len() - 1 { 0.44 } else { 0.22 };
        let len = (dur * SAMPLE_RATE as f32) as usize;
        for i in 0..len {
            if start + i >= out.len() {
                break;
            }
            let t = i as f32 / SAMPLE_RATE as f32;
            let tone = (t * f * 6.283).sin() * 0.65
                + (t * f * 2.0 * 6.283).sin() * 0.24
                + (t * f * 3.0 * 6.283).sin() * 0.08;
            out[start + i] += tone * env(t, 0.003, dur * 0.42) * 0.26;
        }
    }
    out.iter().map(|v| (v.clamp(-1.0, 1.0) * 32000.0) as i16).collect()
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

fn synth_chime_chord(semis: &[i32], dur: f32) -> Vec<i16> {
    let n = (dur * SAMPLE_RATE as f32) as usize;
    let mut out = vec![0i16; n];
    for (i, s) in out.iter_mut().enumerate() {
        let t = i as f32 / SAMPLE_RATE as f32;
        let a = env(t, 0.02, dur * 0.45);
        let mut acc = 0.0f32;
        for (k, &semi) in semis.iter().enumerate() {
            let delay = k as f32 * 0.045;
            if t >= delay {
                let f = note_hz(semi);
                acc += ((t - delay) * f * 6.283).sin() * 0.28
                    + ((t - delay) * f * 2.0 * 6.283).sin() * 0.09;
            }
        }
        *s = (acc * a * 18000.0).clamp(-32000.0, 32000.0) as i16;
    }
    out
}

/// Add a studio felt-piano note (warm fundamental + hammer transient + gentle chorus).
fn add_felt_piano(buf: &mut [f32], start: usize, freq: f32, dur: f32, vol: f32, bright: f32) {
    let n = (dur * SAMPLE_RATE as f32) as usize;
    for i in 0..n {
        let idx = start + i;
        if idx >= buf.len() {
            break;
        }
        let t = i as f32 / SAMPLE_RATE as f32;
        let a = env(t, 0.022, dur * 0.42);
        let hammer = (t * freq * 3.0 * 6.283).sin() * (-t * 28.0).exp() * 0.14 * bright;
        let body = (t * freq * 6.283).sin() * 0.66
            + (t * freq * 1.002 * 6.283).sin() * 0.16
            + (t * freq * 2.0 * 6.283).sin() * 0.14 * bright
            + (t * freq * 0.5 * 6.283).sin() * 0.12;
        buf[idx] += (body + hammer) * a * vol;
    }
}

/// Add a lush warm synth/string pad note.
fn add_warm_pad(buf: &mut [f32], start: usize, freq: f32, dur: f32, vol: f32) {
    let n = (dur * SAMPLE_RATE as f32) as usize;
    for i in 0..n {
        let idx = start + i;
        if idx >= buf.len() {
            break;
        }
        let t = i as f32 / SAMPLE_RATE as f32;
        let a = env(t, dur * 0.22, dur * 0.55);
        let s = (t * freq * 6.283).sin() * 0.52
            + (t * freq * 0.997 * 6.283).sin() * 0.26
            + (t * freq * 1.003 * 6.283).sin() * 0.26;
        buf[idx] += s * a * vol;
    }
}

fn apply_hall_reverb(buf: &mut [f32], warmth: f32) -> Vec<i16> {
    let taps = [
        (1700usize, 0.24f32),
        (3700, 0.18),
        (6900, 0.13),
        (11300, 0.08),
        (16400, 0.05),
    ];
    let dry = buf.to_vec();
    for (delay, gain) in taps {
        for i in delay..buf.len() {
            buf[i] += dry[i - delay] * gain;
        }
    }
    let mut prev = 0.0f32;
    let keep = 1.0 - warmth;
    for v in buf.iter_mut() {
        prev = prev * warmth + *v * keep;
        *v = prev;
    }
    // Gentle fade-in and fade-out at loop boundaries for seamless looping
    let edge = (SAMPLE_RATE as usize).min(buf.len() / 4);
    for i in 0..edge {
        let k = i as f32 / edge as f32;
        buf[i] *= k;
        let j = buf.len() - 1 - i;
        buf[j] *= k;
    }
    buf.iter().map(|v| (v.clamp(-1.0, 1.0) * 23500.0) as i16).collect()
}

/// Original Track 1 (Overworld): "Sunlight & Moss" — D major / B minor / Gmaj7 felt-piano & pad.
fn render_music_overworld() -> Vec<i16> {
    const BPM: f32 = 62.0;
    let beat = 60.0 / BPM;
    let bars = 16usize;
    let steps = bars * 4;
    let total = (steps as f32 * beat * SAMPLE_RATE as f32) as usize;
    let mut buf = vec![0.0f32; total];

    // Dmaj7 -> Bm9 -> Gmaj7 -> A6sus4 -> Em9 -> Bm7 -> G -> A
    // (MIDI-like semitones relative to A4=0: D4=-7, F#4=-3, A4=0, C#5=4, B3=-10, G3=-14, E4=-5)
    let chords: [[i32; 3]; 8] = [
        [-7, -3, 0],   // D
        [-10, -7, -3], // Bm
        [-14, -10, -7],// G
        [-12, -7, -5], // Asus
        [-5, -2, 2],   // Em7
        [-10, -3, 2],  // Bm9
        [-14, -7, 0],  // Gadd9
        [-12, -5, -1], // A6
    ];
    let melody: [i32; 32] = [
        9, 5, -99, 2, 5, -99, 4, -99,
        14, 12, -99, 9, 5, -99, 2, -99,
        5, 9, -99, 12, 9, -99, 5, 2,
        11, 9, 5, -99, 4, 2, 0, -99,
    ];

    for s in 0..steps {
        let start = (s as f32 * beat * SAMPLE_RATE as f32) as usize;
        if s % 8 == 0 {
            let chord = chords[(s / 8) % chords.len()];
            add_warm_pad(&mut buf, start, note_hz(chord[0] - 12), 5.6, 0.13);
            for (ci, &c) in chord.iter().enumerate() {
                add_felt_piano(&mut buf, start + ci * 1300, note_hz(c), 4.6, 0.15, 0.35);
            }
        }
        if s % 2 == 0 {
            let m = melody[(s / 2) % melody.len()];
            if m != -99 {
                add_felt_piano(&mut buf, start, note_hz(m), 3.1, 0.17, 0.62);
            }
        }
    }
    apply_hall_reverb(&mut buf, 0.68)
}

/// Original Track 2 (Nether): "Ember Choir" — dark Phrygian cello/brass drones & distant bells.
fn render_music_nether() -> Vec<i16> {
    const BPM: f32 = 48.0;
    let beat = 60.0 / BPM;
    let steps = 48usize;
    let total = (steps as f32 * beat * SAMPLE_RATE as f32) as usize;
    let mut buf = vec![0.0f32; total];

    let drones: [[i32; 2]; 6] = [
        [-19, -12], // D2 + A2
        [-18, -11], // Eb2 + Bb2
        [-21, -14], // C2 + G2
        [-19, -12], // D2 + A2
        [-16, -9],  // F2 + C3
        [-18, -12], // Eb2 + A2
    ];
    let bells: [i32; 12] = [5, -99, 6, -99, 2, -99, -99, 0, -99, -1, -99, -99];

    for s in 0..steps {
        let start = (s as f32 * beat * SAMPLE_RATE as f32) as usize;
        if s % 8 == 0 {
            let d = drones[(s / 8) % drones.len()];
            add_warm_pad(&mut buf, start, note_hz(d[0]), 9.5, 0.20);
            add_warm_pad(&mut buf, start + 1800, note_hz(d[1]), 8.5, 0.14);
        }
        if s % 4 == 0 {
            let b = bells[(s / 4) % bells.len()];
            if b != -99 {
                add_felt_piano(&mut buf, start, note_hz(b), 4.8, 0.13, 0.45);
            }
        }
    }
    apply_hall_reverb(&mut buf, 0.76)
}

/// Original Track 3 (The End): "Astral Drift" — cosmic Lydian chimes & floating pads.
fn render_music_end() -> Vec<i16> {
    const BPM: f32 = 54.0;
    let beat = 60.0 / BPM;
    let steps = 48usize;
    let total = (steps as f32 * beat * SAMPLE_RATE as f32) as usize;
    let mut buf = vec![0.0f32; total];

    let pads: [[i32; 2]; 6] = [
        [-9, -2],  // C3 + G3
        [-3, 3],   // F#3 + C4
        [-7, 0],   // D3 + A3
        [-5, 2],   // E3 + B3
        [-9, -3],  // C3 + F#3
        [-7, -2],  // D3 + G3
    ];
    let chimes: [i32; 16] = [15, -99, 21, 19, -99, 14, -99, 15, 22, -99, 19, -99, 15, 14, -99, 10];

    for s in 0..steps {
        let start = (s as f32 * beat * SAMPLE_RATE as f32) as usize;
        if s % 8 == 0 {
            let p = pads[(s / 8) % pads.len()];
            add_warm_pad(&mut buf, start, note_hz(p[0]), 8.0, 0.16);
            add_warm_pad(&mut buf, start + 1100, note_hz(p[1]), 7.2, 0.12);
        }
        if s % 3 == 0 {
            let c = chimes[(s / 3) % chimes.len()];
            if c != -99 {
                add_felt_piano(&mut buf, start, note_hz(c), 3.6, 0.14, 0.85);
            }
        }
    }
    apply_hall_reverb(&mut buf, 0.56)
}

/// Original Jukebox Disc 1: "Meadow Promenade" (warm melodic synth + bass + soft brush beat).
fn synth_jukebox_meadow() -> Vec<i16> {
    let steps = 48usize;
    let step_dur = 0.28f32;
    let total = (steps as f32 * step_dur * SAMPLE_RATE as f32) as usize + (0.6 * SAMPLE_RATE as f32) as usize;
    let mut out = vec![0.0f32; total];
    let lead: [i32; 24] = [
        4, 7, 11, 16, 14, 11, 7, 4,
        5, 9, 12, 17, 16, 12, 9, 5,
        7, 11, 14, 19, 16, 14, 11, 7,
    ];
    let bass: [i32; 24] = [
        -20, -13, -8, -13, -20, -13, -8, -13,
        -19, -12, -7, -12, -19, -12, -7, -12,
        -17, -10, -5, -10, -17, -10, -5, -10,
    ];
    let mut st = 40404u32;
    for s in 0..steps {
        let start = (s as f32 * step_dur * SAMPLE_RATE as f32) as usize;
        let lf = note_hz(lead[s % lead.len()]);
        let bf = note_hz(bass[s % bass.len()]);
        let len = (step_dur * 1.25 * SAMPLE_RATE as f32) as usize;
        for j in 0..len {
            if start + j >= out.len() { break; }
            let t = j as f32 / SAMPLE_RATE as f32;
            let a = env(t, 0.008, step_dur * 0.65);
            let mel = ((t * lf * 6.283).sin() * 0.65 + (t * lf * 2.0 * 6.283).sin() * 0.22) * a * 0.24;
            let bs = (t * bf * 6.283).sin() * env(t, 0.006, step_dur * 0.50) * 0.24;
            let mut perc = 0.0f32;
            if s % 2 == 1 && j < 1800 {
                st = st.wrapping_mul(1664525).wrapping_add(1013904223);
                let n = ((st >> 16) as f32 / 32768.0) - 1.0;
                perc = n * (-t * 48.0).exp() * 0.12;
            }
            out[start + j] += mel + bs + perc;
        }
    }
    out.iter().map(|v| (v.clamp(-1.0, 1.0) * 26000.0) as i16).collect()
}

/// Original Jukebox Disc 2: "Basalt Bounce" (funky bass + brassy syncopation + kick/snare).
fn synth_jukebox_basalt() -> Vec<i16> {
    let steps = 48usize;
    let step_dur = 0.22f32;
    let total = (steps as f32 * step_dur * SAMPLE_RATE as f32) as usize + (0.5 * SAMPLE_RATE as f32) as usize;
    let mut out = vec![0.0f32; total];
    let lead: [i32; 16] = [0, -99, 3, 5, -99, 7, 5, 3, 0, -99, 7, 10, -99, 7, 5, 3];
    let bass: [i32; 16] = [-24, -24, -17, -24, -21, -21, -14, -21, -19, -19, -12, -19, -17, -17, -10, -12];
    let mut st = 70707u32;
    for s in 0..steps {
        let start = (s as f32 * step_dur * SAMPLE_RATE as f32) as usize;
        let ln = lead[s % lead.len()];
        let bf = note_hz(bass[s % bass.len()]);
        let len = (step_dur * 1.10 * SAMPLE_RATE as f32) as usize;
        for j in 0..len {
            if start + j >= out.len() { break; }
            let t = j as f32 / SAMPLE_RATE as f32;
            let bs = ((t * bf * 6.283).sin() * 0.7 + (t * bf * 2.0 * 6.283).sin() * 0.25)
                * env(t, 0.004, step_dur * 0.45)
                * 0.30;
            let mel = if ln != -99 {
                let lf = note_hz(ln);
                ((t * lf * 6.283).sin() + (t * lf * 3.0 * 6.283).sin() * 0.25)
                    * env(t, 0.005, step_dur * 0.55)
                    * 0.18
            } else {
                0.0
            };
            let kick = if s % 4 == 0 {
                (t * (110.0 * (-t * 24.0).exp() + 45.0) * 6.283).sin() * (-t * 22.0).exp() * 0.28
            } else {
                0.0
            };
            let snare = if s % 4 == 2 && j < 2400 {
                st = st.wrapping_mul(1664525).wrapping_add(1013904223);
                (((st >> 16) as f32 / 32768.0) - 1.0) * (-t * 38.0).exp() * 0.18
            } else {
                0.0
            };
            out[start + j] += bs + mel + kick + snare;
        }
    }
    out.iter().map(|v| (v.clamp(-1.0, 1.0) * 26000.0) as i16).collect()
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
