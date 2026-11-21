// Software renderer: 16x16 face-aware pixel textures, voxel raycaster,
// 3D rasterized box-model mobs/player, isometric 3D block icons, and Minecraft UI.
use crate::entities::{self, FaceArt, MobKind, Part3D};
use crate::world::{self, Biome, Dimension, RenderCache, World};
use std::f32::consts::PI;
use std::sync::OnceLock;

pub const W: usize = 480;
pub const H: usize = 270;
#[allow(dead_code)]
pub const RW: usize = 480;
#[allow(dead_code)]
pub const RH: usize = 270;
const TEX: usize = 16;
const NUM_BLOCK_TEX: usize = 144;
const NUM_ITEM_TEX: usize = 128;

static ATLAS_BYTES: &[u8] = include_bytes!("../assets/textures.bin");
static ATLAS_CACHE: OnceLock<Textures> = OnceLock::new();

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum GameMode {
    Survival,
    Creative,
    Spectator,
}

impl GameMode {
    pub fn name(self) -> &'static str {
        match self {
            GameMode::Survival => "SURVIVAL",
            GameMode::Creative => "CREATIVE",
            GameMode::Spectator => "SPECTATOR",
        }
    }
    pub fn next(self) -> Self {
        match self {
            GameMode::Survival => GameMode::Creative,
            GameMode::Creative => GameMode::Spectator,
            GameMode::Spectator => GameMode::Survival,
        }
    }
}

pub struct Cam {
    pub x: f32, pub y: f32, pub z: f32,
    pub fx: f32, pub fy: f32, pub fz: f32,
    pub rx: f32, pub ry: f32, pub rz: f32,
    pub ux: f32, pub uy: f32, pub uz: f32,
}

impl Cam {
    pub fn from(yaw: f32, pitch: f32, x: f32, y: f32, z: f32) -> Self {
        let cp = pitch.cos();
        let fx = yaw.sin() * cp;
        let fy = pitch.sin();
        let fz = -yaw.cos() * cp;
        let rx = yaw.cos();
        let rz = yaw.sin();
        let ux = -rz * fy;
        let uy = rz * fx - rx * fz;
        let uz = rx * fy;
        Self { x, y, z, fx, fy, fz, rx, ry: 0.0, rz, ux, uy, uz }
    }
}

fn th(x: i32, y: i32, k: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374761393)
        ^ (y as u32).wrapping_mul(668265263)
        ^ (k as u32).wrapping_mul(2246822519);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

pub struct Textures {
    blocks: Vec<u8>,
    items: Vec<u8>,
}

impl Textures {
    pub fn atlas() -> &'static Textures {
        ATLAS_CACHE.get_or_init(Self::from_embedded)
    }

    pub fn new() -> Self {
        Self::from_embedded()
    }

    fn from_embedded() -> Self {
        assert!(ATLAS_BYTES.len() >= 8 && &ATLAS_BYTES[0..4] == b"MCTX");
        let nb = u16::from_le_bytes([ATLAS_BYTES[4], ATLAS_BYTES[5]]) as usize;
        let ni = u16::from_le_bytes([ATLAS_BYTES[6], ATLAS_BYTES[7]]) as usize;
        let b_end = 8 + nb * TEX * TEX * 4;
        let i_end = b_end + ni * TEX * TEX * 4;
        Self {
            blocks: ATLAS_BYTES[8..b_end].to_vec(),
            items: ATLAS_BYTES[b_end..i_end].to_vec(),
        }
    }

    pub fn face_kind(block: u16, _nx: i32, ny: i32, nz: i32) -> usize {
        match block {
            world::GRASS => {
                if ny > 0 { 1 } else if ny < 0 { 3 } else { 2 }
            }
            world::PODZOL => {
                if ny > 0 { 63 } else if ny < 0 { 3 } else { 2 }
            }
            world::MYCELIUM => {
                if ny > 0 { 64 } else if ny < 0 { 3 } else { 2 }
            }
            world::LOG | world::JUNGLE_LOG => {
                if ny != 0 { 6 } else { 5 }
            }
            world::BIRCH_LOG => {
                if ny != 0 { 6 } else { 71 }
            }
            world::SPRUCE_LOG | world::DARK_LOG => {
                if ny != 0 { 6 } else { 47 }
            }
            world::CRAFTING => {
                if ny > 0 { 41 } else if ny < 0 { 12 } else { 72 }
            }
            world::FURNACE => {
                if ny != 0 { 89 } else if nz > 0 { 42 } else { 73 }
            }
            world::TNT => {
                if ny != 0 { 74 } else { 70 }
            }
            world::BOOKSHELF => {
                if ny != 0 { 12 } else { 55 }
            }
            world::PUMPKIN => {
                if ny > 0 { 75 } else if nz > 0 { 95 } else { 44 }
            }
            world::CACTUS => {
                if ny != 0 { 88 } else { 24 }
            }
            world::CHEST => {
                if nz > 0 { 40 } else { 76 }
            }
            world::END_PORTAL_FRAME => {
                if ny > 0 { 97 } else if ny < 0 { 59 } else { 98 }
            }
            world::END_PORTAL_FRAME_EYE => {
                if ny > 0 { 99 } else if ny < 0 { 59 } else { 98 }
            }
            world::CRIMSON_NYLIUM => {
                if ny > 0 { 101 } else { 60 }
            }
            world::WARPED_NYLIUM => {
                if ny > 0 { 102 } else { 60 }
            }
            world::ANVIL => {
                if ny > 0 { 143 } else { 57 }
            }
            world::ENCHANTING => {
                if ny > 0 { 142 } else if ny < 0 { 35 } else { 58 }
            }
            world::HOPPER => {
                if ny > 0 { 133 } else { 132 }
            }
            world::PISTON => {
                if ny > 0 { 135 } else { 134 }
            }
            world::STICKY_PISTON => {
                if ny > 0 { 136 } else { 134 }
            }
            _ => Self::kind(block),
        }
    }

    pub fn kind(block: u16) -> usize {
        match block {
            world::AIR | world::STONE => 0,
            world::GRASS => 2,
            world::DIRT => 3,
            world::COBBLE => 4,
            world::LOG | world::JUNGLE_LOG => 5,
            world::LEAVES | world::JUNGLE_LEAVES => 7,
            world::SPRUCE_LEAVES | world::DARK_LEAVES => 77,
            world::BIRCH_LEAVES => 78,
            world::SAND => 8,
            world::RED_SAND => 79,
            world::SANDSTONE => 43,
            world::WATER => 9,
            world::SNOW | world::SNOW_LAYER => 10,
            world::ICE => 11,
            world::SPRUCE_LOG | world::DARK_LOG => 47,
            world::BIRCH_LOG => 71,
            world::PLANKS | world::FENCE | world::SIGN => 12,
            world::DOOR | world::DOOR_OPEN => 85,
            world::LADDER => 91,
            world::GLASS => 13,
            world::TORCH | world::CAMPFIRE => 14,
            world::BEDROCK => 15,
            world::COAL_ORE => 16,
            world::IRON_ORE => 17,
            world::GOLD_ORE => 18,
            world::DIAMOND_ORE => 19,
            world::REDSTONE_ORE => 20,
            world::EMERALD_ORE => 21,
            world::GRAVEL => 22,
            world::CLAY => 23,
            world::TERRACOTTA => 87,
            world::CACTUS => 24,
            world::TALL_GRASS | world::VINE => 25,
            world::FLOWER_RED | world::BROWN_MUSHROOM => 26,
            world::FLOWER_YELLOW | world::RED_MUSHROOM => 27,
            world::BRICK => 28,
            world::STONE_BRICKS => 29,
            world::WOOL_WHITE => 30,
            world::WOOL_RED | world::BED => 31,
            world::WOOL_BLUE => 32,
            world::WOOL_GREEN => 33,
            world::WOOL_YELLOW => 34,
            world::BLACKSTONE => 90,
            world::OBSIDIAN => 35,
            world::MAGMA => 36,
            world::LAVA => 37,
            world::FARMLAND => 38,
            world::WHEAT => 39,
            world::CHEST => 40,
            world::CRAFTING => 41,
            world::FURNACE => 42,
            world::PUMPKIN => 44,
            world::CAKE => 94,
            world::HAY => 45,
            world::TNT => 70,
            world::GLOWSTONE => 46,
            world::LANTERN => 92,
            world::SEA_LANTERN => 93,
            world::MOSSY => 84,
            world::COBWEB => 48,
            world::IRON_BLOCK => 83,
            world::GOLD_BLOCK => 80,
            world::DIAMOND_BLOCK => 81,
            world::EMERALD_BLOCK => 82,
            world::PODZOL => 63,
            world::MYCELIUM => 64,
            world::CONCRETE_WHITE => 49,
            world::CONCRETE_BLUE => 50,
            world::CONCRETE_RED => 51,
            world::CONCRETE_GREEN => 52,
            world::CONCRETE_YELLOW => 53,
            world::CONCRETE_BLACK => 54,
            world::BOOKSHELF => 55,
            world::RAIL => 56,
            world::ANVIL => 57,
            world::ENCHANTING => 58,
            world::END_STONE => 59,
            world::NETHERRACK => 60,
            world::SOUL_SAND => 61,
            world::QUARTZ => 62,
            world::SLAB => 0,
            world::STAIRS => 4,
            world::REDSTONE_DUST => 65,
            world::REDSTONE_TORCH => 66,
            world::LEVER => 67,
            world::LAMP => 68,
            world::PRESSURE_PLATE => 69,
            world::NETHER_PORTAL => 96,
            world::END_PORTAL_FRAME => 98,
            world::END_PORTAL_FRAME_EYE => 99,
            world::END_PORTAL => 115,
            world::CRYING_OBSIDIAN => 100,
            world::CRIMSON_NYLIUM => 101,
            world::WARPED_NYLIUM => 102,
            world::CRIMSON_STEM => 103,
            world::CRIMSON_WART => 104,
            world::WARPED_STEM => 105,
            world::WARPED_WART => 106,
            world::BASALT => 107,
            world::SHROOMLIGHT => 108,
            world::NETHER_QUARTZ_ORE => 109,
            world::ANCIENT_DEBRIS => 110,
            world::CHORUS_PLANT => 111,
            world::CHORUS_FLOWER => 112,
            world::PURPUR_BLOCK => 113,
            world::NETHER_BRICKS => 114,
            world::LAPIS_ORE => 128,
            world::LAPIS_BLOCK => 129,
            world::BREWING_STAND => 130,
            world::BEACON => 131,
            world::HOPPER => 132,
            world::PISTON => 134,
            world::STICKY_PISTON => 134,
            world::REPEATER => 137,
            world::COMPARATOR => 139,
            world::SPAWNER => 140,
            world::SAPLING => 141,
            world::NETHER_WART => 104,
            world::BUTTON => 0,
            world::IRON_DOOR | world::IRON_DOOR_OPEN => 83,
            _ => 0,
        }
    }

    #[inline(always)]
    pub fn sample_kind(&self, k: usize, u: f32, v: f32, tick: u32, animated: bool) -> [u8; 4] {
        let kk = k % NUM_BLOCK_TEX;
        let x = ((u.rem_euclid(1.0) * TEX as f32) as usize) & 15;
        let mut y = ((v.rem_euclid(1.0) * TEX as f32) as usize) & 15;
        if animated {
            y = (y + (tick / 6) as usize) & 15;
        }
        let i = (kk * 256 + y * 16 + x) * 4;
        [self.blocks[i], self.blocks[i + 1], self.blocks[i + 2], self.blocks[i + 3]]
    }

    #[inline(always)]
    pub fn sample_face(&self, block: u16, nx: i32, ny: i32, nz: i32, u: f32, v: f32, tick: u32) -> [u8; 4] {
        let k = if block == world::DOOR && ny == 0 {
            86 // door lower/panel look on side
        } else {
            Self::face_kind(block, nx, ny, nz)
        };
        let animated = block == world::WATER || block == world::LAVA || block == world::NETHER_PORTAL;
        let c = self.sample_kind(k, u, v, tick, animated);
        if block == world::ANVIL && c[3] < 128 {
            return [64, 64, 68, 255];
        }
        c
    }

    #[inline(always)]
    pub fn sample_item(&self, idx: usize, x: u32, y: u32) -> [u8; 4] {
        if idx >= NUM_ITEM_TEX { return [0, 0, 0, 0]; }
        let i = (idx * 256 + (y as usize & 15) * 16 + (x as usize & 15)) * 4;
        [self.items[i], self.items[i + 1], self.items[i + 2], self.items[i + 3]]
    }
}

pub struct Framebuffer {
    pub w: usize,
    pub h: usize,
    pub cursor: (i32, i32),
    pub rgba: Vec<u8>,
    pub depth: Vec<f32>,
}

impl Framebuffer {
    pub fn new(w: usize, h: usize) -> Self {
        Self { w, h, cursor: (0, 0), rgba: vec![0; w * h * 4], depth: vec![f32::INFINITY; w * h] }
    }
    pub fn clear(&mut self) {
        for d in self.depth.iter_mut() { *d = f32::INFINITY; }
    }
    #[inline]
    pub fn set(&mut self, x: i32, y: i32, r: u8, g: u8, b: u8, a: u8) {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 { return; }
        let i = (y as usize * self.w + x as usize) * 4;
        if a == 255 {
            self.rgba[i] = r; self.rgba[i + 1] = g; self.rgba[i + 2] = b; self.rgba[i + 3] = 255;
        } else {
            let t = a as f32 / 255.0;
            self.rgba[i] = (self.rgba[i] as f32 * (1.0 - t) + r as f32 * t) as u8;
            self.rgba[i + 1] = (self.rgba[i + 1] as f32 * (1.0 - t) + g as f32 * t) as u8;
            self.rgba[i + 2] = (self.rgba[i + 2] as f32 * (1.0 - t) + b as f32 * t) as u8;
            self.rgba[i + 3] = 255;
        }
    }
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: [u8; 4]) {
        for yy in y..y + h { for xx in x..x + w { self.set(xx, yy, c[0], c[1], c[2], c[3]); } }
    }
    pub fn frame(&mut self, x: i32, y: i32, w: i32, h: i32, c: [u8; 4]) {
        self.rect(x, y, w, 1, c); self.rect(x, y + h - 1, w, 1, c);
        self.rect(x, y, 1, h, c); self.rect(x + w - 1, y, 1, h, c);
    }
}

fn glyph(c: char) -> [u8; 5] {
    let u = c.to_ascii_uppercase();
    match u {
        'A' => [14, 17, 31, 17, 17], 'B' => [30, 17, 30, 17, 30], 'C' => [14, 17, 16, 17, 14],
        'D' => [30, 17, 17, 17, 30], 'E' => [31, 16, 30, 16, 31], 'F' => [31, 16, 30, 16, 16],
        'G' => [14, 17, 19, 17, 15], 'H' => [17, 17, 31, 17, 17], 'I' => [14, 4, 4, 4, 14],
        'J' => [7, 2, 2, 18, 12], 'K' => [17, 18, 28, 18, 17], 'L' => [16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 17, 17], 'N' => [17, 25, 21, 19, 17], 'O' => [14, 17, 17, 17, 14],
        'P' => [30, 17, 30, 16, 16], 'Q' => [14, 17, 21, 18, 13], 'R' => [30, 17, 30, 18, 17],
        'S' => [15, 16, 14, 1, 30], 'T' => [31, 4, 4, 4, 4], 'U' => [17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 10, 4], 'W' => [17, 17, 21, 27, 17], 'X' => [17, 10, 4, 10, 17],
        'Y' => [17, 10, 4, 4, 4], 'Z' => [31, 2, 4, 8, 31],
        '0' => [14, 17, 19, 17, 14], '1' => [4, 12, 4, 4, 14], '2' => [14, 17, 6, 16, 31],
        '3' => [30, 1, 14, 1, 30], '4' => [18, 18, 31, 2, 2], '5' => [31, 16, 30, 1, 30],
        '6' => [14, 16, 30, 17, 14], '7' => [31, 1, 2, 4, 4], '8' => [14, 17, 14, 17, 14],
        '9' => [14, 17, 15, 1, 14], ' ' => [0, 0, 0, 0, 0], '.' => [0, 0, 0, 0, 4],
        ':' => [0, 4, 0, 4, 0], '/' => [1, 1, 2, 4, 16], '-' => [0, 0, 31, 0, 0],
        '+' => [0, 4, 31, 4, 0], '[' => [14, 8, 8, 8, 14], ']' => [14, 2, 2, 2, 14],
        '%' => [17, 2, 4, 8, 17], '#' => [10, 31, 10, 31, 10],
        '<' => [2, 4, 8, 4, 2], '>' => [8, 4, 2, 4, 8], '!' => [4, 4, 4, 0, 4],
        '?' => [14, 17, 2, 0, 4], '_' => [0, 0, 0, 0, 31],
        '(' => [2, 4, 4, 4, 2], ')' => [8, 4, 4, 4, 8], ',' => [0, 0, 0, 4, 8],
        '\'' => [4, 4, 0, 0, 0], '=' => [0, 31, 0, 31, 0], '&' => [12, 18, 12, 19, 13],
        '^' => [4, 10, 17, 0, 0],
        _ => [0, 0, 0, 0, 0],
    }
}

pub fn text(fb: &mut Framebuffer, x: i32, y: i32, s: &str, color: [u8; 4], scale: i32) {
    let mut cx = x;
    for c in s.chars() {
        let g = glyph(c);
        for row in 0..5 {
            for col in 0..5 {
                if g[row] & (1 << (4 - col)) != 0 {
                    fb.rect(cx + col as i32 * scale, y + row as i32 * scale, scale, scale, color);
                }
            }
        }
        cx += 6 * scale;
    }
}

pub fn text_shadow(fb: &mut Framebuffer, x: i32, y: i32, s: &str, color: [u8; 4], scale: i32) {
    text(fb, x + scale, y + scale, s, [30, 30, 30, 220], scale);
    text(fb, x, y, s, color, scale);
}

fn day_factor(time: f32) -> f32 {
    let sun = (time * 2.0 * PI).sin();
    (sun * 2.2 + 0.35).clamp(0.0, 1.0)
}

fn sky_color(dim: Dimension, dir_y: f32, time: f32, x: i32, y: i32) -> [f32; 3] {
    match dim {
        Dimension::Nether => {
            let t = (dir_y * 0.5 + 0.5).clamp(0.0, 1.0);
            [52.0 + (1.0 - t) * 24.0, 12.0 + (1.0 - t) * 10.0, 10.0 + (1.0 - t) * 8.0]
        }
        Dimension::End => {
            let mut c = [14.0, 9.0, 24.0];
            if th(x, y, 91) % 620 == 0 {
                c = [195.0, 170.0, 230.0];
            }
            c
        }
        Dimension::Overworld => {
            let sun = (time * 2.0 * PI).sin();
            let day = day_factor(time);
            let horizon = 0.22 + sun.abs() * 0.10;
            let t = ((dir_y * 0.5 + 0.5) * (1.0 - horizon) + horizon).clamp(0.0, 1.0);
            let night_top = [8.0, 14.0, 36.0];
            let night_bot = [22.0, 26.0, 56.0];
            let day_top = [72.0, 148.0, 242.0];
            let day_bot = [178.0, 218.0, 252.0];
            let dusk = [240.0, 138.0, 88.0];
            let dusk_amt = (1.0 - sun.abs() * 3.0).clamp(0.0, 1.0) * 0.78;
            let mut c = [0.0; 3];
            for i in 0..3 {
                let mut v = night_bot[i] + (night_top[i] - night_bot[i]) * t;
                let dv = day_bot[i] + (day_top[i] - day_bot[i]) * t;
                v = v + (dv - v) * day;
                v = v * (1.0 - dusk_amt) + dusk[i] * dusk_amt * (1.0 - dir_y.abs().min(1.0)) * 0.72;
                c[i] = v;
            }
            if day < 0.2 && dir_y > 0.02 && th(x, y, 91) % 900 == 0 {
                c = [235.0, 235.0, 220.0];
            }
            c
        }
    }
}

#[inline]
fn cloud_color(cam: &Cam, dx: f32, dy: f32, dz: f32, time: f32, day: f32) -> Option<[f32; 3]> {
    if dy <= 0.03 { return None; }
    let t = (108.0 - cam.y) / dy;
    if t <= 0.0 || t > 260.0 { return None; }
    let wx = ((cam.x + dx * t) / 14.0 + time * 14.0).floor() as i32;
    let wz = ((cam.z + dz * t) / 14.0).floor() as i32;
    let h = th(wx / 2, wz / 2, 777) % 11;
    if h < 3 {
        let edge = (wx & 1) == 0 && (wz & 1) == 0;
        let v = if edge { 188.0 + day * 52.0 } else { 205.0 + day * 50.0 };
        Some([v, v, (v + 6.0).min(255.0)])
    } else { None }
}

#[inline]
fn face_light(nx: i32, ny: i32, nz: i32, day: f32) -> [f32; 3] {
    let base = if ny > 0 { 1.00 } else if ny < 0 { 0.46 } else if nz != 0 { 0.80 } else { 0.66 };
    let sun = 0.34 + day * 0.66;
    let warm = if ny > 0 { day * 0.14 } else if nx > 0 { day * 0.05 } else { 0.0 };
    let l = (base * sun + warm).min(1.12);
    if ny > 0 || nx > 0 {
        [l * (1.0 + day * 0.03), l * (1.0 + day * 0.01), l * (1.0 - day * 0.02)]
    } else {
        [l * (0.94 + (1.0 - day) * 0.04), l * 0.97, l * (1.02 + (1.0 - day) * 0.06)]
    }
}

#[inline]
fn ao_cache(cache: &RenderCache, bx: i32, by: i32, bz: i32, nx: i32, ny: i32, nz: i32) -> f32 {
    let mut occ = 0;
    let (ax, ay, az) = if ny != 0 { (1, 0, 0) } else if nx != 0 { (0, 1, 0) } else { (1, 0, 0) };
    let (cx, cy, cz) = if ny != 0 { (0, 0, 1) } else if nx != 0 { (0, 0, 1) } else { (0, 1, 0) };
    let sx = bx + nx; let sy = by + ny; let sz = bz + nz;
    for s in [-1, 1] {
        if world::is_opaque(cache.get(sx + ax * s, sy + ay * s, sz + az * s)) { occ += 1; }
        if world::is_opaque(cache.get(sx + cx * s, sy + cy * s, sz + cz * s)) { occ += 1; }
    }
    1.0 - (occ as f32) * 0.07
}

#[inline]
fn is_crossed_plant(b: u16) -> bool {
    matches!(
        b,
        world::TALL_GRASS
            | world::FLOWER_RED
            | world::FLOWER_YELLOW
            | world::WHEAT
            | world::BROWN_MUSHROOM
            | world::RED_MUSHROOM
            | world::SAPLING
            | world::NETHER_WART
    )
}

#[inline]
fn is_floor_block(b: u16) -> bool {
    matches!(b, world::RAIL | world::REDSTONE_DUST | world::REPEATER | world::COMPARATOR)
}

#[inline]
fn is_cutout_cube(b: u16) -> bool {
    matches!(
        b,
        world::GLASS | world::ICE | world::COBWEB | world::LADDER | world::VINE
            | world::LEVER | world::CAMPFIRE | world::SPAWNER | world::BEACON
    )
}

#[inline]
fn is_model_block(b: u16) -> bool {
    matches!(
        b,
        world::ANVIL | world::ENCHANTING | world::STAIRS | world::SLAB | world::FENCE
            | world::TORCH | world::REDSTONE_TORCH | world::BREWING_STAND | world::LANTERN
            | world::HOPPER | world::PRESSURE_PLATE | world::CAKE | world::FARMLAND
            | world::DOOR | world::DOOR_OPEN | world::IRON_DOOR | world::IRON_DOOR_OPEN
            | world::BUTTON | world::SNOW_LAYER
    )
}

/// Official Minecraft 3D model AABBs ([x0,y0,z0, x1,y1,z1] in 0..1 voxel space) derived from Minecraft 1.20.1 block models.
#[inline]
fn hit_model_boxes(
    cache: &RenderCache,
    b: u16,
    gx: i32, gy: i32, gz: i32,
    ox: f32, oy: f32, oz: f32,
    dx: f32, dy: f32, dz: f32,
    t_min: f32, t_max: f32,
    enx: i32, eny: i32, enz: i32,
) -> Option<(f32, i32, i32, i32, f32, f32)> {
    let mut boxes: [[f32; 6]; 8] = [[0.0; 6]; 8];
    let mut nb = 0usize;
    let p = 1.0 / 16.0;
    match b {
        world::SLAB => {
            boxes[0] = [0.0, 0.0, 0.0, 1.0, 0.5, 1.0]; nb = 1;
        }
        world::STAIRS => {
            boxes[0] = [0.0, 0.0, 0.0, 1.0, 0.5, 1.0];
            boxes[1] = [0.5, 0.5, 0.0, 1.0, 1.0, 1.0];
            nb = 2;
        }
        world::ANVIL => {
            boxes[0] = [2.0*p, 0.0, 2.0*p, 14.0*p, 4.0*p, 14.0*p];
            boxes[1] = [4.0*p, 4.0*p, 3.0*p, 12.0*p, 5.0*p, 13.0*p];
            boxes[2] = [6.0*p, 5.0*p, 4.0*p, 10.0*p, 10.0*p, 12.0*p];
            boxes[3] = [3.0*p, 10.0*p, 0.0, 13.0*p, 1.0, 1.0];
            nb = 4;
        }
        world::ENCHANTING => {
            boxes[0] = [0.0, 0.0, 0.0, 1.0, 12.0*p, 1.0]; nb = 1;
        }
        world::FENCE => {
            boxes[0] = [6.0*p, 0.0, 6.0*p, 10.0*p, 1.0, 10.0*p]; nb = 1;
            let conn = |id: u16| id == world::FENCE || world::is_solid(id);
            if conn(cache.get(gx, gy, gz - 1)) {
                boxes[nb] = [7.0*p, 12.0*p, 0.0, 9.0*p, 15.0*p, 6.0*p]; nb += 1;
                boxes[nb] = [7.0*p, 6.0*p, 0.0, 9.0*p, 9.0*p, 6.0*p]; nb += 1;
            }
            if conn(cache.get(gx, gy, gz + 1)) {
                boxes[nb] = [7.0*p, 12.0*p, 10.0*p, 9.0*p, 15.0*p, 1.0]; nb += 1;
                boxes[nb] = [7.0*p, 6.0*p, 10.0*p, 9.0*p, 9.0*p, 1.0]; nb += 1;
            }
            if conn(cache.get(gx - 1, gy, gz)) {
                boxes[nb] = [0.0, 12.0*p, 7.0*p, 6.0*p, 15.0*p, 9.0*p]; nb += 1;
                boxes[nb] = [0.0, 6.0*p, 7.0*p, 6.0*p, 9.0*p, 9.0*p]; nb += 1;
            }
            if conn(cache.get(gx + 1, gy, gz)) && nb + 2 <= 8 {
                boxes[nb] = [10.0*p, 12.0*p, 7.0*p, 1.0, 15.0*p, 9.0*p]; nb += 1;
                boxes[nb] = [10.0*p, 6.0*p, 7.0*p, 1.0, 9.0*p, 9.0*p]; nb += 1;
            }
        }
        world::TORCH | world::REDSTONE_TORCH => {
            boxes[0] = [7.0*p, 0.0, 7.0*p, 9.0*p, 10.0*p, 9.0*p]; nb = 1;
        }
        world::BREWING_STAND => {
            boxes[0] = [7.0*p, 0.0, 7.0*p, 9.0*p, 14.0*p, 9.0*p];
            boxes[1] = [9.0*p, 0.0, 5.0*p, 15.0*p, 2.0*p, 11.0*p];
            boxes[2] = [1.0*p, 0.0, 1.0*p, 7.0*p, 2.0*p, 7.0*p];
            boxes[3] = [1.0*p, 0.0, 9.0*p, 7.0*p, 2.0*p, 15.0*p];
            nb = 4;
        }
        world::LANTERN => {
            boxes[0] = [5.0*p, 0.0, 5.0*p, 11.0*p, 7.0*p, 11.0*p];
            boxes[1] = [6.0*p, 7.0*p, 6.0*p, 10.0*p, 9.0*p, 10.0*p];
            nb = 2;
        }
        world::HOPPER => {
            boxes[0] = [0.0, 10.0*p, 0.0, 1.0, 1.0, 1.0];
            boxes[1] = [4.0*p, 4.0*p, 4.0*p, 12.0*p, 10.0*p, 12.0*p];
            boxes[2] = [6.0*p, 0.0, 6.0*p, 10.0*p, 4.0*p, 10.0*p];
            nb = 3;
        }
        world::PRESSURE_PLATE => {
            boxes[0] = [1.0*p, 0.0, 1.0*p, 15.0*p, 1.0*p, 15.0*p]; nb = 1;
        }
        world::BUTTON => {
            boxes[0] = [5.0*p, 0.0, 6.0*p, 11.0*p, 2.0*p, 10.0*p]; nb = 1;
        }
        world::CAKE => {
            boxes[0] = [1.0*p, 0.0, 1.0*p, 15.0*p, 8.0*p, 15.0*p]; nb = 1;
        }
        world::FARMLAND => {
            boxes[0] = [0.0, 0.0, 0.0, 1.0, 15.0*p, 1.0]; nb = 1;
        }
        world::SNOW_LAYER => {
            boxes[0] = [0.0, 0.0, 0.0, 1.0, 2.0*p, 1.0]; nb = 1;
        }
        world::DOOR | world::IRON_DOOR => {
            boxes[0] = [0.0, 0.0, 0.0, 1.0, 1.0, 3.0*p]; nb = 1;
        }
        world::DOOR_OPEN | world::IRON_DOOR_OPEN => {
            boxes[0] = [0.0, 0.0, 0.0, 3.0*p, 1.0, 1.0]; nb = 1;
        }
        _ => {}
    }

    let rx = ox - gx as f32;
    let ry = oy - gy as f32;
    let rz = oz - gz as f32;
    let mut best_t = t_max;
    let mut best_hit: Option<(f32, i32, i32, i32, f32, f32)> = None;
    let invx = if dx.abs() > 1e-7 { 1.0 / dx } else { 1e9 };
    let invy = if dy.abs() > 1e-7 { 1.0 / dy } else { 1e9 };
    let invz = if dz.abs() > 1e-7 { 1.0 / dz } else { 1e9 };

    for bi in 0..nb {
        let bx = &boxes[bi];
        let mut t0 = t_min - 1e-4;
        let mut t1 = best_t + 1e-4;
        let mut n = (enx, eny, enz);

        let mut ax0 = (bx[0] - rx) * invx;
        let mut ax1 = (bx[3] - rx) * invx;
        let mut nax = -1;
        if ax0 > ax1 { std::mem::swap(&mut ax0, &mut ax1); nax = 1; }
        if ax0 > t0 { t0 = ax0; n = (nax, 0, 0); }
        t1 = t1.min(ax1);
        if t0 >= t1 { continue; }

        let mut ay0 = (bx[1] - ry) * invy;
        let mut ay1 = (bx[4] - ry) * invy;
        let mut nay = -1;
        if ay0 > ay1 { std::mem::swap(&mut ay0, &mut ay1); nay = 1; }
        if ay0 > t0 { t0 = ay0; n = (0, nay, 0); }
        t1 = t1.min(ay1);
        if t0 >= t1 { continue; }

        let mut az0 = (bx[2] - rz) * invz;
        let mut az1 = (bx[5] - rz) * invz;
        let mut naz = -1;
        if az0 > az1 { std::mem::swap(&mut az0, &mut az1); naz = 1; }
        if az0 > t0 { t0 = az0; n = (0, 0, naz); }
        t1 = t1.min(az1);
        if t0 < t1 && t0 > 0.001 && n != (0, 0, 0) {
            best_t = t0;
            let hx = rx + dx * t0;
            let hy = ry + dy * t0;
            let hz = rz + dz * t0;
            let (u, v) = if n.0 != 0 { (hz, 1.0 - hy) } else if n.1 != 0 { (hx, hz) } else { (hx, 1.0 - hy) };
            best_hit = Some((t0, n.0, n.1, n.2, u.clamp(0.0, 0.999), v.clamp(0.0, 0.999)));
        }
    }
    best_hit
}

#[inline]
fn shade_voxel(
    tex: &Textures,
    cache: &RenderCache,
    b: u16,
    bx: i32, by: i32, bz: i32,
    nx: i32, ny: i32, nz: i32,
    u: f32, v: f32,
    day: f32,
    tick: u32,
    apply_ao: bool,
) -> ([f32; 3], u8) {
    let t = tex.sample_face(b, nx, ny, nz, u, v, tick);
    if t[3] == 0 {
        return ([0.0, 0.0, 0.0], 0);
    }
    let fl = face_light(nx, ny, nz, day);
    let ao_v = if apply_ao { ao_cache(cache, bx, by, bz, nx, ny, nz) } else { 0.94 };
    let emit = world::emits_light(b);
    let mut lr = fl[0] * ao_v;
    let mut lg = fl[1] * ao_v;
    let mut lb = fl[2] * ao_v;
    let bl = cache.get_light(bx + nx, by + ny, bz + nz).max(cache.get_light(bx, by, bz)) as f32 / 15.0;
    if bl > 0.0 {
        let warm = bl * 0.96;
        lr = lr.max(warm * 1.05);
        lg = lg.max(warm * 0.96);
        lb = lb.max(warm * 0.82);
    }
    if emit > 0 {
        let e = 0.98 + emit as f32 * 0.025;
        lr = lr.max(e); lg = lg.max(e * 0.97); lb = lb.max(e * 0.90);
    }
    if b == world::LAMP && cache.is_lit(bx, by, bz) {
        lr = 1.48; lg = 1.38; lb = 1.15;
    }
    ([t[0] as f32 * lr, t[1] as f32 * lg, t[2] as f32 * lb], t[3])
}

#[inline]
fn apply_crack(
    tex: &Textures,
    mut rgb: [f32; 3],
    bx: i32, by: i32, bz: i32,
    u: f32, v: f32,
    tick: u32,
    mining: Option<(i32, i32, i32, u8)>,
) -> [f32; 3] {
    if let Some((mx, my, mz, stage)) = mining {
        if bx == mx && by == my && bz == mz {
            let st = (stage as usize).min(9);
            let cr = tex.sample_kind(116 + st, u, v, tick, false);
            if cr[3] > 20 {
                let m = (cr[0] as f32 / 255.0) * 0.65;
                rgb[0] *= m;
                rgb[1] *= m;
                rgb[2] *= m;
            }
        }
    }
    rgb
}

#[allow(clippy::too_many_arguments)]
fn trace_ray(
    tex: &Textures,
    cache: &RenderCache,
    cam: &Cam,
    dx: f32, dy: f32, dz: f32,
    sky: [f32; 3],
    day: f32,
    tick: u32,
    time: f32,
    underwater: bool,
    dim: Dimension,
    mining: Option<(i32, i32, i32, u8)>,
) -> ([u8; 3], f32) {
    let ox = cam.x; let oy = cam.y; let oz = cam.z;
    let mut x = ox.floor() as i32;
    let mut y = oy.floor() as i32;
    let mut z = oz.floor() as i32;
    let step_x = if dx > 0.0 { 1 } else if dx < 0.0 { -1 } else { 0 };
    let step_y = if dy > 0.0 { 1 } else if dy < 0.0 { -1 } else { 0 };
    let step_z = if dz > 0.0 { 1 } else if dz < 0.0 { -1 } else { 0 };
    let invx = if dx == 0.0 { f32::INFINITY } else { 1.0 / dx };
    let invy = if dy == 0.0 { f32::INFINITY } else { 1.0 / dy };
    let invz = if dz == 0.0 { f32::INFINITY } else { 1.0 / dz };
    let mut tmx = if step_x > 0 { ((x + 1) as f32 - ox) * invx } else if step_x < 0 { (ox - x as f32) * -invx } else { f32::INFINITY };
    let mut tmy = if step_y > 0 { ((y + 1) as f32 - oy) * invy } else if step_y < 0 { (oy - y as f32) * -invy } else { f32::INFINITY };
    let mut tmz = if step_z > 0 { ((z + 1) as f32 - oz) * invz } else if step_z < 0 { (oz - z as f32) * -invz } else { f32::INFINITY };
    let dtx = invx.abs();
    let dty = invy.abs();
    let dtz = invz.abs();
    let mut nx = 0; let mut ny = 0; let mut nz = 0;
    let mut t = 0.0f32;

    let mut has_trans = false;
    let mut trans_rgb = [0.0f32; 3];
    let mut trans_alpha = 0.0f32;
    let mut trans_dist = f32::INFINITY;

    let mut hit_rgb = sky;
    let mut hit_dist = f32::INFINITY;
    let max_dist = 76.0f32;
    let fog_rate = if dim == Dimension::Nether { 0.016 } else { 0.009 };

    // Precompute cloud color for Overworld sky fallback
    if dim == Dimension::Overworld {
        if let Some(cc) = cloud_color(cam, dx, dy, dz, time, day) {
            let fade = 0.50 + day * 0.42;
            hit_rgb = [
                hit_rgb[0] * (1.0 - fade) + cc[0] * fade,
                hit_rgb[1] * (1.0 - fade) + cc[1] * fade,
                hit_rgb[2] * (1.0 - fade) + cc[2] * fade,
            ];
        }
    }

    for _ in 0..220 {
        let t_next = tmx.min(tmy).min(tmz);
        if y >= 0 && y < world::WORLD_H as i32 {
            let b = cache.get(x, y, z);
            if b != world::AIR {
                if is_crossed_plant(b) {
                    let mut cands = [(f32::INFINITY, 0.0f32, 0.0f32); 2];
                    let mut nc = 0usize;
                    let denom1 = dx - dz;
                    if denom1.abs() > 1e-5 {
                        let tp = ((x as f32 - ox) - (z as f32 - oz)) / denom1;
                        if tp > t.max(0.001) && tp < t_next {
                            let px = ox + dx * tp - x as f32;
                            let py = oy + dy * tp - y as f32;
                            if (0.0..=1.0).contains(&px) && (0.0..=1.0).contains(&py) {
                                cands[nc] = (tp, px, 1.0 - py);
                                nc += 1;
                            }
                        }
                    }
                    let denom2 = dx + dz;
                    if denom2.abs() > 1e-5 {
                        let tp = (1.0 - (ox - x as f32) - (oz - z as f32)) / denom2;
                        if tp > t.max(0.001) && tp < t_next {
                            let px = ox + dx * tp - x as f32;
                            let py = oy + dy * tp - y as f32;
                            if (0.0..=1.0).contains(&px) && (0.0..=1.0).contains(&py) {
                                cands[nc] = (tp, px, 1.0 - py);
                                nc += 1;
                            }
                        }
                    }
                    if nc == 2 && cands[1].0 < cands[0].0 { cands.swap(0, 1); }
                    let mut hit_plant = false;
                    for ci in 0..nc {
                        let (tp, u, v) = cands[ci];
                        let (rgb0, a) = shade_voxel(tex, cache, b, x, y, z, 0, 1, 0, u, v, day, tick, false);
                        if a > 25 {
                            let rgb = apply_crack(tex, rgb0, x, y, z, u, v, tick, mining);
                            let fog = (1.0 - (-tp * fog_rate).exp()).min(0.86);
                            hit_rgb = [
                                rgb[0] * (1.0 - fog) + sky[0] * fog,
                                rgb[1] * (1.0 - fog) + sky[1] * fog,
                                rgb[2] * (1.0 - fog) + sky[2] * fog,
                            ];
                            hit_dist = tp;
                            hit_plant = true;
                            break;
                        }
                    }
                    if hit_plant { break; }
                } else if is_model_block(b) {
                    if let Some((tp, mnx, mny, mnz, u, v)) = hit_model_boxes(
                        cache, b, x, y, z, ox, oy, oz, dx, dy, dz, t.max(0.001), t_next, nx, ny, nz,
                    ) {
                        let (rgb0, a) = shade_voxel(tex, cache, b, x, y, z, mnx, mny, mnz, u, v, day, tick, false);
                        if a > 20 {
                            let rgb = apply_crack(tex, rgb0, x, y, z, u, v, tick, mining);
                            let fog = (1.0 - (-tp * fog_rate).exp()).min(0.86);
                            hit_rgb = [
                                rgb[0] * (1.0 - fog) + sky[0] * fog,
                                rgb[1] * (1.0 - fog) + sky[1] * fog,
                                rgb[2] * (1.0 - fog) + sky[2] * fog,
                            ];
                            hit_dist = tp;
                            break;
                        }
                    }
                } else if is_floor_block(b) {
                    if dy.abs() > 1e-5 {
                        let tp = (y as f32 + 0.06 - oy) / dy;
                        if tp > t.max(0.001) && tp < t_next {
                            let px = ox + dx * tp - x as f32;
                            let pz = oz + dz * tp - z as f32;
                            if (0.0..=1.0).contains(&px) && (0.0..=1.0).contains(&pz) {
                                let (rgb0, a) = shade_voxel(tex, cache, b, x, y, z, 0, 1, 0, px, pz, day, tick, false);
                                if a > 25 {
                                    let rgb = apply_crack(tex, rgb0, x, y, z, px, pz, tick, mining);
                                    let fog = (1.0 - (-tp * fog_rate).exp()).min(0.86);
                                    hit_rgb = [
                                        rgb[0] * (1.0 - fog) + sky[0] * fog,
                                        rgb[1] * (1.0 - fog) + sky[1] * fog,
                                        rgb[2] * (1.0 - fog) + sky[2] * fog,
                                    ];
                                    hit_dist = tp;
                                    break;
                                }
                            }
                        }
                    }
                } else if b == world::WATER || b == world::LAVA || b == world::NETHER_PORTAL || b == world::END_PORTAL {
                    if t > 0.001 {
                        let (u, v) = world::hit_uv(ox, oy, oz, dx, dy, dz, t, nx, ny, nz);
                        let tpx = tex.sample_face(b, nx, ny, nz, u, v, tick);
                        if b == world::WATER && !has_trans {
                            let fresnel = ((1.0 - dy.abs()).powf(2.5) * 0.35).clamp(0.05, 0.38);
                            let shimmer = if ((u * 16.0) as i32 + (v * 16.0) as i32 + (tick / 8) as i32) % 9 == 0 { 1.12 } else { 1.0 };
                            let light = 0.55 + day * 0.48;
                            trans_rgb = [
                                tpx[0] as f32 * light * shimmer * (1.0 - fresnel) + sky[0] * fresnel,
                                tpx[1] as f32 * light * shimmer * (1.0 - fresnel) + sky[1] * fresnel,
                                tpx[2] as f32 * light * shimmer * (1.0 - fresnel) + sky[2] * fresnel,
                            ];
                            trans_alpha = 0.62;
                            trans_dist = t;
                            has_trans = true;
                        } else if b == world::NETHER_PORTAL && !has_trans {
                            trans_rgb = [tpx[0] as f32 * 1.25, tpx[1] as f32 * 1.10, tpx[2] as f32 * 1.35];
                            trans_alpha = 0.72;
                            trans_dist = t;
                            has_trans = true;
                        } else if b == world::LAVA || b == world::END_PORTAL {
                            let fog = (1.0 - (-t * fog_rate).exp()).min(0.85);
                            hit_rgb = [
                                tpx[0] as f32 * 1.15 * (1.0 - fog) + sky[0] * fog,
                                tpx[1] as f32 * 1.15 * (1.0 - fog) + sky[1] * fog,
                                tpx[2] as f32 * 1.15 * (1.0 - fog) + sky[2] * fog,
                            ];
                            hit_dist = t;
                            break;
                        }
                    }
                } else if is_cutout_cube(b) {
                    if t > 0.001 {
                        let (u, v) = world::hit_uv(ox, oy, oz, dx, dy, dz, t, nx, ny, nz);
                        let (rgb0, a) = shade_voxel(tex, cache, b, x, y, z, nx, ny, nz, u, v, day, tick, true);
                        if a >= 240 {
                            let rgb = apply_crack(tex, rgb0, x, y, z, u, v, tick, mining);
                            let fog = (1.0 - (-t * fog_rate).exp()).min(0.86);
                            hit_rgb = [
                                rgb[0] * (1.0 - fog) + sky[0] * fog,
                                rgb[1] * (1.0 - fog) + sky[1] * fog,
                                rgb[2] * (1.0 - fog) + sky[2] * fog,
                            ];
                            hit_dist = t;
                            break;
                        } else if a > 20 && !has_trans {
                            trans_rgb = apply_crack(tex, rgb0, x, y, z, u, v, tick, mining);
                            trans_alpha = a as f32 / 255.0;
                            trans_dist = t;
                            has_trans = true;
                        }
                    }
                } else if t > 0.001 {
                    let (u, v) = world::hit_uv(ox, oy, oz, dx, dy, dz, t, nx, ny, nz);
                    let (rgb0, a) = shade_voxel(tex, cache, b, x, y, z, nx, ny, nz, u, v, day, tick, true);
                    if a > 20 {
                        let rgb = apply_crack(tex, rgb0, x, y, z, u, v, tick, mining);
                        let fog = (1.0 - (-t * fog_rate).exp()).min(0.86);
                        hit_rgb = [
                            rgb[0] * (1.0 - fog) + sky[0] * fog,
                            rgb[1] * (1.0 - fog) + sky[1] * fog,
                            rgb[2] * (1.0 - fog) + sky[2] * fog,
                        ];
                        hit_dist = t;
                        break;
                    }
                }
            }
        }
        if tmx < tmy && tmx < tmz { t = tmx; tmx += dtx; x += step_x; nx = -step_x; ny = 0; nz = 0; }
        else if tmy < tmz { t = tmy; tmy += dty; y += step_y; ny = -step_y; nx = 0; nz = 0; }
        else { t = tmz; tmz += dtz; z += step_z; nz = -step_z; nx = 0; ny = 0; }
        if t > max_dist { break; }
    }

    if hit_dist.is_infinite() && dim == Dimension::Overworld {
        let ang = time * 2.0 * PI;
        let mut sx = ang.cos();
        let mut sy = ang.sin();
        let mut sz = 0.22f32;
        let sl = (sx * sx + sy * sy + sz * sz).sqrt();
        sx /= sl; sy /= sl; sz /= sl;
        if sy > -0.1 && (dx - sx).abs() < 0.062 && (dy - sy).abs() < 0.062 && (dz - sz).abs() < 0.062 {
            hit_rgb = [255.0, 246.0, 185.0];
        }
        if -sy > -0.1 && (dx + sx).abs() < 0.048 && (dy + sy).abs() < 0.048 && (dz + sz).abs() < 0.048 {
            hit_rgb = [225.0, 230.0, 242.0];
        }
        if day < 0.35 && dy > 0.05 {
            let st = th((dx * 220.0) as i32, (dz * 220.0 + dy * 140.0) as i32, 91);
            if st % 520 == 0 {
                let br = (0.35 - day) * 2.5;
                hit_rgb[0] = (hit_rgb[0] + 180.0 * br).min(255.0);
                hit_rgb[1] = (hit_rgb[1] + 190.0 * br).min(255.0);
                hit_rgb[2] = (hit_rgb[2] + 220.0 * br).min(255.0);
            }
        }
    }

    if has_trans {
        let fog = (1.0 - (-trans_dist * fog_rate).exp()).min(0.85);
        let tr = trans_rgb[0] * (1.0 - fog) + sky[0] * fog;
        let tg = trans_rgb[1] * (1.0 - fog) + sky[1] * fog;
        let tb = trans_rgb[2] * (1.0 - fog) + sky[2] * fog;
        hit_rgb[0] = hit_rgb[0] * (1.0 - trans_alpha) + tr * trans_alpha;
        hit_rgb[1] = hit_rgb[1] * (1.0 - trans_alpha) + tg * trans_alpha;
        hit_rgb[2] = hit_rgb[2] * (1.0 - trans_alpha) + tb * trans_alpha;
        hit_dist = hit_dist.min(trans_dist);
    }

    if underwater {
        let uf = (1.0 - (-hit_dist.min(32.0) * 0.055).exp()).clamp(0.28, 0.85);
        hit_rgb[0] = hit_rgb[0] * (1.0 - uf) + 22.0 * uf;
        hit_rgb[1] = hit_rgb[1] * (1.0 - uf) + 68.0 * uf;
        hit_rgb[2] = hit_rgb[2] * (1.0 - uf) + 148.0 * uf;
    }

    (
        [
            hit_rgb[0].clamp(0.0, 255.0) as u8,
            hit_rgb[1].clamp(0.0, 255.0) as u8,
            hit_rgb[2].clamp(0.0, 255.0) as u8,
        ],
        hit_dist,
    )
}

pub fn render_world(
    fb: &mut Framebuffer,
    tex: &Textures,
    world: &mut World,
    cam: &Cam,
    time: f32,
    tick: u32,
    mining: Option<(i32, i32, i32, u8)>,
) {
    fb.clear();
    let dim = world.dim;
    let day = match dim {
        Dimension::Overworld => day_factor(time),
        Dimension::Nether => 0.72,
        Dimension::End => 0.64,
    };
    let w = fb.w;
    let h = fb.h;
    let aspect = w as f32 / h as f32;
    let scale = 0.78f32;
    let cache = world.sync_cache(cam.x, cam.z);
    let underwater = cache.get(cam.x.floor() as i32, cam.y.floor() as i32, cam.z.floor() as i32) == world::WATER;

    #[cfg(target_arch = "wasm32")]
    {
        for py in 0..h {
            let ny = 1.0 - ((py as f32 + 0.5) / h as f32) * 2.0;
            for px in 0..w {
                let nx = ((px as f32 + 0.5) / w as f32 * 2.0 - 1.0) * aspect * scale;
                let mut dx = cam.fx + cam.rx * nx + cam.ux * ny * scale;
                let mut dy = cam.fy + cam.ry * nx + cam.uy * ny * scale;
                let mut dz = cam.fz + cam.rz * nx + cam.uz * ny * scale;
                let inv_len = 1.0 / (dx * dx + dy * dy + dz * dz).sqrt();
                dx *= inv_len; dy *= inv_len; dz *= inv_len;
                let sky = sky_color(dim, dy, time, px as i32, py as i32);
                let (rgb, dist) = trace_ray(tex, cache, cam, dx, dy, dz, sky, day, tick, time, underwater, dim, mining);
                let pi = py * w + px;
                fb.depth[pi] = dist;
                let di = pi * 4;
                fb.rgba[di] = rgb[0];
                fb.rgba[di + 1] = rgb[1];
                fb.rgba[di + 2] = rgb[2];
                fb.rgba[di + 3] = 255;
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let nthreads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(1, 16);
        let rows_per = ((h + nthreads - 1) / nthreads).max(1);
        let rgba_chunks = fb.rgba.chunks_mut(rows_per * w * 4);
        let depth_chunks = fb.depth.chunks_mut(rows_per * w);

        std::thread::scope(|s| {
            for (band_idx, (rgba_band, depth_band)) in rgba_chunks.zip(depth_chunks).enumerate() {
                let y_start = band_idx * rows_per;
                let band_rows = depth_band.len() / w;
                s.spawn(move || {
                    for local_y in 0..band_rows {
                        let py = y_start + local_y;
                        let ny = 1.0 - ((py as f32 + 0.5) / h as f32) * 2.0;
                        for px in 0..w {
                            let nx = ((px as f32 + 0.5) / w as f32 * 2.0 - 1.0) * aspect * scale;
                            let mut dx = cam.fx + cam.rx * nx + cam.ux * ny * scale;
                            let mut dy = cam.fy + cam.ry * nx + cam.uy * ny * scale;
                            let mut dz = cam.fz + cam.rz * nx + cam.uz * ny * scale;
                            let inv_len = 1.0 / (dx * dx + dy * dy + dz * dz).sqrt();
                            dx *= inv_len; dy *= inv_len; dz *= inv_len;
                            let sky = sky_color(dim, dy, time, px as i32, py as i32);
                            let (rgb, dist) = trace_ray(tex, cache, cam, dx, dy, dz, sky, day, tick, time, underwater, dim, mining);
                            let pi = local_y * w + px;
                            depth_band[pi] = dist;
                            let di = pi * 4;
                            rgba_band[di] = rgb[0];
                            rgba_band[di + 1] = rgb[1];
                            rgba_band[di + 2] = rgb[2];
                            rgba_band[di + 3] = 255;
                        }
                    }
                });
            }
        });
    }
}

fn project(cam: &Cam, x: f32, y: f32, z: f32, fb: &Framebuffer) -> Option<(f32, f32, f32)> {
    let rx = x - cam.x; let ry = y - cam.y; let rz = z - cam.z;
    let d = rx * cam.fx + ry * cam.fy + rz * cam.fz;
    if d < 0.15 { return None; }
    let s = rx * cam.rx + ry * cam.ry + rz * cam.rz;
    let u = rx * cam.ux + ry * cam.uy + rz * cam.uz;
    let focal = fb.w as f32 / 1.56;
    Some((fb.w as f32 / 2.0 + s / d * focal, fb.h as f32 / 2.0 - u / d * focal, d))
}

/// Crisp wireframe highlight around the block the player is aiming at.
pub fn draw_block_outline(fb: &mut Framebuffer, cam: &Cam, bx: i32, by: i32, bz: i32) {
    let e = 0.006;
    let x0 = bx as f32 - e; let x1 = bx as f32 + 1.0 + e;
    let y0 = by as f32 - e; let y1 = by as f32 + 1.0 + e;
    let z0 = bz as f32 - e; let z1 = bz as f32 + 1.0 + e;
    let pts = [
        (x0, y0, z0), (x1, y0, z0), (x1, y0, z1), (x0, y0, z1),
        (x0, y1, z0), (x1, y1, z0), (x1, y1, z1), (x0, y1, z1),
    ];
    let edges = [
        (0, 1), (1, 2), (2, 3), (3, 0),
        (4, 5), (5, 6), (6, 7), (7, 4),
        (0, 4), (1, 5), (2, 6), (3, 7),
    ];
    for &(a, b) in &edges {
        let Some((ax, ay, ad)) = project(cam, pts[a].0, pts[a].1, pts[a].2, fb) else { continue };
        let Some((bx_p, by_p, bd)) = project(cam, pts[b].0, pts[b].1, pts[b].2, fb) else { continue };
        let steps = ((ax - bx_p).abs().max((ay - by_p).abs())).ceil().max(1.0).min(240.0) as i32;
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let px = (ax + (bx_p - ax) * t).round() as i32;
            let py = (ay + (by_p - ay) * t).round() as i32;
            let d = ad + (bd - ad) * t;
            if px >= 0 && py >= 0 && px < fb.w as i32 && py < fb.h as i32 {
                let td = fb.depth[py as usize * fb.w + px as usize];
                if d <= td + 0.18 {
                    fb.set(px, py, 20, 20, 24, 200);
                }
            }
        }
    }
}

// ===================== 3D Box-Model Entity Rasterizer =====================

pub enum EntityKind {
    Steve,
    Mob(MobKind),
    Boat,
    Minecart,
}

fn face_pixel(face: FaceArt, u: f32, v: f32, base: [u8; 3], alt: [u8; 3]) -> [u8; 3] {
    let x = (u * 8.0).clamp(0.0, 7.99) as i32;
    let y = (v * 8.0).clamp(0.0, 7.99) as i32;
    match face {
        FaceArt::Steve => {
            if y <= 1 || (y == 2 && (x == 0 || x == 7)) { return alt; } // hair
            if y == 4 && (x == 1 || x == 6) { return [242, 242, 246]; } // eye whites
            if y == 4 && (x == 2 || x == 5) { return [60, 88, 175]; } // blue iris
            if (y == 6 && x >= 2 && x <= 5) || (y == 7 && (x == 2 || x == 5)) { return [136, 86, 56]; } // beard/mouth
            base
        }
        FaceArt::Zombie => {
            if y == 4 && (x == 1 || x == 2 || x == 5 || x == 6) { return [22, 34, 22]; }
            if y == 6 && x >= 3 && x <= 4 { return [54, 88, 44]; }
            base
        }
        FaceArt::Skeleton => {
            if y == 4 && (x == 1 || x == 2 || x == 5 || x == 6) { return [24, 24, 28]; }
            if y == 5 && x >= 3 && x <= 4 { return [65, 65, 70]; }
            if y == 6 && x >= 2 && x <= 5 { return [40, 40, 44]; }
            base
        }
        FaceArt::Creeper => {
            let dk = [16, 32, 16];
            if y >= 2 && y <= 3 && ((x >= 1 && x <= 2) || (x >= 5 && x <= 6)) { return dk; }
            if y == 4 && x >= 3 && x <= 4 { return dk; }
            if y >= 5 && y <= 6 && x >= 2 && x <= 5 { return dk; }
            if y == 7 && (x == 2 || x == 5) { return dk; }
            if (x + y) % 3 == 0 { alt } else { base }
        }
        FaceArt::Spider => {
            if y >= 3 && y <= 4 && (x == 1 || x == 2 || x == 5 || x == 6) { return [215, 32, 32]; }
            if y == 6 && (x == 2 || x == 5) { return [145, 25, 25]; }
            base
        }
        FaceArt::Enderman => {
            if y == 4 && (x == 1 || x == 6) { return [225, 130, 255]; }
            if y == 4 && (x == 2 || x == 5) { return [248, 210, 255]; }
            base
        }
        FaceArt::Pig => {
            if y == 3 && (x == 1 || x == 6) { return [248, 248, 248]; }
            if y == 3 && (x == 2 || x == 5) { return [30, 30, 34]; }
            base
        }
        FaceArt::Cow => {
            if y <= 1 && (x == 0 || x == 7) { return [210, 206, 196]; } // horns
            if y == 3 && (x == 1 || x == 6) { return [24, 22, 24]; }
            if y >= 5 && x >= 2 && x <= 5 { return [192, 166, 152]; } // muzzle
            if x + y < 4 { alt } else { base }
        }
        FaceArt::Sheep => {
            if y <= 1 { return alt; }
            if y == 3 && (x == 1 || x == 6) { return [248, 248, 248]; }
            if y == 3 && (x == 2 || x == 5) { return [30, 30, 34]; }
            if y >= 5 && x >= 3 && x <= 4 { return [226, 164, 158]; }
            base
        }
        FaceArt::Chicken => {
            if y == 3 && (x == 1 || x == 6) { return [28, 28, 32]; }
            if y >= 4 && y <= 5 && x >= 2 && x <= 5 { return alt; }
            if y >= 6 && x >= 3 && x <= 4 { return [205, 42, 42]; }
            base
        }
        FaceArt::Villager => {
            if y <= 1 { return alt; }
            if y == 3 && x >= 1 && x <= 6 { return [72, 48, 30]; }
            if y == 4 && (x == 1 || x == 6) { return [245, 245, 245]; }
            if y == 4 && (x == 2 || x == 5) { return [52, 152, 72]; }
            base
        }
        FaceArt::ZombifiedPiglin => {
            if x >= 4 && y <= 4 { return alt; }
            if y == 4 && (x == 1 || x == 2 || x == 5 || x == 6) { return [30, 42, 26]; }
            base
        }
        FaceArt::Piglin => {
            if y <= 1 { return alt; }
            if y == 4 && (x == 1 || x == 6) { return [246, 246, 246]; }
            if y >= 5 && x >= 2 && x <= 5 { return [228, 128, 126]; }
            base
        }
        FaceArt::Ghast => {
            if y == 3 && (x == 1 || x == 2 || x == 5 || x == 6) { return [48, 48, 54]; }
            if y >= 5 && y <= 6 && x >= 3 && x <= 4 { return [158, 54, 60]; }
            base
        }
        FaceArt::Blaze => {
            if y == 4 && (x == 1 || x == 2 || x == 5 || x == 6) { return [255, 250, 175]; }
            if y <= 1 { return alt; }
            base
        }
        FaceArt::WitherSkeleton => {
            if y == 4 && (x == 1 || x == 2 || x == 5 || x == 6) { return [158, 158, 166]; }
            if y == 6 && x >= 2 && x <= 5 { return [20, 18, 22]; }
            base
        }
        FaceArt::Slime => {
            if y == 3 && (x == 1 || x == 2 || x == 5 || x == 6) { return [24, 52, 24]; }
            if y == 6 && x >= 3 && x <= 4 { return [24, 52, 24]; }
            if x == 0 || x == 7 || y == 0 || y == 7 { alt } else { base }
        }
        FaceArt::Wolf => {
            if y == 3 && (x == 1 || x == 6) { return [24, 24, 26]; }
            if y >= 5 && x >= 3 && x <= 4 { return [30, 26, 26]; }
            base
        }
        FaceArt::PolarBear => {
            if y == 3 && (x == 1 || x == 6) { return [22, 24, 28]; }
            if y >= 5 && x >= 3 && x <= 4 { return [32, 34, 40]; }
            base
        }
        FaceArt::Parrot => {
            if y == 3 && (x == 1 || x == 6) { return [24, 24, 28]; }
            if y >= 4 && y <= 5 && x >= 3 && x <= 4 { return [240, 206, 54]; }
            base
        }
        FaceArt::EnderDragon => {
            if y == 4 && (x == 1 || x == 2 || x == 5 || x == 6) { return alt; }
            base
        }
        FaceArt::None => base,
    }
}

fn transform_point(lx: f32, ly: f32, lz: f32, part: &Part3D, sy: f32, cy: f32, wx: f32, wy: f32, wz: f32) -> (f32, f32, f32) {
    // swing around (pivot_y, part.cz) in X axis
    let dy = ly - part.pivot_y;
    let dz = lz - part.cz;
    let (ss, cs) = (part.swing.sin(), part.swing.cos());
    let ry = part.pivot_y + dy * cs - dz * ss;
    let rz = part.cz + dy * ss + dz * cs;
    // rotate around Y by entity yaw
    let rx = lx * cy + rz * sy;
    let rz2 = -lx * sy + rz * cy;
    (wx + rx, wy + ry, wz + rz2)
}

fn raster_tri(
    fb: &mut Framebuffer,
    v0: (f32, f32, f32, f32, f32),
    v1: (f32, f32, f32, f32, f32),
    v2: (f32, f32, f32, f32, f32),
    part: &Part3D,
    is_front_face: bool,
    shade: f32,
    flash: u8,
    front_always: bool,
) {
    let min_x = v0.0.min(v1.0).min(v2.0).floor().max(0.0) as i32;
    let max_x = v0.0.max(v1.0).max(v2.0).ceil().min(fb.w as f32 - 1.0) as i32;
    let min_y = v0.1.min(v1.1).min(v2.1).floor().max(0.0) as i32;
    let max_y = v0.1.max(v1.1).max(v2.1).ceil().min(fb.h as f32 - 1.0) as i32;
    if min_x > max_x || min_y > max_y { return; }

    let area = (v1.0 - v0.0) * (v2.1 - v0.1) - (v2.0 - v0.0) * (v1.1 - v0.1);
    if area.abs() < 0.05 { return; }
    let inv_area = 1.0 / area;

    for py in min_y..=max_y {
        let cy = py as f32 + 0.5;
        for px in min_x..=max_x {
            let cx = px as f32 + 0.5;
            let w0 = ((v1.0 - cx) * (v2.1 - cy) - (v2.0 - cx) * (v1.1 - cy)) * inv_area;
            let w1 = ((v2.0 - cx) * (v0.1 - cy) - (v0.0 - cx) * (v2.1 - cy)) * inv_area;
            let w2 = 1.0 - w0 - w1;
            if w0 < -1e-4 || w1 < -1e-4 || w2 < -1e-4 { continue; }
            let inv_z = w0 * v0.2 + w1 * v1.2 + w2 * v2.2;
            if inv_z <= 0.001 { continue; }
            let z = 1.0 / inv_z;
            let idx = py as usize * fb.w + px as usize;
            if !front_always && z >= fb.depth[idx] { continue; }
            fb.depth[idx] = z;

            let c = if flash == 2 {
                [245, 245, 245]
            } else {
                let u = ((w0 * v0.3 * v0.2 + w1 * v1.3 * v1.2 + w2 * v2.3 * v2.2) * z).clamp(0.0, 1.0);
                let v = ((w0 * v0.4 * v0.2 + w1 * v1.4 * v1.2 + w2 * v2.4 * v2.2) * z).clamp(0.0, 1.0);
                let raw = if is_front_face && part.face != FaceArt::None {
                    face_pixel(part.face, u, v, part.col, part.col_alt)
                } else if part.face == FaceArt::Cow && ((u * 4.0) as i32 + (v * 4.0) as i32) % 3 == 0 {
                    part.col_alt
                } else {
                    part.col
                };
                let lit = [
                    (raw[0] as f32 * shade).min(255.0) as u8,
                    (raw[1] as f32 * shade).min(255.0) as u8,
                    (raw[2] as f32 * shade).min(255.0) as u8,
                ];
                if flash == 1 {
                    [
                        ((lit[0] as f32 * 0.42 + 235.0 * 0.58).min(255.0)) as u8,
                        (lit[1] as f32 * 0.30) as u8,
                        (lit[2] as f32 * 0.30) as u8,
                    ]
                } else {
                    lit
                }
            };
            fb.set(px, py, c[0], c[1], c[2], 255);
        }
    }
}

pub fn draw_entity_3d(
    fb: &mut Framebuffer,
    cam: &Cam,
    wx: f32,
    wy: f32,
    wz: f32,
    yaw: f32,
    walk: f32,
    day: f32,
    kind: EntityKind,
    flash: u8,
    front_always: bool,
) {
    let (_, _, dist) = match project(cam, wx, wy + 0.8, wz, fb) {
        Some(v) => v,
        None => return,
    };
    if dist > 72.0 { return; }
    if matches!(kind, EntityKind::Steve) && dist < 1.8 { return; }

    // ground shadow
    if let Some((gx, gy, gd)) = project(cam, wx, wy + 0.02, wz, fb) {
        let rw = (0.50 / gd * (fb.w as f32 / 1.56)).max(1.0);
        for yy in -rw as i32 / 3..=rw as i32 / 3 {
            let sy_i = gy as i32 + yy;
            if sy_i < 0 || sy_i >= fb.h as i32 { continue; }
            for xx in -rw as i32..=rw as i32 {
                let sx_i = gx as i32 + xx;
                if sx_i < 0 || sx_i >= fb.w as i32 { continue; }
                if xx * xx + yy * yy * 9 > (rw * rw) as i32 { continue; }
                let td = fb.depth[sy_i as usize * fb.w + sx_i as usize];
                if td >= 74.0 || gd > td + 0.8 { continue; }
                let a = ((1.0 - day * 0.45) * 42.0) as u8;
                fb.set(sx_i, sy_i, 10, 12, 16, a);
            }
        }
    }

    let parts = match kind {
        EntityKind::Steve => entities::steve_parts(walk),
        EntityKind::Mob(m) => entities::mob_parts(m, walk),
        EntityKind::Boat => entities::vehicle_parts(true),
        EntityKind::Minecart => entities::vehicle_parts(false),
    };
    let (sy, cy) = (yaw.sin(), yaw.cos());
    let focal = fb.w as f32 / 1.56;
    let hw = fb.w as f32 * 0.5;
    let hh = fb.h as f32 * 0.5;
    let light_base = 0.42 + day * 0.58;

    for part in &parts {
        let (x0, x1) = (part.cx - part.hx, part.cx + part.hx);
        let (y0, y1) = (part.cy - part.hy, part.cy + part.hy);
        let (z0, z1) = (part.cz - part.hz, part.cz + part.hz);
        let local = [
            (x0, y0, z0), (x1, y0, z0), (x1, y1, z0), (x0, y1, z0), // 0..3 back (-Z)
            (x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1), // 4..7 front (+Z)
        ];
        let mut world_p = [(0.0f32, 0.0f32, 0.0f32); 8];
        let mut proj = [(0.0f32, 0.0f32, 0.0f32); 8]; // sx, sy, inv_z
        let mut behind = false;
        for idx in 0..8 {
            let wp = transform_point(local[idx].0, local[idx].1, local[idx].2, part, sy, cy, wx, wy, wz);
            world_p[idx] = wp;
            let rx = wp.0 - cam.x; let ry = wp.1 - cam.y; let rz = wp.2 - cam.z;
            let d = rx * cam.fx + ry * cam.fy + rz * cam.fz;
            if d < 0.25 { behind = true; break; }
            let s = rx * cam.rx + ry * cam.ry + rz * cam.rz;
            let u = rx * cam.ux + ry * cam.uy + rz * cam.uz;
            let inv_z = 1.0 / d;
            proj[idx] = (hw + s * inv_z * focal, hh - u * inv_z * focal, inv_z);
        }
        if behind { continue; }

        // 6 faces: [bottom-left, bottom-right, top-right, top-left] with outward normals
        let faces: [([usize; 4], bool, f32); 6] = [
            ([4, 5, 6, 7], true, 1.00),  // +Z front (face art!)
            ([1, 0, 3, 2], false, 0.72), // -Z back
            ([5, 1, 2, 6], false, 0.84), // +X right
            ([0, 4, 7, 3], false, 0.84), // -X left
            ([7, 6, 2, 3], false, 1.06), // +Y top
            ([0, 1, 5, 4], false, 0.56), // -Y bottom
        ];
        for (q, is_front, fmul) in faces {
            let p0 = world_p[q[0]];
            let p1 = world_p[q[1]];
            let p2 = world_p[q[2]];
            let ux = p1.0 - p0.0; let uy = p1.1 - p0.1; let uz = p1.2 - p0.2;
            let vx = p2.0 - p0.0; let vy = p2.1 - p0.1; let vz = p2.2 - p0.2;
            let nx = uy * vz - uz * vy;
            let ny = uz * vx - ux * vz;
            let nz = ux * vy - uy * vx;
            let to_cam = (cam.x - p0.0) * nx + (cam.y - p0.1) * ny + (cam.z - p0.2) * nz;
            if to_cam <= 0.0 { continue; }

            let v0 = (proj[q[0]].0, proj[q[0]].1, proj[q[0]].2, 0.0, 1.0);
            let v1 = (proj[q[1]].0, proj[q[1]].1, proj[q[1]].2, 1.0, 1.0);
            let v2 = (proj[q[2]].0, proj[q[2]].1, proj[q[2]].2, 1.0, 0.0);
            let v3 = (proj[q[3]].0, proj[q[3]].1, proj[q[3]].2, 0.0, 0.0);
            let shade = light_base * fmul;
            raster_tri(fb, v0, v1, v2, part, is_front, shade, flash, front_always);
            raster_tri(fb, v0, v2, v3, part, is_front, shade, flash, front_always);
        }
    }
}

pub fn draw_drop_sprite(fb: &mut Framebuffer, cam: &Cam, x: f32, y: f32, z: f32, item: u16, tick: u32, day: f32) {
    let (sx, sy, dist) = match project(cam, x, y, z, fb) { Some(v) => v, None => return };
    if dist > 48.0 { return; }
    let size = (0.36 / dist * (fb.w as f32 / 1.56)).clamp(4.0, 28.0) as i32;
    let left = sx as i32 - size / 2;
    let top = sy as i32 - size;
    let light = 0.48 + day * 0.52;
    for py in 0..size {
        let iy = top + py;
        if iy < 0 || iy >= fb.h as i32 { continue; }
        for px in 0..size {
            let ix = left + px;
            if ix < 0 || ix >= fb.w as i32 { continue; }
            if dist >= fb.depth[iy as usize * fb.w + ix as usize] { continue; }
            let c = item_pixel(item, (px as u32 * 16 / size as u32).min(15), (py as u32 * 16 / size as u32).min(15), tick);
            if c[3] == 0 { continue; }
            fb.set(ix, iy, (c[0] as f32 * light) as u8, (c[1] as f32 * light) as u8, (c[2] as f32 * light) as u8, c[3]);
        }
    }
}

// ===================== Item & 3D Isometric Block Icons =====================

pub fn item_pixel(item: u16, x: u32, y: u32, tick: u32) -> [u8; 4] {
    if item == 0 { return [0, 0, 0, 0]; }
    let atlas = Textures::atlas();
    if item >= 256 && item <= 331 {
        return atlas.sample_item((item - 256) as usize, x, y);
    }
    if item >= 332 && item <= 345 {
        return atlas.sample_item((90 + (item - 332)) as usize, x, y);
    }
    if item >= 346 && item <= 366 {
        let idx = match item {
            crate::items::LAPIS_LAZULI => 104,
            crate::items::BONE_MEAL => 105,
            crate::items::WOOD_HOE => 106,
            crate::items::STONE_HOE => 107,
            crate::items::IRON_HOE => 108,
            crate::items::DIAMOND_HOE => 109,
            crate::items::BOAT => 110,
            crate::items::MINECART => 111,
            crate::items::GLASS_BOTTLE => 112,
            crate::items::POTION_HEAL => 113,
            crate::items::POTION_FIRE => 113,
            crate::items::POTION_SPEED => 113,
            crate::items::IRON_LEGGINGS => 114,
            crate::items::IRON_BOOTS => 115,
            crate::items::DIAMOND_LEGGINGS => 116,
            crate::items::DIAMOND_BOOTS => 117,
            crate::items::NETHERITE_HELMET => 118,
            crate::items::NETHERITE_CHESTPLATE => 119,
            crate::items::NETHERITE_LEGGINGS => 120,
            crate::items::NETHERITE_BOOTS => 121,
            crate::items::ENCHANTED_BOOK => 123,
            _ => 113,
        };
        let mut c = atlas.sample_item(idx, x, y);
        if c[3] > 20 {
            if item == crate::items::POTION_HEAL { c = [c[0].max(180), c[1] / 2, c[2] / 2, c[3]]; }
            else if item == crate::items::POTION_FIRE { c = [c[0].max(200), (c[1] as u16 * 3 / 4).min(255) as u8, c[2] / 3, c[3]]; }
            else if item == crate::items::POTION_SPEED { c = [c[0] / 2, c[1].max(165), c[2].max(210), c[3]]; }
        }
        return c;
    }
    let flat_idx = match item {
        world::TORCH => Some(76),
        world::REDSTONE_TORCH => Some(77),
        world::REDSTONE_DUST => Some(78),
        world::DOOR => Some(79),
        world::IRON_DOOR => Some(79),
        world::SIGN => Some(80),
        world::BED => Some(81),
        world::FLOWER_RED => Some(82),
        world::FLOWER_YELLOW => Some(83),
        world::TALL_GRASS => Some(84),
        world::RAIL => Some(85),
        world::WHEAT => Some(86),
        world::LEVER => Some(87),
        world::BROWN_MUSHROOM => Some(88),
        world::RED_MUSHROOM => Some(89),
        world::NETHER_WART => Some(122),
        world::BREWING_STAND => Some(124),
        world::HOPPER => Some(125),
        world::REPEATER => Some(126),
        world::COMPARATOR => Some(127),
        _ => None,
    };
    if let Some(idx) = flat_idx {
        let mut c = atlas.sample_item(idx, x, y);
        if item == world::IRON_DOOR && c[3] > 20 {
            let v = ((c[0] as u16 + c[1] as u16 + c[2] as u16) / 3).max(160).min(235) as u8;
            c = [v, v, v + 8, c[3]];
        }
        return c;
    }
    if item == world::PRESSURE_PLATE {
        if x >= 2 && x <= 13 && y >= 5 && y <= 11 { return [148, 148, 152, 255]; }
        return [0, 0, 0, 0];
    }
    if item == world::BUTTON {
        if x >= 4 && x <= 11 && y >= 6 && y <= 10 { return [140, 140, 144, 255]; }
        return [0, 0, 0, 0];
    }
    let k = Textures::face_kind(item, 0, 0, 1);
    atlas.sample_kind(k, x as f32 / 16.0, y as f32 / 16.0, tick, false)
}

fn is_flat_block_icon(item: u16) -> bool {
    matches!(
        item,
        world::TORCH | world::REDSTONE_TORCH | world::REDSTONE_DUST | world::LEVER
            | world::PRESSURE_PLATE | world::BUTTON | world::FLOWER_RED | world::FLOWER_YELLOW
            | world::TALL_GRASS | world::WHEAT | world::RAIL | world::SIGN | world::DOOR | world::IRON_DOOR
            | world::BED | world::BROWN_MUSHROOM | world::RED_MUSHROOM | world::SAPLING
            | world::NETHER_WART | world::BREWING_STAND | world::HOPPER | world::REPEATER | world::COMPARATOR
    )
}

/// Renders true textured isometric 3D cubes for blocks and real 16x16 pixel art for items!
pub fn draw_item_icon(fb: &mut Framebuffer, fb_x: i32, fb_y: i32, item: u16, size: i32, tick: u32) {
    if item == 0 || size <= 0 { return; }
    let atlas = Textures::atlas();
    if item < 256 && !is_flat_block_icon(item) && size >= 10 {
        let top_k = Textures::face_kind(item, 0, 1, 0);
        let left_k = Textures::face_kind(item, 0, 0, 1);
        let right_k = Textures::face_kind(item, 1, 0, 0);
        let anim = item == world::WATER || item == world::LAVA;
        let cx = size as f32 * 0.5;
        let cy = size as f32 * 0.5;
        let rx = size as f32 * 0.44;
        let ry = size as f32 * 0.23;
        let h = size as f32 * 0.44;
        for py in 0..size {
            for px in 0..size {
                let fx = px as f32 + 0.5 - cx;
                let fy = py as f32 + 0.5 - cy;
                // Isometric diamond top face centered at (0, -ry)
                let ty = fy + ry;
                if (fx / rx).abs() + (ty / ry).abs() <= 1.0 {
                    let u = ((fx / rx + ty / ry) * 0.5 + 0.5).clamp(0.0, 0.999);
                    let v = ((ty / ry - fx / rx) * 0.5 + 0.5).clamp(0.0, 0.999);
                    let c = atlas.sample_kind(top_k, u, v, tick, anim);
                    if c[3] > 20 {
                        let shade = 1.05;
                        fb.set(
                            fb_x + px,
                            fb_y + py,
                            (c[0] as f32 * shade).min(255.0) as u8,
                            (c[1] as f32 * shade).min(255.0) as u8,
                            (c[2] as f32 * shade).min(255.0) as u8,
                            c[3],
                        );
                    }
                    continue;
                }
                // Left side face (-rx..0)
                if fx >= -rx && fx < 0.0 {
                    let top_edge = (fx + rx) / rx * ry;
                    if fy >= top_edge && fy <= top_edge + h {
                        let u = ((fx + rx) / rx).clamp(0.0, 0.999);
                        let v = ((fy - top_edge) / h).clamp(0.0, 0.999);
                        let c = atlas.sample_kind(left_k, u, v, tick, anim);
                        if c[3] > 20 {
                            let shade = 0.84;
                            fb.set(
                                fb_x + px,
                                fb_y + py,
                                (c[0] as f32 * shade) as u8,
                                (c[1] as f32 * shade) as u8,
                                (c[2] as f32 * shade) as u8,
                                c[3],
                            );
                        }
                        continue;
                    }
                }
                // Right side face (0..rx)
                if fx >= 0.0 && fx <= rx {
                    let top_edge = (rx - fx) / rx * ry;
                    if fy >= top_edge && fy <= top_edge + h {
                        let u = (fx / rx).clamp(0.0, 0.999);
                        let v = ((fy - top_edge) / h).clamp(0.0, 0.999);
                        let c = atlas.sample_kind(right_k, u, v, tick, anim);
                        if c[3] > 20 {
                            let shade = 0.66;
                            fb.set(
                                fb_x + px,
                                fb_y + py,
                                (c[0] as f32 * shade) as u8,
                                (c[1] as f32 * shade) as u8,
                                (c[2] as f32 * shade) as u8,
                                c[3],
                            );
                        }
                    }
                }
            }
        }
        return;
    }

    for y in 0..size {
        for x in 0..size {
            let c = item_pixel(item, (x as u32 * 16 / size as u32).min(15), (y as u32 * 16 / size as u32).min(15), tick);
            if c[3] > 20 { fb.set(fb_x + x, fb_y + y, c[0], c[1], c[2], c[3]); }
        }
    }
}

// ===================== HUD & Menus =====================

fn heart(fb: &mut Framebuffer, x: i32, y: i32, state: u8) {
    let rows = [
        [0, 1, 1, 0, 0, 1, 1, 0],
        [1, 2, 2, 1, 1, 2, 2, 1],
        [1, 2, 2, 2, 2, 2, 2, 1],
        [1, 2, 2, 2, 2, 2, 2, 1],
        [0, 1, 2, 2, 2, 2, 1, 0],
        [0, 0, 1, 2, 2, 1, 0, 0],
        [0, 0, 0, 1, 1, 0, 0, 0],
    ];
    for (r, bits) in rows.iter().enumerate() {
        for (c, &v) in bits.iter().enumerate() {
            if v == 1 {
                fb.set(x + c as i32, y + r as i32, 20, 10, 12, 255);
            } else if v == 2 {
                let filled = state == 2 || (state == 1 && c < 4);
                if filled {
                    let hi = r == 1 && (c == 1 || c == 2);
                    if hi { fb.set(x + c as i32, y + r as i32, 255, 170, 170, 255); }
                    else { fb.set(x + c as i32, y + r as i32, 232, 28, 36, 255); }
                } else {
                    fb.set(x + c as i32, y + r as i32, 58, 42, 44, 255);
                }
            }
        }
    }
}

fn drumstick(fb: &mut Framebuffer, x: i32, y: i32, filled: bool) {
    fb.rect(x, y, 7, 7, [24, 16, 12, 255]);
    if filled {
        fb.rect(x + 1, y + 1, 4, 4, [196, 122, 62, 255]);
        fb.rect(x + 2, y + 2, 2, 2, [232, 168, 92, 255]);
        fb.rect(x + 4, y + 4, 2, 2, [238, 228, 208, 255]);
    } else {
        fb.rect(x + 1, y + 1, 5, 5, [52, 44, 38, 255]);
    }
}

pub const SLOT: i32 = 24;
pub const HOT_Y: i32 = 238;
pub fn hotbar_x(i: usize) -> i32 { W as i32 / 2 - (9 * (SLOT + 2)) / 2 + i as i32 * (SLOT + 2) }

pub fn draw_held_item(fb: &mut Framebuffer, item: u16, tick: u32) {
    let x = 412;
    let y = 192;
    if item == 0 {
        // Clean 3D-shaded first-person Steve hand
        fb.rect(x + 26, y + 36, 18, 42, [198, 152, 114, 255]);
        fb.rect(x + 26, y + 36, 14, 40, [218, 170, 130, 255]);
        fb.rect(x + 26, y + 36, 14, 4, [236, 192, 152, 255]);
        fb.rect(x + 23, y + 68, 24, 10, [52, 126, 148, 255]);
    } else {
        fb.rect(x + 22, y + 32, 16, 40, [212, 165, 125, 255]);
        fb.rect(x + 20, y + 64, 20, 8, [52, 126, 148, 255]);
        draw_item_icon(fb, x, y, item, 42, tick);
    }
}

pub fn draw_hud(
    fb: &mut Framebuffer,
    hot: &[u16; 9],
    counts: &[u32; 9],
    selected: usize,
    health: f32,
    hunger: f32,
    air: f32,
    armor: u32,
    offhand: u16,
    shield_blocking: bool,
    potion_fire: f32,
    potion_speed: f32,
    xp: u32,
    level: u32,
    burn: f32,
    tick: u32,
    message: &str,
    message_timer: f32,
    biome: Biome,
    dim: Dimension,
    dragon_hp: Option<f32>,
    pos: (f32, f32, f32),
    fps: u32,
    camera: u8,
    time: f32,
    mode: GameMode,
) {
    let hb_left = hotbar_x(0);
    if mode != GameMode::Spectator {
        if offhand != 0 {
            let ox = hb_left - SLOT - 8;
            fb.rect(ox - 2, HOT_Y - 2, SLOT + 4, SLOT + 4, [28, 28, 30, 210]);
            fb.frame(ox - 2, HOT_Y - 2, SLOT + 4, SLOT + 4, if shield_blocking { [220, 220, 255, 255] } else { [15, 15, 16, 255] });
            draw_item_icon(fb, ox + 3, HOT_Y + 3, offhand, SLOT - 6, tick);
        }
        fb.rect(hb_left - 2, HOT_Y - 2, 9 * (SLOT + 2) + 2, SLOT + 4, [28, 28, 30, 210]);
        fb.frame(hb_left - 2, HOT_Y - 2, 9 * (SLOT + 2) + 2, SLOT + 4, [15, 15, 16, 255]);
        for i in 0..9 {
            let x = hotbar_x(i);
            fb.rect(x, HOT_Y, SLOT, SLOT, [110, 110, 114, 170]);
            fb.frame(x, HOT_Y, SLOT, SLOT, [55, 55, 58, 255]);
            if i == selected {
                fb.frame(x - 2, HOT_Y - 2, SLOT + 4, SLOT + 4, [245, 245, 245, 255]);
                fb.frame(x - 1, HOT_Y - 1, SLOT + 2, SLOT + 2, [200, 200, 200, 255]);
            }
            if hot[i] != 0 { draw_item_icon(fb, x + 3, HOT_Y + 3, hot[i], SLOT - 6, tick); }
            if counts[i] > 1 {
                text_shadow(fb, x + SLOT - 11, HOT_Y + SLOT - 7, &counts[i].to_string(), [255, 255, 255, 255], 1);
            }
        }
        if mode == GameMode::Survival {
            if armor > 0 {
                for i in 0..10 {
                    let threshold = (i as u32 + 1) * 2;
                    let ax = hb_left + i as i32 * 9;
                    let ay = HOT_Y - 26;
                    if armor >= threshold {
                        fb.rect(ax, ay, 7, 7, [175, 185, 205, 255]);
                        fb.frame(ax, ay, 7, 7, [40, 44, 52, 255]);
                    } else if armor >= threshold - 1 {
                        fb.rect(ax, ay, 4, 7, [175, 185, 205, 255]);
                        fb.frame(ax, ay, 7, 7, [40, 44, 52, 255]);
                    }
                }
            }
            for i in 0..10 {
                let threshold = (i as f32 + 1.0) * 2.0;
                let st = if health >= threshold { 2 } else if health >= threshold - 1.0 { 1 } else { 0 };
                heart(fb, hb_left + i * 9, HOT_Y - 16, st);
            }
            for i in 0..10 {
                let filled = hunger >= (i as f32 + 1.0) * 2.0 - 0.5;
                drumstick(fb, hb_left + 9 * (SLOT + 2) - 8 - i * 8, HOT_Y - 16, filled);
            }
            if air < 9.95 {
                let bubbles = air.ceil().clamp(0.0, 10.0) as i32;
                for i in 0..bubbles {
                    let bx = hb_left + 9 * (SLOT + 2) - 8 - i * 8;
                    let by = HOT_Y - 25;
                    fb.rect(bx, by, 6, 6, [40, 140, 240, 220]);
                    fb.rect(bx + 1, by + 1, 2, 2, [215, 245, 255, 240]);
                }
            }
            let xp_bar_w = 9 * (SLOT + 2) - 2;
            let needed = (7 + level * 2).max(1);
            let xp_fill = ((xp as f32 / needed as f32).clamp(0.0, 1.0) * xp_bar_w as f32) as i32;
            fb.rect(hb_left, HOT_Y - 6, xp_bar_w, 3, [20, 20, 20, 220]);
            if xp_fill > 0 {
                fb.rect(hb_left, HOT_Y - 6, xp_fill, 3, [122, 232, 68, 255]);
            }
            if level > 0 {
                let ls = level.to_string();
                text_shadow(fb, W as i32 / 2 - (ls.len() as i32 * 3), HOT_Y - 14, &ls, [136, 246, 82, 255], 1);
            }
        }
        if shield_blocking {
            fb.rect(10, H as i32 - 58, 34, 48, [130, 96, 60, 220]);
            fb.frame(10, H as i32 - 58, 34, 48, [190, 196, 205, 255]);
            text_shadow(fb, 14, H as i32 - 38, "BLK", [255, 255, 255, 255], 1);
        }
        if potion_fire > 0.0 || potion_speed > 0.0 {
            let mut py = 32;
            if potion_fire > 0.0 {
                text_shadow(fb, 8, py, &format!("FIRE RES {:02}s", potion_fire as i32), [255, 165, 65, 255], 1);
                py += 10;
            }
            if potion_speed > 0.0 {
                text_shadow(fb, 8, py, &format!("SPEED {:02}s", potion_speed as i32), [110, 220, 255, 255], 1);
            }
        }
        if burn > 0.0 {
            let flicker = ((tick / 3) % 2) as i32 * 4;
            fb.rect(0, H as i32 - 22 - flicker, W as i32, 22 + flicker, [240, 96, 18, 85]);
        }
    }

    let cx = W as i32 / 2; let cy = H as i32 / 2;
    fb.rect(cx - 4, cy, 9, 1, [240, 240, 240, 200]);
    fb.rect(cx, cy - 4, 1, 9, [240, 240, 240, 200]);

    text_shadow(fb, 8, 8, &format!("{} ({})  XYZ {:.0} {:.0} {:.0}", biome.name(), dim.name(), pos.0, pos.1, pos.2), [240, 240, 240, 255], 1);
    text_shadow(fb, 8, 18, &format!("FPS {}  CAM {}  TIME {:02}:00  [{}]", fps, camera, ((time * 24.0) as i32) % 24, mode.name()), [200, 230, 255, 255], 1);

    if let Some(hp) = dragon_hp {
        let bw = 180;
        let bx = W as i32 / 2 - bw / 2;
        text_shadow(fb, W as i32 / 2 - 36, 28, "Ender Dragon", [228, 130, 248, 255], 1);
        fb.rect(bx, 37, bw, 5, [34, 18, 42, 230]);
        let fill = ((hp / 200.0).clamp(0.0, 1.0) * bw as f32) as i32;
        fb.rect(bx, 37, fill, 5, [198, 56, 224, 255]);
    }

    if message_timer > 0.0 {
        let w = message.len() as i32 * 6 + 10;
        let my = if mode == GameMode::Spectator { H as i32 - 36 } else { HOT_Y - 30 };
        fb.rect(W as i32 / 2 - w / 2, my, w, 12, [0, 0, 0, 175]);
        text_shadow(fb, W as i32 / 2 - w / 2 + 5, my + 3, message, [255, 255, 255, 255], 1);
    }
}

pub fn upscale(src: &Framebuffer, dst: &mut Framebuffer) {
    if src.w == dst.w && src.h == dst.h {
        dst.rgba.copy_from_slice(&src.rgba);
        dst.depth.copy_from_slice(&src.depth);
        return;
    }
    for y in 0..dst.h {
        let sy = (y * src.h / dst.h).min(src.h - 1);
        for x in 0..dst.w {
            let sx = (x * src.w / dst.w).min(src.w - 1);
            let si = (sy * src.w + sx) * 4;
            let di = (y * dst.w + x) * 4;
            dst.rgba[di..di + 4].copy_from_slice(&src.rgba[si..si + 4]);
            dst.depth[y * dst.w + x] = src.depth[sy * src.w + sx];
        }
    }
}

#[cfg(not(windows))]
pub fn encode_png(path: &std::path::Path, rgba: &[u8], w: usize, h: usize) -> std::io::Result<()> {
    fn crc32(data: &[u8]) -> u32 {
        let mut c = 0xffffffffu32;
        for &b in data {
            c ^= b as u32;
            for _ in 0..8 { c = if c & 1 != 0 { (c >> 1) ^ 0xedb88320 } else { c >> 1 }; }
        }
        !c
    }
    fn chunk(out: &mut Vec<u8>, typ: &[u8; 4], data: &[u8]) {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let mut d = Vec::with_capacity(4 + data.len());
        d.extend_from_slice(typ); d.extend_from_slice(data);
        out.extend_from_slice(&d);
        out.extend_from_slice(&crc32(&d).to_be_bytes());
    }
    let mut raw = Vec::with_capacity((w * 4 + 1) * h);
    for y in 0..h {
        raw.push(0);
        raw.extend_from_slice(&rgba[y * w * 4..(y + 1) * w * 4]);
    }
    let mut z = vec![0x78, 0x01];
    let mut off = 0;
    while off < raw.len() {
        let n = (raw.len() - off).min(65535);
        let final_block = (off + n == raw.len()) as u8;
        z.push(final_block);
        z.extend_from_slice(&(n as u16).to_le_bytes());
        z.extend_from_slice(&(!(n as u16)).to_le_bytes());
        z.extend_from_slice(&raw[off..off + n]);
        off += n;
    }
    let mut a = 1u32; let mut b = 0u32;
    for &byte in &raw { a = (a + byte as u32) % 65521; b = (b + a) % 65521; }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());
    let mut out = vec![137, 80, 78, 71, 13, 10, 26, 10];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(w as u32).to_be_bytes()); ihdr.extend_from_slice(&(h as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    std::fs::write(path, out)
}

pub const SLOT_PX: i32 = 24;

pub fn panel(fb: &mut Framebuffer, x: i32, y: i32, w: i32, h: i32) {
    fb.rect(x, y, w, h, [198, 198, 198, 255]);
    fb.rect(x, y, w, 2, [255, 255, 255, 255]);
    fb.rect(x, y, 2, h, [255, 255, 255, 255]);
    fb.rect(x + w - 2, y, 2, h, [85, 85, 85, 255]);
    fb.rect(x, y + h - 2, w, 2, [85, 85, 85, 255]);
    fb.frame(x, y, w, h, [25, 25, 25, 255]);
}

pub fn slot(fb: &mut Framebuffer, x: i32, y: i32, size: i32, hover: bool) {
    fb.rect(x, y, size, size, [139, 139, 139, 255]);
    fb.rect(x, y, size, 1, [55, 55, 55, 255]);
    fb.rect(x, y, 1, size, [55, 55, 55, 255]);
    fb.rect(x + size - 1, y, 1, size, [255, 255, 255, 255]);
    fb.rect(x, y + size - 1, size, 1, [255, 255, 255, 255]);
    if hover { fb.rect(x + 1, y + 1, size - 2, size - 2, [255, 255, 255, 75]); }
}

pub fn button(fb: &mut Framebuffer, x: i32, y: i32, w: i32, h: i32, label: &str, hover: bool) {
    let top = if hover { [120, 140, 186, 255] } else { [116, 116, 120, 255] };
    let bot = if hover { [86, 106, 152, 255] } else { [82, 82, 86, 255] };
    for r in 0..h {
        let t = r as f32 / h.max(1) as f32;
        let c = [
            (top[0] as f32 * (1.0 - t) + bot[0] as f32 * t) as u8,
            (top[1] as f32 * (1.0 - t) + bot[1] as f32 * t) as u8,
            (top[2] as f32 * (1.0 - t) + bot[2] as f32 * t) as u8,
            255,
        ];
        fb.rect(x, y + r, w, 1, c);
    }
    fb.rect(x + 1, y + 1, w - 2, 1, [160, 160, 165, 255]);
    let border = if hover { [255, 255, 255, 255] } else { [28, 28, 32, 255] };
    fb.frame(x, y, w, h, border);
    let tw = label.len() as i32 * 6;
    text_shadow(fb, x + (w - tw) / 2, y + (h - 5) / 2, label, [255, 255, 255, 255], 1);
}

pub fn draw_tooltip(fb: &mut Framebuffer, name: &str) {
    if name.is_empty() || name == "Air" { return; }
    let (mx, my) = fb.cursor;
    let w = name.len() as i32 * 6 + 10;
    let x = (mx + 10).min(W as i32 - w - 4);
    let y = (my - 14).max(4);
    fb.rect(x, y, w, 13, [16, 0, 16, 240]);
    fb.frame(x + 1, y + 1, w - 2, 11, [80, 0, 255, 220]);
    text_shadow(fb, x + 5, y + 4, name, [255, 255, 255, 255], 1);
}

pub fn creeper_face(fb: &mut Framebuffer, x: i32, y: i32, scale: i32) {
    let g1 = [66, 146, 56, 255];
    let g2 = [52, 120, 44, 255];
    let dk = [16, 36, 16, 255];
    fb.rect(x, y, 8 * scale, 8 * scale, g1);
    for r in 0..8 { for c in 0..8 { if (r + c) % 3 == 0 { fb.rect(x + c * scale, y + r * scale, scale, scale, g2); } } }
    fb.rect(x + 1 * scale, y + 2 * scale, 2 * scale, 2 * scale, dk);
    fb.rect(x + 5 * scale, y + 2 * scale, 2 * scale, 2 * scale, dk);
    fb.rect(x + 3 * scale, y + 4 * scale, 2 * scale, 1 * scale, dk);
    fb.rect(x + 2 * scale, y + 5 * scale, 4 * scale, 2 * scale, dk);
    fb.rect(x + 2 * scale, y + 7 * scale, 1 * scale, 1 * scale, dk);
    fb.rect(x + 5 * scale, y + 7 * scale, 1 * scale, 1 * scale, dk);
}

pub fn draw_title(fb: &mut Framebuffer, buttons: &[&str], hover: usize, splash: &str, t: f32) {
    let atlas = Textures::atlas();
    for y in 0..H as i32 {
        let f = y as f32 / H as f32;
        let r = (92.0 * (1.0 - f) + 178.0 * f) as u8;
        let g = (158.0 * (1.0 - f) + 216.0 * f) as u8;
        let b = (238.0 * (1.0 - f) + 252.0 * f) as u8;
        fb.rect(0, y, W as i32, 1, [r, g, b, 255]);
    }
    fb.rect(392, 24, 26, 26, [255, 250, 210, 255]);
    fb.rect(396, 28, 18, 18, [255, 255, 235, 255]);
    for i in 0..4 {
        let cx = ((t * 9.0 + i as f32 * 150.0) % 620.0) - 70.0;
        let cy = 36 + i * 20;
        fb.rect(cx as i32, cy, 74, 10, [245, 246, 250, 255]);
        fb.rect(cx as i32 + 12, cy - 7, 46, 8, [245, 246, 250, 255]);
    }
    // Textured voxel hills + real 16x16 Minecraft grass/dirt ground
    let scroll = (t * 8.0) as i32;
    for y in 198..H as i32 {
        for x in 0..W as i32 {
            let wx = (x + scroll) / 2;
            let ground_y = 204 + ((wx as f32 * 0.04).sin() * 6.0) as i32;
            if y < ground_y { continue; }
            let dy = (y - ground_y) / 2;
            let k = if dy < 16 { 2 } else { 3 };
            let c = atlas.sample_kind(k, (wx & 15) as f32 / 16.0, (dy & 15) as f32 / 16.0, 0, false);
            let shade = if dy < 16 { 0.96 } else { 0.80 };
            fb.set(x, y, (c[0] as f32 * shade) as u8, (c[1] as f32 * shade) as u8, (c[2] as f32 * shade) as u8, 255);
        }
    }

    let title = "MINECRAFT";
    let scale = 4;
    let tw = title.len() as i32 * 6 * scale;
    let tx = (W as i32 - tw) / 2;
    let bob = ((t * 1.6).sin() * 1.5) as i32;
    text(fb, tx + 3, 28 + bob + 3, title, [40, 40, 40, 220], scale);
    for (i, ch) in title.chars().enumerate() {
        let c = if i % 2 == 0 { [225, 225, 225, 255] } else { [245, 245, 245, 255] };
        text(fb, tx + i as i32 * 6 * scale, 28 + bob, &ch.to_string(), c, scale);
    }
    fb.rect(tx, 28 + bob + 8 * scale + 1, tw, 2, [70, 70, 70, 160]);

    let sw = splash.len() as i32 * 6;
    let sx = tx + tw - sw;
    let sy = 28 + bob + 8 * scale + 6 + ((t * 4.0).sin() * 2.0) as i32;
    text_shadow(fb, sx, sy, splash, [255, 255, 40, 255], 1);

    creeper_face(fb, W as i32 - 56, 98, 6);

    let bw = 200;
    let bx = (W as i32 - bw) / 2;
    for (i, label) in buttons.iter().enumerate() {
        let y = 116 + i as i32 * 26;
        button(fb, bx, y, bw, 21, label, hover == i);
    }
    text_shadow(fb, 8, H as i32 - 14, "Minecraft Rust Edition", [235, 235, 235, 230], 1);
}

// ===================== Survival Inventory =====================

pub fn craft_grid_idx(i: usize, grid_size: usize) -> usize {
    (i / grid_size) * 3 + (i % grid_size)
}

pub fn recipe_slot_rect(i: usize) -> (i32, i32, i32, i32) {
    (224 + (i % 3) as i32 * 24, 38 + (i / 3) as i32 * 24, 22, 22)
}

pub fn recipe_prev_rect() -> (i32, i32, i32, i32) { (344, 22, 16, 13) }
pub fn recipe_next_rect() -> (i32, i32, i32, i32) { (394, 22, 16, 13) }
pub fn recipe_fill_rect() -> (i32, i32, i32, i32) { (352, 82, 54, 16) }

pub fn recipe_fits_2x2(pat: &[u16; 9]) -> bool {
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (3usize, 3usize, 0usize, 0usize);
    let mut any = false;
    for y in 0..3 {
        for x in 0..3 {
            if pat[y * 3 + x] != 0 {
                any = true;
                min_x = min_x.min(x); min_y = min_y.min(y);
                max_x = max_x.max(x); max_y = max_y.max(y);
            }
        }
    }
    !any || (max_x - min_x + 1 <= 2 && max_y - min_y + 1 <= 2)
}

pub fn armor_slot_rect(i: usize) -> (i32, i32, i32, i32) {
    (22, 30 + i as i32 * 28, SLOT_PX, SLOT_PX)
}
pub fn offhand_slot_rect() -> (i32, i32, i32, i32) {
    (22, 152, SLOT_PX, SLOT_PX)
}

pub fn draw_inventory_screen(
    fb: &mut Framebuffer,
    hot: &[crate::craft::Stack; 9],
    store: &[crate::craft::Stack; 27],
    armor: &[crate::craft::Stack; 4],
    offhand: crate::craft::Stack,
    grid: &[crate::craft::Stack; 9],
    grid_size: usize,
    result: crate::craft::Stack,
    cursor: crate::craft::Stack,
    tick: u32,
    hover: Option<(usize, usize)>,
    recipe_page: usize,
    recipe_sel: usize,
) {
    fb.rect(0, 0, W as i32, H as i32, [0, 0, 0, 140]);
    panel(fb, 14, 14, 40, 174);
    text(fb, 18, 18, "GEAR", [60, 60, 60, 255], 1);
    let mut tip = String::new();
    for i in 0..4 {
        let (x, y, w, _h) = armor_slot_rect(i);
        let hv = hover == Some((4, i));
        slot(fb, x, y, w, hv);
        draw_item_icon(fb, x + 2, y + 2, armor[i].id, w - 4, tick);
        if hv && !armor[i].is_empty() { tip = crate::items::name(armor[i].id).to_string(); }
    }
    let (ox, oy, ow, _oh) = offhand_slot_rect();
    let ohv = hover == Some((4, 4));
    slot(fb, ox, oy, ow, ohv);
    draw_item_icon(fb, ox + 2, oy + 2, offhand.id, ow - 4, tick);
    if ohv && !offhand.is_empty() { tip = crate::items::name(offhand.id).to_string(); }

    let (px, py, pw, ph) = (60, 14, 360, 242);
    panel(fb, px, py, pw, ph);
    text(fb, 76, 20, if grid_size == 3 { "CRAFTING (3x3)" } else { "CRAFTING (2x2)" }, [60, 60, 60, 255], 1);
    for i in 0..grid_size * grid_size {
        let (x, y, w, h) = craft_slot_rect(i, grid_size);
        let hv = hover == Some((0, i));
        slot(fb, x, y, w, hv);
        let st = grid[craft_grid_idx(i, grid_size)];
        draw_item_icon(fb, x + 2, y + 2, st.id, w - 4, tick);
        if st.count > 1 { text_shadow(fb, x + w - 9, y + h - 8, &format!("{}", st.count), [255, 255, 255, 255], 1); }
        if hv && !st.is_empty() { tip = crate::items::name(st.id).to_string(); }
    }
    let (rx, ry, rw, rh) = craft_result_rect(grid_size);
    let rhv = hover == Some((0, 99));
    slot(fb, rx, ry, rw, rhv);
    draw_item_icon(fb, rx + 2, ry + 2, result.id, rw - 4, tick);
    if result.count > 1 { text_shadow(fb, rx + rw - 9, ry + rh - 8, &format!("{}", result.count), [255, 255, 255, 255], 1); }
    if rhv && !result.is_empty() { tip = crate::items::name(result.id).to_string(); }
    text(fb, rx - 18, ry + 9, "->", [60, 60, 60, 255], 1);

    fb.rect(218, 20, 194, 90, [168, 168, 172, 255]);
    fb.frame(218, 20, 194, 90, [95, 95, 100, 255]);
    text(fb, 224, 24, "RECIPES", [50, 50, 55, 255], 1);
    let recs = crate::craft::recipes();
    let per_page = 6usize;
    let pages = (recs.len() + per_page - 1) / per_page;
    let page = recipe_page.min(pages.saturating_sub(1));
    let (pvx, pvy, pvw, pvh) = recipe_prev_rect();
    let (nvx, nvy, nvw, nvh) = recipe_next_rect();
    button(fb, pvx, pvy, pvw, pvh, "<", hover == Some((3, 100)));
    text(fb, 363, 25, &format!("{}/{}", page + 1, pages), [50, 50, 55, 255], 1);
    button(fb, nvx, nvy, nvw, nvh, ">", hover == Some((3, 101)));

    let start = page * per_page;
    for i in 0..per_page {
        let idx = start + i;
        let (x, y, w, h) = recipe_slot_rect(i);
        let selected = idx == recipe_sel;
        let hv = hover == Some((3, i));
        slot(fb, x, y, w, hv || selected);
        if selected { fb.frame(x, y, w, h, [255, 220, 60, 255]); }
        if idx < recs.len() {
            draw_item_icon(fb, x + 2, y + 2, recs[idx].out, w - 4, tick);
            if recs[idx].count > 1 {
                text_shadow(fb, x + w - 8, y + h - 8, &format!("{}", recs[idx].count), [255, 255, 255, 255], 1);
            }
            if hv { tip = crate::items::name(recs[idx].out).to_string(); }
        }
    }

    if let Some(rec) = recs.get(recipe_sel) {
        for r in 0..3 {
            for c in 0..3 {
                let sx = 302 + c as i32 * 14;
                let sy = 38 + r as i32 * 14;
                slot(fb, sx, sy, 13, false);
                let id = rec.pattern[r * 3 + c];
                if id != 0 { draw_item_icon(fb, sx + 1, sy + 1, id, 11, tick); }
            }
        }
        let (fx, fy, fw, fh) = recipe_fill_rect();
        button(fb, fx, fy, fw, fh, "FILL", hover == Some((3, 102)));
        let out_name = crate::items::name(rec.out);
        let short = if out_name.len() > 21 { &out_name[..21] } else { out_name };
        text(fb, 224, 90, short, [40, 40, 45, 255], 1);
        if grid_size == 2 && !recipe_fits_2x2(&rec.pattern) {
            text(fb, 224, 100, "NEEDS CRAFTING TABLE", [175, 40, 40, 255], 1);
        }
    }

    text(fb, 76, 112, "INVENTORY", [60, 60, 60, 255], 1);
    for i in 0..27 {
        let (x, y, w, h) = inv_slot_rect(i);
        let hv = hover == Some((1, i));
        slot(fb, x, y, w, hv);
        draw_item_icon(fb, x + 2, y + 2, store[i].id, w - 4, tick);
        if store[i].count > 1 { text_shadow(fb, x + w - 9, y + h - 8, &format!("{}", store[i].count), [255, 255, 255, 255], 1); }
        if hv && !store[i].is_empty() { tip = crate::items::name(store[i].id).to_string(); }
    }
    for i in 0..9 {
        let (x, y, w, h) = hot_slot_rect(i);
        let hv = hover == Some((2, i));
        slot(fb, x, y, w, hv);
        draw_item_icon(fb, x + 2, y + 2, hot[i].id, w - 4, tick);
        if hot[i].count > 1 { text_shadow(fb, x + w - 9, y + h - 8, &format!("{}", hot[i].count), [255, 255, 255, 255], 1); }
        if hv && !hot[i].is_empty() { tip = crate::items::name(hot[i].id).to_string(); }
    }
    text(fb, 76, 240, "E / ESC CLOSE   -   CLICK RECIPE TO VIEW / FILL", [60, 60, 60, 255], 1);
    draw_cursor_stack(fb, cursor, tick);
    if cursor.is_empty() && !tip.is_empty() { draw_tooltip(fb, &tip); }
}

// ===================== Minecraft Creative Inventory =====================

pub const CREATIVE_PANEL: (i32, i32, i32, i32) = (64, 30, 352, 206);

pub fn creative_tab_rect(tab: usize) -> (i32, i32, i32, i32) {
    let (px, py, pw, ph) = CREATIVE_PANEL;
    if tab < 5 {
        (px + 2 + tab as i32 * 36, py - 24, 34, 24)
    } else if tab == 5 {
        // Search tab (right-aligned top, just like Minecraft!)
        (px + pw - 36, py - 24, 34, 24)
    } else if tab < 11 {
        (px + 2 + (tab - 6) as i32 * 36, py + ph, 34, 24)
    } else {
        // Survival Inventory tab (right-aligned bottom)
        (px + pw - 36, py + ph, 34, 24)
    }
}

pub fn creative_grid_rect(i: usize) -> (i32, i32, i32, i32) {
    let (px, py, _, _) = CREATIVE_PANEL;
    (px + 14 + (i % 9) as i32 * 26, py + 26 + (i / 9) as i32 * 26, 24, 24)
}

pub fn creative_hot_rect(i: usize) -> (i32, i32, i32, i32) {
    let (px, py, _, ph) = CREATIVE_PANEL;
    (px + 14 + i as i32 * 26, py + ph - 32, 24, 24)
}

pub fn creative_trash_rect() -> (i32, i32, i32, i32) {
    let (px, py, pw, ph) = CREATIVE_PANEL;
    (px + pw - 42, py + ph - 32, 24, 24)
}

pub fn creative_scroll_up_rect() -> (i32, i32, i32, i32) {
    let (px, py, pw, _) = CREATIVE_PANEL;
    (px + pw - 42, py + 26, 24, 18)
}

pub fn creative_scroll_dn_rect() -> (i32, i32, i32, i32) {
    let (px, py, pw, _) = CREATIVE_PANEL;
    (px + pw - 42, py + 136, 24, 18)
}

pub fn creative_tab_info(tab: usize) -> (&'static str, u16) {
    match tab {
        0 => ("Building Blocks", world::BRICK),
        1 => ("Colored Blocks", world::WOOL_BLUE),
        2 => ("Natural Blocks", world::GRASS),
        3 => ("Functional Blocks", world::CRAFTING),
        4 => ("Redstone Blocks", world::REDSTONE_DUST),
        5 => ("Search Items", crate::items::COMPASS),
        6 => ("Tools & Utilities", crate::items::DIAMOND_PICK),
        7 => ("Combat", crate::items::IRON_SWORD),
        8 => ("Food & Drinks", crate::items::GOLDEN_APPLE),
        9 => ("Ingredients", crate::items::IRON_INGOT),
        10 => ("Spawn Eggs", crate::items::SPAWN_CREEPER),
        _ => ("Survival Inventory", world::CHEST),
    }
}

pub fn draw_creative_screen(
    fb: &mut Framebuffer,
    hot: &[crate::craft::Stack; 9],
    cursor: crate::craft::Stack,
    tab: usize,
    scroll: usize,
    search: &str,
    items_list: &[u16],
    tick: u32,
    hover: Option<(usize, usize)>,
) {
    fb.rect(0, 0, W as i32, H as i32, [0, 0, 0, 140]);
    let (px, py, pw, ph) = CREATIVE_PANEL;

    // Inactive tabs behind panel
    for t in 0..12 {
        if t == tab { continue; }
        let (tx, ty, tw, th_r) = creative_tab_rect(t);
        let hv = hover == Some((0, t));
        let bg = if hv { [186, 186, 190, 255] } else { [146, 146, 150, 255] };
        fb.rect(tx, ty, tw, th_r, bg);
        fb.frame(tx, ty, tw, th_r, [32, 32, 34, 255]);
        let (_, icon) = creative_tab_info(t);
        draw_item_icon(fb, tx + 9, ty + 4, icon, 16, tick);
    }

    // Main panel
    panel(fb, px, py, pw, ph);

    // Active tab on top seamlessly merged with panel
    let (atx, aty, atw, ath) = creative_tab_rect(tab);
    fb.rect(atx, aty, atw, ath + 2, [198, 198, 198, 255]);
    fb.rect(atx, aty, atw, 2, [255, 255, 255, 255]);
    fb.rect(atx, aty, 2, ath, [255, 255, 255, 255]);
    fb.rect(atx + atw - 2, aty, 2, ath, [85, 85, 85, 255]);
    fb.frame(atx, aty, atw, ath, [25, 25, 25, 255]);
    if tab < 6 { fb.rect(atx + 2, py, atw - 4, 3, [198, 198, 198, 255]); }
    else { fb.rect(atx + 2, py + ph - 3, atw - 4, 4, [198, 198, 198, 255]); }
    let (tab_title, tab_icon) = creative_tab_info(tab);
    draw_item_icon(fb, atx + 9, aty + 4, tab_icon, 16, tick);

    // Header / Search bar
    text(fb, px + 14, py + 10, tab_title, [55, 55, 60, 255], 1);
    if tab == 5 || !search.is_empty() {
        fb.rect(px + 136, py + 7, 150, 14, [42, 42, 46, 255]);
        fb.frame(px + 136, py + 7, 150, 14, [110, 110, 118, 255]);
        let caret = if (tick / 15) % 2 == 0 { "_" } else { "" };
        text(fb, px + 140, py + 11, &format!("{}{}", search, caret), [240, 240, 240, 255], 1);
    }

    let mut tip = String::new();
    if let Some((0, t)) = hover {
        tip = creative_tab_info(t).0.to_string();
    }

    // 9x5 Creative Item Grid
    let start = scroll * 9;
    for i in 0..45 {
        let (sx, sy, sw, _) = creative_grid_rect(i);
        let hv = hover == Some((1, i));
        slot(fb, sx, sy, sw, hv);
        if start + i < items_list.len() {
            let id = items_list[start + i];
            draw_item_icon(fb, sx + 3, sy + 3, id, 18, tick);
            if hv { tip = crate::items::name(id).to_string(); }
        }
    }

    // Scrollbar + Up/Down buttons
    let (ux, uy, uw, uh) = creative_scroll_up_rect();
    let (dx, dy, dw, dh) = creative_scroll_dn_rect();
    button(fb, ux, uy, uw, uh, "^", hover == Some((3, 0)));
    button(fb, dx, dy, dw, dh, "v", hover == Some((3, 1)));
    fb.rect(ux + 5, uy + uh + 2, 14, dy - (uy + uy - 28 + uh), [110, 110, 114, 255]);
    let track_y = uy + uh + 2;
    let track_h = dy - track_y - 2;
    fb.rect(ux + 5, track_y, 14, track_h, [112, 112, 116, 255]);
    fb.frame(ux + 5, track_y, 14, track_h, [55, 55, 58, 255]);
    let total_rows = ((items_list.len() + 8) / 9).max(5);
    let max_scroll = total_rows.saturating_sub(5);
    let thumb_h = (track_h * 5 / total_rows as i32).max(14);
    let thumb_y = if max_scroll == 0 {
        track_y + 1
    } else {
        track_y + 1 + ((track_h - thumb_h - 2) * scroll as i32 / max_scroll as i32)
    };
    fb.rect(ux + 6, thumb_y, 12, thumb_h, [198, 198, 202, 255]);
    fb.frame(ux + 6, thumb_y, 12, thumb_h, [80, 80, 85, 255]);

    // Hotbar
    for i in 0..9 {
        let (hx, hy, hw, hh) = creative_hot_rect(i);
        let hv = hover == Some((2, i));
        slot(fb, hx, hy, hw, hv);
        draw_item_icon(fb, hx + 3, hy + 3, hot[i].id, 18, tick);
        if hot[i].count > 1 {
            text_shadow(fb, hx + hw - 9, hy + hh - 8, &format!("{}", hot[i].count), [255, 255, 255, 255], 1);
        }
        if hv && !hot[i].is_empty() { tip = crate::items::name(hot[i].id).to_string(); }
    }

    // Trash / Delete Item slot
    let (tx, ty, tw, _th_r) = creative_trash_rect();
    let thv = hover == Some((4, 0));
    slot(fb, tx, ty, tw, thv);
    text(fb, tx + 9, ty + 9, "X", [180, 45, 45, 255], 1);
    if thv { tip = String::from("Destroy Item"); }

    draw_cursor_stack(fb, cursor, tick);
    if cursor.is_empty() && !tip.is_empty() { draw_tooltip(fb, &tip); }
}

pub fn draw_cursor_stack(fb: &mut Framebuffer, cursor: crate::craft::Stack, tick: u32) {
    if !cursor.is_empty() {
        let (mx, my) = fb.cursor;
        draw_item_icon(fb, mx - 11, my - 11, cursor.id, 22, tick);
        if cursor.count > 1 { text_shadow(fb, mx + 2, my + 6, &format!("{}", cursor.count), [255, 255, 255, 255], 1); }
    }
}

pub fn craft_slot_rect(i: usize, grid_size: usize) -> (i32, i32, i32, i32) {
    let gx = 76;
    let gy = 30;
    let (col, row) = (i % grid_size, i / grid_size);
    (gx + col as i32 * (SLOT_PX + 2), gy + row as i32 * (SLOT_PX + 2), SLOT_PX, SLOT_PX)
}
pub fn craft_result_rect(grid_size: usize) -> (i32, i32, i32, i32) {
    let (x, y, _, _) = craft_slot_rect(0, grid_size);
    (x + (grid_size as i32) * (SLOT_PX + 2) + 24, y + (grid_size as i32 - 1) * 10 + 4, SLOT_PX + 4, SLOT_PX + 4)
}
pub fn inv_slot_rect(i: usize) -> (i32, i32, i32, i32) {
    (76 + (i % 9) as i32 * (SLOT_PX + 2), 124 + (i / 9) as i32 * (SLOT_PX + 2), SLOT_PX, SLOT_PX)
}
pub fn hot_slot_rect(i: usize) -> (i32, i32, i32, i32) {
    (76 + i as i32 * (SLOT_PX + 2), 208, SLOT_PX, SLOT_PX)
}

pub fn furnace_slot_rect(i: usize) -> (i32, i32, i32, i32) {
    match i {
        0 => (104, 40, SLOT_PX, SLOT_PX),
        1 => (104, 96, SLOT_PX, SLOT_PX),
        _ => (196, 66, SLOT_PX + 4, SLOT_PX + 4),
    }
}

pub fn draw_furnace_screen(
    fb: &mut Framebuffer,
    input: crate::craft::Stack,
    fuel: crate::craft::Stack,
    output: crate::craft::Stack,
    burn: f32,
    cook: f32,
    hot: &[crate::craft::Stack; 9],
    cursor: crate::craft::Stack,
    tick: u32,
    hover: Option<(usize, usize)>,
) {
    fb.rect(0, 0, W as i32, H as i32, [0, 0, 0, 140]);
    let (px, py, pw, ph) = (60, 16, 360, 238);
    panel(fb, px, py, pw, ph);
    text(fb, px + 10, py + 8, "FURNACE", [60, 60, 60, 255], 1);
    for i in 0..3 {
        let (x, y, w, h) = furnace_slot_rect(i);
        slot(fb, x, y, w, hover == Some((0, i)));
        let st = [input, fuel, output][i];
        draw_item_icon(fb, x + 2, y + 2, st.id, w - 4, tick);
        if st.count > 1 { text_shadow(fb, x + w - 9, y + h - 8, &format!("{}", st.count), [255, 255, 255, 255], 1); }
    }
    let flame = (burn * 24.0) as i32;
    for i in 0..24 {
        let c = if i < flame { [255, 170, 40, 255] } else { [80, 80, 84, 255] };
        fb.rect(142, 122 - i, 12, 1, c);
    }
    text(fb, 136, 128, "FUEL", [60, 60, 60, 255], 1);
    let prog = (cook * 40.0) as i32;
    fb.rect(150, 48, 40, 8, [120, 120, 120, 255]);
    fb.rect(150, 48, prog, 8, [235, 235, 235, 255]);
    text(fb, 150, 38, "SMELTING", [60, 60, 60, 255], 1);

    text(fb, px + 10, py + 178, "INVENTORY", [60, 60, 60, 255], 1);
    for i in 0..9 {
        let (x, y, w, h) = hot_slot_rect(i);
        slot(fb, x, y, w, hover == Some((1, i)));
        draw_item_icon(fb, x + 2, y + 2, hot[i].id, w - 4, tick);
        if hot[i].count > 1 { text_shadow(fb, x + w - 9, y + h - 8, &format!("{}", hot[i].count), [255, 255, 255, 255], 1); }
    }
    text(fb, px + 10, py + ph - 12, "E / ESC CLOSE", [60, 60, 60, 255], 1);
    draw_cursor_stack(fb, cursor, tick);
}

pub fn chest_slot_rect(i: usize) -> (i32, i32, i32, i32) {
    (76 + (i % 9) as i32 * (SLOT_PX + 2), 38 + (i / 9) as i32 * (SLOT_PX + 2), SLOT_PX, SLOT_PX)
}
pub fn chest_inv_slot_rect(i: usize) -> (i32, i32, i32, i32) {
    (76 + (i % 9) as i32 * (SLOT_PX + 2), 128 + (i / 9) as i32 * (SLOT_PX + 2), SLOT_PX, SLOT_PX)
}

pub fn draw_chest_screen(
    fb: &mut Framebuffer,
    chest: &[crate::craft::Stack; 27],
    store: &[crate::craft::Stack; 27],
    hot: &[crate::craft::Stack; 9],
    cursor: crate::craft::Stack,
    tick: u32,
    hover: Option<(usize, usize)>,
) {
    fb.rect(0, 0, W as i32, H as i32, [0, 0, 0, 140]);
    let (px, py, pw, ph) = (64, 16, 352, 238);
    panel(fb, px, py, pw, ph);
    text(fb, 76, 26, "CHEST", [60, 60, 60, 255], 1);
    let mut tip = String::new();
    for i in 0..27 {
        let (x, y, w, h) = (76 + (i % 9) as i32 * (SLOT_PX + 2), 38 + (i / 9) as i32 * (SLOT_PX + 2), SLOT_PX, SLOT_PX);
        let hv = hover == Some((0, i));
        slot(fb, x, y, w, hv);
        draw_item_icon(fb, x + 2, y + 2, chest[i].id, w - 4, tick);
        if chest[i].count > 1 { text_shadow(fb, x + w - 9, y + h - 8, &format!("{}", chest[i].count), [255, 255, 255, 255], 1); }
        if hv && !chest[i].is_empty() { tip = crate::items::name(chest[i].id).to_string(); }
    }
    text(fb, 76, 118, "INVENTORY", [60, 60, 60, 255], 1);
    for i in 0..27 {
        let (x, y, w, h) = (76 + (i % 9) as i32 * (SLOT_PX + 2), 128 + (i / 9) as i32 * (SLOT_PX + 2), SLOT_PX, SLOT_PX);
        let hv = hover == Some((1, i));
        slot(fb, x, y, w, hv);
        draw_item_icon(fb, x + 2, y + 2, store[i].id, w - 4, tick);
        if store[i].count > 1 { text_shadow(fb, x + w - 9, y + h - 8, &format!("{}", store[i].count), [255, 255, 255, 255], 1); }
        if hv && !store[i].is_empty() { tip = crate::items::name(store[i].id).to_string(); }
    }
    for i in 0..9 {
        let (x, y, w, h) = hot_slot_rect(i);
        let hv = hover == Some((2, i));
        slot(fb, x, y, w, hv);
        draw_item_icon(fb, x + 2, y + 2, hot[i].id, w - 4, tick);
        if hot[i].count > 1 { text_shadow(fb, x + w - 9, y + h - 8, &format!("{}", hot[i].count), [255, 255, 255, 255], 1); }
        if hv && !hot[i].is_empty() { tip = crate::items::name(hot[i].id).to_string(); }
    }
    draw_cursor_stack(fb, cursor, tick);
    if cursor.is_empty() && !tip.is_empty() { draw_tooltip(fb, &tip); }
}

pub fn draw_chat(fb: &mut Framebuffer, lines: &[(String, f32)], input: Option<&str>, t: f32) {
    let mut y = 96;
    for (line, age) in lines.iter().rev().take(8).rev() {
        let fade = if *age > 8.0 { 0.35 } else { 1.0 };
        if input.is_some() || *age < 12.0 {
            let a = (200.0 * fade) as u8;
            fb.rect(6, y - 2, (line.len() as i32 * 6 + 8).min(W as i32 - 12), 11, [0, 0, 0, a]);
            text_shadow(fb, 10, y, line, [255, 255, 255, 255], 1);
        }
        y += 12;
    }
    if let Some(text_in) = input {
        fb.rect(6, H as i32 - 40, W as i32 - 12, 12, [0, 0, 0, 175]);
        let caret = if (t * 2.0) as i32 % 2 == 0 { "_" } else { " " };
        text_shadow(fb, 10, H as i32 - 37, &format!("> {}{}", text_in, caret), [255, 255, 255, 255], 1);
    }
}

pub fn draw_pause(fb: &mut Framebuffer, hover: usize, mode: GameMode, sound: bool, music: bool) {
    fb.rect(0, 0, W as i32, H as i32, [15, 20, 35, 165]);
    let names = [
        String::from("BACK TO GAME"),
        format!("GAME MODE: {}", mode.name()),
        format!("SOUND: {}  MUSIC: {}", if sound { "ON" } else { "OFF" }, if music { "ON" } else { "OFF" }),
        String::from("OPTIONS"),
        String::from("SAVE AND QUIT"),
    ];
    text_shadow(fb, (W as i32 - 108) / 2, 36, "GAME MENU", [255, 255, 255, 255], 2);
    let bw = 210; let bx = (W as i32 - bw) / 2;
    for (i, n) in names.iter().enumerate() { button(fb, bx, 76 + i as i32 * 26, bw, 20, n, hover == i); }
}

pub fn draw_multiplayer_screen(fb: &mut Framebuffer, addr: &str, status: &str, hover: usize, hosting: bool) {
    fb.rect(0, 0, W as i32, H as i32, [0, 0, 0, 145]);
    let (px, py, pw, ph) = (50, 20, 380, 230);
    panel(fb, px, py, pw, ph);
    text(fb, px + 10, py + 8, "PLAY MULTIPLAYER", [60, 60, 60, 255], 1);
    text(fb, px + 10, py + 34, "SERVER ADDRESS", [60, 60, 60, 255], 1);
    fb.rect(px + 10, py + 46, pw - 20, 18, [40, 40, 40, 255]);
    text(fb, px + 14, py + 51, &format!("{}_", addr), [235, 235, 235, 255], 1);
    text(fb, px + 10, py + 74, status, [70, 70, 110, 255], 1);
    button(fb, px + 10, py + 96, 170, 22, "JOIN SERVER", hover == 0);
    button(fb, px + 200, py + 96, 170, 22, if hosting { "HOSTING..." } else { "OPEN TO LAN" }, hover == 1);
    button(fb, px + 10, py + 128, 170, 22, "DIRECT CONNECT", hover == 2);
    button(fb, px + 200, py + 128, 170, 22, "BACK", hover == 3);
    text(fb, px + 10, py + 168, "Host a game, then type your friend's IP on their", [60, 60, 60, 255], 1);
    text(fb, px + 10, py + 180, "machine and press JOIN SERVER. Port 25565.", [60, 60, 60, 255], 1);
    text(fb, px + 10, py + 200, "Chat with T. Players and block edits sync live.", [60, 60, 60, 255], 1);
}

pub fn draw_options_screen(fb: &mut Framebuffer, sound: bool, music: bool, scale: i32, camera: u8, hover: usize) {
    fb.rect(0, 0, W as i32, H as i32, [0, 0, 0, 145]);
    let (px, py, pw, ph) = (70, 22, 340, 226);
    panel(fb, px, py, pw, ph);
    text(fb, px + 10, py + 8, "OPTIONS", [60, 60, 60, 255], 1);
    let cam_name = match camera { 1 => "THIRD PERSON", 2 => "DISTANT", _ => "FIRST PERSON" };
    let items = [
        format!("SOUND: {}", if sound { "ON" } else { "OFF" }),
        format!("MUSIC: {}", if music { "ON" } else { "OFF" }),
        format!("PIXEL SCALE: {}x", scale),
        format!("CAMERA (F5/C): {}", cam_name),
        "DONE".to_string(),
    ];
    for (i, label) in items.iter().enumerate() {
        button(fb, px + 60, py + 38 + i as i32 * 28, pw - 120, 22, label, hover == i);
    }
    text(fb, px + 10, py + ph - 16, "Click options to cycle values", [90, 90, 90, 255], 1);
}

pub fn draw_enchant_screen(
    fb: &mut Framebuffer,
    item: crate::craft::Stack,
    lapis: crate::craft::Stack,
    level: u32,
    hot: &[crate::craft::Stack; 9],
    cursor: crate::craft::Stack,
    tick: u32,
    hover: Option<(usize, usize)>,
) {
    fb.rect(0, 0, W as i32, H as i32, [0, 0, 0, 145]);
    let (px, py, pw, ph) = (64, 16, 352, 238);
    panel(fb, px, py, pw, ph);
    text(fb, px + 10, py + 8, &format!("ENCHANTING TABLE  (LVL {})", level), [60, 60, 60, 255], 1);
    slot(fb, 94, 48, SLOT_PX, hover == Some((0, 0)));
    draw_item_icon(fb, 96, 50, item.id, SLOT_PX - 4, tick);
    slot(fb, 126, 48, SLOT_PX, hover == Some((0, 1)));
    draw_item_icon(fb, 128, 50, lapis.id, SLOT_PX - 4, tick);
    if lapis.count > 1 { text_shadow(fb, 141, 64, &format!("{}", lapis.count), [255, 255, 255, 255], 1); }

    let opts = [
        ("I: SHARPNESS / EFFICIENCY I", 1u32),
        ("II: PROTECTION / FORTUNE II", 3u32),
        ("III: UNBREAKING / POWER III", 5u32),
    ];
    for (i, (label, req)) in opts.iter().enumerate() {
        let bx = 170; let by = 36 + i as i32 * 26;
        let can = !item.is_empty() && !lapis.is_empty() && level >= *req;
        button(fb, bx, by, 224, 22, &format!("{} ({} LVL)", label, req), can && hover == Some((0, 10 + i)));
    }
    text(fb, 76, 178, "HOTBAR", [60, 60, 60, 255], 1);
    for i in 0..9 {
        let (x, y, w, h) = hot_slot_rect(i);
        slot(fb, x, y, w, hover == Some((2, i)));
        draw_item_icon(fb, x + 2, y + 2, hot[i].id, w - 4, tick);
        if hot[i].count > 1 { text_shadow(fb, x + w - 9, y + h - 8, &format!("{}", hot[i].count), [255, 255, 255, 255], 1); }
    }
    draw_cursor_stack(fb, cursor, tick);
}

pub fn draw_anvil_screen(
    fb: &mut Framebuffer,
    left: crate::craft::Stack,
    right: crate::craft::Stack,
    output: crate::craft::Stack,
    level: u32,
    hot: &[crate::craft::Stack; 9],
    cursor: crate::craft::Stack,
    tick: u32,
    hover: Option<(usize, usize)>,
) {
    fb.rect(0, 0, W as i32, H as i32, [0, 0, 0, 145]);
    let (px, py, pw, ph) = (64, 16, 352, 238);
    panel(fb, px, py, pw, ph);
    text(fb, px + 10, py + 8, &format!("REPAIR & NAME  (COST: 2 LVL, YOU: {})", level), [60, 60, 60, 255], 1);
    slot(fb, 104, 62, SLOT_PX, hover == Some((0, 0)));
    draw_item_icon(fb, 106, 64, left.id, SLOT_PX - 4, tick);
    text(fb, 138, 70, "+", [60, 60, 60, 255], 2);
    slot(fb, 160, 62, SLOT_PX, hover == Some((0, 1)));
    draw_item_icon(fb, 162, 64, right.id, SLOT_PX - 4, tick);
    text(fb, 194, 70, "->", [60, 60, 60, 255], 2);
    slot(fb, 224, 60, SLOT_PX + 4, hover == Some((0, 2)));
    draw_item_icon(fb, 226, 62, output.id, SLOT_PX, tick);
    for i in 0..9 {
        let (x, y, w, h) = hot_slot_rect(i);
        slot(fb, x, y, w, hover == Some((2, i)));
        draw_item_icon(fb, x + 2, y + 2, hot[i].id, w - 4, tick);
        if hot[i].count > 1 { text_shadow(fb, x + w - 9, y + h - 8, &format!("{}", hot[i].count), [255, 255, 255, 255], 1); }
    }
    draw_cursor_stack(fb, cursor, tick);
}

pub fn draw_brewing_screen(
    fb: &mut Framebuffer,
    ing: crate::craft::Stack,
    bottle: crate::craft::Stack,
    output: crate::craft::Stack,
    prog: f32,
    hot: &[crate::craft::Stack; 9],
    cursor: crate::craft::Stack,
    tick: u32,
    hover: Option<(usize, usize)>,
) {
    fb.rect(0, 0, W as i32, H as i32, [0, 0, 0, 145]);
    let (px, py, pw, ph) = (64, 16, 352, 238);
    panel(fb, px, py, pw, ph);
    text(fb, px + 10, py + 8, "BREWING STAND  (WART / BLAZE / SUGAR)", [60, 60, 60, 255], 1);
    slot(fb, 180, 36, SLOT_PX, hover == Some((0, 0)));
    draw_item_icon(fb, 182, 38, ing.id, SLOT_PX - 4, tick);
    slot(fb, 140, 86, SLOT_PX, hover == Some((0, 1)));
    draw_item_icon(fb, 142, 88, bottle.id, SLOT_PX - 4, tick);
    slot(fb, 220, 86, SLOT_PX, hover == Some((0, 2)));
    draw_item_icon(fb, 222, 88, output.id, SLOT_PX - 4, tick);
    fb.rect(188, 66, 8, (prog * 18.0) as i32, [240, 180, 60, 255]);
    text(fb, 84, 126, "RECIPES: GLISTER/APPLE->HEAL, MAGMA/BLAZE->FIRE RES", [65, 65, 70, 255], 1);
    text(fb, 84, 138, "         WART/REDSTONE->SPEED", [65, 65, 70, 255], 1);
    for i in 0..9 {
        let (x, y, w, h) = hot_slot_rect(i);
        slot(fb, x, y, w, hover == Some((2, i)));
        draw_item_icon(fb, x + 2, y + 2, hot[i].id, w - 4, tick);
        if hot[i].count > 1 { text_shadow(fb, x + w - 9, y + h - 8, &format!("{}", hot[i].count), [255, 255, 255, 255], 1); }
    }
    draw_cursor_stack(fb, cursor, tick);
}

pub fn draw_trade_screen(
    fb: &mut Framebuffer,
    trades: &[(u16, u32, u16, u32)],
    hot: &[crate::craft::Stack; 9],
    tick: u32,
    hover: Option<(usize, usize)>,
) {
    fb.rect(0, 0, W as i32, H as i32, [0, 0, 0, 145]);
    let (px, py, pw, ph) = (64, 16, 352, 238);
    panel(fb, px, py, pw, ph);
    text(fb, px + 10, py + 8, "VILLAGER TRADING  (CLICK TO TRADE)", [60, 60, 60, 255], 1);
    for (i, &(cost_id, cost_n, out_id, out_n)) in trades.iter().enumerate() {
        let by = 30 + i as i32 * 30;
        let hv = hover == Some((0, i));
        button(fb, 82, by, 316, 26, "", hv);
        draw_item_icon(fb, 90, by + 3, cost_id, 20, tick);
        text_shadow(fb, 114, by + 9, &format!("x{} {}", cost_n, crate::items::name(cost_id)), [255, 255, 255, 255], 1);
        text_shadow(fb, 224, by + 9, "->", [255, 235, 120, 255], 1);
        draw_item_icon(fb, 244, by + 3, out_id, 20, tick);
        text_shadow(fb, 268, by + 9, &format!("x{} {}", out_n, crate::items::name(out_id)), [170, 255, 170, 255], 1);
    }
    for i in 0..9 {
        let (x, y, w, h) = hot_slot_rect(i);
        slot(fb, x, y, w, false);
        draw_item_icon(fb, x + 2, y + 2, hot[i].id, w - 4, tick);
        if hot[i].count > 1 { text_shadow(fb, x + w - 9, y + h - 8, &format!("{}", hot[i].count), [255, 255, 255, 255], 1); }
    }
}

pub fn draw_death_screen(fb: &mut Framebuffer, level: u32, hover: usize) {
    fb.rect(0, 0, W as i32, H as i32, [130, 16, 16, 175]);
    text_shadow(fb, W as i32 / 2 - 66, 62, "YOU DIED!", [255, 235, 235, 255], 2);
    text_shadow(fb, W as i32 / 2 - 54, 92, &format!("SCORE / LEVEL: {}", level), [255, 235, 120, 255], 1);
    let bx = (W as i32 - 200) / 2;
    button(fb, bx, 128, 200, 22, "RESPAWN", hover == 0);
    button(fb, bx, 158, 200, 22, "TITLE SCREEN", hover == 1);
}
