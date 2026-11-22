//! WebAssembly platform backend.
//!
//! The Rust game loop is driven by `requestAnimationFrame` from JavaScript.
//! Input arrives through exported functions, and the 480x270 RGBA frame is
//! copied into a thread-local buffer that JavaScript reads through
//! `game_frame_ptr` / `game_frame_len`.

use crate::platform::{Event, Platform};
use crate::render::Framebuffer;
use crate::sound::{push_web_sfx, Sfx};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

#[link(wasm_import_module = "env")]
extern "C" {
    /// Seconds since page load, provided by `performance.now() / 1000`.
    fn js_now() -> f64;
    /// Ask the page to request or release pointer lock.
    fn js_set_capture(captured: u32);
    /// Show or hide the mouse cursor over the canvas.
    fn js_set_cursor(visible: u32);
}

thread_local! {
    static EVENTS: RefCell<VecDeque<Event>> = RefCell::new(VecDeque::new());
    static MOUSE: Cell<(i32, i32)> = Cell::new((240, 135));
    static QUIT: Cell<bool> = Cell::new(false);
    static FRAME: RefCell<Vec<u8>> = RefCell::new(Vec::new());
    static SOUND_ON: Cell<bool> = Cell::new(true);
    static MUSIC_ON: Cell<bool> = Cell::new(true);
    static MUSIC_DIM: Cell<u8> = Cell::new(0);
}

pub fn sound_enabled() -> bool { SOUND_ON.with(|c| c.get()) }
pub fn music_enabled() -> bool { MUSIC_ON.with(|c| c.get()) }
pub fn music_dimension() -> u8 { MUSIC_DIM.with(|c| c.get()) }

pub struct Web;

impl Web {
    pub fn new() -> Self {
        Web
    }
}

pub fn push_event(event: Event) {
    EVENTS.with(|queue| queue.borrow_mut().push_back(event));
}

pub fn set_mouse(x: i32, y: i32) {
    MOUSE.with(|mouse| mouse.set((x.clamp(0, 479), y.clamp(0, 269))));
}

pub fn set_quit() {
    QUIT.with(|quit| quit.set(true));
}

pub fn frame_ptr() -> *const u8 {
    FRAME.with(|frame| frame.borrow().as_ptr())
}

pub fn frame_len() -> usize {
    FRAME.with(|frame| frame.borrow().len())
}

impl Platform for Web {
    fn poll(&mut self) -> Vec<Event> {
        EVENTS.with(|queue| queue.borrow_mut().drain(..).collect())
    }

    fn present(&mut self, fb: &mut Framebuffer) {
        FRAME.with(|frame| {
            let mut frame = frame.borrow_mut();
            frame.clear();
            frame.extend_from_slice(&fb.rgba);
        });
    }

    fn now(&self) -> f64 {
        unsafe { js_now() }
    }

    fn play(&mut self, s: Sfx) {
        if sound_enabled() {
            push_web_sfx(s);
        }
    }

    fn set_music(&mut self, on: bool) {
        MUSIC_ON.with(|c| c.set(on));
    }

    fn set_sound(&mut self, on: bool) {
        SOUND_ON.with(|c| c.set(on));
    }

    fn set_dimension(&mut self, dim: u8) {
        MUSIC_DIM.with(|c| c.set(dim));
    }

    fn set_cursor(&mut self, visible: bool) {
        unsafe { js_set_cursor(visible as u32) }
    }

    fn set_capture(&mut self, captured: bool) {
        unsafe { js_set_capture(captured as u32) }
    }

    fn mouse(&self) -> (i32, i32) {
        MOUSE.with(|mouse| mouse.get())
    }

    fn clipboard(&mut self) -> Option<String> {
        None
    }

    fn set_clipboard(&mut self, _text: &str) {}

    fn should_quit(&self) -> bool {
        QUIT.with(|quit| quit.get())
    }

    fn request_quit(&mut self) {
        set_quit();
    }

    fn set_title(&mut self, _title: &str) {}

    fn sleep_ms(&mut self, _ms: u64) {}
}
