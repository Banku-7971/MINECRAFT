//! Android platform backend.
//!
//! Driven by `MainActivity.java` via `jni_bridge.c` -> `src/android_export.rs`.
//! Input arrives from the Android UI thread into a mutex-protected queue, the
//! 480x270 framebuffer is converted to Android `ARGB_8888` (`0xAARRGGBB`) on
//! `present`, and `Audio` renders 22050 Hz mono PCM directly for `AudioTrack`.

use crate::platform::{Event, Platform};
use crate::render::{Framebuffer, H, OUT_H, OUT_W, W};
use crate::sound::{Audio, Sfx};
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

static START: OnceLock<Instant> = OnceLock::new();
static EVENTS: OnceLock<Mutex<VecDeque<Event>>> = OnceLock::new();
static MOUSE: OnceLock<Mutex<(i32, i32)>> = OnceLock::new();
static QUIT: OnceLock<Mutex<bool>> = OnceLock::new();
static FRAME_ARGB: OnceLock<Mutex<Vec<i32>>> = OnceLock::new();
static AUDIO: OnceLock<Mutex<Audio>> = OnceLock::new();

fn events_q() -> &'static Mutex<VecDeque<Event>> {
    EVENTS.get_or_init(|| Mutex::new(VecDeque::with_capacity(64)))
}

fn mouse_pos() -> &'static Mutex<(i32, i32)> {
    MOUSE.get_or_init(|| Mutex::new((240, 135)))
}

fn quit_flag() -> &'static Mutex<bool> {
    QUIT.get_or_init(|| Mutex::new(false))
}

fn frame_argb() -> &'static Mutex<Vec<i32>> {
    FRAME_ARGB.get_or_init(|| Mutex::new(vec![0; OUT_W * OUT_H]))
}

fn audio_mixer() -> &'static Mutex<Audio> {
    AUDIO.get_or_init(|| Mutex::new(Audio::new()))
}

pub fn push_event(event: Event) {
    if let Ok(mut q) = events_q().lock() {
        q.push_back(event);
    }
}

pub fn set_mouse(x: i32, y: i32) {
    if let Ok(mut m) = mouse_pos().lock() {
        *m = (x.clamp(0, W as i32 - 1), y.clamp(0, H as i32 - 1));
    }
}

pub fn copy_frame_argb(out: &mut [i32]) {
    if let Ok(buf) = frame_argb().lock() {
        let n = out.len().min(buf.len());
        out[..n].copy_from_slice(&buf[..n]);
    }
}

pub fn render_audio(out: &mut [i16]) {
    if let Ok(mut a) = audio_mixer().lock() {
        a.render(out);
    } else {
        out.fill(0);
    }
}

pub struct Android;

impl Android {
    pub fn new() -> Self {
        let _ = START.get_or_init(Instant::now);
        let _ = audio_mixer();
        Self
    }
}

impl Platform for Android {
    fn poll(&mut self) -> Vec<Event> {
        if let Ok(mut q) = events_q().lock() {
            q.drain(..).collect()
        } else {
            Vec::new()
        }
    }

    fn present(&mut self, fb: &mut Framebuffer) {
        if let Ok(mut out) = frame_argb().lock() {
            let n = (fb.w * fb.h).min(out.len());
            for i in 0..n {
                let s = i * 4;
                let r = fb.rgba[s] as i32;
                let g = fb.rgba[s + 1] as i32;
                let b = fb.rgba[s + 2] as i32;
                let a = fb.rgba[s + 3] as i32;
                out[i] = (a << 24) | (r << 16) | (g << 8) | b;
            }
        }
    }

    fn now(&self) -> f64 {
        START.get_or_init(Instant::now).elapsed().as_secs_f64()
    }

    fn play(&mut self, s: Sfx) {
        if let Ok(mut a) = audio_mixer().lock() {
            a.play(s);
        }
    }

    fn set_music(&mut self, on: bool) {
        if let Ok(mut a) = audio_mixer().lock() {
            a.music_on = on;
        }
    }

    fn set_sound(&mut self, on: bool) {
        if let Ok(mut a) = audio_mixer().lock() {
            a.sound_on = on;
        }
    }

    fn set_cursor(&mut self, _visible: bool) {}

    fn set_capture(&mut self, _captured: bool) {}

    fn mouse(&self) -> (i32, i32) {
        mouse_pos().lock().map(|m| *m).unwrap_or((240, 135))
    }

    fn clipboard(&mut self) -> Option<String> {
        None
    }

    fn set_clipboard(&mut self, _text: &str) {}

    fn should_quit(&self) -> bool {
        quit_flag().lock().map(|q| *q).unwrap_or(false)
    }

    fn request_quit(&mut self) {
        if let Ok(mut q) = quit_flag().lock() {
            *q = true;
        }
    }

    fn set_title(&mut self, _title: &str) {}

    fn sleep_ms(&mut self, ms: u64) {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
}
