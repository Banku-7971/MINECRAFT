//! Item ids, names, categories, tool stats, and food values.
//! Blocks are 1..=world::MAX_BLOCK, items start at 256.
use crate::world;

pub const WOOD_PICK: u16 = 256;
pub const STONE_PICK: u16 = 257;
pub const IRON_PICK: u16 = 258;
pub const DIAMOND_PICK: u16 = 259;
pub const NETHERITE_PICK: u16 = 260;
pub const WOOD_AXE: u16 = 261;
pub const IRON_AXE: u16 = 262;
pub const DIAMOND_AXE: u16 = 263;
pub const WOOD_SHOVEL: u16 = 264;
pub const IRON_SHOVEL: u16 = 265;
pub const WOOD_SWORD: u16 = 266;
pub const IRON_SWORD: u16 = 267;
pub const DIAMOND_SWORD: u16 = 268;
pub const APPLE: u16 = 269;
pub const BREAD: u16 = 270;
pub const COOKED_PORK: u16 = 271;
pub const RAW_PORK: u16 = 272;
pub const RAW_BEEF: u16 = 273;
pub const STEAK: u16 = 274;
pub const RAW_CHICKEN: u16 = 275;
pub const COOKED_CHICKEN: u16 = 276;
pub const FISH: u16 = 277;
pub const COAL: u16 = 278;
pub const IRON_INGOT: u16 = 279;
pub const GOLD_INGOT: u16 = 280;
pub const DIAMOND: u16 = 281;
pub const EMERALD: u16 = 282;
pub const STICK: u16 = 283;
pub const FLINT: u16 = 284;
pub const FEATHER: u16 = 285;
pub const LEATHER: u16 = 286;
pub const BONE: u16 = 287;
pub const ROTTEN_FLESH: u16 = 288;
pub const STRING: u16 = 289;
pub const WOOL: u16 = 290;
pub const EGG: u16 = 291;
pub const BUCKET: u16 = 292;
pub const WATER_BUCKET: u16 = 293;
pub const LAVA_BUCKET: u16 = 294;
pub const COMPASS: u16 = 295;
pub const MAP: u16 = 296;
pub const CLOCK: u16 = 297;
pub const ELYTRA: u16 = 298;
pub const FIREWORK: u16 = 299;
pub const SHIELD: u16 = 300;
pub const TOTEM: u16 = 301;
pub const ENDER_PEARL: u16 = 302;
pub const BLAZE_ROD: u16 = 303;
pub const NETHER_STAR: u16 = 304;
pub const XP_BOTTLE: u16 = 305;
pub const GUNPOWDER: u16 = 306;
pub const GOLDEN_APPLE: u16 = 307;
pub const BOW: u16 = 308;
pub const ARROW: u16 = 309;
pub const FLINT_AND_STEEL: u16 = 310;
pub const SHEARS: u16 = 311;
pub const NETHERITE_SWORD: u16 = 312;
pub const NETHERITE_INGOT: u16 = 313;
pub const CARROT: u16 = 314;
pub const COOKIE: u16 = 315;
pub const SLIMEBALL: u16 = 316;
pub const AMETHYST: u16 = 317;
pub const IRON_HELMET: u16 = 318;
pub const IRON_CHESTPLATE: u16 = 319;
pub const DIAMOND_HELMET: u16 = 320;
pub const DIAMOND_CHESTPLATE: u16 = 321;
pub const SPAWN_SHEEP: u16 = 322;
pub const SPAWN_COW: u16 = 323;
pub const SPAWN_PIG: u16 = 324;
pub const SPAWN_CHICKEN: u16 = 325;
pub const SPAWN_ZOMBIE: u16 = 326;
pub const SPAWN_SKELETON: u16 = 327;
pub const SPAWN_CREEPER: u16 = 328;
pub const SPAWN_SPIDER: u16 = 329;
pub const SPAWN_ENDERMAN: u16 = 330;
pub const SPAWN_VILLAGER: u16 = 331;
pub const EYE_OF_ENDER: u16 = 332;
pub const BLAZE_POWDER: u16 = 333;
pub const NETHER_QUARTZ: u16 = 334;
pub const GOLD_NUGGET: u16 = 335;
pub const SPAWN_ZOMBIFIED_PIGLIN: u16 = 336;
pub const SPAWN_PIGLIN: u16 = 337;
pub const SPAWN_GHAST: u16 = 338;
pub const SPAWN_BLAZE: u16 = 339;
pub const SPAWN_WITHER_SKELETON: u16 = 340;
pub const SPAWN_SLIME: u16 = 341;
pub const SPAWN_WOLF: u16 = 342;
pub const SPAWN_POLAR_BEAR: u16 = 343;
pub const SPAWN_PARROT: u16 = 344;
pub const SPAWN_ENDER_DRAGON: u16 = 345;
pub const LAPIS_LAZULI: u16 = 346;
pub const BONE_MEAL: u16 = 347;
pub const WOOD_HOE: u16 = 348;
pub const STONE_HOE: u16 = 349;
pub const IRON_HOE: u16 = 350;
pub const DIAMOND_HOE: u16 = 351;
pub const BOAT: u16 = 352;
pub const MINECART: u16 = 353;
pub const GLASS_BOTTLE: u16 = 354;
pub const POTION_HEAL: u16 = 355;
pub const POTION_FIRE: u16 = 356;
pub const POTION_SPEED: u16 = 357;
pub const IRON_LEGGINGS: u16 = 358;
pub const IRON_BOOTS: u16 = 359;
pub const DIAMOND_LEGGINGS: u16 = 360;
pub const DIAMOND_BOOTS: u16 = 361;
pub const NETHERITE_HELMET: u16 = 362;
pub const NETHERITE_CHESTPLATE: u16 = 363;
pub const NETHERITE_LEGGINGS: u16 = 364;
pub const NETHERITE_BOOTS: u16 = 365;
pub const ENCHANTED_BOOK: u16 = 366;
pub const SPAWN_IRON_GOLEM: u16 = 367;
pub const SPAWN_SNOW_GOLEM: u16 = 368;
pub const SPAWN_WITHER: u16 = 369;
pub const SPAWN_SQUID: u16 = 370;
pub const SPAWN_FOX: u16 = 371;
pub const SPAWN_BEE: u16 = 372;
pub const SPAWN_PHANTOM: u16 = 373;
pub const MUSIC_DISC_CAT: u16 = 374;
pub const MUSIC_DISC_PIGSTEP: u16 = 375;
pub const SPAWN_PANDA: u16 = 376;
pub const SPAWN_ALLAY: u16 = 377;
pub const SPAWN_WANDERING_TRADER: u16 = 378;
pub const SPAWN_GUARDIAN: u16 = 379;
pub const SPAWN_WARDEN: u16 = 380;

pub fn name(id: u16) -> &'static str {
    match id {
        0 => "Air",
        WOOD_PICK => "Wooden Pickaxe", STONE_PICK => "Stone Pickaxe", IRON_PICK => "Iron Pickaxe",
        DIAMOND_PICK => "Diamond Pickaxe", NETHERITE_PICK => "Netherite Pickaxe",
        WOOD_AXE => "Wooden Axe", IRON_AXE => "Iron Axe", DIAMOND_AXE => "Diamond Axe",
        WOOD_SHOVEL => "Wooden Shovel", IRON_SHOVEL => "Iron Shovel",
        WOOD_SWORD => "Wooden Sword", IRON_SWORD => "Iron Sword", DIAMOND_SWORD => "Diamond Sword",
        NETHERITE_SWORD => "Netherite Sword",
        APPLE => "Apple", GOLDEN_APPLE => "Golden Apple", BREAD => "Bread",
        COOKED_PORK => "Cooked Porkchop", RAW_PORK => "Raw Porkchop",
        RAW_BEEF => "Raw Beef", STEAK => "Steak", RAW_CHICKEN => "Raw Chicken",
        COOKED_CHICKEN => "Cooked Chicken", FISH => "Fish", CARROT => "Carrot", COOKIE => "Cookie",
        COAL => "Coal", IRON_INGOT => "Iron Ingot", GOLD_INGOT => "Gold Ingot",
        DIAMOND => "Diamond", EMERALD => "Emerald", NETHERITE_INGOT => "Netherite Ingot",
        AMETHYST => "Amethyst Shard", STICK => "Stick", FLINT => "Flint", FEATHER => "Feather",
        LEATHER => "Leather", BONE => "Bone", ROTTEN_FLESH => "Rotten Flesh", STRING => "String",
        WOOL => "Wool", EGG => "Egg", SLIMEBALL => "Slimeball",
        BUCKET => "Bucket", WATER_BUCKET => "Water Bucket", LAVA_BUCKET => "Lava Bucket",
        COMPASS => "Compass", MAP => "Map", CLOCK => "Clock", ELYTRA => "Elytra",
        FIREWORK => "Firework Rocket", SHIELD => "Shield", TOTEM => "Totem of Undying",
        ENDER_PEARL => "Ender Pearl", BLAZE_ROD => "Blaze Rod", NETHER_STAR => "Nether Star",
        XP_BOTTLE => "Bottle o' Enchanting", GUNPOWDER => "Gunpowder",
        BOW => "Bow", ARROW => "Arrow", FLINT_AND_STEEL => "Flint and Steel", SHEARS => "Shears",
        IRON_HELMET => "Iron Helmet", IRON_CHESTPLATE => "Iron Chestplate",
        DIAMOND_HELMET => "Diamond Helmet", DIAMOND_CHESTPLATE => "Diamond Chestplate",
        SPAWN_SHEEP => "Sheep Spawn Egg", SPAWN_COW => "Cow Spawn Egg",
        SPAWN_PIG => "Pig Spawn Egg", SPAWN_CHICKEN => "Chicken Spawn Egg",
        SPAWN_ZOMBIE => "Zombie Spawn Egg", SPAWN_SKELETON => "Skeleton Spawn Egg",
        SPAWN_CREEPER => "Creeper Spawn Egg", SPAWN_SPIDER => "Spider Spawn Egg",
        SPAWN_ENDERMAN => "Enderman Spawn Egg", SPAWN_VILLAGER => "Villager Spawn Egg",
        EYE_OF_ENDER => "Eye of Ender", BLAZE_POWDER => "Blaze Powder",
        NETHER_QUARTZ => "Nether Quartz", GOLD_NUGGET => "Gold Nugget",
        SPAWN_ZOMBIFIED_PIGLIN => "Zombified Piglin Spawn Egg",
        SPAWN_PIGLIN => "Piglin Spawn Egg",
        SPAWN_GHAST => "Ghast Spawn Egg",
        SPAWN_BLAZE => "Blaze Spawn Egg",
        SPAWN_WITHER_SKELETON => "Wither Skeleton Spawn Egg",
        SPAWN_SLIME => "Slime Spawn Egg",
        SPAWN_WOLF => "Wolf Spawn Egg",
        SPAWN_POLAR_BEAR => "Polar Bear Spawn Egg",
        SPAWN_PARROT => "Parrot Spawn Egg",
        SPAWN_ENDER_DRAGON => "Ender Dragon Spawn Egg",
        LAPIS_LAZULI => "Lapis Lazuli", BONE_MEAL => "Bone Meal",
        WOOD_HOE => "Wooden Hoe", STONE_HOE => "Stone Hoe", IRON_HOE => "Iron Hoe", DIAMOND_HOE => "Diamond Hoe",
        BOAT => "Oak Boat", MINECART => "Minecart",
        GLASS_BOTTLE => "Glass Bottle",
        POTION_HEAL => "Potion of Healing", POTION_FIRE => "Potion of Fire Resistance", POTION_SPEED => "Potion of Swiftness",
        IRON_LEGGINGS => "Iron Leggings", IRON_BOOTS => "Iron Boots",
        DIAMOND_LEGGINGS => "Diamond Leggings", DIAMOND_BOOTS => "Diamond Boots",
        NETHERITE_HELMET => "Netherite Helmet", NETHERITE_CHESTPLATE => "Netherite Chestplate",
        NETHERITE_LEGGINGS => "Netherite Leggings", NETHERITE_BOOTS => "Netherite Boots",
        ENCHANTED_BOOK => "Enchanted Book",
        SPAWN_IRON_GOLEM => "Iron Golem Spawn Egg",
        SPAWN_SNOW_GOLEM => "Snow Golem Spawn Egg",
        SPAWN_WITHER => "Wither Spawn Egg",
        SPAWN_SQUID => "Squid Spawn Egg",
        SPAWN_FOX => "Fox Spawn Egg",
        SPAWN_BEE => "Bee Spawn Egg",
        SPAWN_PHANTOM => "Phantom Spawn Egg",
        SPAWN_PANDA => "Panda Spawn Egg",
        SPAWN_ALLAY => "Allay Spawn Egg",
        SPAWN_WANDERING_TRADER => "Wandering Trader Spawn Egg",
        SPAWN_GUARDIAN => "Guardian Spawn Egg",
        SPAWN_WARDEN => "Warden Spawn Egg",
        MUSIC_DISC_CAT => "Music Disc (Cat)",
        MUSIC_DISC_PIGSTEP => "Music Disc (Pigstep)",
        _ => world::block_name(id),
    }
}

/// Armor slot index (0=helmet, 1=chestplate, 2=leggings, 3=boots).
pub fn armor_slot(id: u16) -> Option<usize> {
    match id {
        IRON_HELMET | DIAMOND_HELMET | NETHERITE_HELMET => Some(0),
        IRON_CHESTPLATE | DIAMOND_CHESTPLATE | NETHERITE_CHESTPLATE | ELYTRA => Some(1),
        IRON_LEGGINGS | DIAMOND_LEGGINGS | NETHERITE_LEGGINGS => Some(2),
        IRON_BOOTS | DIAMOND_BOOTS | NETHERITE_BOOTS => Some(3),
        _ => None,
    }
}

/// Armor defense points (max 20 = 10 full armor icons).
pub fn armor_points(id: u16) -> u32 {
    match id {
        IRON_HELMET | IRON_BOOTS => 2,
        IRON_LEGGINGS => 5,
        IRON_CHESTPLATE => 6,
        DIAMOND_HELMET | DIAMOND_BOOTS | NETHERITE_HELMET | NETHERITE_BOOTS => 3,
        DIAMOND_LEGGINGS | NETHERITE_LEGGINGS => 6,
        DIAMOND_CHESTPLATE | NETHERITE_CHESTPLATE => 8,
        _ => 0,
    }
}

pub fn is_hoe(id: u16) -> bool {
    matches!(id, WOOD_HOE..=DIAMOND_HOE)
}

/// Tool mining speed multiplier and tier (0 = hand).
pub fn tool_tier(id: u16) -> u8 {
    match id {
        WOOD_PICK | WOOD_AXE | WOOD_SHOVEL | WOOD_SWORD => 1,
        STONE_PICK => 2,
        IRON_PICK | IRON_AXE | IRON_SHOVEL | IRON_SWORD => 3,
        DIAMOND_PICK | DIAMOND_AXE | DIAMOND_SWORD => 4,
        NETHERITE_PICK | NETHERITE_SWORD => 5,
        _ => 0,
    }
}

pub fn pickaxe_tier(id: u16) -> u8 {
    match id { WOOD_PICK => 1, STONE_PICK => 2, IRON_PICK => 3, DIAMOND_PICK => 4, NETHERITE_PICK => 5, _ => 0 }
}

pub fn is_food(id: u16) -> f32 {
    match id {
        GOLDEN_APPLE => 10.0,
        APPLE => 4.0, BREAD => 5.0, COOKED_PORK => 8.0, STEAK => 8.0, COOKED_CHICKEN => 6.0,
        FISH => 3.0, CARROT => 3.0, COOKIE => 2.0,
        RAW_PORK | RAW_BEEF => 2.0, RAW_CHICKEN => 1.5, ROTTEN_FLESH => 1.0,
        _ => 0.0,
    }
}

pub fn max_stack(id: u16) -> u32 {
    if matches!(
        id,
        WOOD_PICK..=DIAMOND_SWORD | NETHERITE_SWORD | BUCKET | WATER_BUCKET | LAVA_BUCKET
            | COMPASS | MAP | CLOCK | ELYTRA | SHIELD | TOTEM | BOW | FLINT_AND_STEEL | SHEARS
            | IRON_HELMET..=DIAMOND_CHESTPLATE | WOOD_HOE..=MINECART | POTION_HEAL..=ENCHANTED_BOOK
    ) {
        1
    } else {
        64
    }
}

/// Creative inventory categories (mirrors Minecraft Java 1.20 Creative tabs).
pub fn creative_tab_items(tab: usize) -> &'static [u16] {
    use world::*;
    match tab {
        0 => &[ // Building Blocks
            STONE, COBBLE, MOSSY, STONE_BRICKS, BRICK, QUARTZ, BLACKSTONE, BASALT, NETHER_BRICKS, PURPUR_BLOCK, END_STONE, NETHERRACK,
            LOG, BIRCH_LOG, SPRUCE_LOG, JUNGLE_LOG, DARK_LOG, CRIMSON_STEM, WARPED_STEM, PLANKS, SLAB, STAIRS, FENCE, DOOR, IRON_DOOR,
            GLASS, SANDSTONE, IRON_BLOCK, GOLD_BLOCK, DIAMOND_BLOCK, EMERALD_BLOCK, LAPIS_BLOCK, OBSIDIAN, CRYING_OBSIDIAN, BOOKSHELF,
        ],
        1 => &[ // Colored Blocks
            WOOL_WHITE, WOOL_RED, WOOL_BLUE, WOOL_GREEN, WOOL_YELLOW,
            CONCRETE_WHITE, CONCRETE_BLUE, CONCRETE_RED, CONCRETE_GREEN, CONCRETE_YELLOW, CONCRETE_BLACK,
            TERRACOTTA, PURPUR_BLOCK, BED,
        ],
        2 => &[ // Natural Blocks
            GRASS, DIRT, PODZOL, MYCELIUM, CRIMSON_NYLIUM, WARPED_NYLIUM, FARMLAND, SAND, RED_SAND, GRAVEL, CLAY, SNOW, SNOW_LAYER, ICE,
            LEAVES, BIRCH_LEAVES, SPRUCE_LEAVES, JUNGLE_LEAVES, DARK_LEAVES, CRIMSON_WART, WARPED_WART, SAPLING, NETHER_WART,
            COAL_ORE, IRON_ORE, GOLD_ORE, DIAMOND_ORE, REDSTONE_ORE, LAPIS_ORE, EMERALD_ORE, NETHER_QUARTZ_ORE, ANCIENT_DEBRIS,
            CACTUS, TALL_GRASS, FLOWER_RED, FLOWER_YELLOW, BROWN_MUSHROOM, RED_MUSHROOM, CHORUS_PLANT, CHORUS_FLOWER,
            PUMPKIN, HAY, WHEAT, VINE, COBWEB, MAGMA, SOUL_SAND, BEDROCK, WATER, LAVA, SPAWNER,
        ],
        3 => &[ // Functional Blocks
            CRAFTING, FURNACE, CHEST, ANVIL, ENCHANTING, BREWING_STAND, BEACON, JUKEBOX, END_PORTAL_FRAME,
            TORCH, SOUL_TORCH, LANTERN, CAMPFIRE, GLOWSTONE, SEA_LANTERN, SHROOMLIGHT,
            LADDER, SIGN, BED, CAKE, BOOKSHELF,
        ],
        4 => &[ // Redstone Blocks
            REDSTONE_DUST, REDSTONE_TORCH, LEVER, BUTTON, PRESSURE_PLATE, REPEATER, COMPARATOR,
            PISTON, STICKY_PISTON, HOPPER, LAMP, RAIL, TNT, DOOR, IRON_DOOR,
        ],
        6 => &[ // Tools & Utilities
            WOOD_PICK, STONE_PICK, IRON_PICK, DIAMOND_PICK, NETHERITE_PICK,
            WOOD_AXE, IRON_AXE, DIAMOND_AXE, WOOD_SHOVEL, IRON_SHOVEL,
            WOOD_HOE, STONE_HOE, IRON_HOE, DIAMOND_HOE,
            FLINT_AND_STEEL, SHEARS, BUCKET, WATER_BUCKET, LAVA_BUCKET, BOAT, MINECART,
            EYE_OF_ENDER, COMPASS, CLOCK, MAP, ELYTRA, FIREWORK, XP_BOTTLE, BONE_MEAL,
            MUSIC_DISC_CAT, MUSIC_DISC_PIGSTEP,
        ],
        7 => &[ // Combat
            WOOD_SWORD, IRON_SWORD, DIAMOND_SWORD, NETHERITE_SWORD,
            BOW, ARROW, SHIELD, TOTEM,
            IRON_HELMET, IRON_CHESTPLATE, IRON_LEGGINGS, IRON_BOOTS,
            DIAMOND_HELMET, DIAMOND_CHESTPLATE, DIAMOND_LEGGINGS, DIAMOND_BOOTS,
            NETHERITE_HELMET, NETHERITE_CHESTPLATE, NETHERITE_LEGGINGS, NETHERITE_BOOTS,
            TNT,
        ],
        8 => &[ // Food & Drinks
            APPLE, GOLDEN_APPLE, BREAD, STEAK, COOKED_PORK, COOKED_CHICKEN,
            RAW_BEEF, RAW_PORK, RAW_CHICKEN, FISH, CARROT, COOKIE, CAKE, ROTTEN_FLESH,
            GLASS_BOTTLE, POTION_HEAL, POTION_FIRE, POTION_SPEED,
        ],
        9 => &[ // Ingredients
            COAL, IRON_INGOT, GOLD_INGOT, GOLD_NUGGET, DIAMOND, EMERALD, LAPIS_LAZULI, NETHERITE_INGOT, NETHER_QUARTZ, AMETHYST,
            STICK, FLINT, FEATHER, LEATHER, BONE, BONE_MEAL, STRING, GUNPOWDER, SLIMEBALL,
            ENDER_PEARL, BLAZE_ROD, BLAZE_POWDER, EYE_OF_ENDER, NETHER_STAR, NETHER_WART, EGG, WOOL, REDSTONE_DUST, ENCHANTED_BOOK,
        ],
        10 => &[ // Spawn Eggs
            SPAWN_SHEEP, SPAWN_COW, SPAWN_PIG, SPAWN_CHICKEN, SPAWN_WOLF, SPAWN_FOX, SPAWN_BEE, SPAWN_PANDA, SPAWN_ALLAY, SPAWN_SQUID, SPAWN_POLAR_BEAR, SPAWN_PARROT, SPAWN_SLIME,
            SPAWN_ZOMBIE, SPAWN_SKELETON, SPAWN_CREEPER, SPAWN_SPIDER, SPAWN_PHANTOM, SPAWN_GUARDIAN, SPAWN_ENDERMAN, SPAWN_VILLAGER, SPAWN_WANDERING_TRADER,
            SPAWN_ZOMBIFIED_PIGLIN, SPAWN_PIGLIN, SPAWN_GHAST, SPAWN_BLAZE, SPAWN_WITHER_SKELETON, SPAWN_WITHER, SPAWN_WARDEN, SPAWN_ENDER_DRAGON,
            SPAWN_IRON_GOLEM, SPAWN_SNOW_GOLEM,
        ],
        _ => &[],
    }
}

pub fn all_creative_items() -> Vec<u16> {
    let mut out = Vec::new();
    for tab in [0, 1, 2, 3, 4, 6, 7, 8, 9, 10] {
        for &id in creative_tab_items(tab) {
            if !out.contains(&id) { out.push(id); }
        }
    }
    out
}
