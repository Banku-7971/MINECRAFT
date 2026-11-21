//! Headless platform used for automated testing on Linux: frames are written to
//! runtime/frame.png and input is scripted through runtime/events.log.
use std::path::Path;
use std::time::Instant;

use crate::platform::{Event, Platform};
use crate::render::Framebuffer;
use crate::sound::Sfx;

pub struct Harness {
    start: Instant,
    mouse: (i32, i32),
    offset: u64,
    clipboard: String,
    quit: bool,
    frame_no: u64,
    keymap: Vec<(String, u32)>,
}

impl Harness {
    pub fn new() -> Self {
        let _ = std::fs::create_dir_all("runtime");
        let _ = std::fs::write("runtime/events.log", b"");
        Harness {
            start: Instant::now(),
            mouse: (240, 135),
            offset: 0,
            clipboard: String::new(),
            quit: false,
            frame_no: 0,
            keymap: vec![
                ("W".into(), 0x57), ("A".into(), 0x41), ("S".into(), 0x53), ("D".into(), 0x44),
                ("E".into(), 0x45), ("T".into(), 0x54), ("Q".into(), 0x51), ("F".into(), 0x46),
                ("SPACE".into(), 0x20), ("SHIFT".into(), 0x10), ("CTRL".into(), 0x11), ("ESC".into(), 0x1B),
                ("ENTER".into(), 0x0D), ("BACK".into(), 0x08), ("F4".into(), 0x73), ("F5".into(), 0x74),
                ("LEFT".into(), 0x25), ("UP".into(), 0x26), ("RIGHT".into(), 0x27), ("DOWN".into(), 0x28),
                ("1".into(), 0x31), ("2".into(), 0x32), ("3".into(), 0x33), ("4".into(), 0x34),
                ("5".into(), 0x35), ("6".into(), 0x36), ("7".into(), 0x37), ("8".into(), 0x38),
                ("9".into(), 0x39),
            ],
        }
    }

    fn key_code(&self, name: &str) -> Option<u32> {
        let up = name.to_ascii_uppercase();
        self.keymap.iter().find(|(n, _)| *n == up).map(|(_, c)| *c)
    }
}

impl Platform for Harness {
    fn poll(&mut self) -> Vec<Event> {
        let mut out = Vec::new();
        if let Ok(data) = std::fs::read("runtime/events.log") {
            if data.len() as u64 > self.offset {
                let text = String::from_utf8_lossy(&data[self.offset as usize..]).to_string();
                self.offset = data.len() as u64;
                for line in text.lines() {
                    let mut it = line.split_whitespace();
                    match it.next() {
                        Some("key") => {
                            let k = it.next().unwrap_or("");
                            let st = it.next().unwrap_or("down");
                            if let Some(code) = self.key_code(k) {
                                out.push(Event::Key { code, down: st != "up", repeat: false });
                            } else if k.len() == 1 {
                                out.push(Event::Char(k.chars().next().unwrap()));
                            }
                        }
                        Some("move") => {
                            let dx: i32 = it.next().unwrap_or("0").parse().unwrap_or(0);
                            let dy: i32 = it.next().unwrap_or("0").parse().unwrap_or(0);
                            self.mouse = ((self.mouse.0 + dx).clamp(0, 479), (self.mouse.1 + dy).clamp(0, 269));
                            out.push(Event::MouseMove { dx, dy });
                        }
                        Some("mouse") => {
                            let x: i32 = it.next().unwrap_or("240").parse().unwrap_or(240);
                            let y: i32 = it.next().unwrap_or("135").parse().unwrap_or(135);
                            self.mouse = (x, y);
                            out.push(Event::MouseMove { dx: 0, dy: 0 });
                        }
                        Some("down") => {
                            let b: u8 = it.next().unwrap_or("0").parse().unwrap_or(0);
                            out.push(Event::MouseDown(b));
                        }
                        Some("up") => {
                            let b: u8 = it.next().unwrap_or("0").parse().unwrap_or(0);
                            out.push(Event::MouseUp(b));
                        }
                        Some("wheel") => {
                            let d: i32 = it.next().unwrap_or("0").parse().unwrap_or(0);
                            out.push(Event::Wheel(d));
                        }
                        Some("text") => {
                            for c in it.next().unwrap_or("").chars() { out.push(Event::Char(c)); }
                        }
                        Some("quit") => { self.quit = true; }
                        _ => {}
                    }
                }
            }
        }
        out
    }

    fn present(&mut self, fb: &mut Framebuffer) {
        self.frame_no += 1;
        let w = fb.w;
        let h = fb.h;
        let tmp = Path::new("runtime/frame.tmp.png");
        if render_png(tmp, &fb.rgba, w, h).is_ok() {
            let _ = std::fs::rename(tmp, "runtime/frame.png");
        }
    }

    fn now(&self) -> f64 { self.start.elapsed().as_secs_f64() }

    fn play(&mut self, _s: Sfx) {}
    fn set_music(&mut self, _on: bool) {}
    fn set_sound(&mut self, _on: bool) {}
    fn set_cursor(&mut self, _visible: bool) {}
    fn set_capture(&mut self, _captured: bool) {}
    fn mouse(&self) -> (i32, i32) { self.mouse }
    fn clipboard(&mut self) -> Option<String> { Some(self.clipboard.clone()) }
    fn set_clipboard(&mut self, text: &str) { self.clipboard = text.to_string(); }
    fn should_quit(&self) -> bool { self.quit }
    fn request_quit(&mut self) { self.quit = true; }
    fn set_title(&mut self, _title: &str) {}
    fn sleep_ms(&mut self, ms: u64) { std::thread::sleep(std::time::Duration::from_millis(ms.min(4))); }
}

fn render_png(path: &Path, rgba: &[u8], w: usize, h: usize) -> std::io::Result<()> {
    crate::render::encode_png(path, rgba, w, h)
}
