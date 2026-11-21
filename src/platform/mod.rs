#![allow(dead_code)]
//! Platform layer: the game talks to the OS only through this trait.
//! `win32` implements it for Windows, `harness` implements it for headless tests.

use crate::render::Framebuffer;
use crate::sound::Sfx;

pub const VK_BACK: u32 = 0x08;
pub const VK_TAB: u32 = 0x09;
pub const VK_RETURN: u32 = 0x0D;
pub const VK_SHIFT: u32 = 0x10;
pub const VK_CONTROL: u32 = 0x11;
pub const VK_ESCAPE: u32 = 0x1B;
pub const VK_SPACE: u32 = 0x20;
pub const VK_PRIOR: u32 = 0x21;
pub const VK_NEXT: u32 = 0x22;
pub const VK_LEFT: u32 = 0x25;
pub const VK_UP: u32 = 0x26;
pub const VK_RIGHT: u32 = 0x27;
pub const VK_DOWN: u32 = 0x28;
pub const VK_A: u32 = 0x41;
pub const VK_C: u32 = 0x43;
pub const VK_D: u32 = 0x44;
pub const VK_E: u32 = 0x45;
pub const VK_F: u32 = 0x46;
pub const VK_Q: u32 = 0x51;
pub const VK_S: u32 = 0x53;
pub const VK_T: u32 = 0x54;
pub const VK_W: u32 = 0x57;
pub const VK_F3: u32 = 0x72;
pub const VK_F4: u32 = 0x73;
pub const VK_F5: u32 = 0x74;

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Key { code: u32, down: bool, repeat: bool },
    Char(char),
    MouseMove { dx: i32, dy: i32 },
    MouseDown(u8),
    MouseUp(u8),
    Wheel(i32),
    Resize(i32, i32),
    FocusLost,
    Close,
}

pub trait Platform {
    /// Drain the OS event queue.
    fn poll(&mut self) -> Vec<Event>;
    /// Show a finished 480x270 frame.
    fn present(&mut self, fb: &mut Framebuffer);
    /// Seconds since start, for animation and frame pacing.
    fn now(&self) -> f64;
    fn play(&mut self, s: Sfx);
    fn set_music(&mut self, on: bool);
    fn set_sound(&mut self, on: bool);
    fn set_cursor(&mut self, visible: bool);
    fn set_capture(&mut self, captured: bool);
    /// Mouse position in 480x270 framebuffer space.
    fn mouse(&self) -> (i32, i32);
    fn clipboard(&mut self) -> Option<String>;
    fn set_clipboard(&mut self, text: &str);
    fn should_quit(&self) -> bool;
    fn request_quit(&mut self);
    fn set_title(&mut self, title: &str);
    fn sleep_ms(&mut self, ms: u64);
}

#[cfg(windows)]
pub mod win32;
#[cfg(all(not(windows), not(target_arch = "wasm32")))]
pub mod harness;
#[cfg(target_arch = "wasm32")]
pub mod web;

#[cfg(windows)]
pub fn create() -> impl Platform { win32::Win32::new("Minecraft") }

#[cfg(all(not(windows), not(target_arch = "wasm32")))]
pub fn create() -> impl Platform { harness::Harness::new() }

#[cfg(target_arch = "wasm32")]
pub fn create() -> impl Platform { web::Web::new() }
