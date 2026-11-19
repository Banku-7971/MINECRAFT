//! Item ids and names. Blocks are 1..=world::MAX_BLOCK, items start at 256.
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

pub fn name(id: u16) -> &'static str {
    match id {
        0 => "Air",
        WOOD_PICK => "Wooden Pickaxe", STONE_PICK => "Stone Pickaxe", IRON_PICK => "Iron Pickaxe",
        DIAMOND_PICK => "Diamond Pickaxe", NETHERITE_PICK => "Netherite Pickaxe",
        WOOD_AXE => "Wooden Axe", IRON_AXE => "Iron Axe", DIAMOND_AXE => "Diamond Axe",
        WOOD_SHOVEL => "Wooden Shovel", IRON_SHOVEL => "Iron Shovel",
        WOOD_SWORD => "Wooden Sword", IRON_SWORD => "Iron Sword", DIAMOND_SWORD => "Diamond Sword",
        APPLE => "Apple", BREAD => "Bread", COOKED_PORK => "Cooked Porkchop", RAW_PORK => "Raw Porkchop",
        RAW_BEEF => "Raw Beef", STEAK => "Steak", RAW_CHICKEN => "Raw Chicken",
        COOKED_CHICKEN => "Cooked Chicken", FISH => "Fish", COAL => "Coal", IRON_INGOT => "Iron Ingot",
        GOLD_INGOT => "Gold Ingot", DIAMOND => "Diamond", EMERALD => "Emerald", STICK => "Stick",
        FLINT => "Flint", FEATHER => "Feather", LEATHER => "Leather", BONE => "Bone",
        ROTTEN_FLESH => "Rotten Flesh", STRING => "String", WOOL => "Wool", EGG => "Egg",
        BUCKET => "Bucket", WATER_BUCKET => "Water Bucket", LAVA_BUCKET => "Lava Bucket",
        COMPASS => "Compass", MAP => "Map", CLOCK => "Clock", ELYTRA => "Elytra",
        FIREWORK => "Firework Rocket", SHIELD => "Shield", TOTEM => "Totem of Undying",
        ENDER_PEARL => "Ender Pearl", BLAZE_ROD => "Blaze Rod", NETHER_STAR => "Nether Star",
        XP_BOTTLE => "Bottle o' Enchanting",
        _ => world::block_name(id),
    }
}

/// Tool mining speed multiplier and tier (0 = hand).
pub fn tool_tier(id: u16) -> u8 {
    match id {
        WOOD_PICK | WOOD_AXE | WOOD_SHOVEL | WOOD_SWORD => 1,
        STONE_PICK => 2,
        IRON_PICK | IRON_AXE | IRON_SHOVEL | IRON_SWORD => 3,
        DIAMOND_PICK | DIAMOND_AXE | DIAMOND_SWORD => 4,
        NETHERITE_PICK => 5,
        _ => 0,
    }
}

pub fn pickaxe_tier(id: u16) -> u8 {
    match id { WOOD_PICK => 1, STONE_PICK => 2, IRON_PICK => 3, DIAMOND_PICK => 4, NETHERITE_PICK => 5, _ => 0 }
}

pub fn is_food(id: u16) -> f32 {
    match id {
        APPLE => 4.0, BREAD => 5.0, COOKED_PORK => 8.0, STEAK => 8.0, COOKED_CHICKEN => 6.0,
        FISH => 3.0, RAW_PORK | RAW_BEEF => 2.0, RAW_CHICKEN => 1.5, ROTTEN_FLESH => 1.0,
        _ => 0.0,
    }
}
