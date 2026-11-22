//! Library entry point used by the browser/WebAssembly build.
//!
//! The native binaries keep using `src/main.rs`; this file only exists so the
//! same game code can be compiled to `wasm32-unknown-unknown` and driven by
//! the small JavaScript glue in `web/game.js`.

#![allow(dead_code)]

mod craft;
mod entities;
mod game;
mod items;

#[cfg(not(target_arch = "wasm32"))]
mod net;

#[cfg(target_arch = "wasm32")]
#[path = "net_stub.rs"]
mod net;

mod packs;
mod platform;
mod render;
mod sound;
mod world;

#[cfg(target_arch = "wasm32")]
mod web_export;

#[cfg(target_os = "android")]
mod android_export;
