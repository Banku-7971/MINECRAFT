//! The game itself: screens, world simulation, inventory, crafting, furnaces,
//! redstone, multiplayer and rendering. Talks to the OS only via `Platform`.
use std::collections::{HashMap, HashSet, VecDeque};

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
    pub air: f32,
    pub xp: u32,
    pub level: u32,
    pub burn: f32,
    pub on_ground: bool,
    pub selected: usize,
    pub hot: [Stack; HOT],
    pub store: [Stack; STORE],
    pub armor: [Stack; 4],
    pub offhand: Stack,
    pub enchants: HashMap<u16, u8>,
}

impl Player {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Player {
            x, y, z, vy: 0.0, yaw: 0.0, pitch: -0.08,
            hp: 20.0, hunger: 20.0, air: 10.0, xp: 0, level: 0, burn: 0.0, on_ground: false,
            selected: 0,
            hot: [Stack::EMPTY; HOT],
            store: [Stack::EMPTY; STORE],
            armor: [Stack::EMPTY; 4],
            offhand: Stack::EMPTY,
            enchants: HashMap::new(),
        }
    }

    fn armor_points(&self) -> u32 {
        self.armor.iter().map(|s| items::armor_points(s.id)).sum::<u32>().min(20)
    }

    fn has_totem(&self) -> bool {
        self.held().id == items::TOTEM || self.offhand.id == items::TOTEM
    }

    fn add_xp(&mut self, amt: u32) {
        self.xp += amt;
        loop {
            let need = 7 + self.level * 2;
            if self.xp >= need {
                self.xp -= need;
                self.level += 1;
            } else {
                break;
            }
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

#[derive(Clone)]
pub struct BrewingStand {
    pub ing: Stack,
    pub bottle: Stack,
    pub output: Stack,
    pub prog: f32,
}

impl BrewingStand {
    fn new() -> Self {
        Self { ing: Stack::EMPTY, bottle: Stack::EMPTY, output: Stack::EMPTY, prog: 0.0 }
    }
}

#[derive(Clone)]
pub struct Vehicle {
    pub is_boat: bool,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Screen { Title, Multiplayer, Options, Playing }

#[derive(PartialEq, Clone, Copy)]
pub enum Ui { None, Inventory, Creative, Furnace, Chest, Enchant, Anvil, Brewing, Trade, Death, Chat, Pause }

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
    fluid_levels: HashMap<(u8, i32, i32, i32), u8>,
    fluid_queue: VecDeque<(u8, i32, i32, i32)>,
    fluid_timer: f32,
    chests: HashMap<(u8, i32, i32, i32), [Stack; STORE]>,
    arrows: Vec<(f32, f32, f32, f32, f32, f32)>,
    mob_projectiles: Vec<(f32, f32, f32, f32, f32, f32, bool)>,
    burn_cd: f32,
    drown_cd: f32,
    mouse_right: bool,
    potion_fire: f32,
    potion_speed: f32,
    elytra_gliding: bool,
    enchant_slots: [Stack; 2],
    anvil_slots: [Stack; 2],
    brewing_stands: HashMap<(i32, i32, i32), BrewingStand>,
    buttons: HashMap<(i32, i32, i32), f32>,
    piston_ext: HashSet<(i32, i32, i32)>,
    vehicles: Vec<Vehicle>,
    riding: Option<usize>,
    crop_timer: f32,
    spawner_timer: f32,
    tool_dur: [u16; HOT],
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
        LAPIS_ORE => Stack::new(items::LAPIS_LAZULI, 5),
        REDSTONE_ORE => Stack::new(REDSTONE_DUST, 4),
        NETHER_QUARTZ_ORE => Stack::new(items::NETHER_QUARTZ, 1),
        CRIMSON_NYLIUM | WARPED_NYLIUM => Stack::new(NETHERRACK, 1),
        END_PORTAL_FRAME | END_PORTAL_FRAME_EYE | NETHER_PORTAL | END_PORTAL => Stack::EMPTY,
        IRON_ORE | GOLD_ORE => Stack::new(b, 1),
        CLAY => Stack::new(CLAY, 4),
        GRAVEL => if tick % 10 == 0 { Stack::new(items::FLINT, 1) } else { Stack::new(GRAVEL, 1) },
        LEAVES | SPRUCE_LEAVES | JUNGLE_LEAVES | BIRCH_LEAVES | DARK_LEAVES => {
            if tick % 20 == 0 { Stack::new(items::APPLE, 1) }
            else if tick % 15 == 0 { Stack::new(SAPLING, 1) }
            else if tick % 12 == 0 { Stack::new(items::STICK, 1) }
            else { Stack::EMPTY }
        }
        WHEAT => Stack::new(WHEAT, 2),
        NETHER_WART => Stack::new(NETHER_WART, 2),
        TALL_GRASS => if tick % 8 == 0 { Stack::new(WHEAT, 1) } else { Stack::EMPTY },
        ICE | GLASS => Stack::EMPTY,
        DOOR_OPEN => Stack::new(DOOR, 1),
        IRON_DOOR_OPEN => Stack::new(IRON_DOOR, 1),
        _ => Stack::new(b, 1),
    }
}

fn requires_pickaxe(b: u16) -> u8 {
    use world::*;
    match b {
        STONE | COBBLE | STONE_BRICKS | BRICK | FURNACE | SANDSTONE | COAL_ORE | QUARTZ | BLACKSTONE
        | SLAB | STAIRS | ANVIL | ENCHANTING | MOSSY | IRON_BLOCK | NETHERRACK | NETHER_BRICKS
        | CRIMSON_NYLIUM | WARPED_NYLIUM | BASALT | END_STONE | PURPUR_BLOCK | NETHER_QUARTZ_ORE
        | HOPPER | PISTON | STICKY_PISTON | IRON_DOOR | IRON_DOOR_OPEN | SPAWNER => 1,
        IRON_ORE | LAPIS_ORE | LAPIS_BLOCK | LAMP => 2,
        GOLD_ORE | DIAMOND_ORE | REDSTONE_ORE | EMERALD_ORE | GOLD_BLOCK | DIAMOND_BLOCK | EMERALD_BLOCK => 3,
        OBSIDIAN | CRYING_OBSIDIAN | ANCIENT_DEBRIS | BEACON => 4,
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

// The browser is a little slower than the native build, so start at a
// quarter-resolution scene (240x135) and let players change it in Options.
#[cfg(target_arch = "wasm32")]
const DEFAULT_PIXEL_SCALE: i32 = 4;
#[cfg(not(target_arch = "wasm32"))]
const DEFAULT_PIXEL_SCALE: i32 = 2;

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
        #[cfg(target_arch = "wasm32")]
        let splash = "Now running in your browser!";
        #[cfg(not(target_arch = "wasm32"))]
        let splash = "Now with Nether, End, biomes, crafting & LAN!";
        let (scene_w, scene_h) = scene_dims(DEFAULT_PIXEL_SCALE);
        let mut game = Game {
            plat,
            audio: Audio::new(),
            net: Net::offline(),
            world,
            tex: Textures::new(),
            scene: Framebuffer::new(scene_w, scene_h),
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
            pixel_scale: DEFAULT_PIXEL_SCALE,
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
            fluid_levels: HashMap::new(),
            fluid_queue: VecDeque::new(),
            fluid_timer: 0.0,
            chests: HashMap::new(),
            arrows: Vec::new(),
            mob_projectiles: Vec::new(),
            burn_cd: 0.0,
            drown_cd: 0.0,
            mouse_right: false,
            potion_fire: 0.0,
            potion_speed: 0.0,
            elytra_gliding: false,
            enchant_slots: [Stack::EMPTY; 2],
            anvil_slots: [Stack::EMPTY; 2],
            brewing_stands: HashMap::new(),
            buttons: HashMap::new(),
            piston_ext: HashSet::new(),
            vehicles: Vec::new(),
            riding: None,
            crop_timer: 0.0,
            spawner_timer: 0.0,
            tool_dur: [0; HOT],
        };
        game.load_world();
        game
    }

    pub fn run(mut self) {
        self.plat.set_title("Minecraft");
        self.plat.set_sound(true);
        self.plat.set_music(true);
        while !self.plat.should_quit() && !self.quit {
            let start = self.plat.now();
            self.frame();

            // ~60 fps target on native platforms. The web driver uses
            // requestAnimationFrame instead, so `frame` itself never sleeps.
            let spent = self.plat.now() - start;
            if spent < 0.0166 {
                self.plat.sleep_ms(((0.0166 - spent) * 1000.0) as u64);
            }
        }
        self.save_world();
        self.net.close();
    }

    /// Advance the simulation and render exactly one frame.
    ///
    /// Native builds pace this with `Platform::sleep_ms`; the WebAssembly
    /// build calls it from JavaScript once per animation frame.
    pub fn frame(&mut self) {
        if self.plat.should_quit() || self.quit { return; }
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
                                Ui::Chest => self.click_chest(b == 1),
                                Ui::Creative => self.click_creative(b == 1),
                                Ui::Enchant => self.click_enchant(b == 1),
                                Ui::Anvil => self.click_anvil(b == 1),
                                Ui::Brewing => self.click_brewing(b == 1),
                                Ui::Trade => self.click_trade(),
                                Ui::Death => self.click_death(),
                                Ui::Pause => self.click_pause(),
                                Ui::Chat => {}
                                Ui::None => {
                                    self.plat.set_capture(true);
                                    if self.mode != GameMode::Spectator {
                                        if b == 0 { self.mouse_left = true; self.try_attack(); }
                                        else if b == 1 { self.mouse_right = true; self.try_use(); }
                                    }
                                }
                            }
                        }
                    }
                }
                Event::MouseUp(b) => {
                    if b == 0 { self.mouse_left = false; self.mining = None; }
                    if b == 1 { self.mouse_right = false; }
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
                if matches!(self.ui, Ui::Inventory | Ui::Furnace | Ui::Chest | Ui::Enchant | Ui::Anvil | Ui::Brewing | Ui::Trade) {
                    if code == VK_E || code == VK_ESCAPE { self.close_ui(); }
                    return;
                }
                if self.ui == Ui::Death {
                    if code == VK_RETURN { self.respawn_player(); }
                    return;
                }
                match code {
                    VK_ESCAPE => self.open_pause(),
                    VK_E => {
                        if self.mode == GameMode::Creative { self.open_creative(); }
                        else if self.mode == GameMode::Survival { self.open_inventory(false); }
                    }
                    VK_Q => {
                        if !self.player.hot[self.player.selected].is_empty() {
                            let all = self.keys.contains(&VK_CONTROL) || self.keys.contains(&VK_SHIFT);
                            let (dx, dy, dz) = self.look_dir();
                            let st = self.player.hot[self.player.selected];
                            let drop_n = if all { st.count } else { 1 };
                            for _ in 0..drop_n.min(16) {
                                let d = Drop::new(self.player.x + dx * 2.2, self.player.y + 1.2 + dy * 0.4, self.player.z + dz * 2.2, st.id);
                                self.drops.push(d);
                            }
                            let s = &mut self.player.hot[self.player.selected];
                            s.count -= drop_n;
                            if s.count == 0 { *s = Stack::EMPTY; }
                            self.plat.play(Sfx::Click);
                        }
                    }
                    VK_F => {
                        let tmp = self.player.hot[self.player.selected];
                        self.player.hot[self.player.selected] = self.player.offhand;
                        self.player.offhand = tmp;
                        self.plat.play(Sfx::Click);
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
                        if self.riding.is_some() {
                            self.riding = None;
                            self.player.vy = 4.5;
                            self.message = String::from("Dismounted");
                            self.msg_timer = 1.2;
                        } else if self.mode == GameMode::Creative {
                            let now = self.plat.now();
                            if now - self.space_tap_t < 0.30 {
                                self.flying = !self.flying;
                                self.player.vy = 0.0;
                                self.space_tap_t = -10.0;
                            } else {
                                self.space_tap_t = now;
                            }
                        } else if !self.player.on_ground && self.player.armor[1].id == items::ELYTRA {
                            self.elytra_gliding = !self.elytra_gliding;
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
        // return crafting grid / enchant / anvil contents
        for i in 0..GRID {
            let st = self.craft_grid[i];
            if !st.is_empty() {
                let left = self.player.add(st.id, st.count);
                if left > 0 { self.drops.push(Drop::new(self.player.x, self.player.y + 1.0, self.player.z, st.id)); }
                self.craft_grid[i] = Stack::EMPTY;
            }
        }
        for s in self.enchant_slots.iter_mut().chain(self.anvil_slots.iter_mut()) {
            if !s.is_empty() {
                let left = self.player.add(s.id, s.count);
                if left > 0 { self.drops.push(Drop::new(self.player.x, self.player.y + 1.0, self.player.z, s.id)); }
                *s = Stack::EMPTY;
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
            4 => if idx < 4 { &mut self.player.armor[idx] } else { &mut self.player.offhand },
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

    fn click_chest(&mut self, right: bool) {
        let Some((kind, idx)) = self.chest_hover() else { return; };
        if kind == 0 {
            let key = (self.world.dim.idx() as u8, self.ui_block.0, self.ui_block.1, self.ui_block.2);
            let cur = self.cursor;
            let st = self.chests.get(&key).map(|c| c[idx]).unwrap_or(Stack::EMPTY);
            if right {
                if cur.is_empty() && !st.is_empty() {
                    let half = (st.count + 1) / 2;
                    self.cursor = Stack::new(st.id, half);
                    let arr = self.chests.get_mut(&key).unwrap();
                    arr[idx].count -= half;
                    if arr[idx].count == 0 { arr[idx] = Stack::EMPTY; }
                } else if !cur.is_empty() && (st.is_empty() || (st.id == cur.id && st.count < 64)) {
                    let arr = self.chests.get_mut(&key).unwrap();
                    if arr[idx].is_empty() { arr[idx] = Stack::new(cur.id, 1); } else { arr[idx].count += 1; }
                    self.cursor.count -= 1;
                    if self.cursor.count == 0 { self.cursor = Stack::EMPTY; }
                }
            } else if cur.is_empty() && !st.is_empty() {
                self.cursor = st;
                self.chests.get_mut(&key).unwrap()[idx] = Stack::EMPTY;
            } else if !cur.is_empty() && st.is_empty() {
                self.chests.get_mut(&key).unwrap()[idx] = cur;
                self.cursor = Stack::EMPTY;
            } else if !cur.is_empty() && st.id == cur.id {
                let arr = self.chests.get_mut(&key).unwrap();
                let put = (64 - arr[idx].count).min(cur.count);
                arr[idx].count += put;
                self.cursor.count -= put;
                if self.cursor.count == 0 { self.cursor = Stack::EMPTY; }
            } else if !cur.is_empty() {
                self.cursor = st;
                self.chests.get_mut(&key).unwrap()[idx] = cur;
            }
            self.plat.play(Sfx::Click);
        } else {
            self.slot_click(right, kind, idx);
        }
    }

    fn chest_hover(&self) -> Option<(usize, usize)> {
        let (mx, my) = (self.mouse_x, self.mouse_y);
        for i in 0..STORE {
            let (x, y, w, h) = render::chest_slot_rect(i);
            if mx >= x && mx < x + w && my >= y && my < y + h { return Some((0, i)); }
        }
        for i in 0..STORE {
            let (x, y, w, h) = render::chest_inv_slot_rect(i);
            if mx >= x && mx < x + w && my >= y && my < y + h { return Some((1, i)); }
        }
        for i in 0..HOT {
            let (x, y, w, h) = render::hot_slot_rect(i);
            if mx >= x && mx < x + w && my >= y && my < y + h { return Some((2, i)); }
        }
        None
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
        // armor & offhand
        for i in 0..4 {
            let (x, y, w, h) = render::armor_slot_rect(i);
            if mx >= x && mx < x + w && my >= y && my < y + h {
                if self.cursor.is_empty() || items::armor_slot(self.cursor.id) == Some(i) {
                    self.slot_click(right, 4, i);
                }
                return;
            }
        }
        let (ox, oy, ow, oh) = render::offhand_slot_rect();
        if mx >= ox && mx < ox + ow && my >= oy && my < oy + oh {
            self.slot_click(right, 4, 4);
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

    fn swap_slot_with_cursor(slot: &mut Stack, cursor: &mut Stack, right: bool) {
        if right {
            if cursor.is_empty() && !slot.is_empty() {
                let half = (slot.count + 1) / 2;
                *cursor = Stack::new(slot.id, half);
                slot.count -= half;
                if slot.count == 0 { *slot = Stack::EMPTY; }
            } else if !cursor.is_empty() && (slot.is_empty() || (slot.id == cursor.id && slot.count < 64)) {
                if slot.is_empty() { *slot = Stack::new(cursor.id, 1); } else { slot.count += 1; }
                cursor.count -= 1;
                if cursor.count == 0 { *cursor = Stack::EMPTY; }
            }
        } else if cursor.is_empty() {
            if !slot.is_empty() { *cursor = *slot; *slot = Stack::EMPTY; }
        } else if slot.is_empty() {
            *slot = *cursor; *cursor = Stack::EMPTY;
        } else if slot.id == cursor.id {
            let put = (64 - slot.count).min(cursor.count);
            slot.count += put; cursor.count -= put;
            if cursor.count == 0 { *cursor = Stack::EMPTY; }
        } else {
            std::mem::swap(slot, cursor);
        }
    }

    fn click_enchant(&mut self, right: bool) {
        let (mx, my) = (self.mouse_x, self.mouse_y);
        if mx >= 94 && mx < 94 + render::SLOT_PX && my >= 48 && my < 48 + render::SLOT_PX {
            Self::swap_slot_with_cursor(&mut self.enchant_slots[0], &mut self.cursor, right);
            self.plat.play(Sfx::Click);
            return;
        }
        if mx >= 126 && mx < 126 + render::SLOT_PX && my >= 48 && my < 48 + render::SLOT_PX {
            if self.cursor.is_empty() || self.cursor.id == items::LAPIS_LAZULI {
                Self::swap_slot_with_cursor(&mut self.enchant_slots[1], &mut self.cursor, right);
                self.plat.play(Sfx::Click);
            }
            return;
        }
        let reqs = [(1u32, 1u8), (3u32, 2u8), (5u32, 3u8)];
        for (i, &(req, tier)) in reqs.iter().enumerate() {
            let (bx, by, bw, bh) = (170, 36 + i as i32 * 26, 224, 22);
            if mx >= bx && mx < bx + bw && my >= by && my < by + bh {
                if !self.enchant_slots[0].is_empty() && !self.enchant_slots[1].is_empty() && self.player.level >= req {
                    let id = self.enchant_slots[0].id;
                    self.player.level -= req;
                    self.enchant_slots[1].count -= 1;
                    if self.enchant_slots[1].count == 0 { self.enchant_slots[1] = Stack::EMPTY; }
                    let cur = self.player.enchants.get(&id).copied().unwrap_or(0);
                    self.player.enchants.insert(id, cur.max(tier));
                    self.message = format!("Enchanted {} (Tier {})!", items::name(id), tier);
                    self.msg_timer = 3.0;
                    self.plat.play(Sfx::LevelUp);
                }
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
    }

    fn anvil_output(&self) -> Stack {
        let l = self.anvil_slots[0];
        let r = self.anvil_slots[1];
        if l.is_empty() || r.is_empty() { return Stack::EMPTY; }
        if l.id == r.id { return Stack::new(l.id, 1); }
        if r.id == items::ENCHANTED_BOOK || r.id == items::DIAMOND || r.id == items::NETHERITE_INGOT {
            if r.id == items::NETHERITE_INGOT && l.id == items::DIAMOND_SWORD { return Stack::new(items::DIAMOND_SWORD, 1); }
            if r.id == items::NETHERITE_INGOT && l.id == items::DIAMOND_HELMET { return Stack::new(items::NETHERITE_HELMET, 1); }
            if r.id == items::NETHERITE_INGOT && l.id == items::DIAMOND_CHESTPLATE { return Stack::new(items::NETHERITE_CHESTPLATE, 1); }
            if r.id == items::NETHERITE_INGOT && l.id == items::DIAMOND_LEGGINGS { return Stack::new(items::NETHERITE_LEGGINGS, 1); }
            if r.id == items::NETHERITE_INGOT && l.id == items::DIAMOND_BOOTS { return Stack::new(items::NETHERITE_BOOTS, 1); }
            return Stack::new(l.id, 1);
        }
        Stack::EMPTY
    }

    fn click_anvil(&mut self, right: bool) {
        let (mx, my) = (self.mouse_x, self.mouse_y);
        if mx >= 104 && mx < 104 + render::SLOT_PX && my >= 62 && my < 62 + render::SLOT_PX {
            Self::swap_slot_with_cursor(&mut self.anvil_slots[0], &mut self.cursor, right);
            self.plat.play(Sfx::Click);
            return;
        }
        if mx >= 160 && mx < 160 + render::SLOT_PX && my >= 62 && my < 62 + render::SLOT_PX {
            Self::swap_slot_with_cursor(&mut self.anvil_slots[1], &mut self.cursor, right);
            self.plat.play(Sfx::Click);
            return;
        }
        if mx >= 224 && mx < 224 + render::SLOT_PX + 4 && my >= 60 && my < 60 + render::SLOT_PX + 4 {
            let out = self.anvil_output();
            if !out.is_empty() && self.cursor.is_empty() && self.player.level >= 2 {
                self.player.level -= 2;
                let cur = self.player.enchants.get(&out.id).copied().unwrap_or(0);
                self.player.enchants.insert(out.id, (cur + 1).min(3));
                self.anvil_slots[0] = Stack::EMPTY;
                self.anvil_slots[1].count -= 1;
                if self.anvil_slots[1].count == 0 { self.anvil_slots[1] = Stack::EMPTY; }
                self.cursor = out;
                self.plat.play(Sfx::LevelUp);
            }
            return;
        }
        for i in 0..HOT {
            let (x, y, w, h) = render::hot_slot_rect(i);
            if mx >= x && mx < x + w && my >= y && my < y + h {
                self.slot_click(right, 2, i);
                return;
            }
        }
    }

    fn click_brewing(&mut self, right: bool) {
        let (mx, my) = (self.mouse_x, self.mouse_y);
        let pos = self.ui_block;
        self.brewing_stands.entry(pos).or_insert_with(BrewingStand::new);
        if mx >= 180 && mx < 180 + render::SLOT_PX && my >= 36 && my < 36 + render::SLOT_PX {
            let bs = self.brewing_stands.get_mut(&pos).unwrap();
            Self::swap_slot_with_cursor(&mut bs.ing, &mut self.cursor, right);
            self.plat.play(Sfx::Click);
            return;
        }
        if mx >= 140 && mx < 140 + render::SLOT_PX && my >= 86 && my < 86 + render::SLOT_PX {
            let bs = self.brewing_stands.get_mut(&pos).unwrap();
            Self::swap_slot_with_cursor(&mut bs.bottle, &mut self.cursor, right);
            self.plat.play(Sfx::Click);
            return;
        }
        if mx >= 220 && mx < 220 + render::SLOT_PX && my >= 86 && my < 86 + render::SLOT_PX {
            let bs = self.brewing_stands.get_mut(&pos).unwrap();
            if !bs.output.is_empty() && self.cursor.is_empty() {
                self.cursor = bs.output;
                bs.output = Stack::EMPTY;
                self.plat.play(Sfx::Click);
            }
            return;
        }
        for i in 0..HOT {
            let (x, y, w, h) = render::hot_slot_rect(i);
            if mx >= x && mx < x + w && my >= y && my < y + h {
                self.slot_click(right, 2, i);
                return;
            }
        }
    }

    fn villager_trades() -> [(u16, u32, u16, u32); 5] {
        [
            (items::EMERALD, 1, items::BREAD, 6),
            (items::EMERALD, 1, items::ARROW, 16),
            (items::EMERALD, 3, items::GOLDEN_APPLE, 1),
            (items::EMERALD, 4, items::ENCHANTED_BOOK, 1),
            (items::EMERALD, 5, items::EYE_OF_ENDER, 1),
        ]
    }

    fn click_trade(&mut self) {
        let (mx, my) = (self.mouse_x, self.mouse_y);
        let trades = Self::villager_trades();
        for (i, &(cost_id, cost_n, out_id, out_n)) in trades.iter().enumerate() {
            let by = 30 + i as i32 * 30;
            if mx >= 82 && mx < 398 && my >= by && my < by + 26 {
                let have: u32 = self.player.hot.iter().chain(self.player.store.iter())
                    .filter(|s| s.id == cost_id).map(|s| s.count).sum();
                if have >= cost_n {
                    for _ in 0..cost_n { self.consume_item(cost_id); }
                    let left = self.player.add(out_id, out_n);
                    if left > 0 { self.drops.push(Drop::new(self.player.x, self.player.y + 1.0, self.player.z, out_id)); }
                    self.player.add_xp(3);
                    self.plat.play(Sfx::LevelUp);
                } else {
                    self.message = format!("Need {} {} to trade!", cost_n, items::name(cost_id));
                    self.msg_timer = 2.0;
                    self.plat.play(Sfx::Click);
                }
                return;
            }
        }
    }

    fn respawn_player(&mut self) {
        self.player.hp = 20.0;
        self.player.hunger = 20.0;
        self.player.air = 10.0;
        self.player.burn = 0.0;
        self.potion_fire = 0.0;
        self.potion_speed = 0.0;
        self.world.set_dim(Dimension::Overworld);
        self.player.x = self.spawn.0;
        self.player.y = self.spawn.1;
        self.player.z = self.spawn.2;
        self.fall_y = self.spawn.1;
        self.player.vy = 0.0;
        self.grace = 6.0;
        self.ui = Ui::None;
        self.plat.set_capture(true);
        self.plat.set_cursor(false);
        self.plat.play(Sfx::LevelUp);
    }

    fn click_death(&mut self) {
        let (mx, my) = (self.mouse_x, self.mouse_y);
        let bx = (render::W as i32 - 200) / 2;
        if mx >= bx && mx < bx + 200 && my >= 128 && my < 150 {
            self.respawn_player();
        } else if mx >= bx && mx < bx + 200 && my >= 158 && my < 180 {
            self.respawn_player();
            self.save_world();
            self.screen = Screen::Title;
            self.plat.set_capture(false);
            self.plat.set_cursor(true);
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
            let ench = self.player.enchants.get(&held_id).copied().unwrap_or(0) as f32;
            let dmg = match held_id {
                items::DIAMOND_SWORD => 7.0,
                items::IRON_SWORD => 6.0,
                items::WOOD_SWORD => 4.0,
                items::DIAMOND_AXE => 6.0,
                items::IRON_AXE => 5.0,
                items::WOOD_AXE => 3.5,
                _ => 1.0 + items::tool_tier(held_id) as f32 * 0.6,
            } + ench * 1.5;
            self.mobs[i].hp -= dmg;
            self.mobs[i].hurt = 0.4;
            self.damage_held_tool();
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
                let xp_gain = if m.kind == MobKind::EnderDragon { 500 } else if m.kind.hostile() { 5 } else { 2 };
                self.player.add_xp(xp_gain);
                self.message = format!("Defeated {}", m.kind.name());
                self.msg_timer = 1.5;
                self.plat.play(Sfx::LevelUp);
            }
            return;
        }
        self.mine_step(0.05);
    }

    fn damage_held_tool(&mut self) {
        if self.mode != GameMode::Survival { return; }
        let s = self.player.hot[self.player.selected];
        if items::tool_tier(s.id) == 0 && !items::is_hoe(s.id) && s.id != items::SHEARS && s.id != items::BOW { return; }
        let ench = self.player.enchants.get(&s.id).copied().unwrap_or(0);
        if ench >= 3 && self.tick % 2 == 0 { return; }
        let max_dur = match items::tool_tier(s.id) { 4 => 600, 3 => 180, 2 => 90, _ => 45 };
        self.tool_dur[self.player.selected] += 1;
        if self.tool_dur[self.player.selected] >= max_dur {
            self.tool_dur[self.player.selected] = 0;
            self.player.hot[self.player.selected] = Stack::EMPTY;
            self.plat.play(Sfx::Break);
        }
    }

    fn grow_oak_tree(&mut self, x: i32, y: i32, z: i32) {
        let h = 4 + (self.tick as i32 % 2);
        for ly in (y + h - 2)..=(y + h + 1) {
            let r = if ly >= y + h { 1 } else { 2 };
            for lx in x - r..=x + r {
                for lz in z - r..=z + r {
                    if (lx - x).abs() + (lz - z).abs() <= r + 1 && self.world.get(lx, ly, lz) == world::AIR {
                        self.world.set(lx, ly, lz, world::LEAVES);
                    }
                }
            }
        }
        for ty in y..(y + h) {
            self.world.set(x, ty, z, world::LOG);
        }
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

        let ench = self.player.enchants.get(&held_id).copied().unwrap_or(0) as f32;
        let tool_bonus = (if need > 0 {
            if has_pick { 1.5 + items::pickaxe_tier(held_id) as f32 * 1.1 } else { 0.22 }
        } else if is_wood && is_axe {
            1.6 + items::tool_tier(held_id) as f32 * 0.9
        } else if is_dirt && is_shovel {
            1.8 + items::tool_tier(held_id) as f32 * 1.0
        } else {
            1.0
        }) * (1.0 + ench * 0.35);
        let hardness = match h.block {
            world::OBSIDIAN | world::CRYING_OBSIDIAN | world::ANCIENT_DEBRIS | world::BEACON => 28.0,
            world::IRON_BLOCK | world::GOLD_BLOCK | world::DIAMOND_BLOCK | world::EMERALD_BLOCK | world::LAPIS_BLOCK | world::SPAWNER => 5.0,
            world::COAL_ORE | world::IRON_ORE | world::GOLD_ORE | world::DIAMOND_ORE | world::LAPIS_ORE
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
            world::TALL_GRASS | world::FLOWER_RED | world::FLOWER_YELLOW | world::SAPLING
                | world::BROWN_MUSHROOM | world::RED_MUSHROOM | world::TORCH | world::WHEAT | world::NETHER_WART => 0.08,
            _ => 0.9,
        };
        let mut prog = match self.mining { Some((p, v)) if p == pos => v, _ => 0.0 };
        prog += speed_mul * tool_bonus / hardness;
        if prog >= 1.0 {
            self.mining = None;
            self.damage_held_tool();
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
            let mut drop = block_drop(b, self.tick);
            let held_id = self.player.held().id;
            let fortune = self.player.enchants.get(&held_id).copied().unwrap_or(0);
            if fortune >= 2 && matches!(b, world::COAL_ORE | world::DIAMOND_ORE | world::EMERALD_ORE | world::LAPIS_ORE | world::REDSTONE_ORE | world::NETHER_QUARTZ_ORE) {
                drop.count += 1;
            }
            if !drop.is_empty() {
                let left = self.player.add(drop.id, drop.count);
                if left > 0 { self.drops.push(Drop::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5, drop.id)); }
            }
            let xp_gain = match b {
                world::COAL_ORE => 1,
                world::REDSTONE_ORE | world::NETHER_QUARTZ_ORE | world::LAPIS_ORE => 2,
                world::DIAMOND_ORE | world::EMERALD_ORE | world::SPAWNER => 4,
                _ => 0,
            };
            if xp_gain > 0 { self.player.add_xp(xp_gain); }
        } else if self.mode == GameMode::Survival {
            self.message = String::from("Need a better pickaxe to harvest that!");
            self.msg_timer = 1.8;
        }
        let above = self.world.get(x, y + 1, z);
        if matches!(above, world::TORCH | world::REDSTONE_TORCH | world::FLOWER_RED | world::FLOWER_YELLOW | world::TALL_GRASS | world::SAPLING | world::NETHER_WART) {
            self.world.set(x, y + 1, z, world::AIR);
        }
        self.furnaces.remove(&(x, y, z));
        self.brewing_stands.remove(&(x, y, z));
        self.buttons.remove(&(x, y, z));
        self.levers.remove(&(x, y, z));
        self.redstone_sources.remove(&(x, y, z));
        self.chests.remove(&(self.world.dim.idx() as u8, x, y, z));
        self.redstone_dirty = true;
        self.settle_gravity(x, y + 1, z);
        self.notify_fluid_neighbors(x, y, z);
    }

    fn try_use(&mut self) {
        if self.place_cd > 0.0 { return; }
        self.place_cd = 0.2;
        let (dx, dy, dz) = self.look_dir();
        let (ex, ey, ez) = self.eye();
        let held = self.player.held();

        // Bucket scooping (needs include_liquid = true)
        if held.id == items::BUCKET {
            if let Some(lh) = world::raycast(&mut self.world, ex, ey, ez, dx, dy, dz, REACH, true) {
                if lh.block == world::WATER || lh.block == world::LAVA {
                    let dim_i = self.world.dim.idx() as u8;
                    let is_water = lh.block == world::WATER;
                    let lvl = self.fluid_levels.get(&(dim_i, lh.x, lh.y, lh.z)).copied().unwrap_or(0);
                    if lvl == 0 {
                        self.world.set(lh.x, lh.y, lh.z, world::AIR);
                        self.fluid_levels.remove(&(dim_i, lh.x, lh.y, lh.z));
                        self.notify_fluid_neighbors(lh.x, lh.y, lh.z);
                        let new_id = if is_water { items::WATER_BUCKET } else { items::LAVA_BUCKET };
                        if self.mode == GameMode::Survival {
                            let s = &mut self.player.hot[self.player.selected];
                            if s.count <= 1 { *s = Stack::new(new_id, 1); }
                            else { s.count -= 1; self.player.add(new_id, 1); }
                        } else {
                            self.player.hot[self.player.selected] = Stack::new(new_id, 1);
                        }
                        self.plat.play(Sfx::Pickup);
                        return;
                    }
                }
            }
        }

        // Vehicle mounting / dismounting
        if self.riding.is_some() && !self.keys.contains(&plat::VK_SHIFT) {
            self.riding = None;
            self.player.vy = 4.0;
            self.plat.play(Sfx::Click);
            return;
        }
        for (vi, v) in self.vehicles.iter().enumerate() {
            let ox = v.x - ex; let oy = (v.y + 0.4) - ey; let oz = v.z - ez;
            let proj = ox * dx + oy * dy + oz * dz;
            if proj > 0.0 && proj < 4.5 {
                let cx = ox - dx * proj; let cy = oy - dy * proj; let cz = oz - dz * proj;
                if cx * cx + cy * cy + cz * cz < 0.9 * 0.9 {
                    self.riding = Some(vi);
                    self.message = String::from("Mounted! Press Space to dismount");
                    self.msg_timer = 2.0;
                    self.plat.play(Sfx::Click);
                    return;
                }
            }
        }

        // Mob interactions: Villager Trade, Wolf Tame/Sit, Passive Breeding
        let mut spawn_baby: Option<(MobKind, f32, f32, f32)> = None;
        for m in self.mobs.iter_mut() {
            let ox = m.x - ex; let oy = (m.y + 0.7) - ey; let oz = m.z - ez;
            let proj = ox * dx + oy * dy + oz * dz;
            if proj <= 0.0 || proj > 4.5 { continue; }
            let cx = ox - dx * proj; let cy = oy - dy * proj; let cz = oz - dz * proj;
            if cx * cx + cy * cy + cz * cz >= 0.85 * 0.85 { continue; }
            if m.kind == MobKind::Villager {
                self.ui = Ui::Trade;
                self.plat.set_capture(false);
                self.plat.set_cursor(true);
                self.plat.play(Sfx::Click);
                return;
            }
            if m.kind == MobKind::Wolf {
                if !m.tamed && held.id == items::BONE {
                    m.tamed = true;
                    m.hp = 20.0;
                    if self.mode == GameMode::Survival {
                        let s = &mut self.player.hot[self.player.selected];
                        s.count -= 1; if s.count == 0 { *s = Stack::EMPTY; }
                    }
                    self.message = String::from("Tamed Wolf!");
                    self.msg_timer = 2.5;
                    self.plat.play(Sfx::LevelUp);
                    return;
                } else if m.tamed {
                    m.sitting = !m.sitting;
                    self.message = String::from(if m.sitting { "Wolf sitting" } else { "Wolf following" });
                    self.msg_timer = 1.5;
                    self.plat.play(Sfx::Click);
                    return;
                }
            }
            if matches!(m.kind, MobKind::Cow | MobKind::Sheep | MobKind::Pig | MobKind::Chicken)
                && matches!(held.id, world::WHEAT | items::CARROT) && m.breed_cd <= 0.0
            {
                m.breed_cd = 20.0;
                if self.mode == GameMode::Survival {
                    let s = &mut self.player.hot[self.player.selected];
                    s.count -= 1; if s.count == 0 { *s = Stack::EMPTY; }
                }
                spawn_baby = Some((m.kind, m.x + 0.5, m.y, m.z + 0.5));
                break;
            }
        }
        if let Some((kind, bx, by, bz)) = spawn_baby {
            self.mobs.push(Mob::spawn(kind, bx, by, bz));
            self.player.add_xp(3);
            self.plat.play(Sfx::LevelUp);
            return;
        }

        // Firework boost while Elytra gliding, Potions, XP Bottle, Egg
        if held.id == items::FIREWORK && self.elytra_gliding {
            self.player.vy = (self.player.vy + dy * 12.0 + 4.0).clamp(-10.0, 14.0);
            self.player.x += dx * 2.5;
            self.player.z += dz * 2.5;
            if self.mode == GameMode::Survival {
                let s = &mut self.player.hot[self.player.selected];
                s.count -= 1; if s.count == 0 { *s = Stack::EMPTY; }
            }
            self.plat.play(Sfx::Explode);
            return;
        }
        if matches!(held.id, items::POTION_HEAL | items::POTION_FIRE | items::POTION_SPEED) {
            match held.id {
                items::POTION_HEAL => { self.player.hp = (self.player.hp + 8.0).min(20.0); self.message = String::from("Potion of Healing (+4 hearts)"); }
                items::POTION_FIRE => { self.potion_fire = 90.0; self.player.burn = 0.0; self.message = String::from("Fire Resistance (90s)"); }
                _ => { self.potion_speed = 90.0; self.message = String::from("Swiftness (90s)"); }
            }
            self.msg_timer = 2.5;
            if self.mode == GameMode::Survival {
                let s = &mut self.player.hot[self.player.selected];
                if s.count <= 1 { *s = Stack::new(items::GLASS_BOTTLE, 1); }
                else { s.count -= 1; self.player.add(items::GLASS_BOTTLE, 1); }
            }
            self.plat.play(Sfx::LevelUp);
            return;
        }
        if held.id == items::XP_BOTTLE {
            self.player.add_xp(10);
            if self.mode == GameMode::Survival {
                let s = &mut self.player.hot[self.player.selected];
                s.count -= 1; if s.count == 0 { *s = Stack::EMPTY; }
            }
            self.plat.play(Sfx::LevelUp);
            return;
        }
        if held.id == items::EGG {
            if self.tick % 4 == 0 {
                self.mobs.push(Mob::spawn(MobKind::Chicken, ex + dx * 2.0, ey, ez + dz * 2.0));
            }
            if self.mode == GameMode::Survival {
                let s = &mut self.player.hot[self.player.selected];
                s.count -= 1; if s.count == 0 { *s = Stack::EMPTY; }
            }
            self.plat.play(Sfx::Click);
            return;
        }

        // Boat placement on Water
        if held.id == items::BOAT {
            if let Some(lh) = world::raycast(&mut self.world, ex, ey, ez, dx, dy, dz, REACH, true) {
                if lh.block == world::WATER {
                    self.vehicles.push(Vehicle { is_boat: true, x: lh.x as f32 + 0.5, y: lh.y as f32 + 0.65, z: lh.z as f32 + 0.5, yaw: self.player.yaw });
                    if self.mode == GameMode::Survival {
                        let s = &mut self.player.hot[self.player.selected];
                        s.count -= 1; if s.count == 0 { *s = Stack::EMPTY; }
                    }
                    self.plat.play(Sfx::Place);
                    return;
                }
            }
        }

        // Bow shooting
        if held.id == items::BOW {
            let has_arrow = self.mode != GameMode::Survival || self.consume_item(items::ARROW);
            if has_arrow {
                self.arrows.push((ex + dx * 0.6, ey + dy * 0.6, ez + dz * 0.6, dx * 24.0, dy * 24.0 + 1.5, dz * 24.0));
                self.plat.play(Sfx::Click);
            } else {
                self.message = String::from("Out of Arrows!");
                self.msg_timer = 1.5;
            }
            return;
        }

        // Shears on Sheep
        if held.id == items::SHEARS {
            for m in self.mobs.iter_mut() {
                if m.kind == MobKind::Sheep {
                    let ox = m.x - ex; let oy = (m.y + 0.7) - ey; let oz = m.z - ez;
                    let proj = ox * dx + oy * dy + oz * dz;
                    if proj > 0.0 && proj < 4.5 {
                        let cx = ox - dx * proj; let cy = oy - dy * proj; let cz = oz - dz * proj;
                        if cx * cx + cy * cy + cz * cz < 0.8 * 0.8 {
                            self.drops.push(Drop::new(m.x, m.y + 0.8, m.z, world::WOOL_WHITE));
                            self.drops.push(Drop::new(m.x + 0.2, m.y + 0.8, m.z, world::WOOL_WHITE));
                            self.plat.play(Sfx::Break);
                            return;
                        }
                    }
                }
            }
        }

        let hit = world::raycast(&mut self.world, ex, ey, ez, dx, dy, dz, REACH, false);
        let Some(h) = hit else {
            if held.id == items::EYE_OF_ENDER {
                let dx_s = 100.0 - self.player.x;
                let dz_s = -140.0 - self.player.z;
                let dist = (dx_s * dx_s + dz_s * dz_s).sqrt();
                self.message = format!("Eye of Ender points toward Stronghold (100, 19, -140) - {:.0}m away", dist);
                self.msg_timer = 4.0;
                self.plat.play(Sfx::Click);
            }
            return;
        };
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
                world::CHEST => {
                    self.open_chest(target);
                    return;
                }
                world::ENCHANTING => {
                    self.ui = Ui::Enchant;
                    self.ui_block = target;
                    self.plat.set_capture(false);
                    self.plat.set_cursor(true);
                    self.plat.play(Sfx::Click);
                    return;
                }
                world::ANVIL => {
                    self.ui = Ui::Anvil;
                    self.ui_block = target;
                    self.plat.set_capture(false);
                    self.plat.set_cursor(true);
                    self.plat.play(Sfx::Click);
                    return;
                }
                world::BREWING_STAND => {
                    self.ui = Ui::Brewing;
                    self.ui_block = target;
                    self.brewing_stands.entry(target).or_insert_with(BrewingStand::new);
                    self.plat.set_capture(false);
                    self.plat.set_cursor(true);
                    self.plat.play(Sfx::Click);
                    return;
                }
                world::DOOR => {
                    self.world.set(h.x, h.y, h.z, world::DOOR_OPEN);
                    self.plat.play(Sfx::Click);
                    return;
                }
                world::DOOR_OPEN => {
                    self.world.set(h.x, h.y, h.z, world::DOOR);
                    self.plat.play(Sfx::Click);
                    return;
                }
                world::IRON_DOOR | world::IRON_DOOR_OPEN => {
                    self.message = String::from("Iron Doors open with Redstone power!");
                    self.msg_timer = 1.8;
                    self.plat.play(Sfx::Click);
                    return;
                }
                world::BUTTON => {
                    self.buttons.insert(target, 1.2);
                    self.redstone_sources.insert(target);
                    self.redstone_dirty = true;
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

        if held.is_empty() { return; }
        // Hoe -> till Dirt/Grass into Farmland
        if items::is_hoe(held.id) && matches!(h.block, world::DIRT | world::GRASS | world::PODZOL) {
            self.world.set(h.x, h.y, h.z, world::FARMLAND);
            self.damage_held_tool();
            self.plat.play(Sfx::Place);
            return;
        }
        // Bone Meal -> grow Sapling into Oak Tree, or sprout grass/flowers
        if held.id == items::BONE_MEAL {
            let used = if h.block == world::SAPLING {
                self.grow_oak_tree(h.x, h.y, h.z);
                true
            } else if h.block == world::GRASS && self.world.get(h.x, h.y + 1, h.z) == world::AIR {
                let plant = if self.tick % 3 == 0 { world::FLOWER_RED } else if self.tick % 3 == 1 { world::FLOWER_YELLOW } else { world::TALL_GRASS };
                self.world.set(h.x, h.y + 1, h.z, plant);
                true
            } else { false };
            if used {
                if self.mode == GameMode::Survival {
                    let s = &mut self.player.hot[self.player.selected];
                    s.count -= 1; if s.count == 0 { *s = Stack::EMPTY; }
                }
                self.plat.play(Sfx::LevelUp);
                return;
            }
        }
        // Minecart placement on Rail
        if held.id == items::MINECART && h.block == world::RAIL {
            self.vehicles.push(Vehicle { is_boat: false, x: h.x as f32 + 0.5, y: h.y as f32 + 0.15, z: h.z as f32 + 0.5, yaw: self.player.yaw });
            if self.mode == GameMode::Survival {
                let s = &mut self.player.hot[self.player.selected];
                s.count -= 1; if s.count == 0 { *s = Stack::EMPTY; }
            }
            self.plat.play(Sfx::Place);
            return;
        }
        // Water Bucket / Lava Bucket placement
        if held.id == items::WATER_BUCKET || held.id == items::LAVA_BUCKET {
            let px = h.x + h.nx;
            let py = h.y + h.ny;
            let pz = h.z + h.nz;
            let ex_b = self.world.get(px, py, pz);
            if !world::is_solid(ex_b) {
                let liq = if held.id == items::WATER_BUCKET { world::WATER } else { world::LAVA };
                let dim_i = self.world.dim.idx() as u8;
                self.world.set(px, py, pz, liq);
                self.fluid_levels.insert((dim_i, px, py, pz), 0);
                self.fluid_queue.push_back((dim_i, px, py, pz));
                self.notify_fluid_neighbors(px, py, pz);
                if self.mode == GameMode::Survival {
                    self.player.hot[self.player.selected] = Stack::new(items::BUCKET, 1);
                }
                self.plat.play(Sfx::Place);
            }
            return;
        }
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
            if held.id == world::BREWING_STAND { self.brewing_stands.insert((px, py, pz), BrewingStand::new()); }
            if matches!(held.id, world::REDSTONE_TORCH | world::LEVER | world::PRESSURE_PLATE | world::BUTTON | world::REPEATER | world::COMPARATOR) {
                self.redstone_sources.insert((px, py, pz));
            }
            self.redstone_dirty = true;
            self.settle_gravity(px, py, pz);
            self.notify_fluid_neighbors(px, py, pz);
        }
    }

    fn consume_item(&mut self, id: u16) -> bool {
        for s in self.player.hot.iter_mut() {
            if s.id == id && s.count > 0 {
                s.count -= 1;
                if s.count == 0 { *s = Stack::EMPTY; }
                return true;
            }
        }
        for s in self.player.store.iter_mut() {
            if s.id == id && s.count > 0 {
                s.count -= 1;
                if s.count == 0 { *s = Stack::EMPTY; }
                return true;
            }
        }
        false
    }

    fn open_chest(&mut self, pos: (i32, i32, i32)) {
        let key = (self.world.dim.idx() as u8, pos.0, pos.1, pos.2);
        if !self.chests.contains_key(&key) {
            let mut inv = [Stack::EMPTY; STORE];
            let h = ((pos.0 as u32).wrapping_mul(73856093))
                ^ ((pos.1 as u32).wrapping_mul(19349663))
                ^ ((pos.2 as u32).wrapping_mul(83492791));
            // Give dungeon / village / ruined portal chests realistic starter loot
            let loot = [
                items::BREAD, items::IRON_INGOT, items::COAL, world::TORCH,
                items::APPLE, items::GOLD_INGOT, items::ARROW, items::EMERALD,
            ];
            let count = 2 + (h % 4) as usize;
            for i in 0..count {
                let idx = ((h >> (i * 4)) as usize) % STORE;
                let item = loot[((h >> (i * 3 + 2)) as usize) % loot.len()];
                let qty = 1 + ((h >> (i * 2)) % 4);
                inv[idx] = Stack::new(item, qty);
            }
            if (h % 7) == 0 { inv[13] = Stack::new(items::DIAMOND, 1); }
            self.chests.insert(key, inv);
        }
        self.ui = Ui::Chest;
        self.ui_block = pos;
        self.plat.set_capture(false);
        self.plat.set_cursor(true);
        self.plat.play(Sfx::Click);
    }

    fn settle_gravity(&mut self, x: i32, start_y: i32, z: i32) {
        let mut y = start_y;
        while y > 1 && y < 126 {
            let b = self.world.get(x, y, z);
            if !matches!(b, world::SAND | world::RED_SAND | world::GRAVEL) { break; }
            let mut dst = y;
            while dst > 1 {
                let below = self.world.get(x, dst - 1, z);
                if world::is_solid(below) { break; }
                dst -= 1;
            }
            if dst < y {
                self.world.set(x, y, z, world::AIR);
                self.world.set(x, dst, z, b);
                self.notify_fluid_neighbors(x, y, z);
                self.notify_fluid_neighbors(x, dst, z);
                y += 1;
            } else {
                break;
            }
        }
    }

    fn notify_fluid_neighbors(&mut self, x: i32, y: i32, z: i32) {
        let dim_i = self.world.dim.idx() as u8;
        for (dx, dy, dz) in [(0, 0, 0), (1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)] {
            let nx = x + dx; let ny = y + dy; let nz = z + dz;
            if ny < 1 || ny >= 126 { continue; }
            let nb = self.world.get(nx, ny, nz);
            if nb == world::WATER || nb == world::LAVA {
                self.fluid_queue.push_back((dim_i, nx, ny, nz));
            }
        }
    }

    fn step_fluids(&mut self) {
        let dim_i = self.world.dim.idx() as u8;
        let max_steps = 48;
        let mut steps = 0;
        while steps < max_steps {
            let Some((qd, x, y, z)) = self.fluid_queue.pop_front() else { break; };
            if qd != dim_i || y <= 1 || y >= 126 { continue; }
            let b = self.world.get(x, y, z);
            if b != world::WATER && b != world::LAVA { continue; }
            steps += 1;

            let is_water = b == world::WATER;
            let max_spread: u8 = if is_water { 7 } else { 3 };
            let lvl = self.fluid_levels.get(&(dim_i, x, y, z)).copied().unwrap_or(0);

            // Fluid mixing with neighbors
            let mut self_solidified = false;
            for (dx, dy, dz) in [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)] {
                let nx = x + dx; let ny = y + dy; let nz = z + dz;
                let nb = self.world.get(nx, ny, nz);
                if is_water && nb == world::LAVA {
                    let nl = self.fluid_levels.get(&(dim_i, nx, ny, nz)).copied().unwrap_or(0);
                    let res = if nl == 0 { world::OBSIDIAN } else { world::COBBLE };
                    self.world.set(nx, ny, nz, res);
                    self.fluid_levels.remove(&(dim_i, nx, ny, nz));
                    self.notify_fluid_neighbors(nx, ny, nz);
                } else if !is_water && nb == world::WATER {
                    if dy == -1 {
                        self.world.set(nx, ny, nz, world::STONE);
                        self.fluid_levels.remove(&(dim_i, nx, ny, nz));
                        self.notify_fluid_neighbors(nx, ny, nz);
                    } else if lvl == 0 {
                        // Source lava touching water: obsidian if water is moving, cobble on contact
                        self.world.set(x, y, z, world::OBSIDIAN);
                        self.fluid_levels.remove(&(dim_i, x, y, z));
                        self.notify_fluid_neighbors(x, y, z);
                        self_solidified = true;
                        break;
                    } else {
                        self.world.set(x, y, z, world::COBBLE);
                        self.fluid_levels.remove(&(dim_i, x, y, z));
                        self.notify_fluid_neighbors(x, y, z);
                        self_solidified = true;
                        break;
                    }
                }
            }
            if self_solidified { continue; }

            let can_enter = |id: u16| -> bool {
                id == world::AIR || matches!(
                    id,
                    world::TALL_GRASS | world::FLOWER_RED | world::FLOWER_YELLOW | world::SNOW_LAYER | world::TORCH | world::REDSTONE_TORCH
                )
            };

            // Flow downward first
            let below = self.world.get(x, y - 1, z);
            if can_enter(below) {
                self.world.set(x, y - 1, z, b);
                self.fluid_levels.insert((dim_i, x, y - 1, z), 1);
                self.fluid_queue.push_back((dim_i, x, y - 1, z));
                continue;
            } else if below == b {
                continue;
            }

            // Spread horizontally only if resting on a solid block and within spread limit
            if world::is_solid(below) && lvl < max_spread {
                let next_lvl = lvl + 1;
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let nx = x + dx; let nz = z + dz;
                    if (nx - self.player.x as i32).abs() > 72 || (nz - self.player.z as i32).abs() > 72 { continue; }
                    let nb = self.world.get(nx, y, nz);
                    if can_enter(nb) {
                        self.world.set(nx, y, nz, b);
                        self.fluid_levels.insert((dim_i, nx, y, nz), next_lvl);
                        self.fluid_queue.push_back((dim_i, nx, y, nz));
                    } else if nb == b {
                        let cur_nl = self.fluid_levels.get(&(dim_i, nx, y, nz)).copied().unwrap_or(0);
                        if cur_nl > next_lvl {
                            self.fluid_levels.insert((dim_i, nx, y, nz), next_lvl);
                            self.fluid_queue.push_back((dim_i, nx, y, nz));
                        }
                    }
                }
            }
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
                world::BUTTON => {
                    if self.buttons.contains_key(&(x, y, z)) { queue.push(((x, y, z), 15)); }
                }
                world::PRESSURE_PLATE => {
                    let on = (self.player.x - (x as f32 + 0.5)).abs() < 0.85
                        && (self.player.z - (z as f32 + 0.5)).abs() < 0.85
                        && (self.player.y - y as f32).abs() < 1.2;
                    if on { self.plates.insert((x, y, z)); queue.push(((x, y, z), 15)); }
                    else { self.plates.remove(&(x, y, z)); }
                }
                world::REPEATER | world::COMPARATOR => {}
                _ => { self.redstone_sources.remove(&(x, y, z)); }
            }
        }
        let mut qi = 0;
        let mut door_checks: HashSet<(i32, i32, i32)> = HashSet::new();
        let mut piston_checks: HashSet<(i32, i32, i32)> = HashSet::new();
        while qi < queue.len() {
            let ((x, y, z), p) = queue[qi];
            qi += 1;
            if self.power.get(&(x, y, z)).map_or(false, |v| *v >= p) { continue; }
            self.power.insert((x, y, z), p);
            for (dx, dy, dz) in [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)] {
                let n = (x + dx, y + dy, z + dz);
                let nb = self.world.get(n.0, n.1, n.2);
                if nb == world::LAMP { self.lamps.insert(n); }
                if matches!(nb, world::DOOR | world::DOOR_OPEN | world::IRON_DOOR | world::IRON_DOOR_OPEN) {
                    door_checks.insert(n);
                }
                if matches!(nb, world::PISTON | world::STICKY_PISTON) {
                    piston_checks.insert(n);
                }
                if nb == world::TNT {
                    self.world.set(n.0, n.1, n.2, world::AIR);
                    self.tnt_primed.push((n.0 as f32 + 0.5, n.1 as f32, n.2 as f32 + 0.5, 3.5));
                    self.plat.play(Sfx::Furnace);
                }
                if matches!(nb, world::REPEATER | world::COMPARATOR) && p > 0 {
                    if self.power.get(&n).map_or(true, |v| *v < 15) { queue.push((n, 15)); }
                } else if nb == world::REDSTONE_DUST && p > 1 {
                    let np = p - 1;
                    if self.power.get(&n).map_or(true, |v| *v < np) { queue.push((n, np)); }
                }
            }
        }
        // Automatically open/close powered Doors & Iron Doors
        for &(x, y, z) in &door_checks {
            let b = self.world.get(x, y, z);
            if b == world::DOOR { self.world.set(x, y, z, world::DOOR_OPEN); }
            else if b == world::IRON_DOOR { self.world.set(x, y, z, world::IRON_DOOR_OPEN); }
        }
        // Pistons extend upward (+Y) when powered, and Sticky Pistons retract on unpower
        for &(x, y, z) in &piston_checks {
            if !self.piston_ext.contains(&(x, y, z)) {
                let above = self.world.get(x, y + 1, z);
                let above2 = self.world.get(x, y + 2, z);
                if above != world::AIR && above != world::BEDROCK && above2 == world::AIR {
                    self.world.set(x, y + 2, z, above);
                    self.world.set(x, y + 1, z, world::AIR);
                    self.piston_ext.insert((x, y, z));
                    self.plat.play(Sfx::Place);
                }
            }
        }
        let prev_ext: Vec<(i32, i32, i32)> = self.piston_ext.iter().copied().collect();
        for pos in prev_ext {
            if !piston_checks.contains(&pos) {
                self.piston_ext.remove(&pos);
                if self.world.get(pos.0, pos.1, pos.2) == world::STICKY_PISTON {
                    let above = self.world.get(pos.0, pos.1 + 1, pos.2);
                    let above2 = self.world.get(pos.0, pos.1 + 2, pos.2);
                    if above == world::AIR && above2 != world::AIR && above2 != world::BEDROCK {
                        self.world.set(pos.0, pos.1 + 1, pos.2, above2);
                        self.world.set(pos.0, pos.1 + 2, pos.2, world::AIR);
                        self.plat.play(Sfx::Place);
                    }
                }
            }
        }
        self.world.lit = self.lamps.clone();
    }

    fn is_shield_blocking(&self) -> bool {
        self.mouse_right && (self.player.held().id == items::SHIELD || self.player.offhand.id == items::SHIELD)
    }

    fn damage_player(&mut self, raw: f32, is_fire: bool) {
        if self.mode != GameMode::Survival || self.grace > 0.0 { return; }
        if is_fire && self.potion_fire > 0.0 { return; }
        let mut dmg = raw;
        if !is_fire {
            let ap = self.player.armor_points() as f32;
            dmg *= 1.0 - (ap * 0.04).min(0.8);
            if self.is_shield_blocking() { dmg *= 0.25; }
        }
        self.player.hp -= dmg.max(0.5);
        self.hurt_flash = 0.45;
        self.plat.play(Sfx::Hurt);
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
            self.damage_player(dmg, false);
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
        self.potion_fire = (self.potion_fire - dt).max(0.0);
        self.potion_speed = (self.potion_speed - dt).max(0.0);
        let mut expired_btns = Vec::new();
        for (pos, t) in self.buttons.iter_mut() {
            *t -= dt;
            if *t <= 0.0 { expired_btns.push(*pos); }
        }
        if !expired_btns.is_empty() {
            for pos in expired_btns { self.buttons.remove(&pos); }
            self.redstone_dirty = true;
        }
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

        let ui_open = matches!(
            self.ui,
            Ui::Inventory | Ui::Creative | Ui::Furnace | Ui::Chest | Ui::Enchant | Ui::Anvil | Ui::Brewing | Ui::Trade | Ui::Death
        );
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
            if let Some(vi) = self.riding {
                if vi < self.vehicles.len() {
                    let is_boat = self.vehicles[vi].is_boat;
                    let vsp = if is_boat { 8.5 } else { 9.5 };
                    let vx = (sy * fwd + cy * strafe) * vsp;
                    let vz = (-cy * fwd + sy * strafe) * vsp;
                    let nx = self.vehicles[vi].x + vx * dt;
                    let nz = self.vehicles[vi].z + vz * dt;
                    let vy_b = self.world.get(nx.floor() as i32, self.vehicles[vi].y.floor() as i32, nz.floor() as i32);
                    let vy_below = self.world.get(nx.floor() as i32, (self.vehicles[vi].y - 0.5).floor() as i32, nz.floor() as i32);
                    let can_move = if is_boat { vy_b == world::WATER || vy_below == world::WATER || !world::is_solid(vy_b) }
                                   else { vy_b == world::RAIL || vy_below == world::RAIL || !world::is_solid(vy_b) };
                    if can_move {
                        self.vehicles[vi].x = nx;
                        self.vehicles[vi].z = nz;
                        if fwd.abs() + strafe.abs() > 0.05 { self.vehicles[vi].yaw = self.player.yaw; }
                    }
                    self.player.x = self.vehicles[vi].x;
                    self.player.y = self.vehicles[vi].y + 0.35;
                    self.player.z = self.vehicles[vi].z;
                    self.player.vy = 0.0;
                    self.fall_y = self.player.y;
                } else {
                    self.riding = None;
                }
            } else if self.mode == GameMode::Spectator {
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
                let feet_b = self.world.get(self.player.x.floor() as i32, (self.player.y + 0.2).floor() as i32, self.player.z.floor() as i32);
                let head_b = self.world.get(self.player.x.floor() as i32, (self.player.y + 1.2).floor() as i32, self.player.z.floor() as i32);
                let under_b = self.world.get(self.player.x.floor() as i32, (self.player.y - 0.15).floor() as i32, self.player.z.floor() as i32);
                let on_climbable = matches!(feet_b, world::LADDER | world::VINE) || matches!(head_b, world::LADDER | world::VINE);
                let in_cobweb = feet_b == world::COBWEB || head_b == world::COBWEB;

                let mut speed = if self.flying {
                    if sprinting { 11.0 } else { 7.8 }
                } else if self.elytra_gliding {
                    14.0
                } else if sneaking {
                    2.0
                } else if sprinting {
                    6.2
                } else {
                    4.3
                };
                if self.potion_speed > 0.0 { speed *= 1.35; }
                if under_b == world::SOUL_SAND { speed *= 0.55; }
                if under_b == world::ICE { speed *= 1.28; }
                if in_cobweb { speed *= 0.25; }

                let vx = (sy * fwd + cy * strafe) * speed;
                let vz = (-cy * fwd + sy * strafe) * speed;

                let in_water = feet_b == world::WATER;

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
                } else if on_climbable {
                    self.fall_y = self.player.y;
                    self.player.vy = if self.keys.contains(&plat::VK_SPACE) || fwd > 0.0 { 4.2 } else if sneaking { 0.0 } else { -2.2 };
                } else if self.elytra_gliding && self.player.armor[1].id == items::ELYTRA {
                    self.player.vy = (self.player.vy - 4.0 * dt).max(-2.2);
                    self.fall_y = self.player.y;
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
                    self.player.vy -= (if in_cobweb { 4.0 } else { 22.0 }) * dt;
                }
                let ny = self.player.y + self.player.vy * dt;
                if collides(&mut self.world, self.player.x, ny, self.player.z) {
                    if self.player.vy < 0.0 {
                        self.elytra_gliding = false;
                        if self.flying {
                            self.flying = false;
                        } else if !self.player.on_ground && !in_water && !on_climbable && self.mode == GameMode::Survival {
                            let fall_dist = self.fall_y - self.player.y;
                            if fall_dist > 3.5 && self.grace <= 0.0 {
                                let dmg = (fall_dist - 3.0).floor();
                                self.damage_player(dmg, false);
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

        // fluid simulation
        self.fluid_timer += dt;
        if self.fluid_timer >= 0.16 {
            self.fluid_timer = 0.0;
            self.step_fluids();
        }

        // arrows projectile update
        let mut arrows_keep = Vec::with_capacity(self.arrows.len());
        for (mut ax, mut ay, mut az, vx, mut vy, vz) in self.arrows.drain(..) {
            vy -= 18.0 * dt;
            ax += vx * dt;
            ay += vy * dt;
            az += vz * dt;
            let bx = ax.floor() as i32; let by = ay.floor() as i32; let bz = az.floor() as i32;
            let hit_b = self.world.get(bx, by, bz);
            let mut hit_mob = false;
            for m in self.mobs.iter_mut() {
                if (m.x - ax).abs() < 0.7 && (m.z - az).abs() < 0.7 && ay >= m.y && ay <= m.y + 1.8 {
                    m.hp -= 6.0;
                    m.hurt = 0.4;
                    m.vx += vx * 0.12;
                    m.vz += vz * 0.12;
                    hit_mob = true;
                    self.plat.play(Sfx::Hurt);
                    break;
                }
            }
            if !hit_mob && !world::is_solid(hit_b) && ay > 0.0 && ay < 126.0 && (ax - self.player.x).abs() < 80.0 {
                arrows_keep.push((ax, ay, az, vx, vy, vz));
            } else if hit_b == world::TNT {
                self.world.set(bx, by, bz, world::AIR);
                self.tnt_primed.push((bx as f32 + 0.5, by as f32, bz as f32 + 0.5, 3.5));
            }
        }
        self.arrows = arrows_keep;

        // mob ranged projectiles (Skeleton arrows, Ghast/Blaze/Dragon fireballs)
        let prev_mp = std::mem::take(&mut self.mob_projectiles);
        let mut mp_keep = Vec::with_capacity(prev_mp.len());
        let mut fireball_blasts: Vec<(f32, f32, f32)> = Vec::new();
        for (mut mx, mut my, mut mz, vx, mut vy, vz, is_fb) in prev_mp {
            if !is_fb { vy -= 9.0 * dt; }
            mx += vx * dt; my += vy * dt; mz += vz * dt;
            let hit_b = self.world.get(mx.floor() as i32, my.floor() as i32, mz.floor() as i32);
            let pd = ((mx - self.player.x).powi(2) + (my - (self.player.y + 1.0)).powi(2) + (mz - self.player.z).powi(2)).sqrt();
            if pd < 0.9 && self.mode == GameMode::Survival {
                self.damage_player(if is_fb { 5.0 } else { 3.0 }, is_fb);
                if is_fb { fireball_blasts.push((mx, my, mz)); }
            } else if world::is_solid(hit_b) {
                if is_fb { fireball_blasts.push((mx, my, mz)); }
            } else if my > 0.0 && my < 126.0 && (mx - self.player.x).abs() < 80.0 {
                mp_keep.push((mx, my, mz, vx, vy, vz, is_fb));
            }
        }
        self.mob_projectiles = mp_keep;
        for (bx, by, bz) in fireball_blasts { self.explode(bx, by, bz, 1.8); }

        // mobs + creeper blasts + ranged attacks + tamed Wolves + Ender Dragon victory + mob lava/XP
        let mut creeper_blasts: Vec<(f32, f32, f32)> = Vec::new();
        let mut dragon_killed = false;
        let mut xp_from_mobs = 0u32;
        for m in self.mobs.iter_mut() {
            m.update(&mut self.world, self.player.x, self.player.y, self.player.z, dt, self.tick, day, self.grace, self.mode != GameMode::Survival);
            if matches!(m.kind, MobKind::Skeleton | MobKind::Ghast | MobKind::Blaze | MobKind::EnderDragon)
                && self.mode == GameMode::Survival && self.grace <= 0.0 && m.ranged_cd <= 0.0
            {
                let dx_p = self.player.x - m.x;
                let dy_p = (self.player.y + 1.0) - (m.y + 1.2);
                let dz_p = self.player.z - m.z;
                let d = (dx_p * dx_p + dy_p * dy_p + dz_p * dz_p).sqrt();
                let max_r = if m.kind == MobKind::Ghast || m.kind == MobKind::EnderDragon { 36.0 } else { 18.0 };
                if d > 3.5 && d < max_r {
                    m.ranged_cd = if m.kind == MobKind::EnderDragon { 2.8 } else { 3.2 };
                    let sp = if m.kind == MobKind::Skeleton { 16.0 } else { 12.0 };
                    let is_fb = m.kind != MobKind::Skeleton;
                    self.mob_projectiles.push((m.x, m.y + 1.2, m.z, dx_p / d * sp, dy_p / d * sp + if is_fb { 0.0 } else { 1.5 }, dz_p / d * sp, is_fb));
                }
            }
            let mb = self.world.get(m.x.floor() as i32, (m.y + 0.2).floor() as i32, m.z.floor() as i32);
            if mb == world::LAVA && !matches!(m.kind, MobKind::Blaze | MobKind::Ghast | MobKind::ZombifiedPiglin) {
                m.hp -= 6.0 * dt;
                m.hurt = 0.2;
            }
            if m.kind == MobKind::Creeper && m.fuse >= 1.5 {
                m.hp = 0.0;
                creeper_blasts.push((m.x, m.y + 0.5, m.z));
            }
            if m.hp <= 0.0 {
                xp_from_mobs += if m.kind == MobKind::EnderDragon { 500 } else if m.kind.hostile() { 5 } else { 2 };
                if m.kind == MobKind::EnderDragon { dragon_killed = true; }
            }
        }
        // Tamed Wolves protect the player from nearby hostiles
        let wolf_positions: Vec<(f32, f32, f32)> = self.mobs.iter()
            .filter(|m| m.kind == MobKind::Wolf && m.tamed && !m.sitting && m.hp > 0.0)
            .map(|m| (m.x, m.y, m.z))
            .collect();
        for (wx, wy, wz) in wolf_positions {
            for target in self.mobs.iter_mut() {
                if target.kind.hostile() && target.hp > 0.0 && target.attack_cd <= 0.0 {
                    let d = ((target.x - wx).powi(2) + (target.y - wy).powi(2) + (target.z - wz).powi(2)).sqrt();
                    if d < 2.5 {
                        target.hp -= 3.5;
                        target.hurt = 0.35;
                        target.attack_cd = 0.8;
                        break;
                    }
                }
            }
        }
        if xp_from_mobs > 0 { self.player.add_xp(xp_from_mobs); }
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
        for i in 0..self.mobs.len() {
            if self.mobs[i].kind.hostile() && self.mobs[i].kind != MobKind::Creeper {
                let d = ((self.mobs[i].x - self.player.x).powi(2) + (self.mobs[i].y - self.player.y).powi(2) + (self.mobs[i].z - self.player.z).powi(2)).sqrt();
                if self.mode == GameMode::Survival && d < 1.8 && self.mobs[i].attack_cd <= 0.0 && self.hurt_cd <= 0.0 && self.grace <= 0.0 {
                    self.mobs[i].attack_cd = 1.2;
                    self.hurt_cd = 0.7;
                    self.damage_player(2.0, false);
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

        // Brewing stands
        for bs in self.brewing_stands.values_mut() {
            let out_id = match bs.ing.id {
                items::APPLE | items::GOLDEN_APPLE => Some(items::POTION_HEAL),
                items::BLAZE_ROD | world::MAGMA => Some(items::POTION_FIRE),
                world::NETHER_WART | world::REDSTONE_DUST => Some(items::POTION_SPEED),
                _ => None,
            };
            if let Some(pid) = out_id {
                if bs.bottle.id == items::GLASS_BOTTLE && bs.bottle.count > 0 && bs.output.is_empty() {
                    bs.prog += dt / 5.0;
                    if bs.prog >= 1.0 {
                        bs.prog = 0.0;
                        bs.output = Stack::new(pid, 1);
                        bs.ing.count -= 1; if bs.ing.count == 0 { bs.ing = Stack::EMPTY; }
                        bs.bottle.count -= 1; if bs.bottle.count == 0 { bs.bottle = Stack::EMPTY; }
                    }
                } else { bs.prog = 0.0; }
            } else { bs.prog = 0.0; }
        }

        // Hoppers pull nearby drops into an adjacent Chest
        let mut hopper_picked = Vec::new();
        let dim_i = self.world.dim.idx() as u8;
        for (i, d) in self.drops.iter().enumerate() {
            let hx = d.x.floor() as i32; let hy = d.y.floor() as i32; let hz = d.z.floor() as i32;
            if self.world.get(hx, hy - 1, hz) == world::HOPPER || self.world.get(hx, hy, hz) == world::HOPPER {
                let hp_y = if self.world.get(hx, hy, hz) == world::HOPPER { hy } else { hy - 1 };
                for (dx, dy, dz) in [(0, -1, 0), (1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1)] {
                    let cx = hx + dx; let cy = hp_y + dy; let cz = hz + dz;
                    if self.world.get(cx, cy, cz) == world::CHEST {
                        let inv = self.chests.entry((dim_i, cx, cy, cz)).or_insert([Stack::EMPTY; STORE]);
                        for s in inv.iter_mut() {
                            if s.is_empty() { *s = Stack::new(d.item, 1); hopper_picked.push(i); break; }
                            if s.id == d.item && s.count < 64 { s.count += 1; hopper_picked.push(i); break; }
                        }
                        break;
                    }
                }
            }
        }
        for i in hopper_picked.into_iter().rev() { self.drops.remove(i); }

        // Sapling growth & Spawner mob spawning
        self.crop_timer += dt;
        if self.crop_timer >= 6.0 {
            self.crop_timer = 0.0;
            let px0 = self.player.x.floor() as i32;
            let py0 = self.player.y.floor() as i32;
            let pz0 = self.player.z.floor() as i32;
            let sx = px0 + ((self.tick as i32 * 7) % 17) - 8;
            let sz = pz0 + ((self.tick as i32 * 13) % 17) - 8;
            for sy in (py0 - 4).max(2)..=(py0 + 4).min(120) {
                if self.world.get(sx, sy, sz) == world::SAPLING {
                    self.grow_oak_tree(sx, sy, sz);
                    break;
                }
            }
        }
        self.spawner_timer += dt;
        if self.spawner_timer >= 8.0 {
            self.spawner_timer = 0.0;
            let px0 = self.player.x.floor() as i32;
            let py0 = self.player.y.floor() as i32;
            let pz0 = self.player.z.floor() as i32;
            if self.mobs.len() < 30 {
                'sp: for ox in -10..=10 {
                    for oy in -5..=5 {
                        for oz in -10..=10 {
                            if self.world.get(px0 + ox, py0 + oy, pz0 + oz) == world::SPAWNER {
                                let k = if self.world.dim == Dimension::Nether { MobKind::Blaze }
                                        else if (self.tick % 2) == 0 { MobKind::Zombie } else { MobKind::Skeleton };
                                self.mobs.push(Mob::spawn(k, (px0 + ox) as f32 + 1.5, (py0 + oy) as f32, (pz0 + oz) as f32 + 0.5));
                                break 'sp;
                            }
                        }
                    }
                }
            }
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

        // hunger + drowning + lava/magma/cactus hazards + health regeneration (Survival only)
        if self.mode != GameMode::Survival {
            self.player.hp = 20.0;
            self.player.hunger = 20.0;
            self.player.air = 10.0;
            self.player.burn = 0.0;
        } else {
        let head_b = self.world.get(self.player.x.floor() as i32, (self.player.y + 1.55).floor() as i32, self.player.z.floor() as i32);
        let feet_b = self.world.get(self.player.x.floor() as i32, (self.player.y + 0.1).floor() as i32, self.player.z.floor() as i32);
        let under_b = self.world.get(self.player.x.floor() as i32, (self.player.y - 0.1).floor() as i32, self.player.z.floor() as i32);
        if head_b == world::WATER {
            self.player.air = (self.player.air - dt * 0.7).max(0.0);
            if self.player.air <= 0.0 {
                self.drown_cd -= dt;
                if self.drown_cd <= 0.0 {
                    self.drown_cd = 1.0;
                    self.player.hp -= 2.0;
                    self.hurt_flash = 0.4;
                    self.plat.play(Sfx::Hurt);
                }
            }
        } else {
            self.player.air = (self.player.air + dt * 4.0).min(10.0);
            self.drown_cd = 0.0;
        }
        if feet_b == world::WATER || head_b == world::WATER {
            self.player.burn = 0.0;
        } else if (feet_b == world::LAVA || head_b == world::LAVA) && self.potion_fire <= 0.0 {
            self.player.burn = 6.0;
            if self.hurt_cd <= 0.0 {
                self.hurt_cd = 0.5;
                self.damage_player(4.0, true);
            }
        } else if under_b == world::MAGMA && !self.keys.contains(&plat::VK_SHIFT) && self.potion_fire <= 0.0 {
            if self.hurt_cd <= 0.0 {
                self.hurt_cd = 0.7;
                self.damage_player(1.0, true);
            }
        } else if feet_b == world::CACTUS || under_b == world::CACTUS {
            if self.hurt_cd <= 0.0 {
                self.hurt_cd = 0.6;
                self.damage_player(1.0, false);
            }
        }
        if self.player.burn > 0.0 {
            self.player.burn = (self.player.burn - dt).max(0.0);
            if self.potion_fire <= 0.0 {
                self.burn_cd -= dt;
                if self.burn_cd <= 0.0 {
                    self.burn_cd = 1.0;
                    self.damage_player(1.0, true);
                }
            }
        }
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

        // death / Totem of Undying
        if self.player.hp <= 0.0 {
            if self.player.has_totem() {
                if self.player.offhand.id == items::TOTEM {
                    self.player.offhand.count -= 1;
                    if self.player.offhand.count == 0 { self.player.offhand = Stack::EMPTY; }
                } else {
                    let s = &mut self.player.hot[self.player.selected];
                    s.count -= 1; if s.count == 0 { *s = Stack::EMPTY; }
                }
                self.player.hp = 10.0;
                self.player.burn = 0.0;
                self.potion_fire = 30.0;
                self.grace = 2.5;
                self.message = String::from("Totem of Undying saved you!");
                self.msg_timer = 4.0;
                self.plat.play(Sfx::LevelUp);
            } else {
                self.ui = Ui::Death;
                self.mouse_left = false;
                self.mouse_right = false;
                self.riding = None;
                self.plat.set_capture(false);
                self.plat.set_cursor(true);
                self.plat.play(Sfx::Hurt);
            }
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
        for v in self.vehicles.iter() {
            let kind = if v.is_boat { EntityKind::Boat } else { EntityKind::Minecart };
            render::draw_entity_3d(&mut self.scene, &cam, v.x, v.y, v.z, v.yaw, 0.0, day, kind, false, false);
        }
        for t in self.tnt_primed.iter() {
            let id = if (self.tick / 4) % 2 == 0 { world::SNOW } else { world::TNT };
            render::draw_drop_sprite(&mut self.scene, &cam, t.0, t.1, t.2, id, self.tick, day);
        }
        for d in self.drops.iter() {
            render::draw_drop_sprite(&mut self.scene, &cam, d.x, d.y, d.z, d.item, self.tick, day);
        }
        for &(ax, ay, az, _, _, _) in self.arrows.iter() {
            render::draw_drop_sprite(&mut self.scene, &cam, ax, ay, az, items::ARROW, self.tick, day);
        }
        for &(mx, my, mz, _, _, _, is_fb) in self.mob_projectiles.iter() {
            let id = if is_fb { items::BLAZE_ROD } else { items::ARROW };
            render::draw_drop_sprite(&mut self.scene, &cam, mx, my, mz, id, self.tick, day);
        }
        render::upscale(&self.scene, &mut self.fb);

        if self.ui == Ui::None || self.ui == Ui::Chat {
            if self.camera == 0 && self.mode != GameMode::Spectator {
                render::draw_held_item(&mut self.fb, self.player.hot[self.player.selected].id, self.tick);
            }
            let dragon_hp = if self.world.dim == Dimension::End {
                self.mobs.iter().find(|m| m.kind == MobKind::EnderDragon).map(|m| m.hp)
            } else { None };
            let shield_blk = self.is_shield_blocking();
            let armor_pts = self.player.armor_points();
            let cur_biome = self.world.biome(self.player.x.floor() as i32, self.player.z.floor() as i32);
            render::draw_hud(
                &mut self.fb,
                &self.player.hot.map(|s| s.id),
                &self.player.hot.map(|s| s.count),
                self.player.selected,
                self.player.hp,
                self.player.hunger,
                self.player.air,
                armor_pts,
                self.player.offhand.id,
                shield_blk,
                self.potion_fire,
                self.potion_speed,
                self.player.xp,
                self.player.level,
                self.player.burn,
                self.tick,
                &self.message,
                self.msg_timer,
                cur_biome,
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
                &mut self.fb, &self.player.hot, &self.player.store, &self.player.armor, self.player.offhand, &self.craft_grid,
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
        if self.ui == Ui::Chest {
            let key = (self.world.dim.idx() as u8, self.ui_block.0, self.ui_block.1, self.ui_block.2);
            let chest = self.chests.get(&key).copied().unwrap_or([Stack::EMPTY; STORE]);
            let hover = self.chest_hover();
            render::draw_chest_screen(
                &mut self.fb, &chest, &self.player.store, &self.player.hot, self.cursor, self.tick, hover,
            );
        }
        if self.ui == Ui::Enchant {
            let (mx, my) = (self.mouse_x, self.mouse_y);
            let mut hover = None;
            if mx >= 94 && mx < 94 + render::SLOT_PX && my >= 48 && my < 48 + render::SLOT_PX { hover = Some((0, 0)); }
            if mx >= 126 && mx < 126 + render::SLOT_PX && my >= 48 && my < 48 + render::SLOT_PX { hover = Some((0, 1)); }
            for i in 0..3 {
                let (bx, by, bw, bh) = (170, 36 + i as i32 * 26, 224, 22);
                if mx >= bx && mx < bx + bw && my >= by && my < by + bh { hover = Some((0, 10 + i)); }
            }
            for i in 0..HOT {
                let (x, y, w, h) = render::hot_slot_rect(i);
                if mx >= x && mx < x + w && my >= y && my < y + h { hover = Some((2, i)); }
            }
            render::draw_enchant_screen(&mut self.fb, self.enchant_slots[0], self.enchant_slots[1], self.player.level, &self.player.hot, self.cursor, self.tick, hover);
        }
        if self.ui == Ui::Anvil {
            let (mx, my) = (self.mouse_x, self.mouse_y);
            let mut hover = None;
            if mx >= 104 && mx < 104 + render::SLOT_PX && my >= 62 && my < 62 + render::SLOT_PX { hover = Some((0, 0)); }
            if mx >= 160 && mx < 160 + render::SLOT_PX && my >= 62 && my < 62 + render::SLOT_PX { hover = Some((0, 1)); }
            if mx >= 224 && mx < 224 + render::SLOT_PX + 4 && my >= 60 && my < 60 + render::SLOT_PX + 4 { hover = Some((0, 2)); }
            for i in 0..HOT {
                let (x, y, w, h) = render::hot_slot_rect(i);
                if mx >= x && mx < x + w && my >= y && my < y + h { hover = Some((2, i)); }
            }
            let anvil_out = self.anvil_output();
            render::draw_anvil_screen(&mut self.fb, self.anvil_slots[0], self.anvil_slots[1], anvil_out, self.player.level, &self.player.hot, self.cursor, self.tick, hover);
        }
        if self.ui == Ui::Brewing {
            let bs = self.brewing_stands.get(&self.ui_block).cloned().unwrap_or_else(BrewingStand::new);
            let (mx, my) = (self.mouse_x, self.mouse_y);
            let mut hover = None;
            if mx >= 180 && mx < 180 + render::SLOT_PX && my >= 36 && my < 36 + render::SLOT_PX { hover = Some((0, 0)); }
            if mx >= 140 && mx < 140 + render::SLOT_PX && my >= 86 && my < 86 + render::SLOT_PX { hover = Some((0, 1)); }
            if mx >= 220 && mx < 220 + render::SLOT_PX && my >= 86 && my < 86 + render::SLOT_PX { hover = Some((0, 2)); }
            for i in 0..HOT {
                let (x, y, w, h) = render::hot_slot_rect(i);
                if mx >= x && mx < x + w && my >= y && my < y + h { hover = Some((2, i)); }
            }
            render::draw_brewing_screen(&mut self.fb, bs.ing, bs.bottle, bs.output, bs.prog, &self.player.hot, self.cursor, self.tick, hover);
        }
        if self.ui == Ui::Trade {
            let (mx, my) = (self.mouse_x, self.mouse_y);
            let mut hover = None;
            for i in 0..5 {
                let by = 30 + i as i32 * 30;
                if mx >= 82 && mx < 398 && my >= by && my < by + 26 { hover = Some((0, i)); }
            }
            render::draw_trade_screen(&mut self.fb, &Self::villager_trades(), &self.player.hot, self.tick, hover);
        }
        if self.ui == Ui::Death {
            let (mx, my) = (self.mouse_x, self.mouse_y);
            let bx = (render::W as i32 - 200) / 2;
            let mut hover = usize::MAX;
            if mx >= bx && mx < bx + 200 && my >= 128 && my < 150 { hover = 0; }
            else if mx >= bx && mx < bx + 200 && my >= 158 && my < 180 { hover = 1; }
            render::draw_death_screen(&mut self.fb, self.player.level, hover);
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
        for i in 0..4 {
            let (x, y, w, h) = render::armor_slot_rect(i);
            if mx >= x && mx < x + w && my >= y && my < y + h { return Some((4, i)); }
        }
        let (ox, oy, ow, oh) = render::offhand_slot_rect();
        if mx >= ox && mx < ox + ow && my >= oy && my < oy + oh { return Some((4, 4)); }
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
        if cfg!(target_arch = "wasm32") { return; }
        let mut out = Vec::with_capacity(4096);
        out.extend_from_slice(b"MCS2");
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
        out.extend_from_slice(&self.player.xp.to_le_bytes());
        out.extend_from_slice(&self.player.level.to_le_bytes());
        for s in self.player.hot.iter().chain(self.player.store.iter()).chain(self.player.armor.iter()).chain(std::iter::once(&self.player.offhand)) {
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
        // MCS2 extras: enchants, chests, furnaces, levers, vehicles
        out.extend_from_slice(&(self.player.enchants.len() as u32).to_le_bytes());
        for (&id, &lvl) in &self.player.enchants {
            out.extend_from_slice(&id.to_le_bytes());
            out.push(lvl);
        }
        out.extend_from_slice(&(self.chests.len() as u32).to_le_bytes());
        for (&(d, x, y, z), inv) in &self.chests {
            out.push(d);
            out.extend_from_slice(&x.to_le_bytes());
            out.extend_from_slice(&y.to_le_bytes());
            out.extend_from_slice(&z.to_le_bytes());
            for s in inv {
                out.extend_from_slice(&s.id.to_le_bytes());
                out.extend_from_slice(&s.count.to_le_bytes());
            }
        }
        out.extend_from_slice(&(self.furnaces.len() as u32).to_le_bytes());
        for (&(x, y, z), f) in &self.furnaces {
            out.extend_from_slice(&x.to_le_bytes());
            out.extend_from_slice(&y.to_le_bytes());
            out.extend_from_slice(&z.to_le_bytes());
            for s in [f.input, f.fuel, f.output] {
                out.extend_from_slice(&s.id.to_le_bytes());
                out.extend_from_slice(&s.count.to_le_bytes());
            }
        }
        out.extend_from_slice(&(self.levers.len() as u32).to_le_bytes());
        for &(x, y, z) in &self.levers {
            out.extend_from_slice(&x.to_le_bytes());
            out.extend_from_slice(&y.to_le_bytes());
            out.extend_from_slice(&z.to_le_bytes());
        }
        out.extend_from_slice(&(self.vehicles.len() as u32).to_le_bytes());
        for v in &self.vehicles {
            out.push(if v.is_boat { 1 } else { 0 });
            out.extend_from_slice(&v.x.to_le_bytes());
            out.extend_from_slice(&v.y.to_le_bytes());
            out.extend_from_slice(&v.z.to_le_bytes());
            out.extend_from_slice(&v.yaw.to_le_bytes());
        }
        let _ = std::fs::write(Self::save_path(), out);
    }

    fn load_world(&mut self) {
        if cfg!(target_arch = "wasm32") { return; }
        let Ok(data) = std::fs::read(Self::save_path()) else { return; };
        if data.len() < 8 + 44 + 36 * 6 + 4 { return; }
        let is_v2 = &data[0..4] == b"MCS2";
        if !is_v2 && &data[0..4] != b"MCS1" { return; }
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
        if is_v2 {
            self.player.xp = read_u32(&data, &mut p);
            self.player.level = read_u32(&data, &mut p);
        }

        for s in self.player.hot.iter_mut().chain(self.player.store.iter_mut()) {
            let id = read_u16(&data, &mut p);
            let cnt = read_u32(&data, &mut p);
            *s = if id == 0 || cnt == 0 { Stack::EMPTY } else { Stack::new(id, cnt) };
        }
        if is_v2 {
            for s in self.player.armor.iter_mut().chain(std::iter::once(&mut self.player.offhand)) {
                let id = read_u16(&data, &mut p);
                let cnt = read_u32(&data, &mut p);
                *s = if id == 0 || cnt == 0 { Stack::EMPTY } else { Stack::new(id, cnt) };
            }
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
        if is_v2 && p + 4 <= data.len() {
            let n_ench = read_u32(&data, &mut p) as usize;
            for _ in 0..n_ench {
                if p + 3 > data.len() { break; }
                let id = read_u16(&data, &mut p);
                let lvl = read_u8(&data, &mut p);
                self.player.enchants.insert(id, lvl);
            }
            if p + 4 <= data.len() {
                let n_ch = read_u32(&data, &mut p) as usize;
                for _ in 0..n_ch {
                    if p + 13 + STORE * 6 > data.len() { break; }
                    let d = read_u8(&data, &mut p);
                    let x = read_i32(&data, &mut p);
                    let y = read_i32(&data, &mut p);
                    let z = read_i32(&data, &mut p);
                    let mut inv = [Stack::EMPTY; STORE];
                    for s in inv.iter_mut() {
                        let id = read_u16(&data, &mut p);
                        let cnt = read_u32(&data, &mut p);
                        *s = if id == 0 || cnt == 0 { Stack::EMPTY } else { Stack::new(id, cnt) };
                    }
                    self.chests.insert((d, x, y, z), inv);
                }
            }
            if p + 4 <= data.len() {
                let n_fur = read_u32(&data, &mut p) as usize;
                for _ in 0..n_fur {
                    if p + 12 + 18 > data.len() { break; }
                    let x = read_i32(&data, &mut p);
                    let y = read_i32(&data, &mut p);
                    let z = read_i32(&data, &mut p);
                    let mut f = Furnace::new();
                    for s in [&mut f.input, &mut f.fuel, &mut f.output] {
                        let id = read_u16(&data, &mut p);
                        let cnt = read_u32(&data, &mut p);
                        *s = if id == 0 || cnt == 0 { Stack::EMPTY } else { Stack::new(id, cnt) };
                    }
                    self.furnaces.insert((x, y, z), f);
                }
            }
            if p + 4 <= data.len() {
                let n_lev = read_u32(&data, &mut p) as usize;
                for _ in 0..n_lev {
                    if p + 12 > data.len() { break; }
                    let x = read_i32(&data, &mut p);
                    let y = read_i32(&data, &mut p);
                    let z = read_i32(&data, &mut p);
                    self.levers.insert((x, y, z));
                    self.redstone_sources.insert((x, y, z));
                    self.redstone_dirty = true;
                }
            }
            if p + 4 <= data.len() {
                let n_veh = read_u32(&data, &mut p) as usize;
                for _ in 0..n_veh {
                    if p + 17 > data.len() { break; }
                    let is_boat = read_u8(&data, &mut p) != 0;
                    let vx = read_f32(&data, &mut p);
                    let vy = read_f32(&data, &mut p);
                    let vz = read_f32(&data, &mut p);
                    let vyaw = read_f32(&data, &mut p);
                    self.vehicles.push(Vehicle { is_boat, x: vx, y: vy, z: vz, yaw: vyaw });
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
