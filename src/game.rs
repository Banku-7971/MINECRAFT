//! The game itself: screens, world simulation, inventory, crafting, furnaces,
//! redstone, multiplayer and rendering. Talks to the OS only via `Platform`.
use std::collections::{HashMap, HashSet};

use crate::craft::{self, Stack, GRID};
use crate::entities::{self, Drop, Mob, MobKind};
use crate::items;
use crate::net::{Net, NetEvent};
use crate::platform::{self as plat, Event, Platform};
use crate::render::{self, Cam, Framebuffer, SpriteKind, Textures};
use crate::sound::{Audio, Sfx};
use crate::world::{self, Biome, World};

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
        let mut hot = [Stack::EMPTY; HOT];
        // A small starter kit so the first minute is fun, not empty.
        hot[0] = Stack::new(world::TORCH, 16);
        hot[1] = Stack::new(world::CRAFTING, 1);
        hot[2] = Stack::new(world::PLANKS, 12);
        hot[3] = Stack::new(items::WOOD_PICK, 1);
        hot[4] = Stack::new(items::BREAD, 4);
        hot[5] = Stack::new(world::FURNACE, 1);
        hot[6] = Stack::new(world::LAMP, 2);
        hot[7] = Stack::new(world::REDSTONE_TORCH, 2);
        hot[8] = Stack::new(world::REDSTONE_DUST, 8);
        let mut store = [Stack::EMPTY; STORE];
        store[0] = Stack::new(world::LEVER, 2);
        store[1] = Stack::new(world::COBBLE, 32);
        store[2] = Stack::new(items::COAL, 16);
        store[3] = Stack::new(world::IRON_ORE, 8);
        store[4] = Stack::new(world::LOG, 8);
        Player { x, y, z, vy: 0.0, yaw: 0.0, pitch: -0.08, hp: 20.0, hunger: 20.0, on_ground: false, selected: 0, hot, store }
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
pub enum Ui { None, Inventory, Furnace, Chat, Pause }

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
    mouse_right: bool,
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
fn block_drop(b: u16) -> Stack {
    use world::*;
    match b {
        GRASS | PODZOL | MYCELIUM | FARMLAND => Stack::new(DIRT, 1),
        STONE => Stack::new(COBBLE, 1),
        COAL_ORE => Stack::new(items::COAL, 1),
        IRON_ORE | GOLD_ORE | DIAMOND_ORE | EMERALD_ORE | REDSTONE_ORE => Stack::new(b, 1),
        CLAY => Stack::new(CLAY, 4),
        GRAVEL => Stack::new(items::FLINT, 1),
        LEAVES | SPRUCE_LEAVES | JUNGLE_LEAVES | BIRCH_LEAVES | DARK_LEAVES => Stack::new(b, 1),
        WHEAT => Stack::new(WHEAT, 2),
        TALL_GRASS => Stack::new(WHEAT, 1),
        ICE => Stack::EMPTY,
        _ => Stack::new(b, 1),
    }
}

fn requires_pickaxe(b: u16) -> u8 {
    use world::*;
    match b {
        STONE | COBBLE | STONE_BRICKS | BRICK | FURNACE | OBSIDIAN | IRON_BLOCK | GOLD_BLOCK | DIAMOND_BLOCK
        | EMERALD_BLOCK | COAL_ORE | QUARTZ | BLACKSTONE | SLAB | STAIRS | ANVIL | ENCHANTING | MOSSY => 1,
        IRON_ORE | LAMP => 2,
        GOLD_ORE | DIAMOND_ORE | REDSTONE_ORE | EMERALD_ORE => 3,
        _ => 0,
    }
}

impl<P: Platform> Game<P> {
    pub fn new(plat: P) -> Self {
        let mut world = World::new();
        let spawn_x = 0.5;
        let spawn_z = -239.5;
        let spawn_y = world.highest(spawn_x as i32, spawn_z as i32) as f32 + 0.2;
        let player = Player::new(spawn_x, spawn_y, spawn_z);
        let mut mobs = Vec::new();
        for i in 0..26 {
            let a = i as f32 * 2.39996;
            let d = 11.0 + (i % 9) as f32 * 2.6;
            let x = spawn_x + a.cos() * d;
            let z = spawn_z + a.sin() * d;
            let k = entities::random_kind(i * 17 + 3, 0.6);
            mobs.push(Mob::spawn(k, x, world.highest(x as i32, z as i32) as f32, z));
        }
        let splash = "Now with crafting, furnaces, redstone & LAN!";
        Game {
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
            mouse_right: false,
            chat_input: String::new(),
            chat_lines: Vec::new(),
            net_status: String::from("Host a game or join by IP"),
            net_addr: String::from("127.0.0.1"),
            menu_hover: 0,
            camera: 0,
            time: 0.30,
            tick: 0,
            message: String::from("Welcome! WASD to move, F5 camera, E inventory, T chat"),
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
        }
    }

    pub fn run(mut self) {
        self.plat.set_title("Minecraft");
        self.plat.set_sound(true);
        self.plat.set_music(true);
        loop {
            if self.plat.should_quit() || self.quit { break; }
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
                Event::FocusLost => { self.keys.clear(); self.mouse_left = false; self.mouse_right = false; }
                Event::MouseMove { dx, dy } => {
                    if self.screen == Screen::Playing && self.ui != Ui::Inventory && self.ui != Ui::Furnace && self.ui != Ui::Pause {
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
                                Ui::Inventory | Ui::Furnace => self.click_inventory(),
                                Ui::Pause => self.click_pause(),
                                Ui::Chat => {}
                                Ui::None => {
                                    self.plat.set_capture(true);
                                    if b == 0 { self.mouse_left = true; self.try_attack(); }
                                    else if b == 1 { self.mouse_right = true; self.try_use(); }
                                }
                            }
                        }
                    }
                }
                Event::MouseUp(b) => {
                    if b == 0 { self.mouse_left = false; self.mining = None; }
                    else if b == 1 { self.mouse_right = false; }
                }
                Event::Wheel(d) => {
                    if self.screen == Screen::Playing && self.ui == Ui::None {
                        let sel = (self.player.selected as i32 - d).rem_euclid(9) as usize;
                        self.player.selected = sel;
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
                            self.chat_lines.push((format!("<Player> {}", text), 0.0));
                            self.net.send(format!("CHAT|Player|{}", text));
                        }
                        self.chat_input.clear();
                        self.ui = Ui::None;
                        self.plat.set_capture(true);
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
                if self.ui == Ui::Inventory || self.ui == Ui::Furnace {
                    if code == VK_E || code == VK_ESCAPE { self.close_ui(); }
                    return;
                }
                match code {
                    VK_ESCAPE => self.open_pause(),
                    VK_E => { self.open_inventory(false); }
                    VK_T => {
                        self.ui = Ui::Chat;
                        self.chat_input.clear();
                        self.plat.set_capture(false);
                        self.plat.set_cursor(true);
                    }
                    VK_F5 => {
                        self.camera = (self.camera + 1) % 3;
                        self.message = format!("Camera: {}", ["First person", "Third person", "Distant"][self.camera as usize]);
                        self.msg_timer = 2.0;
                    }
                    VK_Q => {
                        if let Some((_, prog)) = self.mining {
                            let _ = prog;
                        }
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
    fn start_singleplayer(&mut self) {
        self.screen = Screen::Playing;
        self.ui = Ui::None;
        self.plat.set_capture(true);
        self.plat.set_cursor(false);
        self.plat.play(Sfx::LevelUp);
        self.message = String::from("Punch a tree, craft a pickaxe, dig down!");
        self.msg_timer = 6.0;
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
                (0..4).map(|i| (bx, 132 + i as i32 * 28, 200, 22)).collect()
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
                        1 => { self.screen = Screen::Multiplayer; }
                        2 => { self.screen = Screen::Options; }
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
                        2 => { self.pixel_scale = if self.pixel_scale >= 4 { 1 } else { self.pixel_scale + 1 }; }
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
        let bx = (render::W as i32 - 200) / 2;
        for i in 0..5 {
            let (x, y, w, h) = (bx, 76 + i as i32 * 26, 200, 20);
            if mx >= x && mx < x + w && my >= y && my < y + h {
                match i {
                    0 => self.resume(),
                    1 => { self.sound_on = !self.sound_on; self.audio.sound_on = self.sound_on; self.plat.set_sound(self.sound_on); }
                    2 => { self.music_on = !self.music_on; self.audio.music_on = self.music_on; self.plat.set_music(self.music_on); }
                    3 => { self.return_to_pause = true; self.screen = Screen::Options; }
                    _ => { self.net.close(); self.screen = Screen::Title; self.ui = Ui::None; self.plat.set_cursor(true); self.plat.set_capture(false); }
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

    fn click_inventory(&mut self) {
        let (mx, my) = (self.mouse_x, self.mouse_y);
        let right = self.mouse_right;
        self.mouse_right = false;
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

    fn eye(&self) -> (f32, f32, f32) { (self.player.x, self.player.y + 1.62, self.player.z) }

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
            let dmg = match self.player.held().id {
                items::IRON_SWORD | items::DIAMOND_SWORD => 6.0,
                items::WOOD_SWORD => 3.5,
                _ => 2.0,
            };
            self.mobs[i].hp -= dmg;
            self.mobs[i].hurt = 0.4;
            self.plat.play(Sfx::Hurt);
            if self.mobs[i].hp <= 0.0 {
                let m = self.mobs.remove(i);
                let id = m.kind.drop();
                self.drops.push(Drop::new(m.x, m.y + 0.4, m.z, id));
                self.plat.play(Sfx::LevelUp);
            }
            return;
        }
        self.mine_step(1.0);
    }

    fn mine_step(&mut self, speed_mul: f32) {
        let (dx, dy, dz) = self.look_dir();
        let (ex, ey, ez) = self.eye();
        let hit = world::raycast(&mut self.world, ex, ey, ez, dx, dy, dz, REACH, false);
        let Some(h) = hit else { self.mining = None; return; };
        let pos = (h.x, h.y, h.z);
        let tier = self.player.held().id;
        let need = requires_pickaxe(h.block);
        if need > 0 && items::pickaxe_tier(tier) < need {
            self.message = String::from("You need a better pickaxe for that");
            self.msg_timer = 1.5;
            self.mining = None;
            return;
        }
        let mut prog = match self.mining { Some((p, v)) if p == pos => v, _ => 0.0 };
        let hardness = if need > 0 { 0.9 } else { 0.45 };
        let tool_bonus = 1.0 + items::pickaxe_tier(tier) as f32 * 0.8;
        prog += speed_mul * tool_bonus / hardness * 0.35;
        if prog >= 1.0 {
            self.mining = None;
            self.break_block(h.x, h.y, h.z);
        } else {
            self.mining = Some((pos, prog));
        }
    }

    fn break_block(&mut self, x: i32, y: i32, z: i32) {
        let b = self.world.get(x, y, z);
        if b == world::AIR || b == world::BEDROCK { return; }
        self.world.set(x, y, z, world::AIR);
        self.net.send(format!("BLOCK|{}|{}|{}|0", x, y, z));
        self.plat.play(Sfx::Break);
        let drop = block_drop(b);
        if !drop.is_empty() {
            let left = self.player.add(drop.id, drop.count);
            if left > 0 { self.drops.push(Drop::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5, drop.id)); }
        }
        // block above falls if it was supported
        let above = self.world.get(x, y + 1, z);
        if matches!(above, world::TORCH | world::REDSTONE_TORCH | world::FLOWER_RED | world::FLOWER_YELLOW | world::TALL_GRASS) {
            self.world.set(x, y + 1, z, world::AIR);
        }
        self.furnaces.remove(&(x, y, z));
        self.levers.remove(&(x, y, z));
        self.update_redstone();
    }

    fn try_use(&mut self) {
        if self.place_cd > 0.0 { return; }
        self.place_cd = 0.2;
        let (dx, dy, dz) = self.look_dir();
        let (ex, ey, ez) = self.eye();
        let hit = world::raycast(&mut self.world, ex, ey, ez, dx, dy, dz, REACH, false);
        let Some(h) = hit else { return; };
        let target = (h.x, h.y, h.z);

        // use the block itself first (crafting table, furnace, lever)
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
                self.plat.play(Sfx::Click);
                self.update_redstone();
                return;
            }
            world::REDSTONE_TORCH => { self.plat.play(Sfx::Click); return; }
            _ => {}
        }

        let held = self.player.held();
        if held.is_empty() { return; }
        // eat food
        if items::is_food(held.id) > 0.0 {
            if self.player.hunger < 20.0 {
                self.player.hunger = (self.player.hunger + items::is_food(held.id) * 2.0).min(20.0);
                self.player.hp = (self.player.hp + 2.0).min(20.0);
                let s = &mut self.player.hot[self.player.selected];
                s.count -= 1;
                if s.count == 0 { *s = Stack::EMPTY; }
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
            // don't place inside the player
            let (fx, fy, fz) = (px as f32 + 0.5, py as f32, pz as f32 + 0.5);
            if (fx - self.player.x).abs() < 0.9 && (fz - self.player.z).abs() < 0.9
                && fy + 1.0 > self.player.y && fy < self.player.y + 1.8 {
                return;
            }
            self.world.set(px, py, pz, held.id);
            self.net.send(format!("BLOCK|{}|{}|{}|{}", px, py, pz, held.id));
            let s = &mut self.player.hot[self.player.selected];
            s.count -= 1;
            if s.count == 0 { *s = Stack::EMPTY; }
            self.plat.play(Sfx::Place);
            if held.id == world::FURNACE { self.furnaces.insert((px, py, pz), Furnace::new()); }
            self.update_redstone();
        }
    }

    // --------------------------------------------------------------- redstone
    fn update_redstone(&mut self) {
        self.power.clear();
        self.lamps.clear();
        let mut queue: Vec<((i32, i32, i32), u8)> = Vec::new();
        // gather sources in the area the player can see
        let (px, py, pz) = (self.player.x as i32, self.player.y as i32, self.player.z as i32);
        for x in px - 40..=px + 40 {
            for y in (py - 24).max(0)..=(py + 24).min(world::WORLD_H as i32 - 1) {
                for z in pz - 40..=pz + 40 {
                    let b = self.world.get(x, y, z);
                    match b {
                        world::REDSTONE_TORCH => queue.push(((x, y, z), 15)),
                        world::LEVER => {
                            if self.levers.contains(&(x, y, z)) { queue.push(((x, y, z), 15)); }
                        }
                        world::PRESSURE_PLATE => {
                            let on = (self.player.x - x as f32).abs() < 1.0
                                && (self.player.z - z as f32).abs() < 1.0
                                && (self.player.y - y as f32).abs() < 1.3;
                            if on { self.plates.insert((x, y, z)); queue.push(((x, y, z), 15)); }
                            else { self.plates.remove(&(x, y, z)); }
                        }
                        _ => {}
                    }
                }
            }
        }
        // BFS through dust
        let mut qi = 0;
        while qi < queue.len() {
            let ((x, y, z), p) = queue[qi];
            qi += 1;
            if self.power.get(&(x, y, z)).map_or(false, |v| *v >= p) { continue; }
            self.power.insert((x, y, z), p);
            // lamp next to a powered thing lights up
            for (dx, dy, dz) in [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)] {
                let n = (x + dx, y + dy, z + dz);
                let nb = self.world.get(n.0, n.1, n.2);
                if nb == world::LAMP { self.lamps.insert(n); }
                if nb == world::REDSTONE_DUST && p > 1 {
                    let np = p - 1;
                    if self.power.get(&n).map_or(true, |v| *v < np) { queue.push((n, np)); }
                }
            }
        }
        self.world.lit = self.lamps.clone();
    }

    // --------------------------------------------------------------- simulation
    fn update_playing(&mut self, dt: f32) {
        let dt = dt.min(0.05);
        self.attack_cd = (self.attack_cd - dt).max(0.0);
        self.place_cd = (self.place_cd - dt).max(0.0);
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

        let ui_open = self.ui == Ui::Inventory || self.ui == Ui::Furnace;
        if !ui_open {
            let mut mx: f32 = 0.0;
            let mut mz: f32 = 0.0;
            if self.keys.contains(&plat::VK_W) { mz -= 1.0; }
            if self.keys.contains(&plat::VK_S) { mz += 1.0; }
            if self.keys.contains(&plat::VK_A) { mx -= 1.0; }
            if self.keys.contains(&plat::VK_D) { mx += 1.0; }
            let len = (mx * mx + mz * mz).sqrt().max(1.0);
            mx /= len;
            mz /= len;
            let sprint = self.keys.contains(&plat::VK_SHIFT) && self.player.hunger > 3.0;
            let speed = if sprint { 6.4 } else { 4.3 };
            let (sy, cy) = (self.player.yaw.sin(), self.player.yaw.cos());
            let vx = (mx * cy + mz * sy) * speed;
            let vz = (mx * sy - mz * cy) * speed;
            let nx = self.player.x + vx * dt;
            if !collides(&mut self.world, nx, self.player.y, self.player.z) { self.player.x = nx; }
            let nz = self.player.z + vz * dt;
            if !collides(&mut self.world, self.player.x, self.player.y, nz) { self.player.z = nz; }
            if self.keys.contains(&plat::VK_SPACE) && self.player.on_ground {
                self.player.vy = 8.4;
                self.player.on_ground = false;
                self.plat.play(Sfx::Step);
            }
            self.player.vy -= 22.0 * dt;
            let ny = self.player.y + self.player.vy * dt;
            if collides(&mut self.world, self.player.x, ny, self.player.z) {
                if self.player.vy < 0.0 { self.player.on_ground = true; }
                self.player.vy = 0.0;
            } else {
                self.player.y = ny;
                self.player.on_ground = false;
            }
            // walk step sounds
            if (vx.abs() + vz.abs()) > 0.1 && self.player.on_ground {
                self.audio_timer += dt as f64;
                if self.audio_timer > 0.42 {
                    self.audio_timer = 0.0;
                    self.plat.play(Sfx::Step);
                }
            }
            if self.mouse_left { self.mine_step(dt); }
        }

        // mobs
        for m in self.mobs.iter_mut() {
            m.update(&mut self.world, self.player.x, self.player.y, self.player.z, dt, self.tick, day);
        }
        self.mobs.retain(|m| m.hp > 0.0 && (m.x - self.player.x).abs() < 90.0 && (m.z - self.player.z).abs() < 90.0);
        for m in self.mobs.iter_mut() {
            if m.kind.hostile() {
                let d = ((m.x - self.player.x).powi(2) + (m.z - self.player.z).powi(2)).sqrt();
                if d < 1.7 && m.attack_cd <= 0.0 && self.hurt_cd <= 0.0 && self.grace <= 0.0 {
                    self.player.hp -= 2.0;
                    self.hurt_flash = 0.4;
                    self.hurt_cd = 0.7;
                    m.attack_cd = 1.2;
                    self.plat.play(Sfx::Hurt);
                }
            }
        }
        // spawn mobs
        self.regen += dt;
        if self.regen > 1.6 {
            self.regen = 0.0;
            if self.mobs.len() < 30 {
                let a = (self.tick as f32 * 0.618).sin() * 6.283;
                let d = 24.0 + (self.tick % 17) as f32;
                let x = self.player.x + a.cos() * d;
                let z = self.player.z + a.sin() * d;
                let k = entities::random_kind(self.tick.wrapping_mul(7919), day);
                self.mobs.push(Mob::spawn(k, x, self.world.highest(x as i32, z as i32) as f32, z));
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

        // plates keep lamps updated as the player walks
        if self.tick % 6 == 0 { self.update_redstone(); }

        // hunger + regen
        self.player.hunger = (self.player.hunger - dt * 0.045).max(0.0);
        if self.player.hunger < 1.0 && self.hurt_cd <= 0.0 {
            self.player.hp -= 0.5;
            self.hurt_cd = 2.0;
        }
        if self.player.hp < 20.0 && self.player.hunger > 8.0 {
            self.regen = self.regen; // regen timer shared with spawn pacing
        }
        self.audio_timer = self.audio_timer;

        // death
        if self.player.hp <= 0.0 {
            self.player.hp = 20.0;
            self.player.hunger = 20.0;
            self.player.x = self.spawn.0;
            self.player.y = self.spawn.1;
            self.player.z = self.spawn.2;
            self.message = String::from("You died! Respawning...");
            self.msg_timer = 4.0;
            self.grace = 6.0;
            self.plat.play(Sfx::Hurt);
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
        render::draw_title(&mut self.fb, &["Singleplayer", "Multiplayer", "Options", "Quit"], hover, &self.splash, self.title_t);
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
        let eye_y = self.player.y + 1.62;
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
        self.scene.cursor = (self.mouse_x, self.mouse_y);
        render::render_world(&mut self.scene, &self.tex, &mut self.world, &cam, self.time, self.tick);
        if self.camera != 0 {
            render::draw_sprite(&mut self.scene, &cam, self.player.x, self.player.y, self.player.z, 1.8, self.tick, day, SpriteKind::Steve, true);
        }
        for (id, (x, y, z, yaw, _)) in self.remote.iter() {
            let _ = id;
            render::draw_sprite(&mut self.scene, &cam, *x, *y, *z, 1.8, self.tick, day, SpriteKind::Steve, false);
            let _ = yaw;
        }
        for m in self.mobs.iter() {
            render::draw_sprite(&mut self.scene, &cam, m.x, m.y, m.z, m.kind.height(), self.tick, day, SpriteKind::Mob(m.kind), false);
        }
        for d in self.drops.iter() {
            render::draw_sprite(&mut self.scene, &cam, d.x, d.y, d.z, 0.35, self.tick, day, SpriteKind::Drop(d.item), false);
        }
        render::upscale(&self.scene, &mut self.fb);

        if self.ui == Ui::None || self.ui == Ui::Chat {
            if self.camera == 0 {
                render::draw_held_item(&mut self.fb, self.player.hot[self.player.selected].id, self.tick);
            }
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
                world::biome_at(self.player.x.floor() as i32, self.player.z.floor() as i32),
                (self.player.x, self.player.y, self.player.z),
                self.fps,
                self.camera,
                self.time,
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
            let bx = (render::W as i32 - 200) / 2;
            let mut hover = usize::MAX;
            for i in 0..5 {
                let (x, y, w, h) = (bx, 76 + i as i32 * 26, 200, 20);
                if mx >= x && mx < x + w && my >= y && my < y + h { hover = i; }
            }
            render::draw_pause(&mut self.fb, hover, self.title_t);
            let labels = [
                "BACK TO GAME".to_string(),
                format!("SOUND: {}", if self.sound_on { "ON" } else { "OFF" }),
                format!("MUSIC: {}", if self.music_on { "ON" } else { "OFF" }),
                "OPTIONS".to_string(),
                "SAVE AND QUIT".to_string(),
            ];
            for (i, l) in labels.iter().enumerate() {
                render::button(&mut self.fb, bx, 76 + i as i32 * 26, 200, 20, l, hover == i);
            }
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
}

impl Clone for Furnace {
    fn clone(&self) -> Self {
        Furnace { input: self.input, fuel: self.fuel, output: self.output, burn: self.burn, burn_max: self.burn_max, cook: self.cook }
    }
}
