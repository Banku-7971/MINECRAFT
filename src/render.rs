// Software renderer: 16x16 face-aware pixel textures, voxel raycaster,
// 3D rasterized box-model mobs/player, isometric 3D block icons, and Minecraft UI.
use crate::entities::{self, FaceArt, MobKind, Part3D};
use crate::world::{self, Biome, Hit, World};
use std::f32::consts::PI;

pub const W: usize = 480;
pub const H: usize = 270;
pub const RW: usize = 320;
pub const RH: usize = 180;
const TEX: usize = 16;

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

fn clamp255(v: f32) -> u8 { v.clamp(0.0, 255.0) as u8 }

pub struct Textures {
    data: Vec<Vec<u8>>,
}

impl Textures {
    pub fn new() -> Self {
        let mut data = Vec::with_capacity(84);
        for k in 0..84 {
            data.push(Self::generate(k));
        }
        Self { data }
    }

    pub fn face_kind(block: u16, _nx: i32, ny: i32, nz: i32) -> usize {
        match block {
            world::GRASS => {
                if ny > 0 { 1 }
                else if ny < 0 { 3 }
                else { 2 }
            }
            world::PODZOL => {
                if ny > 0 { 63 }
                else if ny < 0 { 3 }
                else { 2 }
            }
            world::MYCELIUM => {
                if ny > 0 { 64 }
                else if ny < 0 { 3 }
                else { 2 }
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
                if ny > 0 { 41 }
                else if ny < 0 { 12 }
                else { 72 }
            }
            world::FURNACE => {
                if ny != 0 { 73 }
                else if nz > 0 { 42 }
                else { 73 }
            }
            world::TNT => {
                if ny != 0 { 74 } else { 70 }
            }
            world::BOOKSHELF => {
                if ny != 0 { 12 } else { 55 }
            }
            world::PUMPKIN => {
                if ny > 0 { 75 } else { 44 }
            }
            world::CHEST => {
                if nz > 0 { 40 } else { 76 }
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
            world::PLANKS | world::DOOR | world::LADDER | world::FENCE | world::SIGN => 12,
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
            world::CLAY | world::TERRACOTTA => 23,
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
            world::BLACKSTONE | world::OBSIDIAN => 35,
            world::MAGMA => 36,
            world::LAVA => 37,
            world::FARMLAND => 38,
            world::WHEAT => 39,
            world::CHEST => 40,
            world::CRAFTING => 41,
            world::FURNACE => 42,
            world::PUMPKIN | world::CAKE => 44,
            world::HAY => 45,
            world::TNT => 70,
            world::GLOWSTONE | world::SEA_LANTERN | world::LANTERN => 46,
            world::MOSSY => 4,
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
            _ => 0,
        }
    }

    fn generate(k: usize) -> Vec<u8> {
        let mut out = vec![0u8; TEX * TEX * 4];
        for y in 0..TEX {
            for x in 0..TEX {
                let i = (y * TEX + x) * 4;
                let r = th(x as i32, y as i32, k as i32) % 256;
                let j = ((r as f32 / 255.0) - 0.5) * 20.0;
                let (cr, cg, cb, a) = match k {
                    0 => { // stone
                        let streak = if (x + y * 3) % 11 == 0 { -10.0 } else { 0.0 };
                        (126.0 + j + streak, 126.0 + j + streak, 128.0 + j + streak, 255.0)
                    }
                    1 => { // grass top
                        let blade = if r % 5 == 0 { 14.0 } else if r % 7 == 0 { -14.0 } else { 0.0 };
                        (88.0 + j * 0.7 + blade, 152.0 + j * 0.7 + blade, 54.0 + j * 0.5, 255.0)
                    }
                    2 => { // grass side
                        let edge = 3 + (th(x as i32, 0, 202) % 3) as usize;
                        if y < edge {
                            let blade = if r % 4 == 0 { -12.0 } else { 0.0 };
                            (84.0 + j * 0.5 + blade, 148.0 + j * 0.5 + blade, 52.0, 255.0)
                        } else if y == edge && r % 3 == 0 {
                            (74.0, 126.0, 46.0, 255.0)
                        } else {
                            let speck = if r % 11 == 0 { -18.0 } else if r % 13 == 0 { 15.0 } else { 0.0 };
                            (130.0 + j + speck, 92.0 + j + speck, 62.0 + j * 0.8 + speck, 255.0)
                        }
                    }
                    3 => { // dirt
                        let speck = if r % 9 == 0 { -18.0 } else if r % 14 == 0 { 16.0 } else { 0.0 };
                        (130.0 + j + speck, 92.0 + j + speck, 62.0 + j * 0.8 + speck, 255.0)
                    }
                    4 => { // cobblestone
                        let mortar = x == 0 || y == 0 || y == 8 || (y < 8 && x == 8) || (y >= 8 && (x == 4 || x == 12));
                        if mortar { (84.0, 84.0, 86.0, 255.0) }
                        else {
                            let tone = ((th(x as i32 / 4, y as i32 / 4, 4) % 3) as f32 - 1.0) * 14.0;
                            (134.0 + j * 0.6 + tone, 134.0 + j * 0.6 + tone, 136.0 + j * 0.6 + tone, 255.0)
                        }
                    }
                    5 | 47 => { // oak / spruce bark
                        let base = if k == 47 { (74.0, 52.0, 34.0) } else { (106.0, 80.0, 50.0) };
                        let groove = (x % 4 == 0) || (x % 4 == 2 && (y + x) % 5 == 0);
                        let mul = if groove { 0.72 } else { 1.0 };
                        (base.0 * mul + j * 0.5, base.1 * mul + j * 0.5, base.2 * mul + j * 0.4, 255.0)
                    }
                    6 => { // log rings top
                        let d = ((x as i32 - 8).abs()).max((y as i32 - 8).abs());
                        if d >= 7 { (104.0, 78.0, 48.0, 255.0) }
                        else if d == 4 || d == 2 { (142.0, 108.0, 68.0, 255.0) }
                        else { (182.0 + j * 0.4, 146.0 + j * 0.4, 96.0 + j * 0.3, 255.0) }
                    }
                    7 => { // oak leaves (dense opaque pixel foliage)
                        if r % 5 == 0 { (36.0, 74.0, 28.0, 255.0) }
                        else if r % 4 == 0 { (78.0, 142.0, 52.0, 255.0) }
                        else { (56.0 + j * 0.5, 114.0 + j * 0.6, 40.0 + j * 0.4, 255.0) }
                    }
                    77 => { // spruce leaves
                        if r % 5 == 0 { (26.0, 54.0, 32.0, 255.0) }
                        else { (42.0 + j * 0.4, 86.0 + j * 0.5, 48.0 + j * 0.4, 255.0) }
                    }
                    78 => { // birch leaves
                        if r % 5 == 0 { (54.0, 96.0, 42.0, 255.0) }
                        else { (92.0 + j * 0.5, 152.0 + j * 0.5, 64.0 + j * 0.4, 255.0) }
                    }
                    8 => (224.0 + j * 0.4, 212.0 + j * 0.4, 162.0 + j * 0.4, 255.0), // sand
                    79 => (196.0 + j * 0.4, 108.0 + j * 0.4, 52.0 + j * 0.3, 255.0), // red sand
                    9 => { // water
                        let crest = (x + y * 2) % 8 == 0;
                        if crest { (78.0, 148.0, 224.0, 180.0) } else { (44.0, 108.0, 200.0, 168.0) }
                    }
                    10 => (240.0 + j * 0.25, 246.0 + j * 0.2, 250.0, 255.0), // snow
                    11 => { // ice
                        let shine = (x + y) == 6 || (x + y) == 7 || (x + y) == 22;
                        if shine { (210.0, 236.0, 255.0, 210.0) } else { (145.0 + j * 0.4, 198.0 + j * 0.3, 240.0, 195.0) }
                    }
                    12 => { // planks
                        let seam = y % 4 == 0 || ((y / 4) * 5 + x) % 16 == 0;
                        if seam { (122.0, 88.0, 50.0, 255.0) }
                        else { (178.0 + j * 0.4, 134.0 + j * 0.4, 82.0 + j * 0.3, 255.0) }
                    }
                    13 => { // glass
                        let border = x == 0 || y == 0 || x == 15 || y == 15;
                        let glint = (x + y == 5 && x < 5) || (x + y == 25 && x > 10);
                        if border { (175.0, 215.0, 228.0, 220.0) }
                        else if glint { (240.0, 250.0, 255.0, 190.0) }
                        else { (205.0, 235.0, 245.0, 40.0) }
                    }
                    14 => { // torch
                        if x >= 7 && x <= 8 && y >= 5 { (112.0, 76.0, 42.0, 255.0) }
                        else if x >= 6 && x <= 9 && y >= 2 && y <= 5 { (252.0, 190.0, 55.0, 255.0) }
                        else { (0.0, 0.0, 0.0, 0.0) }
                    }
                    15 => (42.0 + j * 0.8, 42.0 + j * 0.8, 44.0 + j * 0.8, 255.0), // bedrock
                    16 | 17 | 18 | 19 | 20 | 21 => { // ores
                        let speck = ((x == 4 || x == 5) && (y == 3 || y == 4))
                            || ((x == 10 || x == 11) && (y == 6 || y == 7))
                            || ((x == 5 || x == 6) && (y == 11 || y == 12))
                            || ((x == 11 || x == 12) && (y == 10 || y == 11));
                        if speck {
                            match k {
                                16 => (34.0, 34.0, 38.0, 255.0),
                                17 => (214.0, 178.0, 142.0, 255.0),
                                18 => (248.0, 206.0, 56.0, 255.0),
                                19 => (76.0, 226.0, 232.0, 255.0),
                                20 => (224.0, 38.0, 38.0, 255.0),
                                _ => (52.0, 212.0, 102.0, 255.0),
                            }
                        } else {
                            (126.0 + j, 126.0 + j, 128.0 + j, 255.0)
                        }
                    }
                    22 => (150.0 + j, 146.0 + j, 142.0 + j, 255.0), // gravel
                    23 => (162.0 + j * 0.5, 166.0 + j * 0.5, 174.0 + j * 0.5, 255.0), // clay
                    24 => { // cactus
                        if x % 4 == 0 && y % 3 == 0 { (36.0, 82.0, 30.0, 255.0) }
                        else { (64.0 + j * 0.5, 134.0 + j * 0.5, 50.0 + j * 0.4, 255.0) }
                    }
                    25 => { // tall grass
                        if (x == 3 || x == 7 || x == 11 || x == 14) && y >= 5 { (72.0, 138.0, 48.0, 255.0) }
                        else if (x == 4 || x == 8 || x == 12) && y >= 3 { (92.0, 158.0, 60.0, 255.0) }
                        else { (0.0, 0.0, 0.0, 0.0) }
                    }
                    26 => { // red flower
                        if x >= 6 && x <= 9 && y >= 4 && y <= 8 { (222.0, 48.0, 50.0, 255.0) }
                        else if x >= 7 && x <= 8 && y >= 9 { (64.0, 132.0, 48.0, 255.0) }
                        else { (0.0, 0.0, 0.0, 0.0) }
                    }
                    27 => { // yellow flower
                        if x >= 6 && x <= 9 && y >= 4 && y <= 8 { (242.0, 212.0, 50.0, 255.0) }
                        else if x >= 7 && x <= 8 && y >= 9 { (64.0, 132.0, 48.0, 255.0) }
                        else { (0.0, 0.0, 0.0, 0.0) }
                    }
                    28 => { // brick
                        let mortar = y % 4 == 0 || ((y / 4) % 2 == 0 && x % 8 == 0) || ((y / 4) % 2 == 1 && (x + 4) % 8 == 0);
                        if mortar { (182.0, 176.0, 168.0, 255.0) } else { (152.0 + j * 0.6, 72.0 + j * 0.4, 58.0 + j * 0.4, 255.0) }
                    }
                    29 => { // stone bricks
                        let mortar = y % 8 == 0 || ((y / 8) == 0 && x % 8 == 0) || ((y / 8) == 1 && (x + 4) % 8 == 0);
                        if mortar { (84.0, 84.0, 88.0, 255.0) } else { (124.0 + j * 0.5, 124.0 + j * 0.5, 128.0 + j * 0.5, 255.0) }
                    }
                    30 => (236.0 + j * 0.3, 236.0 + j * 0.3, 232.0 + j * 0.3, 255.0),
                    31 => (182.0 + j * 0.4, 48.0 + j * 0.3, 46.0 + j * 0.3, 255.0),
                    32 => (52.0 + j * 0.3, 86.0 + j * 0.3, 182.0 + j * 0.3, 255.0),
                    33 => (54.0 + j * 0.3, 128.0 + j * 0.4, 54.0 + j * 0.3, 255.0),
                    34 => (224.0 + j * 0.3, 192.0 + j * 0.3, 52.0 + j * 0.3, 255.0),
                    35 => (38.0 + j * 0.4, 30.0 + j * 0.4, 48.0 + j * 0.5, 255.0),
                    36 => { let hot = r % 4 == 0; if hot { (235.0, 110.0, 25.0, 255.0) } else { (110.0, 38.0, 28.0, 255.0) } }
                    37 => { let bright = (x + y) % 5 == 0; if bright { (252.0, 180.0, 42.0, 255.0) } else { (220.0, 88.0, 20.0, 255.0) } }
                    38 => { if y % 4 == 0 { (72.0, 48.0, 30.0, 255.0) } else { (104.0 + j * 0.4, 72.0 + j * 0.4, 46.0 + j * 0.4, 255.0) } }
                    39 => { if x % 4 == 1 && y >= 3 { (210.0, 182.0, 68.0, 255.0) } else { (0.0, 0.0, 0.0, 0.0) } }
                    40 => { // chest front
                        let border = x == 0 || y == 0 || x == 15 || y == 15 || y == 5;
                        if x >= 7 && x <= 8 && y >= 4 && y <= 7 { (222.0, 195.0, 95.0, 255.0) }
                        else if border { (78.0, 52.0, 28.0, 255.0) }
                        else { (152.0 + j * 0.4, 104.0 + j * 0.4, 54.0 + j * 0.3, 255.0) }
                    }
                    41 => { // crafting table top
                        let border = x == 0 || y == 0 || x == 15 || y == 15;
                        let grid = x % 5 == 0 || y % 5 == 0;
                        if border { (94.0, 62.0, 34.0, 255.0) }
                        else if grid { (68.0, 44.0, 24.0, 255.0) }
                        else { (176.0 + j * 0.3, 128.0 + j * 0.3, 76.0 + j * 0.3, 255.0) }
                    }
                    42 => { // furnace front
                        let rim = x <= 1 || y <= 1 || x >= 14 || y >= 14;
                        let top_mouth = x >= 4 && x <= 11 && y >= 3 && y <= 6;
                        let firebox = x >= 5 && x <= 10 && y >= 10 && y <= 13;
                        if rim { (110.0, 110.0, 114.0, 255.0) }
                        else if top_mouth || firebox { (38.0, 38.0, 42.0, 255.0) }
                        else { (134.0 + j * 0.4, 134.0 + j * 0.4, 136.0 + j * 0.4, 255.0) }
                    }
                    43 => { // sandstone
                        let band = y == 2 || y == 6 || y == 11;
                        if band { (198.0, 180.0, 132.0, 255.0) } else { (220.0 + j * 0.3, 202.0 + j * 0.3, 150.0 + j * 0.3, 255.0) }
                    }
                    44 => { // pumpkin side
                        if x % 4 == 0 { (180.0, 92.0, 24.0, 255.0) } else { (222.0 + j * 0.4, 124.0 + j * 0.4, 34.0, 255.0) }
                    }
                    45 => { // hay
                        if y == 5 || y == 11 { (182.0, 48.0, 36.0, 255.0) }
                        else { (212.0 + j * 0.4, 182.0 + j * 0.4, 72.0 + j * 0.3, 255.0) }
                    }
                    46 => { // glowstone
                        if x % 5 == 0 || y % 5 == 0 { (172.0, 124.0, 56.0, 255.0) } else { (248.0, 224.0, 146.0, 255.0) }
                    }
                    48 => { if (x + y) % 4 == 0 || (x as i32 - y as i32).abs() % 5 == 0 { (225.0, 225.0, 225.0, 230.0) } else { (0.0, 0.0, 0.0, 0.0) } }
                    49 => (235.0 + j * 0.2, 235.0 + j * 0.2, 235.0 + j * 0.2, 255.0),
                    50 => (54.0 + j * 0.3, 92.0 + j * 0.3, 184.0 + j * 0.3, 255.0),
                    51 => (188.0 + j * 0.3, 48.0 + j * 0.2, 48.0 + j * 0.2, 255.0),
                    52 => (52.0 + j * 0.3, 134.0 + j * 0.3, 58.0 + j * 0.3, 255.0),
                    53 => (224.0 + j * 0.2, 192.0 + j * 0.3, 48.0 + j * 0.2, 255.0),
                    54 => (38.0 + j * 0.3, 38.0 + j * 0.3, 42.0 + j * 0.3, 255.0),
                    55 => { // bookshelf
                        if y < 2 || y > 13 || y == 7 || y == 8 { (168.0, 126.0, 78.0, 255.0) }
                        else if x == 0 || x == 15 { (134.0, 96.0, 56.0, 255.0) }
                        else {
                            match (x / 2 + y / 8) % 4 {
                                0 => (178.0, 52.0, 48.0, 255.0),
                                1 => (58.0, 132.0, 62.0, 255.0),
                                2 => (62.0, 88.0, 174.0, 255.0),
                                _ => (212.0, 174.0, 68.0, 255.0),
                            }
                        }
                    }
                    56 => { if x == 3 || x == 12 { (170.0, 172.0, 178.0, 255.0) } else if y % 4 == 1 { (124.0, 88.0, 52.0, 255.0) } else { (0.0, 0.0, 0.0, 0.0) } }
                    57 => (74.0 + j * 0.3, 74.0 + j * 0.3, 80.0 + j * 0.3, 255.0),
                    58 => (82.0 + j * 0.3, 46.0 + j * 0.3, 128.0 + j * 0.3, 255.0),
                    59 => (218.0 + j * 0.4, 218.0 + j * 0.4, 188.0 + j * 0.4, 255.0),
                    60 => (108.0 + j * 0.5, 42.0 + j * 0.4, 42.0 + j * 0.4, 255.0),
                    61 => (92.0 + j * 0.4, 74.0 + j * 0.4, 58.0 + j * 0.4, 255.0),
                    62 => (234.0 + j * 0.2, 232.0 + j * 0.2, 224.0 + j * 0.2, 255.0),
                    63 => (98.0 + j * 0.4, 70.0 + j * 0.3, 44.0 + j * 0.3, 255.0),
                    64 => (124.0 + j * 0.4, 108.0 + j * 0.4, 128.0 + j * 0.4, 255.0),
                    65 => {
                        let center = (x as i32 - 8).abs() <= 1 || (y as i32 - 8).abs() <= 1;
                        if center && r % 3 != 0 { (212.0, 24.0, 24.0, 255.0) }
                        else { (118.0 + j * 0.3, 118.0 + j * 0.3, 120.0 + j * 0.3, 255.0) }
                    }
                    66 => {
                        if y <= 5 && x >= 6 && x <= 9 { (252.0, 46.0, 34.0, 255.0) }
                        else if x >= 7 && x <= 8 && y >= 5 { (112.0, 76.0, 42.0, 255.0) }
                        else { (0.0, 0.0, 0.0, 0.0) }
                    }
                    67 => {
                        if y >= 10 && x >= 4 && x <= 11 { (128.0, 128.0, 130.0, 255.0) }
                        else if x >= 7 && x <= 8 && y >= 4 { (122.0, 86.0, 52.0, 255.0) }
                        else { (0.0, 0.0, 0.0, 0.0) }
                    }
                    68 => {
                        let frame = x == 0 || y == 0 || x == 15 || y == 15 || x == 8 || y == 8;
                        if frame { (110.0, 78.0, 42.0, 255.0) } else { (208.0 + j * 0.3, 166.0 + j * 0.3, 88.0, 255.0) }
                    }
                    69 => { if x >= 2 && x <= 13 && y >= 2 && y <= 13 { (148.0, 148.0, 152.0, 255.0) } else { (0.0, 0.0, 0.0, 0.0) } }
                    70 => { // TNT side
                        if y >= 5 && y <= 10 {
                            let letter = ((x >= 2 && x <= 4 && y == 6) || (x == 3 && y >= 6 && y <= 9))
                                || ((x == 6 || x == 9) && y >= 6 && y <= 9) || (x == 7 && y == 7) || (x == 8 && y == 8)
                                || ((x >= 11 && x <= 13 && y == 6) || (x == 12 && y >= 6 && y <= 9));
                            if letter { (24.0, 24.0, 24.0, 255.0) } else { (240.0, 238.0, 232.0, 255.0) }
                        } else {
                            let plank = if x % 3 == 0 { -22.0 } else { 0.0 };
                            (196.0 + plank + j * 0.3, 46.0 + j * 0.2, 38.0 + j * 0.2, 255.0)
                        }
                    }
                    71 => { // birch bark
                        let mark = (y % 5 == 1 && (x + y * 3) % 7 < 3) || (y % 4 == 3 && x % 6 == 2);
                        if mark { (54.0, 52.0, 48.0, 255.0) } else { (228.0 + j * 0.25, 224.0 + j * 0.25, 214.0 + j * 0.25, 255.0) }
                    }
                    72 => { // crafting table side
                        let border = x == 0 || y == 0 || x == 15 || y == 15;
                        let tool = (x >= 4 && x <= 6 && y >= 4 && y <= 11) || (x >= 10 && x <= 12 && y >= 4 && y <= 9);
                        if border { (104.0, 72.0, 42.0, 255.0) }
                        else if tool { (132.0, 134.0, 140.0, 255.0) }
                        else { (168.0 + j * 0.3, 122.0 + j * 0.3, 74.0 + j * 0.3, 255.0) }
                    }
                    73 => { // furnace side/top
                        let border = x == 0 || y == 0 || x == 15 || y == 15;
                        if border { (102.0, 102.0, 106.0, 255.0) }
                        else { (134.0 + j * 0.4, 134.0 + j * 0.4, 136.0 + j * 0.4, 255.0) }
                    }
                    74 => { // TNT top
                        let inner = x >= 4 && x <= 11 && y >= 4 && y <= 11;
                        if inner { (62.0 + j * 0.4, 58.0 + j * 0.4, 56.0 + j * 0.4, 255.0) }
                        else { (192.0 + j * 0.3, 52.0 + j * 0.2, 44.0 + j * 0.2, 255.0) }
                    }
                    75 => { // pumpkin top
                        if x >= 7 && x <= 8 && y >= 7 && y <= 8 { (96.0, 132.0, 52.0, 255.0) }
                        else { (222.0 + j * 0.3, 124.0 + j * 0.3, 34.0, 255.0) }
                    }
                    76 => { // chest side
                        let border = x == 0 || y == 0 || x == 15 || y == 15 || y == 5;
                        if border { (78.0, 52.0, 28.0, 255.0) }
                        else { (152.0 + j * 0.4, 104.0 + j * 0.4, 54.0 + j * 0.3, 255.0) }
                    }
                    80 | 81 | 82 | 83 => { // gold, diamond, emerald, iron blocks
                        let rim = x == 0 || y == 0 || x == 15 || y == 15;
                        let shine = (x == 2 && y >= 2 && y <= 8) || (y == 2 && x >= 2 && x <= 8);
                        let base = match k {
                            80 => (246.0, 206.0, 54.0),
                            81 => (82.0, 224.0, 232.0),
                            82 => (62.0, 206.0, 102.0),
                            _ => (220.0, 222.0, 226.0),
                        };
                        let m = if rim { 0.78 } else if shine { 1.12 } else { 1.0 };
                        (base.0 * m + j * 0.2, base.1 * m + j * 0.2, base.2 * m + j * 0.2, 255.0)
                    }
                    _ => (130.0, 130.0, 130.0, 255.0),
                };
                out[i] = clamp255(cr);
                out[i + 1] = clamp255(cg);
                out[i + 2] = clamp255(cb);
                out[i + 3] = clamp255(a);
            }
        }
        out
    }

    pub fn sample_kind(&self, k: usize, u: f32, v: f32, tick: u32, animated: bool) -> [u8; 4] {
        let x = ((u.fract().abs() * TEX as f32) as usize) % TEX;
        let mut y = ((v.fract().abs() * TEX as f32) as usize) % TEX;
        if animated {
            y = (y + (tick / 6) as usize) % TEX;
        }
        let d = &self.data[k];
        let i = (y * TEX + x) * 4;
        [d[i], d[i + 1], d[i + 2], d[i + 3]]
    }

    pub fn sample_face(&self, block: u16, nx: i32, ny: i32, nz: i32, u: f32, v: f32, tick: u32) -> [u8; 4] {
        let k = Self::face_kind(block, nx, ny, nz);
        let animated = block == world::WATER || block == world::LAVA;
        self.sample_kind(k, u, v, tick, animated)
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

fn sky_color(dir_y: f32, time: f32, x: i32, y: i32) -> [u8; 3] {
    let sun = (time * 2.0 * PI).sin();
    let day = day_factor(time);
    let horizon = 0.22 + sun.abs() * 0.1;
    let t = ((dir_y * 0.5 + 0.5) * (1.0 - horizon) + horizon).clamp(0.0, 1.0);
    let night_top = [8.0, 14.0, 36.0];
    let night_bot = [22.0, 26.0, 56.0];
    let day_top = [78.0, 152.0, 238.0];
    let day_bot = [172.0, 212.0, 248.0];
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
    if day < 0.2 && dir_y > 0.02 && th(x, y, 91) % 900 == 0 {
        c = [235.0, 235.0, 220.0];
    }
    [c[0] as u8, c[1] as u8, c[2] as u8]
}

fn cloud_color(cam: &Cam, dx: f32, dy: f32, dz: f32, time: f32, day: f32) -> Option<[u8; 3]> {
    if dy <= 0.03 { return None; }
    let t = (108.0 - cam.y) / dy;
    if t <= 0.0 || t > 260.0 { return None; }
    let wx = ((cam.x + dx * t) / 14.0 + time * 12.0).floor() as i32;
    let wz = ((cam.z + dz * t) / 14.0).floor() as i32;
    let h = th(wx / 2, wz / 2, 777) % 10;
    if h < 3 {
        let v = 195.0 + day * 55.0;
        Some([v as u8, v as u8, (v + 5.0).min(255.0) as u8])
    } else { None }
}

fn face_shade(nx: i32, ny: i32, nz: i32, day: f32) -> f32 {
    let base = if ny > 0 { 1.0 } else if ny < 0 { 0.48 } else if nz != 0 { 0.80 } else { 0.65 };
    let sun = 0.34 + day * 0.66;
    let direct = if ny > 0 { day * 0.14 } else if nx > 0 { day * 0.04 } else { 0.0 };
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
                let fade = 0.45 + day * 0.45;
                r = r * (1.0 - fade) + cc[0] as f32 * fade;
                g = g * (1.0 - fade) + cc[1] as f32 * fade;
                b = b * (1.0 - fade) + cc[2] as f32 * fade;
            }
            if let Some(hit) = world::raycast(world, cam.x, cam.y, cam.z, dx, dy, dz, 76.0, true) {
                if hit.block == world::WATER || hit.block == world::LAVA {
                    let behind = world::raycast(
                        world,
                        cam.x + dx * (hit.dist + 0.04),
                        cam.y + dy * (hit.dist + 0.04),
                        cam.z + dz * (hit.dist + 0.04),
                        dx, dy, dz, 76.0 - hit.dist, false,
                    );
                    if let Some(bh) = behind {
                        let (br, bg, bb, _) = shade_hit(tex, world, &bh, time, day, tick);
                        let fog = (1.0 - (-bh.dist * 0.010).exp()).min(0.85);
                        r = br * (1.0 - fog) + sky[0] as f32 * fog;
                        g = bg * (1.0 - fog) + sky[1] as f32 * fog;
                        b = bb * (1.0 - fog) + sky[2] as f32 * fog;
                    }
                    let t = tex.sample_face(hit.block, hit.nx, hit.ny, hit.nz, hit.u, hit.v, tick);
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
                    } else {
                        fb.depth[py * fb.w + px] = hit.dist;
                    }
                }
            }
            if fb.depth[py * fb.w + px].is_infinite() {
                let ang = time * 2.0 * PI;
                let mut sx = ang.cos();
                let mut sy = ang.sin();
                let mut sz = 0.22f32;
                let sl = (sx * sx + sy * sy + sz * sz).sqrt();
                sx /= sl; sy /= sl; sz /= sl;
                // Square Minecraft sun & moon
                if sy > -0.1 && (dx - sx).abs() < 0.062 && (dy - sy).abs() < 0.062 && (dz - sz).abs() < 0.062 {
                    r = 255.0; g = 246.0; b = 185.0;
                }
                if -sy > -0.1 && (dx + sx).abs() < 0.048 && (dy + sy).abs() < 0.048 && (dz + sz).abs() < 0.048 {
                    r = 225.0; g = 230.0; b = 242.0;
                }
                if day < 0.35 && dy > 0.05 {
                    let st = th((dx * 220.0) as i32, (dz * 220.0 + dy * 140.0) as i32, 91);
                    if st % 520 == 0 {
                        let br = (0.35 - day) * 2.5;
                        r = (r + 180.0 * br).min(255.0);
                        g = (g + 190.0 * br).min(255.0);
                        b = (b + 220.0 * br).min(255.0);
                    }
                }
            }
            fb.rgba[(py * fb.w + px) * 4] = r as u8;
            fb.rgba[(py * fb.w + px) * 4 + 1] = g as u8;
            fb.rgba[(py * fb.w + px) * 4 + 2] = b as u8;
            fb.rgba[(py * fb.w + px) * 4 + 3] = 255;
        }
    }
}

fn shade_hit(tex: &Textures, world: &mut World, h: &Hit, _time: f32, day: f32, tick: u32) -> (f32, f32, f32, u8) {
    let t = tex.sample_face(h.block, h.nx, h.ny, h.nz, h.u, h.v, tick);
    let mut light = face_shade(h.nx, h.ny, h.nz, day) * ao(world, h);
    let emit = world::emits_light(h.block);
    if emit > 0 { light = light.max(0.95 + emit as f32 * 0.02); }
    if h.block == world::LAMP && world.lit.contains(&(h.x, h.y, h.z)) { light = 1.45; }
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
                if d <= td + 0.12 {
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
            if y == 3 && x >= 1 && x <= 6 { return [72, 48, 30]; } // unibrow
            if y == 4 && (x == 1 || x == 6) { return [245, 245, 245]; }
            if y == 4 && (x == 2 || x == 5) { return [52, 152, 72]; } // emerald eyes
            base
        }
        FaceArt::None => base,
    }
}

fn transform_point(lx: f32, ly: f32, lz: f32, part: &Part3D, sy: f32, cy: f32, wx: f32, wy: f32, wz: f32) -> (f32, f32, f32) {
    // swing around pivot_y in X axis
    let dy = ly - part.pivot_y;
    let dz = lz;
    let (ss, cs) = (part.swing.sin(), part.swing.cos());
    let ry = part.pivot_y + dy * cs - dz * ss;
    let rz = dy * ss + dz * cs;
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
    flash: bool,
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

            let c = if flash {
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
                [
                    (raw[0] as f32 * shade).min(255.0) as u8,
                    (raw[1] as f32 * shade).min(255.0) as u8,
                    (raw[2] as f32 * shade).min(255.0) as u8,
                ]
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
    flash: bool,
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
    let _ = tick;
    if item == 0 { return [0, 0, 0, 0]; }
    if item >= 256 {
        let xi = x as i32; let yi = y as i32;
        match item {
            256..=260 => { // pickaxe
                let head = match item {
                    256 => [165, 118, 70, 255],
                    257 => [145, 145, 152, 255],
                    258 => [220, 224, 232, 255],
                    259 => [68, 225, 232, 255],
                    _ => [72, 62, 74, 255],
                };
                if (yi == 2 && xi >= 4 && xi <= 11) || (yi == 3 && (xi == 3 || xi == 12 || (xi >= 8 && xi <= 10))) || (yi == 4 && xi == 12) { return head; }
                if (xi + yi == 15) && xi >= 3 && xi <= 11 { return [132, 88, 48, 255]; }
            }
            261..=263 => { // axe
                let head = match item { 261 => [165, 118, 70, 255], 262 => [220, 224, 232, 255], _ => [68, 225, 232, 255] };
                if xi >= 7 && xi <= 12 && yi >= 2 && yi <= 6 && (xi + yi >= 10) { return head; }
                if (xi + yi == 15) && xi >= 3 && xi <= 11 { return [132, 88, 48, 255]; }
            }
            264..=265 => { // shovel
                let head = if item == 264 { [165, 118, 70, 255] } else { [220, 224, 232, 255] };
                if xi >= 9 && xi <= 13 && yi >= 2 && yi <= 6 { return head; }
                if (xi + yi == 15) && xi >= 3 && xi <= 10 { return [132, 88, 48, 255]; }
            }
            266..=268 | 312 => { // swords
                let blade = match item {
                    266 => [165, 118, 70, 255],
                    267 => [220, 224, 232, 255],
                    268 => [68, 225, 232, 255],
                    _ => [72, 62, 74, 255],
                };
                if (xi + yi == 15 || xi + yi == 14) && xi >= 6 && xi <= 13 && yi >= 2 && yi <= 9 { return blade; }
                if (xi - yi == 5 || (xi >= 4 && xi <= 7 && yi >= 9 && yi <= 11)) && xi + yi >= 13 { return [110, 74, 42, 255]; }
                if xi >= 2 && xi <= 4 && yi >= 11 && yi <= 13 { return [138, 94, 52, 255]; }
            }
            269 => { // apple
                if (xi - 8).pow(2) + (yi - 9).pow(2) <= 22 { return if xi <= 6 && yi <= 8 { [248, 120, 110, 255] } else { [214, 38, 42, 255] }; }
                if xi == 8 && yi >= 2 && yi <= 4 { return [110, 72, 38, 255]; }
                if xi >= 9 && xi <= 11 && yi == 3 { return [72, 162, 56, 255]; }
            }
            307 => { // golden apple
                if (xi - 8).pow(2) + (yi - 9).pow(2) <= 22 { return if xi <= 6 && yi <= 8 { [255, 244, 150, 255] } else { [246, 198, 42, 255] }; }
                if xi == 8 && yi >= 2 && yi <= 4 { return [110, 72, 38, 255]; }
            }
            270 => { if xi >= 3 && xi <= 12 && yi >= 5 && yi <= 11 { return if yi <= 7 { [224, 174, 92, 255] } else { [186, 128, 56, 255] }; } }
            271..=277 => {
                let raw = matches!(item, 272 | 273 | 275);
                if xi >= 3 && xi <= 12 && yi >= 5 && yi <= 11 {
                    return if raw { [218, 108, 108, 255] } else { [162, 86, 52, 255] };
                }
            }
            278 => { if (xi - 8).pow(2) + (yi - 8).pow(2) <= 20 { return [38, 38, 42, 255]; } }
            279 => { if xi >= 3 && xi <= 12 && yi >= 5 && yi <= 10 { return if yi <= 6 { [242, 244, 248, 255] } else { [202, 206, 212, 255] }; } }
            280 => { if xi >= 3 && xi <= 12 && yi >= 5 && yi <= 10 { return if yi <= 6 { [255, 234, 118, 255] } else { [242, 188, 44, 255] }; } }
            281 => { if (xi - 8).abs() + (yi - 8).abs() <= 5 { return if xi <= 8 && yi <= 7 { [160, 250, 255, 255] } else { [62, 215, 228, 255] }; } }
            282 => { if (xi - 8).abs() + (yi - 8).abs() <= 5 { return [52, 214, 102, 255]; } }
            313 => { if xi >= 3 && xi <= 12 && yi >= 5 && yi <= 10 { return [72, 62, 74, 255]; } }
            317 => { if (xi - 8).abs() + (yi - 8).abs() <= 5 { return [184, 102, 232, 255]; } }
            283 => { if (xi + yi == 15) && xi >= 3 && xi <= 12 { return [142, 98, 54, 255]; } }
            284 => { if xi >= 5 && xi <= 10 && yi >= 5 && yi <= 11 { return [62, 62, 72, 255]; } }
            285 => { if (xi + yi == 15 || xi + yi == 14) && xi >= 4 && xi <= 12 { return [238, 234, 224, 255]; } }
            286 => { if xi >= 4 && xi <= 11 && yi >= 4 && yi <= 11 { return [146, 86, 48, 255]; } }
            287 => { if (xi + yi == 15 && xi >= 4 && xi <= 11) || (xi <= 5 && yi >= 10 && yi <= 12) || (xi >= 10 && yi >= 3 && yi <= 5) { return [242, 240, 228, 255]; } }
            288 => { if xi >= 4 && xi <= 11 && yi >= 5 && yi <= 11 { return [142, 48, 38, 255]; } }
            289 => { if (xi - 8).abs() <= 3 && (yi - 8).abs() <= 4 { return [225, 225, 222, 255]; } }
            290 => { if xi >= 3 && xi <= 12 && yi >= 4 && yi <= 12 { return [235, 235, 232, 255]; } }
            291 => { if (xi - 8).pow(2) + (yi - 8).pow(2) <= 20 { return [246, 238, 212, 255]; } }
            292..=294 => {
                if xi >= 4 && xi <= 11 && yi >= 5 && yi <= 12 {
                    if yi <= 6 && item == 293 { return [54, 128, 222, 255]; }
                    if yi <= 6 && item == 294 { return [242, 112, 24, 255]; }
                    return [182, 186, 192, 255];
                }
            }
            295 => {
                if (xi - 8).pow(2) + (yi - 8).pow(2) <= 25 {
                    if (xi - 8).abs() <= 1 && yi >= 4 && yi <= 9 { return [220, 42, 42, 255]; }
                    return [135, 142, 152, 255];
                }
            }
            296 => { if xi >= 3 && xi <= 12 && yi >= 3 && yi <= 12 { return [224, 206, 162, 255]; } }
            297 => { if (xi - 8).pow(2) + (yi - 8).pow(2) <= 24 { return if xi < 8 { [90, 140, 210, 255] } else { [242, 206, 68, 255] }; } }
            298 => { if (xi - 8).abs() <= 4 && (yi - 8).abs() <= 5 { return [134, 118, 168, 255]; } }
            299 => { if xi >= 7 && xi <= 9 && yi >= 4 && yi <= 12 { return [220, 72, 54, 255]; } }
            300 => { if xi >= 4 && xi <= 11 && yi >= 3 && yi <= 12 { return if xi == 4 || xi == 11 { [175, 180, 186, 255] } else { [146, 98, 56, 255] }; } }
            301 => { if (xi - 8).abs() <= 3 && (yi - 8).abs() <= 4 { return if yi <= 5 { [72, 205, 112, 255] } else { [244, 208, 62, 255] }; } }
            302 => { if (xi - 8).pow(2) + (yi - 8).pow(2) <= 20 { return [24, 154, 134, 255]; } }
            303 => { if xi >= 7 && xi <= 8 && yi >= 2 && yi <= 13 { return [248, 184, 38, 255]; } }
            304 => { if (xi - 8).abs() + (yi - 8).abs() <= 5 { return [252, 252, 238, 255]; } }
            305 => { if xi >= 5 && xi <= 10 && yi >= 5 && yi <= 12 { return [124, 224, 92, 255]; } }
            306 => { if (xi - 8).pow(2) + (yi - 8).pow(2) <= 22 { return if (xi + yi) % 2 == 0 { [116, 116, 122, 255] } else { [74, 74, 78, 255] }; } }
            308 => { // bow
                if xi == 5 && yi >= 3 && yi <= 12 { return [215, 215, 215, 255]; }
                if (xi >= 6 && xi <= 10 && (yi == 3 || yi == 12)) || (xi == 11 && yi >= 4 && yi <= 11) { return [142, 96, 52, 255]; }
            }
            309 => { // arrow
                if xi + yi == 15 && xi >= 4 && xi <= 11 { return [142, 98, 54, 255]; }
                if xi >= 11 && yi <= 4 && xi + yi <= 16 { return [185, 190, 196, 255]; }
                if xi <= 4 && yi >= 11 { return [236, 234, 228, 255]; }
            }
            310 => { // flint and steel
                if xi >= 4 && xi <= 7 && yi >= 5 && yi <= 11 { return [192, 196, 202, 255]; }
                if xi >= 9 && xi <= 12 && yi >= 6 && yi <= 10 { return [62, 62, 72, 255]; }
            }
            311 => { // shears
                if (xi >= 4 && xi <= 8 && yi >= 4 && yi <= 8) || (xi >= 8 && xi <= 12 && yi >= 7 && yi <= 11) { return [196, 200, 206, 255]; }
            }
            314 => { // carrot
                if xi >= 5 && xi <= 10 && yi >= 6 && yi <= 12 { return [242, 128, 32, 255]; }
                if xi >= 7 && xi <= 9 && yi >= 3 && yi <= 5 { return [68, 168, 52, 255]; }
            }
            315 => { // cookie
                if (xi - 8).pow(2) + (yi - 8).pow(2) <= 22 {
                    return if (xi + yi * 3) % 5 == 0 { [68, 42, 24, 255] } else { [214, 158, 92, 255] };
                }
            }
            316 => { if (xi - 8).pow(2) + (yi - 8).pow(2) <= 22 { return [106, 212, 92, 255]; } }
            318 | 320 => { // helmet
                let c = if item == 318 { [210, 214, 222, 255] } else { [68, 222, 232, 255] };
                if xi >= 4 && xi <= 11 && yi >= 4 && yi <= 10 && !(xi >= 6 && xi <= 9 && yi >= 8) { return c; }
            }
            319 | 321 => { // chestplate
                let c = if item == 319 { [210, 214, 222, 255] } else { [68, 222, 232, 255] };
                if (xi >= 3 && xi <= 12 && yi >= 4 && yi <= 7) || (xi >= 5 && xi <= 10 && yi >= 4 && yi <= 13) { return c; }
            }
            322..=331 => { // spawn eggs (oval egg with spots!)
                let d2 = ((xi - 8) * 5).pow(2) + ((yi - 8) * 4).pow(2);
                if d2 <= 25 * 16 {
                    let (base, spot) = match item {
                        322 => ([232, 228, 220, 255], [220, 172, 150, 255]), // sheep
                        323 => ([82, 58, 42, 255], [228, 228, 224, 255]),   // cow
                        324 => ([242, 166, 174, 255], [218, 98, 118, 255]), // pig
                        325 => ([242, 242, 238, 255], [222, 48, 48, 255]),  // chicken
                        326 => ([48, 136, 150, 255], [92, 144, 72, 255]),   // zombie
                        327 => ([218, 218, 210, 255], [78, 78, 82, 255]),   // skeleton
                        328 => ([42, 168, 52, 255], [16, 36, 16, 255]),     // creeper
                        329 => ([48, 42, 46, 255], [210, 32, 32, 255]),     // spider
                        330 => ([22, 20, 28, 255], [210, 110, 248, 255]),   // enderman
                        _ => ([122, 82, 52, 255], [210, 162, 126, 255]),    // villager
                    };
                    return if (xi + yi * 2) % 5 == 0 { spot } else { base };
                }
            }
            _ => { if xi >= 4 && xi <= 11 && yi >= 4 && yi <= 11 { return [190, 190, 195, 255]; } }
        }
        return [0, 0, 0, 0];
    }

    // Flat item sprites for non-cubic blocks
    if matches!(item, world::TORCH | world::REDSTONE_TORCH) {
        if x >= 7 && x <= 8 && y >= 6 && y <= 13 { return [132, 88, 48, 255]; }
        if x >= 6 && x <= 9 && y >= 3 && y <= 5 {
            return if item == world::REDSTONE_TORCH { [248, 48, 36, 255] } else { [252, 192, 52, 255] };
        }
        return [0, 0, 0, 0];
    }
    if item == world::REDSTONE_DUST {
        if (x as i32 - 8).pow(2) + (y as i32 - 8).pow(2) <= 22 { return if (x + y) % 2 == 0 { [224, 34, 34, 255] } else { [172, 20, 20, 255] }; }
        return [0, 0, 0, 0];
    }
    if item == world::SIGN {
        if x >= 2 && x <= 13 && y >= 3 && y <= 9 { return [178, 134, 82, 255]; }
        if x >= 7 && x <= 8 && y >= 9 && y <= 13 { return [132, 92, 48, 255]; }
        return [0, 0, 0, 0];
    }
    if item == world::DOOR {
        if x >= 4 && x <= 11 && y >= 2 && y <= 14 {
            if x == 9 && y == 8 { return [225, 195, 90, 255]; }
            return [154, 108, 60, 255];
        }
        return [0, 0, 0, 0];
    }
    if matches!(item, world::FLOWER_RED | world::FLOWER_YELLOW | world::TALL_GRASS | world::WHEAT | world::RAIL) {
        return [90, 152, 58, if x >= 5 && x <= 10 && y >= 3 && y <= 13 { 255 } else { 0 }];
    }
    // Fallback solid tint
    let (r, g, b) = block_rgb(item);
    [r, g, b, 255]
}

fn block_rgb(item: u16) -> (u8, u8, u8) {
    match item {
        world::GRASS => (88, 152, 54), world::DIRT => (130, 92, 62), world::STONE => (126, 126, 128),
        world::COBBLE => (134, 134, 136), world::SAND | world::SANDSTONE => (222, 208, 160),
        world::RED_SAND => (196, 108, 52), world::WATER => (55, 118, 205),
        world::LEAVES | world::JUNGLE_LEAVES => (56, 114, 40),
        world::SPRUCE_LEAVES | world::DARK_LEAVES => (42, 86, 48),
        world::BIRCH_LEAVES => (92, 152, 64),
        world::LOG | world::JUNGLE_LOG => (106, 80, 50),
        world::BIRCH_LOG => (224, 220, 210),
        world::SPRUCE_LOG | world::DARK_LOG => (74, 52, 34),
        world::PLANKS | world::SLAB | world::STAIRS | world::FENCE => (178, 134, 82),
        world::GLASS => (185, 224, 238),
        world::COAL_ORE => (82, 82, 86), world::IRON_ORE => (182, 156, 132),
        world::GOLD_ORE => (232, 198, 72), world::DIAMOND_ORE => (84, 212, 222),
        world::REDSTONE_ORE => (206, 62, 62), world::EMERALD_ORE => (68, 202, 102),
        world::WOOL_RED => (182, 48, 46), world::WOOL_BLUE => (52, 86, 182),
        world::WOOL_GREEN => (54, 128, 54), world::WOOL_YELLOW => (224, 192, 52),
        world::WOOL_WHITE | world::CONCRETE_WHITE | world::QUARTZ => (235, 235, 232),
        world::LAVA => (238, 108, 28), world::BEDROCK => (48, 48, 50),
        world::SNOW | world::SNOW_LAYER => (240, 246, 250), world::ICE => (155, 208, 245),
        world::BRICK => (152, 72, 58), world::STONE_BRICKS => (124, 124, 128),
        world::CACTUS => (64, 134, 50), world::TNT => (196, 48, 42),
        world::OBSIDIAN | world::BLACKSTONE => (38, 30, 48),
        world::CHEST => (152, 104, 54), world::FURNACE => (126, 126, 128),
        world::CRAFTING => (176, 128, 76), world::BOOKSHELF => (162, 118, 72),
        world::CONCRETE_BLUE => (54, 92, 184), world::CONCRETE_RED => (188, 48, 48),
        world::CONCRETE_GREEN => (52, 134, 58), world::CONCRETE_YELLOW => (224, 192, 48),
        world::CONCRETE_BLACK => (38, 38, 42),
        world::GOLD_BLOCK => (246, 206, 54), world::DIAMOND_BLOCK => (82, 224, 232),
        world::EMERALD_BLOCK => (62, 206, 102), world::IRON_BLOCK => (220, 222, 226),
        world::LAMP => (208, 166, 88), world::GLOWSTONE => (248, 224, 146),
        world::PUMPKIN => (222, 124, 34), world::HAY => (212, 182, 72),
        _ => (156, 156, 160),
    }
}

fn is_flat_block_icon(item: u16) -> bool {
    matches!(
        item,
        world::TORCH | world::REDSTONE_TORCH | world::REDSTONE_DUST | world::LEVER
            | world::PRESSURE_PLATE | world::FLOWER_RED | world::FLOWER_YELLOW
            | world::TALL_GRASS | world::WHEAT | world::RAIL | world::SIGN | world::DOOR
    )
}

/// Renders true isometric 3D cubes for blocks and crisp pixel art for items!
pub fn draw_item_icon(fb: &mut Framebuffer, fb_x: i32, fb_y: i32, item: u16, size: i32, tick: u32) {
    if item == 0 || size <= 0 { return; }
    if item < 256 && !is_flat_block_icon(item) && size >= 10 {
        let (top_rgb, side_rgb) = match item {
            world::GRASS => ((88, 152, 54), (130, 92, 62)),
            world::PODZOL => ((98, 70, 44), (130, 92, 62)),
            world::MYCELIUM => ((124, 108, 128), (130, 92, 62)),
            world::LOG | world::JUNGLE_LOG => ((182, 146, 96), (106, 80, 50)),
            world::BIRCH_LOG => ((182, 146, 96), (224, 220, 210)),
            world::SPRUCE_LOG | world::DARK_LOG => ((182, 146, 96), (74, 52, 34)),
            world::CRAFTING => ((176, 128, 76), (146, 102, 58)),
            world::BOOKSHELF => ((178, 134, 82), (168, 74, 58)),
            world::TNT => ((192, 52, 44), (232, 230, 224)),
            _ => {
                let c = block_rgb(item);
                (c, c)
            }
        };
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
                    let shade = 1.06;
                    fb.set(
                        fb_x + px,
                        fb_y + py,
                        (top_rgb.0 as f32 * shade).min(255.0) as u8,
                        (top_rgb.1 as f32 * shade).min(255.0) as u8,
                        (top_rgb.2 as f32 * shade).min(255.0) as u8,
                        255,
                    );
                    continue;
                }
                // Left side face (-rx..0)
                if fx >= -rx && fx < 0.0 {
                    let top_edge = (fx + rx) / rx * ry;
                    if fy >= top_edge && fy <= top_edge + h {
                        let c = if item == world::GRASS && fy < top_edge + h * 0.25 {
                            top_rgb
                        } else if item == world::TNT && fy > top_edge + h * 0.3 && fy < top_edge + h * 0.65 {
                            (240, 238, 232)
                        } else if item == world::TNT {
                            (196, 46, 38)
                        } else {
                            side_rgb
                        };
                        let shade = 0.80;
                        fb.set(fb_x + px, fb_y + py, (c.0 as f32 * shade) as u8, (c.1 as f32 * shade) as u8, (c.2 as f32 * shade) as u8, 255);
                        continue;
                    }
                }
                // Right side face (0..rx)
                if fx >= 0.0 && fx <= rx {
                    let top_edge = (rx - fx) / rx * ry;
                    if fy >= top_edge && fy <= top_edge + h {
                        let c = if item == world::GRASS && fy < top_edge + h * 0.25 {
                            top_rgb
                        } else if item == world::TNT && fy > top_edge + h * 0.3 && fy < top_edge + h * 0.65 {
                            (240, 238, 232)
                        } else if item == world::TNT {
                            (196, 46, 38)
                        } else {
                            side_rgb
                        };
                        let shade = 0.62;
                        fb.set(fb_x + px, fb_y + py, (c.0 as f32 * shade) as u8, (c.1 as f32 * shade) as u8, (c.2 as f32 * shade) as u8, 255);
                    }
                }
            }
        }
        return;
    }

    for y in 0..size {
        for x in 0..size {
            let c = item_pixel(item, (x as u32 * 16 / size as u32).min(15), (y as u32 * 16 / size as u32).min(15), tick);
            if c[3] > 0 { fb.set(fb_x + x, fb_y + y, c[0], c[1], c[2], c[3]); }
        }
    }
}

// ===================== HUD & Menus =====================

fn heart(fb: &mut Framebuffer, x: i32, y: i32, state: u8) {
    // state: 0=empty, 1=half, 2=full
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
    let x = 414;
    let y = 193;
    fb.rect(x + 20, y + 28, 18, 42, [220, 175, 135, 255]);
    fb.rect(x + 20, y + 28, 18, 3, [245, 205, 165, 255]);
    fb.rect(x + 17, y + 62, 24, 5, [60, 135, 155, 255]);
    if item != 0 { draw_item_icon(fb, x, y, item, 40, tick); }
}

pub fn draw_hud(
    fb: &mut Framebuffer,
    hot: &[u16; 9],
    counts: &[u32; 9],
    selected: usize,
    health: f32,
    hunger: f32,
    tick: u32,
    message: &str,
    message_timer: f32,
    biome: Biome,
    pos: (f32, f32, f32),
    fps: u32,
    camera: u8,
    time: f32,
    mode: GameMode,
) {
    let hb_left = hotbar_x(0);
    if mode != GameMode::Spectator {
        // Hotbar background + beveled Minecraft slots
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
            for i in 0..10 {
                let threshold = (i as f32 + 1.0) * 2.0;
                let st = if health >= threshold { 2 } else if health >= threshold - 1.0 { 1 } else { 0 };
                heart(fb, hb_left + i * 9, HOT_Y - 16, st);
            }
            for i in 0..10 {
                let filled = hunger >= (i as f32 + 1.0) * 2.0 - 0.5;
                drumstick(fb, hb_left + 9 * (SLOT + 2) - 8 - i * 8, HOT_Y - 16, filled);
            }
            // XP bar
            fb.rect(hb_left, HOT_Y - 6, 9 * (SLOT + 2) - 2, 3, [20, 20, 20, 220]);
            fb.rect(hb_left, HOT_Y - 6, 118, 3, [122, 232, 68, 255]);
        }
    }

    // Crosshair
    let cx = W as i32 / 2; let cy = H as i32 / 2;
    fb.rect(cx - 4, cy, 9, 1, [240, 240, 240, 200]);
    fb.rect(cx, cy - 4, 1, 9, [240, 240, 240, 200]);

    text_shadow(fb, 8, 8, &format!("{}  XYZ {:.0} {:.0} {:.0}", biome.name(), pos.0, pos.1, pos.2), [240, 240, 240, 255], 1);
    text_shadow(fb, 8, 18, &format!("FPS {}  CAM {}  TIME {:02}:00  [{}]", fps, camera, ((time * 24.0) as i32) % 24, mode.name()), [200, 230, 255, 255], 1);

    if message_timer > 0.0 {
        let w = message.len() as i32 * 6 + 10;
        let my = if mode == GameMode::Spectator { H as i32 - 36 } else { HOT_Y - 30 };
        fb.rect(W as i32 / 2 - w / 2, my, w, 12, [0, 0, 0, 175]);
        text_shadow(fb, W as i32 / 2 - w / 2 + 5, my + 3, message, [255, 255, 255, 255], 1);
    }
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
    for y in 0..H as i32 {
        let f = y as f32 / H as f32;
        let r = (108.0 * (1.0 - f) + 178.0 * f) as u8;
        let g = (165.0 * (1.0 - f) + 212.0 * f) as u8;
        let b = (232.0 * (1.0 - f) + 252.0 * f) as u8;
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
    for x in 0..W as i32 {
        let h1 = 196.0 + (x as f32 * 0.012 + 1.2).sin() * 12.0 + (x as f32 * 0.031).sin() * 5.0;
        let h2 = 208.0 + (x as f32 * 0.008 + 2.7).sin() * 18.0;
        fb.rect(x, h1 as i32, 1, H as i32 - h1 as i32, [96, 156, 74, 255]);
        fb.rect(x, h2 as i32, 1, H as i32 - h2 as i32, [74, 128, 58, 255]);
    }
    fb.rect(0, 238, W as i32, 32, [122, 88, 62, 255]);
    for x in (0..W as i32).step_by(12) { fb.rect(x, 238, 6, 4, [104, 74, 52, 255]); }

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

pub fn draw_inventory_screen(
    fb: &mut Framebuffer,
    hot: &[crate::craft::Stack; 9],
    store: &[crate::craft::Stack; 27],
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
    let (px, py, pw, ph) = (60, 14, 360, 242);
    panel(fb, px, py, pw, ph);
    text(fb, 76, 20, if grid_size == 3 { "CRAFTING (3x3)" } else { "CRAFTING (2x2)" }, [60, 60, 60, 255], 1);

    let mut tip = String::new();
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

pub fn draw_options_screen(fb: &mut Framebuffer, sound: bool, music: bool, scale: i32, hover: usize) {
    fb.rect(0, 0, W as i32, H as i32, [0, 0, 0, 145]);
    let (px, py, pw, ph) = (70, 30, 340, 210);
    panel(fb, px, py, pw, ph);
    text(fb, px + 10, py + 8, "OPTIONS", [60, 60, 60, 255], 1);
    let items = [
        format!("SOUND: {}", if sound { "ON" } else { "OFF" }),
        format!("MUSIC: {}", if music { "ON" } else { "OFF" }),
        format!("PIXEL SCALE: {}x", scale),
        "DONE".to_string(),
    ];
    for (i, label) in items.iter().enumerate() {
        button(fb, px + 60, py + 44 + i as i32 * 30, pw - 120, 22, label, hover == i);
    }
    text(fb, px + 10, py + ph - 16, "Click options to cycle values", [90, 90, 90, 255], 1);
}
