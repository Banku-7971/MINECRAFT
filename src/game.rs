//! The game itself: screens, world simulation, inventory, crafting, furnaces,
//! redstone, multiplayer and rendering. Talks to the OS only via `Platform`.
use std::collections::{HashMap, HashSet};

use crate::craft::{self, Stack, GRID};
use crate::entities::{self, Drop, Mob, MobKind};
use crate::items;
use crate::net::{Net, NetEvent};
use crate::platform::{self as plat, Event, Platform};
use crate::render::{self, Cam, EntityKind, Framebuffer, GameMode, Textures};
use crate::sound::{Audio, Sfx};
use crate::world::{self, Dimension, World};

pub const HOT: usize = 9;
pub const STORE: usize = 27;
const REACH: f32 = 5.5;

pub struct Player {
    pub x: f32, pub y: f32, pub z: f32, pub vy: f32,
    pub yaw: f32, pub pitch: f32,
    pub hp: f32, pub hunger: f32,
    pub on_ground: bool,
    pub selected: usize,
    pub hot: [Stack; HOT],
    pub store: [Stack; STORE],
}

impl Player {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Player {
            x, y, z, vy: 0.0, yaw: 0.0, pitch: -0.08,
            hp: 20.0, hunger: 20.0, on_ground: false,
            selected: 0,
            hot: [Stack::EMPTY; HOT],
            store: [Stack::EMPTY; STORE],
        }
    }

    fn held(&self) -> Stack { self.hot[self.selected] }

    fn add(&mut self, id: u16, mut n: u32) -> u32 {
        if id == 0 || n == 0 { return 0; }
        for s in self.hot.iter_mut().chain(self.store.iter_mut()) {
            if s.id == id && s.count < 64 {
                let room = 64 - s.count;
                let put = room.min(n);
                s.count += put; n -= put;
                if n == 0 { return 0; }
            }
        }
        for s in self.hot.iter_mut().chain(self.store.iter_mut()) {
            if s.is_empty() {
                let put = n.min(64);
                *s = Stack::new(id, put); n -= put;
                if n == 0 { return 0; }
            }
        }
        n
    }
}

pub struct Furnace {
    pub input: Stack,
    pub fuel: Stack,
    pub output: Stack,
    pub burn: f32,
    pub burn_max: f32,
    pub cook: f32,
}

impl Furnace {
    fn new() -> Self {
        Furnace { input: Stack::EMPTY, fuel: Stack::EMPTY, output: Stack::EMPTY, burn: 0.0, burn_max: 0.0, cook: 0.0 }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Screen { Title, Multiplayer, Options, Playing }

#[derive(PartialEq, Clone, Copy)]
pub enum Ui { None, Inventory, Creative, Furnace, Chat, Pause }

pub struct Game<P: Platform> {
    plat: P,
    pub audio: Audio,
    net: Net,
    world: World,
    tex: Textures,
    scene: Framebuffer,
    fb: Framebuffer,
    pub player: Player,
    mobs: Vec<Mob>,
    drops: Vec<Drop>,
    furnaces: HashMap<(i32, i32, i32), Furnace>,
    power: HashMap<(i32, i32, i32), u8>,
    levers: HashSet<(i32, i32, i32)>,
    plates: HashSet<(i32, i32, i32)>,
    lamps: HashSet<(i32, i32, i32)>,
    screen: Screen,
    ui: Ui,
    ui_block: (i32, i32, i32),
    craft3: bool,
    cursor: Stack,
    craft_grid: [Stack; GRID],
    keys: HashSet<u32>,
    mouse_left: bool,
    tnt_primed: Vec<(f32, f32, f32, f32)>,
    redstone_dirty: bool,
    redstone_sources: HashSet<(i32, i32, i32)>,
    fall_y: f32,
    spawn_timer: f32,
    w_tap_t: f64,
    sprinting: bool,
    chat_input: String,
    chat_lines: Vec<(String, f32)>,
    net_status: String,
    net_addr: String,
    menu_hover: usize,
    camera: u8,
    time: f32,
    tick: u32,
    message: String,
    msg_timer: f32,
    mining: Option<((i32, i32, i32), f32)>,
    attack_cd: f32,
    place_cd: f32,
    hurt_flash: f32,
    grace: f32,
    regen: f32,
    hurt_cd: f32,
    fps: u32,
    frames: u32,
    fps_timer: f64,
    last_frame: f64,
    spawn: (f32, f32, f32),
    remote: HashMap<u32, (f32, f32, f32, f32, f32)>,
    net_timer: f64,
    sound_on: bool,
    music_on: bool,
    pixel_scale: i32,
    audio_timer: f64,
    quit: bool,
    splash: String,
    mouse_x: i32,
    mouse_y: i32,
    title_t: f32,
    recipe_page: usize,
    recipe_sel: usize,
    return_to_pause: bool,
    mode: GameMode,
    flying: bool,
    space_tap_t: f64,
    spec_speed: f32,
    creative_tab: usize,
    creative_scroll: usize,
    creative_search: String,
    creative_break_cd: f32,
    portal_cd: f32,
    save_timer: f32,
    dragon_defeated: bool,
    loaded_save: bool,
}

fn collides(world: &mut World, x: f32, y: f32, z: f32) -> bool {
    let r = 0.29;
    for dx in [-r, r] {
        for dz in [-r, r] {
            for dy in [0.1, 0.9, 1.7] {
                let b = world.get((x + dx).floor() as i32, (y + dy).floor() as i32, (z + dz).floor() as i32);
                if world::is_solid(b) { return true; }
            }
        }
    }
    false
}

/// What a block leaves behind when mined.
fn block_drop(b: u16, tick: u32) -> Stack {
    use world::*;
    match b {
        GRASS | PODZOL | MYCELIUM | FARMLAND => Stack::new(DIRT, 1),
        STONE => Stack::new(COBBLE, 1),
        COAL_ORE => Stack::new(items::COAL, 1),
        DIAMOND_ORE => Stack::new(items::DIAMOND, 1),
        EMERALD_ORE => Stack::new(items::EMERALD, 1),
        REDSTONE_ORE => Stack::new(REDSTONE_DUST, 4),
        NETHER_QUARTZ_ORE => Stack::new(items::NETHER_QUARTZ, 1),
        CRIMSON_NYLIUM | WARPED_NYLIUM => Stack::new(NETHERRACK, 1),
        END_PORTAL_FRAME | END_PORTAL_FRAME_EYE | NETHER_PORTAL | END_PORTAL => Stack::EMPTY,
        IRON_ORE | GOLD_ORE => Stack::new(b, 1),
        CLAY => Stack::new(CLAY, 4),
        GRAVEL => if tick % 10 == 0 { Stack::new(items::FLINT, 1) } else { Stack::new(GRAVEL, 1) },
        LEAVES | SPRUCE_LEAVES | JUNGLE_LEAVES | BIRCH_LEAVES | DARK_LEAVES => {
            if tick % 20 == 0 { Stack::new(items::APPLE, 1) }
            else if tick % 12 == 0 { Stack::new(items::STICK, 1) }
            else { Stack::EMPTY }
        }
        WHEAT => Stack::new(WHEAT, 2),
        TALL_GRASS => if tick % 8 == 0 { Stack::new(WHEAT, 1) } else { Stack::EMPTY },
        ICE | GLASS => Stack::EMPTY,
        _ => Stack::new(b, 1),
    }
}

fn requires_pickaxe(b: u16) -> u8 {
    use world::*;
    match b {
        STONE | COBBLE | STONE_BRICKS | BRICK | FURNACE | SANDSTONE | COAL_ORE | QUARTZ | BLACKSTONE
        | SLAB | STAIRS | ANVIL | ENCHANTING | MOSSY | IRON_BLOCK | NETHERRACK | NETHER_BRICKS
        | CRIMSON_NYLIUM | WARPED_NYLIUM | BASALT | END_STONE | PURPUR_BLOCK | NETHER_QUARTZ_ORE => 1,
        IRON_ORE | LAMP => 2,
        GOLD_ORE | DIAMOND_ORE | REDSTONE_ORE | EMERALD_ORE | GOLD_BLOCK | DIAMOND_BLOCK | EMERALD_BLOCK => 3,
        OBSIDIAN | CRYING_OBSIDIAN | ANCIENT_DEBRIS => 4,
        _ => 0,
    }
}

fn scene_dims(scale: i32) -> (usize, usize) {
    match scale {
        1 => (480, 270),
        2 => (320, 180),
        3 => (240, 135),
        _ => (160, 90),
    }
}

impl<P: Platform> Game<P> {
    pub fn new(plat: P) -> Self {
        let mut world = World::new();
        let spawn_x = 0.5;
        let spawn_z = -239.5;
        let spawn_y = world.highest_ground(spawn_x as i32, spawn_z as i32) as f32 + 0.2;
        let player = Player::new(spawn_x, spawn_y, spawn_z);
        let mut mobs = Vec::new();
        for i in 0..26 {
            let a = i as f32 * 2.39996;
            let d = 11.0 + (i % 9) as f32 * 2.6;
            let x = spawn_x + a.cos() * d;
            let z = spawn_z + a.sin() * d;
            let bm = world.biome(x as i32, z as i32);
            let k = entities::biome_mob(bm, world.dim, i * 17 + 3, 0.6);
            mobs.push(Mob::spawn(k, x, world.highest_ground(x as i32, z as i32) as f32, z));
        }
        let splash = "Now with Nether, End, biomes, crafting & LAN!";
        let mut game = Game {
            plat,
            audio: Audio::new(),
            net: Net::offline(),
            world,
            tex: Textures::new(),
            scene: Framebuffer::new(render::RW, render::RH),
            fb: Framebuffer::new(render::W, render::H),
            player,
            mobs,
            drops: Vec::new(),
            furnaces: HashMap::new(),
            power: HashMap::new(),
            levers: HashSet::new(),
            plates: HashSet::new(),
            lamps: HashSet::new(),
            screen: Screen::Title,
            ui: Ui::None,
            ui_block: (0, 0, 0),
            craft3: false,
            cursor: Stack::EMPTY,
            craft_grid: [Stack::EMPTY; GRID],
            keys: HashSet::new(),
            mouse_left: false,
            tnt_primed: Vec::new(),
            redstone_dirty: false,
            redstone_sources: HashSet::new(),
            fall_y: spawn_y,
            spawn_timer: 0.0,
            w_tap_t: -10.0,
            sprinting: false,
            chat_input: String::new(),
            chat_lines: Vec::new(),
            net_status: String::from("Host a game or join by IP"),
            net_addr: String::from("127.0.0.1"),
            menu_hover: 0,
            camera: 0,
            time: 0.30,
            tick: 0,
            message: String::from("Punch trees for wood! WASD move, Shift sneak, E inventory"),
            msg_timer: 6.0,
            mining: None,
            attack_cd: 0.0,
            place_cd: 0.0,
            hurt_flash: 0.0,
            grace: 15.0,
            regen: 0.0,
            hurt_cd: 0.0,
            fps: 0,
            frames: 0,
            fps_timer: 0.0,
            last_frame: 0.0,
            spawn: (spawn_x, spawn_y, spawn_z),
            remote: HashMap::new(),
            net_timer: 0.0,
            sound_on: true,
            music_on: true,
            pixel_scale: 2,
            audio_timer: 0.0,
            quit: false,
            splash: splash.to_string(),
            mouse_x: 240,
            mouse_y: 135,
            title_t: 0.0,
            recipe_page: 0,
            recipe_sel: 0,
            return_to_pause: false,
            mode: GameMode::Survival,
            flying: false,
            space_tap_t: -10.0,
            spec_speed: 1.0,
            creative_tab: 0,
            creative_scroll: 0,
            creative_search: String::new(),
            creative_break_cd: 0.0,
            portal_cd: 0.0,
            save_timer: 0.0,
            dragon_defeated: false,
            loaded_save: false,
        };
        game.load_world();
        game
    }

    pub fn run(mut self) {
        self.plat.set_title("Minecraft");
        self.plat.set_sound(true);
        self.plat.set_music(true);
        loop {
            if self.plat.should_quit() || self.quit {
                self.save_world();
                break;
            }
            let now = self.plat.now();
            let dt = if self.last_frame == 0.0 { 0.016 } else { (now - self.last_frame).clamp(0.0005, 0.1) };
            self.last_frame = now;
            self.title_t += dt as f32;
            self.tick = self.tick.wrapping_add(1);

            let events = self.plat.poll();
            self.handle_events(events);
            (self.mouse_x, self.mouse_y) = self.plat.mouse();
            self.fb.cursor = (self.mouse_x, self.mouse_y);

            match self.screen {
                Screen::Title => {
                    self.draw_title();
                }
                Screen::Multiplayer => self.draw_multiplayer(),
                Screen::Options => self.draw_options(),
                Screen::Playing => {
                    self.update_playing(dt as f32);
                    self.draw_playing();
                }
            }

            self.frames += 1;
            if now - self.fps_timer > 1.0 {
                self.fps = self.frames;
                self.frames = 0;
                self.fps_timer = now;
            }

            let mut frame = std::mem::replace(&mut self.fb, Framebuffer::new(render::W, render::H));
            frame.cursor = (self.mouse_x, self.mouse_y);
            self.plat.present(&mut frame);
            self.fb = frame;

            // ~60 fps target
            let spent = self.plat.now() - now;
            if spent < 0.0166 {
                self.plat.sleep_ms(((0.0166 - spent) * 1000.0) as u64);
            }
        }
        self.net.close();
    }

    // ------------------------------------------------------------------ input
    fn handle_events(&mut self, events: Vec<Event>) {
        for e in events {
            match e {
                Event::Close => { self.quit = true; }
                Event::Resize(_, _) => {}
                Event::FocusLost => { self.keys.clear(); self.mouse_left = false; self.sprinting = false; }
                Event::MouseMove { dx, dy } => {
                    if self.screen == Screen::Playing && (self.ui == Ui::None || self.ui == Ui::Chat) {
                        let sens = 0.0026;
                        self.player.yaw += dx as f32 * sens;
                        self.player.pitch = (self.player.pitch - dy as f32 * 0.0024).clamp(-1.52, 1.52);
                        (self.mouse_x, self.mouse_y) = self.plat.mouse();
                    } else {
                        let (mx, my) = self.plat.mouse();
                        self.mouse_x = mx;
                        self.mouse_y = my;
                    }
                }
                Event::Key { code, down, repeat } => {
                    if down {
                        self.keys.insert(code);
                        if !repeat { self.key_pressed(code); }
                    } else {
                        self.keys.remove(&code);
                    }
                }
                Event::Char(c) => {
                    if self.ui == Ui::Chat {
                        if !c.is_control() && self.chat_input.len() < 80 { self.chat_input.push(c); }
                    } else if self.ui == Ui::Creative {
                        if !c.is_control() && self.creative_search.len() < 24 {
                            self.creative_search.push(c);
                            self.creative_tab = 5;
                            self.creative_scroll = 0;
                        }
                    } else if self.screen == Screen::Multiplayer {
                        if c.is_ascii_graphic() || c == '.' || c == ':' {
                            if self.net_addr.len() < 40 { self.net_addr.push(c); }
                        }
                    }
                }
                Event::MouseDown(b) => {
                    match self.screen {
                        Screen::Title | Screen::Multiplayer | Screen::Options => {
                            self.plat.play(Sfx::Click);
                            self.click_menu();
                        }
                        Screen::Playing => {
                            match self.ui {
                                Ui::Inventory | Ui::Furnace => self.click_inventory(b == 1),
                                Ui::Creative => self.click_creative(b == 1),
                                Ui::Pause => self.click_pause(),
                                Ui::Chat => {}
                                Ui::None => {
                                    self.plat.set_capture(true);
                                    if self.mode != GameMode::Spectator {
                                        if b == 0 { self.mouse_left = true; self.try_attack(); }
                                        else if b == 1 { self.try_use(); }
                                    }
                                }
                            }
                        }
                    }
                }
                Event::MouseUp(b) => {
                    if b == 0 { self.mouse_left = false; self.mining = None; }
                }
                Event::Wheel(d) => {
                    if self.screen == Screen::Playing {
                        if self.ui == Ui::Creative {
                            let max_scroll = ((self.creative_items_filtered().len() + 8) / 9).saturating_sub(5);
                            if d > 0 { self.creative_scroll = self.creative_scroll.saturating_sub(1); }
                            else if d < 0 { self.creative_scroll = (self.creative_scroll + 1).min(max_scroll); }
                        } else if self.ui == Ui::None {
                            if self.mode == GameMode::Spectator {
                                self.spec_speed = (self.spec_speed + d as f32 * 0.25).clamp(0.4, 4.0);
                                self.message = format!("Fly speed: {:.2}x", self.spec_speed);
                                self.msg_timer = 1.2;
                            } else {
                                let sel = (self.player.selected as i32 - d).rem_euclid(9) as usize;
                                self.player.selected = sel;
                            }
                        }
                    }
                }
            }
        }
    }

    fn key_pressed(&mut self, code: u32) {
        use plat::*;
        match self.screen {
            Screen::Title => {
                if code == VK_RETURN { self.start_singleplayer(); }
            }
            Screen::Options => {
                if code == VK_ESCAPE { if self.return_to_pause { self.return_to_pause = false; self.screen = Screen::Playing; self.ui = Ui::Pause; } else { self.screen = Screen::Title; } }
            }
            Screen::Multiplayer => {
                if code == VK_ESCAPE { self.screen = Screen::Title; }
                if code == VK_BACK { self.net_addr.pop(); }
                if code == VK_RETURN { self.join_game(); }
            }
            Screen::Playing => {
                if self.ui == Ui::Chat {
                    if code == VK_RETURN {
                        let text = self.chat_input.clone();
                        if !text.is_empty() {
                            if !self.handle_command(&text) {
                                self.chat_lines.push((format!("<Player> {}", text), 0.0));
                                self.net.send(format!("CHAT|Player|{}", text));
                            }
                        }
                        self.chat_input.clear();
                        self.ui = Ui::None;
                        self.plat.set_capture(true);
                    } else if code == VK_BACK {
                        self.chat_input.pop();
                    } else if code == VK_ESCAPE {
                        self.chat_input.clear();
                        self.ui = Ui::None;
                        self.plat.set_capture(true);
                    }
                    return;
                }
                if self.ui == Ui::Pause {
                    if code == VK_ESCAPE { self.resume(); }
                    return;
                }
                if self.ui == Ui::Creative {
                    if code == VK_ESCAPE || (code == VK_E && self.creative_tab != 5) {
                        self.close_ui();
                    } else if code == VK_BACK {
                        self.creative_search.pop();
                        self.creative_scroll = 0;
                    }
                    return;
                }
                if self.ui == Ui::Inventory || self.ui == Ui::Furnace {
                    if code == VK_E || code == VK_ESCAPE { self.close_ui(); }
                    return;
                }
                match code {
                    VK_ESCAPE => self.open_pause(),
                    VK_E => {
                        if self.mode == GameMode::Creative { self.open_creative(); }
                        else if self.mode == GameMode::Survival { self.open_inventory(false); }
                    }
                    VK_T => {
                        self.ui = Ui::Chat;
                        self.chat_input.clear();
                        self.plat.set_capture(false);
                        self.plat.set_cursor(true);
                    }
                    VK_F4 => {
                        let next = self.mode.next();
                        self.set_mode(next);
                    }
                    VK_F5 => {
                        self.camera = (self.camera + 1) % 3;
                        self.message = format!("Camera: {}", ["First person", "Third person", "Distant"][self.camera as usize]);
                        self.msg_timer = 2.0;
                    }
                    VK_SPACE => {
                        if self.mode == GameMode::Creative {
                            let now = self.plat.now();
                            if now - self.space_tap_t < 0.30 {
                                self.flying = !self.flying;
                                self.player.vy = 0.0;
                                self.space_tap_t = -10.0;
                            } else {
                                self.space_tap_t = now;
                            }
                        }
                    }
                    VK_W => {
                        let now = self.plat.now();
                        if now - self.w_tap_t < 0.30 { self.sprinting = true; }
                        self.w_tap_t = now;
                    }
                    _ => {}
                }
                if (0x31..=0x39).contains(&code) {
                    self.player.selected = (code - 0x31) as usize;
                }
            }
        }
    }

    // ------------------------------------------------------------------ menus
    fn set_mode(&mut self, mode: GameMode) {
        self.mode = mode;
        self.mining = None;
        match mode {
            GameMode::Survival => {
                self.flying = false;
                self.message = String::from("Game Mode: Survival");
            }
            GameMode::Creative => {
                self.flying = true;
                self.player.hp = 20.0;
                self.player.hunger = 20.0;
                self.message = String::from("Game Mode: Creative (E inventory, double-Space fly)");
            }
            GameMode::Spectator => {
                self.flying = true;
                self.message = String::from("Game Mode: Spectator (Fly through blocks, scroll speed)");
            }
        }
        self.msg_timer = 3.5;
    }

    fn handle_command(&mut self, cmd: &str) -> bool {
        let trim = cmd.trim();
        if !trim.starts_with('/') { return false; }
        let parts: Vec<&str> = trim.split_whitespace().collect();
        match parts.first().copied() {
            Some("/gamemode") | Some("/gm") => {
                if let Some(arg) = parts.get(1) {
                    match arg.to_ascii_lowercase().as_str() {
                        "survival" | "s" | "0" => self.set_mode(GameMode::Survival),
                        "creative" | "c" | "1" => self.set_mode(GameMode::Creative),
                        "spectator" | "sp" | "3" => self.set_mode(GameMode::Spectator),
                        _ => self.push_chat("Usage: /gamemode <survival|creative|spectator>"),
                    }
                } else {
                    let next = self.mode.next();
                    self.set_mode(next);
                }
                true
            }
            Some("/dim") | Some("/dimension") => {
                if let Some(arg) = parts.get(1) {
                    match arg.to_ascii_lowercase().as_str() {
                        "overworld" | "ow" | "0" => self.travel_to(Dimension::Overworld, self.spawn.0, self.spawn.1, self.spawn.2),
                        "nether" | "1" => {
                            let nx = (self.player.x / 8.0).floor() + 0.5;
                            let nz = (self.player.z / 8.0).floor() + 0.5;
                            self.travel_to(Dimension::Nether, nx, 36.0, nz);
                        }
                        "end" | "2" => self.travel_to(Dimension::End, 0.5, 62.5, -18.5),
                        _ => self.push_chat("Usage: /dim <overworld|nether|end>"),
                    }
                } else {
                    self.push_chat("Usage: /dim <overworld|nether|end>");
                }
                true
            }
            _ => {
                self.push_chat("Commands: /gamemode <survival|creative|spectator>, /dim <overworld|nether|end>");
                true
            }
        }
    }

    fn start_singleplayer(&mut self) {
        self.screen = Screen::Playing;
        self.ui = Ui::None;
        self.plat.set_capture(true);
        self.plat.set_cursor(false);
        self.plat.play(Sfx::LevelUp);
        self.set_mode(self.mode);
        if self.loaded_save {
            self.message = format!("Loaded world in {}", self.world.dim.name());
            self.msg_timer = 3.5;
        } else if self.mode == GameMode::Survival {
            self.message = String::from("Punch trees for wood! E inventory, F4 mode");
            self.msg_timer = 6.0;
        }
    }

    fn open_creative(&mut self) {
        self.ui = Ui::Creative;
        self.plat.set_capture(false);
        self.plat.set_cursor(true);
        self.plat.play(Sfx::Click);
        self.mouse_left = false;
    }

    fn join_game(&mut self) {
        self.net.close();
        let addr = self.net_addr.trim().to_string();
        if addr.is_empty() { return; }
        self.net = Net::join(&addr);
        self.screen = Screen::Playing;
        self.ui = Ui::None;
        self.plat.set_capture(true);
        self.plat.set_cursor(false);
        self.message = format!("Connecting to {}...", addr);
        self.msg_timer = 4.0;
    }

    fn host_game(&mut self) {
        self.net.close();
        self.net = Net::host(crate::net::DEFAULT_PORT);
        self.screen = Screen::Playing;
        self.ui = Ui::None;
        self.plat.set_capture(true);
        self.plat.set_cursor(false);
        self.message = String::from("LAN world open on port 25565");
        self.msg_timer = 5.0;
    }

    fn open_pause(&mut self) {
        self.ui = Ui::Pause;
        self.plat.set_capture(false);
        self.plat.set_cursor(true);
        self.mouse_left = false;
    }

    fn resume(&mut self) {
        self.ui = Ui::None;
        self.plat.set_capture(true);
        self.plat.set_cursor(false);
    }

    fn open_inventory(&mut self, table: bool) {
        self.ui = Ui::Inventory;
        self.craft3 = table;
        self.plat.set_capture(false);
        self.plat.set_cursor(true);
        self.plat.play(Sfx::Click);
        self.mouse_left = false;
    }

    fn close_ui(&mut self) {
        // return crafting grid contents
        for i in 0..GRID {
            let st = self.craft_grid[i];
            if !st.is_empty() {
                let left = self.player.add(st.id, st.count);
                if left > 0 { self.drops.push(Drop::new(self.player.x, self.player.y + 1.0, self.player.z, st.id)); }
                self.craft_grid[i] = Stack::EMPTY;
            }
        }
        if !self.cursor.is_empty() {
            let st = self.cursor;
            let left = self.player.add(st.id, st.count);
            if left > 0 { self.drops.push(Drop::new(self.player.x, self.player.y + 1.0, self.player.z, st.id)); }
            self.cursor = Stack::EMPTY;
        }
        self.ui = Ui::None;
        self.plat.set_capture(true);
        self.plat.set_cursor(false);
    }

    fn menu_layout(&self) -> Vec<(i32, i32, i32, i32)> {
        match self.screen {
            Screen::Title => {
                let bx = (render::W as i32 - 200) / 2;
                (0..5).map(|i| (bx, 116 + i as i32 * 26, 200, 21)).collect()
            }
            Screen::Multiplayer => {
                let (px, py) = (50, 20);
                vec![
                    (px + 10, py + 96, 170, 22),
                    (px + 200, py + 96, 170, 22),
                    (px + 10, py + 128, 170, 22),
                    (px + 200, py + 128, 170, 22),
                ]
            }
            Screen::Options => {
                let (px, py) = (70, 30);
                (0..4).map(|i| (px + 60, py + 44 + i * 30, 220, 22)).collect()
            }
            Screen::Playing => Vec::new(),
        }
    }

    fn click_menu(&mut self) {
        let (mx, my) = (self.mouse_x, self.mouse_y);
        let rects = self.menu_layout();
        for (i, (x, y, w, h)) in rects.iter().enumerate() {
            if mx >= *x && mx < *x + *w && my >= *y && my < *y + *h {
                self.menu_hover = i;
                match self.screen {
                    Screen::Title => match i {
                        0 => self.start_singleplayer(),
                        1 => { self.mode = self.mode.next(); }
                        2 => { self.screen = Screen::Multiplayer; }
                        3 => { self.screen = Screen::Options; }
                        _ => self.plat.request_quit(),
                    },
                    Screen::Multiplayer => match i {
                        0 => self.join_game(),
                        1 => self.host_game(),
                        2 => self.join_game(),
                        _ => { self.screen = Screen::Title; }
                    },
                    Screen::Options => match i {
                        0 => { self.sound_on = !self.sound_on; self.audio.sound_on = self.sound_on; self.plat.set_sound(self.sound_on); }
                        1 => { self.music_on = !self.music_on; self.audio.music_on = self.music_on; self.plat.set_music(self.music_on); }
                        2 => { self.pixel_scale = if self.pixel_scale >= 4 { 1 } else { self.pixel_scale + 1 }; let (sw, sh) = scene_dims(self.pixel_scale); self.scene = Framebuffer::new(sw, sh); }
                        _ => { if self.return_to_pause { self.return_to_pause = false; self.screen = Screen::Playing; self.ui = Ui::Pause; } else { self.screen = Screen::Title; } }
                    },
                    Screen::Playing => {}
                }
                return;
            }
        }
    }

    fn click_pause(&mut self) {
        let (mx, my) = (self.mouse_x, self.mouse_y);
        let bx = (render::W as i32 - 210) / 2;
        for i in 0..5 {
            let (x, y, w, h) = (bx, 76 + i as i32 * 26, 210, 20);
            if mx >= x && mx < x + w && my >= y && my < y + h {
                match i {
                    0 => self.resume(),
                    1 => { let next = self.mode.next(); self.set_mode(next); }
                    2 => {
                        self.sound_on = !self.sound_on; self.audio.sound_on = self.sound_on; self.plat.set_sound(self.sound_on);
                        self.music_on = self.sound_on; self.audio.music_on = self.music_on; self.plat.set_music(self.music_on);
                    }
                    3 => { self.return_to_pause = true; self.screen = Screen::Options; }
                    _ => {
                        self.save_world();
                        self.net.close();
                        self.screen = Screen::Title;
                        self.ui = Ui::None;
                        self.plat.set_cursor(true);
                        self.plat.set_capture(false);
                    }
                }
                return;
            }
        }
    }

    // ------------------------------------------------------------- inventory
    fn craft_result(&self) -> Stack {
        if let Some(r) = craft::match_recipe(&self.craft_grid) { return r; }
        Stack::EMPTY
    }

    fn slot_mut(&mut self, kind: usize, idx: usize) -> &mut Stack {
        match kind {
            0 => &mut self.craft_grid[idx],
            1 => &mut self.player.store[idx],
            2 => &mut self.player.hot[idx],
            _ => &mut self.cursor,
        }
    }

    fn creative_items_filtered(&self) -> Vec<u16> {
        if self.creative_tab == 5 {
            let q = self.creative_search.to_ascii_lowercase();
            items::all_creative_items()
                .into_iter()
                .filter(|&id| q.is_empty() || items::name(id).to_ascii_lowercase().contains(&q))
                .collect()
        } else {
            items::creative_tab_items(self.creative_tab).to_vec()
        }
    }

    fn creative_hover(&self) -> Option<(usize, usize)> {
        let (mx, my) = (self.mouse_x, self.mouse_y);
        for t in 0..12 {
            let (x, y, w, h) = render::creative_tab_rect(t);
            if mx >= x && mx < x + w && my >= y && my < y + h { return Some((0, t)); }
        }
        for i in 0..45 {
            let (x, y, w, h) = render::creative_grid_rect(i);
            if mx >= x && mx < x + w && my >= y && my < y + h { return Some((1, i)); }
        }
        for i in 0..HOT {
            let (x, y, w, h) = render::creative_hot_rect(i);
            if mx >= x && mx < x + w && my >= y && my < y + h { return Some((2, i)); }
        }
        let (ux, uy, uw, uh) = render::creative_scroll_up_rect();
        if mx >= ux && mx < ux + uw && my >= uy && my < uy + uh { return Some((3, 0)); }
        let (dx, dy, dw, dh) = render::creative_scroll_dn_rect();
        if mx >= dx && mx < dx + dw && my >= dy && my < dy + dh { return Some((3, 1)); }
        let (tx, ty, tw, th_r) = render::creative_trash_rect();
        if mx >= tx && mx < tx + tw && my >= ty && my < ty + th_r { return Some((4, 0)); }
        None
    }

    fn click_creative(&mut self, right: bool) {
        let Some(hv) = self.creative_hover() else { return; };
        let shift = self.keys.contains(&plat::VK_SHIFT);
        match hv {
            (0, t) => {
                self.plat.play(Sfx::Click);
                if t == 11 {
                    self.open_inventory(false);
                } else {
                    self.creative_tab = t;
                    self.creative_scroll = 0;
                    if t != 5 { self.creative_search.clear(); }
                }
            }
            (1, i) => {
                let list = self.creative_items_filtered();
                let idx = self.creative_scroll * 9 + i;
                if let Some(&id) = list.get(idx) {
                    let amt = if right { 1 } else { items::max_stack(id) };
                    if shift {
                        self.player.add(id, amt);
                    } else if self.cursor.id == id {
                        self.cursor.count = (self.cursor.count + amt).min(items::max_stack(id));
                    } else {
                        self.cursor = Stack::new(id, amt);
                    }
                    self.plat.play(Sfx::Click);
                }
            }
            (2, i) => {
                self.slot_click(right, 2, i);
            }
            (3, dir) => {
                let max_scroll = ((self.creative_items_filtered().len() + 8) / 9).saturating_sub(5);
                if dir == 0 { self.creative_scroll = self.creative_scroll.saturating_sub(1); }
                else { self.creative_scroll = (self.creative_scroll + 1).min(max_scroll); }
                self.plat.play(Sfx::Click);
            }
            (4, _) => {
                self.cursor = Stack::EMPTY;
                self.plat.play(Sfx::Click);
            }
            _ => {}
        }
    }

    fn click_inventory(&mut self, right: bool) {
        let (mx, my) = (self.mouse_x, self.mouse_y);
        let grid_size = if self.craft3 { 3 } else { 2 };

        // The furnace screen only has its three slots plus the hotbar.
        if self.ui == Ui::Furnace {
            for i in 0..3 {
                let (x, y, w, h) = render::furnace_slot_rect(i);
                if mx >= x && mx < x + w && my >= y && my < y + h {
                    self.furnace_click(right, i);
                    return;
                }
            }
            for i in 0..HOT {
                let (x, y, w, h) = render::hot_slot_rect(i);
                if mx >= x && mx < x + w && my >= y && my < y + h {
                    self.slot_click(right, 2, i);
                    return;
                }
            }
            return;
        }

        // crafting output
        let (rx, ry, rw, rh) = render::craft_result_rect(grid_size);
        if mx >= rx && mx < rx + rw && my >= ry && my < ry + rh {
            let result = self.craft_result();
            if !result.is_empty() {
                if self.cursor.is_empty() {
                    self.cursor = result;
                    craft::consume(&mut self.craft_grid);
                    self.plat.play(Sfx::Craft);
                } else if self.cursor.id == result.id && self.cursor.count + result.count <= 64 {
                    self.cursor.count += result.count;
                    craft::consume(&mut self.craft_grid);
                    self.plat.play(Sfx::Craft);
                }
            }
            return;
        }
        // crafting grid
        for i in 0..grid_size * grid_size {
            let (x, y, w, h) = render::craft_slot_rect(i, grid_size);
            if mx >= x && mx < x + w && my >= y && my < y + h {
                self.slot_click(right, 0, render::craft_grid_idx(i, grid_size));
                return;
            }
        }
        // recipe book controls
        let recs = craft::recipes();
        let pages = (recs.len() + 5) / 6;
        let (pvx, pvy, pvw, pvh) = render::recipe_prev_rect();
        if mx >= pvx && mx < pvx + pvw && my >= pvy && my < pvy + pvh {
            self.recipe_page = if self.recipe_page == 0 { pages.saturating_sub(1) } else { self.recipe_page - 1 };
            self.plat.play(Sfx::Click);
            return;
        }
        let (nvx, nvy, nvw, nvh) = render::recipe_next_rect();
        if mx >= nvx && mx < nvx + nvw && my >= nvy && my < nvy + nvh {
            self.recipe_page = (self.recipe_page + 1) % pages.max(1);
            self.plat.play(Sfx::Click);
            return;
        }
        for i in 0..6 {
            let (x, y, w, h) = render::recipe_slot_rect(i);
            if mx >= x && mx < x + w && my >= y && my < y + h {
                let idx = self.recipe_page * 6 + i;
                if idx < recs.len() {
                    self.recipe_sel = idx;
                    self.plat.play(Sfx::Click);
                }
                return;
            }
        }
        let (fx, fy, fw, fh) = render::recipe_fill_rect();
        if mx >= fx && mx < fx + fw && my >= fy && my < fy + fh {
            self.fill_recipe(self.recipe_sel);
            return;
        }
        // storage
        for i in 0..STORE {
            let (x, y, w, h) = render::inv_slot_rect(i);
            if mx >= x && mx < x + w && my >= y && my < y + h {
                self.slot_click(right, 1, i);
                return;
            }
        }
        // hotbar
        for i in 0..HOT {
            let (x, y, w, h) = render::hot_slot_rect(i);
            if mx >= x && mx < x + w && my >= y && my < y + h {
                self.slot_click(right, 2, i);
                return;
            }
        }
    }

    fn fill_recipe(&mut self, recipe_idx: usize) {
        let recs = craft::recipes();
        let Some(rec) = recs.get(recipe_idx) else { return; };
        let grid_size = if self.craft3 { 3 } else { 2 };
        if grid_size == 2 && !render::recipe_fits_2x2(&rec.pattern) {
            self.message = String::from("Place and use a Crafting Table for 3x3 recipes");
            self.msg_timer = 3.0;
            self.plat.play(Sfx::Click);
            return;
        }
        // Return existing grid items to inventory first.
        for g in 0..GRID {
            let st = self.craft_grid[g];
            if !st.is_empty() {
                let left = self.player.add(st.id, st.count);
                if left > 0 { self.drops.push(Drop::new(self.player.x, self.player.y + 1.0, self.player.z, st.id)); }
                self.craft_grid[g] = Stack::EMPTY;
            }
        }
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (3usize, 3usize, 0usize, 0usize);
        for y in 0..3 {
            for x in 0..3 {
                if rec.pattern[y * 3 + x] != 0 {
                    min_x = min_x.min(x); min_y = min_y.min(y);
                    max_x = max_x.max(x); max_y = max_y.max(y);
                }
            }
        }
        let h = max_y - min_y + 1;
        let w = max_x - min_x + 1;
        let mut missing = false;
        for r in 0..h {
            for c in 0..w {
                let need = rec.pattern[(min_y + r) * 3 + (min_x + c)];
                if need == 0 { continue; }
                let mut taken = false;
                for slot in self.player.hot.iter_mut().chain(self.player.store.iter_mut()) {
                    if !slot.is_empty() && slot.id == need {
                        slot.count -= 1;
                        if slot.count == 0 { *slot = Stack::EMPTY; }
                        taken = true;
                        break;
                    }
                }
                if taken {
                    self.craft_grid[r * 3 + c] = Stack::new(need, 1);
                } else {
                    missing = true;
                }
            }
        }
        if missing {
            self.message = format!("Missing some ingredients for {}", items::name(rec.out));
            self.msg_timer = 2.5;
        }
        self.plat.play(Sfx::Click);
    }

    fn slot_click(&mut self, right: bool, kind: usize, idx: usize) {
        let cur = self.cursor;
        let slot = *self.slot_mut(kind, idx);
        if right {
            if cur.is_empty() {
                if !slot.is_empty() {
                    let half = (slot.count + 1) / 2;
                    self.cursor = Stack::new(slot.id, half);
                    let s = self.slot_mut(kind, idx);
                    s.count -= half;
                    if s.count == 0 { *s = Stack::EMPTY; }
                    self.plat.play(Sfx::Click);
                }
            } else {
                let cid = cur.id;
                let s = self.slot_mut(kind, idx);
                if s.is_empty() {
                    *s = Stack::new(cid, 1);
                    self.cursor.count -= 1;
                    if self.cursor.count == 0 { self.cursor = Stack::EMPTY; }
                } else if s.id == cid && s.count < 64 {
                    s.count += 1;
                    self.cursor.count -= 1;
                    if self.cursor.count == 0 { self.cursor = Stack::EMPTY; }
                }
                self.plat.play(Sfx::Click);
            }
        } else if cur.is_empty() {
            if !slot.is_empty() {
                self.cursor = slot;
                *self.slot_mut(kind, idx) = Stack::EMPTY;
                self.plat.play(Sfx::Click);
            }
        } else if slot.is_empty() {
            *self.slot_mut(kind, idx) = cur;
            self.cursor = Stack::EMPTY;
            self.plat.play(Sfx::Click);
        } else if slot.id == cur.id {
            let s = self.slot_mut(kind, idx);
            let room = 64 - s.count;
            let put = room.min(cur.count);
            s.count += put;
            self.cursor.count -= put;
            if self.cursor.count == 0 { self.cursor = Stack::EMPTY; }
            self.plat.play(Sfx::Click);
        } else {
            *self.slot_mut(kind, idx) = cur;
            self.cursor = slot;
            self.plat.play(Sfx::Click);
        }
    }

    fn furnace_click(&mut self, right: bool, slot: usize) {
        let pos = self.ui_block;
        if !self.furnaces.contains_key(&pos) { self.furnaces.insert(pos, Furnace::new()); }
        let cur = self.cursor;
        let f = self.furnaces.get_mut(&pos).unwrap();
        let current = match slot { 0 => f.input, 1 => f.fuel, _ => f.output };
        if slot == 2 {
            if !current.is_empty() && (cur.is_empty() || (cur.id == current.id && cur.count + current.count <= 64)) {
                self.cursor = if cur.is_empty() { current } else { Stack::new(cur.id, cur.count + current.count) };
                f.output = Stack::EMPTY;
                self.plat.play(Sfx::Click);
            }
            return;
        }
        if cur.is_empty() {
            if !current.is_empty() {
                self.cursor = current;
                if slot == 0 { f.input = Stack::EMPTY; } else { f.fuel = Stack::EMPTY; }
                self.plat.play(Sfx::Click);
            }
        } else if current.is_empty() {
            let n = if right { 1 } else { cur.count };
            let placed = Stack::new(cur.id, n);
            if slot == 0 { f.input = placed; } else { f.fuel = placed; }
            self.cursor.count -= n;
            if self.cursor.count == 0 { self.cursor = Stack::EMPTY; }
            self.plat.play(Sfx::Click);
        } else if current.id == cur.id {
            let room = 64 - current.count;
            let put = if right { room.min(1) } else { room.min(cur.count) };
            if slot == 0 { f.input.count += put; } else { f.fuel.count += put; }
            self.cursor.count -= put;
            if self.cursor.count == 0 { self.cursor = Stack::EMPTY; }
            self.plat.play(Sfx::Click);
        }
    }

    // ------------------------------------------------------------ interaction
    fn look_dir(&self  ) -> (f32, f32, f32) {
        let cp = self.player.pitch.cos();
        (self.player.yaw.sin() * cp, self.player.pitch.sin(), -self.player.yaw.cos() * cp)
    }

    fn eye(&self) -> (f32, f32, f32) { let dy = if self.keys.contains(&plat::VK_SHIFT) { 1.48 } else { 1.62 }; (self.player.x, self.player.y + dy, self.player.z) }

    fn try_attack(&mut self) {
        if self.attack_cd > 0.0 { return; }
        self.attack_cd = 0.25;
        let (dx, dy, dz) = self.look_dir();
        let (ex, ey, ez) = self.eye();
        // mobs first
        let mut best: Option<(usize, f32)> = None;
        for (i, m) in self.mobs.iter().enumerate() {
            let h = m.kind.height();
            let ox = m.x - ex; let oy = (m.y + h * 0.5) - ey; let oz = m.z - ez;
            let proj = ox * dx + oy * dy + oz * dz;
            if proj <= 0.0 || proj > 4.0 { continue; }
            let cx = ox - dx * proj; let cy = oy - dy * proj; let cz = oz - dz * proj;
            if cx * cx + cy * cy + cz * cz < 0.6 * 0.6 && best.map_or(true, |(_, t)| proj < t) { best = Some((i, proj)); }
        }
        if let Some((i, _)) = best {
            let held_id = self.player.held().id;
            let dmg = match held_id {
                items::DIAMOND_SWORD => 7.0,
                items::IRON_SWORD => 6.0,
                items::WOOD_SWORD => 4.0,
                items::DIAMOND_AXE => 6.0,
                items::IRON_AXE => 5.0,
                items::WOOD_AXE => 3.5,
                _ => 1.0 + items::tool_tier(held_id) as f32 * 0.6,
            };
            self.mobs[i].hp -= dmg;
            self.mobs[i].hurt = 0.4;
            self.plat.play(Sfx::Hurt);
            if self.mobs[i].hp <= 0.0 {
                let m = self.mobs.remove(i);
                let id = m.kind.drop();
                if id != 0 {
                    self.drops.push(Drop::new(m.x, m.y + 0.4, m.z, id));
                    if m.kind == MobKind::Cow && self.tick % 2 == 0 {
                        self.drops.push(Drop::new(m.x + 0.2, m.y + 0.4, m.z, items::LEATHER));
                    }
                }
                self.message = format!("Defeated {}", m.kind.name());
                self.msg_timer = 1.5;
                self.plat.play(Sfx::LevelUp);
            }
            return;
        }
        self.mine_step(0.05);
    }

    fn mine_step(&mut self, speed_mul: f32) {
        if self.mode == GameMode::Spectator { self.mining = None; return; }
        let (dx, dy, dz) = self.look_dir();
        let (ex, ey, ez) = self.eye();
        let hit = world::raycast(&mut self.world, ex, ey, ez, dx, dy, dz, REACH, false);
        let Some(h) = hit else { self.mining = None; return; };
        if self.mode == GameMode::Creative {
            if self.creative_break_cd <= 0.0 {
                self.creative_break_cd = 0.18;
                self.mining = None;
                self.break_block(h.x, h.y, h.z, false);
            }
            return;
        }
        if matches!(h.block, world::BEDROCK | world::END_PORTAL_FRAME | world::END_PORTAL_FRAME_EYE | world::END_PORTAL) {
            self.mining = None;
            return;
        }
        let pos = (h.x, h.y, h.z);
        let held_id = self.player.held().id;
        let need = requires_pickaxe(h.block);
        let has_pick = items::pickaxe_tier(held_id) >= need;
        let is_wood = matches!(
            h.block,
            world::LOG | world::BIRCH_LOG | world::SPRUCE_LOG | world::JUNGLE_LOG | world::DARK_LOG
                | world::CRIMSON_STEM | world::WARPED_STEM | world::PLANKS | world::CRAFTING
                | world::CHEST | world::FENCE | world::BOOKSHELF
        );
        let is_dirt = matches!(
            h.block,
            world::GRASS | world::DIRT | world::SAND | world::RED_SAND | world::GRAVEL | world::CLAY
                | world::SNOW | world::PODZOL | world::MYCELIUM | world::FARMLAND | world::SOUL_SAND
        );
        let is_axe = matches!(held_id, items::WOOD_AXE | items::IRON_AXE | items::DIAMOND_AXE);
        let is_shovel = matches!(held_id, items::WOOD_SHOVEL | items::IRON_SHOVEL);

        let tool_bonus = if need > 0 {
            if has_pick { 1.5 + items::pickaxe_tier(held_id) as f32 * 1.1 } else { 0.22 }
        } else if is_wood && is_axe {
            1.6 + items::tool_tier(held_id) as f32 * 0.9
        } else if is_dirt && is_shovel {
            1.8 + items::tool_tier(held_id) as f32 * 1.0
        } else {
            1.0
        };
        let hardness = match h.block {
            world::OBSIDIAN | world::CRYING_OBSIDIAN | world::ANCIENT_DEBRIS => 28.0,
            world::IRON_BLOCK | world::GOLD_BLOCK | world::DIAMOND_BLOCK | world::EMERALD_BLOCK => 5.0,
            world::COAL_ORE | world::IRON_ORE | world::GOLD_ORE | world::DIAMOND_ORE
                | world::REDSTONE_ORE | world::EMERALD_ORE | world::NETHER_QUARTZ_ORE => 3.0,
            world::END_STONE => 3.0,
            world::NETHERRACK | world::CRIMSON_NYLIUM | world::WARPED_NYLIUM => 0.45,
            _ if need > 0 => 1.8,
            _ if is_wood => 2.0,
            _ if is_dirt => 0.65,
            world::LEAVES | world::BIRCH_LEAVES | world::SPRUCE_LEAVES | world::JUNGLE_LEAVES
                | world::DARK_LEAVES | world::CRIMSON_WART | world::WARPED_WART
                | world::GLASS | world::WOOL_WHITE | world::WOOL_RED | world::WOOL_BLUE
                | world::WOOL_GREEN | world::WOOL_YELLOW => 0.30,
            world::TALL_GRASS | world::FLOWER_RED | world::FLOWER_YELLOW
                | world::BROWN_MUSHROOM | world::RED_MUSHROOM | world::TORCH | world::WHEAT => 0.08,
            _ => 0.9,
        };
        let mut prog = match self.mining { Some((p, v)) if p == pos => v, _ => 0.0 };
        prog += speed_mul * tool_bonus / hardness;
        if prog >= 1.0 {
            self.mining = None;
            self.break_block(h.x, h.y, h.z, has_pick);
        } else {
            self.mining = Some((pos, prog));
        }
    }

    fn break_block(&mut self, x: i32, y: i32, z: i32, harvest_ok: bool) {
        let b = self.world.get(x, y, z);
        if b == world::AIR || b == world::BEDROCK { return; }
        self.world.set(x, y, z, world::AIR);
        self.net.send(format!("BLOCK|{}|{}|{}|0", x, y, z));
        self.plat.play(Sfx::Break);
        if harvest_ok {
            let drop = block_drop(b, self.tick);
            if !drop.is_empty() {
                let left = self.player.add(drop.id, drop.count);
                if left > 0 { self.drops.push(Drop::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5, drop.id)); }
            }
        } else if self.mode == GameMode::Survival {
            self.message = String::from("Need a better pickaxe to harvest that!");
            self.msg_timer = 1.8;
        }
        let above = self.world.get(x, y + 1, z);
        if matches!(above, world::TORCH | world::REDSTONE_TORCH | world::FLOWER_RED | world::FLOWER_YELLOW | world::TALL_GRASS) {
            self.world.set(x, y + 1, z, world::AIR);
        }
        self.furnaces.remove(&(x, y, z));
        self.levers.remove(&(x, y, z));
        self.redstone_sources.remove(&(x, y, z));
        self.redstone_dirty = true;
    }

    fn try_use(&mut self) {
        if self.place_cd > 0.0 { return; }
        self.place_cd = 0.2;
        let (dx, dy, dz) = self.look_dir();
        let (ex, ey, ez) = self.eye();
        let hit = world::raycast(&mut self.world, ex, ey, ez, dx, dy, dz, REACH, false);
        let Some(h) = hit else { return; };
        let target = (h.x, h.y, h.z);

        let sneaking = self.keys.contains(&plat::VK_SHIFT);
        // Shift + Right-click bypasses interactive blocks so you can place blocks on them (Minecraft sneak-place).
        if !sneaking {
            match h.block {
                world::CRAFTING => { self.open_inventory(true); return; }
                world::FURNACE => {
                    self.ui = Ui::Furnace;
                    self.ui_block = target;
                    self.furnaces.entry(target).or_insert_with(Furnace::new);
                    self.plat.set_capture(false);
                    self.plat.set_cursor(true);
                    self.plat.play(Sfx::Click);
                    return;
                }
                world::LEVER => {
                    if self.levers.contains(&target) { self.levers.remove(&target); } else { self.levers.insert(target); }
                    self.redstone_sources.insert(target);
                    self.plat.play(Sfx::Click);
                    self.redstone_dirty = true;
                    return;
                }
                world::TNT => {
                    self.world.set(h.x, h.y, h.z, world::AIR);
                    self.tnt_primed.push((h.x as f32 + 0.5, h.y as f32, h.z as f32 + 0.5, 3.5));
                    self.plat.play(Sfx::Furnace);
                    self.message = String::from("TNT primed!");
                    self.msg_timer = 1.5;
                    return;
                }
                world::BED => {
                    self.spawn = (h.x as f32 + 0.5, h.y as f32 + 1.0, h.z as f32 + 0.5);
                    let day = (self.time * std::f32::consts::TAU).sin() * 0.5 + 0.5;
                    if day < 0.45 {
                        self.time = 0.05;
                        self.message = String::from("Slept until morning - Respawn point set");
                    } else {
                        self.message = String::from("Respawn point set");
                    }
                    self.msg_timer = 2.5;
                    self.plat.play(Sfx::Click);
                    return;
                }
                world::CAKE => {
                    if self.player.hunger < 20.0 {
                        self.player.hunger = (self.player.hunger + 4.0).min(20.0);
                        self.plat.play(Sfx::Pickup);
                    }
                    return;
                }
                _ => {}
            }
        }

        let held = self.player.held();
        if held.is_empty() { return; }
        // Flint and Steel -> ignite Nether Portal inside an Obsidian frame!
        if held.id == items::FLINT_AND_STEEL {
            let px = h.x + h.nx;
            let py = h.y + h.ny;
            let pz = h.z + h.nz;
            if self.try_ignite_nether_portal(px, py, pz) {
                self.plat.play(Sfx::LevelUp);
                self.message = String::from("Nether Portal ignited!");
                self.msg_timer = 3.0;
            } else {
                self.plat.play(Sfx::Click);
                self.message = String::from("Build a closed Obsidian frame (at least 4x5) to ignite a Nether Portal!");
                self.msg_timer = 3.0;
            }
            return;
        }
        // Eye of Ender -> activate End Portal Frame or locate Stronghold!
        if held.id == items::EYE_OF_ENDER {
            if h.block == world::END_PORTAL_FRAME {
                self.world.set(h.x, h.y, h.z, world::END_PORTAL_FRAME_EYE);
                if self.mode == GameMode::Survival {
                    let s = &mut self.player.hot[self.player.selected];
                    s.count -= 1;
                    if s.count == 0 { *s = Stack::EMPTY; }
                }
                self.plat.play(Sfx::Place);
                if self.try_activate_end_portal(h.x, h.y, h.z) {
                    self.plat.play(Sfx::LevelUp);
                    self.message = String::from("End Portal opened!");
                    self.msg_timer = 4.0;
                }
            } else {
                let dx_s = 100.0 - self.player.x;
                let dz_s = -140.0 - self.player.z;
                let dist = (dx_s * dx_s + dz_s * dz_s).sqrt();
                self.message = format!("Eye of Ender points toward Stronghold (100, 19, -140) - {:.0}m away", dist);
                self.msg_timer = 4.0;
                self.plat.play(Sfx::Click);
            }
            return;
        }
        // Spawn Eggs (322..=331 and 336..=345)
        if (items::SPAWN_SHEEP..=items::SPAWN_VILLAGER).contains(&held.id)
            || (items::SPAWN_ZOMBIFIED_PIGLIN..=items::SPAWN_ENDER_DRAGON).contains(&held.id)
        {
            let kind = match held.id {
                items::SPAWN_SHEEP => MobKind::Sheep,
                items::SPAWN_COW => MobKind::Cow,
                items::SPAWN_PIG => MobKind::Pig,
                items::SPAWN_CHICKEN => MobKind::Chicken,
                items::SPAWN_ZOMBIE => MobKind::Zombie,
                items::SPAWN_SKELETON => MobKind::Skeleton,
                items::SPAWN_CREEPER => MobKind::Creeper,
                items::SPAWN_SPIDER => MobKind::Spider,
                items::SPAWN_ENDERMAN => MobKind::Enderman,
                items::SPAWN_VILLAGER => MobKind::Villager,
                items::SPAWN_ZOMBIFIED_PIGLIN => MobKind::ZombifiedPiglin,
                items::SPAWN_PIGLIN => MobKind::Piglin,
                items::SPAWN_GHAST => MobKind::Ghast,
                items::SPAWN_BLAZE => MobKind::Blaze,
                items::SPAWN_WITHER_SKELETON => MobKind::WitherSkeleton,
                items::SPAWN_SLIME => MobKind::Slime,
                items::SPAWN_WOLF => MobKind::Wolf,
                items::SPAWN_POLAR_BEAR => MobKind::PolarBear,
                items::SPAWN_PARROT => MobKind::Parrot,
                items::SPAWN_ENDER_DRAGON => MobKind::EnderDragon,
                _ => MobKind::Villager,
            };
            let mx = (h.x + h.nx) as f32 + 0.5;
            let my = (h.y + h.ny) as f32 + 0.05;
            let mz = (h.z + h.nz) as f32 + 0.5;
            let mut m = Mob::spawn(kind, mx, my, mz);
            m.yaw = (self.player.x - mx).atan2(self.player.z - mz);
            self.mobs.push(m);
            if self.mode == GameMode::Survival {
                let s = &mut self.player.hot[self.player.selected];
                s.count -= 1;
                if s.count == 0 { *s = Stack::EMPTY; }
            }
            self.plat.play(Sfx::Click);
            return;
        }
        // Ender Pearl teleport
        if held.id == items::ENDER_PEARL {
            if let Some(tp) = world::raycast(&mut self.world, ex, ey, ez, dx, dy, dz, 24.0, false) {
                self.player.x = (tp.x + tp.nx) as f32 + 0.5;
                self.player.y = (tp.y + tp.ny) as f32 + 0.2;
                self.player.z = (tp.z + tp.nz) as f32 + 0.5;
                self.fall_y = self.player.y;
                if self.mode == GameMode::Survival {
                    let s = &mut self.player.hot[self.player.selected];
                    s.count -= 1;
                    if s.count == 0 { *s = Stack::EMPTY; }
                }
                self.plat.play(Sfx::LevelUp);
            }
            return;
        }
        // eat food
        if items::is_food(held.id) > 0.0 {
            if self.player.hunger < 20.0 || held.id == items::GOLDEN_APPLE {
                self.player.hunger = (self.player.hunger + items::is_food(held.id) * 2.0).min(20.0);
                let heal = if held.id == items::GOLDEN_APPLE { 8.0 } else { 2.0 };
                self.player.hp = (self.player.hp + heal).min(20.0);
                if self.mode == GameMode::Survival {
                    let s = &mut self.player.hot[self.player.selected];
                    s.count -= 1;
                    if s.count == 0 { *s = Stack::EMPTY; }
                }
                self.plat.play(Sfx::Pickup);
                self.message = String::from("Yum!");
                self.msg_timer = 1.2;
            }
            return;
        }
        // place block
        if held.id <= world::MAX_BLOCK {
            let px = h.x + h.nx;
            let py = h.y + h.ny;
            let pz = h.z + h.nz;
            let existing = self.world.get(px, py, pz);
            if existing != world::AIR && existing != world::WATER && existing != world::TALL_GRASS { return; }
            let (fx, fy, fz) = (px as f32 + 0.5, py as f32, pz as f32 + 0.5);
            if (fx - self.player.x).abs() < 0.9 && (fz - self.player.z).abs() < 0.9
                && fy + 1.0 > self.player.y && fy < self.player.y + 1.8 {
                return;
            }
            self.world.set(px, py, pz, held.id);
            self.net.send(format!("BLOCK|{}|{}|{}|{}", px, py, pz, held.id));
            if self.mode == GameMode::Survival {
                let s = &mut self.player.hot[self.player.selected];
                s.count -= 1;
                if s.count == 0 { *s = Stack::EMPTY; }
            }
            self.plat.play(Sfx::Place);
            if held.id == world::FURNACE { self.furnaces.insert((px, py, pz), Furnace::new()); }
            if matches!(held.id, world::REDSTONE_TORCH | world::LEVER | world::PRESSURE_PLATE) {
                self.redstone_sources.insert((px, py, pz));
            }
            self.redstone_dirty = true;
        }
    }

    // --------------------------------------------------------------- redstone
    fn update_redstone(&mut self) {
        self.redstone_dirty = false;
        self.power.clear();
        self.lamps.clear();
        let mut queue: Vec<((i32, i32, i32), u8)> = Vec::new();
        let sources: Vec<(i32, i32, i32)> = self.redstone_sources.iter().copied().collect();
        for (x, y, z) in sources {
            let b = self.world.get(x, y, z);
            match b {
                world::REDSTONE_TORCH => queue.push(((x, y, z), 15)),
                world::LEVER => {
                    if self.levers.contains(&(x, y, z)) { queue.push(((x, y, z), 15)); }
                }
                world::PRESSURE_PLATE => {
                    let on = (self.player.x - (x as f32 + 0.5)).abs() < 0.85
                        && (self.player.z - (z as f32 + 0.5)).abs() < 0.85
                        && (self.player.y - y as f32).abs() < 1.2;
                    if on { self.plates.insert((x, y, z)); queue.push(((x, y, z), 15)); }
                    else { self.plates.remove(&(x, y, z)); }
                }
                _ => { self.redstone_sources.remove(&(x, y, z)); }
            }
        }
        let mut qi = 0;
        while qi < queue.len() {
            let ((x, y, z), p) = queue[qi];
            qi += 1;
            if self.power.get(&(x, y, z)).map_or(false, |v| *v >= p) { continue; }
            self.power.insert((x, y, z), p);
            for (dx, dy, dz) in [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)] {
                let n = (x + dx, y + dy, z + dz);
                let nb = self.world.get(n.0, n.1, n.2);
                if nb == world::LAMP { self.lamps.insert(n); }
                if nb == world::TNT {
                    self.world.set(n.0, n.1, n.2, world::AIR);
                    self.tnt_primed.push((n.0 as f32 + 0.5, n.1 as f32, n.2 as f32 + 0.5, 3.5));
                    self.plat.play(Sfx::Furnace);
                }
                if nb == world::REDSTONE_DUST && p > 1 {
                    let np = p - 1;
                    if self.power.get(&n).map_or(true, |v| *v < np) { queue.push((n, np)); }
                }
            }
        }
        self.world.lit = self.lamps.clone();
    }

    fn explode(&mut self, cx: f32, cy: f32, cz: f32, radius: f32) {
        self.plat.play(Sfx::Explode);
        let r = radius.ceil() as i32;
        let r2 = radius * radius;
        let (ix, iy, iz) = (cx.floor() as i32, cy.floor() as i32, cz.floor() as i32);
        for x in ix - r..=ix + r {
            for y in (iy - r).max(1)..=(iy + r).min(world::WORLD_H as i32 - 1) {
                for z in iz - r..=iz + r {
                    let d2 = (x as f32 + 0.5 - cx).powi(2)
                        + (y as f32 + 0.5 - cy).powi(2)
                        + (z as f32 + 0.5 - cz).powi(2);
                    if d2 <= r2 {
                        let b = self.world.get(x, y, z);
                        if b == world::TNT {
                            self.world.set(x, y, z, world::AIR);
                            self.tnt_primed.push((x as f32 + 0.5, y as f32, z as f32 + 0.5, 0.35));
                        } else if b != world::AIR && b != world::BEDROCK && b != world::OBSIDIAN {
                            self.world.set(x, y, z, world::AIR);
                            self.net.send(format!("BLOCK|{}|{}|{}|0", x, y, z));
                            self.furnaces.remove(&(x, y, z));
                            self.levers.remove(&(x, y, z));
                            self.redstone_sources.remove(&(x, y, z));
                        }
                    }
                }
            }
        }
        // damage player
        let pd = ((self.player.x - cx).powi(2) + (self.player.y + 0.9 - cy).powi(2) + (self.player.z - cz).powi(2)).sqrt();
        let blast_reach = radius * 1.8;
        if self.mode == GameMode::Survival && pd < blast_reach {
            let dmg = ((1.0 - pd / blast_reach) * 14.0).max(1.0);
            self.player.hp -= dmg;
            self.hurt_flash = 0.6;
            self.plat.play(Sfx::Hurt);
        }
        // damage mobs
        for m in self.mobs.iter_mut() {
            let md = ((m.x - cx).powi(2) + (m.y + 0.5 - cy).powi(2) + (m.z - cz).powi(2)).sqrt();
            if md < blast_reach {
                m.hp -= (1.0 - md / blast_reach) * 16.0;
                m.hurt = 0.5;
            }
        }
        self.redstone_dirty = true;
    }

    // --------------------------------------------------------------- simulation
    fn update_playing(&mut self, dt: f32) {
        let dt = dt.min(0.05);
        self.attack_cd = (self.attack_cd - dt).max(0.0);
        self.place_cd = (self.place_cd - dt).max(0.0);
        self.creative_break_cd = (self.creative_break_cd - dt).max(0.0);
        self.msg_timer = (self.msg_timer - dt).max(0.0);
        self.hurt_flash = (self.hurt_flash - dt * 2.5).max(0.0);
        self.grace = (self.grace - dt).max(0.0);
        self.hurt_cd = (self.hurt_cd - dt).max(0.0);
        for l in self.chat_lines.iter_mut() { l.1 += dt; }
        self.time = (self.time + dt * 0.0016) % 1.0;
        let day = (self.time * std::f32::consts::TAU).sin() * 0.5 + 0.5;

        // multiplayer pump
        self.net_timer += dt as f64;
        for e in self.net.poll() {
            match e {
                NetEvent::Status(s) => { self.net_status = s.clone(); self.push_chat(&s); }
                NetEvent::Error(s) => { self.message = s; self.msg_timer = 5.0; }
                NetEvent::Closed => { self.push_chat("Disconnected"); }
                NetEvent::Line(line) => self.net_line(&line),
            }
        }
        if self.net.connected && self.net_timer > 0.1 {
            self.net_timer = 0.0;
            self.net.send(format!(
                "POS|{:.2}|{:.2}|{:.2}|{:.3}|{:.3}",
                self.player.x, self.player.y, self.player.z, self.player.yaw, self.player.pitch
            ));
        }
        for (_, p) in self.remote.iter_mut() {
            p.4 = (p.4 + dt * 0.6) % 6.283;
        }

        let ui_open = self.ui == Ui::Inventory || self.ui == Ui::Creative || self.ui == Ui::Furnace;
        if !ui_open {
            let mut fwd: f32 = 0.0;
            let mut strafe: f32 = 0.0;
            if self.keys.contains(&plat::VK_W) { fwd += 1.0; } else { self.sprinting = false; }
            if self.keys.contains(&plat::VK_S) { fwd -= 1.0; }
            if self.keys.contains(&plat::VK_A) { strafe -= 1.0; }
            if self.keys.contains(&plat::VK_D) { strafe += 1.0; }
            let len = (fwd * fwd + strafe * strafe).sqrt().max(1.0);
            fwd /= len;
            strafe /= len;
            let sneaking = self.keys.contains(&plat::VK_SHIFT);
            let sprinting = (self.keys.contains(&plat::VK_CONTROL) || self.sprinting)
                && !sneaking && fwd > 0.0 && (self.mode != GameMode::Survival || self.player.hunger > 6.0);
            let (sy, cy) = (self.player.yaw.sin(), self.player.yaw.cos());
            if self.mode == GameMode::Spectator {
                let fly_sp = 9.0 * self.spec_speed * if sprinting { 1.6 } else { 1.0 };
                let (fx, fy, fz) = self.look_dir();
                self.player.x += (fx * fwd + cy * strafe) * fly_sp * dt;
                self.player.y += fy * fwd * fly_sp * dt;
                self.player.z += (fz * fwd + sy * strafe) * fly_sp * dt;
                if self.keys.contains(&plat::VK_SPACE) { self.player.y += fly_sp * dt; }
                if sneaking { self.player.y -= fly_sp * dt; }
                self.player.y = self.player.y.clamp(1.0, 120.0);
                self.player.vy = 0.0;
                self.fall_y = self.player.y;
            } else {
                let speed = if self.flying {
                    if sprinting { 11.0 } else { 7.8 }
                } else if sneaking {
                    2.0
                } else if sprinting {
                    6.2
                } else {
                    4.3
                };
                let vx = (sy * fwd + cy * strafe) * speed;
                let vz = (-cy * fwd + sy * strafe) * speed;

                let in_water = self.world.get(
                    self.player.x.floor() as i32,
                    (self.player.y + 0.4).floor() as i32,
                    self.player.z.floor() as i32,
                ) == world::WATER;

                let nx = self.player.x + vx * dt;
                if !collides(&mut self.world, nx, self.player.y, self.player.z) {
                    if self.flying || !sneaking || !self.player.on_ground || collides(&mut self.world, nx, self.player.y - 0.6, self.player.z) {
                        self.player.x = nx;
                    }
                }
                let nz = self.player.z + vz * dt;
                if !collides(&mut self.world, self.player.x, self.player.y, nz) {
                    if self.flying || !sneaking || !self.player.on_ground || collides(&mut self.world, self.player.x, self.player.y - 0.6, nz) {
                        self.player.z = nz;
                    }
                }
                if self.flying {
                    self.fall_y = self.player.y;
                    let mut vy = 0.0f32;
                    if self.keys.contains(&plat::VK_SPACE) { vy += 6.8; }
                    if sneaking { vy -= 6.8; }
                    self.player.vy = vy;
                } else if in_water {
                    self.fall_y = self.player.y;
                    if self.keys.contains(&plat::VK_SPACE) {
                        self.player.vy = 4.2;
                    } else {
                        self.player.vy = (self.player.vy - 6.0 * dt).max(-3.0);
                    }
                } else {
                    if self.keys.contains(&plat::VK_SPACE) && self.player.on_ground {
                        self.player.vy = 8.4;
                        self.player.on_ground = false;
                        self.plat.play(Sfx::Step);
                    }
                    self.player.vy -= 22.0 * dt;
                }
                let ny = self.player.y + self.player.vy * dt;
                if collides(&mut self.world, self.player.x, ny, self.player.z) {
                    if self.player.vy < 0.0 {
                        if self.flying {
                            self.flying = false;
                        } else if !self.player.on_ground && !in_water && self.mode == GameMode::Survival {
                            let fall_dist = self.fall_y - self.player.y;
                            if fall_dist > 3.5 && self.grace <= 0.0 {
                                let dmg = (fall_dist - 3.0).floor();
                                self.player.hp -= dmg;
                                self.hurt_flash = 0.45;
                                self.plat.play(Sfx::Hurt);
                            }
                        }
                        self.player.on_ground = true;
                        self.fall_y = self.player.y;
                    }
                    self.player.vy = 0.0;
                } else {
                    self.player.y = ny;
                    self.player.on_ground = false;
                    if self.player.y > self.fall_y { self.fall_y = self.player.y; }
                }
                if (vx.abs() + vz.abs()) > 0.1 && self.player.on_ground && !sneaking && !self.flying {
                    self.audio_timer += dt as f64;
                    if self.audio_timer > if sprinting { 0.28 } else { 0.42 } {
                        self.audio_timer = 0.0;
                        self.plat.play(Sfx::Step);
                    }
                }
                if sprinting && self.mode == GameMode::Survival {
                    self.player.hunger = (self.player.hunger - dt * 0.12).max(0.0);
                }
                if self.mouse_left { self.mine_step(dt); }
            }
        }

        // mobs + creeper blasts + Ender Dragon victory
        let mut creeper_blasts: Vec<(f32, f32, f32)> = Vec::new();
        let mut dragon_killed = false;
        for m in self.mobs.iter_mut() {
            m.update(&mut self.world, self.player.x, self.player.y, self.player.z, dt, self.tick, day, self.grace, self.mode != GameMode::Survival);
            if m.kind == MobKind::Creeper && m.fuse >= 1.5 {
                m.hp = 0.0;
                creeper_blasts.push((m.x, m.y + 0.5, m.z));
            }
            if m.kind == MobKind::EnderDragon && m.hp <= 0.0 {
                dragon_killed = true;
            }
        }
        if dragon_killed && !self.dragon_defeated {
            self.dragon_defeated = true;
            self.message = String::from("The Ender Dragon is defeated! Enter the Exit Portal at (0, 61, 0)!");
            self.msg_timer = 6.0;
            self.plat.play(Sfx::LevelUp);
            if self.world.dim == Dimension::End {
                for ox in -1..=1 {
                    for oz in -1..=1 {
                        if ox != 0 || oz != 0 {
                            self.world.set(ox, 61, oz, world::END_PORTAL);
                        }
                    }
                }
            }
        }
        for (bx, by, bz) in creeper_blasts {
            self.explode(bx, by, bz, 3.2);
        }
        self.mobs.retain(|m| m.hp > 0.0 && (m.x - self.player.x).abs() < 75.0 && (m.z - self.player.z).abs() < 75.0);
        for m in self.mobs.iter_mut() {
            if m.kind.hostile() && m.kind != MobKind::Creeper {
                let d = ((m.x - self.player.x).powi(2) + (m.y - self.player.y).powi(2) + (m.z - self.player.z).powi(2)).sqrt();
                if self.mode == GameMode::Survival && d < 1.8 && m.attack_cd <= 0.0 && self.hurt_cd <= 0.0 && self.grace <= 0.0 {
                    self.player.hp -= 2.0;
                    self.hurt_flash = 0.4;
                    self.hurt_cd = 0.7;
                    m.attack_cd = 1.2;
                    self.plat.play(Sfx::Hurt);
                }
            }
        }

        // primed TNT countdown
        let mut tnt_blasts: Vec<(f32, f32, f32)> = Vec::new();
        for t in self.tnt_primed.iter_mut() {
            t.3 -= dt;
            if t.3 <= 0.0 { tnt_blasts.push((t.0, t.1 + 0.5, t.2)); }
        }
        self.tnt_primed.retain(|t| t.3 > 0.0);
        for (bx, by, bz) in tnt_blasts {
            self.explode(bx, by, bz, 4.0);
        }

        // spawn mobs
        self.spawn_timer += dt;
        if self.spawn_timer > 1.8 {
            self.spawn_timer = 0.0;
            if self.mobs.len() < 26 {
                let a = (self.tick as f32 * 0.618).sin() * 6.283;
                let d = 22.0 + (self.tick % 17) as f32;
                let x = self.player.x + a.cos() * d;
                let z = self.player.z + a.sin() * d;
                let sy_m = self.world.highest_ground(x as i32, z as i32) as f32;
                let ground_b = self.world.get(x as i32, sy_m as i32 - 1, z as i32);
                let k = if self.world.dim == Dimension::Overworld && matches!(ground_b, world::FARMLAND | world::PLANKS) && day > 0.45 {
                    MobKind::Villager
                } else {
                    let bm = self.world.biome(x as i32, z as i32);
                    entities::biome_mob(bm, self.world.dim, self.tick.wrapping_mul(7919), day)
                };
                self.mobs.push(Mob::spawn(k, x, sy_m, z));
            }
        }

        // drops
        for d in self.drops.iter_mut() { d.update(&mut self.world, dt); }
        let mut picked: Vec<usize> = Vec::new();
        for (i, d) in self.drops.iter().enumerate() {
            let dist = ((d.x - self.player.x).powi(2) + (d.y - self.player.y - 0.8).powi(2) + (d.z - self.player.z).powi(2)).sqrt();
            if dist < 1.8 { picked.push(i); }
        }
        for i in picked.into_iter().rev() {
            let d = self.drops.remove(i);
            let left = self.player.add(d.item, 1);
            if left == 0 { self.plat.play(Sfx::Pickup); }
        }
        self.drops.retain(|d| (d.x - self.player.x).abs() < 50.0);

        // furnaces
        let mut keys: Vec<(i32, i32, i32)> = self.furnaces.keys().copied().collect();
        keys.sort();
        for k in keys {
            let mut done = false;
            if let Some(f) = self.furnaces.get_mut(&k) {
                let can_smelt = !f.input.is_empty()
                    && craft::smelt_result(f.input.id).is_some()
                    && (f.output.is_empty() || (f.output.id == craft::smelt_result(f.input.id).unwrap() && f.output.count < 64));
                if f.burn <= 0.0 && can_smelt && !f.fuel.is_empty() {
                    let t = craft::fuel_time(f.fuel.id);
                    if t > 0.0 {
                        f.burn = t;
                        f.burn_max = t;
                        f.fuel.count -= 1;
                        if f.fuel.count == 0 { f.fuel = Stack::EMPTY; }
                    }
                }
                if f.burn > 0.0 {
                    f.burn -= dt;
                    if can_smelt {
                        f.cook += dt;
                        if f.cook >= 8.0 {
                            f.cook = 0.0;
                            let out = craft::smelt_result(f.input.id).unwrap();
                            f.output = if f.output.is_empty() { Stack::new(out, 1) } else { Stack::new(out, f.output.count + 1) };
                            f.input.count -= 1;
                            if f.input.count == 0 { f.input = Stack::EMPTY; }
                            done = true;
                        }
                    } else {
                        f.cook = 0.0;
                    }
                    if f.burn <= 0.0 { f.burn_max = 0.0; }
                }
            }
            if done { self.plat.play(Sfx::Furnace); }
        }

        // Only update redstone when dirty or when stepping on/off a pressure plate.
        let (px, py, pz) = (self.player.x.floor() as i32, self.player.y.floor() as i32, self.player.z.floor() as i32);
        if self.world.get(px, py, pz) == world::PRESSURE_PLATE
            || self.world.get(px, py - 1, pz) == world::PRESSURE_PLATE
            || !self.plates.is_empty()
        {
            self.redstone_dirty = true;
        }
        if self.redstone_dirty { self.update_redstone(); }

        // hunger + health regeneration (Survival only)
        if self.mode != GameMode::Survival {
            self.player.hp = 20.0;
            self.player.hunger = 20.0;
        } else {
        self.player.hunger = (self.player.hunger - dt * 0.035).max(0.0);
        if self.player.hunger <= 0.5 && self.hurt_cd <= 0.0 && self.player.hp > 1.0 {
            self.player.hp = (self.player.hp - 1.0).max(1.0);
            self.hurt_cd = 4.0;
            self.hurt_flash = 0.25;
            self.plat.play(Sfx::Hurt);
        }
        if self.player.hp < 20.0 && self.player.hunger >= 18.0 {
            self.regen += dt;
            if self.regen >= 4.0 {
                self.regen = 0.0;
                self.player.hp = (self.player.hp + 1.0).min(20.0);
                self.player.hunger = (self.player.hunger - 0.5).max(0.0);
            }
        } else {
            self.regen = 0.0;
        }
        }

        // death
        if self.player.hp <= 0.0 {
            self.player.hp = 20.0;
            self.player.hunger = 20.0;
            self.world.set_dim(Dimension::Overworld);
            self.player.x = self.spawn.0;
            self.player.y = self.spawn.1;
            self.player.z = self.spawn.2;
            self.fall_y = self.spawn.1;
            self.player.vy = 0.0;
            self.message = String::from("You died! Respawning...");
            self.msg_timer = 4.0;
            self.grace = 6.0;
            self.plat.play(Sfx::Hurt);
        }
        // Dimension portal travel
        self.portal_cd = (self.portal_cd - dt).max(0.0);
        if self.portal_cd <= 0.0 {
            let feet_b = self.world.get(px, py, pz);
            let head_b = self.world.get(px, py + 1, pz);
            if feet_b == world::NETHER_PORTAL || head_b == world::NETHER_PORTAL {
                if self.world.dim == Dimension::Overworld {
                    let nx = (self.player.x / 8.0).floor() + 0.5;
                    let nz = (self.player.z / 8.0).floor() + 0.5;
                    self.travel_to(Dimension::Nether, nx, 36.0, nz);
                } else if self.world.dim == Dimension::Nether {
                    let ox = (self.player.x * 8.0).floor() + 0.5;
                    let oz = (self.player.z * 8.0).floor() + 0.5;
                    let oy = self.world.highest_ground(ox as i32, oz as i32) as f32 + 0.2;
                    self.travel_to(Dimension::Overworld, ox, oy, oz);
                }
            } else if feet_b == world::END_PORTAL || self.world.get(px, py - 1, pz) == world::END_PORTAL {
                if self.world.dim == Dimension::End {
                    self.travel_to(Dimension::Overworld, self.spawn.0, self.spawn.1, self.spawn.2);
                } else {
                    self.travel_to(Dimension::End, 0.5, 62.5, -18.5);
                }
            }
        }
        // Auto-save every 20 seconds
        self.save_timer += dt;
        if self.save_timer >= 20.0 {
            self.save_timer = 0.0;
            self.save_world();
        }
        self.world.prune(self.player.x as i32 / 16, self.player.z as i32 / 16);
    }

    fn push_chat(&mut self, text: &str) {
        self.chat_lines.push((text.to_string(), 0.0));
        if self.chat_lines.len() > 40 { self.chat_lines.remove(0); }
    }

    fn net_line(&mut self, line: &str) {
        let parts: Vec<&str> = line.split('|').collect();
        match parts.first().copied() {
            Some("POS") if parts.len() >= 6 => {
                if let (Ok(x), Ok(y), Ok(z), Ok(yaw), Ok(pitch)) = (
                    parts[1].parse::<f32>(), parts[2].parse::<f32>(), parts[3].parse::<f32>(),
                    parts[4].parse::<f32>(), parts[5].parse::<f32>(),
                ) {
                    let id = parts.get(6).and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);
                    self.remote.insert(id, (x, y, z, yaw, pitch));
                }
            }
            Some("BLOCK") if parts.len() >= 5 => {
                if let (Ok(x), Ok(y), Ok(z), Ok(id)) = (
                    parts[1].parse::<i32>(), parts[2].parse::<i32>(), parts[3].parse::<i32>(), parts[4].parse::<u16>(),
                ) {
                    self.world.set(x, y, z, id);
                }
            }
            Some("CHAT") if parts.len() >= 2 => {
                let msg = parts[2..].join("|");
                self.push_chat(&msg);
                self.plat.play(Sfx::Click);
            }
            _ => {}
        }
    }

    // -------------------------------------------------------------- rendering
    fn draw_title(&mut self) {
        if self.ui != Ui::None { self.ui = Ui::None; }
        let hover = self.hover_index();
        let gm_label = format!("Game Mode: {}", self.mode.name());
        render::draw_title(&mut self.fb, &["Singleplayer", &gm_label, "Multiplayer", "Options", "Quit"], hover, &self.splash, self.title_t);
        self.plat.set_capture(false);
        self.plat.set_cursor(true);
    }

    fn hover_index(&self) -> usize {
        let (mx, my) = (self.mouse_x, self.mouse_y);
        for (i, (x, y, w, h)) in self.menu_layout().iter().enumerate() {
            if mx >= *x && mx < *x + *w && my >= *y && my < *y + *h { return i; }
        }
        usize::MAX
    }

    fn draw_multiplayer(&mut self) {
        let hover = self.hover_index();
        let status = format!("{}  ({})", self.net_status, if self.net.hosting { "hosting" } else { "offline" });
        render::draw_multiplayer_screen(&mut self.fb, &self.net_addr, &status, hover, self.net.hosting);
        self.plat.set_capture(false);
        self.plat.set_cursor(true);
    }

    fn draw_options(&mut self) {
        let hover = self.hover_index();
        render::draw_options_screen(&mut self.fb, self.sound_on, self.music_on, self.pixel_scale, hover);
        self.plat.set_capture(false);
        self.plat.set_cursor(true);
    }

    fn draw_playing(&mut self) {
        let day = (self.time * std::f32::consts::TAU).sin() * 0.5 + 0.5;
        // camera
        let (fx, fy, fz) = self.look_dir();
        let eye_y = self.eye().1;
        let (cam_x, cam_y, cam_z) = if self.camera == 0 {
            (self.player.x, eye_y, self.player.z)
        } else {
            let back = if self.camera == 1 { 3.6 } else { 6.5 };
            let tx = self.player.x - fx * back;
            let ty = eye_y - fy * back - 0.35;
            let tz = self.player.z - fz * back;
            if let Some(h) = world::raycast(&mut self.world, self.player.x, eye_y, self.player.z, -fx, -fy, -fz, back + 0.5, false) {
                if h.dist > 1.0 {
                    let min_back = if self.camera == 1 { 2.4 } else { 4.6 };
                    let f = ((h.dist - 0.35).max(min_back) / back).min(1.0);
                    (self.player.x - fx * back * f, eye_y - fy * back * f - 0.35 * f, self.player.z - fz * back * f)
                } else { (tx, ty, tz) }
            } else { (tx, ty, tz) }
        };
        let cam = Cam::from(self.player.yaw, self.player.pitch, cam_x, cam_y, cam_z);
        self.scene.clear();
        self.fb.cursor = (self.mouse_x, self.mouse_y);
        let mining_overlay = self.mining.map(|((mx, my, mz), prog)| {
            (mx, my, mz, (prog * 10.0).clamp(0.0, 9.0) as u8)
        });
        render::render_world(&mut self.scene, &self.tex, &mut self.world, &cam, self.time, self.tick, mining_overlay);
        if self.mode != GameMode::Spectator && self.ui == Ui::None {
            if let Some(h) = world::raycast(&mut self.world, self.player.x, eye_y, self.player.z, fx, fy, fz, REACH, false) {
                render::draw_block_outline(&mut self.scene, &cam, h.x, h.y, h.z);
            }
        }
        if self.camera != 0 && self.mode != GameMode::Spectator {
            render::draw_entity_3d(&mut self.scene, &cam, self.player.x, self.player.y, self.player.z, self.player.yaw, self.title_t * 2.0, day, EntityKind::Steve, false, true);
        }
        for (_, (x, y, z, yaw, _)) in self.remote.iter() {
            render::draw_entity_3d(&mut self.scene, &cam, *x, *y, *z, *yaw, self.title_t * 2.0, day, EntityKind::Steve, false, false);
        }
        for m in self.mobs.iter() {
            let flash = m.hurt > 0.15 || (m.kind == MobKind::Creeper && m.fuse > 0.0 && (self.tick / 3) % 2 == 0);
            render::draw_entity_3d(&mut self.scene, &cam, m.x, m.y, m.z, m.yaw, m.walk, day, EntityKind::Mob(m.kind), flash, false);
        }
        for t in self.tnt_primed.iter() {
            let id = if (self.tick / 4) % 2 == 0 { world::SNOW } else { world::TNT };
            render::draw_drop_sprite(&mut self.scene, &cam, t.0, t.1, t.2, id, self.tick, day);
        }
        for d in self.drops.iter() {
            render::draw_drop_sprite(&mut self.scene, &cam, d.x, d.y, d.z, d.item, self.tick, day);
        }
        render::upscale(&self.scene, &mut self.fb);

        if self.ui == Ui::None || self.ui == Ui::Chat {
            if self.camera == 0 && self.mode != GameMode::Spectator {
                render::draw_held_item(&mut self.fb, self.player.hot[self.player.selected].id, self.tick);
            }
            let dragon_hp = if self.world.dim == Dimension::End {
                self.mobs.iter().find(|m| m.kind == MobKind::EnderDragon).map(|m| m.hp)
            } else { None };
            render::draw_hud(
                &mut self.fb,
                &self.player.hot.map(|s| s.id),
                &self.player.hot.map(|s| s.count),
                self.player.selected,
                self.player.hp,
                self.player.hunger,
                self.tick,
                &self.message,
                self.msg_timer,
                self.world.biome(self.player.x.floor() as i32, self.player.z.floor() as i32),
                self.world.dim,
                dragon_hp,
                (self.player.x, self.player.y, self.player.z),
                self.fps,
                self.camera,
                self.time,
                self.mode,
            );
            // crosshair progress
            if let Some((_, prog)) = self.mining {
                let (cx, cy) = (render::W as i32 / 2, render::H as i32 / 2);
                self.fb.rect(cx - 10, cy + 8, 20, 2, [40, 40, 40, 200]);
                self.fb.rect(cx - 10, cy + 8, (prog * 20.0) as i32, 2, [240, 240, 240, 255]);
            }
            render::draw_chat(&mut self.fb, &self.chat_lines, if self.ui == Ui::Chat { Some(self.chat_input.as_str()) } else { None }, self.tick as f32);
            if self.hurt_flash > 0.0 {
                let a = (self.hurt_flash * 140.0) as u8;
                self.fb.rect(0, 0, render::W as i32, render::H as i32, [180, 20, 20, a]);
            }
        }
        if self.ui == Ui::Creative {
            let list = self.creative_items_filtered();
            let hover = self.creative_hover();
            render::draw_creative_screen(
                &mut self.fb, &self.player.hot, self.cursor, self.creative_tab,
                self.creative_scroll, &self.creative_search, &list, self.tick, hover,
            );
        }
        if self.ui == Ui::Inventory {
            let grid_size = if self.craft3 { 3 } else { 2 };
            let result = self.craft_result();
            let hover = self.inv_hover(grid_size);
            render::draw_inventory_screen(
                &mut self.fb, &self.player.hot, &self.player.store, &self.craft_grid,
                grid_size, result, self.cursor, self.tick, hover, self.recipe_page, self.recipe_sel,
            );
        }
        if self.ui == Ui::Furnace {
            let f = self.furnaces.get(&self.ui_block).cloned().unwrap_or_else(Furnace::new);
            let burn = if f.burn_max > 0.0 { (f.burn / f.burn_max).max(0.0) } else { 0.0 };
            let hover = self.furnace_hover();
            render::draw_furnace_screen(
                &mut self.fb, f.input, f.fuel, f.output, burn, (f.cook / 8.0).min(1.0),
                &self.player.hot, self.cursor, self.tick, hover,
            );
        }
        if self.ui == Ui::Pause {
            let (mx, my) = (self.mouse_x, self.mouse_y);
            let bx = (render::W as i32 - 210) / 2;
            let mut hover = usize::MAX;
            for i in 0..5 {
                let (x, y, w, h) = (bx, 76 + i as i32 * 26, 210, 20);
                if mx >= x && mx < x + w && my >= y && my < y + h { hover = i; }
            }
            render::draw_pause(&mut self.fb, hover, self.mode, self.sound_on, self.music_on);
        }
        if self.ui == Ui::None {
            self.plat.set_cursor(false);
        }
        // keep mouse locked while playing
        if self.ui == Ui::None || self.ui == Ui::Chat {
            self.plat.set_capture(true);
        }
    }

    fn inv_hover(&self, grid_size: usize) -> Option<(usize, usize)> {
        let (mx, my) = (self.mouse_x, self.mouse_y);
        for i in 0..grid_size * grid_size {
            let (x, y, w, h) = render::craft_slot_rect(i, grid_size);
            if mx >= x && mx < x + w && my >= y && my < y + h { return Some((0, i)); }
        }
        let (rx, ry, rw, rh) = render::craft_result_rect(grid_size);
        if mx >= rx && mx < rx + rw && my >= ry && my < ry + rh { return Some((0, 99)); }
        let (pvx, pvy, pvw, pvh) = render::recipe_prev_rect();
        if mx >= pvx && mx < pvx + pvw && my >= pvy && my < pvy + pvh { return Some((3, 100)); }
        let (nvx, nvy, nvw, nvh) = render::recipe_next_rect();
        if mx >= nvx && mx < nvx + nvw && my >= nvy && my < nvy + nvh { return Some((3, 101)); }
        let (fx, fy, fw, fh) = render::recipe_fill_rect();
        if mx >= fx && mx < fx + fw && my >= fy && my < fy + fh { return Some((3, 102)); }
        for i in 0..6 {
            let (x, y, w, h) = render::recipe_slot_rect(i);
            if mx >= x && mx < x + w && my >= y && my < y + h { return Some((3, i)); }
        }
        for i in 0..STORE {
            let (x, y, w, h) = render::inv_slot_rect(i);
            if mx >= x && mx < x + w && my >= y && my < y + h { return Some((1, i)); }
        }
        for i in 0..HOT {
            let (x, y, w, h) = render::hot_slot_rect(i);
            if mx >= x && mx < x + w && my >= y && my < y + h { return Some((2, i)); }
        }
        None
    }

    fn furnace_hover(&self) -> Option<(usize, usize)> {
        let (mx, my) = (self.mouse_x, self.mouse_y);
        for i in 0..3 {
            let (x, y, w, h) = render::furnace_slot_rect(i);
            if mx >= x && mx < x + w && my >= y && my < y + h { return Some((0, i)); }
        }
        for i in 0..HOT {
            let (x, y, w, h) = render::hot_slot_rect(i);
            if mx >= x && mx < x + w && my >= y && my < y + h { return Some((1, i)); }
        }
        None
    }

    // --------------------------------------------------- dimensions & portals
    fn try_ignite_nether_portal(&mut self, ax: i32, ay: i32, az: i32) -> bool {
        for axis_x in [true, false] {
            let mut min_h = 0i32;
            while min_h > -8 {
                let (cx, cz) = if axis_x { (ax + min_h, az) } else { (ax, az + min_h) };
                if self.world.get(cx, ay, cz) == world::OBSIDIAN { break; }
                min_h -= 1;
            }
            let mut max_h = 0i32;
            while max_h < 8 {
                let (cx, cz) = if axis_x { (ax + max_h, az) } else { (ax, az + max_h) };
                if self.world.get(cx, ay, cz) == world::OBSIDIAN { break; }
                max_h += 1;
            }
            let width = max_h - min_h - 1;
            if width < 2 || width > 6 { continue; }
            let mut min_y = ay;
            while min_y > ay - 8 && self.world.get(ax, min_y, az) != world::OBSIDIAN { min_y -= 1; }
            let mut max_y = ay;
            while max_y < ay + 8 && self.world.get(ax, max_y, az) != world::OBSIDIAN { max_y += 1; }
            let height = max_y - min_y - 1;
            if height < 3 || height > 6 { continue; }
            let mut valid = true;
            for h_off in (min_h + 1)..max_h {
                let (cx, cz) = if axis_x { (ax + h_off, az) } else { (ax, az + h_off) };
                if self.world.get(cx, min_y, cz) != world::OBSIDIAN || self.world.get(cx, max_y, cz) != world::OBSIDIAN {
                    valid = false; break;
                }
                for y in (min_y + 1)..max_y {
                    let b = self.world.get(cx, y, cz);
                    if b != world::AIR && b != world::NETHER_PORTAL { valid = false; break; }
                }
                if !valid { break; }
            }
            for y in (min_y + 1)..max_y {
                let (lx, lz) = if axis_x { (ax + min_h, az) } else { (ax, az + min_h) };
                let (rx, rz) = if axis_x { (ax + max_h, az) } else { (ax, az + max_h) };
                if self.world.get(lx, y, lz) != world::OBSIDIAN || self.world.get(rx, y, rz) != world::OBSIDIAN {
                    valid = false; break;
                }
            }
            if valid {
                for h_off in (min_h + 1)..max_h {
                    let (cx, cz) = if axis_x { (ax + h_off, az) } else { (ax, az + h_off) };
                    for y in (min_y + 1)..max_y {
                        self.world.set(cx, y, cz, world::NETHER_PORTAL);
                    }
                }
                return true;
            }
        }
        false
    }

    fn try_activate_end_portal(&mut self, fx: i32, fy: i32, fz: i32) -> bool {
        // Scan nearby 5x5 centers to find a complete 3x3 ring of END_PORTAL_FRAME_EYE
        for cx in (fx - 3)..=(fx + 3) {
            for cz in (fz - 3)..=(fz + 3) {
                let mut ok = true;
                for ox in -1..=1 {
                    for oz in -1..=1 {
                        if self.world.get(cx + ox, fy, cz + oz) == world::END_PORTAL_FRAME
                            || self.world.get(cx + ox, fy, cz + oz) == world::END_PORTAL_FRAME_EYE
                        {
                            ok = false;
                        }
                    }
                }
                if !ok { continue; }
                for d in -1..=1 {
                    let ring = [
                        (cx - 2, cz + d), (cx + 2, cz + d),
                        (cx + d, cz - 2), (cx + d, cz + 2),
                    ];
                    for (rx, rz) in ring {
                        if self.world.get(rx, fy, rz) != world::END_PORTAL_FRAME_EYE {
                            ok = false; break;
                        }
                    }
                    if !ok { break; }
                }
                if ok {
                    for ox in -1..=1 {
                        for oz in -1..=1 {
                            self.world.set(cx + ox, fy, cz + oz, world::END_PORTAL);
                        }
                    }
                    return true;
                }
            }
        }
        false
    }

    fn travel_to(&mut self, target_dim: Dimension, tx: f32, ty: f32, tz: f32) {
        self.world.set_dim(target_dim);
        self.portal_cd = 3.5;
        self.player.vy = 0.0;
        let ix = tx.floor() as i32;
        let iy = ty.floor() as i32;
        let iz = tz.floor() as i32;

        if target_dim == Dimension::Nether {
            // Ensure a safe Obsidian platform & return Nether Portal in The Nether
            for ox in -2..=2 {
                for oz in -2..=2 {
                    self.world.set(ix + ox, iy - 1, iz + oz, world::OBSIDIAN);
                    for oy in 0..=4 {
                        self.world.set(ix + ox, iy + oy, iz + oz, world::AIR);
                    }
                }
            }
            for ox in -1..=2 {
                self.world.set(ix + ox, iy - 1, iz + 2, world::OBSIDIAN);
                self.world.set(ix + ox, iy + 3, iz + 2, world::OBSIDIAN);
            }
            for oy in 0..=3 {
                self.world.set(ix - 1, iy + oy, iz + 2, world::OBSIDIAN);
                self.world.set(ix + 2, iy + oy, iz + 2, world::OBSIDIAN);
            }
            for ox in 0..=1 {
                for oy in 0..=2 {
                    self.world.set(ix + ox, iy + oy, iz + 2, world::NETHER_PORTAL);
                }
            }
        } else if target_dim == Dimension::End {
            for ox in -2..=2 {
                for oz in -2..=2 {
                    self.world.set(ix + ox, iy - 1, iz + oz, world::OBSIDIAN);
                    for oy in 0..=3 {
                        self.world.set(ix + ox, iy + oy, iz + oz, world::AIR);
                    }
                }
            }
        }

        self.player.x = tx;
        self.player.y = ty;
        self.player.z = tz;
        self.fall_y = ty;

        // Repopulate dimension-appropriate mobs
        self.mobs.clear();
        if target_dim == Dimension::End && !self.dragon_defeated {
            self.mobs.push(Mob::spawn(MobKind::EnderDragon, 0.0, 72.0, 0.0));
        }
        for i in 0..18 {
            let a = i as f32 * 2.39996;
            let d = 12.0 + (i % 7) as f32 * 3.0;
            let mx = tx + a.cos() * d;
            let mz = tz + a.sin() * d;
            let bm = self.world.biome(mx as i32, mz as i32);
            let k = entities::biome_mob(bm, target_dim, i * 31 + 7, 0.2);
            let my = if target_dim == Dimension::Nether { ty } else { self.world.highest_ground(mx as i32, mz as i32) as f32 };
            self.mobs.push(Mob::spawn(k, mx, my, mz));
        }
        self.message = format!("Entered {}", target_dim.name());
        self.msg_timer = 4.0;
        self.plat.play(Sfx::LevelUp);
        self.save_world();
    }

    fn save_path() -> std::path::PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("world_save.dat")))
            .unwrap_or_else(|| std::path::PathBuf::from("world_save.dat"))
    }

    fn save_world(&self) {
        let mut out = Vec::with_capacity(4096);
        out.extend_from_slice(b"MCS1");
        out.push(match self.mode {
            GameMode::Survival => 0,
            GameMode::Creative => 1,
            GameMode::Spectator => 2,
        });
        out.push(self.world.dim.idx() as u8);
        out.push(if self.dragon_defeated { 1 } else { 0 });
        out.push(self.player.selected as u8);
        for f in [
            self.player.x, self.player.y, self.player.z, self.player.yaw, self.player.pitch,
            self.player.hp, self.player.hunger, self.spawn.0, self.spawn.1, self.spawn.2, self.time,
        ] {
            out.extend_from_slice(&f.to_le_bytes());
        }
        for s in self.player.hot.iter().chain(self.player.store.iter()) {
            out.extend_from_slice(&s.id.to_le_bytes());
            out.extend_from_slice(&s.count.to_le_bytes());
        }
        let edits = self.world.all_edits();
        out.extend_from_slice(&(edits.len() as u32).to_le_bytes());
        for (d, x, y, z, id) in edits {
            out.push(d);
            out.extend_from_slice(&x.to_le_bytes());
            out.extend_from_slice(&y.to_le_bytes());
            out.extend_from_slice(&z.to_le_bytes());
            out.extend_from_slice(&id.to_le_bytes());
        }
        out.extend_from_slice(&(self.mobs.len() as u32).to_le_bytes());
        for m in &self.mobs {
            out.push(m.kind.to_u8());
            out.extend_from_slice(&m.x.to_le_bytes());
            out.extend_from_slice(&m.y.to_le_bytes());
            out.extend_from_slice(&m.z.to_le_bytes());
            out.extend_from_slice(&m.hp.to_le_bytes());
        }
        let _ = std::fs::write(Self::save_path(), out);
    }

    fn load_world(&mut self) {
        let Ok(data) = std::fs::read(Self::save_path()) else { return; };
        if data.len() < 8 + 44 + 36 * 6 + 4 || &data[0..4] != b"MCS1" { return; }
        self.loaded_save = true;
        let mut p = 4usize;
        let read_u8 = |b: &[u8], pos: &mut usize| -> u8 { let v = b[*pos]; *pos += 1; v };
        let read_u16 = |b: &[u8], pos: &mut usize| -> u16 {
            let v = u16::from_le_bytes([b[*pos], b[*pos + 1]]); *pos += 2; v
        };
        let read_u32 = |b: &[u8], pos: &mut usize| -> u32 {
            let v = u32::from_le_bytes([b[*pos], b[*pos + 1], b[*pos + 2], b[*pos + 3]]); *pos += 4; v
        };
        let read_i32 = |b: &[u8], pos: &mut usize| -> i32 {
            let v = i32::from_le_bytes([b[*pos], b[*pos + 1], b[*pos + 2], b[*pos + 3]]); *pos += 4; v
        };
        let read_f32 = |b: &[u8], pos: &mut usize| -> f32 {
            let v = f32::from_le_bytes([b[*pos], b[*pos + 1], b[*pos + 2], b[*pos + 3]]); *pos += 4; v
        };

        self.mode = match read_u8(&data, &mut p) {
            1 => GameMode::Creative,
            2 => GameMode::Spectator,
            _ => GameMode::Survival,
        };
        let dim = Dimension::from_idx(read_u8(&data, &mut p));
        self.world.set_dim(dim);
        self.dragon_defeated = read_u8(&data, &mut p) != 0;
        self.player.selected = (read_u8(&data, &mut p) as usize).min(HOT - 1);
        self.player.x = read_f32(&data, &mut p);
        self.player.y = read_f32(&data, &mut p);
        self.player.z = read_f32(&data, &mut p);
        self.player.yaw = read_f32(&data, &mut p);
        self.player.pitch = read_f32(&data, &mut p);
        self.player.hp = read_f32(&data, &mut p);
        self.player.hunger = read_f32(&data, &mut p);
        self.spawn.0 = read_f32(&data, &mut p);
        self.spawn.1 = read_f32(&data, &mut p);
        self.spawn.2 = read_f32(&data, &mut p);
        self.time = read_f32(&data, &mut p);
        self.fall_y = self.player.y;

        for s in self.player.hot.iter_mut().chain(self.player.store.iter_mut()) {
            let id = read_u16(&data, &mut p);
            let cnt = read_u32(&data, &mut p);
            *s = if id == 0 || cnt == 0 { Stack::EMPTY } else { Stack::new(id, cnt) };
        }
        let n_edits = read_u32(&data, &mut p) as usize;
        for _ in 0..n_edits {
            if p + 15 > data.len() { break; }
            let d = Dimension::from_idx(read_u8(&data, &mut p));
            let x = read_i32(&data, &mut p);
            let y = read_i32(&data, &mut p);
            let z = read_i32(&data, &mut p);
            let id = read_u16(&data, &mut p);
            self.world.insert_dim_edit(d, x, y, z, id);
        }
        if p + 4 <= data.len() {
            let n_mobs = read_u32(&data, &mut p) as usize;
            if n_mobs > 0 {
                self.mobs.clear();
                for _ in 0..n_mobs {
                    if p + 17 > data.len() { break; }
                    let kind = MobKind::from_u8(read_u8(&data, &mut p));
                    let mx = read_f32(&data, &mut p);
                    let my = read_f32(&data, &mut p);
                    let mz = read_f32(&data, &mut p);
                    let hp = read_f32(&data, &mut p);
                    let mut m = Mob::spawn(kind, mx, my, mz);
                    m.hp = hp;
                    self.mobs.push(m);
                }
            }
        }
    }
}

impl Clone for Furnace {
    fn clone(&self) -> Self {
        Furnace { input: self.input, fuel: self.fuel, output: self.output, burn: self.burn, burn_max: self.burn_max, cook: self.cook }
    }
}
