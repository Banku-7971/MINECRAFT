//! C ABI exposed to `web/game.js`.
//!
//! JavaScript drives the game with `game_frame()` once per animation frame and
//! feeds input through the `game_*` functions below. The rendered RGBA frame
//! is read directly from the exported Wasm memory.

use crate::game::Game;
use crate::packs;
use crate::platform::web;
use crate::platform::Event;
use std::cell::RefCell;

thread_local! {
    static GAME: RefCell<Option<Game<web::Web>>> = RefCell::new(None);
    static LAST_STATUS: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {}
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn set_status_json(ok: bool, msg: &str, latest: Option<&packs::LoadedPackInfo>) {
    let shader = packs::active_shader();
    let list = packs::loaded_packs();
    let mut j = String::new();
    j.push('{');
    j.push_str(&format!("\"ok\":{},", if ok { "true" } else { "false" }));
    j.push_str(&format!("\"message\":{},", json_escape(msg)));
    j.push_str(&format!("\"shader\":{},", json_escape(&shader.name)));
    if let Some(p) = latest {
        j.push_str(&format!(
            "\"latest\":{{\"name\":{},\"kind\":{},\"desc\":{},\"textures\":{},\"recipes\":{},\"mods\":[",
            json_escape(&p.name),
            json_escape(p.kind.label()),
            json_escape(&p.description),
            p.textures_replaced,
            p.recipes_added
        ));
        for (i, (fnm, url)) in p.mod_files.iter().take(64).enumerate() {
            if i > 0 { j.push(','); }
            j.push_str(&format!("{{\"file\":{},\"url\":{}}}", json_escape(fnm), json_escape(url)));
        }
        j.push_str("]},");
    }
    j.push_str("\"packs\":[");
    for (i, p) in list.iter().enumerate() {
        if i > 0 { j.push(','); }
        j.push_str(&format!(
            "{{\"name\":{},\"kind\":{},\"desc\":{},\"textures\":{},\"recipes\":{},\"modsCount\":{}}}",
            json_escape(&p.name),
            json_escape(p.kind.label()),
            json_escape(&p.description),
            p.textures_replaced,
            p.recipes_added,
            p.mod_files.len()
        ));
    }
    j.push_str("]}");
    LAST_STATUS.with(|s| *s.borrow_mut() = j.into_bytes());
}

#[no_mangle]
pub extern "C" fn game_alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::<u8>::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub extern "C" fn game_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(ptr, 0, len);
        }
    }
}

#[no_mangle]
pub extern "C" fn game_load_pack(
    name_ptr: *const u8,
    name_len: usize,
    data_ptr: *const u8,
    data_len: usize,
) -> i32 {
    let name_bytes = unsafe { core::slice::from_raw_parts(name_ptr, name_len) };
    let data_bytes = unsafe { core::slice::from_raw_parts(data_ptr, data_len) };
    let fname = core::str::from_utf8(name_bytes).unwrap_or("pack.zip");
    match packs::load_pack_bytes(fname, data_bytes) {
        Ok(info) => {
            let summary = format!("Loaded {} [{}]: {}", info.name, info.kind.label(), info.description);
            set_status_json(true, &summary, Some(&info));
            1
        }
        Err(err) => {
            set_status_json(false, &err, None);
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn game_reset_packs() {
    packs::reset_all_packs();
    set_status_json(true, "Reset to vanilla textures, shaders, and recipes", None);
}

#[no_mangle]
pub extern "C" fn game_cycle_shader() {
    let sh = packs::cycle_shader_preset();
    set_status_json(true, &format!("Shader: {}", sh.name), None);
}

#[no_mangle]
pub extern "C" fn game_set_shader(idx: u32) {
    let sh = packs::set_shader_preset(idx as usize);
    set_status_json(true, &format!("Shader: {}", sh.name), None);
}

#[no_mangle]
pub extern "C" fn game_query_packs() {
    let sh = packs::active_shader();
    set_status_json(true, &format!("Shader: {}", sh.name), None);
}

#[no_mangle]
pub extern "C" fn game_status_ptr() -> *const u8 {
    LAST_STATUS.with(|s| {
        let b = s.borrow();
        if b.is_empty() { core::ptr::null() } else { b.as_ptr() }
    })
}

#[no_mangle]
pub extern "C" fn game_status_len() -> usize {
    LAST_STATUS.with(|s| s.borrow().len())
}

#[no_mangle]
pub extern "C" fn game_init() {
    GAME.with(|slot| {
        if slot.borrow().is_none() {
            let game = Game::new(web::Web::new());
            *slot.borrow_mut() = Some(game);
        }
    });
}

#[no_mangle]
pub extern "C" fn game_frame() {
    GAME.with(|slot| {
        if let Some(game) = slot.borrow_mut().as_mut() {
            game.frame();
        }
    });
}

#[no_mangle]
pub extern "C" fn game_frame_ptr() -> *const u8 {
    web::frame_ptr()
}

#[no_mangle]
pub extern "C" fn game_frame_len() -> usize {
    web::frame_len()
}

#[no_mangle]
pub extern "C" fn game_key(code: u32, down: u32) {
    web::push_event(Event::Key { code, down: down != 0, repeat: false });
}

#[no_mangle]
pub extern "C" fn game_char(code: u32) {
    if let Some(c) = char::from_u32(code) {
        web::push_event(Event::Char(c));
    }
}

#[no_mangle]
pub extern "C" fn game_mouse_move(dx: i32, dy: i32) {
    web::push_event(Event::MouseMove { dx, dy });
}

/// Move the software cursor to an absolute framebuffer position. Used for
/// menus/inventory while pointer lock is released.
#[no_mangle]
pub extern "C" fn game_mouse_to(x: i32, y: i32) {
    web::set_mouse(x, y);
    web::push_event(Event::MouseMove { dx: 0, dy: 0 });
}

#[no_mangle]
pub extern "C" fn game_mouse_button(button: u32, down: u32) {
    let button = button.min(255) as u8;
    if down != 0 {
        web::push_event(Event::MouseDown(button));
    } else {
        web::push_event(Event::MouseUp(button));
    }
}

#[no_mangle]
pub extern "C" fn game_wheel(delta: i32) {
    web::push_event(Event::Wheel(delta));
}

#[no_mangle]
pub extern "C" fn game_blur() {
    web::push_event(Event::FocusLost);
}

#[no_mangle]
pub extern "C" fn game_audio_pop() -> i32 {
    crate::sound::pop_web_sfx()
}

#[no_mangle]
pub extern "C" fn game_audio_flags() -> u32 {
    let m = if web::music_enabled() { 1 } else { 0 };
    let s = if web::sound_enabled() { 2 } else { 0 };
    m | s
}
