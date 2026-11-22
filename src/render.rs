// Software renderer: 16x16 face-aware pixel textures, voxel raycaster,
// 3D rasterized box-model mobs/player, isometric 3D block icons, and Minecraft UI.
use crate::entities::{self, FaceArt, MobKind, Part3D};
use crate::world::{self, Biome, Dimension, RenderCache, World};
use std::f32::consts::PI;
use std::sync::{OnceLock, RwLock, RwLockReadGuard};

pub const W: usize = 480;
pub const H: usize = 270;
pub const OUT_W: usize = 960;
pub const OUT_H: usize = 540;
#[allow(dead_code)]
pub const RW: usize = 480;
#[allow(dead_code)]
pub const RH: usize = 270;
const TEX: usize = 16;
const NUM_BLOCK_TEX: usize = 144;
const NUM_ITEM_TEX: usize = 128;

static ATLAS_BYTES: &[u8] = include_bytes!("../assets/textures.bin");
static ATLAS_CACHE: OnceLock<RwLock<Textures>> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct ShaderConfig {
    pub name: String,
    pub sun_shadows: bool,
    pub water_reflect: bool,
    pub warm_torch: f32,
    pub saturation: f32,
    pub contrast: f32,
    pub brightness: f32,
    pub fog_density: f32,
    pub sky_warmth: f32,
    pub tonemap_aces: bool,
}

impl ShaderConfig {
    pub fn vanilla() -> Self {
        Self {
            name: String::from("VANILLA"),
            sun_shadows: false,
            water_reflect: false,
            warm_torch: 1.0,
            saturation: 1.0,
            contrast: 1.0,
            brightness: 1.0,
            fog_density: 1.0,
            sky_warmth: 0.0,
            tonemap_aces: false,
        }
    }

    pub fn preset(idx: usize) -> Self {
        match idx % 6 {
            1 => Self {
                name: String::from("BSL WARM"),
                sun_shadows: true,
                water_reflect: true,
                warm_torch: 1.38,
                saturation: 1.16,
                contrast: 1.08,
                brightness: 1.04,
                fog_density: 1.22,
                sky_warmth: 0.16,
                tonemap_aces: true,
            },
            2 => Self {
                name: String::from("COMPLEMENTARY"),
                sun_shadows: true,
                water_reflect: true,
                warm_torch: 1.28,
                saturation: 1.26,
                contrast: 1.10,
                brightness: 1.06,
                fog_density: 1.05,
                sky_warmth: 0.08,
                tonemap_aces: true,
            },
            3 => Self {
                name: String::from("SILDURS VIBRANT"),
                sun_shadows: true,
                water_reflect: true,
                warm_torch: 1.45,
                saturation: 1.22,
                contrast: 1.14,
                brightness: 1.02,
                fog_density: 1.30,
                sky_warmth: 0.22,
                tonemap_aces: true,
            },
            4 => Self {
                name: String::from("NOSTALGIA FOG"),
                sun_shadows: false,
                water_reflect: true,
                warm_torch: 1.15,
                saturation: 0.90,
                contrast: 1.04,
                brightness: 1.0,
                fog_density: 1.75,
                sky_warmth: 0.10,
                tonemap_aces: false,
            },
            5 => Self {
                name: String::from("HORIZON HDR"),
                sun_shadows: true,
                water_reflect: true,
                warm_torch: 1.42,
                saturation: 1.18,
                contrast: 1.09,
                brightness: 1.05,
                fog_density: 1.10,
                sky_warmth: 0.14,
                tonemap_aces: true,
            },
            _ => Self::vanilla(),
        }
    }
}

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
    pub fov_scale: f32,
}

impl Cam {
    pub fn from(yaw: f32, pitch: f32, x: f32, y: f32, z: f32) -> Self {
        Self::with_fov(yaw, pitch, x, y, z, 1.0)
    }

    pub fn with_fov(yaw: f32, pitch: f32, x: f32, y: f32, z: f32, fov_mul: f32) -> Self {
        let cp = pitch.cos();
        let fx = yaw.sin() * cp;
        let fy = pitch.sin();
        let fz = -yaw.cos() * cp;
        let rx = yaw.cos();
        let rz = yaw.sin();
        let ux = -rz * fy;
        let uy = rz * fx - rx * fz;
        let uz = rx * fy;
        Self { x, y, z, fx, fy, fz, rx, ry: 0.0, rz, ux, uy, uz, fov_scale: FOV_SCALE * fov_mul }
    }
}

fn th(x: i32, y: i32, k: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374761393)
        ^ (y as u32).wrapping_mul(668265263)
        ^ (k as u32).wrapping_mul(2246822519);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

#[derive(Clone)]
pub struct Textures {
    blocks: Vec<u8>,
    items: Vec<u8>,
}

impl Textures {
    pub fn atlas() -> RwLockReadGuard<'static, Textures> {
        ATLAS_CACHE
            .get_or_init(|| RwLock::new(Self::from_embedded()))
            .read()
            .expect("textures rwlock")
    }

    pub fn set_active(tex: &Textures) {
        let lock = ATLAS_CACHE.get_or_init(|| RwLock::new(Self::from_embedded()));
        if let Ok(mut w) = lock.write() {
            *w = tex.clone();
        }
    }

    pub fn reset(&mut self) {
        *self = Self::from_embedded();
    }

    pub fn set_block_tile(&mut self, idx: usize, rgba_16x16: &[u8]) {
        if idx < NUM_BLOCK_TEX && rgba_16x16.len() == TEX * TEX * 4 {
            let off = idx * TEX * TEX * 4;
            self.blocks[off..off + TEX * TEX * 4].copy_from_slice(rgba_16x16);
        }
    }

    pub fn set_item_tile(&mut self, idx: usize, rgba_16x16: &[u8]) {
        if idx < NUM_ITEM_TEX && rgba_16x16.len() == TEX * TEX * 4 {
            let off = idx * TEX * TEX * 4;
            self.items[off..off + TEX * TEX * 4].copy_from_slice(rgba_16x16);
        }
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
            world::TORCH | world::SOUL_TORCH | world::CAMPFIRE => 14,
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
            world::JUKEBOX => 12,
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
    pub fn clear_transparent(&mut self) {
        self.rgba.fill(0);
        self.clear();
    }
    #[inline]
    pub fn set(&mut self, x: i32, y: i32, r: u8, g: u8, b: u8, a: u8) {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 || a == 0 { return; }
        let i = (y as usize * self.w + x as usize) * 4;
        if a == 255 {
            self.rgba[i] = r; self.rgba[i + 1] = g; self.rgba[i + 2] = b; self.rgba[i + 3] = 255;
        } else {
            let da = self.rgba[i + 3];
            if da == 0 {
                self.rgba[i] = r; self.rgba[i + 1] = g; self.rgba[i + 2] = b; self.rgba[i + 3] = a;
            } else if da == 255 {
                let t = a as f32 / 255.0;
                self.rgba[i] = (self.rgba[i] as f32 * (1.0 - t) + r as f32 * t) as u8;
                self.rgba[i + 1] = (self.rgba[i + 1] as f32 * (1.0 - t) + g as f32 * t) as u8;
                self.rgba[i + 2] = (self.rgba[i + 2] as f32 * (1.0 - t) + b as f32 * t) as u8;
                self.rgba[i + 3] = 255;
            } else {
                let sa = a as f32 / 255.0;
                let daf = da as f32 / 255.0;
                let oa = sa + daf * (1.0 - sa);
                let inv = 1.0 / oa.max(1e-5);
                self.rgba[i] = ((r as f32 * sa + self.rgba[i] as f32 * daf * (1.0 - sa)) * inv) as u8;
                self.rgba[i + 1] = ((g as f32 * sa + self.rgba[i + 1] as f32 * daf * (1.0 - sa)) * inv) as u8;
                self.rgba[i + 2] = ((b as f32 * sa + self.rgba[i + 2] as f32 * daf * (1.0 - sa)) * inv) as u8;
                self.rgba[i + 3] = (oa * 255.0).clamp(0.0, 255.0) as u8;
            }
        }
    }
    /// Horizon-style soft edge vignette on the output framebuffer.
    pub fn apply_vignette(&mut self, strength: f32) {
        if strength <= 0.0 { return; }
        let w = self.w as f32;
        let h = self.h as f32;
        for y in 0..self.h {
            let ny = (y as f32 + 0.5) / h * 2.0 - 1.0;
            let ny2 = ny * ny;
            for x in 0..self.w {
                let nx = (x as f32 + 0.5) / w * 2.0 - 1.0;
                let r2 = (nx * nx * 0.65 + ny2 * 0.85).min(1.4);
                let v = (1.0 - r2 * strength).clamp(0.62, 1.0);
                if v < 0.995 {
                    let i = (y * self.w + x) * 4;
                    self.rgba[i] = (self.rgba[i] as f32 * v) as u8;
                    self.rgba[i + 1] = (self.rgba[i + 1] as f32 * v) as u8;
                    self.rgba[i + 2] = (self.rgba[i + 2] as f32 * v) as u8;
                }
            }
        }
    }
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: [u8; 4]) {
        for yy in y..y + h { for xx in x..x + w { self.set(xx, yy, c[0], c[1], c[2], c[3]); } }
    }
    pub fn frame(&mut self, x: i32, y: i32, w: i32, h: i32, c: [u8; 4]) {
        self.rect(x, y, w, 1, c); self.rect(x, y + h - 1, w, 1, c);
        self.rect(x, y, 1, h, c); self.rect(x + w - 1, y, 1, h, c);
    }
    /// Fast separable box blur + dark scrim for modern Java / Horizon menu backgrounds.
    pub fn blur_dim(&mut self, radius: i32, dark_alpha: u8) {
        let w = self.w as i32;
        let h = self.h as i32;
        if radius > 0 {
            let mut tmp = vec![0u8; self.rgba.len()];
            let span = radius * 2 + 1;
            // Horizontal pass
            for y in 0..h {
                for x in 0..w {
                    let (mut sr, mut sg, mut sb) = (0i32, 0i32, 0i32);
                    for dx in -radius..=radius {
                        let sx = (x + dx).clamp(0, w - 1) as usize;
                        let idx = (y as usize * self.w + sx) * 4;
                        sr += self.rgba[idx] as i32;
                        sg += self.rgba[idx + 1] as i32;
                        sb += self.rgba[idx + 2] as i32;
                    }
                    let di = (y as usize * self.w + x as usize) * 4;
                    tmp[di] = (sr / span) as u8;
                    tmp[di + 1] = (sg / span) as u8;
                    tmp[di + 2] = (sb / span) as u8;
                    tmp[di + 3] = 255;
                }
            }
            // Vertical pass + dim
            let mul = 1.0 - dark_alpha as f32 / 255.0;
            for y in 0..h {
                for x in 0..w {
                    let (mut sr, mut sg, mut sb) = (0i32, 0i32, 0i32);
                    for dy in -radius..=radius {
                        let sy = (y + dy).clamp(0, h - 1) as usize;
                        let idx = (sy * self.w + x as usize) * 4;
                        sr += tmp[idx] as i32;
                        sg += tmp[idx + 1] as i32;
                        sb += tmp[idx + 2] as i32;
                    }
                    let di = (y as usize * self.w + x as usize) * 4;
                    self.rgba[di] = ((sr / span) as f32 * mul) as u8;
                    self.rgba[di + 1] = ((sg / span) as f32 * mul) as u8;
                    self.rgba[di + 2] = ((sb / span) as f32 * mul) as u8;
                    self.rgba[di + 3] = 255;
                }
            }
        } else if dark_alpha > 0 {
            self.rect(0, 0, w, h, [0, 0, 0, dark_alpha]);
        }
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

fn sky_color(dim: Dimension, dir_x: f32, dir_y: f32, dir_z: f32, time: f32, x: i32, y: i32) -> [f32; 3] {
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
            let sun_x = (time * 2.0 * PI).cos();
            let sun_y = (time * 2.0 * PI).sin();
            let sun_z = 0.24f32;
            let sun_len = (sun_x * sun_x + sun_y * sun_y + sun_z * sun_z).sqrt();
            let sun_dot = ((dir_x * sun_x + dir_y * sun_y + dir_z * sun_z) / sun_len).clamp(-1.0, 1.0);
            let day = day_factor(time);
            let horizon = 0.22 + sun_y.abs() * 0.10;
            let t = ((dir_y * 0.5 + 0.5) * (1.0 - horizon) + horizon).clamp(0.0, 1.0);
            let night_top = [8.0, 14.0, 36.0];
            let night_bot = [22.0, 26.0, 56.0];
            let day_top = [98.0, 156.0, 250.0];
            let day_bot = [184.0, 220.0, 255.0];
            let dusk = [246.0, 132.0, 74.0];
            let dusk_amt = (1.0 - sun_y.abs() * 3.0).clamp(0.0, 1.0);
            let dir_warm = ((sun_dot * 0.5 + 0.5).powf(2.0)) * dusk_amt * (1.0 - dir_y.abs().min(1.0));
            let corona = (sun_dot.max(0.0)).powf(12.0) * day * 0.28;
            let mut c = [0.0; 3];
            for i in 0..3 {
                let mut v = night_bot[i] + (night_top[i] - night_bot[i]) * t;
                let dv = day_bot[i] + (day_top[i] - day_bot[i]) * t;
                v = v + (dv - v) * day;
                v = v * (1.0 - dir_warm * 0.72) + dusk[i] * dir_warm * 0.85;
                c[i] = v;
            }
            c[0] = (c[0] + corona * 255.0).min(255.0);
            c[1] = (c[1] + corona * 228.0).min(255.0);
            c[2] = (c[2] + corona * 165.0).min(255.0);
            if day < 0.2 && dir_y > 0.02 && th(x, y, 91) % 900 == 0 {
                c = [235.0, 235.0, 220.0];
            }
            c
        }
    }
}

#[inline]
fn cloud_cell_solid(cx: i32, cz: i32) -> bool {
    let macro_h = th(cx.div_euclid(2), cz.div_euclid(2), 777) % 11;
    if macro_h >= 3 {
        return false;
    }
    th(cx, cz, 779) % 7 != 0
}

#[inline]
fn cloud_color_3d(cam: &Cam, dx: f32, dy: f32, dz: f32, time: f32, day: f32) -> Option<([f32; 3], f32, f32)> {
    const Y_BOT: f32 = 108.0;
    const Y_TOP: f32 = 112.0;
    const CELL: f32 = 12.0;
    const MAX_T: f32 = 260.0;

    let (t0, t1, enter_ny) = if cam.y < Y_BOT {
        if dy <= 0.005 { return None; }
        ((Y_BOT - cam.y) / dy, (Y_TOP - cam.y) / dy, -1)
    } else if cam.y > Y_TOP {
        if dy >= -0.005 { return None; }
        ((Y_TOP - cam.y) / dy, (Y_BOT - cam.y) / dy, 1)
    } else {
        let exit_t = if dy > 0.001 {
            (Y_TOP - cam.y) / dy
        } else if dy < -0.001 {
            (Y_BOT - cam.y) / dy
        } else {
            MAX_T
        };
        (0.0, exit_t, 0)
    };
    if t0 > MAX_T || t1 <= 0.0 { return None; }
    let t_end = t1.min(MAX_T);

    let drift_x = time * 196.0;
    let t_start = t0 + 0.002;
    let sx = cam.x + drift_x + dx * t_start;
    let sz = cam.z + dz * t_start;
    let mut cx = (sx / CELL).floor() as i32;
    let mut cz = (sz / CELL).floor() as i32;

    if enter_ny != 0 && cloud_cell_solid(cx, cz) {
        let shade = if enter_ny > 0 { 1.00 } else { 0.78 };
        let base = 204.0 + day * 51.0;
        let dist_fade = (1.0 - (t_start / MAX_T).powf(2.2)).clamp(0.0, 1.0);
        let alpha = (0.58 + day * 0.34) * dist_fade;
        if alpha <= 0.01 { return None; }
        return Some(([base * shade, (base + 1.0) * shade, (base + 6.0) * shade], t_start, alpha));
    }

    let step_x = if dx >= 0.0 { 1 } else { -1 };
    let step_z = if dz >= 0.0 { 1 } else { -1 };
    let dtx = if dx.abs() > 1e-6 { CELL / dx.abs() } else { 1e9 };
    let dtz = if dz.abs() > 1e-6 { CELL / dz.abs() } else { 1e9 };
    let mut tmx = if dx.abs() > 1e-6 {
        let border = if step_x > 0 { (cx + 1) as f32 * CELL } else { cx as f32 * CELL };
        t_start + (border - sx) / dx
    } else {
        1e9
    };
    let mut tmz = if dz.abs() > 1e-6 {
        let border = if step_z > 0 { (cz + 1) as f32 * CELL } else { cz as f32 * CELL };
        t_start + (border - sz) / dz
    } else {
        1e9
    };

    for _ in 0..28 {
        let (t_hit, shade) = if tmx < tmz {
            let t_h = tmx;
            tmx += dtx;
            cx += step_x;
            (t_h, 0.85)
        } else {
            let t_h = tmz;
            tmz += dtz;
            cz += step_z;
            (t_h, 0.90)
        };
        if t_hit > t_end {
            break;
        }
        if cloud_cell_solid(cx, cz) {
            let base = 204.0 + day * 51.0;
            let dist_fade = (1.0 - (t_hit / MAX_T).powf(2.2)).clamp(0.0, 1.0);
            let alpha = (0.58 + day * 0.34) * dist_fade;
            if alpha <= 0.01 { return None; }
            return Some(([base * shade, (base + 1.0) * shade, (base + 6.0) * shade], t_hit, alpha));
        }
    }
    None
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
fn ao_smooth(cache: &RenderCache, bx: i32, by: i32, bz: i32, nx: i32, ny: i32, nz: i32, u: f32, v: f32) -> f32 {
    // Tangent axes aligned with world::hit_uv(u, v)
    let (ux, uy, uz, vx, vy, vz) = if nx != 0 {
        (0, 0, if nx > 0 { -1 } else { 1 }, 0, -1, 0)
    } else if ny != 0 {
        (1, 0, 0, 0, 0, 1)
    } else {
        (if nz > 0 { 1 } else { -1 }, 0, 0, 0, -1, 0)
    };
    let sx = bx + nx;
    let sy = by + ny;
    let sz = bz + nz;
    let corner_ao = |su: i32, sv: i32| -> f32 {
        let s1 = world::is_opaque(cache.get(sx + ux * su, sy + uy * su, sz + uz * su));
        let s2 = world::is_opaque(cache.get(sx + vx * sv, sy + vy * sv, sz + vz * sv));
        let occ = if s1 && s2 {
            3
        } else {
            let c = world::is_opaque(cache.get(
                sx + ux * su + vx * sv,
                sy + uy * su + vy * sv,
                sz + uz * su + vz * sv,
            ));
            s1 as usize + s2 as usize + c as usize
        };
        [1.00, 0.85, 0.70, 0.54][occ]
    };
    let a00 = corner_ao(-1, -1);
    let a10 = corner_ao(1, -1);
    let a01 = corner_ao(-1, 1);
    let a11 = corner_ao(1, 1);
    let u = u.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    let top = a00 + (a10 - a00) * u;
    let bot = a01 + (a11 - a01) * u;
    top + (bot - top) * v
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
            | world::TORCH | world::SOUL_TORCH | world::REDSTONE_TORCH | world::BREWING_STAND | world::LANTERN
            | world::HOPPER | world::PRESSURE_PLATE | world::CAKE | world::FARMLAND
            | world::DOOR | world::DOOR_OPEN | world::IRON_DOOR | world::IRON_DOOR_OPEN
            | world::BUTTON | world::SNOW_LAYER | world::BED | world::CHEST | world::SIGN
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
        world::TORCH | world::SOUL_TORCH | world::REDSTONE_TORCH => {
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
        world::BED => {
            // Mattress + 4 wooden corner legs (authentic Minecraft bed height 9/16)
            boxes[0] = [0.0, 3.0*p, 0.0, 1.0, 9.0*p, 1.0];
            boxes[1] = [0.0, 0.0, 0.0, 3.0*p, 3.0*p, 3.0*p];
            boxes[2] = [13.0*p, 0.0, 0.0, 1.0, 3.0*p, 3.0*p];
            boxes[3] = [0.0, 0.0, 13.0*p, 3.0*p, 3.0*p, 1.0];
            boxes[4] = [13.0*p, 0.0, 13.0*p, 1.0, 3.0*p, 1.0];
            nb = 5;
        }
        world::CHEST => {
            // Official 14/16 x 14/16 x 14/16 Minecraft chest + latch
            boxes[0] = [1.0*p, 0.0, 1.0*p, 15.0*p, 14.0*p, 15.0*p];
            boxes[1] = [7.0*p, 7.0*p, 15.0*p, 9.0*p, 11.0*p, 1.0];
            nb = 2;
        }
        world::SIGN => {
            boxes[0] = [7.0*p, 0.0, 7.0*p, 9.0*p, 9.0*p, 9.0*p];
            boxes[1] = [1.0*p, 9.0*p, 7.0*p, 15.0*p, 1.0, 9.0*p];
            nb = 2;
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
fn sun_shadow(cache: &RenderCache, hx: f32, hy: f32, hz: f32, sx: f32, sy: f32, sz: f32) -> f32 {
    if sy <= 0.12 { return 0.68; }
    for step in 1..=12 {
        let d = step as f32 * 0.72;
        let bx = (hx + sx * d).floor() as i32;
        let by = (hy + sy * d).floor() as i32;
        let bz = (hz + sz * d).floor() as i32;
        if by >= world::WORLD_H as i32 { break; }
        let b = cache.get(bx, by, bz);
        if world::is_opaque(b) {
            return 0.56;
        }
        if matches!(b, world::LEAVES | world::BIRCH_LEAVES | world::SPRUCE_LEAVES | world::JUNGLE_LEAVES | world::DARK_LEAVES) {
            return 0.70;
        }
    }
    1.0
}

#[allow(clippy::too_many_arguments)]
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
    shader: &ShaderConfig,
    dyn_light: Option<(f32, f32, f32, f32)>,
) -> ([f32; 3], u8) {
    let mut t = tex.sample_face(b, nx, ny, nz, u, v, tick);
    if t[3] == 0 {
        return ([0.0, 0.0, 0.0], 0);
    }
    if b == world::BED {
        if ny > 0 && v < 0.34 {
            t = [242, 242, 245, 255]; // crisp white pillow
        } else if ny != 0 && ny < 0 || (ny == 0 && v > 0.78) {
            t = [152, 114, 70, 255]; // oak frame & legs
        }
    } else if b == world::JUKEBOX {
        if ny > 0 && u > 0.25 && u < 0.75 && v > 0.44 && v < 0.56 {
            t = [22, 22, 26, 255]; // record slot
        } else {
            t = [(t[0] as f32 * 0.65) as u8, (t[1] as f32 * 0.50) as u8, (t[2] as f32 * 0.44) as u8, 255];
        }
    } else if b == world::SOUL_TORCH && v < 0.35 {
        let flick = ((tick / 4) % 2) as u8 * 25;
        t = [80 + flick, 225, 255, 255];
    }
    let fl = face_light(nx, ny, nz, day);
    let ao_v = if apply_ao { ao_smooth(cache, bx, by, bz, nx, ny, nz, u, v) } else { 0.94 };
    let emit = world::emits_light(b);
    let sky_lv = cache
        .get_sky_light(bx + nx, by + ny, bz + nz)
        .max(cache.get_sky_light(bx, by, bz)) as f32
        / 15.0;
    let sky_mul = 0.09 + 0.91 * sky_lv.powf(1.25);
    let mut lr = fl[0] * ao_v * sky_mul;
    let mut lg = fl[1] * ao_v * sky_mul;
    let mut lb = fl[2] * ao_v * sky_mul;
    let mut bl = cache.get_light(bx + nx, by + ny, bz + nz).max(cache.get_light(bx, by, bz)) as f32 / 15.0;
    if let Some((lx, ly, lz, lstr)) = dyn_light {
        let sx = bx as f32 + 0.5 + nx as f32 * 0.55;
        let sy = by as f32 + 0.5 + ny as f32 * 0.55;
        let sz = bz as f32 + 0.5 + nz as f32 * 0.55;
        let d2 = (sx - lx) * (sx - lx) + (sy - ly) * (sy - ly) + (sz - lz) * (sz - lz);
        if d2 < 100.0 {
            let falloff = (1.0 - d2.sqrt() * 0.10).clamp(0.0, 1.0);
            bl = bl.max(lstr * falloff * falloff.sqrt());
        }
    }
    if bl > 0.0 {
        let warm = bl * 0.98 * shader.warm_torch;
        lr = lr.max(warm * 1.12);
        lg = lg.max(warm * 0.96);
        lb = lb.max(warm * 0.76);
    }
    if emit > 0 {
        let e = (0.98 + emit as f32 * 0.025) * shader.warm_torch.min(1.35);
        lr = lr.max(e); lg = lg.max(e * 0.97); lb = lb.max(e * 0.88);
    }
    if b == world::LAMP && cache.is_lit(bx, by, bz) {
        lr = 1.48 * shader.warm_torch; lg = 1.38 * shader.warm_torch; lb = 1.15;
    }
    // Underwater sunlight caustics on submerged blocks
    if shader.water_reflect && day > 0.18 && cache.get(bx, by + 1, bz) == world::WATER {
        let phase = tick as f32 * 0.09;
        let wx = bx as f32 + u;
        let wz = bz as f32 + v;
        let caust = (((wx * 4.2 + phase).sin() * (wz * 4.2 - phase * 0.85).cos()) + 1.0) * 0.5;
        let shimmer = caust.powf(3.0) * 0.34 * day;
        lr += shimmer * 0.55;
        lg += shimmer * 0.95;
        lb += shimmer * 1.15;
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
    spectator: bool,
    dim: Dimension,
    mining: Option<(i32, i32, i32, u8)>,
    shader: &ShaderConfig,
    dyn_light: Option<(f32, f32, f32, f32)>,
) -> ([u8; 3], f32) {
    let ox = cam.x; let oy = cam.y; let oz = cam.z;
    let mut x = ox.floor() as i32;
    let mut y = oy.floor() as i32;
    let mut z = oz.floor() as i32;
    let start_b = if y >= 0 && y < world::WORLD_H as i32 { cache.get(x, y, z) } else { world::AIR };
    let mut prev_b = start_b;
    let mut spec_exited = !(spectator && world::is_solid(start_b));
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
    let fog_rate = (if dim == Dimension::Nether { 0.015 } else { 0.0085 }) * shader.fog_density;
    let ang = time * 2.0 * PI;
    let (mut sun_x, mut sun_y, mut sun_z) = (ang.cos(), ang.sin(), 0.22f32);
    let sun_l = (sun_x * sun_x + sun_y * sun_y + sun_z * sun_z).sqrt();
    sun_x /= sun_l; sun_y /= sun_l; sun_z /= sun_l;

    for _ in 0..210 {
        let t_next = tmx.min(tmy).min(tmz);
        let mut step_b = world::AIR;
        if y >= 0 && y < world::WORLD_H as i32 {
            let b = cache.get(x, y, z);
            step_b = b;
            // In Spectator mode inside solid blocks, skip contiguous solid terrain until exiting into open air/cave
            if !spec_exited {
                if !world::is_solid(b) {
                    spec_exited = true;
                } else {
                    prev_b = b;
                    if tmx < tmy && tmx < tmz { t = tmx; tmx += dtx; x += step_x; nx = -step_x; ny = 0; nz = 0; }
                    else if tmy < tmz { t = tmy; tmy += dty; y += step_y; ny = -step_y; nx = 0; nz = 0; }
                    else { t = tmz; tmz += dtz; z += step_z; nz = -step_z; nx = 0; ny = 0; }
                    if t > max_dist { break; }
                    continue;
                }
            }
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
                        let (rgb0, a) = shade_voxel(tex, cache, b, x, y, z, 0, 1, 0, u, v, day, tick, false, shader, dyn_light);
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
                        let (rgb0, a) = shade_voxel(tex, cache, b, x, y, z, mnx, mny, mnz, u, v, day, tick, false, shader, dyn_light);
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
                                let (rgb0, a) = shade_voxel(tex, cache, b, x, y, z, 0, 1, 0, px, pz, day, tick, false, shader, dyn_light);
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
                        let (mut u, mut v) = world::hit_uv(ox, oy, oz, dx, dy, dz, t, nx, ny, nz);
                        if b == world::WATER && shader.water_reflect {
                            let wave = ((u * 12.56 + tick as f32 * 0.12).sin() * (v * 12.56 + tick as f32 * 0.08).cos()) * 0.04;
                            u = (u + wave).rem_euclid(1.0);
                            v = (v - wave * 0.8).rem_euclid(1.0);
                        }
                        let tpx = tex.sample_face(b, nx, ny, nz, u, v, tick);
                        if b == world::WATER && !has_trans && prev_b != world::WATER {
                            let fres_base = if shader.water_reflect { 0.58 } else { 0.35 };
                            let fresnel = ((1.0 - dy.abs()).powf(2.2) * fres_base).clamp(0.08, 0.65);
                            let ripple = if shader.water_reflect {
                                1.0 + ((u * 18.0 + tick as f32 * 0.16).sin() * 0.14)
                            } else if ((u * 16.0) as i32 + (v * 16.0) as i32 + (tick / 8) as i32) % 9 == 0 {
                                1.12
                            } else {
                                1.0
                            };
                            let sun_glint = if shader.water_reflect && sun_y > 0.08 {
                                let rx = dx; let ry = -dy; let rz = dz;
                                let spec = (rx * sun_x + ry * sun_y + rz * sun_z).max(0.0).powf(20.0) * 145.0;
                                spec
                            } else { 0.0 };
                            let light = 0.55 + day * 0.48;
                            trans_rgb = [
                                tpx[0] as f32 * light * ripple * (1.0 - fresnel) + sky[0] * fresnel + sun_glint,
                                tpx[1] as f32 * light * ripple * (1.0 - fresnel) + sky[1] * fresnel + sun_glint * 0.94,
                                tpx[2] as f32 * light * ripple * (1.0 - fresnel) + sky[2] * fresnel + sun_glint * 0.78,
                            ];
                            trans_alpha = if shader.water_reflect { 0.46 } else { 0.52 };
                            trans_dist = t;
                            has_trans = true;
                        } else if b == world::NETHER_PORTAL && !has_trans {
                            trans_rgb = [tpx[0] as f32 * 1.25, tpx[1] as f32 * 1.10, tpx[2] as f32 * 1.35];
                            trans_alpha = 0.72;
                            trans_dist = t;
                            has_trans = true;
                        } else if b == world::LAVA || b == world::END_PORTAL {
                            let pulse = 1.08 + ((tick as f32 * 0.09 + u * 8.0).sin() * 0.12);
                            let fog = (1.0 - (-t * fog_rate).exp()).min(0.85);
                            hit_rgb = [
                                tpx[0] as f32 * pulse * shader.warm_torch.min(1.3) * (1.0 - fog) + sky[0] * fog,
                                tpx[1] as f32 * pulse * (1.0 - fog) + sky[1] * fog,
                                tpx[2] as f32 * pulse * (1.0 - fog) + sky[2] * fog,
                            ];
                            hit_dist = t;
                            break;
                        }
                    }
                } else if is_cutout_cube(b) {
                    if t > 0.001 {
                        let (u, v) = world::hit_uv(ox, oy, oz, dx, dy, dz, t, nx, ny, nz);
                        let (rgb0, a) = shade_voxel(tex, cache, b, x, y, z, nx, ny, nz, u, v, day, tick, true, shader, dyn_light);
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
                    let (mut rgb0, a) = shade_voxel(tex, cache, b, x, y, z, nx, ny, nz, u, v, day, tick, true, shader, dyn_light);
                    if a > 20 {
                        if shader.sun_shadows && dim == Dimension::Overworld && day > 0.25 && t < 34.0 {
                            let sh = sun_shadow(
                                cache,
                                ox + dx * t + nx as f32 * 0.04,
                                oy + dy * t + ny as f32 * 0.04,
                                oz + dz * t + nz as f32 * 0.04,
                                sun_x, sun_y, sun_z,
                            );
                            rgb0[0] *= sh; rgb0[1] *= sh; rgb0[2] *= sh;
                        }
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
        prev_b = step_b;
        if tmx < tmy && tmx < tmz { t = tmx; tmx += dtx; x += step_x; nx = -step_x; ny = 0; nz = 0; }
        else if tmy < tmz { t = tmy; tmy += dty; y += step_y; ny = -step_y; nx = 0; nz = 0; }
        else { t = tmz; tmz += dtz; z += step_z; nz = -step_z; nx = 0; ny = 0; }
        if t > max_dist { break; }
    }

    if hit_dist.is_infinite() && !spec_exited {
        hit_rgb = [14.0, 14.0, 18.0];
    }

    if hit_dist.is_infinite() && dim == Dimension::Overworld && spec_exited {
        if sun_y > -0.1 && (dx - sun_x).abs() < 0.062 && (dy - sun_y).abs() < 0.062 && (dz - sun_z).abs() < 0.062 {
            hit_rgb = [255.0, 246.0, 185.0];
        }
        if -sun_y > -0.1 && (dx + sun_x).abs() < 0.048 && (dy + sun_y).abs() < 0.048 && (dz + sun_z).abs() < 0.048 {
            let phase = (tick / 2400) % 4;
            if phase != 2 || (dx + sun_x) > 0.0 {
                hit_rgb = [228.0, 234.0, 246.0];
            }
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

    if !hit_dist.is_infinite() && hit_dist > 56.0 {
        let edge_fog = ((hit_dist - 56.0) / 20.0).clamp(0.0, 1.0);
        let ef = edge_fog * edge_fog;
        hit_rgb[0] = hit_rgb[0] * (1.0 - ef) + sky[0] * ef;
        hit_rgb[1] = hit_rgb[1] * (1.0 - ef) + sky[1] * ef;
        hit_rgb[2] = hit_rgb[2] * (1.0 - ef) + sky[2] * ef;
    }

    if dim == Dimension::Overworld && spec_exited && (hit_dist.is_infinite() || cam.y >= 106.0 || hit_dist > 40.0) {
        if let Some((cc, ct, ca)) = cloud_color_3d(cam, dx, dy, dz, time, day) {
            if ct < hit_dist {
                hit_rgb[0] = hit_rgb[0] * (1.0 - ca) + cc[0] * ca;
                hit_rgb[1] = hit_rgb[1] * (1.0 - ca) + cc[1] * ca;
                hit_rgb[2] = hit_rgb[2] * (1.0 - ca) + cc[2] * ca;
                hit_dist = hit_dist.min(ct);
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
        let uf = (1.0 - (-hit_dist.min(28.0) * 0.042).exp()).clamp(0.16, 0.72);
        hit_rgb[0] = hit_rgb[0] * (1.0 - uf) + 26.0 * uf;
        hit_rgb[1] = hit_rgb[1] * (1.0 - uf) + 92.0 * uf;
        hit_rgb[2] = hit_rgb[2] * (1.0 - uf) + 178.0 * uf;
    }

    // Shader post-processing (brightness, sky warmth, saturation, contrast, ACES tone map)
    if shader.sky_warmth != 0.0 || shader.brightness != 1.0 || shader.saturation != 1.0 || shader.contrast != 1.0 || shader.tonemap_aces {
        let mut r = (hit_rgb[0] / 255.0) * shader.brightness * (1.0 + shader.sky_warmth * 0.35);
        let mut g = (hit_rgb[1] / 255.0) * shader.brightness * (1.0 + shader.sky_warmth * 0.10);
        let mut b = (hit_rgb[2] / 255.0) * shader.brightness * (1.0 - shader.sky_warmth * 0.15);
        if shader.tonemap_aces {
            let aces = |v: f32| -> f32 {
                let x = v.max(0.0);
                (x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14)
            };
            r = aces(r * 1.15);
            g = aces(g * 1.15);
            b = aces(b * 1.15);
        }
        if shader.saturation != 1.0 {
            let luma = r * 0.299 + g * 0.587 + b * 0.114;
            r = luma + (r - luma) * shader.saturation;
            g = luma + (g - luma) * shader.saturation;
            b = luma + (b - luma) * shader.saturation;
        }
        if shader.contrast != 1.0 {
            r = (r - 0.5) * shader.contrast + 0.5;
            g = (g - 0.5) * shader.contrast + 0.5;
            b = (b - 0.5) * shader.contrast + 0.5;
        }
        hit_rgb = [r * 255.0, g * 255.0, b * 255.0];
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

const FOV_SCALE: f32 = 0.70;

#[inline]
fn focal_len(fb: &Framebuffer, cam: &Cam) -> f32 {
    fb.h as f32 / (2.0 * cam.fov_scale)
}

#[allow(clippy::too_many_arguments)]
pub fn render_world(
    fb: &mut Framebuffer,
    tex: &Textures,
    world: &mut World,
    cam: &Cam,
    time: f32,
    tick: u32,
    mining: Option<(i32, i32, i32, u8)>,
    shader: &ShaderConfig,
    spectator: bool,
    dyn_light: Option<(f32, f32, f32, f32)>,
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
    let scale = cam.fov_scale;
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
                let sky = sky_color(dim, dx, dy, dz, time, px as i32, py as i32);
                let (rgb, dist) = trace_ray(tex, cache, cam, dx, dy, dz, sky, day, tick, time, underwater, spectator, dim, mining, shader, dyn_light);
                let forward_cos = (dx * cam.fx + dy * cam.fy + dz * cam.fz).max(0.05);
                let pi = py * w + px;
                fb.depth[pi] = dist * forward_cos;
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
                            let sky = sky_color(dim, dx, dy, dz, time, px as i32, py as i32);
                            let (rgb, dist) = trace_ray(tex, cache, cam, dx, dy, dz, sky, day, tick, time, underwater, spectator, dim, mining, shader, dyn_light);
                            let forward_cos = (dx * cam.fx + dy * cam.fy + dz * cam.fz).max(0.05);
                            let pi = local_y * w + px;
                            depth_band[pi] = dist * forward_cos;
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
    let focal = focal_len(fb, cam);
    Some((fb.w as f32 / 2.0 + s / d * focal, fb.h as f32 / 2.0 - u / d * focal, d))
}

/// Crisp wireframe highlight around the block the player is aiming at.
pub fn draw_block_outline(fb: &mut Framebuffer, cam: &Cam, bx: i32, by: i32, bz: i32) {
    let e = 0.004;
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
                if d <= td + 0.06 {
                    fb.set(px, py, 20, 20, 24, 195);
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
        FaceArt::IronGolem => {
            if y == 2 && x >= 1 && x <= 6 { return [94, 108, 86]; } // unibrow
            if y == 3 && (x == 1 || x == 6) { return [220, 42, 42]; } // red eyes
            if y >= 5 && x >= 3 && x <= 4 { return [168, 166, 160]; }
            base
        }
        FaceArt::SnowGolem => {
            if y == 3 && (x == 1 || x == 2 || x == 5 || x == 6) { return [32, 22, 16]; }
            if y == 5 && (x == 1 || x == 6) { return [32, 22, 16]; }
            if y == 6 && x >= 2 && x <= 5 { return [32, 22, 16]; }
            base
        }
        FaceArt::Wither => {
            if y == 3 && (x == 1 || x == 2 || x == 5 || x == 6) { return alt; }
            if y == 5 && x >= 2 && x <= 5 { return [150, 195, 255]; }
            base
        }
        FaceArt::Squid => {
            if y == 4 && (x == 1 || x == 6) { return [230, 236, 250]; }
            if y == 4 && (x == 2 || x == 5) { return [18, 22, 36]; }
            base
        }
        FaceArt::Fox => {
            if y >= 5 && (x <= 1 || x >= 6) { return [242, 238, 230]; }
            if y == 3 && (x == 1 || x == 6) { return [22, 20, 24]; }
            if y == 6 && x >= 3 && x <= 4 { return [22, 20, 24]; }
            base
        }
        FaceArt::Bee => {
            if y == 3 && (x == 1 || x == 6) { return [22, 24, 30]; }
            if y == 2 && (x == 1 || x == 6) { return [130, 215, 250]; }
            if (y == 1 || y == 5) && x >= 1 && x <= 6 { return alt; }
            base
        }
        FaceArt::Phantom => {
            if y == 3 && (x == 1 || x == 2 || x == 5 || x == 6) { return alt; }
            base
        }
        FaceArt::Panda => {
            if (y == 2 || y == 3) && (x == 1 || x == 2 || x == 5 || x == 6) { return [22, 22, 26]; }
            if y == 3 && (x == 2 || x == 5) { return [240, 240, 244]; }
            if y == 5 && (x == 3 || x == 4) { return [26, 26, 30]; }
            base
        }
        FaceArt::Allay => {
            if (y == 3 || y == 4) && (x == 2 || x == 5) { return alt; }
            base
        }
        FaceArt::WanderingTrader => {
            if y <= 1 || x == 0 || x == 7 { return base; }
            if y == 2 && x >= 1 && x <= 6 { return alt; }
            if y == 3 && (x == 2 || x == 5) { return [48, 128, 72]; }
            [188, 134, 102]
        }
        FaceArt::Guardian => {
            if y >= 2 && y <= 5 && x >= 2 && x <= 5 { return [244, 240, 232]; }
            if (y == 3 || y == 4) && (x == 3 || x == 4) { return alt; }
            if y == 0 || y == 7 { return alt; }
            base
        }
        FaceArt::Warden => {
            if y == 5 && x >= 2 && x <= 5 { return alt; }
            if y == 2 && (x == 1 || x == 6) { return [14, 34, 42]; }
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
                } else if part.face == FaceArt::Steve && (shade > 0.99 || v < 0.34 || shade < 0.75) {
                    // Hair on top, back, and upper sides of Steve's head
                    if shade < 0.75 && v > 0.72 { part.col } else { part.col_alt }
                } else if part.face == FaceArt::Cow
                    && (((u * 4.0) as i32 + (v * 4.0) as i32 * 2) % 3 == 0)
                {
                    part.col_alt
                } else if part.face == FaceArt::Creeper && (((u * 6.0) as i32 + (v * 6.0) as i32) % 3 == 0) {
                    part.col_alt
                } else if part.face == FaceArt::Sheep && (((u * 5.0) as i32 ^ (v * 5.0) as i32) & 1 == 0) {
                    part.col_alt
                } else if part.col_alt == [46, 154, 174] && v < 0.36 {
                    // Steve's iconic cyan short-sleeve cuffs on arms
                    part.col_alt
                } else if v > 0.78 && part.pivot_y > 0.2 && part.pivot_y < 0.75 {
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
    airborne: bool,
    swimming: bool,
) {
    let (_, _, dist) = match project(cam, wx, wy + 0.8, wz, fb) {
        Some(v) => v,
        None => return,
    };
    if dist > 72.0 { return; }
    if matches!(kind, EntityKind::Steve) && dist < 1.8 { return; }

    // ground shadow (aligned to focal length & flush on terrain)
    let focal = focal_len(fb, cam);
    if let Some((gx, gy, gd)) = project(cam, wx, wy + 0.01, wz, fb) {
        let rw = (0.42 / gd * focal).clamp(1.0, 22.0);
        let ry_lim = (rw * 0.42).max(1.0) as i32;
        for yy in -ry_lim..=ry_lim {
            let sy_i = gy as i32 + yy;
            if sy_i < 0 || sy_i >= fb.h as i32 { continue; }
            for xx in -rw as i32..=rw as i32 {
                let sx_i = gx as i32 + xx;
                if sx_i < 0 || sx_i >= fb.w as i32 { continue; }
                if xx * xx + yy * yy * 6 > (rw * rw) as i32 { continue; }
                let td = fb.depth[sy_i as usize * fb.w + sx_i as usize];
                if td >= 74.0 || (gd - td).abs() > 0.45 { continue; }
                let a = ((1.0 - day * 0.40) * 48.0) as u8;
                fb.set(sx_i, sy_i, 10, 12, 16, a);
            }
        }
    }

    let parts = match kind {
        EntityKind::Steve => entities::steve_parts(walk, airborne, swimming),
        EntityKind::Mob(m) => entities::mob_parts(m, walk, airborne),
        EntityKind::Boat => entities::vehicle_parts(true),
        EntityKind::Minecart => entities::vehicle_parts(false),
    };
    let (sy, cy) = (yaw.sin(), yaw.cos());
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
    let size = (0.34 / dist * focal_len(fb, cam)).clamp(4.0, 26.0) as i32;
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

pub fn block_particle_color(b: u16) -> [u8; 3] {
    let atlas = Textures::atlas();
    let px = atlas.sample_face(b, 0, 1, 0, 0.42, 0.42, 0);
    if px[3] > 20 {
        [px[0], px[1], px[2]]
    } else {
        [150, 150, 150]
    }
}

pub fn draw_particle_3d(
    fb: &mut Framebuffer,
    cam: &Cam,
    x: f32,
    y: f32,
    z: f32,
    size: f32,
    color: [u8; 3],
    day: f32,
) {
    let (sx, sy, dist) = match project(cam, x, y, z, fb) { Some(v) => v, None => return };
    if dist > 42.0 { return; }
    let ps = ((size / dist * focal_len(fb, cam)).round() as i32).clamp(1, 10);
    let left = sx as i32 - ps / 2;
    let top = sy as i32 - ps / 2;
    let light = 0.45 + day * 0.55;
    let r = (color[0] as f32 * light).min(255.0) as u8;
    let g = (color[1] as f32 * light).min(255.0) as u8;
    let b = (color[2] as f32 * light).min(255.0) as u8;
    for py in 0..ps {
        let iy = top + py;
        if iy < 0 || iy >= fb.h as i32 { continue; }
        for px in 0..ps {
            let ix = left + px;
            if ix < 0 || ix >= fb.w as i32 { continue; }
            if dist >= fb.depth[iy as usize * fb.w + ix as usize] { continue; }
            fb.set(ix, iy, r, g, b, 255);
        }
    }
}

pub fn draw_beacon_beam(fb: &mut Framebuffer, cam: &Cam, bx: f32, by: f32, bz: f32, tick: u32) {
    let pulse = 0.85 + (tick as f32 * 0.14).sin() * 0.15;
    for step in 1..=28 {
        let wy = by + step as f32 * 1.4;
        let (sx, sy, dist) = match project(cam, bx, wy, bz, fb) { Some(v) => v, None => continue };
        if dist > 72.0 { continue; }
        let half_w = ((0.22 / dist * focal_len(fb, cam)).round() as i32).clamp(1, 8);
        let seg_h = ((1.5 / dist * focal_len(fb, cam)).ceil() as i32).clamp(2, 24);
        let top = sy as i32 - seg_h / 2;
        for py in 0..seg_h {
            let iy = top + py;
            if iy < 0 || iy >= fb.h as i32 { continue; }
            for px in -half_w..=half_w {
                let ix = sx as i32 + px;
                if ix < 0 || ix >= fb.w as i32 { continue; }
                if dist >= fb.depth[iy as usize * fb.w + ix as usize] { continue; }
                let core = px.abs() <= half_w / 2;
                let (r, g, b) = if core {
                    ((215.0 * pulse).min(255.0) as u8, (248.0 * pulse).min(255.0) as u8, 255)
                } else {
                    (90, (220.0 * pulse).min(255.0) as u8, 255)
                };
                fb.set(ix, iy, r, g, b, 210);
            }
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
    if (crate::items::SPAWN_IRON_GOLEM..=crate::items::SPAWN_PHANTOM).contains(&item)
        || (crate::items::SPAWN_PANDA..=crate::items::SPAWN_WARDEN).contains(&item)
    {
        let mut c = atlas.sample_item(66, x, y);
        if c[3] > 20 {
            let spot = ((x + y * 3) % 5) == 0;
            c = match item {
                crate::items::SPAWN_IRON_GOLEM => if spot { [86, 138, 64, c[3]] } else { [214, 214, 218, c[3]] },
                crate::items::SPAWN_SNOW_GOLEM => if spot { [224, 118, 28, c[3]] } else { [240, 248, 255, c[3]] },
                crate::items::SPAWN_WITHER => if spot { [120, 180, 255, c[3]] } else { [34, 34, 42, c[3]] },
                crate::items::SPAWN_SQUID => if spot { [22, 34, 68, c[3]] } else { [54, 76, 128, c[3]] },
                crate::items::SPAWN_FOX => if spot { [246, 242, 234, c[3]] } else { [220, 108, 34, c[3]] },
                crate::items::SPAWN_BEE => if spot { [36, 30, 26, c[3]] } else { [244, 198, 42, c[3]] },
                crate::items::SPAWN_PANDA => if spot { [24, 24, 28, c[3]] } else { [238, 238, 236, c[3]] },
                crate::items::SPAWN_ALLAY => if spot { [220, 250, 255, c[3]] } else { [70, 214, 244, c[3]] },
                crate::items::SPAWN_WANDERING_TRADER => if spot { [224, 186, 62, c[3]] } else { [48, 86, 158, c[3]] },
                crate::items::SPAWN_GUARDIAN => if spot { [228, 128, 58, c[3]] } else { [74, 152, 140, c[3]] },
                crate::items::SPAWN_WARDEN => if spot { [56, 224, 212, c[3]] } else { [22, 54, 64, c[3]] },
                _ => if spot { [112, 248, 86, c[3]] } else { [66, 78, 126, c[3]] },
            };
        }
        return c;
    }
    if item == crate::items::MUSIC_DISC_CAT || item == crate::items::MUSIC_DISC_PIGSTEP {
        let dx = x as i32 - 8;
        let dy = y as i32 - 8;
        let d2 = dx * dx + dy * dy;
        if d2 > 42 || d2 < 2 { return [0, 0, 0, 0]; }
        if d2 <= 8 {
            return if item == crate::items::MUSIC_DISC_CAT { [74, 214, 96, 255] } else { [238, 182, 54, 255] };
        }
        return if d2 % 4 == 0 { [48, 48, 56, 255] } else { [22, 22, 26, 255] };
    }
    if (crate::items::STONE_SWORD..=crate::items::POTION_JUMP).contains(&item) {
        let (base_id, tint): (u16, Option<[u8; 3]>) = match item {
            crate::items::STONE_SWORD => (crate::items::IRON_SWORD, Some([152, 152, 156])),
            crate::items::STONE_AXE => (crate::items::IRON_AXE, Some([152, 152, 156])),
            crate::items::STONE_SHOVEL => (crate::items::IRON_SHOVEL, Some([152, 152, 156])),
            crate::items::DIAMOND_SHOVEL => (crate::items::IRON_SHOVEL, Some([86, 228, 224])),
            crate::items::GOLD_PICK => (crate::items::IRON_PICK, Some([246, 212, 62])),
            crate::items::GOLD_SWORD => (crate::items::IRON_SWORD, Some([246, 212, 62])),
            crate::items::GOLD_AXE => (crate::items::IRON_AXE, Some([246, 212, 62])),
            crate::items::GOLD_SHOVEL => (crate::items::IRON_SHOVEL, Some([246, 212, 62])),
            crate::items::GOLD_HOE => (crate::items::IRON_HOE, Some([246, 212, 62])),
            crate::items::GOLD_HELMET => (crate::items::IRON_HELMET, Some([246, 212, 62])),
            crate::items::GOLD_CHESTPLATE => (crate::items::IRON_CHESTPLATE, Some([246, 212, 62])),
            crate::items::GOLD_LEGGINGS => (crate::items::IRON_LEGGINGS, Some([246, 212, 62])),
            crate::items::GOLD_BOOTS => (crate::items::IRON_BOOTS, Some([246, 212, 62])),
            crate::items::NETHERITE_AXE => (crate::items::DIAMOND_AXE, Some([74, 66, 74])),
            crate::items::NETHERITE_SHOVEL => (crate::items::IRON_SHOVEL, Some([74, 66, 74])),
            crate::items::NETHERITE_HOE => (crate::items::DIAMOND_HOE, Some([74, 66, 74])),
            crate::items::WHEAT_SEEDS => (world::WHEAT, Some([148, 198, 72])),
            crate::items::BEETROOT => (crate::items::CARROT, Some([188, 42, 64])),
            crate::items::BAKED_POTATO => (crate::items::BREAD, Some([216, 166, 84])),
            crate::items::PUMPKIN_PIE => (crate::items::COOKIE, Some([224, 134, 48])),
            crate::items::GOLDEN_CARROT => (crate::items::CARROT, Some([246, 210, 62])),
            crate::items::PAPER => (crate::items::MAP, Some([238, 238, 232])),
            crate::items::BOOK => (crate::items::ENCHANTED_BOOK, Some([158, 92, 52])),
            crate::items::SUGAR => (crate::items::BONE_MEAL, Some([248, 248, 250])),
            crate::items::BAMBOO => (crate::items::STICK, Some([112, 188, 62])),
            crate::items::FISHING_ROD => (crate::items::BOW, Some([168, 128, 84])),
            crate::items::SPYGLASS => (crate::items::BLAZE_ROD, Some([192, 138, 64])),
            crate::items::POTION_STRENGTH => (crate::items::POTION_HEAL, Some([178, 42, 42])),
            crate::items::POTION_REGEN => (crate::items::POTION_HEAL, Some([228, 102, 178])),
            crate::items::POTION_JUMP => (crate::items::POTION_HEAL, Some([118, 236, 92])),
            _ => (crate::items::STICK, None),
        };
        let mut c = item_pixel(base_id, x, y, tick);
        if c[3] > 20 {
            if let Some(t) = tint {
                // Keep wooden stick handles brown on tools while tinting the metal/head
                let is_handle = matches!(
                    item,
                    crate::items::STONE_SWORD | crate::items::STONE_AXE | crate::items::STONE_SHOVEL
                    | crate::items::DIAMOND_SHOVEL | crate::items::GOLD_PICK | crate::items::GOLD_SWORD
                    | crate::items::GOLD_AXE | crate::items::GOLD_SHOVEL | crate::items::GOLD_HOE
                    | crate::items::NETHERITE_AXE | crate::items::NETHERITE_SHOVEL | crate::items::NETHERITE_HOE
                ) && c[0] > c[2] + 25 && c[1] < 130 && (x + (15 - y)) < 14;
                if !is_handle {
                    c[0] = ((c[0] as u16 * t[0] as u16) / 210).min(255) as u8;
                    c[1] = ((c[1] as u16 * t[1] as u16) / 210).min(255) as u8;
                    c[2] = ((c[2] as u16 * t[2] as u16) / 210).min(255) as u8;
                }
            }
        }
        return c;
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
        world::SOUL_TORCH => Some(76),
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
        if item == world::SOUL_TORCH && c[3] > 20 && y < 7 {
            c = [85, 225, 255, c[3]];
        }
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
        world::TORCH | world::SOUL_TORCH | world::REDSTONE_TORCH | world::REDSTONE_DUST | world::LEVER
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
        let ry = size as f32 * 0.22;
        let h = size as f32 * 0.44;
        let top_y = -h * 0.5;
        for py in 0..size {
            for px in 0..size {
                let fx = px as f32 + 0.5 - cx;
                let fy = py as f32 + 0.5 - cy;
                // Isometric diamond top face centered at (0, top_y)
                let ty = fy - top_y;
                if (fx / rx).abs() + (ty / ry).abs() <= 1.0 {
                    let u = ((fx / rx + ty / ry) * 0.5 + 0.5).clamp(0.0, 0.999);
                    let v = ((ty / ry - fx / rx) * 0.5 + 0.5).clamp(0.0, 0.999);
                    let c = atlas.sample_kind(top_k, u, v, tick, anim);
                    if c[3] > 20 {
                        let shade = 1.06;
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
                // Left side face (-rx..0) seamlessly flush with top diamond bottom-left edge
                if fx >= -rx && fx < 0.0 {
                    let top_edge = top_y + ((fx + rx) / rx) * ry;
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
                // Right side face (0..rx) seamlessly flush with top diamond bottom-right edge
                if fx >= 0.0 && fx <= rx {
                    let top_edge = top_y + ((rx - fx) / rx) * ry;
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
        [0, 1, 1, 0, 0, 0, 1, 1, 0],
        [1, 2, 2, 1, 0, 1, 2, 2, 1],
        [1, 3, 3, 2, 1, 2, 2, 2, 1],
        [1, 3, 2, 2, 2, 2, 2, 2, 1],
        [1, 2, 2, 2, 2, 2, 2, 2, 1],
        [0, 1, 2, 2, 2, 2, 2, 1, 0],
        [0, 0, 1, 2, 2, 2, 1, 0, 0],
        [0, 0, 0, 1, 2, 1, 0, 0, 0],
        [0, 0, 0, 0, 1, 0, 0, 0, 0],
    ];
    for (r, bits) in rows.iter().enumerate() {
        for (c, &v) in bits.iter().enumerate() {
            if v == 1 {
                fb.set(x + c as i32, y + r as i32, 18, 10, 12, 255);
            } else if v >= 2 {
                let filled = state == 2 || (state == 1 && c < 4);
                if filled {
                    if v == 3 {
                        fb.set(x + c as i32, y + r as i32, 255, 188, 188, 255);
                    } else if c >= 6 || r >= 6 {
                        fb.set(x + c as i32, y + r as i32, 178, 18, 24, 255);
                    } else {
                        fb.set(x + c as i32, y + r as i32, 238, 32, 38, 255);
                    }
                } else {
                    fb.set(x + c as i32, y + r as i32, 48, 38, 40, 235);
                }
            }
        }
    }
}

fn drumstick(fb: &mut Framebuffer, x: i32, y: i32, state: u8) {
    // 9x9 Minecraft-style drumstick: 1=outline, 2=meat, 3=meat highlight, 4=bone
    let rows = [
        [0, 0, 0, 1, 1, 1, 0, 0, 0],
        [0, 0, 1, 3, 3, 2, 1, 0, 0],
        [0, 1, 3, 2, 2, 2, 2, 1, 0],
        [0, 1, 2, 2, 2, 2, 2, 1, 0],
        [0, 1, 2, 2, 2, 2, 1, 0, 0],
        [0, 0, 1, 2, 2, 4, 1, 0, 0],
        [0, 0, 0, 1, 1, 4, 4, 1, 0],
        [0, 0, 0, 0, 0, 1, 4, 4, 1],
        [0, 0, 0, 0, 0, 0, 1, 1, 0],
    ];
    for (r, bits) in rows.iter().enumerate() {
        for (c, &v) in bits.iter().enumerate() {
            if v == 1 {
                fb.set(x + c as i32, y + r as i32, 22, 14, 10, 255);
            } else if v >= 2 {
                let filled = state == 2 || (state == 1 && (c + r) <= 7);
                if !filled {
                    fb.set(x + c as i32, y + r as i32, 48, 40, 36, 230);
                } else if v == 4 {
                    fb.set(x + c as i32, y + r as i32, 238, 228, 208, 255);
                } else if v == 3 {
                    fb.set(x + c as i32, y + r as i32, 234, 168, 92, 255);
                } else {
                    fb.set(x + c as i32, y + r as i32, 194, 114, 52, 255);
                }
            }
        }
    }
}

fn armor_icon(fb: &mut Framebuffer, x: i32, y: i32, state: u8) {
    let rows = [
        [0, 1, 1, 0, 0, 0, 1, 1, 0],
        [1, 2, 2, 1, 1, 1, 2, 2, 1],
        [1, 2, 3, 2, 2, 2, 2, 2, 1],
        [1, 1, 2, 2, 2, 2, 2, 1, 1],
        [0, 1, 2, 2, 2, 2, 2, 1, 0],
        [0, 1, 2, 2, 2, 2, 2, 1, 0],
        [0, 1, 2, 2, 2, 2, 2, 1, 0],
        [0, 0, 1, 1, 1, 1, 1, 0, 0],
    ];
    for (r, bits) in rows.iter().enumerate() {
        for (c, &v) in bits.iter().enumerate() {
            if v == 1 {
                fb.set(x + c as i32, y + r as i32, 22, 24, 30, 255);
            } else if v >= 2 {
                let filled = state == 2 || (state == 1 && c < 4);
                if !filled {
                    fb.set(x + c as i32, y + r as i32, 42, 46, 54, 220);
                } else if v == 3 {
                    fb.set(x + c as i32, y + r as i32, 225, 234, 248, 255);
                } else {
                    fb.set(x + c as i32, y + r as i32, 168, 182, 206, 255);
                }
            }
        }
    }
}

pub const SLOT: i32 = 20;
pub const HOT_Y: i32 = 246;
pub fn hotbar_x(i: usize) -> i32 { (W as i32 - 182) / 2 + 1 + i as i32 * 20 }

fn rot_3d_point(p: [f32; 3], yaw: f32, pitch: f32, roll: f32, t: [f32; 3]) -> [f32; 3] {
    let (sr, cr) = (roll.sin(), roll.cos());
    let (x1, y1, z1) = (p[0] * cr - p[1] * sr, p[0] * sr + p[1] * cr, p[2]);
    let (sp, cp) = (pitch.sin(), pitch.cos());
    let (x2, y2, z2) = (x1, y1 * cp - z1 * sp, y1 * sp + z1 * cp);
    let (sy, cy) = (yaw.sin(), yaw.cos());
    [x2 * cy + z2 * sy + t[0], y2 + t[1], -x2 * sy + z2 * cy + t[2]]
}

fn fill_textured_quad_3d<F: Fn(f32, f32) -> [u8; 4]>(
    fb: &mut Framebuffer,
    pts: [[f32; 3]; 4],
    shade: f32,
    sample: F,
) {
    let focal = 245.0;
    let cx = W as f32 * 0.5;
    let cy = H as f32 * 0.5;
    let mut s = [[0.0f32; 2]; 4];
    let mut min_x = W as i32;
    let mut max_x = -1i32;
    let mut min_y = H as i32;
    let mut max_y = -1i32;
    for i in 0..4 {
        let z = pts[i][2].max(0.08);
        let sx = cx + (pts[i][0] / z) * focal;
        let sy = cy - (pts[i][1] / z) * focal;
        s[i] = [sx, sy];
        min_x = min_x.min(sx.floor() as i32);
        max_x = max_x.max(sx.ceil() as i32);
        min_y = min_y.min(sy.floor() as i32);
        max_y = max_y.max(sy.ceil() as i32);
    }
    min_x = min_x.max(0);
    max_x = max_x.min(W as i32 - 1);
    min_y = min_y.max(0);
    max_y = max_y.min(H as i32 - 1);
    // Split quad (0,1,2,3) into triangles (0,1,2) with UVs (0,0),(1,0),(1,1) and (0,2,3) with (0,0),(1,1),(0,1)
    let tris = [
        (s[0], s[1], s[2], [0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0]),
        (s[0], s[2], s[3], [0.0f32, 0.0], [1.0, 1.0], [0.0, 1.0]),
    ];
    for (p0, p1, p2, uv0, uv1, uv2) in tris {
        let area = (p1[0] - p0[0]) * (p2[1] - p0[1]) - (p1[1] - p0[1]) * (p2[0] - p0[0]);
        if area.abs() < 0.001 { continue; }
        let inv = 1.0 / area;
        for py in min_y..=max_y {
            let fy = py as f32 + 0.5;
            for px in min_x..=max_x {
                let fx = px as f32 + 0.5;
                let w0 = ((p1[0] - fx) * (p2[1] - fy) - (p1[1] - fy) * (p2[0] - fx)) * inv;
                let w1 = ((p2[0] - fx) * (p0[1] - fy) - (p2[1] - fy) * (p0[0] - fx)) * inv;
                let w2 = 1.0 - w0 - w1;
                if w0 >= -0.005 && w1 >= -0.005 && w2 >= -0.005 {
                    let u = (w0 * uv0[0] + w1 * uv1[0] + w2 * uv2[0]).clamp(0.0, 0.999);
                    let v = (w0 * uv0[1] + w1 * uv1[1] + w2 * uv2[1]).clamp(0.0, 0.999);
                    let c = sample(u, v);
                    if c[3] > 20 {
                        fb.set(
                            px,
                            py,
                            (c[0] as f32 * shade).min(255.0) as u8,
                            (c[1] as f32 * shade).min(255.0) as u8,
                            (c[2] as f32 * shade).min(255.0) as u8,
                            c[3],
                        );
                    }
                }
            }
        }
    }
}

fn draw_box_3d_screen<F: Fn(usize, f32, f32) -> [u8; 4]>(
    fb: &mut Framebuffer,
    hx: f32,
    hy: f32,
    hz: f32,
    yaw: f32,
    pitch: f32,
    roll: f32,
    t: [f32; 3],
    sample_face: F,
) {
    let corners = [
        [-hx, -hy, -hz], [ hx, -hy, -hz], [ hx,  hy, -hz], [-hx,  hy, -hz],
        [-hx, -hy,  hz], [ hx, -hy,  hz], [ hx,  hy,  hz], [-hx,  hy,  hz],
    ];
    let v: Vec<[f32; 3]> = corners.iter().map(|&c| rot_3d_point(c, yaw, pitch, roll, t)).collect();
    // 6 faces: (idx[4], shade, face_id)
    let faces = [
        ([3, 2, 6, 7], 1.00f32, 0usize), // +Y top
        ([0, 1, 5, 4], 0.58f32, 1usize), // -Y bottom
        ([7, 4, 0, 3], 0.84f32, 2usize), // -X left (facing player)
        ([2, 1, 5, 6], 0.72f32, 3usize), // +X right
        ([3, 2, 1, 0], 0.78f32, 4usize), // -Z front (toward camera)
        ([6, 7, 4, 5], 0.65f32, 5usize), // +Z back
    ];
    let mut order: Vec<(f32, usize)> = Vec::with_capacity(6);
    for (fi, (idx, _, _)) in faces.iter().enumerate() {
        let cz = (v[idx[0]][2] + v[idx[1]][2] + v[idx[2]][2] + v[idx[3]][2]) * 0.25;
        order.push((cz, fi));
    }
    order.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    for (_, fi) in order {
        let (idx, shade, f_id) = faces[fi];
        let pts = [v[idx[0]], v[idx[1]], v[idx[2]], v[idx[3]]];
        fill_textured_quad_3d(fb, pts, shade, |u, vv| sample_face(f_id, u, vv));
    }
}

fn steve_arm_texel(v_along: f32, u_across: f32, face_id: usize) -> [u8; 4] {
    let tx = (u_across * 4.0).floor() as i32;
    let ty = (v_along * 12.0).floor() as i32;
    let n = (th(tx, ty + face_id as i32 * 13, 91) % 16) as i32 - 8;
    if v_along > 0.62 {
        // Cyan shirt sleeve on back 38% of arm
        let r = (46 + n).clamp(24, 72) as u8;
        let g = (156 + n).clamp(110, 185) as u8;
        let b = (176 + n).clamp(130, 205) as u8;
        [r, g, b, 255]
    } else {
        // Steve forearm skin with natural pixel variation
        let r = (224 + n).clamp(175, 245) as u8;
        let g = (172 + n).clamp(130, 198) as u8;
        let b = (132 + n).clamp(95, 160) as u8;
        [r, g, b, 255]
    }
}

pub fn draw_held_item(fb: &mut Framebuffer, item: u16, tick: u32, walk_bob: f32, swing: f32) {
    let sw = swing.sin();
    let bob_x = (walk_bob * 0.85).cos() * 0.025;
    let bob_y = (walk_bob * 1.7).sin().abs() * 0.024;
    if item == 0 {
        // Real 3D perspective-projected Steve arm in classic Minecraft Java bottom-right pose
        let tx = 0.56 + bob_x - sw * 0.34;
        let ty = -0.52 - bob_y + (swing * 0.6).sin() * 0.14 - sw * 0.08;
        let tz = 1.06 - sw * 0.18;
        let yaw = -0.72 - sw * 0.52;
        let pitch = 0.52 - sw * 0.58;
        let roll = 0.22 + sw * 0.26;
        draw_box_3d_screen(fb, 0.14, 0.14, 0.44, yaw, pitch, roll, [tx, ty, tz], |face_id, u, v| {
            // Map length along Z axis (v for side/top faces)
            let v_along = match face_id {
                0 | 1 => v,
                2 | 3 => u,
                _ => 0.0, // front fist end is skin
            };
            steve_arm_texel(v_along, u, face_id)
        });
    } else if item < 256 && !is_flat_block_icon(item) {
        // Real 3D perspective-projected held block in Minecraft Java bottom-right pose
        let atlas = Textures::atlas();
        let anim = item == world::WATER || item == world::LAVA || item == world::NETHER_PORTAL;
        let top_k = Textures::face_kind(item, 0, 1, 0);
        let left_k = Textures::face_kind(item, -1, 0, 0);
        let right_k = Textures::face_kind(item, 0, 0, 1);
        let tx = 0.48 + bob_x - sw * 0.28;
        let ty = -0.44 - bob_y - sw * 0.16;
        let tz = 1.05 - sw * 0.16;
        let yaw = -0.76 - sw * 0.44;
        let pitch = 0.24 - sw * 0.42;
        let roll = 0.04 + sw * 0.18;
        draw_box_3d_screen(fb, 0.21, 0.21, 0.21, yaw, pitch, roll, [tx, ty, tz], |face_id, u, v| {
            let k = match face_id {
                0 | 1 => top_k,
                2 | 3 => left_k,
                _ => right_k,
            };
            atlas.sample_kind(k, u, v, tick, anim)
        });
    } else {
        // Held tool/sword/item with subtle 3D arm grip + extruded item sprite
        let tx = 0.64 + bob_x - sw * 0.30;
        let ty = -0.60 - bob_y - sw * 0.12;
        let tz = 1.12 - sw * 0.15;
        draw_box_3d_screen(fb, 0.12, 0.12, 0.38, -0.68 - sw * 0.45, 0.48 - sw * 0.50, 0.20, [tx, ty, tz], |face_id, u, v| {
            let v_along = match face_id { 0 | 1 => v, 2 | 3 => u, _ => 0.0 };
            steve_arm_texel(v_along, u, face_id)
        });
        let bx = (370.0 + bob_x * 160.0 - sw * 28.0) as i32;
        let by = (146.0 + bob_y * 160.0 + sw * 18.0) as i32;
        let sz = 54i32;
        for depth in (1..=4).rev() {
            for py in 0..sz {
                for px in 0..sz {
                    let c = item_pixel(item, (px as u32 * 16 / sz as u32).min(15), (py as u32 * 16 / sz as u32).min(15), tick);
                    if c[3] > 20 {
                        fb.set(bx + px + depth, by + py + depth, (c[0] as f32 * 0.56) as u8, (c[1] as f32 * 0.56) as u8, (c[2] as f32 * 0.56) as u8, c[3]);
                    }
                }
            }
        }
        draw_item_icon(fb, bx, by, item, sz, tick);
    }
}

pub fn draw_minimap(fb: &mut Framebuffer, world: &mut World, px: f32, pz: f32, yaw: f32) {
    let map_sz = 56i32;
    let x0 = W as i32 - map_sz - 10;
    let y0 = 10i32;
    // Parchment map frame
    fb.rect(x0 - 4, y0 - 4, map_sz + 8, map_sz + 8, [206, 184, 136, 235]);
    fb.frame(x0 - 4, y0 - 4, map_sz + 8, map_sz + 8, [104, 78, 44, 255]);
    fb.frame(x0 - 1, y0 - 1, map_sz + 2, map_sz + 2, [142, 114, 72, 255]);
    let cx = px.floor() as i32;
    let cz = pz.floor() as i32;
    let step = 2i32;
    for my in 0..map_sz {
        for mx in 0..map_sz {
            let wx = cx + (mx - map_sz / 2) * step;
            let wz = cz + (my - map_sz / 2) * step;
            let h = world.highest_ground(wx, wz);
            let top = world.get(wx, h, wz);
            let col = if h < world::SEA || top == world::WATER {
                [56, 104, 198]
            } else if top == world::SAND || top == world::RED_SAND {
                [218, 202, 142]
            } else if top == world::SNOW || top == world::ICE {
                [232, 240, 248]
            } else if top == world::LAVA {
                [236, 98, 24]
            } else if top == world::LEAVES || top == world::SPRUCE_LEAVES || top == world::JUNGLE_LEAVES {
                [54, 118, 42]
            } else if top == world::STONE || top == world::COBBLE || top == world::END_STONE {
                [132, 132, 138]
            } else {
                match world.biome(wx, wz) {
                    Biome::Desert | Biome::Badlands => [216, 196, 138],
                    Biome::Snowy => [226, 236, 244],
                    Biome::Swamp => [74, 104, 64],
                    Biome::Jungle => [68, 152, 46],
                    _ => [92, 152, 64],
                }
            };
            let shade = if ((wx + wz) & 3) == 0 { 0.92 } else { 1.0 };
            fb.set(x0 + mx, y0 + my, (col[0] as f32 * shade) as u8, (col[1] as f32 * shade) as u8, (col[2] as f32 * shade) as u8, 245);
        }
    }
    let pmx = x0 + map_sz / 2;
    let pmy = y0 + map_sz / 2;
    fb.rect(pmx - 1, pmy - 1, 3, 3, [255, 255, 255, 255]);
    let dir_x = (yaw.sin() * 3.0).round() as i32;
    let dir_y = (-yaw.cos() * 3.0).round() as i32;
    fb.set(pmx + dir_x, pmy + dir_y, 240, 42, 42, 255);
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
    wither_hp: Option<f32>,
    warden_hp: Option<f32>,
    beacon_active: bool,
    pos: (f32, f32, f32),
    fps: u32,
    camera: u8,
    time: f32,
    mode: GameMode,
    show_debug: bool,
) {
    let hb_left = hotbar_x(0);
    if mode != GameMode::Spectator {
        if offhand != 0 {
            let ox = hb_left - 28;
            fb.rect(ox - 1, HOT_Y - 1, 22, 22, [28, 28, 30, 220]);
            fb.rect(ox, HOT_Y, 20, 20, [48, 48, 52, 175]);
            fb.frame(ox - 1, HOT_Y - 1, 22, 22, if shield_blocking { [220, 220, 255, 255] } else { [20, 20, 22, 240] });
            draw_item_icon(fb, ox + 2, HOT_Y + 2, offhand, 16, tick);
        }
        // 100% Classic Minecraft Java Edition 182x22 hotbar
        let bar_x = hb_left - 1;
        let bar_y = HOT_Y - 1;
        let bar_w = 182;
        fb.rect(bar_x, bar_y, bar_w, 22, [26, 26, 28, 215]);
        fb.frame(bar_x, bar_y, bar_w, 22, [18, 18, 20, 245]);
        fb.rect(bar_x + 1, bar_y + 1, bar_w - 2, 1, [68, 68, 72, 200]);
        fb.rect(bar_x + 1, bar_y + 20, bar_w - 2, 1, [68, 68, 72, 200]);
        for i in 0..9 {
            let x = hotbar_x(i);
            fb.rect(x + 1, HOT_Y + 1, 18, 18, [44, 44, 48, 165]);
            if i > 0 {
                fb.rect(x, HOT_Y + 1, 1, 18, [72, 72, 76, 210]);
            }
            if hot[i] != 0 { draw_item_icon(fb, x + 2, HOT_Y + 2, hot[i], 16, tick); }
            if counts[i] > 1 {
                let cs = counts[i].to_string();
                text_shadow(fb, x + 19 - cs.len() as i32 * 6, HOT_Y + 13, &cs, [255, 255, 255, 255], 1);
            }
        }
        // Classic Java 24x24 white-bevelled active slot selector protruding 1px outside the hotbar
        let sx = hotbar_x(selected);
        fb.frame(sx - 2, HOT_Y - 2, 24, 24, [22, 22, 24, 245]);
        fb.frame(sx - 1, HOT_Y - 1, 22, 22, [234, 236, 240, 255]);
        fb.frame(sx, HOT_Y, 20, 20, [190, 192, 196, 255]);
        fb.frame(sx + 1, HOT_Y + 1, 18, 18, [48, 48, 52, 215]);
        if mode == GameMode::Survival {
            if armor > 0 {
                for i in 0..10 {
                    let threshold = (i as u32 + 1) * 2;
                    let st = if armor >= threshold { 2 } else if armor >= threshold - 1 { 1 } else { 0 };
                    armor_icon(fb, bar_x + i as i32 * 8, HOT_Y - 28, st);
                }
            }
            for i in 0..10 {
                let threshold = (i as f32 + 1.0) * 2.0;
                let st = if health >= threshold { 2 } else if health >= threshold - 1.0 { 1 } else { 0 };
                heart(fb, bar_x + i * 8, HOT_Y - 18, st);
            }
            for i in 0..10 {
                let threshold = (i as f32 + 1.0) * 2.0;
                let st = if hunger >= threshold { 2 } else if hunger >= threshold - 1.0 { 1 } else { 0 };
                drumstick(fb, bar_x + bar_w - 9 - i * 8, HOT_Y - 18, st);
            }
            if air < 9.95 {
                let bubbles = air.ceil().clamp(0.0, 10.0) as i32;
                for i in 0..bubbles {
                    let bx = bar_x + bar_w - 9 - i * 8;
                    let by = HOT_Y - 28;
                    fb.rect(bx, by, 7, 7, [40, 140, 240, 220]);
                    fb.rect(bx + 1, by + 1, 2, 2, [215, 245, 255, 240]);
                }
            }
            let needed = (7 + level * 2).max(1);
            let xp_fill = ((xp as f32 / needed as f32).clamp(0.0, 1.0) * 180.0) as i32;
            fb.rect(bar_x, HOT_Y - 7, bar_w, 5, [16, 22, 16, 235]);
            fb.frame(bar_x, HOT_Y - 7, bar_w, 5, [14, 16, 14, 255]);
            if xp_fill > 0 {
                fb.rect(bar_x + 1, HOT_Y - 6, xp_fill, 3, [128, 255, 32, 255]);
                fb.rect(bar_x + 1, HOT_Y - 4, xp_fill, 1, [88, 200, 20, 255]);
            }
            for t in 1..18 {
                fb.rect(bar_x + 1 + t * 10, HOT_Y - 6, 1, 3, [20, 28, 20, 140]);
            }
            if level > 0 {
                let ls = level.to_string();
                let lx = W as i32 / 2 - (ls.len() as i32 * 3);
                let ly = HOT_Y - 14;
                for &(ox, oy) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    text(fb, lx + ox, ly + oy, &ls, [0, 0, 0, 255], 1);
                }
                text(fb, lx, ly, &ls, [128, 255, 32, 255], 1);
            }
        }
        if shield_blocking {
            fb.rect(10, H as i32 - 58, 34, 48, [130, 96, 60, 220]);
            fb.frame(10, H as i32 - 58, 34, 48, [190, 196, 205, 255]);
            text_shadow(fb, 14, H as i32 - 38, "BLK", [255, 255, 255, 255], 1);
        }
        if potion_fire > 0.0 || potion_speed > 0.0 || beacon_active {
            let mut py = 32;
            if potion_fire > 0.0 {
                text_shadow(fb, 8, py, &format!("FIRE RES {:02}s", potion_fire as i32), [255, 165, 65, 255], 1);
                py += 10;
            }
            if potion_speed > 0.0 {
                text_shadow(fb, 8, py, &format!("SPEED {:02}s", potion_speed as i32), [110, 220, 255, 255], 1);
                py += 10;
            }
            if beacon_active {
                text_shadow(fb, 8, py, "BEACON: HASTE + STR", [110, 245, 235, 255], 1);
            }
        }
        if burn > 0.0 {
            let flicker = ((tick / 3) % 2) as i32 * 4;
            fb.rect(0, H as i32 - 22 - flicker, W as i32, 22 + flicker, [240, 96, 18, 85]);
        }
    }

    let cx = W as i32 / 2; let cy = H as i32 / 2;
    fb.rect(cx - 4, cy, 9, 1, [235, 235, 235, 210]);
    fb.rect(cx, cy - 4, 1, 9, [235, 235, 235, 210]);

    if show_debug {
        text_shadow(fb, 8, 8, &format!("{} ({})  XYZ {:.0} {:.0} {:.0}", biome.name(), dim.name(), pos.0, pos.1, pos.2), [240, 240, 240, 255], 1);
        text_shadow(fb, 8, 18, &format!("FPS {}  CAM {}  TIME {:02}:00  [{}]", fps, camera, ((time * 24.0) as i32) % 24, mode.name()), [200, 230, 255, 255], 1);
    }

    if let Some(hp) = dragon_hp {
        let bw = 180;
        let bx = W as i32 / 2 - bw / 2;
        text_shadow(fb, W as i32 / 2 - 36, 28, "Ender Dragon", [228, 130, 248, 255], 1);
        fb.rect(bx, 37, bw, 5, [34, 18, 42, 230]);
        let fill = ((hp / 200.0).clamp(0.0, 1.0) * bw as f32) as i32;
        fb.rect(bx, 37, fill, 5, [198, 56, 224, 255]);
    } else if let Some(whp) = wither_hp {
        let bw = 180;
        let bx = W as i32 / 2 - bw / 2;
        text_shadow(fb, W as i32 / 2 - 18, 28, "Wither", [218, 185, 255, 255], 1);
        fb.rect(bx, 37, bw, 5, [28, 18, 38, 230]);
        let fill = ((whp / 300.0).clamp(0.0, 1.0) * bw as f32) as i32;
        fb.rect(bx, 37, fill, 5, [155, 85, 240, 255]);
    } else if let Some(wdhp) = warden_hp {
        let bw = 180;
        let bx = W as i32 / 2 - bw / 2;
        text_shadow(fb, W as i32 / 2 - 18, 28, "Warden", [120, 245, 235, 255], 1);
        fb.rect(bx, 37, bw, 5, [12, 28, 34, 230]);
        let fill = ((wdhp / 500.0).clamp(0.0, 1.0) * bw as f32) as i32;
        fb.rect(bx, 37, fill, 5, [52, 218, 206, 255]);
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

/// Composite 480x270 logical UI framebuffer cleanly onto the 960x540 HD output buffer.
pub fn composite_ui_2x(ui: &Framebuffer, out: &mut Framebuffer) {
    let sx_scale = (out.w / ui.w.max(1)).max(1);
    let sy_scale = (out.h / ui.h.max(1)).max(1);
    for uy in 0..ui.h {
        for ux in 0..ui.w {
            let si = (uy * ui.w + ux) * 4;
            let a = ui.rgba[si + 3];
            if a == 0 { continue; }
            let r = ui.rgba[si];
            let g = ui.rgba[si + 1];
            let b = ui.rgba[si + 2];
            let oy0 = uy * sy_scale;
            let ox0 = ux * sx_scale;
            if a == 255 {
                for dy in 0..sy_scale {
                    let row = (oy0 + dy) * out.w;
                    for dx in 0..sx_scale {
                        let di = (row + ox0 + dx) * 4;
                        out.rgba[di] = r;
                        out.rgba[di + 1] = g;
                        out.rgba[di + 2] = b;
                        out.rgba[di + 3] = 255;
                    }
                }
            } else {
                let t = a as i32;
                let inv = 255 - t;
                for dy in 0..sy_scale {
                    let row = (oy0 + dy) * out.w;
                    for dx in 0..sx_scale {
                        let di = (row + ox0 + dx) * 4;
                        out.rgba[di] = ((out.rgba[di] as i32 * inv + r as i32 * t) / 255) as u8;
                        out.rgba[di + 1] = ((out.rgba[di + 1] as i32 * inv + g as i32 * t) / 255) as u8;
                        out.rgba[di + 2] = ((out.rgba[di + 2] as i32 * inv + b as i32 * t) / 255) as u8;
                        out.rgba[di + 3] = 255;
                    }
                }
            }
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
    // Classic Minecraft stone-textured GUI button with bevel & hover highlight
    for r in 0..h {
        for c in 0..w {
            let n = (th(c / 2, r / 2, 313) % 15) as i32 - 7;
            let base = if hover { 120 } else { 116 };
            let shade = base + n - (r * 14 / h.max(1));
            let (rr, gg, bb) = if hover {
                ((shade + 12).clamp(40, 220) as u8, (shade + 20).clamp(40, 220) as u8, (shade + 56).clamp(60, 245) as u8)
            } else {
                (shade.clamp(40, 210) as u8, shade.clamp(40, 210) as u8, (shade + 3).clamp(40, 210) as u8)
            };
            fb.set(x + c, y + r, rr, gg, bb, 255);
        }
    }
    fb.rect(x + 1, y + 1, w - 2, 1, if hover { [186, 198, 242, 255] } else { [170, 170, 174, 255] });
    fb.rect(x + 1, y + 1, 1, h - 2, if hover { [170, 182, 226, 255] } else { [155, 155, 159, 255] });
    fb.rect(x + 1, y + h - 2, w - 2, 1, [68, 68, 72, 255]);
    fb.rect(x + w - 2, y + 1, 1, h - 2, [68, 68, 72, 255]);
    let border = if hover { [255, 255, 255, 255] } else { [18, 18, 20, 255] };
    fb.frame(x, y, w, h, border);
    let tw = label.len() as i32 * 6;
    let txt_col = if hover { [255, 255, 160, 255] } else { [240, 240, 240, 255] };
    text_shadow(fb, x + (w - tw) / 2, y + (h - 5) / 2, label, txt_col, 1);
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

#[allow(dead_code)]
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

pub fn draw_title_overlay(fb: &mut Framebuffer, buttons: &[&str], hover: usize, splash: &str, t: f32) {
    // Subtle translucent vignette over the live 3D rotating panorama
    fb.rect(0, 0, W as i32, H as i32, [10, 14, 22, 42]);

    // Classic chunky stone-chiseled MINECRAFT logo with 3D extruded shadow
    let title = "MINECRAFT";
    let scale = 4;
    let tw = title.len() as i32 * 6 * scale;
    let tx = (W as i32 - tw) / 2;
    let ty = 26;
    for d in (1..=4).rev() {
        let du = d as u8;
        let sh = [26 + du * 4, 26 + du * 4, 30 + du * 4, 230];
        text(fb, tx + d, ty + d, title, sh, scale);
    }
    for (i, ch) in title.chars().enumerate() {
        let cx = tx + i as i32 * 6 * scale;
        let g = glyph(ch);
        for row in 0..5 {
            for col in 0..5 {
                if g[row] & (1 << (4 - col)) != 0 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let n = (th(cx + sx, row as i32 * scale + sy, 42) % 24) as i32 - 12;
                            let base = 222 - (row as i32 * 10 + sy * 2) + n;
                            let v = base.clamp(120, 248) as u8;
                            fb.set(cx + col as i32 * scale + sx, ty + row as i32 * scale + sy, v, v, (v as u16 + 4).min(255) as u8, 255);
                        }
                    }
                }
            }
        }
    }
    // Edition subtitle bar underneath MINECRAFT
    let sub = "JAVA EDITION";
    let sub_w = sub.len() as i32 * 6;
    text_shadow(fb, (W as i32 - sub_w) / 2, ty + 25, sub, [225, 228, 234, 255], 1);

    // Classic bouncing yellow splash text cleanly below the subtitle
    let sw = splash.len() as i32 * 6;
    let sx = (tx + tw - sw / 2).clamp(8, W as i32 - sw - 8);
    let sy = ty + 38 + ((t * 4.5).sin() * 2.5) as i32;
    text_shadow(fb, sx, sy, splash, [255, 255, 42, 255], 1);

    let bw = 200;
    let bx = (W as i32 - bw) / 2;
    for (i, label) in buttons.iter().enumerate() {
        let y = 108 + i as i32 * 25;
        button(fb, bx, y, bw, 21, label, hover == i);
    }
    text_shadow(fb, 8, H as i32 - 12, "Minecraft 1.20 Rust", [235, 235, 235, 230], 1);
    let cr = "Press F3 in-game for XYZ";
    text_shadow(fb, W as i32 - cr.len() as i32 * 6 - 8, H as i32 - 12, cr, [225, 225, 230, 210], 1);
}

#[allow(dead_code)]
pub fn draw_title(fb: &mut Framebuffer, buttons: &[&str], hover: usize, splash: &str, t: f32) {
    for y in 0..H as i32 {
        let f = y as f32 / H as f32;
        let r = (110.0 * (1.0 - f) + 188.0 * f) as u8;
        let g = (162.0 * (1.0 - f) + 222.0 * f) as u8;
        let b = (252.0 * (1.0 - f) + 255.0 * f) as u8;
        fb.rect(0, y, W as i32, 1, [r, g, b, 255]);
    }
    draw_title_overlay(fb, buttons, hover, splash, t);
}

// ===================== Survival Inventory =====================

pub fn craft_grid_idx(i: usize, grid_size: usize) -> usize {
    (i / grid_size) * 3 + (i % grid_size)
}

pub fn recipe_slot_rect(i: usize) -> (i32, i32, i32, i32) {
    (274 + (i % 6) as i32 * 26, 52 + (i / 6) as i32 * 26, 24, 24)
}

pub fn recipe_prev_rect() -> (i32, i32, i32, i32) { (366, 32, 16, 13) }
pub fn recipe_next_rect() -> (i32, i32, i32, i32) { (418, 32, 16, 13) }
pub fn recipe_fill_rect() -> (i32, i32, i32, i32) { (374, 146, 52, 18) }

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
    (46, 34 + i as i32 * 22, 22, 22)
}
pub fn offhand_slot_rect() -> (i32, i32, i32, i32) {
    (136, 100, 22, 22)
}
pub fn inv_panel_slot_rect(i: usize) -> (i32, i32, i32, i32) {
    (46 + (i % 9) as i32 * 22, 134 + (i / 9) as i32 * 22, 22, 22)
}
pub fn inv_panel_hot_rect(i: usize) -> (i32, i32, i32, i32) {
    (46 + i as i32 * 22, 208, 22, 22)
}
pub fn inv_craft_slot_rect(i: usize, grid_size: usize) -> (i32, i32, i32, i32) {
    let (gx, gy) = if grid_size == 3 { (68, 44) } else { (160, 48) };
    (gx + (i % grid_size) as i32 * 22, gy + (i / grid_size) as i32 * 22, 22, 22)
}
pub fn inv_craft_result_rect(grid_size: usize) -> (i32, i32, i32, i32) {
    if grid_size == 3 { (172, 64, 26, 26) } else { (222, 58, 24, 24) }
}

fn armor_tint(id: u16) -> Option<[u8; 3]> {
    match id {
        crate::items::LEATHER_HELMET | crate::items::LEATHER_CHESTPLATE | crate::items::LEATHER_LEGGINGS | crate::items::LEATHER_BOOTS => Some([148, 92, 52]),
        crate::items::IRON_HELMET | crate::items::IRON_CHESTPLATE | crate::items::IRON_LEGGINGS | crate::items::IRON_BOOTS => Some([208, 212, 218]),
        crate::items::GOLD_HELMET | crate::items::GOLD_CHESTPLATE | crate::items::GOLD_LEGGINGS | crate::items::GOLD_BOOTS => Some([245, 210, 62]),
        crate::items::DIAMOND_HELMET | crate::items::DIAMOND_CHESTPLATE | crate::items::DIAMOND_LEGGINGS | crate::items::DIAMOND_BOOTS => Some([88, 225, 232]),
        crate::items::NETHERITE_HELMET | crate::items::NETHERITE_CHESTPLATE | crate::items::NETHERITE_LEGGINGS | crate::items::NETHERITE_BOOTS => Some([68, 62, 72]),
        crate::items::ELYTRA => Some([128, 124, 146]),
        _ => None,
    }
}

fn draw_armor_silhouette(fb: &mut Framebuffer, x: i32, y: i32, kind: usize) {
    let col = [98, 98, 102, 255];
    match kind {
        0 => {
            fb.rect(x + 6, y + 6, 10, 4, col);
            fb.rect(x + 6, y + 10, 3, 4, col);
            fb.rect(x + 13, y + 10, 3, 4, col);
        }
        1 => {
            fb.rect(x + 5, y + 6, 12, 4, col);
            fb.rect(x + 7, y + 10, 8, 6, col);
        }
        2 => {
            fb.rect(x + 6, y + 6, 10, 4, col);
            fb.rect(x + 6, y + 10, 4, 6, col);
            fb.rect(x + 12, y + 10, 4, 6, col);
        }
        3 => {
            fb.rect(x + 6, y + 10, 4, 6, col);
            fb.rect(x + 12, y + 10, 4, 6, col);
            fb.rect(x + 5, y + 14, 5, 2, col);
            fb.rect(x + 12, y + 14, 5, 2, col);
        }
        _ => {
            // Shield silhouette
            fb.rect(x + 6, y + 5, 10, 12, col);
            fb.rect(x + 8, y + 17, 6, 1, col);
        }
    }
}

fn draw_steve_paperdoll(fb: &mut Framebuffer, bx: i32, by: i32, bw: i32, bh: i32, armor: &[crate::craft::Stack; 4], offhand: crate::craft::Stack) {
    fb.rect(bx, by, bw, bh, [0, 0, 0, 255]);
    fb.rect(bx, by, bw, 1, [55, 55, 55, 255]);
    fb.rect(bx, by, 1, bh, [55, 55, 55, 255]);
    fb.rect(bx + bw - 1, by, 1, bh, [255, 255, 255, 255]);
    fb.rect(bx, by + bh - 1, bw, 1, [255, 255, 255, 255]);

    let (mx, my) = fb.cursor;
    let cx = bx + bw / 2;
    let look_x = ((mx - cx) as f32 / 80.0).clamp(-1.0, 1.0);
    let look_y = ((my - (by + 24)) as f32 / 80.0).clamp(-1.0, 1.0);
    let hx_off = (look_x * 2.5).round() as i32;
    let hy_off = (look_y * 1.5).round() as i32;

    // Legs (dark indigo jeans) + optional leggings/boots
    let leg_col = armor_tint(armor[2].id).unwrap_or([52, 54, 128]);
    let boot_col = armor_tint(armor[3].id);
    fb.rect(cx - 8, by + 54, 7, 22, [leg_col[0], leg_col[1], leg_col[2], 255]);
    fb.rect(cx + 1, by + 54, 7, 22, [leg_col[0], leg_col[1], leg_col[2], 255]);
    fb.rect(cx - 1, by + 54, 2, 22, [28, 30, 74, 255]);
    if let Some(bc) = boot_col {
        fb.rect(cx - 8, by + 69, 7, 7, [bc[0], bc[1], bc[2], 255]);
        fb.rect(cx + 1, by + 69, 7, 7, [bc[0], bc[1], bc[2], 255]);
    } else {
        fb.rect(cx - 8, by + 72, 7, 4, [96, 96, 102, 255]);
        fb.rect(cx + 1, by + 72, 7, 4, [96, 96, 102, 255]);
    }

    // Torso (Steve cyan shirt) + optional chestplate
    let body_col = armor_tint(armor[1].id).unwrap_or([46, 154, 174]);
    fb.rect(cx - 8, by + 30, 16, 24, [body_col[0], body_col[1], body_col[2], 255]);
    fb.rect(cx - 8, by + 30, 16, 2, [(body_col[0] as f32 * 0.8) as u8, (body_col[1] as f32 * 0.8) as u8, (body_col[2] as f32 * 0.8) as u8, 255]);

    // Arms (cyan shoulder + skin forearm)
    let arm_top = armor_tint(armor[1].id).unwrap_or([46, 154, 174]);
    fb.rect(cx - 15, by + 30, 7, 10, [arm_top[0], arm_top[1], arm_top[2], 255]);
    fb.rect(cx - 15, by + 40, 7, 14, [214, 166, 126, 255]);
    fb.rect(cx + 8, by + 30, 7, 10, [arm_top[0], arm_top[1], arm_top[2], 255]);
    fb.rect(cx + 8, by + 40, 7, 14, [214, 166, 126, 255]);
    if !offhand.is_empty() {
        fb.rect(cx + 10, by + 38, 6, 14, [145, 106, 68, 255]);
        fb.frame(cx + 10, by + 38, 6, 14, [190, 194, 204, 255]);
    }

    // Head (skin, hair, eyes, beard) + optional helmet
    let head_x = cx - 9;
    let head_y = by + 11;
    fb.rect(head_x, head_y, 18, 18, [220, 172, 132, 255]);
    fb.rect(head_x, head_y, 18, 5, [74, 46, 28, 255]);
    fb.rect(head_x, head_y + 5, 2, 3, [74, 46, 28, 255]);
    fb.rect(head_x + 16, head_y + 5, 2, 3, [74, 46, 28, 255]);
    // Eyes looking subtly toward mouse cursor
    let ex = head_x + 3 + hx_off;
    let ey = head_y + 8 + hy_off;
    fb.rect(ex, ey, 4, 2, [255, 255, 255, 255]);
    fb.rect(ex + 2 + (hx_off > 0) as i32 - (hx_off < 0) as i32, ey, 2, 2, [64, 68, 152, 255]);
    fb.rect(ex + 8, ey, 4, 2, [255, 255, 255, 255]);
    fb.rect(ex + 8 + (hx_off > 0) as i32 - (hx_off < 0) as i32, ey, 2, 2, [64, 68, 152, 255]);
    // Nose + beard
    fb.rect(head_x + 7 + hx_off / 2, head_y + 11, 4, 2, [192, 138, 102, 255]);
    fb.rect(head_x + 5, head_y + 14, 8, 4, [120, 74, 48, 255]);
    fb.rect(head_x + 7, head_y + 14, 4, 2, [172, 102, 82, 255]);
    if let Some(hc) = armor_tint(armor[0].id) {
        fb.rect(head_x - 1, head_y - 1, 20, 7, [hc[0], hc[1], hc[2], 255]);
        fb.rect(head_x - 1, head_y + 6, 3, 6, [hc[0], hc[1], hc[2], 255]);
        fb.rect(head_x + 16, head_y + 6, 3, 6, [hc[0], hc[1], hc[2], 255]);
    }
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
    fb.rect(0, 0, W as i32, H as i32, [0, 0, 0, 145]);

    // Classic Java Main Inventory / Crafting Panel
    let (px, py, pw, ph) = (36, 26, 218, 218);
    panel(fb, px, py, pw, ph);
    let mut tip = String::new();

    if grid_size == 2 {
        // 4 Armor slots vertically + 3D Steve Preview Box + Offhand slot inside main panel
        for i in 0..4 {
            let (x, y, w, _h) = armor_slot_rect(i);
            let hv = hover == Some((4, i));
            slot(fb, x, y, w, hv);
            if armor[i].is_empty() {
                draw_armor_silhouette(fb, x, y, i);
            } else {
                draw_item_icon(fb, x + 3, y + 3, armor[i].id, w - 6, tick);
            }
            if hv && !armor[i].is_empty() { tip = crate::items::name(armor[i].id).to_string(); }
        }
        draw_steve_paperdoll(fb, 70, 34, 64, 88, armor, offhand);
        let (ox, oy, ow, _oh) = offhand_slot_rect();
        let ohv = hover == Some((4, 4));
        slot(fb, ox, oy, ow, ohv);
        if offhand.is_empty() {
            draw_armor_silhouette(fb, ox, oy, 4);
        } else {
            draw_item_icon(fb, ox + 3, oy + 3, offhand.id, ow - 6, tick);
        }
        if ohv && !offhand.is_empty() { tip = crate::items::name(offhand.id).to_string(); }
        text(fb, 160, 38, "Crafting", [64, 64, 64, 255], 1);
    } else {
        text(fb, 68, 34, "Crafting", [64, 64, 64, 255], 1);
        text(fb, 46, 122, "Inventory", [64, 64, 64, 255], 1);
    }

    // Crafting grid & output
    for i in 0..grid_size * grid_size {
        let (x, y, w, h) = inv_craft_slot_rect(i, grid_size);
        let hv = hover == Some((0, i));
        slot(fb, x, y, w, hv);
        let st = grid[craft_grid_idx(i, grid_size)];
        draw_item_icon(fb, x + 3, y + 3, st.id, w - 6, tick);
        if st.count > 1 { text_shadow(fb, x + w - 9, y + h - 8, &format!("{}", st.count), [255, 255, 255, 255], 1); }
        if hv && !st.is_empty() { tip = crate::items::name(st.id).to_string(); }
    }
    let (rx, ry, rw, rh) = inv_craft_result_rect(grid_size);
    let rhv = hover == Some((0, 99));
    slot(fb, rx, ry, rw, rhv);
    draw_item_icon(fb, rx + 3, ry + 3, result.id, rw - 6, tick);
    if result.count > 1 { text_shadow(fb, rx + rw - 9, ry + rh - 8, &format!("{}", result.count), [255, 255, 255, 255], 1); }
    if rhv && !result.is_empty() { tip = crate::items::name(result.id).to_string(); }
    let (ax, ay) = if grid_size == 3 { (146, 73) } else { (206, 66) };
    fb.rect(ax, ay, 8, 2, [130, 130, 130, 255]);
    fb.rect(ax + 6, ay - 2, 2, 6, [130, 130, 130, 255]);
    fb.rect(ax + 8, ay - 1, 2, 4, [130, 130, 130, 255]);

    // 3x9 Main Inventory + 1x9 Hotbar inside the main panel
    for i in 0..27 {
        let (x, y, w, h) = inv_panel_slot_rect(i);
        let hv = hover == Some((1, i));
        slot(fb, x, y, w, hv);
        draw_item_icon(fb, x + 3, y + 3, store[i].id, w - 6, tick);
        if store[i].count > 1 { text_shadow(fb, x + w - 9, y + h - 8, &format!("{}", store[i].count), [255, 255, 255, 255], 1); }
        if hv && !store[i].is_empty() { tip = crate::items::name(store[i].id).to_string(); }
    }
    for i in 0..9 {
        let (x, y, w, h) = inv_panel_hot_rect(i);
        let hv = hover == Some((2, i));
        slot(fb, x, y, w, hv);
        draw_item_icon(fb, x + 3, y + 3, hot[i].id, w - 6, tick);
        if hot[i].count > 1 { text_shadow(fb, x + w - 9, y + h - 8, &format!("{}", hot[i].count), [255, 255, 255, 255], 1); }
        if hv && !hot[i].is_empty() { tip = crate::items::name(hot[i].id).to_string(); }
    }

    // Docked Java-style Recipe Book side panel
    panel(fb, 260, 26, 184, 218);
    text(fb, 272, 35, "Recipe Book", [64, 64, 64, 255], 1);
    let recs = crate::craft::recipes();
    let per_page = 12usize;
    let pages = (recs.len() + per_page - 1) / per_page;
    let page = recipe_page.min(pages.saturating_sub(1));
    let (pvx, pvy, pvw, pvh) = recipe_prev_rect();
    let (nvx, nvy, nvw, nvh) = recipe_next_rect();
    button(fb, pvx, pvy, pvw, pvh, "<", hover == Some((3, 100)));
    text(fb, 386, 35, &format!("{}/{}", page + 1, pages), [64, 64, 64, 255], 1);
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
            draw_item_icon(fb, x + 3, y + 3, recs[idx].out, w - 6, tick);
            if recs[idx].count > 1 {
                text_shadow(fb, x + w - 9, y + h - 8, &format!("{}", recs[idx].count), [255, 255, 255, 255], 1);
            }
            if hv { tip = crate::items::name(recs[idx].out).to_string(); }
        }
    }

    fb.rect(270, 110, 164, 122, [168, 168, 172, 255]);
    fb.frame(270, 110, 164, 122, [95, 95, 100, 255]);
    if let Some(rec) = recs.get(recipe_sel) {
        for r in 0..3 {
            for c in 0..3 {
                let sx = 280 + c as i32 * 18;
                let sy = 122 + r as i32 * 18;
                slot(fb, sx, sy, 18, false);
                let id = rec.pattern[r * 3 + c];
                if id != 0 { draw_item_icon(fb, sx + 2, sy + 2, id, 14, tick); }
            }
        }
        fb.rect(338, 148, 8, 2, [110, 110, 114, 255]);
        fb.rect(344, 146, 2, 6, [110, 110, 114, 255]);
        slot(fb, 350, 138, 20, false);
        draw_item_icon(fb, 352, 140, rec.out, 16, tick);
        let (fx, fy, fw, fh) = recipe_fill_rect();
        button(fb, fx, fy, fw, fh, "Fill", hover == Some((3, 102)));
        let out_name = crate::items::name(rec.out);
        let short = if out_name.len() > 24 { &out_name[..24] } else { out_name };
        text(fb, 280, 186, short, [50, 50, 55, 255], 1);
        if grid_size == 2 && !recipe_fits_2x2(&rec.pattern) {
            text(fb, 280, 198, "Needs Crafting Table", [175, 40, 40, 255], 1);
        }
    }

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
    fb.rect(0, 0, W as i32, H as i32, [10, 14, 24, 172]);
    let names = [
        String::from("Back to Game"),
        String::from("Save World"),
        format!("Game Mode: {}", mode.name()),
        format!("Sounds: {}   Music: {}", if sound { "ON" } else { "OFF" }, if music { "ON" } else { "OFF" }),
        String::from("Options..."),
        String::from("Save and Quit to Title"),
    ];
    let title = "Game Menu";
    text_shadow(fb, (W as i32 - title.len() as i32 * 12) / 2, 24, title, [255, 255, 255, 255], 2);
    let bw = 204; let bx = (W as i32 - bw) / 2;
    for (i, n) in names.iter().enumerate() { button(fb, bx, 60 + i as i32 * 25, bw, 20, n, hover == i); }
}

pub fn draw_multiplayer_screen(fb: &mut Framebuffer, addr: &str, status: &str, hover: usize, hosting: bool) {
    fb.rect(0, 0, W as i32, H as i32, [0, 0, 0, 145]);
    let (px, py, pw, ph) = (50, 20, 380, 230);
    panel(fb, px, py, pw, ph);
    text(fb, px + 10, py + 8, "Play Multiplayer", [60, 60, 60, 255], 1);
    text(fb, px + 10, py + 34, "Server Address (host:port)", [60, 60, 60, 255], 1);
    fb.rect(px + 10, py + 46, pw - 20, 18, [22, 22, 22, 255]);
    fb.frame(px + 10, py + 46, pw - 20, 18, [160, 160, 160, 255]);
    text(fb, px + 14, py + 51, &format!("{}_", addr), [240, 240, 240, 255], 1);
    text(fb, px + 10, py + 74, status, [40, 80, 140, 255], 1);
    button(fb, px + 10, py + 96, 170, 22, "Join Server", hover == 0);
    button(fb, px + 200, py + 96, 170, 22, if hosting { "Hosting on 25565..." } else { "Open to LAN" }, hover == 1);
    button(fb, px + 10, py + 128, 170, 22, "Direct Connect (127.0.0.1)", hover == 2);
    button(fb, px + 200, py + 128, 170, 22, "Cancel", hover == 3);
    text(fb, px + 10, py + 166, "Host clicks Open to LAN (TCP port 25565).", [60, 60, 60, 255], 1);
    text(fb, px + 10, py + 180, "Friends enter Host IP:25565 and click Join Server.", [60, 60, 60, 255], 1);
    text(fb, px + 10, py + 196, "Seed, time, players, & block edits sync automatically!", [60, 60, 60, 255], 1);
}

pub fn draw_select_world_screen(
    fb: &mut Framebuffer,
    worlds: &[(String, String)],
    selected: usize,
    hover: usize,
) {
    // Classic Java dirt background
    let atlas = Textures::atlas();
    let dirt_k = Textures::face_kind(world::DIRT, 0, 1, 0);
    for y in 0..H as i32 {
        for x in 0..W as i32 {
            let c = atlas.sample_kind(dirt_k, ((x / 2) % 16) as f32 / 16.0, ((y / 2) % 16) as f32 / 16.0, 0, false);
            fb.set(x, y, c[0] / 3, c[1] / 3, c[2] / 3, 255);
        }
    }
    let title = "Select World";
    text_shadow(fb, (W as i32 - title.len() as i32 * 6) / 2, 12, title, [255, 255, 255, 255], 1);
    fb.rect(40, 26, W as i32 - 80, 166, [0, 0, 0, 150]);
    fb.frame(40, 26, W as i32 - 80, 166, [85, 85, 90, 255]);
    if worlds.is_empty() {
        text_shadow(fb, 150, 96, "No saved worlds yet! Create New World below.", [190, 190, 190, 255], 1);
    } else {
        for (i, (wname, meta)) in worlds.iter().take(6).enumerate() {
            let y = 32 + i as i32 * 26;
            let is_sel = i == selected;
            let hv = hover == 4 + i;
            if is_sel || hv {
                fb.rect(46, y, W as i32 - 92, 24, if is_sel { [40, 48, 62, 230] } else { [32, 32, 38, 190] });
                fb.frame(46, y, W as i32 - 92, 24, if is_sel { [255, 255, 255, 255] } else { [130, 130, 140, 255] });
            }
            text_shadow(fb, 54, y + 4, wname, [255, 255, 255, 255], 1);
            text_shadow(fb, 54, y + 14, meta, [175, 175, 185, 255], 1);
        }
    }
    button(fb, 46, 200, 188, 22, "Play Selected World", hover == 0);
    button(fb, 246, 200, 188, 22, "Create New World", hover == 1);
    button(fb, 46, 228, 188, 22, "Delete World", hover == 2);
    button(fb, 246, 228, 188, 22, "Cancel", hover == 3);
}

/// Classic Minecraft Java Edition chunk-grid world generation/loading screen.
pub fn draw_loading_screen(fb: &mut Framebuffer, label: &str, pct: f32, tick: u32) {
    let atlas = Textures::atlas();
    let dirt_k = Textures::face_kind(world::DIRT, 0, 1, 0);
    for y in 0..H as i32 {
        for x in 0..W as i32 {
            let c = atlas.sample_kind(dirt_k, ((x / 2) % 16) as f32 / 16.0, ((y / 2) % 16) as f32 / 16.0, 0, false);
            fb.set(x, y, c[0] / 3, c[1] / 3, c[2] / 3, 255);
        }
    }
    let title = label;
    text_shadow(fb, (W as i32 - title.len() as i32 * 6) / 2, 54, title, [255, 255, 255, 255], 1);
    let pct_str = format!("{}%", (pct * 100.0).clamp(0.0, 100.0) as i32);
    text_shadow(fb, (W as i32 - pct_str.len() as i32 * 6) / 2, 68, &pct_str, [200, 220, 200, 255], 1);

    // Classic Java 11x11 chunk-loading square grid in the center
    let grid = 11i32;
    let cell = 7i32;
    let gx0 = (W as i32 - grid * cell) / 2;
    let gy0 = 92;
    let radius_f = pct * 7.8;
    fb.rect(gx0 - 3, gy0 - 3, grid * cell + 6, grid * cell + 6, [18, 18, 22, 230]);
    for rz in 0..grid {
        for rx in 0..grid {
            let dx = (rx - 5) as f32;
            let dz = (rz - 5) as f32;
            let d = (dx * dx + dz * dz).sqrt();
            let col = if d <= radius_f - 1.5 {
                [242, 242, 242, 255] // Fully lit center chunks
            } else if d <= radius_f {
                [92, 212, 92, 255] // Active green chunk rim
            } else if d <= radius_f + 1.0 && ((rx + rz + tick as i32) & 1 == 0) {
                [224, 198, 72, 255] // Pre-lighting yellow
            } else {
                [46, 52, 68, 255] // Unloaded chunk
            };
            fb.rect(gx0 + rx * cell, gy0 + rz * cell, cell - 1, cell - 1, col);
        }
    }
}

pub fn draw_options_screen(
    fb: &mut Framebuffer,
    fov: f32,
    sound: bool,
    music: bool,
    scale: i32,
    camera: u8,
    shader_name: &str,
    difficulty: u8,
    auto_jump: bool,
    pack_count: usize,
    hover: usize,
) {
    fb.rect(0, 0, W as i32, H as i32, [0, 0, 0, 165]);
    let (px, py, pw, ph) = (32, 10, 416, 250);
    panel(fb, px, py, pw, ph);
    let hdr = "Options";
    text(fb, (W as i32 - hdr.len() as i32 * 6) / 2, py + 7, hdr, [45, 45, 45, 255], 1);
    let cam_name = match camera { 1 => "3rd Person Back", 2 => "3rd Person Front", _ => "1st Person" };
    let res_label = match scale {
        0 => "Native HD (960x540)",
        1 => "Balanced (640x360)",
        2 => "Fast (480x270)",
        _ => "Ultra Fast (320x180)",
    };
    let diff_name = match difficulty { 0 => "Peaceful", 1 => "Easy", 2 => "Normal", _ => "Hard" };
    let items = [
        format!("FOV: {} (C=Zoom)", fov.round() as i32),
        format!("Difficulty: {}", diff_name),
        format!("Music: {}", if music { "ON" } else { "OFF" }),
        format!("Sounds: {}", if sound { "ON" } else { "OFF" }),
        format!("Graphics: {}", res_label),
        format!("Camera (F5): {}", cam_name),
        format!("Auto-Jump: {}", if auto_jump { "ON" } else { "OFF" }),
        format!("Shader: {}", shader_name),
        format!("Resource Packs ({})", pack_count),
        "Done".to_string(),
    ];
    for (i, label) in items.iter().enumerate() {
        if i < 8 {
            let col = i % 2;
            let row = i / 2;
            button(fb, px + 14 + col as i32 * 198, py + 24 + row as i32 * 28, 190, 22, label, hover == i);
        } else if i == 8 {
            button(fb, px + 14, py + 144, 388, 22, label, hover == 8);
        } else {
            button(fb, px + 108, py + 210, 200, 22, label, hover == 9);
        }
    }
    text(fb, px + 14, py + 174, "Keys: F1 HUD | F2 Screenshot | F3 Debug | F4 Mode | F5 Cam | F11 Full", [65, 65, 70, 255], 1);
    text(fb, px + 14, py + 188, "      C Zoom | F Offhand | Q Drop | Ctrl Sprint | Shift Sneak", [65, 65, 70, 255], 1);
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
    profession: &str,
    trades: &[(u16, u32, u16, u32)],
    hot: &[crate::craft::Stack; 9],
    tick: u32,
    hover: Option<(usize, usize)>,
) {
    fb.rect(0, 0, W as i32, H as i32, [0, 0, 0, 150]);
    let (px, py, pw, ph) = (56, 12, 368, 246);
    panel(fb, px, py, pw, ph);
    text(fb, px + 10, py + 7, &format!("{} (Click Trade)", profession), [50, 50, 55, 255], 1);
    for (i, &(cost_id, cost_n, out_id, out_n)) in trades.iter().enumerate().take(6) {
        let by = 26 + i as i32 * 28;
        let hv = hover == Some((0, i));
        button(fb, 72, by, 336, 24, "", hv);
        draw_item_icon(fb, 80, by + 2, cost_id, 20, tick);
        text_shadow(fb, 104, by + 8, &format!("{}x {}", cost_n, crate::items::name(cost_id)), [255, 255, 255, 255], 1);
        text_shadow(fb, 222, by + 8, "->", [255, 235, 120, 255], 1);
        draw_item_icon(fb, 242, by + 2, out_id, 20, tick);
        text_shadow(fb, 266, by + 8, &format!("{}x {}", out_n, crate::items::name(out_id)), [170, 255, 170, 255], 1);
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
