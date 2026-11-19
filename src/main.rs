//! Minecraft, written in Rust. Native Windows build; the same game code also
//! builds headless on Linux for automated testing.
#![cfg_attr(windows, windows_subsystem = "windows")]

mod craft;
mod entities;
mod game;
mod items;
mod net;
mod platform;
mod render;
mod sound;
mod world;

fn main() {
    // Without a console we still want crashes to be diagnosable.
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("Minecraft crashed:\n{}\n", info);
        let _ = std::fs::write("minecraft-crash.log", msg.as_bytes());
        eprintln!("{}", msg);
    }));

    let plat = platform::create();
    game::Game::new(plat).run();
}
