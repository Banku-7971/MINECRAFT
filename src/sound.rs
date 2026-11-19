#![cfg_attr(not(windows), allow(dead_code))]
//! Procedural sound: a tiny software mixer with Minecraft-style effects and a
//! calm looping soundtrack. Pure Rust, no assets, so the .exe stays self-contained.

pub const SAMPLE_RATE: u32 = 22050;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sfx {
    Click,
    Place,
    Break,
    Step,
    Hurt,
    Pickup,
    Craft,
    Furnace,
    Explode,
    Splash,
    Zombie,
    Skeleton,
    Cow,
    Sheep,
    Door,
    LevelUp,
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
            (Sfx::Place, synth_thud(180.0, 0.09, 0.5)),
            (Sfx::Break, synth_break()),
            (Sfx::Step, synth_thud(120.0, 0.06, 0.25)),
            (Sfx::Hurt, synth_hurt()),
            (Sfx::Pickup, synth_blip(880.0, 0.07)),
            (Sfx::Craft, synth_blip(660.0, 0.09)),
            (Sfx::Furnace, synth_noise(0.25, 0.2, 900.0)),
            (Sfx::Explode, synth_explode()),
            (Sfx::Splash, synth_noise(0.3, 0.25, 500.0)),
            (Sfx::Zombie, synth_growl(110.0, 0.55)),
            (Sfx::Skeleton, synth_blip(340.0, 0.12)),
            (Sfx::Cow, synth_growl(150.0, 0.4)),
            (Sfx::Sheep, synth_growl(320.0, 0.3)),
            (Sfx::Door, synth_thud(90.0, 0.18, 0.4)),
            (Sfx::LevelUp, synth_arp()),
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
        if let Some((_, data)) = self.sfx.iter().find(|(k, _)| *k == s) {
            if self.voices.len() < 12 {
                self.voices.push(Voice { data: data.clone(), pos: 0, volume: 0.55 });
            }
        }
    }

    /// Fill an output buffer (mono i16) with mixed music + effects.
    pub fn render(&mut self, out: &mut [i16]) {
        let n = out.len();
        for i in 0..n {
            let mut v = 0.0f32;
            if self.music_on && !self.music.is_empty() {
                v += self.music[self.music_pos] as f32 / 32768.0 * 0.55;
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

fn synth_click() -> Vec<i16> {
    let n = (0.05 * SAMPLE_RATE as f32) as usize;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let s = (t * 1400.0 * 6.283).sin() * env(t, 0.001, 0.01) * 0.35;
        (s * 32000.0) as i16
    }).collect()
}

fn synth_break() -> Vec<i16> {
    let n = (0.18 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut st = 987654321u32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        st = st.wrapping_mul(1103515245).wrapping_add(12345);
        let noise = ((st >> 16) as f32 / 32768.0) - 1.0;
        let s = noise * env(t, 0.002, 0.05) * 0.5;
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
    let n = (0.22 * SAMPLE_RATE as f32) as usize;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let f = 320.0 - t * 700.0;
        let s = (t * f.max(80.0) * 6.283).sin() * env(t, 0.005, 0.08) * 0.5;
        (s.clamp(-1.0, 1.0) * 32000.0) as i16
    }).collect()
}

fn synth_blip(freq: f32, dur: f32) -> Vec<i16> {
    let n = (dur * SAMPLE_RATE as f32) as usize;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let sq = if (t * freq * 2.0).fract() < 0.5 { 1.0 } else { -1.0 };
        let s = sq * env(t, 0.002, dur * 0.6) * 0.3;
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
    let notes = [523.25, 659.25, 783.99, 1046.5];
    let per = (0.09 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::new();
    for (k, f) in notes.iter().enumerate() {
        for i in 0..per {
            let t = (i + k * per) as f32 / SAMPLE_RATE as f32;
            let s = (t * f * 6.283).sin() * env(i as f32 / SAMPLE_RATE as f32, 0.004, 0.09) * 0.3;
            out.push((s.clamp(-1.0, 1.0) * 32000.0) as i16);
        }
    }
    out
}

fn note_hz(semi: i32) -> f32 {
    // A4 = 69 -> 440 Hz
    440.0 * 2f32.powf((semi as f32 - 9.0) / 12.0)
}

/// A calm 8-bar loop built from soft piano-ish tones, bass and a little pad.
fn render_music() -> Vec<i16> {
    const BPM: f32 = 72.0;
    let beat = 60.0 / BPM;
    let step = beat / 2.0; // eighth notes
    let bars = 8usize;
    let steps = bars * 8;
    let total = (steps as f32 * step * SAMPLE_RATE as f32) as usize;
    let mut buf = vec![0.0f32; total];

    // chords: Cmaj, Amin, Fmaj, Gmaj, Cmaj, Amin, Fmaj, Gsus
    let chords: [[i32; 3]; 8] = [
        [0, 4, 7], [-3, 0, 4], [-7, -3, 0], [-5, -1, 2],
        [0, 4, 7], [-3, 0, 4], [-7, -3, 0], [-5, -1, 2],
    ];
    let melody: [i32; 32] = [
        12, 7, 4, 7, 9, 7, 4, 0,
        7, 4, 0, 4, 9, 12, 9, 7,
        4, 7, 12, 16, 14, 12, 9, 7,
        7, 9, 12, 9, 7, 4, 0, -1,
    ];

    let add_tone = |buf: &mut Vec<f32>, start: usize, freq: f32, dur: f32, vol: f32, bright: f32| {
        let n = (dur * SAMPLE_RATE as f32) as usize;
        for i in 0..n {
            let idx = start + i;
            if idx >= buf.len() { break; }
            let t = i as f32 / SAMPLE_RATE as f32;
            let a = env(t, 0.02, dur * 0.35);
            let s = (t * freq * 6.283).sin() * 0.7
                + (t * freq * 2.0 * 6.283).sin() * 0.2 * bright
                + (t * freq * 3.0 * 6.283).sin() * 0.08 * bright;
            buf[idx] += s * a * vol;
        }
    };

    for s in 0..steps {
        let start = (s as f32 * step * SAMPLE_RATE as f32) as usize;
        let chord = chords[(s / 8).min(7)];
        if s % 8 == 0 {
            for c in chord.iter() {
                add_tone(&mut buf, start, note_hz(c - 12), 2.1, 0.22, 0.4);
            }
        }
        if s % 2 == 0 {
            let c = chord[(s / 2) % 3] - 24;
            add_tone(&mut buf, start, note_hz(c), 0.85, 0.20, 0.2);
        }
        if s % 2 == 0 {
            let m = melody[(s / 2) % 32];
            add_tone(&mut buf, start, note_hz(m), 1.1, 0.16, 0.7);
        }
    }

    // gentle low-pass to soften the result
    let mut prev = 0.0f32;
    for v in buf.iter_mut() {
        prev = prev * 0.62 + *v * 0.38;
        *v = prev;
    }
    buf.iter().map(|v| (v.clamp(-1.0, 1.0) * 26000.0) as i16).collect()
}

