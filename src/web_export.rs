//! C ABI exposed to `web/game.js`.
//!
//! JavaScript drives the game with `game_frame()` once per animation frame and
//! feeds input through the `game_*` functions below. The rendered RGBA frame
//! is read directly from the exported Wasm memory.

use crate::game::Game;
use crate::platform::web;
use crate::platform::Event;
use std::cell::RefCell;

thread_local! {
    static GAME: RefCell<Option<Game<web::Web>>> = RefCell::new(None);
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
