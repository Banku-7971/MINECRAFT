//! Crafting recipes and furnace smelting, Minecraft style.
use crate::{items, world};
use std::sync::{OnceLock, RwLock};

pub const GRID: usize = 9;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Stack {
    pub id: u16,
    pub count: u32,
}

impl Stack {
    pub const EMPTY: Stack = Stack { id: 0, count: 0 };
    pub fn new(id: u16, count: u32) -> Self { Stack { id, count } }
    pub fn is_empty(&self) -> bool { self.id == 0 || self.count == 0 }
}

#[derive(Clone)]
pub struct Recipe {
    /// 3x3 pattern, 0 = empty. Rows are top to bottom.
    pub pattern: [u16; 9],
    pub out: u16,
    pub count: u32,
}

#[derive(Clone)]
pub struct ShapelessRecipe {
    pub ingredients: Vec<u16>,
    pub out: u16,
    pub count: u32,
}

static CUSTOM_SHAPED: OnceLock<RwLock<Vec<Recipe>>> = OnceLock::new();
static CUSTOM_SHAPELESS: OnceLock<RwLock<Vec<ShapelessRecipe>>> = OnceLock::new();
static CUSTOM_SMELT: OnceLock<RwLock<Vec<(u16, u16)>>> = OnceLock::new();

pub fn clear_custom_recipes() {
    if let Ok(mut g) = CUSTOM_SHAPED.get_or_init(|| RwLock::new(Vec::new())).write() { g.clear(); }
    if let Ok(mut g) = CUSTOM_SHAPELESS.get_or_init(|| RwLock::new(Vec::new())).write() { g.clear(); }
    if let Ok(mut g) = CUSTOM_SMELT.get_or_init(|| RwLock::new(Vec::new())).write() { g.clear(); }
}

pub fn add_custom_shaped(r: Recipe) {
    if let Ok(mut g) = CUSTOM_SHAPED.get_or_init(|| RwLock::new(Vec::new())).write() { g.push(r); }
}

pub fn add_custom_shapeless(r: ShapelessRecipe) {
    if let Ok(mut g) = CUSTOM_SHAPELESS.get_or_init(|| RwLock::new(Vec::new())).write() { g.push(r); }
}

pub fn add_custom_smelt(input: u16, output: u16) {
    if let Ok(mut g) = CUSTOM_SMELT.get_or_init(|| RwLock::new(Vec::new())).write() {
        g.retain(|(i, _)| *i != input);
        g.push((input, output));
    }
}

macro_rules! shaped {
    ($out:expr, $count:expr, [$($a:expr, $b:expr, $c:expr);+]) => {
        Recipe { pattern: [ $( $a, $b, $c ),+ ], out: $out, count: $count }
    };
}

pub fn recipes() -> Vec<Recipe> {
    use items::*;
    let p = world::PLANKS;
    let c = world::COBBLE;
    let s = STICK;
    let ir = IRON_INGOT;
    let di = DIAMOND;
    vec![
        // --- basics ---
        shaped!(p, 4, [0, world::LOG, 0; 0, 0, 0; 0, 0, 0]),
        shaped!(p, 4, [0, world::BIRCH_LOG, 0; 0, 0, 0; 0, 0, 0]),
        shaped!(p, 4, [0, world::SPRUCE_LOG, 0; 0, 0, 0; 0, 0, 0]),
        shaped!(p, 4, [0, world::JUNGLE_LOG, 0; 0, 0, 0; 0, 0, 0]),
        shaped!(p, 4, [0, world::DARK_LOG, 0; 0, 0, 0; 0, 0, 0]),
        shaped!(p, 4, [0, world::CRIMSON_STEM, 0; 0, 0, 0; 0, 0, 0]),
        shaped!(p, 4, [0, world::WARPED_STEM, 0; 0, 0, 0; 0, 0, 0]),
        shaped!(BLAZE_POWDER, 2, [0, BLAZE_ROD, 0; 0, 0, 0; 0, 0, 0]),
        shaped!(EYE_OF_ENDER, 1, [ENDER_PEARL, BLAZE_POWDER, 0; 0, 0, 0; 0, 0, 0]),
        shaped!(world::NETHER_BRICKS, 1, [world::NETHERRACK, world::NETHERRACK, 0; world::NETHERRACK, world::NETHERRACK, 0; 0, 0, 0]),
        shaped!(s, 4, [0, p, 0; 0, p, 0; 0, 0, 0]),
        shaped!(world::CRAFTING, 1, [p, p, 0; p, p, 0; 0, 0, 0]),
        shaped!(world::FURNACE, 1, [c, c, c; c, 0, c; c, c, c]),
        shaped!(world::CHEST, 1, [p, p, p; p, 0, p; p, p, p]),
        shaped!(world::TORCH, 4, [0, COAL, 0; 0, s, 0; 0, 0, 0]),
        shaped!(world::BED, 1, [world::WOOL_WHITE, world::WOOL_WHITE, world::WOOL_WHITE; p, p, p; 0, 0, 0]),
        shaped!(world::BOOKSHELF, 1, [p, p, p; world::WOOL_WHITE, world::WOOL_WHITE, world::WOOL_WHITE; p, p, p]),
        shaped!(world::LANTERN, 1, [0, 0, 0; ir, world::TORCH, ir; 0, 0, 0]),
        shaped!(world::LADDER, 3, [s, 0, s; s, s, s; s, 0, s]),
        shaped!(world::FENCE, 3, [p, s, p; p, s, p; 0, 0, 0]),
        shaped!(world::STAIRS, 4, [c, 0, 0; c, c, 0; c, c, c]),
        shaped!(world::SLAB, 6, [c, c, c; 0, 0, 0; 0, 0, 0]),
        shaped!(world::BRICK, 1, [world::CLAY, world::CLAY, 0; world::CLAY, world::CLAY, 0; 0, 0, 0]),
        // --- redstone ---
        shaped!(world::REDSTONE_TORCH, 1, [0, world::REDSTONE_DUST, 0; 0, s, 0; 0, 0, 0]),
        shaped!(world::LEVER, 1, [0, s, 0; 0, c, 0; 0, 0, 0]),
        shaped!(world::PRESSURE_PLATE, 1, [c, c, 0; 0, 0, 0; 0, 0, 0]),
        shaped!(world::LAMP, 1, [0, world::REDSTONE_DUST, 0;
                                world::REDSTONE_DUST, world::GLOWSTONE, world::REDSTONE_DUST;
                                0, world::REDSTONE_DUST, 0]),
        // --- tools ---
        shaped!(WOOD_PICK, 1, [p, p, p; 0, s, 0; 0, s, 0]),
        shaped!(STONE_PICK, 1, [c, c, c; 0, s, 0; 0, s, 0]),
        shaped!(IRON_PICK, 1, [ir, ir, ir; 0, s, 0; 0, s, 0]),
        shaped!(GOLD_PICK, 1, [GOLD_INGOT, GOLD_INGOT, GOLD_INGOT; 0, s, 0; 0, s, 0]),
        shaped!(DIAMOND_PICK, 1, [di, di, di; 0, s, 0; 0, s, 0]),
        shaped!(WOOD_AXE, 1, [p, p, 0; p, s, 0; 0, s, 0]),
        shaped!(STONE_AXE, 1, [c, c, 0; c, s, 0; 0, s, 0]),
        shaped!(IRON_AXE, 1, [ir, ir, 0; ir, s, 0; 0, s, 0]),
        shaped!(GOLD_AXE, 1, [GOLD_INGOT, GOLD_INGOT, 0; GOLD_INGOT, s, 0; 0, s, 0]),
        shaped!(DIAMOND_AXE, 1, [di, di, 0; di, s, 0; 0, s, 0]),
        shaped!(WOOD_SHOVEL, 1, [0, p, 0; 0, s, 0; 0, s, 0]),
        shaped!(STONE_SHOVEL, 1, [0, c, 0; 0, s, 0; 0, s, 0]),
        shaped!(IRON_SHOVEL, 1, [0, ir, 0; 0, s, 0; 0, s, 0]),
        shaped!(GOLD_SHOVEL, 1, [0, GOLD_INGOT, 0; 0, s, 0; 0, s, 0]),
        shaped!(DIAMOND_SHOVEL, 1, [0, di, 0; 0, s, 0; 0, s, 0]),
        shaped!(WOOD_SWORD, 1, [0, p, 0; 0, p, 0; 0, s, 0]),
        shaped!(STONE_SWORD, 1, [0, c, 0; 0, c, 0; 0, s, 0]),
        shaped!(IRON_SWORD, 1, [0, ir, 0; 0, ir, 0; 0, s, 0]),
        shaped!(GOLD_SWORD, 1, [0, GOLD_INGOT, 0; 0, GOLD_INGOT, 0; 0, s, 0]),
        shaped!(DIAMOND_SWORD, 1, [0, di, 0; 0, di, 0; 0, s, 0]),
        shaped!(BUCKET, 1, [ir, 0, ir; 0, ir, 0; 0, 0, 0]),
        shaped!(SHIELD, 1, [p, ir, p; p, ir, p; 0, ir, 0]),
        shaped!(BOW, 1, [0, s, STRING; s, 0, STRING; 0, s, STRING]),
        shaped!(FISHING_ROD, 1, [0, 0, s; 0, s, STRING; s, 0, STRING]),
        shaped!(SPYGLASS, 1, [0, AMETHYST, 0; 0, GOLD_INGOT, 0; 0, GOLD_INGOT, 0]),
        shaped!(ARROW, 4, [0, FLINT, 0; 0, s, 0; 0, FEATHER, 0]),
        shaped!(FLINT_AND_STEEL, 1, [ir, 0, 0; 0, FLINT, 0; 0, 0, 0]),
        shaped!(SHEARS, 1, [0, ir, 0; ir, 0, 0; 0, 0, 0]),
        shaped!(NETHERITE_SWORD, 1, [0, DIAMOND_SWORD, 0; 0, NETHERITE_INGOT, 0; 0, 0, 0]),
        shaped!(NETHERITE_PICK, 1, [0, DIAMOND_PICK, 0; 0, NETHERITE_INGOT, 0; 0, 0, 0]),
        shaped!(NETHERITE_AXE, 1, [0, DIAMOND_AXE, 0; 0, NETHERITE_INGOT, 0; 0, 0, 0]),
        shaped!(NETHERITE_SHOVEL, 1, [0, DIAMOND_SHOVEL, 0; 0, NETHERITE_INGOT, 0; 0, 0, 0]),
        shaped!(NETHERITE_HOE, 1, [0, DIAMOND_HOE, 0; 0, NETHERITE_INGOT, 0; 0, 0, 0]),
        shaped!(GOLD_HELMET, 1, [GOLD_INGOT, GOLD_INGOT, GOLD_INGOT; GOLD_INGOT, 0, GOLD_INGOT; 0, 0, 0]),
        shaped!(GOLD_CHESTPLATE, 1, [GOLD_INGOT, 0, GOLD_INGOT; GOLD_INGOT, GOLD_INGOT, GOLD_INGOT; GOLD_INGOT, GOLD_INGOT, GOLD_INGOT]),
        shaped!(GOLD_LEGGINGS, 1, [GOLD_INGOT, GOLD_INGOT, GOLD_INGOT; GOLD_INGOT, 0, GOLD_INGOT; GOLD_INGOT, 0, GOLD_INGOT]),
        shaped!(GOLD_BOOTS, 1, [GOLD_INGOT, 0, GOLD_INGOT; GOLD_INGOT, 0, GOLD_INGOT; 0, 0, 0]),
        shaped!(IRON_HELMET, 1, [ir, ir, ir; ir, 0, ir; 0, 0, 0]),
        shaped!(IRON_CHESTPLATE, 1, [ir, 0, ir; ir, ir, ir; ir, ir, ir]),
        shaped!(IRON_LEGGINGS, 1, [ir, ir, ir; ir, 0, ir; ir, 0, ir]),
        shaped!(IRON_BOOTS, 1, [ir, 0, ir; ir, 0, ir; 0, 0, 0]),
        shaped!(DIAMOND_HELMET, 1, [di, di, di; di, 0, di; 0, 0, 0]),
        shaped!(DIAMOND_CHESTPLATE, 1, [di, 0, di; di, di, di; di, di, di]),
        shaped!(DIAMOND_LEGGINGS, 1, [di, di, di; di, 0, di; di, 0, di]),
        shaped!(DIAMOND_BOOTS, 1, [di, 0, di; di, 0, di; 0, 0, 0]),
        shaped!(NETHERITE_HELMET, 1, [0, DIAMOND_HELMET, 0; 0, NETHERITE_INGOT, 0; 0, 0, 0]),
        shaped!(NETHERITE_CHESTPLATE, 1, [0, DIAMOND_CHESTPLATE, 0; 0, NETHERITE_INGOT, 0; 0, 0, 0]),
        shaped!(NETHERITE_LEGGINGS, 1, [0, DIAMOND_LEGGINGS, 0; 0, NETHERITE_INGOT, 0; 0, 0, 0]),
        shaped!(NETHERITE_BOOTS, 1, [0, DIAMOND_BOOTS, 0; 0, NETHERITE_INGOT, 0; 0, 0, 0]),
        shaped!(WOOD_HOE, 1, [p, p, 0; 0, s, 0; 0, s, 0]),
        shaped!(STONE_HOE, 1, [c, c, 0; 0, s, 0; 0, s, 0]),
        shaped!(IRON_HOE, 1, [ir, ir, 0; 0, s, 0; 0, s, 0]),
        shaped!(GOLD_HOE, 1, [GOLD_INGOT, GOLD_INGOT, 0; 0, s, 0; 0, s, 0]),
        shaped!(DIAMOND_HOE, 1, [di, di, 0; 0, s, 0; 0, s, 0]),
        shaped!(PAPER, 3, [BAMBOO, BAMBOO, BAMBOO; 0, 0, 0; 0, 0, 0]),
        shaped!(SUGAR, 1, [0, BAMBOO, 0; 0, 0, 0; 0, 0, 0]),
        shaped!(BOOK, 1, [PAPER, PAPER, 0; PAPER, LEATHER, 0; 0, 0, 0]),
        shaped!(world::BOOKSHELF, 1, [p, p, p; BOOK, BOOK, BOOK; p, p, p]),
        shaped!(world::SIGN, 3, [p, p, p; p, p, p; 0, s, 0]),
        shaped!(world::CAMPFIRE, 1, [0, s, 0; s, COAL, s; world::LOG, world::LOG, world::LOG]),
        shaped!(world::STONE_BRICKS, 4, [world::STONE, world::STONE, 0; world::STONE, world::STONE, 0; 0, 0, 0]),
        shaped!(COOKIE, 8, [world::WHEAT, SUGAR, world::WHEAT; 0, 0, 0; 0, 0, 0]),
        shaped!(PUMPKIN_PIE, 1, [world::PUMPKIN, SUGAR, EGG; 0, 0, 0; 0, 0, 0]),
        shaped!(GOLDEN_CARROT, 1, [GOLD_NUGGET, GOLD_NUGGET, GOLD_NUGGET; GOLD_NUGGET, CARROT, GOLD_NUGGET; GOLD_NUGGET, GOLD_NUGGET, GOLD_NUGGET]),
        shaped!(GOLD_INGOT, 1, [GOLD_NUGGET, GOLD_NUGGET, GOLD_NUGGET; GOLD_NUGGET, GOLD_NUGGET, GOLD_NUGGET; GOLD_NUGGET, GOLD_NUGGET, GOLD_NUGGET]),
        shaped!(GOLD_NUGGET, 9, [0, GOLD_INGOT, 0; 0, 0, 0; 0, 0, 0]),
        shaped!(IRON_INGOT, 9, [0, world::IRON_BLOCK, 0; 0, 0, 0; 0, 0, 0]),
        shaped!(GOLD_INGOT, 9, [0, world::GOLD_BLOCK, 0; 0, 0, 0; 0, 0, 0]),
        shaped!(DIAMOND, 9, [0, world::DIAMOND_BLOCK, 0; 0, 0, 0; 0, 0, 0]),
        shaped!(world::EMERALD_BLOCK, 1, [EMERALD, EMERALD, EMERALD; EMERALD, EMERALD, EMERALD; EMERALD, EMERALD, EMERALD]),
        shaped!(EMERALD, 9, [0, world::EMERALD_BLOCK, 0; 0, 0, 0; 0, 0, 0]),
        shaped!(world::QUARTZ, 1, [NETHER_QUARTZ, NETHER_QUARTZ, 0; NETHER_QUARTZ, NETHER_QUARTZ, 0; 0, 0, 0]),
        shaped!(world::CAKE, 1, [world::WHEAT, SUGAR, world::WHEAT; EGG, APPLE, EGG; world::WHEAT, world::WHEAT, world::WHEAT]),
        shaped!(BONE_MEAL, 3, [0, BONE, 0; 0, 0, 0; 0, 0, 0]),
        shaped!(BOAT, 1, [p, 0, p; p, p, p; 0, 0, 0]),
        shaped!(MINECART, 1, [ir, 0, ir; ir, ir, ir; 0, 0, 0]),
        shaped!(GLASS_BOTTLE, 3, [world::GLASS, 0, world::GLASS; 0, world::GLASS, 0; 0, 0, 0]),
        shaped!(world::BREWING_STAND, 1, [0, BLAZE_ROD, 0; c, c, c; 0, 0, 0]),
        shaped!(world::ENCHANTING, 1, [0, world::BOOKSHELF, 0; di, world::OBSIDIAN, di; world::OBSIDIAN, world::OBSIDIAN, world::OBSIDIAN]),
        shaped!(world::ANVIL, 1, [world::IRON_BLOCK, world::IRON_BLOCK, world::IRON_BLOCK; 0, ir, 0; ir, ir, ir]),
        shaped!(world::BEACON, 1, [world::GLASS, world::GLASS, world::GLASS; world::GLASS, NETHER_STAR, world::GLASS; world::OBSIDIAN, world::OBSIDIAN, world::OBSIDIAN]),
        shaped!(world::JUKEBOX, 1, [p, p, p; p, di, p; p, p, p]),
        shaped!(world::SOUL_TORCH, 4, [0, COAL, 0; 0, s, 0; 0, world::SOUL_SAND, 0]),
        shaped!(FIREWORK, 3, [0, GUNPOWDER, 0; 0, FEATHER, 0; 0, 0, 0]),
        shaped!(MAP, 1, [FEATHER, FEATHER, FEATHER; FEATHER, COMPASS, FEATHER; FEATHER, FEATHER, FEATHER]),
        shaped!(world::HOPPER, 1, [ir, 0, ir; ir, world::CHEST, ir; 0, ir, 0]),
        shaped!(world::PISTON, 1, [p, p, p; c, ir, c; c, world::REDSTONE_DUST, c]),
        shaped!(world::STICKY_PISTON, 1, [0, SLIMEBALL, 0; 0, world::PISTON, 0; 0, 0, 0]),
        shaped!(world::REPEATER, 1, [world::REDSTONE_TORCH, world::REDSTONE_DUST, world::REDSTONE_TORCH; world::STONE, world::STONE, world::STONE; 0, 0, 0]),
        shaped!(world::COMPARATOR, 1, [0, world::REDSTONE_TORCH, 0; world::REDSTONE_TORCH, NETHER_QUARTZ, world::REDSTONE_TORCH; world::STONE, world::STONE, world::STONE]),
        shaped!(world::BUTTON, 1, [0, world::STONE, 0; 0, 0, 0; 0, 0, 0]),
        shaped!(world::DOOR, 3, [p, p, 0; p, p, 0; p, p, 0]),
        shaped!(world::IRON_DOOR, 3, [ir, ir, 0; ir, ir, 0; ir, ir, 0]),
        shaped!(world::LAPIS_BLOCK, 1, [LAPIS_LAZULI, LAPIS_LAZULI, LAPIS_LAZULI; LAPIS_LAZULI, LAPIS_LAZULI, LAPIS_LAZULI; LAPIS_LAZULI, LAPIS_LAZULI, LAPIS_LAZULI]),
        shaped!(LAPIS_LAZULI, 9, [0, world::LAPIS_BLOCK, 0; 0, 0, 0; 0, 0, 0]),
        shaped!(GOLDEN_APPLE, 1, [GOLD_INGOT, GOLD_INGOT, GOLD_INGOT; GOLD_INGOT, APPLE, GOLD_INGOT; GOLD_INGOT, GOLD_INGOT, GOLD_INGOT]),
        // --- materials ---
        shaped!(world::IRON_BLOCK, 1, [ir, ir, ir; ir, ir, ir; ir, ir, ir]),
        shaped!(world::GOLD_BLOCK, 1, [GOLD_INGOT, GOLD_INGOT, GOLD_INGOT; GOLD_INGOT, GOLD_INGOT, GOLD_INGOT; GOLD_INGOT, GOLD_INGOT, GOLD_INGOT]),
        shaped!(world::DIAMOND_BLOCK, 1, [di, di, di; di, di, di; di, di, di]),
        shaped!(world::WOOL_WHITE, 1, [STRING, STRING, 0; STRING, STRING, 0; 0, 0, 0]),
        shaped!(BREAD, 1, [world::WHEAT, world::WHEAT, world::WHEAT; 0, 0, 0; 0, 0, 0]),
        shaped!(world::HAY, 1, [world::WHEAT, world::WHEAT, world::WHEAT; world::WHEAT, world::WHEAT, world::WHEAT; world::WHEAT, world::WHEAT, world::WHEAT]),
        shaped!(COMPASS, 1, [0, ir, 0; ir, world::REDSTONE_DUST, ir; 0, ir, 0]),
        shaped!(CLOCK, 1, [0, GOLD_INGOT, 0; GOLD_INGOT, world::REDSTONE_DUST, GOLD_INGOT; 0, GOLD_INGOT, 0]),
        shaped!(world::TNT, 1, [GUNPOWDER, world::SAND, GUNPOWDER; world::SAND, GUNPOWDER, world::SAND; GUNPOWDER, world::SAND, GUNPOWDER]),
    ]
}

/// Find a recipe whose pattern matches the 3x3 grid (any offset, 2x2 allowed).
pub fn match_recipe(grid: &[Stack; GRID]) -> Option<Stack> {
    let mut list = Vec::new();
    if let Ok(custom) = CUSTOM_SHAPED.get_or_init(|| RwLock::new(Vec::new())).read() {
        list.extend(custom.iter().cloned());
    }
    list.extend(recipes());
    let mut cells = [(0u16, 0usize); GRID];
    let mut min_x = 3usize; let mut min_y = 3usize; let mut max_x = 0usize; let mut max_y = 0usize;
    let mut any = false;
    let mut flat_items: Vec<u16> = Vec::new();
    for y in 0..3 {
        for x in 0..3 {
            let st = grid[y * 3 + x];
            let id = if st.is_empty() { 0 } else { st.id };
            cells[y * 3 + x] = (id, st.count as usize);
            if id != 0 {
                any = true;
                flat_items.push(id);
                min_x = min_x.min(x); min_y = min_y.min(y);
                max_x = max_x.max(x); max_y = max_y.max(y);
            }
        }
    }
    if !any { return None; }
    let w = max_x - min_x + 1;
    let h = max_y - min_y + 1;
    'outer: for r in list.iter() {
        // bounding box of the recipe
        let mut r_min_x = 3usize; let mut r_min_y = 3usize; let mut r_max_x = 0usize; let mut r_max_y = 0usize;
        for y in 0..3 { for x in 0..3 {
            if r.pattern[y * 3 + x] != 0 {
                r_min_x = r_min_x.min(x); r_min_y = r_min_y.min(y);
                r_max_x = r_max_x.max(x); r_max_y = r_max_y.max(y);
            }
        }}
        if r_max_x < r_min_x || r_max_y < r_min_y { continue; }
        if r_max_x - r_min_x + 1 != w || r_max_y - r_min_y + 1 != h { continue; }
        for y in 0..h { for x in 0..w {
            let want = r.pattern[(r_min_y + y) * 3 + (r_min_x + x)];
            let got = cells[(min_y + y) * 3 + (min_x + x)].0;
            if want != got { continue 'outer; }
        }}
        return Some(Stack::new(r.out, r.count));
    }
    if let Ok(sl) = CUSTOM_SHAPELESS.get_or_init(|| RwLock::new(Vec::new())).read() {
        flat_items.sort_unstable();
        for r in sl.iter() {
            if r.ingredients.len() != flat_items.len() { continue; }
            let mut want = r.ingredients.clone();
            want.sort_unstable();
            if want == flat_items {
                return Some(Stack::new(r.out, r.count));
            }
        }
    }
    None
}

/// Consume one of every non-empty grid cell after a craft.
pub fn consume(grid: &mut [Stack; GRID]) {
    for cell in grid.iter_mut() {
        if cell.count > 1 { cell.count -= 1; } else { *cell = Stack::EMPTY; }
    }
}

pub fn smelt_result(id: u16) -> Option<u16> {
    if let Ok(custom) = CUSTOM_SMELT.get_or_init(|| RwLock::new(Vec::new())).read() {
        if let Some((_, out)) = custom.iter().find(|(inp, _)| *inp == id) {
            return Some(*out);
        }
    }
    use items::*;
    Some(match id {
        world::IRON_ORE => IRON_INGOT,
        world::GOLD_ORE => GOLD_INGOT,
        world::SAND | world::RED_SAND => world::GLASS,
        world::COBBLE => world::STONE,
        world::CLAY => world::BRICK,
        world::REDSTONE_ORE => world::REDSTONE_DUST,
        world::LOG | world::BIRCH_LOG | world::SPRUCE_LOG | world::JUNGLE_LOG | world::DARK_LOG => COAL,
        RAW_PORK => COOKED_PORK,
        RAW_BEEF => STEAK,
        RAW_CHICKEN => COOKED_CHICKEN,
        ROTTEN_FLESH => LEATHER,
        world::CACTUS => world::WOOL_GREEN,
        world::EMERALD_ORE => EMERALD,
        world::LAPIS_ORE => LAPIS_LAZULI,
        world::NETHER_QUARTZ_ORE => NETHER_QUARTZ,
        world::ANCIENT_DEBRIS => NETHERITE_INGOT,
        world::NETHERRACK => world::NETHER_BRICKS,
        _ => return None,
    })
}

/// Seconds of burn time a fuel item provides (0 = not a fuel).
pub fn fuel_time(id: u16) -> f32 {
    match id {
        items::COAL => 80.0,
        items::BLAZE_ROD => 120.0,
        items::LAVA_BUCKET => 1000.0,
        world::PLANKS | world::FENCE | world::CRAFTING | world::BOOKSHELF => 15.0,
        world::LOG | world::BIRCH_LOG | world::SPRUCE_LOG | world::JUNGLE_LOG | world::DARK_LOG => 15.0,
        items::STICK | world::LADDER | world::SIGN => 5.0,
        world::TNT => 20.0,
        _ => 0.0,
    }
}
