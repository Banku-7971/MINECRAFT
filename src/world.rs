// Procedural voxel world: biomes, caves, ores, trees, villages and ruins.
use std::collections::HashMap;

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
pub const MAX_BLOCK: u16 = 100;

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
pub enum Biome { Plains, Forest, Desert, Snowy, Jungle, Swamp, Mountains, Ocean, Beach, Taiga, Savanna }
impl Biome {
    pub fn name(self) -> &'static str {
        match self {
            Biome::Plains => "Plains", Biome::Forest => "Forest", Biome::Desert => "Desert",
            Biome::Snowy => "Snowy Tundra", Biome::Jungle => "Jungle", Biome::Swamp => "Swamp",
            Biome::Mountains => "Mountains", Biome::Ocean => "Ocean", Biome::Beach => "Beach",
            Biome::Taiga => "Taiga", Biome::Savanna => "Savanna",
        }
    }
}

pub fn biome_at(x: i32, z: i32) -> Biome {
    let temp = fbm2(x as f32 / 420.0 + 17.0, z as f32 / 420.0 - 8.0, 3);
    let humid = fbm2(x as f32 / 360.0 - 31.0, z as f32 / 360.0 + 12.0, 3);
    let cont = fbm2(x as f32 / 560.0, z as f32 / 560.0, 3);
    if cont < -0.24 { return Biome::Ocean; }
    if cont < -0.15 { return Biome::Beach; }
    if cont > 0.32 { return Biome::Mountains; }
    if temp > 0.24 { return if humid < -0.1 { Biome::Savanna } else { Biome::Desert }; }
    if temp < -0.30 { return Biome::Snowy; }
    if humid > 0.26 { return if temp > 0.08 { Biome::Jungle } else { Biome::Swamp }; }
    if temp < -0.08 { return Biome::Taiga; }
    if humid > 0.08 { return Biome::Forest; }
    Biome::Plains
}

pub fn surface_height(x: i32, z: i32) -> i32 {
    let b = biome_at(x, z);
    let rolling = fbm2(x as f32 / 96.0, z as f32 / 96.0, 4) * 9.0;
    let detail = noise2(x as f32 / 22.0, z as f32 / 22.0) * 2.0;
    let mut h = SEA as f32 + 6.0 + rolling + detail;
    match b {
        Biome::Ocean => h -= 12.0 + fbm2(x as f32 / 60.0, z as f32 / 60.0, 2) * 4.0,
        Biome::Beach => h = SEA as f32 + 1.0 + detail * 0.5,
        Biome::Desert | Biome::Savanna => h += 3.0,
        Biome::Swamp => h -= 3.0,
        Biome::Mountains => h += 26.0 + fbm2(x as f32 / 48.0, z as f32 / 48.0, 3) * 12.0,
        Biome::Snowy | Biome::Taiga => h += 4.0,
        Biome::Jungle => h += 2.0,
        _ => {}
    }
    h.max(4.0) as i32
}

fn cave_at(x: i32, y: i32, z: i32) -> bool {
    if y < 3 || y > 62 { return false; }
    let n = noise3(x as f32 / 17.0, y as f32 / 12.0, z as f32 / 17.0);
    let n2 = noise3(x as f32 / 31.0 + 40.0, y as f32 / 18.0, z as f32 / 31.0 - 20.0);
    (n + n2 * 0.5) > 0.46
}

fn ore_at(x: i32, y: i32, z: i32) -> Option<u16> {
    let h = hash3(x / 2, y / 2, z / 2);
    if y < 16 && h % 190 == 0 { return Some(DIAMOND_ORE); }
    if y < 30 && h % 97 == 0 { return Some(GOLD_ORE); }
    if y < 46 && h % 47 == 0 { return Some(IRON_ORE); }
    if y < 60 && h % 31 == 0 { return Some(COAL_ORE); }
    if y < 40 && hash3(x / 3, y / 3, z / 3) % 140 == 0 { return Some(REDSTONE_ORE); }
    if y < 44 && hash3(x / 4, y / 4, z / 4) % 110 == 0 { return Some(EMERALD_ORE); }
    None
}

fn tree_center(x: i32, z: i32) -> Option<(Biome, i32)> {
    let b = biome_at(x, z);
    let density = match b {
        Biome::Forest => 28, Biome::Jungle => 20, Biome::Taiga => 36,
        Biome::Plains => 110, Biome::Savanna => 85, Biome::Swamp => 55, _ => return None,
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
            let big = b == Biome::Jungle;
            let height = if big { 8 } else if b == Biome::Taiga { 7 } else { 5 + (hash2(x - dx, z - dz) % 3) as i32 };
            let trunk = if b == Biome::Jungle { JUNGLE_LOG } else if b == Biome::Taiga || b == Biome::Snowy { SPRUCE_LOG } else if b == Biome::Savanna { LOG } else { LOG };
            let leaf = if b == Biome::Jungle { JUNGLE_LEAVES } else if b == Biome::Taiga { SPRUCE_LEAVES } else { LEAVES };
            let top = h + height;
            if lx == 0 && lz == 0 && y >= h + 1 && y <= top { return Some(trunk); }
            let dy = y - top;
            if dy < -3 || dy > 1 { continue; }
            let rad = if dy >= 0 { 1 } else if dy == -1 { 2 } else { 2 };
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
    Some((cx * size + 64 + (hash2(cx, cz) % 80) as i32, cz * size + 64 + (hash2(cx + 7, cz - 3) % 80) as i32))
}

fn structure_block(x: i32, y: i32, z: i32) -> Option<u16> {
    let (vcx, vcz) = structure_cell(x, z, 256);
    for cx in vcx - 1..=vcx + 1 {
        for cz in vcz - 1..=vcz + 1 {
            if let Some((vx, vz)) = village_center(cx, cz) {
                let dx = x - vx;
                let dz = z - vz;
                let h = surface_height(x, z);
                if dx.abs() <= 20 && dz.abs() <= 20 {
                    if dx.abs() <= 1 || dz.abs() <= 1 {
                        if y == h { return Some(COBBLE); }
                    }
                    if dx.abs() <= 1 && dz.abs() <= 1 && y > h && y <= h + 1 {
                        return if dx == 0 && dz == 0 { Some(WATER) } else { Some(COBBLE) };
                    }
                    let houses = [(-13, -11), (11, -10), (-12, 12), (12, 12)];
                    for (ox, oz) in houses {
                        let hx = x - (vx + ox);
                        let hz = z - (vz + oz);
                        if hx >= -3 && hx <= 3 && hz >= -3 && hz <= 3 {
                            let wall = hx.abs() == 3 || hz.abs() == 3;
                            if y == h { return Some(PLANKS); }
                            if y > h && y <= h + 4 {
                                if wall {
                                    if hx == 0 && hz == 3 && y <= h + 2 { return Some(AIR); }
                                    if (hx.abs() == 3 && hz.abs() == 3) || (y == h + 4) { return Some(LOG); }
                                    if (hx.abs() == 3 && hz == 0) || (hz.abs() == 3 && hx == 0) { return Some(GLASS); }
                                    return Some(PLANKS);
                                }
                                return Some(AIR);
                            }
                            if y == h + 5 && hx.abs() <= 3 && hz.abs() <= 3 { return Some(STAIRS); }
                            if y == h + 6 && hx.abs() <= 2 && hz.abs() <= 2 { return Some(STAIRS); }
                            if y == h + 7 && hx.abs() <= 1 && hz.abs() <= 1 { return Some(STAIRS); }
                        }
                    }
                    if dx.abs() > 5 && dz.abs() > 5 && y == h && (hash2(x, z) % 5 == 0) {
                        return Some(FARMLAND);
                    }
                    if y == h + 1 && dx.abs() > 5 && dz.abs() > 5 && hash2(x + 9, z - 4) % 13 == 0 {
                        return Some(WHEAT);
                    }
                }
            }
        }
    }
    // Desert / jungle temples and ocean shipwrecks on a separate grid.
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
        _ => "Unknown",
    }
}

pub fn is_solid(id: u16) -> bool {
    !matches!(id, AIR | WATER | LAVA | TALL_GRASS | FLOWER_RED | FLOWER_YELLOW | COBWEB | VINE | WHEAT | TORCH | SNOW_LAYER | RAIL
        | REDSTONE_DUST | LEVER | PRESSURE_PLATE | REDSTONE_TORCH)
}

pub fn is_opaque(id: u16) -> bool {
    is_solid(id) && !matches!(id, GLASS | LEAVES | SPRUCE_LEAVES | JUNGLE_LEAVES | BIRCH_LEAVES | DARK_LEAVES | ICE)
}

pub fn emits_light(id: u16) -> u8 {
    match id { TORCH | CAMPFIRE | LANTERN => 14, GLOWSTONE | SEA_LANTERN => 15, LAVA | MAGMA => 12, _ => 0 }
}

fn chunk_may_have_structure(cx: i32, cz: i32) -> bool {
    let x0 = cx * CHUNK;
    let z0 = cz * CHUNK;
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

fn generate_chunk(cx: i32, cz: i32) -> Chunk {
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
                    if cave_at(x, y, z) && y < h - 2 && y > 3 {
                        id = if y < 18 && hash3(x, y, z) % 90 == 0 { LAVA } else { AIR };
                    } else {
                        id = STONE;
                        if let Some(o) = ore_at(x, y, z) { id = o; }
                        if y > h - 4 {
                            id = match b {
                                Biome::Desert => if y == h { SAND } else { SANDSTONE },
                                Biome::Savanna => if y == h { TERRACOTTA } else { DIRT },
                                Biome::Snowy => if y == h { SNOW } else { DIRT },
                                Biome::Taiga => if y == h && hash2(x, z) % 3 == 0 { PODZOL } else { DIRT },
                                Biome::Swamp => if y == h && hash2(x, z) % 4 == 0 { MYCELIUM } else { DIRT },
                                Biome::Ocean | Biome::Beach => if y > h - 2 { SAND } else { STONE },
                                _ => if y == h { GRASS } else { DIRT },
                            };
                        }
                    }
                } else if y <= SEA && (b == Biome::Ocean || b == Biome::Beach || b == Biome::Swamp) {
                    id = WATER;
                }
                if id == AIR || is_solid(id) {
                    if has_struct && y >= SEA - 4 {
                        if let Some(s) = structure_block(x, y, z) { id = s; }
                    }
                    if id == AIR && y > h && y <= h + 10 {
                        if let Some(t) = tree_block(x, y, z) { id = t; }
                    }
                }
                if id == AIR && y == h + 1 {
                    let hh = hash2(x + 5, z - 7);
                    if matches!(b, Biome::Plains | Biome::Forest | Biome::Savanna) {
                        if hh % 17 == 0 { id = TALL_GRASS; }
                        else if hh % 43 == 0 { id = FLOWER_RED; }
                        else if hh % 47 == 0 { id = FLOWER_YELLOW; }
                    } else if b == Biome::Desert && hh % 61 == 0 { id = CACTUS; }
                    else if b == Biome::Swamp && hh % 71 == 0 { id = if hh % 2 == 0 { RED_MUSHROOM } else { BROWN_MUSHROOM }; }
                }
                if id == AIR && y == h + 2 && b == Biome::Desert && hash2(x - 3, z + 8) % 61 == 0 { id = CACTUS; }
                c.set(lx, y as usize, lz, id);
            }
        }
    }
    // Mineshaft corridors: carved tunnels with supports and rails.
    let (mcx, mcz) = structure_cell(cx, cz, 3);
    for mx in mcx - 1..=mcx + 1 {
        for mz in mcz - 1..=mcz + 1 {
            if hash2(mx + 300, mz - 44) % 4 != 0 { continue; }
            let along_x = hash2(mx, mz) % 2 == 0;
            let origin = if along_x { mz * 48 + 24 } else { mx * 48 + 24 };
            for t in 0..48 {
                let (wx, wz) = if along_x { (mx * 48 + t, origin) } else { (origin, mz * 48 + t) };
                if wx / CHUNK != cx || wz / CHUNK != cz { continue; }
                let lx = wx - cx * CHUNK;
                let lz = wz - cz * CHUNK;
                let y = 24 + (hash2(mx, mz) % 6) as i32;
                for dy in 0..4 {
                    for off in -1..=1 {
                        let (px, pz) = if along_x { (wx, wz + off) } else { (wx + off, wz) };
                        if px / CHUNK != cx || pz / CHUNK != cz { continue; }
                        c.set(px - cx * CHUNK, (y + dy) as usize, pz - cz * CHUNK, AIR);
                    }
                }
                c.set(lx, y as usize, lz, RAIL);
                if t % 10 == 0 {
                    for off in -2..=2 {
                        for dy in 0..4 {
                            let (px, pz) = if along_x { (wx, wz + off) } else { (wx + off, wz) };
                            if px / CHUNK != cx || pz / CHUNK != cz { continue; }
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

pub struct World {
    chunks: HashMap<(i32, i32), Chunk>,
    pub edits: HashMap<BPos, u16>,
    hot: Option<((i32, i32), Chunk)>,
    /// Redstone lamps that are currently powered and glowing.
    pub lit: std::collections::HashSet<(i32, i32, i32)>,
}

impl World {
    pub fn new() -> Self {
        Self { chunks: HashMap::new(), edits: HashMap::new(), hot: None, lit: std::collections::HashSet::new() }
    }

    fn chunk(&mut self, cx: i32, cz: i32) -> &mut Chunk {
        let need = self.hot.as_ref().map(|(c, _)| *c) != Some((cx, cz));
        if need {
            if let Some((coord, ch)) = self.hot.take() { self.chunks.insert(coord, ch); }
            let ch = self.chunks.remove(&(cx, cz)).unwrap_or_else(|| generate_chunk(cx, cz));
            self.hot = Some(((cx, cz), ch));
        }
        &mut self.hot.as_mut().unwrap().1
    }

    #[inline]
    pub fn get(&mut self, x: i32, y: i32, z: i32) -> u16 {
        if y < 0 { return BEDROCK; }
        if y >= WORLD_H as i32 { return AIR; }
        if !self.edits.is_empty() {
            if let Some(v) = self.edits.get(&BPos(x, y, z)) { return *v; }
        }
        let cx = x >> 4;
        let cz = z >> 4;
        let lx = x & 15;
        let lz = z & 15;
        self.chunk(cx, cz).get(lx, y as usize, lz)
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, id: u16) {
        if y < 0 || y >= WORLD_H as i32 { return; }
        self.edits.insert(BPos(x, y, z), id);
    }

    pub fn highest_ground(&mut self, x: i32, z: i32) -> i32 {
        for y in (0..WORLD_H as i32).rev() {
            let b = self.get(x, y, z);
            if is_solid(b) && b != WATER && !matches!(b, LEAVES | SPRUCE_LEAVES | JUNGLE_LEAVES | BIRCH_LEAVES | DARK_LEAVES) {
                return y + 1;
            }
        }
        SEA + 1
    }

    pub fn prune(&mut self, cx: i32, cz: i32) {
        if self.chunks.len() < 220 { return; }
        let mut keys: Vec<(i32, i32)> = self.chunks.keys().copied().collect();
        keys.sort_by_key(|k| ((k.0 - cx).abs() + (k.1 - cz).abs()) as i64);
        for k in keys.iter().skip(100) { self.chunks.remove(k); }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Hit {
    pub block: u16,
    pub x: i32, pub y: i32, pub z: i32,
    pub nx: i32, pub ny: i32, pub nz: i32,
    pub dist: f32,
    pub u: f32, pub v: f32,
}

// Amanatides & Woo voxel traversal.
pub fn raycast(world: &mut World, ox: f32, oy: f32, oz: f32, dx: f32, dy: f32, dz: f32, max_dist: f32, include_liquid: bool) -> Option<Hit> {
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
    for _ in 0..128 {
        let b = world.get(x, y, z);
        let solid = is_solid(b) || (include_liquid && (b == WATER || b == LAVA));
        if solid && t > 0.0001 {
            let (u, v) = if nx != 0 {
                ((oz + dz * t).fract(), (oy + dy * t).fract())
            } else if ny != 0 {
                ((ox + dx * t).fract(), (oz + dz * t).fract())
            } else {
                ((ox + dx * t).fract(), (oy + dy * t).fract())
            };
            return Some(Hit { block: b, x, y, z, nx, ny, nz, dist: t, u, v });
        }
        if tmx < tmy && tmx < tmz { t = tmx; tmx += dtx; x += step_x; nx = -step_x; ny = 0; nz = 0; }
        else if tmy < tmz { t = tmy; tmy += dty; y += step_y; ny = -step_y; nx = 0; nz = 0; }
        else { t = tmz; tmz += dtz; z += step_z; nz = -step_z; nx = 0; ny = 0; }
        if t > max_dist { break; }
        if y < 0 || y >= WORLD_H as i32 { break; }
    }
    None
}
