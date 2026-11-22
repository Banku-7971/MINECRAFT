//! C ABI exported for `android/app/src/main/cpp/jni_bridge.c`.

use crate::game::{self, Game};
use crate::platform::{android, Event};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static GAME: OnceLock<Mutex<Game<android::Android>>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn android_init(save_dir: *const c_char) {
    if !save_dir.is_null() {
        if let Ok(s) = unsafe { CStr::from_ptr(save_dir) }.to_str() {
            if !s.is_empty() {
                game::set_save_dir(PathBuf::from(s));
            }
        }
    }
    let _ = GAME.get_or_init(|| Mutex::new(Game::new(android::Android::new())));
}

#[no_mangle]
pub extern "C" fn android_frame(out_argb: *mut i32, len: i32) {
    if let Some(m) = GAME.get() {
        if let Ok(mut g) = m.lock() {
            g.frame();
        }
    }
    if !out_argb.is_null() && len > 0 {
        let slice = unsafe { std::slice::from_raw_parts_mut(out_argb, len as usize) };
        android::copy_frame_argb(slice);
    }
}

#[no_mangle]
pub extern "C" fn android_audio(out_pcm: *mut i16, len: i32) {
    if out_pcm.is_null() || len <= 0 {
        return;
    }
    let slice = unsafe { std::slice::from_raw_parts_mut(out_pcm, len as usize) };
    android::render_audio(slice);
}

#[no_mangle]
pub extern "C" fn android_key(code: i32, down: i32) {
    android::push_event(Event::Key {
        code: code.max(0) as u32,
        down: down != 0,
        repeat: false,
    });
}

#[no_mangle]
pub extern "C" fn android_char(code: i32) {
    if code > 0 {
        if let Some(c) = char::from_u32(code as u32) {
            android::push_event(Event::Char(c));
        }
    }
}

#[no_mangle]
pub extern "C" fn android_mouse_move(dx: i32, dy: i32) {
    android::push_event(Event::MouseMove { dx, dy });
}

#[no_mangle]
pub extern "C" fn android_mouse_to(x: i32, y: i32) {
    android::set_mouse(x, y);
    android::push_event(Event::MouseMove { dx: 0, dy: 0 });
}

#[no_mangle]
pub extern "C" fn android_mouse_button(button: i32, down: i32) {
    let b = button.clamp(0, 255) as u8;
    if down != 0 {
        android::push_event(Event::MouseDown(b));
    } else {
        android::push_event(Event::MouseUp(b));
    }
}

#[no_mangle]
pub extern "C" fn android_wheel(delta: i32) {
    android::push_event(Event::Wheel(delta));
}

#[no_mangle]
pub extern "C" fn android_blur() {
    android::push_event(Event::FocusLost);
}

#[no_mangle]
pub extern "C" fn android_ui_state() -> i32 {
    if let Some(m) = GAME.get() {
        if let Ok(g) = m.lock() {
            return g.ui_state() as i32;
        }
    }
    0
}
