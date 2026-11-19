// Software renderer: pixel textures, raycast voxel lighting, sprite billboards and HUD.
use crate::entities::{self, Mob, MobKind, Drop};
use crate::world::{self, World, Biome, Hit};
use std::f32::consts::PI;

pub const W: usize = 480;
pub const H: usize = 270;
pub const RW: usize = 320;
pub const RH: usize = 180;
const TEX: usize = 32;

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
        // up = right x forward
        let ux = -rz * fy;
        let uy = rz * fx - rx * fz;
        let uz = rx * fy;
        Self { x, y, z, fx, fy, fz, rx, ry: 0.0, rz, ux, uy, uz }
    }
}

fn th(x: i32, y: i32, k: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374761393) ^ (y as u32).wrapping_mul(668265263) ^ (k as u32).wrapping_mul(2246822519);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    (h ^ (h >> 16)) as u32
}

fn clamp255(v: f32) -> u8 { v.max(0.0).min(255.0) as u8 }

pub struct Textures { data: Vec<Vec<u8>> }

impl Textures {
    pub fn new() -> Self {
        let mut data = Vec::with_capacity(65);
        for k in 0..65 { data.push(Self::generate(k)); }
        Self { data }
    }

    pub fn kind(block: u16) -> usize {
        match block {
            world::AIR => 0,
            world::STONE => 0, world::GRASS => 2, world::DIRT => 3, world::COBBLE => 4,
            world::LOG => 5, world::LEAVES => 7, world::SAND => 8, world::SANDSTONE => 43,
            world::WATER => 9, world::SNOW => 10, world::ICE => 11, world::SPRUCE_LOG => 47,
            world::SPRUCE_LEAVES => 7, world::JUNGLE_LOG => 5, world::JUNGLE_LEAVES => 7,
            world::PLANKS => 12, world::GLASS => 13, world::TORCH => 14, world::BEDROCK => 15,
            world::COAL_ORE => 16, world::IRON_ORE => 17, world::GOLD_ORE => 18,
            world::DIAMOND_ORE => 19, world::REDSTONE_ORE => 20, world::GRAVEL => 22,
            world::CLAY => 23, world::CACTUS => 24, world::TALL_GRASS => 25,
            world::FLOWER_RED => 26, world::FLOWER_YELLOW => 27, world::BRICK => 28,
            world::STONE_BRICKS => 29, world::WOOL_WHITE => 30, world::WOOL_RED => 31,
            world::WOOL_BLUE => 32, world::WOOL_GREEN => 33, world::WOOL_YELLOW => 34,
            world::BLACKSTONE => 35, world::MAGMA => 36, world::LAVA => 37,
            world::FARMLAND => 38, world::WHEAT => 39, world::CHEST => 40,
            world::CRAFTING => 41, world::FURNACE => 42, world::BIRCH_LOG => 5,
            world::BIRCH_LEAVES => 7, world::BROWN_MUSHROOM => 26, world::RED_MUSHROOM => 27,
            world::PUMPKIN => 44, world::HAY => 45, world::TNT => 52, world::OBSIDIAN => 35,
            world::GLOWSTONE => 46, world::SEA_LANTERN => 46, world::DARK_LOG => 47,
            world::DARK_LEAVES => 7, world::MOSSY => 4, world::COBWEB => 48, world::VINE => 25,
            world::DOOR => 12, world::IRON_BLOCK => 64, world::GOLD_BLOCK => 18,
            world::DIAMOND_BLOCK => 19, world::EMERALD_ORE => 21, world::EMERALD_BLOCK => 21,
            world::SNOW_LAYER => 10, world::PODZOL => 63, world::MYCELIUM => 64,
            world::RED_SAND => 8, world::TERRACOTTA => 23,
            world::CONCRETE_WHITE => 49, world::CONCRETE_BLUE => 50, world::CONCRETE_RED => 51,
            world::CONCRETE_GREEN => 52, world::CONCRETE_YELLOW => 53, world::CONCRETE_BLACK => 54,
            world::LANTERN => 46, world::CAMPFIRE => 14, world::BOOKSHELF => 55, world::BED => 31,
            world::CAKE => 44, world::LADDER => 12, world::RAIL => 56, world::ANVIL => 57,
            world::ENCHANTING => 58, world::END_STONE => 59, world::NETHERRACK => 60,
            world::SOUL_SAND => 61, world::QUARTZ => 62, world::SLAB => 0, world::STAIRS => 4,
            world::FENCE => 12, world::SIGN => 12, _ => 0,
        }
    }

    fn generate(k: usize) -> Vec<u8> {
        let mut out = vec![0u8; TEX * TEX * 4];
        for y in 0..TEX {
            for x in 0..TEX {
                let i = (y * TEX + x) * 4;
                let r = th(x as i32, y as i32, k as i32) % 255;
                let n = (th(x as i32 / 3, y as i32 / 3, k as i32 + 7) % 40) as f32;
                let noise = ((r as f32 * 0.35 + n * 0.65) - 45.0) * 0.55;
                let (cr, cg, cb, mut a) = match k {
                    0 => (135.0 + noise, 135.0 + noise, 138.0 + noise, 255.0), // stone
                    1 => (95.0 + noise, 150.0 + noise, 70.0 + noise, 255.0), // grass top
                    2 => {
                        let edge = (y as f32 / TEX as f32) < 0.28 + ((th(x as i32, 0, k as i32) % 5) as f32) * 0.02;
                        if edge && r % 7 != 0 { (94.0 + noise, 152.0 + noise, 68.0 + noise, 255.0) }
                        else { (120.0 + noise, 88.0 + noise, 62.0 + noise, 255.0) }
                    }
                    3 => (122.0 + noise, 88.0 + noise, 62.0 + noise, 255.0),
                    4 => {
                        let crack = x % 11 == 0 || (y % 10 == 0 && x % 7 < 3);
                        if crack { (88.0 + noise, 88.0 + noise, 90.0 + noise, 255.0) }
                        else { (142.0 + noise, 142.0 + noise, 144.0 + noise, 255.0) }
                    }
                    5 | 47 => {
                        let bark = if k == 47 { (88.0, 72.0, 55.0) } else { (108.0, 80.0, 52.0) };
                        let line = (x + (th(y as i32, 0, k as i32) % 3) as usize) % 7;
                        if line == 0 { (bark.0 * 0.7, bark.1 * 0.7, bark.2 * 0.7, 255.0) }
                        else { (bark.0 + noise, bark.1 + noise, bark.2 + noise, 255.0) }
                    }
                    6 => {
                        let rings = (((x as i32 - 16).pow(2) + (y as i32 - 16).pow(2)) as f32).sqrt();
                        if rings % 5.0 < 1.0 { (120.0, 88.0, 55.0, 255.0) }
                        else { (165.0 + noise, 128.0 + noise, 80.0 + noise, 255.0) }
                    }
                    7 => {
                        let hole = r % 17 == 0;
                        if hole { (0.0, 0.0, 0.0, 0.0) }
                        else { (60.0 + noise * 1.4, 112.0 + noise, 54.0 + noise, 255.0) }
                    }
                    8 => (222.0 + noise * 0.5, 208.0 + noise * 0.5, 165.0 + noise * 0.5, 255.0),
                    9 => {
                        let wave = ((y as f32 + (x as f32 * 0.4).sin() * 3.0) % 9.0) < 2.0;
                        if wave { (65.0, 125.0, 205.0, 175.0) } else { (45.0, 100.0, 185.0, 165.0) }
                    }
                    10 => (238.0 + noise * 0.3, 244.0 + noise * 0.3, 248.0, 255.0),
                    11 => (155.0 + noise * 0.6, 205.0 + noise * 0.4, 235.0 + noise * 0.2, 190.0),
                    12 => {
                        let seam = y % 8 == 0 || (x + (y / 8) * 8) % 16 == 0;
                        if seam { (135.0, 96.0, 58.0, 255.0) } else { (176.0 + noise, 132.0 + noise, 82.0 + noise, 255.0) }
                    }
                    13 => {
                        let border = x < 2 || y < 2 || x > 29 || y > 29;
                        if border || (x == 8 && y == 8) || (x == 23 && y == 21) { (190.0, 215.0, 225.0, 190.0) }
                        else { (205.0, 235.0, 245.0, 55.0) }
                    }
                    14 => {
                        if x >= 14 && x <= 17 && y >= 8 && y <= 31 { (90.0, 62.0, 38.0, 255.0) }
                        else if x >= 13 && x <= 18 && y >= 4 && y <= 10 { (245.0, 165.0, 35.0, 255.0) }
                        else { (0.0, 0.0, 0.0, 0.0) }
                    }
                    15 => (45.0 + noise * 0.4, 45.0 + noise * 0.4, 48.0 + noise * 0.4, 255.0),
                    16 | 17 | 18 | 19 | 20 | 21 => {
                        let base = 135.0 + noise;
                        let spot = (x as i32 - 12).pow(2) + (y as i32 - 14).pow(2) < 42
                            || (x as i32 - 23).pow(2) + (y as i32 - 22).pow(2) < 25;
                        if spot {
                            match k {
                                16 => (40.0, 40.0, 45.0, 255.0),
                                17 => (215.0, 190.0, 150.0, 255.0),
                                18 => (245.0, 205.0, 75.0, 255.0),
                                19 => (75.0, 225.0, 235.0, 255.0),
                                20 => (220.0, 45.0, 45.0, 255.0),
                                _ => (65.0, 200.0, 105.0, 255.0),
                            }
                        } else { (base, base, base + 2.0, 255.0) }
                    }
                    22 => (165.0 + noise, 160.0 + noise, 150.0 + noise, 255.0),
                    23 => (170.0 + noise, 160.0 + noise, 145.0 + noise, 255.0),
                    24 => {
                        let spike = x % 8 == 0 || y % 8 == 0;
                        if spike { (55.0, 110.0, 45.0, 255.0) } else { (70.0 + noise, 135.0 + noise, 55.0 + noise, 255.0) }
                    }
                    25 => {
                        if (x + y) % 5 == 0 { (65.0, 125.0, 50.0, 255.0) } else { (0.0, 0.0, 0.0, 0.0) }
                    }
                    26 => {
                        if x >= 12 && x <= 19 && y >= 14 && y <= 21 { (215.0, 55.0, 60.0, 255.0) }
                        else if x >= 14 && x <= 17 && y >= 22 { (70.0, 135.0, 60.0, 255.0) }
                        else { (0.0, 0.0, 0.0, 0.0) }
                    }
                    27 => {
                        if x >= 12 && x <= 19 && y >= 14 && y <= 21 { (235.0, 205.0, 60.0, 255.0) }
                        else if x >= 14 && x <= 17 && y >= 22 { (70.0, 135.0, 60.0, 255.0) }
                        else { (0.0, 0.0, 0.0, 0.0) }
                    }
                    28 => {
                        if x % 16 < 2 || y % 8 < 2 { (165.0, 160.0, 155.0, 255.0) }
                        else { (150.0 + noise, 75.0 + noise, 60.0 + noise, 255.0) }
                    }
                    29 => {
                        if x % 16 < 2 || y % 16 < 2 { (95.0, 95.0, 98.0, 255.0) }
                        else { (125.0 + noise, 125.0 + noise, 128.0 + noise, 255.0) }
                    }
                    30 => (238.0 + noise * 0.4, 238.0 + noise * 0.4, 235.0 + noise * 0.4, 255.0),
                    31 => (190.0 + noise * 0.5, 55.0 + noise * 0.3, 50.0 + noise * 0.3, 255.0),
                    32 => (55.0 + noise * 0.4, 90.0 + noise * 0.4, 190.0 + noise * 0.3, 255.0),
                    33 => (55.0 + noise * 0.4, 135.0 + noise * 0.5, 60.0 + noise * 0.4, 255.0),
                    34 => (225.0 + noise * 0.3, 195.0 + noise * 0.4, 60.0 + noise * 0.3, 255.0),
                    35 => (45.0 + noise * 0.4, 44.0 + noise * 0.4, 48.0 + noise * 0.4, 255.0),
                    36 | 37 => {
                        let v = ((th(x as i32 / 2, y as i32 / 2, 37) % 100) as f32) / 100.0;
                        if k == 36 { (180.0 + v * 60.0, 60.0 + v * 50.0, 20.0, 255.0) }
                        else { (230.0, 90.0 + v * 110.0, 25.0, 255.0) }
                    }
                    38 => {
                        if x % 8 == 0 { (95.0, 65.0, 40.0, 255.0) } else { (115.0 + noise, 80.0 + noise, 50.0 + noise, 255.0) }
                    }
                    39 => {
                        if x == 15 || y % 7 == 0 { (200.0, 180.0, 70.0, 255.0) } else { (0.0, 0.0, 0.0, 0.0) }
                    }
                    40 => {
                        let border = x < 3 || x > 28 || y < 4 || y > 27;
                        if border { (95.0, 62.0, 35.0, 255.0) }
                        else if y > 12 && x >= 13 && x <= 18 { (220.0, 185.0, 80.0, 255.0) }
                        else { (145.0 + noise, 95.0 + noise, 55.0 + noise, 255.0) }
                    }
                    41 => {
                        if x % 10 < 2 || y % 10 < 2 { (130.0, 92.0, 55.0, 255.0) }
                        else { (175.0 + noise, 130.0 + noise, 80.0 + noise, 255.0) }
                    }
                    42 => {
                        if x >= 9 && x <= 22 && y >= 10 && y <= 23 { (65.0, 65.0, 68.0, 255.0) }
                        else if x >= 11 && x <= 20 && y >= 13 && y <= 20 { (110.0, 110.0, 115.0, 255.0) }
                        else { (125.0 + noise, 125.0 + noise, 125.0 + noise, 255.0) }
                    }
                    43 => (215.0 + noise * 0.5, 195.0 + noise * 0.5, 150.0 + noise * 0.5, 255.0),
                    44 => {
                        if x > 4 && x < 28 && y > 4 && y < 28 { (225.0 + noise, 120.0 + noise, 45.0, 255.0) }
                        else { (70.0, 125.0, 50.0, 255.0) }
                    }
                    45 => {
                        if y % 6 == 0 { (190.0, 155.0, 75.0, 255.0) } else { (220.0 + noise, 190.0 + noise, 100.0 + noise, 255.0) }
                    }
                    46 => (245.0 + noise * 0.2, 235.0 + noise * 0.2, 180.0 + noise * 0.3, 255.0),
                    48 => { let v = ((x * 2 + y) % 9 == 0) as u8 as f32 * 220.0; (v, v, v, 255.0) },
                    49 => (235.0 + noise * 0.2, 235.0 + noise * 0.2, 235.0 + noise * 0.2, 255.0),
                    50 => (60.0 + noise * 0.4, 100.0 + noise * 0.4, 190.0 + noise * 0.3, 255.0),
                    51 => (200.0 + noise * 0.4, 55.0 + noise * 0.3, 55.0 + noise * 0.3, 255.0),
                    52 => (55.0 + noise * 0.4, 140.0 + noise * 0.4, 65.0 + noise * 0.3, 255.0),
                    53 => (225.0 + noise * 0.3, 195.0 + noise * 0.4, 55.0 + noise * 0.3, 255.0),
                    54 => (45.0 + noise * 0.3, 45.0 + noise * 0.3, 50.0 + noise * 0.3, 255.0),
                    55 => {
                        if y % 8 < 2 { (110.0, 70.0, 40.0, 255.0) }
                        else { ((th(x as i32, y as i32 / 8, 55) % 3) as f32 * 70.0 + 80.0, 80.0, 60.0, 255.0) }
                    }
                    56 => { let v = ((x % 8 == 3 || x % 8 == 4) && y > 8 && y < 24) as u8 as f32 * 170.0; (v, v, v, 255.0) },
                    57 => (85.0 + noise * 0.4, 85.0 + noise * 0.4, 90.0 + noise * 0.4, 255.0),
                    58 => (95.0 + noise * 0.4, 60.0 + noise * 0.3, 135.0 + noise * 0.4, 255.0),
                    59 => (215.0 + noise * 0.5, 215.0 + noise * 0.5, 200.0 + noise * 0.5, 255.0),
                    60 => (110.0 + noise * 0.5, 45.0 + noise * 0.4, 45.0 + noise * 0.4, 255.0),
                    61 => (100.0 + noise * 0.5, 85.0 + noise * 0.4, 60.0 + noise * 0.4, 255.0),
                    62 => (230.0 + noise * 0.3, 228.0 + noise * 0.3, 220.0 + noise * 0.3, 255.0),
                    63 => (95.0 + noise * 0.5, 75.0 + noise * 0.4, 50.0 + noise * 0.4, 255.0),
                    64 => (135.0 + noise * 0.6, 120.0 + noise * 0.5, 145.0 + noise * 0.5, 255.0),
                    _ => (128.0, 128.0, 128.0, 255.0),
                };
                out[i] = clamp255(cr);
                out[i + 1] = clamp255(cg);
                out[i + 2] = clamp255(cb);
                out[i + 3] = clamp255(a);
            }
        }
        out
    }

    pub fn sample(&self, block: u16, u: f32, v: f32, tick: u32) -> [u8; 4] {
        let k = Self::kind(block);
        let mut x = ((u.fract().abs() * TEX as f32) as usize) % TEX;
        let mut y = ((v.fract().abs() * TEX as f32) as usize) % TEX;
        if block == world::WATER || block == world::LAVA { y = (y + (tick / 4) as usize) % TEX; }
        let d = &self.data[k];
        let i = (y * TEX + x) * 4;
        x += 0;
        [d[i], d[i + 1], d[i + 2], d[i + 3]]
    }

    pub fn grass_top(&self, _u: f32, _v: f32) -> [u8; 4] { self.sample(world::GRASS, _u, _v, 0) }
}

pub struct Framebuffer {
    pub w: usize,
    pub h: usize,
    pub rgba: Vec<u8>,
    pub depth: Vec<f32>,
}

impl Framebuffer {
    pub fn new(w: usize, h: usize) -> Self {
        Self { w, h, rgba: vec![0; w * h * 4], depth: vec![f32::INFINITY; w * h] }
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
        '%' => [17, 2, 4, 8, 17], '#' => [10, 31, 10, 31, 10], _ => [0, 0, 0, 0, 0],
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

fn day_factor(time: f32) -> f32 {
    let sun = (time * 2.0 * PI).sin();
    (sun * 2.2 + 0.35).clamp(0.0, 1.0)
}

fn sky_color(dir_y: f32, time: f32, x: i32, y: i32) -> [u8; 3] {
    let sun = (time * 2.0 * PI).sin();
    let day = day_factor(time);
    let horizon = 0.22 + sun.abs() * 0.1;
    let t = ((dir_y * 0.5 + 0.5) * (1.0 - horizon) + horizon).clamp(0.0, 1.0);
    let night_top = [8.0, 14.0, 38.0];
    let night_bot = [24.0, 26.0, 58.0];
    let day_top = [75.0, 145.0, 225.0];
    let day_bot = [165.0, 205.0, 240.0];
    let dusk = [235.0, 135.0, 90.0];
    let dusk_amt = (1.0 - sun.abs() * 3.0).clamp(0.0, 1.0) * 0.75;
    let mut c = [0.0; 3];
    for i in 0..3 {
        let mut v = night_bot[i] + (night_top[i] - night_bot[i]) * t;
        let dv = day_bot[i] + (day_top[i] - day_bot[i]) * t;
        v = v + (dv - v) * day;
        v = v * (1.0 - dusk_amt) + dusk[i] * dusk_amt * (1.0 - dir_y.abs().min(1.0)) * 0.7;
        c[i] = v;
    }
    // stars at night
    if day < 0.2 && dir_y > 0.02 && th(x, y, 91) % 900 == 0 {
        c = [235.0, 235.0, 220.0];
    }
    [c[0] as u8, c[1] as u8, c[2] as u8]
}

fn cloud_color(cam: &Cam, dx: f32, dy: f32, dz: f32, time: f32, day: f32) -> Option<[u8; 3]> {
    if dy <= 0.03 { return None; }
    let t = (110.0 - cam.y) / dy;
    if t <= 0.0 || t > 260.0 { return None; }
    let wx = cam.x + dx * t;
    let wz = cam.z + dz * t;
    let n = world::noise2(wx / 42.0 + time * 0.02, wz / 42.0 - time * 0.015);
    let n2 = world::noise2(wx / 90.0 - 12.0, wz / 90.0 + 30.0);
    if n + n2 * 0.45 > 0.22 {
        let v = 180.0 + day * 65.0;
        Some([v as u8, v as u8, (v + 5.0) as u8])
    } else { None }
}

fn face_shade(nx: i32, ny: i32, nz: i32, day: f32) -> f32 {
    let base = if ny > 0 { 1.0 } else if ny < 0 { 0.45 } else if nz != 0 { 0.80 } else { 0.64 };
    let sun = 0.32 + day * 0.68;
    let direct = if ny > 0 { day * 0.16 } else if nx > 0 { day * 0.05 } else { 0.0 };
    (base * sun + direct).min(1.12)
}

fn ao(world: &mut World, h: &Hit) -> f32 {
    let mut occ = 0;
    let (ax, ay, az) = if h.ny != 0 { (1, 0, 0) } else if h.nx != 0 { (0, 1, 0) } else { (1, 0, 0) };
    let (bx, by, bz) = if h.ny != 0 { (0, 0, 1) } else if h.nx != 0 { (0, 0, 1) } else { (0, 1, 0) };
    for s in [-1, 1] {
        if world::is_opaque(world.get(h.x + h.nx + ax * s, h.y + h.ny + ay * s, h.z + h.nz + az * s)) { occ += 1; }
        if world::is_opaque(world.get(h.x + h.nx + bx * s, h.y + h.ny + by * s, h.z + h.nz + bz * s)) { occ += 1; }
    }
    1.0 - (occ as f32) * 0.075
}

pub fn render_world(fb: &mut Framebuffer, tex: &Textures, world: &mut World, cam: &Cam, time: f32, tick: u32) {
    fb.clear();
    let day = day_factor(time);
    let aspect = fb.w as f32 / fb.h as f32;
    let scale = 0.78;
    for py in 0..fb.h {
        let ny = 1.0 - ((py as f32 + 0.5) / fb.h as f32) * 2.0;
        for px in 0..fb.w {
            let nx = ((px as f32 + 0.5) / fb.w as f32 * 2.0 - 1.0) * aspect * scale;
            let mut dx = cam.fx + cam.rx * nx + cam.ux * ny * scale;
            let mut dy = cam.fy + cam.ry * nx + cam.uy * ny * scale;
            let mut dz = cam.fz + cam.rz * nx + cam.uz * ny * scale;
            let len = (dx * dx + dy * dy + dz * dz).sqrt();
            dx /= len; dy /= len; dz /= len;
            let sky = sky_color(dy, time, px as i32, py as i32);
            let mut r = sky[0] as f32; let mut g = sky[1] as f32; let mut b = sky[2] as f32;
            if let Some(cc) = cloud_color(cam, dx, dy, dz, time, day) {
                let fade = 0.45 + day * 0.5;
                r = r * (1.0 - fade) + cc[0] as f32 * fade;
                g = g * (1.0 - fade) + cc[1] as f32 * fade;
                b = b * (1.0 - fade) + cc[2] as f32 * fade;
            }
            if let Some(hit) = world::raycast(world, cam.x, cam.y, cam.z, dx, dy, dz, 76.0, true) {
                if hit.block == world::WATER || hit.block == world::LAVA {
                    let behind = world::raycast(world,
                        cam.x + dx * (hit.dist + 0.04), cam.y + dy * (hit.dist + 0.04), cam.z + dz * (hit.dist + 0.04),
                        dx, dy, dz, 76.0 - hit.dist, false);
                    if let Some(bh) = behind {
                        let (br, bg, bb, _) = shade_hit(tex, world, &bh, time, day, tick);
                        let fog = (1.0 - (-bh.dist * 0.010).exp()).min(0.85);
                        r = br * (1.0 - fog) + sky[0] as f32 * fog;
                        g = bg * (1.0 - fog) + sky[1] as f32 * fog;
                        b = bb * (1.0 - fog) + sky[2] as f32 * fog;
                    }
                    let t = tex.sample(hit.block, hit.u, hit.v, tick);
                    let alpha = if hit.block == world::WATER { 0.56 } else { 0.86 };
                    r = r * (1.0 - alpha) + t[0] as f32 * alpha;
                    g = g * (1.0 - alpha) + t[1] as f32 * alpha;
                    b = b * (1.0 - alpha) + t[2] as f32 * alpha;
                    fb.depth[py * fb.w + px] = hit.dist;
                } else {
                    let (cr, cg, cb, ca) = shade_hit(tex, world, &hit, time, day, tick);
                    let fog = (1.0 - (-hit.dist * 0.010).exp()).min(0.88);
                    r = cr * (1.0 - fog) + sky[0] as f32 * fog;
                    g = cg * (1.0 - fog) + sky[1] as f32 * fog;
                    b = cb * (1.0 - fog) + sky[2] as f32 * fog;
                    if ca < 255 {
                        let a = ca as f32 / 255.0;
                        r = r * a + sky[0] as f32 * (1.0 - a);
                        g = g * a + sky[1] as f32 * (1.0 - a);
                        b = b * a + sky[2] as f32 * (1.0 - a);
                    } else { fb.depth[py * fb.w + px] = hit.dist; }
                }
            }
            // sun / moon disc
            let sun_dir = [time * 2.0 * PI - PI / 2.0, (time * 2.0 * PI).sin() * 0.7 + 0.25];
            let sunx = sun_dir[0].cos();
            let suny = sun_dir[1];
            let sd = ((dx - sunx) * (dx - sunx) + (dy - suny) * (dy - suny) + dz * dz).sqrt();
            if sd < 0.035 && day > 0.5 { r = 255.0; g = 245.0; b = 190.0; }
            if sd < 0.022 && day < 0.25 { r = 220.0; g = 225.0; b = 235.0; }
            fb.rgba[(py * fb.w + px) * 4] = r as u8;
            fb.rgba[(py * fb.w + px) * 4 + 1] = g as u8;
            fb.rgba[(py * fb.w + px) * 4 + 2] = b as u8;
            fb.rgba[(py * fb.w + px) * 4 + 3] = 255;
        }
    }
}

fn shade_hit(tex: &Textures, world: &mut World, h: &Hit, _time: f32, day: f32, tick: u32) -> (f32, f32, f32, u8) {
    let mut t = tex.sample(h.block, h.u, h.v, tick);
    if h.ny > 0 && h.block == world::GRASS { t = tex.grass_top(h.u, h.v); }
    let mut light = face_shade(h.nx, h.ny, h.nz, day) * ao(world, h);
    if h.block == world::TORCH || h.block == world::CAMPFIRE || h.block == world::GLOWSTONE || h.block == world::SEA_LANTERN { light = 1.25; }
    if h.block == world::LAVA || h.block == world::GLOWSTONE || h.block == world::SEA_LANTERN { light = 1.15; }
    (t[0] as f32 * light, t[1] as f32 * light, t[2] as f32 * light, t[3])
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

pub fn draw_sprite(fb: &mut Framebuffer, cam: &Cam, x: f32, y: f32, z: f32, height: f32, tick: u32, day: f32, kind: SpriteKind, front: bool) {
    let sh = height * (fb.w as f32 / 1.56);
    let (sx, sy, dist) = match project(cam, x, y, z, fb) { Some(v) => v, None => { if matches!(kind, SpriteKind::Steve) { eprintln!("steve behind cam"); } return } };
    if dist > 100.0 { return; }
    if matches!(kind, SpriteKind::Steve) && dist < 1.9 { return; }
    let pxh = (height / dist * (fb.w as f32 / 1.56)).max(1.0);
    let sprite_size = match kind { SpriteKind::Steve => 32u32, SpriteKind::Drop(_) => 16, _ => 16 };
    let pw = pxh * (if sprite_size == 32 { 1.0 } else { 1.0 });
    let left = sx - pw / 2.0;
    let top = sy - pxh;
    let light = 0.45 + day * 0.55;
    // ground shadow
    if let Some((gx, gy, gd)) = project(cam, x, y + 0.02, z, fb) {
        let rw = (0.55 / gd * (fb.w as f32 / 1.56)).max(1.0);
        for yy in -rw as i32 / 3..=rw as i32 / 3 {
            for xx in -rw as i32..=rw as i32 {
                if xx * xx + yy * yy * 9 > (rw * rw) as i32 { continue; }
                let a = ((1.0 - day * 0.45) * 38.0) as u8;
                fb.set(gx as i32 + xx, gy as i32 + yy, 10, 12, 16, a);
            }
        }
    }
    for py in 0..sprite_size {
        for px in 0..sprite_size {
            let c = match kind {
                SpriteKind::Steve => entities::steve_pixel(px, py, tick, false),
                SpriteKind::Mob(m) => entities::sprite_pixel(m, px, py, tick, false),
                SpriteKind::Drop(item) => item_pixel(item, px, py, tick),
            };
            if c[3] == 0 { continue; }
            let fx0 = left + px as f32 / sprite_size as f32 * pw;
            let fx1 = left + (px + 1) as f32 / sprite_size as f32 * pw;
            let fy0 = top + py as f32 / sprite_size as f32 * pxh;
            let fy1 = top + (py + 1) as f32 / sprite_size as f32 * pxh;
            let x0 = fx0.floor() as i32; let x1 = (fx1.ceil() as i32).max(x0 + 1);
            let y0 = fy0.floor() as i32; let y1 = (fy1.ceil() as i32).max(y0 + 1);
            for iy in y0..y1 {
                if iy < 0 || iy >= fb.h as i32 { continue; }
                if !front && dist >= fb.depth[iy as usize * fb.w + x0.clamp(0, fb.w as i32 - 1) as usize] { continue; }
                for ix in x0..x1 {
                    if ix < 0 || ix >= fb.w as i32 { continue; }
                    fb.set(ix, iy, (c[0] as f32 * light) as u8, (c[1] as f32 * light) as u8, (c[2] as f32 * light) as u8, c[3]);
                }
            }
        }
    }
    let _ = sh;
}

pub enum SpriteKind { Steve, Mob(MobKind), Drop(u16) }

pub fn item_pixel(item: u16, x: u32, y: u32, tick: u32) -> [u8; 4] {
    let spin = ((tick / 8) % 4) as i32;
    let z = if spin == 0 || spin == 2 { 1 } else { 0 };
    if item >= 256 {
        match item {
            256..=260 => { // pickaxe
                if (x as i32 == 8 + z && y >= 3 && y <= 13) || (y == 4 && (4..12).contains(&(x + z as u32))) { return [120, 80, 45, 255]; }
                if y >= 1 && y <= 4 && (x >= 3 && x <= 12) { return [190, 195, 200, 255]; }
            }
            261..=263 => { if y >= 2 && y <= 12 && x == 8 { return [125, 85, 50, 255]; } if y <= 4 && x >= 5 && x <= 12 { return [200, 205, 210, 255]; } }
            264..=265 => { if x >= 6 && x <= 9 && y >= 2 && y <= 12 { return [140, 100, 55, 255]; } if y <= 5 && x >= 3 && x <= 12 { return [175, 180, 185, 255]; } }
            266..=268 => { if x >= 7 && x <= 9 && y >= 2 && y <= 10 { return [205, 210, 215, 255]; } if y >= 9 && y <= 13 && x >= 5 && x <= 11 { return [110, 75, 45, 255]; } }
            269 => { if (x as i32 - 8).pow(2) + (y as i32 - 8).pow(2) < 36 { return [225, 65, 60, 255]; } if y < 4 { return [65, 130, 55, 255]; } }
            270 => { if x >= 3 && x <= 12 && y >= 5 && y <= 12 { return [225, 190, 130, 255]; } }
            271 | 273 | 275 => { if x >= 4 && x <= 12 && y >= 5 && y <= 12 { return [235, 190, 175, 255]; } }
            272 | 274 | 276 | 277 => { if x >= 4 && x <= 12 && y >= 5 && y <= 12 { return [220, 155, 140, 255]; } }
            278 => { if x >= 4 && x <= 11 && y >= 6 && y <= 10 { return [45, 45, 50, 255]; } }
            279 | 280 => { if x >= 3 && x <= 12 && y >= 6 && y <= 10 { return [215, 215, 220, 255]; } }
            281 => { if x >= 5 && x <= 11 && y >= 5 && y <= 11 { return [80, 225, 235, 255]; } }
            282 => { if x >= 5 && x <= 11 && y >= 5 && y <= 11 { return [80, 200, 120, 255]; } }
            283 => { if x >= 7 && x <= 9 && y >= 3 && y <= 13 { return [150, 105, 60, 255]; } }
            285 => { if x >= 6 && x <= 9 && y >= 4 && y <= 12 { return [235, 230, 215, 255]; } }
            286 => { if x >= 4 && x <= 11 && y >= 6 && y <= 10 { return [130, 85, 55, 255]; } }
            287 => { if x >= 5 && x <= 10 && y >= 5 && y <= 11 { return [245, 245, 235, 255]; } }
            288 => { if x >= 4 && x <= 11 && y >= 5 && y <= 11 { return [140, 30, 35, 255]; } }
            289 => { if x >= 5 && x <= 10 && y >= 5 && y <= 10 { return [215, 215, 210, 255]; } }
            293 => { if x >= 4 && x <= 11 && y >= 4 && y <= 12 { return [70, 140, 220, 255]; } }
            303 => { if x >= 7 && x <= 9 && y >= 3 && y <= 13 { return [255, 90, 40, 255]; } }
            290 => { if x >= 3 && x <= 12 && y >= 4 && y <= 12 { return [(232) as u8, 232, 228, 255]; } }
            291 => { if (x as i32 - 8).pow(2) + ((y as i32 - 8) as i32).pow(2) < 20 { return [245, 238, 220, 255]; } }
            292 => { if x >= 4 && x <= 11 && y >= 5 && y <= 12 { return if y <= 6 { [150, 155, 160, 255] } else { [185, 190, 195, 255] }; } }
            294 => { if x >= 4 && x <= 11 && y >= 4 && y <= 12 { return if y <= 6 { [240, 110, 25, 255] } else { [175, 178, 182, 255] }; } }
            295 => { if (x as i32 - 8).pow(2) + ((y as i32 - 8) as i32).pow(2) < 26 { if (x as i32 - 8).abs() < 2 && y >= 4 && y <= 10 { return [215, 55, 50, 255]; } return [75, 90, 130, 255]; } }
            296 => { if x >= 3 && x <= 12 && y >= 4 && y <= 11 { return if (x + y) % 4 == 0 { [190, 165, 120, 255] } else { [228, 210, 170, 255] }; } }
            297 => { if (x as i32 - 8).pow(2) + ((y as i32 - 8) as i32).pow(2) < 24 { return [225, 200, 90, 255]; } if (x as i32 - 8).abs() < 2 && y <= 8 { return [70, 60, 55, 255]; } }
            298 => { if (x as i32 - 8).abs() <= 4 && (y as i32 - 8).abs() <= 5 { return [140, 120, 170, 255]; } }
            299 => { if x >= 7 && x <= 9 && y >= 4 && y <= 12 { return [170, 120, 90, 255]; } if y <= 4 && x >= 5 && x <= 11 { return [240, 90, 70, 255]; } }
            300 => { if (x as i32 - 8).pow(2) + ((y as i32 - 8) as i32).pow(2) < 30 { return if y < 7 { [170, 120, 70, 255] } else { [120, 85, 50, 255] }; } }
            301 => { if (x as i32 - 8).pow(2) + ((y as i32 - 8) as i32).pow(2) < 22 { return [80, 195, 120, 255]; } if y <= 4 && x >= 6 && x <= 10 { return [245, 215, 80, 255]; } }
            302 => { if (x as i32 - 8).pow(2) + ((y as i32 - 8) as i32).pow(2) < 20 { return [50, 200, 170, 255]; } }
            304 => { if (x as i32 - 8).pow(2) + ((y as i32 - 8) as i32).pow(2) < 24 { return [250, 250, 240, 255]; } }
            305 => { if x >= 5 && x <= 10 && y >= 4 && y <= 12 { return [110, 205, 120, 255]; } if y <= 5 && x >= 6 && x <= 9 { return [190, 160, 105, 255]; } }
            _ => { if x >= 4 && x <= 11 && y >= 4 && y <= 11 { return [190, 190, 195, 255]; } }
        }
        return [0, 0, 0, 0];
    }
    if (item == world::TALL_GRASS || item == world::FLOWER_RED || item == world::FLOWER_YELLOW || item == world::WHEAT || item == world::TORCH) && (x < 5 || x > 10) {
        return [0, 0, 0, 0];
    }
    let (r, g, b) = match item {
        world::GRASS => (90, 150, 70), world::DIRT => (122, 88, 62), world::STONE => (135, 135, 138),
        world::COBBLE => (142, 142, 144), world::SAND | world::SANDSTONE => (222, 208, 165),
        world::WATER => (55, 115, 195), world::LEAVES | world::SPRUCE_LEAVES => (70, 135, 65),
        world::LOG | world::PLANKS => (165, 125, 80), world::GLASS => (190, 225, 235),
        world::COAL_ORE => (100, 100, 105), world::IRON_ORE => (190, 170, 140),
        world::GOLD_ORE => (235, 200, 85), world::DIAMOND_ORE => (90, 215, 225),
        world::REDSTONE_ORE => (215, 70, 70), world::EMERALD_ORE => (80, 200, 110),
        world::WOOL_RED => (190, 60, 55), world::WOOL_BLUE => (60, 95, 190),
        world::WOOL_GREEN => (60, 135, 65), world::WOOL_YELLOW => (225, 195, 60),
        world::WOOL_WHITE => (235, 235, 230), world::LAVA => (240, 110, 30),
        world::BEDROCK => (55, 55, 58), world::SNOW | world::ICE => (225, 240, 245),
        world::BRICK => (155, 80, 65), world::CACTUS => (70, 135, 55),
        world::TNT => (190, 65, 55), world::OBSIDIAN => (40, 35, 55),
        world::CHEST => (145, 95, 55), world::TORCH => (235, 170, 50),
        world::FURNACE => (125, 125, 128), world::CRAFTING => (175, 130, 80),
        world::CONCRETE_BLUE => (60, 100, 190), world::CONCRETE_RED => (200, 60, 60),
        world::CONCRETE_GREEN => (60, 140, 70), world::CONCRETE_YELLOW => (225, 195, 60),
        world::CONCRETE_BLACK => (45, 45, 50), world::CONCRETE_WHITE => (235, 235, 235),
        world::GOLD_BLOCK => (245, 210, 80), world::DIAMOND_BLOCK => (80, 225, 235),
        world::EMERALD_BLOCK => (80, 205, 120), world::IRON_BLOCK => (220, 220, 225),
        _ => (160, 160, 160),
    };
    let shade = if x < 3 || y > 12 { 0.72 } else if x > 12 || y < 3 { 0.88 } else { 1.0 };
    [(r as f32 * shade) as u8, (g as f32 * shade) as u8, (b as f32 * shade) as u8, 255]
}

fn heart(fb: &mut Framebuffer, x: i32, y: i32, filled: bool) {
    let c: [u8; 4] = if filled { [255, 45, 55, 255] } else { [70, 35, 40, 255] };
    let rows = [[0, 1, 1, 0, 0, 1, 1, 0], [1, 1, 1, 1, 1, 1, 1, 1], [1, 1, 1, 1, 1, 1, 1, 1], [0, 1, 1, 1, 1, 1, 1, 0], [0, 0, 1, 1, 1, 1, 0, 0], [0, 0, 0, 1, 1, 0, 0, 0]];
    for (row, bits) in rows.iter().enumerate() {
        for (col, v) in bits.iter().enumerate() { if *v == 1 { fb.set(x + col as i32, y + row as i32, c[0], c[1], c[2], 255); } }
    }
}

pub const SLOT: i32 = 27;
pub const HOT_Y: i32 = 236;
pub fn hotbar_x(i: usize) -> i32 { W as i32 / 2 - (9 * (SLOT + 2)) / 2 + i as i32 * (SLOT + 2) }

pub fn draw_item_icon(fb: &mut Framebuffer, fb_x: i32, fb_y: i32, item: u16, size: i32, tick: u32) {
    for y in 0..size {
        for x in 0..size {
            let c = item_pixel(item, (x as u32 * 16 / size as u32).max(0), (y as u32 * 16 / size as u32).max(0), tick);
            if c[3] > 0 { fb.set(fb_x + x, fb_y + y, c[0], c[1], c[2], c[3]); }
        }
    }
}

pub fn draw_held_item(fb: &mut Framebuffer, item: u16, tick: u32) {
    let x = 414;
    let y = 193;
    // Steve's arm holding the selected item.
    fb.rect(x + 20, y + 28, 18, 42, [220, 175, 135, 255]);
    fb.rect(x + 20, y + 28, 18, 3, [245, 205, 165, 255]);
    fb.rect(x + 17, y + 62, 24, 5, [60, 135, 155, 255]);
    if item != 0 { draw_item_icon(fb, x, y, item, 40, tick); }
}

pub fn draw_hud(fb: &mut Framebuffer, hot: &[u16; 9], counts: &[u32; 9], selected: usize, health: f32, hunger: f32, tick: u32, message: &str, message_timer: f32, biome: Biome, pos: (f32, f32, f32), fps: u32, camera: u8, time: f32) {
    for i in 0..9 {
        let x = hotbar_x(i);
        fb.rect(x, HOT_Y, SLOT, SLOT, [20, 20, 22, 190]);
        fb.frame(x, HOT_Y, SLOT, SLOT, [90, 90, 95, 255]);
        if i == selected { fb.frame(x - 2, HOT_Y - 2, SLOT + 4, SLOT + 4, [255, 255, 255, 255]); }
        if hot[i] != 0 { draw_item_icon(fb, x + 4, HOT_Y + 4, hot[i], SLOT - 8, tick); }
        text(fb, x + 2, HOT_Y + SLOT - 6, &(i + 1).to_string(), [180, 180, 180, 255], 1);
        if counts[i] > 1 { text(fb, x + SLOT - 12, HOT_Y + SLOT - 7, &counts[i].to_string(), [255, 255, 255, 255], 1); }
    }
    for i in 0..10 {
        heart(fb, 8 + i * 9, 8, health > (i as f32 + 1.0) * 2.0);
    }
    for i in 0..10 {
        let c = if hunger > (i as f32 + 1.0) * 2.0 { [220, 175, 85, 255] } else { [70, 55, 35, 255] };
        fb.rect(148 + i * 7, 8, 5, 6, c);
    }
    fb.rect(W as i32 / 2 - 20, 8, 40, 4, [20, 20, 20, 200]);
    fb.rect(W as i32 / 2 - 20, 8, 28, 4, [60, 230, 90, 255]);
    // crosshair
    let cx = W as i32 / 2; let cy = H as i32 / 2;
    fb.rect(cx - 4, cy, 9, 1, [20, 20, 20, 255]); fb.rect(cx, cy - 4, 1, 9, [20, 20, 20, 255]);
    fb.rect(cx - 3, cy, 7, 1, [240, 240, 240, 255]); fb.rect(cx, cy - 3, 1, 7, [240, 240, 240, 255]);
    text(fb, 8, 20, &format!("{}  XYZ {:.0} {:.0} {:.0}", biome.name(), pos.0, pos.1, pos.2), [235, 235, 235, 255], 1);
    text(fb, 8, 30, &format!("FPS {} CAM {} TIME {:02}:00", fps, camera, ((time * 24.0) as i32) % 24), [200, 220, 255, 255], 1);
    if message_timer > 0.0 {
        let w = message.len() as i32 * 6 + 8;
        fb.rect(W as i32 / 2 - w / 2, H as i32 - 46, w, 11, [0, 0, 0, 170]);
        text(fb, W as i32 / 2 - w / 2 + 4, H as i32 - 44, message, [255, 255, 255, 255], 1);
    }
}

pub const INV_X: i32 = 8;
pub const INV_Y: i32 = 18;
pub const INV_W: i32 = 464;
pub const INV_H: i32 = 236;
pub fn inv_storage_rect(i: usize) -> (i32, i32, i32, i32) {
    let col = i % 9; let row = i / 9;
    (INV_X + 12 + col as i32 * 30, INV_Y + 52 + row as i32 * 30, 27, 27)
}
pub fn inv_hotbar_rect(i: usize) -> (i32, i32, i32, i32) {
    (INV_X + 12 + i as i32 * 30, INV_Y + 146, 27, 27)
}
pub fn inv_list_rect(i: usize) -> (i32, i32, i32, i32) {
    let col = i % 4; let row = i / 4;
    (INV_X + 300 + col as i32 * 36, INV_Y + 48 + row as i32 * 26, 30, 24)
}
pub fn inv_prev_rect() -> (i32, i32, i32, i32) { (INV_X + 300, INV_Y + 214, 26, 20) }
pub fn inv_next_rect() -> (i32, i32, i32, i32) { (INV_X + INV_W - 34, INV_Y + 214, 26, 20) }

pub fn draw_inventory(fb: &mut Framebuffer, storage: &[u16; 27], counts: &[u32; 27], hot: &[u16; 9], hot_counts: &[u32; 9], selected: usize, list: &[u16], page: usize, tick: u32) {
    fb.rect(0, 0, fb.w as i32, fb.h as i32, [0, 0, 0, 165]);
    fb.rect(INV_X, INV_Y, INV_W, INV_H, [198, 198, 198, 255]);
    fb.frame(INV_X, INV_Y, INV_W, INV_H, [80, 80, 85, 255]);
    fb.rect(INV_X + 4, INV_Y + 4, INV_W - 8, 16, [70, 70, 75, 255]);
    text(fb, INV_X + 10, INV_Y + 8, "CREATIVE INVENTORY  -  CLICK AN ITEM TO PUT IT IN YOUR SELECTED HOTBAR SLOT", [255, 255, 255, 255], 1);
    text(fb, INV_X + 400, INV_Y + 8, "Steve", [220, 220, 255, 255], 1);
    text(fb, INV_X + 12, INV_Y + 40, "STORAGE", [60, 60, 60, 255], 1);
    for i in 0..27 {
        let (x, y, w, h) = inv_storage_rect(i);
        fb.rect(x, y, w, h, [140, 140, 145, 255]); fb.frame(x, y, w, h, [100, 100, 105, 255]);
        if storage[i] != 0 { draw_item_icon(fb, x + 3, y + 3, storage[i], 21, tick); if counts[i] > 1 { text(fb, x + w - 11, y + h - 7, &counts[i].to_string(), [255, 255, 255, 255], 1); } }
    }
    for i in 0..9 {
        let (x, y, w, h) = inv_hotbar_rect(i);
        fb.rect(x, y, w, h, [140, 140, 145, 255]);
        fb.frame(x, y, w, h, if i == selected { [255, 255, 255, 255] } else { [100, 100, 105, 255] });
        if hot[i] != 0 { draw_item_icon(fb, x + 3, y + 3, hot[i], 21, tick); }
    }
    text(fb, INV_X + 300, INV_Y + 40, "ALL ITEMS", [60, 60, 60, 255], 1);
    for i in 0..24 {
        let idx = page * 24 + i;
        if idx >= list.len() { break; }
        let (x, y, w, h) = inv_list_rect(i);
        fb.rect(x, y, w, h, [225, 225, 225, 255]); fb.frame(x, y, w, h, [120, 120, 125, 255]);
        draw_item_icon(fb, x + 6, y + 2, list[idx], 16, tick);
    }
    let (px, py, pw, ph) = inv_prev_rect();
    fb.rect(px, py, pw, ph, [160, 160, 165, 255]); text(fb, px + 7, py + 6, "<", [30, 30, 30, 255], 1);
    let (nx, ny, nw, nh) = inv_next_rect();
    fb.rect(nx, ny, nw, nh, [160, 160, 165, 255]); text(fb, nx + 7, ny + 6, ">", [30, 30, 30, 255], 1);
    text(fb, INV_X + 334, INV_Y + 218, &format!("PAGE {}/{}", page + 1, (list.len() + 23) / 24), [40, 40, 40, 255], 1);
    text(fb, INV_X + 12, INV_Y + 218, "E / ESC CLOSE", [40, 40, 40, 255], 1);
}

pub fn upscale(src: &Framebuffer, dst: &mut Framebuffer) {
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
