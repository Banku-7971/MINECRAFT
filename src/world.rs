// Multi-dimension procedural voxel world: Overworld, The Nether, The End,
// biomes, caves, ores, trees, villages, fortresses, ruined portals, strongholds, and RenderCache.
use std::collections::{HashMap, HashSet};

pub const WORLD_H: usize = 96;
pub const CHUNK: i32 = 16;
pub const SEA: i32 = 30;

pub const AIR: u16 = 0;
pub const STONE: u16 = 1;
pub const GRASS: u16 = 2;
pub const DIRT: u16 = 3;
pub const COBBLE: u16 = 4;
pub const LOG: u16 = 5;
pub const LEAVES: u16 = 6;
pub const SAND: u16 = 7;
pub const SANDSTONE: u16 = 8;
pub const WATER: u16 = 9;
pub const SNOW: u16 = 10;
pub const ICE: u16 = 11;
pub const SPRUCE_LOG: u16 = 12;
pub const SPRUCE_LEAVES: u16 = 13;
pub const JUNGLE_LOG: u16 = 14;
pub const JUNGLE_LEAVES: u16 = 15;
pub const PLANKS: u16 = 16;
pub const GLASS: u16 = 17;
pub const TORCH: u16 = 18;
pub const BEDROCK: u16 = 19;
pub const COAL_ORE: u16 = 20;
pub const IRON_ORE: u16 = 21;
pub const GOLD_ORE: u16 = 22;
pub const DIAMOND_ORE: u16 = 23;
pub const REDSTONE_ORE: u16 = 24;
pub const GRAVEL: u16 = 25;
pub const CLAY: u16 = 26;
pub const CACTUS: u16 = 27;
pub const TALL_GRASS: u16 = 28;
pub const FLOWER_RED: u16 = 29;
pub const FLOWER_YELLOW: u16 = 30;
pub const BRICK: u16 = 31;
pub const STONE_BRICKS: u16 = 32;
pub const WOOL_WHITE: u16 = 33;
pub const WOOL_RED: u16 = 34;
pub const WOOL_BLUE: u16 = 35;
pub const WOOL_GREEN: u16 = 36;
pub const WOOL_YELLOW: u16 = 37;
pub const BLACKSTONE: u16 = 38;
pub const MAGMA: u16 = 39;
pub const LAVA: u16 = 40;
pub const FARMLAND: u16 = 41;
pub const WHEAT: u16 = 42;
pub const CHEST: u16 = 43;
pub const CRAFTING: u16 = 44;
pub const FURNACE: u16 = 45;
pub const BIRCH_LOG: u16 = 46;
pub const BIRCH_LEAVES: u16 = 47;
pub const BROWN_MUSHROOM: u16 = 48;
pub const RED_MUSHROOM: u16 = 49;
pub const PUMPKIN: u16 = 50;
pub const HAY: u16 = 51;
pub const TNT: u16 = 52;
pub const OBSIDIAN: u16 = 53;
pub const GLOWSTONE: u16 = 54;
pub const SEA_LANTERN: u16 = 55;
pub const DARK_LOG: u16 = 56;
pub const DARK_LEAVES: u16 = 57;
pub const MOSSY: u16 = 58;
pub const COBWEB: u16 = 59;
pub const VINE: u16 = 60;
pub const DOOR: u16 = 61;
pub const IRON_BLOCK: u16 = 62;
pub const GOLD_BLOCK: u16 = 63;
pub const DIAMOND_BLOCK: u16 = 64;
pub const EMERALD_ORE: u16 = 65;
pub const EMERALD_BLOCK: u16 = 66;
pub const SNOW_LAYER: u16 = 67;
pub const PODZOL: u16 = 68;
pub const MYCELIUM: u16 = 69;
pub const RED_SAND: u16 = 70;
pub const TERRACOTTA: u16 = 71;
pub const CONCRETE_WHITE: u16 = 72;
pub const CONCRETE_BLUE: u16 = 73;
pub const CONCRETE_RED: u16 = 74;
pub const CONCRETE_GREEN: u16 = 75;
pub const CONCRETE_YELLOW: u16 = 76;
pub const CONCRETE_BLACK: u16 = 77;
pub const LANTERN: u16 = 78;
pub const CAMPFIRE: u16 = 79;
pub const BOOKSHELF: u16 = 80;
pub const BED: u16 = 81;
pub const CAKE: u16 = 82;
pub const LADDER: u16 = 83;
pub const RAIL: u16 = 84;
pub const ANVIL: u16 = 85;
pub const ENCHANTING: u16 = 86;
pub const END_STONE: u16 = 87;
pub const NETHERRACK: u16 = 88;
pub const SOUL_SAND: u16 = 89;
pub const QUARTZ: u16 = 90;
pub const SLAB: u16 = 91;
pub const STAIRS: u16 = 92;
pub const FENCE: u16 = 93;
pub const SIGN: u16 = 94;
pub const REDSTONE_DUST: u16 = 96;
pub const REDSTONE_TORCH: u16 = 97;
pub const LEVER: u16 = 98;
pub const LAMP: u16 = 99;
pub const PRESSURE_PLATE: u16 = 100;
pub const NETHER_PORTAL: u16 = 101;
pub const END_PORTAL_FRAME: u16 = 102;
pub const END_PORTAL: u16 = 103;
pub const CRYING_OBSIDIAN: u16 = 104;
pub const CRIMSON_NYLIUM: u16 = 105;
pub const WARPED_NYLIUM: u16 = 106;
pub const CRIMSON_STEM: u16 = 107;
pub const CRIMSON_WART: u16 = 108;
pub const WARPED_STEM: u16 = 109;
pub const WARPED_WART: u16 = 110;
pub const BASALT: u16 = 111;
pub const SHROOMLIGHT: u16 = 112;
pub const NETHER_QUARTZ_ORE: u16 = 113;
pub const ANCIENT_DEBRIS: u16 = 114;
pub const CHORUS_PLANT: u16 = 115;
pub const CHORUS_FLOWER: u16 = 116;
pub const PURPUR_BLOCK: u16 = 117;
pub const NETHER_BRICKS: u16 = 118;
pub const END_PORTAL_FRAME_EYE: u16 = 119;
pub const DOOR_OPEN: u16 = 120;
pub const LAPIS_ORE: u16 = 121;
pub const LAPIS_BLOCK: u16 = 122;
pub const BREWING_STAND: u16 = 123;
pub const BEACON: u16 = 124;
pub const HOPPER: u16 = 125;
pub const PISTON: u16 = 126;
pub const STICKY_PISTON: u16 = 127;
pub const REPEATER: u16 = 128;
pub const COMPARATOR: u16 = 129;
pub const SPAWNER: u16 = 130;
pub const SAPLING: u16 = 131;
pub const NETHER_WART: u16 = 132;
pub const BUTTON: u16 = 133;
pub const IRON_DOOR: u16 = 134;
pub const IRON_DOOR_OPEN: u16 = 135;
pub const MAX_BLOCK: u16 = 135;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Dimension {
    Overworld = 0,
    Nether = 1,
    End = 2,
}

impl Dimension {
    pub fn idx(self) -> usize { self as usize }
    pub fn from_idx(v: u8) -> Self {
        match v {
            1 => Dimension::Nether,
            2 => Dimension::End,
            _ => Dimension::Overworld,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Dimension::Overworld => "Overworld",
            Dimension::Nether => "The Nether",
            Dimension::End => "The End",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct BPos(pub i32, pub i32, pub i32);

fn hash2(x: i32, z: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x9e3779b1) ^ (z as u32).wrapping_mul(0x85ebca6b);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297a2d39);
    h ^= h >> 15;
    h
}

fn hash3(x: i32, y: i32, z: i32) -> u32 {
    let mut h = hash2(x, z) ^ (y as u32).wrapping_mul(0x27d4eb2d);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb352d);
    h ^= h >> 15;
    h
}

fn fade(t: f32) -> f32 { t * t * (3.0 - 2.0 * t) }
fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }

pub fn noise2(x: f32, z: f32) -> f32 {
    let xi = x.floor() as i32;
    let zi = z.floor() as i32;
    let tx = fade(x - xi as f32);
    let tz = fade(z - zi as f32);
    let a = (hash2(xi, zi) as f32) / u32::MAX as f32;
    let b = (hash2(xi + 1, zi) as f32) / u32::MAX as f32;
    let c = (hash2(xi, zi + 1) as f32) / u32::MAX as f32;
    let d = (hash2(xi + 1, zi + 1) as f32) / u32::MAX as f32;
    lerp(lerp(a, b, tx), lerp(c, d, tx), tz) * 2.0 - 1.0
}

pub fn noise3(x: f32, y: f32, z: f32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let zi = z.floor() as i32;
    let tx = fade(x - xi as f32);
    let ty = fade(y - yi as f32);
    let tz = fade(z - zi as f32);
    let v = |dx: i32, dy: i32, dz: i32| (hash3(xi + dx, yi + dy, zi + dz) as f32) / u32::MAX as f32 * 2.0 - 1.0;
    let z0 = lerp(lerp(v(0, 0, 0), v(1, 0, 0), tx), lerp(v(0, 1, 0), v(1, 1, 0), tx), ty);
    let z1 = lerp(lerp(v(0, 0, 1), v(1, 0, 1), tx), lerp(v(0, 1, 1), v(1, 1, 1), tx), ty);
    lerp(z0, z1, tz)
}

fn fbm2(x: f32, z: f32, oct: i32) -> f32 {
    let mut a = 0.0;
    let mut amp = 1.0;
    let mut f = 1.0;
    for _ in 0..oct {
        a += noise2(x * f, z * f) * amp;
        amp *= 0.5;
        f *= 2.0;
    }
    a
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Biome {
    Plains,
    Forest,
    BirchForest,
    DarkForest,
    Desert,
    Badlands,
    Snowy,
    Jungle,
    Swamp,
    Mountains,
    Ocean,
    Beach,
    Taiga,
    Savanna,
    MushroomIsland,
    NetherWastes,
    CrimsonForest,
    WarpedForest,
    SoulSandValley,
    BasaltDeltas,
    TheEnd,
    EndHighlands,
}

impl Biome {
    pub fn name(self) -> &'static str {
        match self {
            Biome::Plains => "Plains",
            Biome::Forest => "Forest",
            Biome::BirchForest => "Birch Forest",
            Biome::DarkForest => "Dark Forest",
            Biome::Desert => "Desert",
            Biome::Badlands => "Badlands",
            Biome::Snowy => "Snowy Tundra",
            Biome::Jungle => "Jungle",
            Biome::Swamp => "Swamp",
            Biome::Mountains => "Mountains",
            Biome::Ocean => "Ocean",
            Biome::Beach => "Beach",
            Biome::Taiga => "Taiga",
            Biome::Savanna => "Savanna",
            Biome::MushroomIsland => "Mushroom Fields",
            Biome::NetherWastes => "Nether Wastes",
            Biome::CrimsonForest => "Crimson Forest",
            Biome::WarpedForest => "Warped Forest",
            Biome::SoulSandValley => "Soul Sand Valley",
            Biome::BasaltDeltas => "Basalt Deltas",
            Biome::TheEnd => "The End",
            Biome::EndHighlands => "End Highlands",
        }
    }
}

pub fn biome_at(x: i32, z: i32) -> Biome {
    let temp = fbm2(x as f32 / 420.0 + 17.0, z as f32 / 420.0 - 8.0, 3);
    let humid = fbm2(x as f32 / 360.0 - 31.0, z as f32 / 360.0 + 12.0, 3);
    let cont = fbm2(x as f32 / 560.0, z as f32 / 560.0, 3);
    let weird = fbm2(x as f32 / 290.0 + 73.0, z as f32 / 290.0 - 51.0, 2);
    if cont < -0.34 && weird > 0.36 { return Biome::MushroomIsland; }
    if cont < -0.24 { return Biome::Ocean; }
    if cont < -0.15 { return Biome::Beach; }
    if cont > 0.32 { return Biome::Mountains; }
    if temp > 0.24 {
        if humid < -0.25 { return Biome::Badlands; }
        return if humid < 0.02 { Biome::Savanna } else { Biome::Desert };
    }
    if temp < -0.30 { return Biome::Snowy; }
    if humid > 0.26 { return if temp > 0.06 { Biome::Jungle } else { Biome::Swamp }; }
    if temp < -0.08 { return Biome::Taiga; }
    if humid > 0.14 {
        return if weird > 0.18 { Biome::DarkForest } else if weird < -0.15 { Biome::BirchForest } else { Biome::Forest };
    }
    if humid > 0.04 { return Biome::Forest; }
    Biome::Plains
}

pub fn biome_at_dim(dim: Dimension, x: i32, z: i32) -> Biome {
    match dim {
        Dimension::Overworld => biome_at(x, z),
        Dimension::Nether => {
            let n1 = fbm2(x as f32 / 180.0 + 11.0, z as f32 / 180.0 - 19.0, 3);
            let n2 = fbm2(x as f32 / 150.0 - 43.0, z as f32 / 150.0 + 27.0, 3);
            if n1 > 0.22 { Biome::CrimsonForest }
            else if n1 < -0.24 { Biome::WarpedForest }
            else if n2 > 0.22 { Biome::SoulSandValley }
            else if n2 < -0.24 { Biome::BasaltDeltas }
            else { Biome::NetherWastes }
        }
        Dimension::End => {
            let d2 = x * x + z * z;
            if d2 <= 95 * 95 { Biome::TheEnd } else { Biome::EndHighlands }
        }
    }
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn surface_height(x: i32, z: i32) -> i32 {
    let fx = x as f32;
    let fz = z as f32;
    let temp = fbm2(fx / 420.0 + 17.0, fz / 420.0 - 8.0, 3);
    let humid = fbm2(fx / 360.0 - 31.0, fz / 360.0 + 12.0, 3);
    let cont = fbm2(fx / 560.0, fz / 560.0, 3);
    let weird = fbm2(fx / 290.0 + 73.0, fz / 290.0 - 51.0, 2);

    let rolling = fbm2(fx / 96.0, fz / 96.0, 4) * 9.0;
    let detail = noise2(fx / 22.0, fz / 22.0) * 2.0;

    let ocean_t = smoothstep(-0.26, -0.20, cont);
    let land_t = smoothstep(-0.16, -0.05, cont);
    let mtn_t = smoothstep(0.22, 0.38, cont);

    let ocean_h = SEA as f32 - 8.0 - smoothstep(-0.24, -0.45, cont) * 7.0 + fbm2(fx / 60.0, fz / 60.0, 2) * 3.5;
    let beach_h = SEA as f32 + 1.0 + detail * 0.5;
    let inland_h = SEA as f32 + 5.5 + rolling + detail;

    let mut h = lerp(ocean_h, beach_h, ocean_t);
    h = lerp(h, inland_h, land_t);

    h += mtn_t * (22.0 + fbm2(fx / 48.0, fz / 48.0, 3) * 11.0);
    let bad_w = land_t * smoothstep(0.18, 0.28, temp) * (1.0 - smoothstep(-0.30, -0.20, humid));
    h += bad_w * (8.0 + (fbm2(fx / 36.0, fz / 36.0, 2) * 7.0).abs());
    let swamp_w = land_t * smoothstep(0.20, 0.30, humid) * (1.0 - smoothstep(0.02, 0.10, temp));
    h -= swamp_w * 3.5;
    let warm_w = land_t * smoothstep(0.18, 0.28, temp);
    h += warm_w * 2.0;
    let cold_w = land_t * (1.0 - smoothstep(-0.14, -0.02, temp));
    h += cold_w * 3.0;
    let mush_w = (1.0 - smoothstep(-0.38, -0.28, cont)) * smoothstep(0.30, 0.40, weird);
    h = lerp(h, SEA as f32 + 4.0 + rolling * 0.4, mush_w);

    h.max(4.0).round() as i32
}

fn cave_at(x: i32, y: i32, z: i32) -> bool {
    if y < 3 || y > 62 { return false; }
    let n = noise3(x as f32 / 17.0, y as f32 / 12.0, z as f32 / 17.0);
    let n2 = noise3(x as f32 / 31.0 + 40.0, y as f32 / 18.0, z as f32 / 31.0 - 20.0);
    (n + n2 * 0.5) > 0.46
}

fn ore_at(x: i32, y: i32, z: i32) -> Option<u16> {
    let (ox, oy, oz) = (x.div_euclid(2), y.div_euclid(2), z.div_euclid(2));
    let h = hash3(ox, oy, oz);
    if y < 16 && h % 190 == 0 { return Some(DIAMOND_ORE); }
    if y < 30 && h % 97 == 0 { return Some(GOLD_ORE); }
    if y < 46 && h % 47 == 0 { return Some(IRON_ORE); }
    if y < 60 && h % 31 == 0 { return Some(COAL_ORE); }
    let (rx, ry, rz) = (x.div_euclid(3), y.div_euclid(3), z.div_euclid(3));
    if y < 40 && hash3(rx, ry, rz) % 140 == 0 { return Some(REDSTONE_ORE); }
    if y < 32 && hash3(rx + 7, ry, rz - 5) % 135 == 0 { return Some(LAPIS_ORE); }
    if y < 44 && hash3(x.div_euclid(4), y.div_euclid(4), z.div_euclid(4)) % 110 == 0 { return Some(EMERALD_ORE); }
    None
}

fn tree_center(x: i32, z: i32) -> Option<(Biome, i32)> {
    let b = biome_at(x, z);
    let density = match b {
        Biome::Forest => 28,
        Biome::BirchForest => 26,
        Biome::DarkForest => 16,
        Biome::Jungle => 20,
        Biome::Taiga => 36,
        Biome::Plains => 110,
        Biome::Savanna => 85,
        Biome::Swamp => 55,
        Biome::MushroomIsland => 75,
        _ => return None,
    };
    if hash2(x, z) % density == 0 {
        let h = surface_height(x, z);
        if h >= SEA { return Some((b, h)); }
    }
    None
}

fn tree_block(x: i32, y: i32, z: i32) -> Option<u16> {
    for dx in -3..=3 {
        for dz in -3..=3 {
            let (b, h) = match tree_center(x - dx, z - dz) { Some(v) => v, None => continue };
            let (lx, lz) = (dx, dz);
            if b == Biome::MushroomIsland {
                let top = h + 5;
                if lx == 0 && lz == 0 && y >= h + 1 && y < top { return Some(QUARTZ); }
                if y == top && lx.abs() <= 2 && lz.abs() <= 2 && (lx.abs() + lz.abs() <= 3) {
                    return Some(RED_MUSHROOM);
                }
                if y == top - 1 && (lx.abs() == 2 || lz.abs() == 2) && lx.abs() <= 2 && lz.abs() <= 2 && !(lx.abs() == 2 && lz.abs() == 2) {
                    return Some(RED_MUSHROOM);
                }
                continue;
            }
            let big = b == Biome::Jungle || b == Biome::DarkForest;
            let height = if big { 7 } else if b == Biome::Taiga { 7 } else { 5 + (hash2(x - dx, z - dz) % 3) as i32 };
            let trunk = match b {
                Biome::Jungle => JUNGLE_LOG,
                Biome::BirchForest => BIRCH_LOG,
                Biome::DarkForest => DARK_LOG,
                Biome::Taiga | Biome::Snowy => SPRUCE_LOG,
                _ => LOG,
            };
            let leaf = match b {
                Biome::Jungle => JUNGLE_LEAVES,
                Biome::BirchForest => BIRCH_LEAVES,
                Biome::DarkForest => DARK_LEAVES,
                Biome::Taiga | Biome::Snowy => SPRUCE_LEAVES,
                _ => LEAVES,
            };
            let top = h + height;
            if lx == 0 && lz == 0 && y >= h + 1 && y <= top { return Some(trunk); }
            let dy = y - top;
            if dy < -3 || dy > 1 { continue; }
            let rad = if dy >= 0 { 1 } else if b == Biome::DarkForest { 3 } else { 2 };
            let ell = lx * lx + lz * lz;
            if ell <= rad * rad + 1 && ell > 0 && !(lx == 0 && lz == 0 && y <= top) {
                if (hash3(x, y, z) % 11) != 0 { return Some(leaf); }
            }
        }
    }
    None
}

fn structure_cell(x: i32, z: i32, size: i32) -> (i32, i32) {
    (x.div_euclid(size), z.div_euclid(size))
}

fn village_center(cx: i32, cz: i32) -> Option<(i32, i32)> {
    if hash2(cx + 91, cz - 37) % 3 != 0 { return None; }
    let size = 256;
    let vx = cx * size + 64 + (hash2(cx, cz) % 80) as i32;
    let vz = cz * size + 64 + (hash2(cx + 7, cz - 3) % 80) as i32;
    if matches!(biome_at(vx, vz), Biome::Ocean) { return None; }
    Some((vx, vz))
}

/// Find the closest village center `(x, surface_y, z)` near `(px, pz)`.
pub fn nearest_village(px: i32, pz: i32) -> Option<(i32, i32, i32)> {
    let (vcx, vcz) = structure_cell(px, pz, 256);
    let mut best: Option<(i32, i32, i32, i64)> = None;
    for r in 0..=4 {
        for cx in vcx - r..=vcx + r {
            for cz in vcz - r..=vcz + r {
                if let Some((vx, vz)) = village_center(cx, cz) {
                    let dx = (vx - px) as i64;
                    let dz = (vz - pz) as i64;
                    let d2 = dx * dx + dz * dz;
                    if best.map_or(true, |b| d2 < b.3) {
                        best = Some((vx, surface_height(vx, vz) + 1, vz, d2));
                    }
                }
            }
        }
        if let Some((vx, vy, vz, _)) = best {
            return Some((vx, vy, vz));
        }
    }
    None
}

/// Deterministic Stronghold End Portal room in Overworld at (100, 19, -140).
fn stronghold_block(x: i32, y: i32, z: i32) -> Option<u16> {
    let dx = x - 100;
    let dz = z + 140;
    if dx.abs() > 5 || dz.abs() > 5 || y < 18 || y > 24 { return None; }
    if y == 18 || y == 24 || dx.abs() == 5 || dz.abs() == 5 {
        return Some(if (x + y + z) % 5 == 0 { MOSSY } else { STONE_BRICKS });
    }
    // 3x3 End Portal ring at y == 19
    if y == 19 {
        let on_ring = (dx.abs() == 2 && dz.abs() <= 1) || (dz.abs() == 2 && dx.abs() <= 1);
        if on_ring {
            let has_eye = (dx == 2 && dz == 0) || (dx == -2 && dz == 1);
            return Some(if has_eye { END_PORTAL_FRAME_EYE } else { END_PORTAL_FRAME });
        }
        if dx.abs() <= 1 && dz.abs() <= 1 { return Some(LAVA); }
    }
    if y == 20 && ((dx.abs() == 4 && dz.abs() == 4) || (dx == 0 && dz == 4)) {
        return Some(TORCH);
    }
    Some(AIR)
}

fn structure_block(x: i32, y: i32, z: i32) -> Option<u16> {
    if let Some(sb) = stronghold_block(x, y, z) { return Some(sb); }
    let (vcx, vcz) = structure_cell(x, z, 256);
    for cx in vcx - 1..=vcx + 1 {
        for cz in vcz - 1..=vcz + 1 {
            if let Some((vx, vz)) = village_center(cx, cz) {
                let dx = x - vx;
                let dz = z - vz;
                if dx.abs() > 24 || dz.abs() > 24 { continue; }
                let vh = surface_height(vx, vz);
                let col_h = surface_height(x, z);
                let desert_style = matches!(biome_at(vx, vz), Biome::Desert | Biome::Badlands | Biome::Savanna);
                let wall_b = if desert_style { SANDSTONE } else { PLANKS };
                let trim_b = if desert_style { TERRACOTTA } else { LOG };
                let roof_b = if desert_style { SANDSTONE } else { STAIRS };
                let road_b = if desert_style { SANDSTONE } else { COBBLE };

                // 1. Town Well at (0, 0)
                if dx.abs() <= 2 && dz.abs() <= 2 {
                    if y < vh { return Some(COBBLE); }
                    if y == vh {
                        return Some(if dx.abs() <= 1 && dz.abs() <= 1 { WATER } else { MOSSY });
                    }
                    if y == vh + 1 && (dx.abs() == 2 || dz.abs() == 2) { return Some(COBBLE); }
                    if y > vh + 1 && y <= vh + 3 && dx.abs() == 2 && dz.abs() == 2 { return Some(FENCE); }
                    if y == vh + 4 { return Some(PLANKS); }
                    if y > vh && y <= vh + 7 { return Some(AIR); }
                }

                // 2. Village Roads + Torch Lamp Posts
                if (dx.abs() <= 1 && dz.abs() <= 22) || (dz.abs() <= 1 && dx.abs() <= 22) {
                    if y == col_h { return Some(road_b); }
                    if y > col_h && y <= col_h + 4 {
                        // Lamp posts along road corners
                        let is_lamp = (dx.abs() == 1 && dz.abs() == 6) || (dz.abs() == 1 && dx.abs() == 6)
                            || (dx.abs() == 1 && dz.abs() == 16) || (dz.abs() == 1 && dx.abs() == 16);
                        if is_lamp && (dx == 1 || dz == 1) {
                            if y <= col_h + 2 { return Some(FENCE); }
                            if y == col_h + 3 { return Some(LANTERN); }
                        }
                        return Some(AIR);
                    }
                }

                // 3. Blacksmith Forge at (+13, -12)
                let bx = x - (vx + 13);
                let bz = z - (vz - 12);
                if bx >= -4 && bx <= 4 && bz >= -3 && bz <= 3 {
                    let wall = bx.abs() == 4 || bz.abs() == 3;
                    if y < vh && y >= col_h.min(vh - 3) { return Some(COBBLE); }
                    if y == vh { return Some(COBBLE); }
                    if y > vh && y <= vh + 4 {
                        if bz == 3 && bx == 0 && y <= vh + 2 { return Some(AIR); }
                        if bx <= -1 && bz <= 0 && y == vh + 1 {
                            if bx == -3 && bz == -2 { return Some(LAVA); }
                            if bx == -2 && bz == -2 { return Some(FURNACE); }
                            if bx == -1 && bz == -2 { return Some(ANVIL); }
                            if bx == -3 && bz == -1 { return Some(CHEST); }
                        }
                        if wall {
                            if (bx.abs() == 4 && bz.abs() == 3) || y == vh + 4 { return Some(COBBLE); }
                            if bx == 4 && bz == 0 && y == vh + 2 { return Some(GLASS); }
                            return Some(COBBLE);
                        }
                        if y == vh + 3 && bx == 3 && bz == 2 { return Some(TORCH); }
                        return Some(AIR);
                    }
                    if y == vh + 5 && bx.abs() <= 4 && bz.abs() <= 3 { return Some(SLAB); }
                    if y > vh + 5 && y <= vh + 8 { return Some(AIR); }
                }

                // 4. Three Villager Houses
                let houses = [(-13, -11), (-12, 12), (12, 12)];
                for &(ox, oz) in &houses {
                    let hx = x - (vx + ox);
                    let hz = z - (vz + oz);
                    if hx >= -3 && hx <= 3 && hz >= -3 && hz <= 3 {
                        let wall = hx.abs() == 3 || hz.abs() == 3;
                        if y < vh && y >= col_h.min(vh - 3) { return Some(COBBLE); }
                        if y == vh { return Some(wall_b); }
                        if y > vh && y <= vh + 4 {
                            if hx == 0 && hz == 3 {
                                if y == vh + 1 { return Some(DOOR); }
                                if y == vh + 2 { return Some(AIR); }
                            }
                            if wall {
                                if (hx.abs() == 3 && hz.abs() == 3) || (y == vh + 4) { return Some(trim_b); }
                                if ((hx.abs() == 3 && hz == 0) || (hz.abs() == 3 && hx == 0)) && y == vh + 2 { return Some(GLASS); }
                                return Some(wall_b);
                            }
                            if y == vh + 1 {
                                if hx == -2 && hz == -2 { return Some(BED); }
                                if hx == 2 && hz == -2 { return Some(CHEST); }
                                if hx == -2 && hz == 2 { return Some(CRAFTING); }
                            }
                            if y == vh + 3 && hx == 0 && hz == -2 { return Some(TORCH); }
                            return Some(AIR);
                        }
                        if y == vh + 5 && hx.abs() <= 3 && hz.abs() <= 3 { return Some(roof_b); }
                        if y == vh + 6 && hx.abs() <= 2 && hz.abs() <= 2 { return Some(roof_b); }
                        if y == vh + 7 && hx.abs() <= 1 && hz.abs() <= 1 { return Some(roof_b); }
                        if y > vh + 7 && y <= vh + 9 { return Some(AIR); }
                    }
                }

                // 5. Irrigated Wheat Farm at (-13, 2) and (+13, 3)
                let farms = [(-14, 2), (13, 3)];
                for &(fx, fz) in &farms {
                    let lx = x - (vx + fx);
                    let lz = z - (vz + fz);
                    if lx.abs() <= 4 && lz.abs() <= 3 {
                        let fh = surface_height(vx + fx, vz + fz);
                        if y < fh && y >= col_h.min(fh - 2) { return Some(DIRT); }
                        if y == fh {
                            if lx.abs() == 4 || lz.abs() == 3 { return Some(LOG); }
                            if lz == 0 { return Some(WATER); }
                            return Some(FARMLAND);
                        }
                        if y == fh + 1 {
                            if lx.abs() < 4 && lz.abs() < 3 && lz != 0 { return Some(WHEAT); }
                            return Some(AIR);
                        }
                        if y > fh + 1 && y <= fh + 5 { return Some(AIR); }
                    }
                }
            }
        }
    }
    let (scx, scz) = structure_cell(x, z, 192);
    for cx in scx - 1..=scx + 1 {
        for cz in scz - 1..=scz + 1 {
            let seed = hash2(cx + 411, cz - 177);
            let sx = cx * 192 + 96;
            let sz = cz * 192 + 96;
            let dx = x - sx;
            let dz = z - sz;
            if dx.abs() > 12 || dz.abs() > 12 { continue; }
            let b = biome_at(sx, sz);
            if seed % 5 == 1 && (b == Biome::Desert || b == Biome::Savanna) {
                let h = surface_height(x, z);
                let layer = (12 - dx.abs()).min(12 - dz.abs());
                if y >= h && y < h + 5 {
                    if y == h && dx.abs() <= 3 && dz.abs() <= 3 { return Some(AIR); }
                    if y == h + 1 && dx.abs() <= 2 && dz.abs() <= 2 { return Some(CHEST); }
                    if dx.abs() <= layer && dz.abs() <= layer { return Some(SANDSTONE); }
                }
            }
            if seed % 7 == 3 && b == Biome::Ocean {
                let h = surface_height(x, z) + 1;
                if dz.abs() <= 4 && dx.abs() <= 9 {
                    let hull = dz.abs() == 4 || dx.abs() == 9;
                    if y == h && hull { return Some(PLANKS); }
                    if y == h && !hull && dx.abs() < 8 { return Some(PLANKS); }
                    if y > h && y <= h + 3 && dx == 0 && dz == 0 { return Some(LOG); }
                    if y > h && y <= h + 1 && hull { return Some(PLANKS); }
                    if y == h + 1 && dx == -4 { return Some(CHEST); }
                }
            }
            // Ruined Nether Portal
            if seed % 9 == 4 && !matches!(b, Biome::Ocean) {
                let h = surface_height(sx, sz);
                if dz == 0 && dx.abs() <= 2 && y >= h && y <= h + 4 {
                    if y == h { return Some(if (dx + dz) % 2 == 0 { NETHERRACK } else { MAGMA }); }
                    if dx.abs() == 2 || y == h + 4 {
                        return Some(if hash3(x, y, z) % 4 == 0 { CRYING_OBSIDIAN } else { OBSIDIAN });
                    }
                    if y == h + 1 && dx == 0 { return Some(CHEST); }
                }
            }
        }
    }
    None
}

pub fn block_name(id: u16) -> &'static str {
    match id {
        AIR => "Air", STONE => "Stone", GRASS => "Grass Block", DIRT => "Dirt", COBBLE => "Cobblestone",
        LOG => "Oak Log", LEAVES => "Oak Leaves", SAND => "Sand", SANDSTONE => "Sandstone", WATER => "Water",
        SNOW => "Snow Block", ICE => "Ice", SPRUCE_LOG => "Spruce Log", SPRUCE_LEAVES => "Spruce Leaves",
        JUNGLE_LOG => "Jungle Log", JUNGLE_LEAVES => "Jungle Leaves", PLANKS => "Oak Planks", GLASS => "Glass",
        TORCH => "Torch", BEDROCK => "Bedrock", COAL_ORE => "Coal Ore", IRON_ORE => "Iron Ore",
        GOLD_ORE => "Gold Ore", DIAMOND_ORE => "Diamond Ore", REDSTONE_ORE => "Redstone Ore",
        GRAVEL => "Gravel", CLAY => "Clay", CACTUS => "Cactus", TALL_GRASS => "Tall Grass",
        FLOWER_RED => "Poppy", FLOWER_YELLOW => "Dandelion", BRICK => "Bricks", STONE_BRICKS => "Stone Bricks",
        WOOL_WHITE => "White Wool", WOOL_RED => "Red Wool", WOOL_BLUE => "Blue Wool", WOOL_GREEN => "Green Wool",
        WOOL_YELLOW => "Yellow Wool", BLACKSTONE => "Blackstone", MAGMA => "Magma Block", LAVA => "Lava",
        FARMLAND => "Farmland", WHEAT => "Wheat", CHEST => "Chest", CRAFTING => "Crafting Table",
        FURNACE => "Furnace", BIRCH_LOG => "Birch Log", BIRCH_LEAVES => "Birch Leaves",
        BROWN_MUSHROOM => "Brown Mushroom", RED_MUSHROOM => "Red Mushroom", PUMPKIN => "Pumpkin",
        HAY => "Hay Block", TNT => "TNT", OBSIDIAN => "Obsidian", GLOWSTONE => "Glowstone",
        SEA_LANTERN => "Sea Lantern", DARK_LOG => "Dark Oak Log", DARK_LEAVES => "Dark Oak Leaves",
        MOSSY => "Mossy Cobblestone", COBWEB => "Cobweb", VINE => "Vines", DOOR => "Oak Door",
        IRON_BLOCK => "Block of Iron", GOLD_BLOCK => "Block of Gold", DIAMOND_BLOCK => "Block of Diamond",
        EMERALD_ORE => "Emerald Ore", EMERALD_BLOCK => "Block of Emerald", SNOW_LAYER => "Snow Layer",
        PODZOL => "Podzol", MYCELIUM => "Mycelium", RED_SAND => "Red Sand", TERRACOTTA => "Terracotta",
        CONCRETE_WHITE => "White Concrete", CONCRETE_BLUE => "Blue Concrete", CONCRETE_RED => "Red Concrete",
        CONCRETE_GREEN => "Green Concrete", CONCRETE_YELLOW => "Yellow Concrete", CONCRETE_BLACK => "Black Concrete",
        LANTERN => "Lantern", CAMPFIRE => "Campfire", BOOKSHELF => "Bookshelf", BED => "Bed", CAKE => "Cake",
        LADDER => "Ladder", RAIL => "Rail", ANVIL => "Anvil", ENCHANTING => "Enchanting Table",
        END_STONE => "End Stone", NETHERRACK => "Netherrack", SOUL_SAND => "Soul Sand", QUARTZ => "Quartz Block",
        SLAB => "Stone Slab", STAIRS => "Cobble Stairs", FENCE => "Oak Fence", SIGN => "Oak Sign",
        REDSTONE_DUST => "Redstone Dust", REDSTONE_TORCH => "Redstone Torch", LEVER => "Lever",
        LAMP => "Redstone Lamp", PRESSURE_PLATE => "Pressure Plate",
        NETHER_PORTAL => "Nether Portal", END_PORTAL_FRAME => "End Portal Frame",
        END_PORTAL => "End Portal", CRYING_OBSIDIAN => "Crying Obsidian",
        CRIMSON_NYLIUM => "Crimson Nylium", WARPED_NYLIUM => "Warped Nylium",
        CRIMSON_STEM => "Crimson Stem", CRIMSON_WART => "Crimson Wart Block",
        WARPED_STEM => "Warped Stem", WARPED_WART => "Warped Wart Block",
        BASALT => "Basalt", SHROOMLIGHT => "Shroomlight",
        NETHER_QUARTZ_ORE => "Nether Quartz Ore", ANCIENT_DEBRIS => "Ancient Debris",
        CHORUS_PLANT => "Chorus Plant", CHORUS_FLOWER => "Chorus Flower",
        PURPUR_BLOCK => "Purpur Block", NETHER_BRICKS => "Nether Bricks",
        END_PORTAL_FRAME_EYE => "End Portal Frame (Eye)",
        DOOR_OPEN => "Oak Door",
        LAPIS_ORE => "Lapis Lazuli Ore", LAPIS_BLOCK => "Lapis Lazuli Block",
        BREWING_STAND => "Brewing Stand", BEACON => "Beacon",
        HOPPER => "Hopper", PISTON => "Piston", STICKY_PISTON => "Sticky Piston",
        REPEATER => "Redstone Repeater", COMPARATOR => "Redstone Comparator",
        SPAWNER => "Monster Spawner", SAPLING => "Oak Sapling", NETHER_WART => "Nether Wart",
        BUTTON => "Stone Button", IRON_DOOR | IRON_DOOR_OPEN => "Iron Door",
        _ => "Unknown",
    }
}

pub fn is_solid(id: u16) -> bool {
    !matches!(
        id,
        AIR | WATER | LAVA | TALL_GRASS | FLOWER_RED | FLOWER_YELLOW | COBWEB | VINE | WHEAT
            | TORCH | SNOW_LAYER | RAIL | REDSTONE_DUST | LEVER | PRESSURE_PLATE | REDSTONE_TORCH
            | NETHER_PORTAL | END_PORTAL | DOOR_OPEN | IRON_DOOR_OPEN | SAPLING | NETHER_WART
            | BUTTON | REPEATER | COMPARATOR
    )
}

pub fn is_targetable(id: u16) -> bool {
    !matches!(id, AIR | WATER | LAVA | NETHER_PORTAL | END_PORTAL)
}

pub fn is_replaceable(id: u16) -> bool {
    matches!(id, AIR | WATER | LAVA | TALL_GRASS | SNOW_LAYER)
}

pub fn is_opaque(id: u16) -> bool {
    is_solid(id)
        && !matches!(
            id,
            GLASS | LEAVES | SPRUCE_LEAVES | JUNGLE_LEAVES | BIRCH_LEAVES | DARK_LEAVES | ICE
                | BEACON | BREWING_STAND | SPAWNER | FENCE | ANVIL | ENCHANTING | HOPPER
                | SLAB | STAIRS | DOOR | IRON_DOOR | LADDER | CAMPFIRE | LANTERN | CAKE
        )
}

pub fn emits_light(id: u16) -> u8 {
    match id {
        TORCH | CAMPFIRE | LANTERN | SHROOMLIGHT | NETHER_PORTAL | END_PORTAL => 14,
        GLOWSTONE | SEA_LANTERN | BEACON => 15,
        LAVA | MAGMA | CRYING_OBSIDIAN | ENCHANTING => 12,
        REDSTONE_TORCH => 7,
        _ => 0,
    }
}

fn chunk_may_have_structure(cx: i32, cz: i32) -> bool {
    let x0 = cx * CHUNK;
    let z0 = cz * CHUNK;
    if 106 >= x0 && 94 < x0 + CHUNK && -134 >= z0 && -146 < z0 + CHUNK {
        return true;
    }
    let (vcx, vcz) = structure_cell(x0 + 8, z0 + 8, 256);
    for ix in vcx - 1..=vcx + 1 {
        for iz in vcz - 1..=vcz + 1 {
            if let Some((vx, vz)) = village_center(ix, iz) {
                if vx + 20 >= x0 && vx - 20 < x0 + CHUNK && vz + 20 >= z0 && vz - 20 < z0 + CHUNK {
                    return true;
                }
            }
        }
    }
    let (scx, scz) = structure_cell(x0 + 8, z0 + 8, 192);
    for ix in scx - 1..=scx + 1 {
        for iz in scz - 1..=scz + 1 {
            let sx = ix * 192 + 96;
            let sz = iz * 192 + 96;
            if sx + 12 >= x0 && sx - 12 < x0 + CHUNK && sz + 12 >= z0 && sz - 12 < z0 + CHUNK {
                return true;
            }
        }
    }
    false
}

fn generate_overworld_chunk(cx: i32, cz: i32) -> Chunk {
    let mut c = Chunk::new();
    let has_struct = chunk_may_have_structure(cx, cz);
    for lz in 0..CHUNK {
        for lx in 0..CHUNK {
            let x = cx * CHUNK + lx;
            let z = cz * CHUNK + lz;
            let b = biome_at(x, z);
            let h = surface_height(x, z);
            for y in 0..WORLD_H as i32 {
                let mut id = AIR;
                if y == 0 { id = BEDROCK; }
                else if y <= h {
                    if !matches!(b, Biome::Ocean | Biome::Beach) && cave_at(x, y, z) && y < h - 2 && y > 3 {
                        id = if y <= 10 { LAVA } else { AIR };
                    } else {
                        id = STONE;
                        if let Some(o) = ore_at(x, y, z) { id = o; }
                        if y > h - 4 {
                            id = match b {
                                Biome::Desert => if y == h { SAND } else { SANDSTONE },
                                Biome::Badlands => if y == h && hash2(x, z) % 4 == 0 { RED_SAND } else { TERRACOTTA },
                                Biome::Savanna => if y == h { GRASS } else { DIRT },
                                Biome::Snowy => if y == h { SNOW } else { DIRT },
                                Biome::Taiga => if y == h && hash2(x, z) % 3 == 0 { PODZOL } else { DIRT },
                                Biome::Swamp => if y == h && hash2(x, z) % 4 == 0 { MYCELIUM } else { DIRT },
                                Biome::MushroomIsland => if y == h { MYCELIUM } else { DIRT },
                                Biome::Ocean | Biome::Beach => if y > h - 2 { SAND } else { STONE },
                                _ => if y == h { GRASS } else { DIRT },
                            };
                        }
                    }
                } else if y <= SEA && (h < SEA || matches!(b, Biome::Ocean | Biome::Beach | Biome::Swamp)) {
                    id = WATER;
                }
                if has_struct && y >= 16 {
                    if let Some(s) = structure_block(x, y, z) {
                        id = s;
                    }
                }
                if id == AIR && y > h && y <= h + 10 {
                    if let Some(t) = tree_block(x, y, z) { id = t; }
                }
                if id == AIR && y == h + 1 {
                    let hh = hash2(x + 5, z - 7);
                    if matches!(b, Biome::Plains | Biome::Forest | Biome::BirchForest | Biome::Savanna) {
                        if hh % 17 == 0 { id = TALL_GRASS; }
                        else if hh % 43 == 0 { id = FLOWER_RED; }
                        else if hh % 47 == 0 { id = FLOWER_YELLOW; }
                    } else if b == Biome::Desert && hh % 61 == 0 { id = CACTUS; }
                    else if (b == Biome::Swamp || b == Biome::MushroomIsland) && hh % 41 == 0 {
                        id = if hh % 2 == 0 { RED_MUSHROOM } else { BROWN_MUSHROOM };
                    }
                }
                if id == AIR && y == h + 2 && b == Biome::Desert && hash2(x - 3, z + 8) % 61 == 0 { id = CACTUS; }
                c.set(lx, y as usize, lz, id);
            }
        }
    }
    let (mcx, mcz) = structure_cell(cx, cz, 3);
    for mx in mcx - 1..=mcx + 1 {
        for mz in mcz - 1..=mcz + 1 {
            if hash2(mx + 300, mz - 44) % 4 != 0 { continue; }
            let along_x = hash2(mx, mz) % 2 == 0;
            let origin = if along_x { mz * 48 + 24 } else { mx * 48 + 24 };
            for t in 0..48 {
                let (wx, wz) = if along_x { (mx * 48 + t, origin) } else { (origin, mz * 48 + t) };
                if wx.div_euclid(CHUNK) != cx || wz.div_euclid(CHUNK) != cz { continue; }
                if matches!(biome_at(wx, wz), Biome::Ocean | Biome::Beach) { continue; }
                let lx = wx - cx * CHUNK;
                let lz = wz - cz * CHUNK;
                let y = 24 + (hash2(mx, mz) % 6) as i32;
                for dy in 0..4 {
                    for off in -1..=1 {
                        let (px, pz) = if along_x { (wx, wz + off) } else { (wx + off, wz) };
                        if px.div_euclid(CHUNK) != cx || pz.div_euclid(CHUNK) != cz { continue; }
                        c.set(px - cx * CHUNK, (y + dy) as usize, pz - cz * CHUNK, AIR);
                    }
                }
                c.set(lx, y as usize, lz, RAIL);
                if t % 10 == 0 {
                    for off in -2..=2 {
                        for dy in 0..4 {
                            let (px, pz) = if along_x { (wx, wz + off) } else { (wx + off, wz) };
                            if px.div_euclid(CHUNK) != cx || pz.div_euclid(CHUNK) != cz { continue; }
                            let id = if off.abs() == 2 && (dy == 0 || dy == 3) { PLANKS } else if off.abs() == 2 { FENCE } else { AIR };
                            c.set(px - cx * CHUNK, (y + dy) as usize, pz - cz * CHUNK, id);
                        }
                    }
                }
            }
        }
    }
    c
}

fn nether_fungus_block(x: i32, y: i32, z: i32, floor_y: i32, b: Biome) -> Option<u16> {
    if b != Biome::CrimsonForest && b != Biome::WarpedForest { return None; }
    for dx in -2..=2 {
        for dz in -2..=2 {
            let tx = x - dx;
            let tz = z - dz;
            if biome_at_dim(Dimension::Nether, tx, tz) != b { continue; }
            if hash2(tx + 19, tz - 23) % 34 != 0 { continue; }
            let base_y = 26 + ((fbm2(tx as f32 / 48.0, tz as f32 / 48.0, 2) + 1.0) * 9.0) as i32;
            if (base_y - floor_y).abs() > 3 { continue; }
            let h = 5 + (hash2(tx, tz) % 3) as i32;
            let top = base_y + h;
            let (stem, wart) = if b == Biome::CrimsonForest { (CRIMSON_STEM, CRIMSON_WART) } else { (WARPED_STEM, WARPED_WART) };
            if dx == 0 && dz == 0 && y > base_y && y <= top { return Some(stem); }
            if y >= top - 2 && y <= top + 1 && dx.abs() <= 2 && dz.abs() <= 2 {
                if y == top && (dx.abs() == 1 && dz == 0) && hash3(tx, y, tz) % 3 == 0 {
                    return Some(SHROOMLIGHT);
                }
                if !(dx.abs() == 2 && dz.abs() == 2 && y == top + 1) && !(dx == 0 && dz == 0 && y <= top) {
                    return Some(wart);
                }
            }
        }
    }
    None
}

fn generate_nether_chunk(cx: i32, cz: i32) -> Chunk {
    let mut c = Chunk::new();
    for lz in 0..CHUNK {
        for lx in 0..CHUNK {
            let x = cx * CHUNK + lx;
            let z = cz * CHUNK + lz;
            let b = biome_at_dim(Dimension::Nether, x, z);
            let floor_h = (25.0 + (fbm2(x as f32 / 48.0, z as f32 / 48.0, 2) + 1.0) * 9.0) as i32;
            let ceil_h = (68.0 + fbm2(x as f32 / 55.0 + 30.0, z as f32 / 55.0 - 14.0, 2) * 10.0) as i32;
            let basalt_pillar = b == Biome::SoulSandValley && hash2(x / 2, z / 2) % 37 == 0;
            // Nether Fortress bridge along z == -48..=-36
            let on_fortress_bridge = (z + 40).abs() <= 2 && x.rem_euclid(128) < 48;

            for y in 0..WORLD_H as i32 {
                let mut id = AIR;
                if y == 0 || y >= 88 {
                    id = BEDROCK;
                } else if y <= floor_h || y >= ceil_h {
                    id = match b {
                        Biome::BasaltDeltas => if hash3(x, y, z) % 4 == 0 { MAGMA } else if hash3(x, y, z) % 2 == 0 { BASALT } else { BLACKSTONE },
                        Biome::SoulSandValley => if y >= floor_h - 3 && y <= floor_h { SOUL_SAND } else { NETHERRACK },
                        Biome::CrimsonForest => if y == floor_h && y > 24 { CRIMSON_NYLIUM } else { NETHERRACK },
                        Biome::WarpedForest => if y == floor_h && y > 24 { WARPED_NYLIUM } else { NETHERRACK },
                        Biome::NetherWastes => NETHERRACK,
                        _ => NETHERRACK,
                    };
                    if id == NETHERRACK {
                        let h = hash3(x / 2, y / 2, z / 2);
                        if y < 22 && h % 210 == 0 { id = ANCIENT_DEBRIS; }
                        else if h % 38 == 0 { id = NETHER_QUARTZ_ORE; }
                        else if y >= ceil_h && y <= ceil_h + 2 && hash2(x / 2, z / 2) % 29 == 0 { id = GLOWSTONE; }
                    }
                } else if y <= 24 {
                    id = LAVA;
                } else if basalt_pillar && y <= ceil_h {
                    id = BASALT;
                }

                if id == AIR && y > floor_h && y <= floor_h + 9 {
                    if let Some(fb) = nether_fungus_block(x, y, z, floor_h, b) {
                        id = fb;
                    }
                }

                if on_fortress_bridge {
                    if y == 36 { id = NETHER_BRICKS; }
                    else if y == 37 && (z + 40).abs() == 2 && x % 3 != 0 { id = NETHER_BRICKS; }
                    else if y < 36 && x.rem_euclid(16) == 0 && (z + 40).abs() == 2 && y > 18 { id = NETHER_BRICKS; }
                }

                c.set(lx, y as usize, lz, id);
            }
        }
    }
    c
}

fn generate_end_chunk(cx: i32, cz: i32) -> Chunk {
    let mut c = Chunk::new();
    let pillars: [(i32, i32, i32); 8] = [
        (34, 0, 74), (24, 24, 69), (0, 34, 77), (-24, 24, 67),
        (-34, 0, 72), (-24, -24, 70), (0, -34, 75), (24, -24, 68),
    ];

    for lz in 0..CHUNK {
        for lx in 0..CHUNK {
            let x = cx * CHUNK + lx;
            let z = cz * CHUNK + lz;
            let d2 = x * x + z * z;
            let dist = (d2 as f32).sqrt();
            let in_center = dist <= 56.0 + noise2(x as f32 / 14.0, z as f32 / 14.0) * 5.0;
            let in_outer = dist > 128.0 && fbm2(x as f32 / 42.0, z as f32 / 42.0, 2) > 0.24;

            for y in 0..WORLD_H as i32 {
                let mut id = AIR;
                if in_center {
                    let top = (60.0 + noise2(x as f32 / 24.0, z as f32 / 24.0) * 2.5) as i32;
                    let bot = (42.0 + (dist / 56.0) * 14.0 + noise2(x as f32 / 16.0, z as f32 / 16.0) * 3.0) as i32;
                    if y >= bot && y <= top { id = END_STONE; }
                } else if in_outer {
                    let top = 60 + (noise2(x as f32 / 18.0, z as f32 / 18.0) * 3.0) as i32;
                    let bot = 50 + (noise2(x as f32 / 14.0, z as f32 / 14.0) * 3.0) as i32;
                    if y >= bot && y <= top { id = END_STONE; }
                    // Chorus plants on outer islands
                    if hash2(x, z) % 73 == 0 && y > top && y <= top + 5 {
                        id = if y == top + 5 { CHORUS_FLOWER } else { CHORUS_PLANT };
                    }
                }

                // 8 Obsidian Pillars + Glowing End Crystal beacons
                for &(px, pz, ph) in &pillars {
                    let pdx = (x - px).abs();
                    let pdz = (z - pz).abs();
                    if pdx <= 2 && pdz <= 2 && (pdx + pdz) <= 3 && y >= 50 && y <= ph {
                        id = OBSIDIAN;
                    }
                    if x == px && z == pz && (y == ph + 1 || y == ph + 2) {
                        id = if y == ph + 1 { BEDROCK } else { GLOWSTONE };
                    }
                }

                // Exit portal fountain at (0, 61, 0)
                if d2 <= 9 && y >= 58 && y <= 65 {
                    if y == 60 && d2 <= 9 { id = BEDROCK; }
                    else if y == 61 && d2 >= 5 { id = BEDROCK; }
                    else if y == 61 && d2 < 5 { id = END_PORTAL; }
                    else if x == 0 && z == 0 && y >= 61 && y <= 64 { id = BEDROCK; }
                    else if (x.abs() == 1 && z == 0 || z.abs() == 1 && x == 0) && y == 63 { id = TORCH; }
                }

                c.set(lx, y as usize, lz, id);
            }
        }
    }
    c
}

fn generate_chunk(dim: Dimension, cx: i32, cz: i32) -> Chunk {
    match dim {
        Dimension::Overworld => generate_overworld_chunk(cx, cz),
        Dimension::Nether => generate_nether_chunk(cx, cz),
        Dimension::End => generate_end_chunk(cx, cz),
    }
}

struct Chunk { blocks: Vec<u16> }
impl Chunk {
    fn new() -> Self { Self { blocks: vec![0; CHUNK as usize * WORLD_H * CHUNK as usize] } }
    #[inline]
    fn idx(lx: i32, y: usize, lz: i32) -> usize { (y * CHUNK as usize + lx as usize) * CHUNK as usize + lz as usize }
    #[inline]
    fn get(&self, lx: i32, y: usize, lz: i32) -> u16 { self.blocks[Self::idx(lx, y, lz)] }
    #[inline]
    fn set(&mut self, lx: i32, y: usize, lz: i32, id: u16) { let i = Self::idx(lx, y, lz); self.blocks[i] = id; }
}

const CACHE_R: i32 = 5;
const CACHE_SIDE: usize = (CACHE_R as usize) * 2 + 1;
const CACHE_BLOCKS: i32 = CACHE_SIDE as i32 * CHUNK;
const CHUNK_LEN: usize = CHUNK as usize * WORLD_H * CHUNK as usize;

pub struct RenderCache {
    valid: bool,
    light_dirty: bool,
    dim: Dimension,
    cx: i32,
    cz: i32,
    ox: i32,
    oz: i32,
    data: Vec<u16>,
    light: Vec<u8>,
    pub lit: Vec<(i32, i32, i32)>,
}

impl RenderCache {
    pub fn new() -> Self {
        Self {
            valid: false,
            light_dirty: true,
            dim: Dimension::Overworld,
            cx: 0,
            cz: 0,
            ox: 0,
            oz: 0,
            data: vec![0; CACHE_SIDE * CACHE_SIDE * CHUNK_LEN],
            light: vec![0; CACHE_SIDE * CACHE_SIDE * CHUNK_LEN],
            lit: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn get_light(&self, x: i32, y: i32, z: i32) -> u8 {
        if y < 0 || y >= WORLD_H as i32 { return 0; }
        let rx = x - self.ox;
        let rz = z - self.oz;
        if (rx as u32) >= CACHE_BLOCKS as u32 || (rz as u32) >= CACHE_BLOCKS as u32 { return 0; }
        let cxi = (rx >> 4) as usize;
        let czi = (rz >> 4) as usize;
        let lx = (rx & 15) as usize;
        let lz = (rz & 15) as usize;
        let ci = (cxi * CACHE_SIDE + czi) * CHUNK_LEN;
        self.light[ci + (y as usize * CHUNK as usize + lx) * CHUNK as usize + lz] & 0x0F
    }

    #[inline(always)]
    pub fn get_sky_light(&self, x: i32, y: i32, z: i32) -> u8 {
        if y >= WORLD_H as i32 { return 15; }
        if y < 0 { return 0; }
        let rx = x - self.ox;
        let rz = z - self.oz;
        if (rx as u32) >= CACHE_BLOCKS as u32 || (rz as u32) >= CACHE_BLOCKS as u32 { return 15; }
        let cxi = (rx >> 4) as usize;
        let czi = (rz >> 4) as usize;
        let lx = (rx & 15) as usize;
        let lz = (rz & 15) as usize;
        let ci = (cxi * CACHE_SIDE + czi) * CHUNK_LEN;
        (self.light[ci + (y as usize * CHUNK as usize + lx) * CHUNK as usize + lz] >> 4) & 0x0F
    }

    #[inline(always)]
    fn idx_of(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        if y < 0 || y >= WORLD_H as i32 { return None; }
        let rx = x - self.ox;
        let rz = z - self.oz;
        if (rx as u32) >= CACHE_BLOCKS as u32 || (rz as u32) >= CACHE_BLOCKS as u32 { return None; }
        let cxi = (rx >> 4) as usize;
        let czi = (rz >> 4) as usize;
        let lx = (rx & 15) as usize;
        let lz = (rz & 15) as usize;
        Some((cxi * CACHE_SIDE + czi) * CHUNK_LEN + (y as usize * CHUNK as usize + lx) * CHUNK as usize + lz)
    }

    pub fn rebuild_light(&mut self) {
        self.light.fill(0);
        let mut q: std::collections::VecDeque<(i32, i32, i32, u8)> = std::collections::VecDeque::new();
        let mut sq: std::collections::VecDeque<(i32, i32, i32, u8)> = std::collections::VecDeque::new();
        let base_sky = match self.dim {
            Dimension::Overworld => 15u8,
            Dimension::End => 11u8,
            Dimension::Nether => 4u8,
        };

        for cxi in 0..CACHE_SIDE {
            for czi in 0..CACHE_SIDE {
                let ci = (cxi * CACHE_SIDE + czi) * CHUNK_LEN;
                let bx = self.ox + cxi as i32 * CHUNK;
                let bz = self.oz + czi as i32 * CHUNK;
                if self.dim != Dimension::Nether {
                    for lx in 0..CHUNK as usize {
                        for lz in 0..CHUNK as usize {
                            let mut sky = base_sky;
                            for y in (0..WORLD_H).rev() {
                                let idx = ci + (y * CHUNK as usize + lx) * CHUNK as usize + lz;
                                let id = self.data[idx];
                                if is_opaque(id) {
                                    break;
                                }
                                if matches!(
                                    id,
                                    WATER | ICE | LEAVES | BIRCH_LEAVES | SPRUCE_LEAVES | JUNGLE_LEAVES | DARK_LEAVES
                                ) {
                                    sky = sky.saturating_sub(2);
                                }
                                if sky > 0 {
                                    self.light[idx] = (self.light[idx] & 0x0F) | (sky << 4);
                                    if sky > 1 {
                                        sq.push_back((bx + lx as i32, y as i32, bz + lz as i32, sky));
                                    }
                                }
                            }
                        }
                    }
                }
                for y in 0..WORLD_H {
                    let yoff = ci + y * CHUNK as usize * CHUNK as usize;
                    for lx in 0..CHUNK as usize {
                        let xoff = yoff + lx * CHUNK as usize;
                        for lz in 0..CHUNK as usize {
                            let idx = xoff + lz;
                            let id = self.data[idx];
                            if self.dim == Dimension::Nether && !is_opaque(id) {
                                self.light[idx] = (self.light[idx] & 0x0F) | (base_sky << 4);
                            }
                            let e = emits_light(id);
                            if e > 0 {
                                self.light[idx] = (self.light[idx] & 0xF0) | (e & 0x0F);
                                q.push_back((bx + lx as i32, y as i32, bz + lz as i32, e));
                            }
                        }
                    }
                }
            }
        }
        for &(lx, ly, lz) in &self.lit {
            if let Some(i) = self.idx_of(lx, ly, lz) {
                if self.data[i] == LAMP && (self.light[i] & 0x0F) < 15 {
                    self.light[i] = (self.light[i] & 0xF0) | 15;
                    q.push_back((lx, ly, lz, 15));
                }
            }
        }
        while let Some((x, y, z, sl)) = sq.pop_front() {
            if sl <= 1 { continue; }
            let nsl = sl - 1;
            for (dx, dy, dz) in [(1, 0, 0), (-1, 0, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)] {
                let nx = x + dx; let ny = y + dy; let nz = z + dz;
                if let Some(ni) = self.idx_of(nx, ny, nz) {
                    if !is_opaque(self.data[ni]) && ((self.light[ni] >> 4) & 0x0F) < nsl {
                        self.light[ni] = (self.light[ni] & 0x0F) | (nsl << 4);
                        sq.push_back((nx, ny, nz, nsl));
                    }
                }
            }
        }
        while let Some((x, y, z, lv)) = q.pop_front() {
            if lv <= 1 { continue; }
            let nlv = lv - 1;
            for (dx, dy, dz) in [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)] {
                let nx = x + dx; let ny = y + dy; let nz = z + dz;
                if let Some(ni) = self.idx_of(nx, ny, nz) {
                    if !is_opaque(self.data[ni]) && (self.light[ni] & 0x0F) < nlv {
                        self.light[ni] = (self.light[ni] & 0xF0) | nlv;
                        q.push_back((nx, ny, nz, nlv));
                    }
                }
            }
        }
        self.light_dirty = false;
    }

    #[inline(always)]
    pub fn get(&self, x: i32, y: i32, z: i32) -> u16 {
        if y < 0 {
            return if self.dim == Dimension::End { AIR } else { BEDROCK };
        }
        if y >= WORLD_H as i32 { return AIR; }
        let rx = x - self.ox;
        let rz = z - self.oz;
        if (rx as u32) >= CACHE_BLOCKS as u32 || (rz as u32) >= CACHE_BLOCKS as u32 {
            return AIR;
        }
        let cxi = (rx >> 4) as usize;
        let czi = (rz >> 4) as usize;
        let lx = (rx & 15) as usize;
        let lz = (rz & 15) as usize;
        let ci = (cxi * CACHE_SIDE + czi) * CHUNK_LEN;
        self.data[ci + (y as usize * CHUNK as usize + lx) * CHUNK as usize + lz]
    }

    #[inline]
    pub fn set_if_in_bounds(&mut self, x: i32, y: i32, z: i32, id: u16) {
        if !self.valid || y < 0 || y >= WORLD_H as i32 { return; }
        let rx = x - self.ox;
        let rz = z - self.oz;
        if (rx as u32) >= CACHE_BLOCKS as u32 || (rz as u32) >= CACHE_BLOCKS as u32 { return; }
        let cxi = (rx >> 4) as usize;
        let czi = (rz >> 4) as usize;
        let lx = (rx & 15) as usize;
        let lz = (rz & 15) as usize;
        let ci = (cxi * CACHE_SIDE + czi) * CHUNK_LEN;
        self.data[ci + (y as usize * CHUNK as usize + lx) * CHUNK as usize + lz] = id;
    }

    #[inline]
    pub fn is_lit(&self, x: i32, y: i32, z: i32) -> bool {
        !self.lit.is_empty() && self.lit.contains(&(x, y, z))
    }
}

struct DimState {
    chunks: HashMap<(i32, i32), Chunk>,
    edits: HashMap<BPos, u16>,
    hot: Option<((i32, i32), Chunk)>,
}

impl DimState {
    fn new() -> Self {
        Self { chunks: HashMap::new(), edits: HashMap::new(), hot: None }
    }
}

pub struct World {
    pub dim: Dimension,
    dims: [DimState; 3],
    pub lit: HashSet<(i32, i32, i32)>,
    cache: RenderCache,
}

impl World {
    pub fn new() -> Self {
        Self {
            dim: Dimension::Overworld,
            dims: [DimState::new(), DimState::new(), DimState::new()],
            lit: HashSet::new(),
            cache: RenderCache::new(),
        }
    }

    pub fn set_dim(&mut self, dim: Dimension) {
        if self.dim != dim {
            self.dim = dim;
            self.cache.valid = false;
        }
    }

    pub fn biome(&self, x: i32, z: i32) -> Biome {
        biome_at_dim(self.dim, x, z)
    }

    fn chunk(&mut self, cx: i32, cz: i32) -> &mut Chunk {
        let dim = self.dim;
        let st = &mut self.dims[dim.idx()];
        let need = st.hot.as_ref().map(|(c, _)| *c) != Some((cx, cz));
        if need {
            if let Some((coord, ch)) = st.hot.take() { st.chunks.insert(coord, ch); }
            let ch = st.chunks.remove(&(cx, cz)).unwrap_or_else(|| generate_chunk(dim, cx, cz));
            st.hot = Some(((cx, cz), ch));
        }
        &mut st.hot.as_mut().unwrap().1
    }

    #[inline]
    pub fn get(&mut self, x: i32, y: i32, z: i32) -> u16 {
        if y < 0 { return if self.dim == Dimension::End { AIR } else { BEDROCK }; }
        if y >= WORLD_H as i32 { return AIR; }
        let st = &self.dims[self.dim.idx()];
        if !st.edits.is_empty() {
            if let Some(v) = st.edits.get(&BPos(x, y, z)) { return *v; }
        }
        let cx = x >> 4;
        let cz = z >> 4;
        let lx = x & 15;
        let lz = z & 15;
        self.chunk(cx, cz).get(lx, y as usize, lz)
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, id: u16) {
        if y < 0 || y >= WORLD_H as i32 { return; }
        self.dims[self.dim.idx()].edits.insert(BPos(x, y, z), id);
        self.cache.set_if_in_bounds(x, y, z, id);
        self.cache.light_dirty = true;
    }

    pub fn all_edits(&self) -> Vec<(u8, i32, i32, i32, u16)> {
        let mut out = Vec::new();
        for (d, st) in self.dims.iter().enumerate() {
            for (&BPos(x, y, z), &id) in &st.edits {
                out.push((d as u8, x, y, z, id));
            }
        }
        out
    }

    pub fn insert_dim_edit(&mut self, dim: Dimension, x: i32, y: i32, z: i32, id: u16) {
        if y < 0 || y >= WORLD_H as i32 { return; }
        self.dims[dim.idx()].edits.insert(BPos(x, y, z), id);
        if self.dim == dim {
            self.cache.set_if_in_bounds(x, y, z, id);
        }
    }

    pub fn sync_cache(&mut self, cam_x: f32, cam_z: f32) -> &RenderCache {
        let pcx = (cam_x.floor() as i32) >> 4;
        let pcz = (cam_z.floor() as i32) >> 4;
        if !self.cache.valid || self.cache.dim != self.dim || self.cache.cx != pcx || self.cache.cz != pcz {
            self.cache.dim = self.dim;
            self.cache.cx = pcx;
            self.cache.cz = pcz;
            self.cache.ox = (pcx - CACHE_R) * CHUNK;
            self.cache.oz = (pcz - CACHE_R) * CHUNK;
            let mut data = std::mem::take(&mut self.cache.data);
            for dcx in 0..CACHE_SIDE {
                let cx = pcx - CACHE_R + dcx as i32;
                for dcz in 0..CACHE_SIDE {
                    let cz = pcz - CACHE_R + dcz as i32;
                    let off = (dcx * CACHE_SIDE + dcz) * CHUNK_LEN;
                    let ch = self.chunk(cx, cz);
                    data[off..off + CHUNK_LEN].copy_from_slice(&ch.blocks);
                }
            }
            self.cache.data = data;
            self.cache.valid = true;
            let edits_ref = &self.dims[self.dim.idx()].edits;
            for (&BPos(ex, ey, ez), &id) in edits_ref {
                self.cache.set_if_in_bounds(ex, ey, ez, id);
            }
            self.cache.light_dirty = true;
        }
        if self.cache.lit != self.lit.iter().copied().collect::<Vec<_>>() {
            self.cache.lit.clear();
            for &p in &self.lit {
                self.cache.lit.push(p);
            }
            self.cache.light_dirty = true;
        }
        if self.cache.light_dirty {
            self.cache.rebuild_light();
        }
        &self.cache
    }

    pub fn highest_ground(&mut self, x: i32, z: i32) -> i32 {
        let top_y = if self.dim == Dimension::Nether { 78 } else { WORLD_H as i32 - 1 };
        for y in (1..=top_y).rev() {
            let b = self.get(x, y, z);
            if is_solid(b)
                && b != WATER
                && b != LAVA
                && !matches!(
                    b,
                    LEAVES | SPRUCE_LEAVES | JUNGLE_LEAVES | BIRCH_LEAVES | DARK_LEAVES
                        | CRIMSON_WART | WARPED_WART
                )
            {
                return y + 1;
            }
        }
        if self.dim == Dimension::Nether { 34 } else if self.dim == Dimension::End { 61 } else { SEA + 1 }
    }

    pub fn prune(&mut self, cx: i32, cz: i32) {
        let st = &mut self.dims[self.dim.idx()];
        if st.chunks.len() < 220 { return; }
        let mut keys: Vec<(i32, i32)> = st.chunks.keys().copied().collect();
        keys.sort_by_key(|k| ((k.0 - cx).abs() + (k.1 - cz).abs()) as i64);
        for k in keys.iter().skip(100) { st.chunks.remove(k); }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Hit {
    pub block: u16,
    pub x: i32, pub y: i32, pub z: i32,
    pub nx: i32, pub ny: i32, pub nz: i32,
    pub dist: f32,
}

#[inline]
pub fn hit_uv(ox: f32, oy: f32, oz: f32, dx: f32, dy: f32, dz: f32, t: f32, nx: i32, ny: i32, nz: i32) -> (f32, f32) {
    if nx != 0 {
        let raw_u = (oz + dz * t).rem_euclid(1.0);
        let u = if nx > 0 { 1.0 - raw_u } else { raw_u };
        let v = 1.0 - (oy + dy * t).rem_euclid(1.0);
        (u, v)
    } else if ny != 0 {
        ((ox + dx * t).rem_euclid(1.0), (oz + dz * t).rem_euclid(1.0))
    } else {
        let raw_u = (ox + dx * t).rem_euclid(1.0);
        let u = if nz > 0 { raw_u } else { 1.0 - raw_u };
        let v = 1.0 - (oy + dy * t).rem_euclid(1.0);
        (u, v)
    }
}

pub fn raycast(world: &mut World, ox: f32, oy: f32, oz: f32, dx: f32, dy: f32, dz: f32, max_dist: f32, include_liquid: bool) -> Option<Hit> {
    raycast_opt(world, ox, oy, oz, dx, dy, dz, max_dist, include_liquid, true)
}

pub fn raycast_solid(world: &mut World, ox: f32, oy: f32, oz: f32, dx: f32, dy: f32, dz: f32, max_dist: f32) -> Option<Hit> {
    raycast_opt(world, ox, oy, oz, dx, dy, dz, max_dist, false, false)
}

fn raycast_opt(world: &mut World, ox: f32, oy: f32, oz: f32, dx: f32, dy: f32, dz: f32, max_dist: f32, include_liquid: bool, hit_non_solid: bool) -> Option<Hit> {
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
    let mut t = 0.0;
    for _ in 0..220 {
        let b = world.get(x, y, z);
        let hit_b = if hit_non_solid {
            is_targetable(b)
        } else {
            is_solid(b)
        };
        let solid = hit_b || (include_liquid && (b == WATER || b == LAVA || b == NETHER_PORTAL || b == END_PORTAL));
        if solid && t > 0.0001 {
            return Some(Hit { block: b, x, y, z, nx, ny, nz, dist: t });
        }
        if tmx < tmy && tmx < tmz { t = tmx; tmx += dtx; x += step_x; nx = -step_x; ny = 0; nz = 0; }
        else if tmy < tmz { t = tmy; tmy += dty; y += step_y; ny = -step_y; nx = 0; nz = 0; }
        else { t = tmz; tmz += dtz; z += step_z; nz = -step_z; nx = 0; ny = 0; }
        if t > max_dist { break; }
        if y < 0 || y >= WORLD_H as i32 { break; }
    }
    None
}
