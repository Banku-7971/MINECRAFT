use super::*;
use crate::entities::MobKind;
use crate::items;
use crate::packs;
use crate::sound::Sfx;
use crate::world::{self, Dimension};

const ALL_COMMANDS: &[&str] = &[
    "/help", "/gamemode", "/gm", "/tp", "/teleport", "/give", "/clear",
    "/time", "/weather", "/summon", "/kill", "/setblock", "/fill", "/clone",
    "/effect", "/enchant", "/xp", "/experience", "/seed", "/spawnpoint",
    "/setworldspawn", "/difficulty", "/gamerule", "/attribute", "/locate",
    "/dim", "/dimension", "/execute", "/playsound", "/stopsound", "/particle",
    "/title", "/say", "/me", "/msg", "/tell", "/w", "/list",
    "/heal", "/feed", "/fly", "/speed", "/home", "/spawn",
    "/shader", "/packs", "/reloadpacks",
];

fn normalize_key(s: &str) -> String {
    let stripped = s.strip_prefix("minecraft:").unwrap_or(s);
    stripped
        .to_ascii_lowercase()
        .replace([' ', '-', '\''], "_")
}

pub fn resolve_item_or_block(raw: &str) -> Option<u16> {
    if let Ok(id) = raw.parse::<u16>() {
        if id <= 366 {
            return Some(id);
        }
    }
    let q = normalize_key(raw);
    match q.as_str() {
        "air" => return Some(world::AIR),
        "stone" => return Some(world::STONE),
        "grass" | "grass_block" => return Some(world::GRASS),
        "dirt" => return Some(world::DIRT),
        "cobble" | "cobblestone" => return Some(world::COBBLE),
        "oak_log" | "log" | "wood" => return Some(world::LOG),
        "oak_leaves" | "leaves" => return Some(world::LEAVES),
        "oak_planks" | "planks" => return Some(world::PLANKS),
        "red_flower" | "poppy" | "rose" => return Some(world::FLOWER_RED),
        "yellow_flower" | "dandelion" => return Some(world::FLOWER_YELLOW),
        "crafting_table" | "workbench" => return Some(world::CRAFTING),
        "enchanting_table" | "enchantment_table" => return Some(world::ENCHANTING),
        "wooden_pickaxe" | "wood_pickaxe" | "wood_pick" => return Some(items::WOOD_PICK),
        "stone_pickaxe" | "stone_pick" => return Some(items::STONE_PICK),
        "iron_pickaxe" | "iron_pick" => return Some(items::IRON_PICK),
        "diamond_pickaxe" | "diamond_pick" => return Some(items::DIAMOND_PICK),
        "netherite_pickaxe" | "netherite_pick" => return Some(items::NETHERITE_PICK),
        "wooden_axe" | "wood_axe" => return Some(items::WOOD_AXE),
        "wooden_shovel" | "wood_shovel" => return Some(items::WOOD_SHOVEL),
        "wooden_sword" | "wood_sword" => return Some(items::WOOD_SWORD),
        "wooden_hoe" | "wood_hoe" => return Some(items::WOOD_HOE),
        "cooked_beef" | "steak" => return Some(items::STEAK),
        "cooked_porkchop" => return Some(items::COOKED_PORK),
        "porkchop" | "raw_porkchop" => return Some(items::RAW_PORK),
        "beef" | "raw_beef" => return Some(items::RAW_BEEF),
        "chicken" | "raw_chicken" => return Some(items::RAW_CHICKEN),
        "cooked_chicken" => return Some(items::COOKED_CHICKEN),
        "cod" | "fish" => return Some(items::FISH),
        "ender_eye" | "eye_of_ender" => return Some(items::EYE_OF_ENDER),
        "experience_bottle" | "xp_bottle" => return Some(items::XP_BOTTLE),
        "firework_rocket" | "firework" => return Some(items::FIREWORK),
        "totem_of_undying" | "totem" => return Some(items::TOTEM),
        "oak_boat" | "boat" => return Some(items::BOAT),
        "tnt" => return Some(world::TNT),
        "oak_door" | "door" => return Some(world::DOOR),
        "oak_sapling" | "sapling" => return Some(world::SAPLING),
        "oak_fence" | "fence" => return Some(world::FENCE),
        "oak_slab" | "slab" => return Some(world::SLAB),
        "oak_stairs" | "stairs" => return Some(world::STAIRS),
        "oak_sign" | "sign" => return Some(world::SIGN),
        "redstone" | "redstone_dust" => return Some(world::REDSTONE_DUST),
        "redstone_lamp" | "lamp" => return Some(world::LAMP),
        "quartz_block" | "quartz" => return Some(world::QUARTZ),
        "bricks" | "brick" => return Some(world::BRICK),
        "mossy_cobblestone" | "mossy" => return Some(world::MOSSY),
        "hay_block" | "hay" | "hay_bale" => return Some(world::HAY),
        "white_wool" | "wool" => return Some(world::WOOL_WHITE),
        "red_wool" => return Some(world::WOOL_RED),
        "blue_wool" => return Some(world::WOOL_BLUE),
        "green_wool" => return Some(world::WOOL_GREEN),
        "yellow_wool" => return Some(world::WOOL_YELLOW),
        "potion" | "healing_potion" | "potion_of_healing" => return Some(items::POTION_HEAL),
        "fire_resistance_potion" => return Some(items::POTION_FIRE),
        "swiftness_potion" | "speed_potion" => return Some(items::POTION_SPEED),
        _ => {}
    }
    for id in 1..=366u16 {
        let nm = normalize_key(items::name(id));
        if nm == q {
            return Some(id);
        }
    }
    for id in 1..=366u16 {
        let nm = normalize_key(items::name(id));
        if nm.contains(&q) {
            return Some(id);
        }
    }
    None
}

pub fn resolve_block(raw: &str) -> Option<u16> {
    let id = resolve_item_or_block(raw)?;
    if id <= world::MAX_BLOCK {
        Some(id)
    } else {
        None
    }
}

pub fn resolve_mob(raw: &str) -> Option<MobKind> {
    let q = normalize_key(raw);
    match q.as_str() {
        "sheep" => Some(MobKind::Sheep),
        "cow" => Some(MobKind::Cow),
        "pig" => Some(MobKind::Pig),
        "chicken" => Some(MobKind::Chicken),
        "zombie" => Some(MobKind::Zombie),
        "skeleton" => Some(MobKind::Skeleton),
        "creeper" => Some(MobKind::Creeper),
        "spider" => Some(MobKind::Spider),
        "enderman" => Some(MobKind::Enderman),
        "villager" => Some(MobKind::Villager),
        "zombified_piglin" | "zombie_pigman" | "pigman" => Some(MobKind::ZombifiedPiglin),
        "piglin" => Some(MobKind::Piglin),
        "ghast" => Some(MobKind::Ghast),
        "blaze" => Some(MobKind::Blaze),
        "wither_skeleton" => Some(MobKind::WitherSkeleton),
        "slime" => Some(MobKind::Slime),
        "wolf" | "dog" => Some(MobKind::Wolf),
        "polar_bear" => Some(MobKind::PolarBear),
        "parrot" => Some(MobKind::Parrot),
        "ender_dragon" | "dragon" => Some(MobKind::EnderDragon),
        _ => None,
    }
}

fn parse_rel_f32(tok: &str, base: f32) -> Option<f32> {
    if let Some(rest) = tok.strip_prefix('~') {
        if rest.is_empty() {
            Some(base)
        } else {
            rest.parse::<f32>().ok().map(|d| base + d)
        }
    } else {
        tok.parse::<f32>().ok()
    }
}

fn parse_rel_i32(tok: &str, base: f32) -> Option<i32> {
    parse_rel_f32(tok, base).map(|v| v.floor() as i32)
}

fn is_player_selector(tok: &str) -> bool {
    matches!(tok, "@s" | "@p" | "@a" | "@r" | "Player" | "player")
}

pub fn autocomplete(input: &str) -> Option<String> {
    if !input.starts_with('/') {
        return None;
    }
    let ends_space = input.ends_with(' ');
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() == 1 && !ends_space {
        let prefix = parts[0].to_ascii_lowercase();
        for &cmd in ALL_COMMANDS {
            if cmd.starts_with(&prefix) && cmd != prefix {
                return Some(format!("{} ", cmd));
            }
        }
        return None;
    }
    let cmd = parts[0].to_ascii_lowercase();
    let last = if ends_space { "" } else { *parts.last().unwrap_or(&"") };
    let ql = normalize_key(last);

    let candidate: Option<&str> = match cmd.as_str() {
        "/gamemode" | "/gm" => ["survival", "creative", "spectator"]
            .into_iter()
            .find(|s| s.starts_with(&ql)),
        "/weather" => ["clear", "rain", "thunder"]
            .into_iter()
            .find(|s| s.starts_with(&ql)),
        "/time" if parts.len() <= 2 => ["set", "add", "query"]
            .into_iter()
            .find(|s| s.starts_with(&ql)),
        "/time" => ["day", "noon", "sunset", "night", "midnight"]
            .into_iter()
            .find(|s| s.starts_with(&ql)),
        "/difficulty" => ["peaceful", "easy", "normal", "hard"]
            .into_iter()
            .find(|s| s.starts_with(&ql)),
        "/dim" | "/dimension" => ["overworld", "nether", "end"]
            .into_iter()
            .find(|s| s.starts_with(&ql)),
        "/gamerule" => [
            "keepInventory",
            "doDaylightCycle",
            "doMobSpawning",
            "mobGriefing",
            "fallDamage",
            "drowningDamage",
            "doWeatherCycle",
            "pvp",
        ]
        .into_iter()
        .find(|s| s.to_ascii_lowercase().starts_with(&ql)),
        "/summon" => [
            "cow", "sheep", "pig", "chicken", "wolf", "villager",
            "zombie", "skeleton", "creeper", "spider", "enderman",
            "slime", "blaze", "ghast", "wither_skeleton", "piglin",
            "zombified_piglin", "polar_bear", "parrot", "ender_dragon", "tnt",
        ]
        .into_iter()
        .find(|s| s.starts_with(&ql)),
        "/effect" if parts.len() <= 2 => ["give", "clear"]
            .into_iter()
            .find(|s| s.starts_with(&ql)),
        "/effect" => [
            "speed", "fire_resistance", "regeneration", "instant_health",
            "saturation", "jump_boost", "strength", "haste", "night_vision",
        ]
        .into_iter()
        .find(|s| s.starts_with(&ql)),
        "/enchant" => ["sharpness", "efficiency", "unbreaking", "fortune", "protection"]
            .into_iter()
            .find(|s| s.starts_with(&ql)),
        "/attribute" => [
            "block_interaction_range",
            "entity_interaction_range",
            "movement_speed",
            "max_health",
        ]
        .into_iter()
        .find(|s| s.starts_with(&ql)),
        "/give" | "/setblock" | "/fill" => {
            if parts.len() >= 2 && !ql.is_empty() {
                for id in 1..=366u16 {
                    let nm = normalize_key(items::name(id));
                    if nm.starts_with(&ql) {
                        let base = if ends_space {
                            input.to_string()
                        } else {
                            let idx = input.rfind(last).unwrap_or(input.len());
                            input[..idx].to_string()
                        };
                        return Some(format!("{}{} ", base, nm));
                    }
                }
            }
            None
        }
        _ => None,
    };

    if let Some(c) = candidate {
        let base = if ends_space {
            input.to_string()
        } else {
            let idx = input.rfind(last).unwrap_or(input.len());
            input[..idx].to_string()
        };
        return Some(format!("{}{} ", base, c));
    }
    None
}

pub fn execute<P: Platform>(game: &mut Game<P>, cmd: &str) -> bool {
    let trim = cmd.trim();
    if !trim.starts_with('/') {
        return false;
    }
    let parts: Vec<&str> = trim.split_whitespace().collect();
    let head = parts.first().copied().unwrap_or("").to_ascii_lowercase();
    match head.as_str() {
        "/help" | "/?" => {
            let pages: &[&str] = &[
                "1/4: /gamemode /tp /give /clear /time /weather /summon /kill",
                "2/4: /setblock /fill /clone /effect /enchant /xp /seed /dim",
                "3/4: /gamerule /difficulty /attribute /locate /spawnpoint /execute",
                "4/4: /title /say /me /msg /list /heal /feed /fly /speed /shader /packs",
            ];
            let p = parts.get(1).and_then(|s| s.parse::<usize>().ok());
            if let Some(pg) = p {
                let idx = pg.saturating_sub(1).min(pages.len() - 1);
                game.push_chat(pages[idx]);
            } else if let Some(q) = parts.get(1) {
                match q.trim_start_matches('/').to_ascii_lowercase().as_str() {
                    "give" => game.push_chat("/give <@s|player> <item_name|id> [count]"),
                    "tp" | "teleport" => game.push_chat("/tp [target] <x> <y> <z> (supports ~ relative)"),
                    "setblock" => game.push_chat("/setblock <x> <y> <z> <block>"),
                    "fill" => game.push_chat("/fill <x1> <y1> <z1> <x2> <y2> <z2> <block> [replace|keep|destroy|hollow]"),
                    "summon" => game.push_chat("/summon <mob|tnt> [x] [y] [z]"),
                    "time" => game.push_chat("/time <set|add|query> <day|noon|night|midnight|ticks>"),
                    "weather" => game.push_chat("/weather <clear|rain|thunder>"),
                    "effect" => game.push_chat("/effect <give|clear> <@s> <speed|fire_resistance|regeneration|instant_health|saturation|jump_boost> [sec]"),
                    "attribute" => game.push_chat("/attribute @s <block_interaction_range|entity_interaction_range> <get|base set <n>>"),
                    "gamerule" => game.push_chat("/gamerule <keepInventory|doDaylightCycle|doMobSpawning|mobGriefing|fallDamage|drowningDamage|doWeatherCycle> [true|false]"),
                    _ => {
                        for line in pages { game.push_chat(line); }
                    }
                }
            } else {
                for line in pages {
                    game.push_chat(line);
                }
                game.push_chat("Tip: Press Tab to autocomplete, Up/Down for command history");
            }
            true
        }

        "/gamemode" | "/gm" => {
            if let Some(arg) = parts.get(1) {
                match arg.to_ascii_lowercase().as_str() {
                    "survival" | "s" | "0" => {
                        game.set_mode(GameMode::Survival);
                        game.push_chat("Set own game mode to Survival Mode");
                    }
                    "creative" | "c" | "1" => {
                        game.set_mode(GameMode::Creative);
                        game.push_chat("Set own game mode to Creative Mode");
                    }
                    "adventure" | "a" | "2" => {
                        game.set_mode(GameMode::Survival);
                        game.push_chat("Set own game mode to Survival/Adventure Mode");
                    }
                    "spectator" | "sp" | "3" => {
                        game.set_mode(GameMode::Spectator);
                        game.push_chat("Set own game mode to Spectator Mode");
                    }
                    _ => game.push_chat("Usage: /gamemode <survival|creative|spectator>"),
                }
            } else {
                let next = game.mode.next();
                game.set_mode(next);
                game.push_chat(&format!("Set own game mode to {}", next.name()));
            }
            true
        }

        "/dim" | "/dimension" => {
            if let Some(arg) = parts.get(1) {
                match arg.to_ascii_lowercase().as_str() {
                    "overworld" | "ow" | "0" | "minecraft:overworld" => {
                        game.travel_to(Dimension::Overworld, game.spawn.0, game.spawn.1, game.spawn.2);
                        game.push_chat("Teleported to Overworld");
                    }
                    "nether" | "the_nether" | "1" | "minecraft:the_nether" => {
                        let nx = (game.player.x / 8.0).floor() + 0.5;
                        let nz = (game.player.z / 8.0).floor() + 0.5;
                        game.travel_to(Dimension::Nether, nx, 36.0, nz);
                        game.push_chat("Teleported to The Nether");
                    }
                    "end" | "the_end" | "2" | "minecraft:the_end" => {
                        game.travel_to(Dimension::End, 0.5, 62.5, -18.5);
                        game.push_chat("Teleported to The End");
                    }
                    _ => game.push_chat("Usage: /dim <overworld|nether|end>"),
                }
            } else {
                game.push_chat("Usage: /dim <overworld|nether|end>");
            }
            true
        }

        "/tp" | "/teleport" => {
            let idx = if parts.get(1).map_or(false, |s| is_player_selector(s)) { 2 } else { 1 };
            if parts.len() >= idx + 3 {
                if let (Some(x), Some(y), Some(z)) = (
                    parse_rel_f32(parts[idx], game.player.x),
                    parse_rel_f32(parts[idx + 1], game.player.y),
                    parse_rel_f32(parts[idx + 2], game.player.z),
                ) {
                    game.player.x = x;
                    game.player.y = y.clamp(1.0, 125.0);
                    game.player.z = z;
                    game.player.vy = 0.0;
                    game.fall_y = game.player.y;
                    game.push_chat(&format!("Teleported Player to {:.1}, {:.1}, {:.1}", x, game.player.y, z));
                } else {
                    game.push_chat("Usage: /tp [target] <x> <y> <z>");
                }
            } else if parts.len() == 2 && is_player_selector(parts[1]) {
                game.push_chat(&format!("Player is at {:.1}, {:.1}, {:.1}", game.player.x, game.player.y, game.player.z));
            } else {
                game.push_chat("Usage: /tp [target] <x> <y> <z>");
            }
            true
        }

        "/give" => {
            let (item_idx, count_idx) = if parts.get(1).map_or(false, |s| is_player_selector(s)) {
                (2, 3)
            } else {
                (1, 2)
            };
            if let Some(raw_item) = parts.get(item_idx) {
                if let Some(id) = resolve_item_or_block(raw_item) {
                    if id == 0 {
                        game.push_chat("Cannot give Air");
                        return true;
                    }
                    let count = parts
                        .get(count_idx)
                        .and_then(|s| s.parse::<u16>().ok())
                        .unwrap_or(1)
                        .clamp(1, 256);
                    let mut rem = count;
                    while rem > 0 {
                        let chunk = rem.min(64);
                        let left = game.player.add(id, chunk as u32);
                        if left > 0 {
                            game.drops.push(Drop::new(game.player.x, game.player.y + 0.5, game.player.z, id));
                        }
                        rem -= chunk;
                    }
                    game.plat.play(Sfx::Pickup);
                    game.push_chat(&format!("Gave {} [{}] to Player", count, items::name(id)));
                } else {
                    game.push_chat(&format!("Unknown item '{}'", raw_item));
                }
            } else {
                game.push_chat("Usage: /give <@s|player> <item> [count]");
            }
            true
        }

        "/clear" => {
            let filter = parts.get(2).or_else(|| {
                if parts.get(1).map_or(false, |s| !is_player_selector(s)) {
                    parts.get(1)
                } else {
                    None
                }
            });
            let filter_id = filter.and_then(|s| resolve_item_or_block(s));
            let mut cleared = 0u32;
            for slot in game.player.hot.iter_mut().chain(game.player.store.iter_mut()) {
                if !slot.is_empty() && filter_id.map_or(true, |fid| slot.id == fid) {
                    cleared += slot.count as u32;
                    *slot = Stack::EMPTY;
                }
            }
            if !game.player.offhand.is_empty() && filter_id.map_or(true, |fid| game.player.offhand.id == fid) {
                cleared += game.player.offhand.count as u32;
                game.player.offhand = Stack::EMPTY;
            }
            if filter_id.is_none() {
                for a in game.player.armor.iter_mut() {
                    if !a.is_empty() {
                        cleared += a.count as u32;
                        *a = Stack::EMPTY;
                    }
                }
            }
            game.push_chat(&format!("Removed {} item(s) from Player", cleared));
            true
        }

        "/time" => {
            let sub = parts.get(1).map(|s| s.to_ascii_lowercase());
            let val = parts.get(2).map(|s| s.to_ascii_lowercase());
            match sub.as_deref() {
                Some("set") => match val.as_deref() {
                    Some("day") => { game.time = 0.08; game.push_chat("Set the time to 1000 (day)"); }
                    Some("noon") => { game.time = 0.25; game.push_chat("Set the time to 6000 (noon)"); }
                    Some("sunset") | Some("dusk") => { game.time = 0.48; game.push_chat("Set the time to 12000 (sunset)"); }
                    Some("night") => { game.time = 0.56; game.push_chat("Set the time to 13000 (night)"); }
                    Some("midnight") => { game.time = 0.75; game.push_chat("Set the time to 18000 (midnight)"); }
                    Some("sunrise") | Some("dawn") => { game.time = 0.96; game.push_chat("Set the time to 23000 (sunrise)"); }
                    Some(n) => {
                        if let Ok(ticks) = n.parse::<f32>() {
                            game.time = (ticks / 24000.0).rem_euclid(1.0);
                            game.push_chat(&format!("Set the time to {}", ticks as i32));
                        } else {
                            game.push_chat("Usage: /time set <day|noon|sunset|night|midnight|ticks>");
                        }
                    }
                    None => game.push_chat("Usage: /time set <day|noon|night|midnight|ticks>"),
                },
                Some("add") => {
                    if let Some(Ok(ticks)) = val.as_deref().map(|s| s.parse::<f32>()) {
                        game.time = (game.time + ticks / 24000.0).rem_euclid(1.0);
                        game.push_chat(&format!("Added {} ticks to time", ticks as i32));
                    } else {
                        game.push_chat("Usage: /time add <ticks>");
                    }
                }
                Some("query") => {
                    let mc_ticks = (game.time * 24000.0).round() as i32;
                    game.push_chat(&format!("The time is {} ticks", mc_ticks));
                }
                _ => game.push_chat("Usage: /time <set|add|query> <value>"),
            }
            true
        }

        "/weather" => {
            match parts.get(1).map(|s| s.to_ascii_lowercase()).as_deref() {
                Some("clear") => {
                    game.weather = 0;
                    game.weather_timer = 300.0;
                    game.push_chat("Changing to clear weather");
                }
                Some("rain") => {
                    game.weather = 1;
                    game.weather_timer = 300.0;
                    game.push_chat("Changing to rainy weather");
                }
                Some("thunder") => {
                    game.weather = 2;
                    game.weather_timer = 300.0;
                    game.push_chat("Changing to rain & thunder");
                }
                _ => game.push_chat("Usage: /weather <clear|rain|thunder>"),
            }
            true
        }

        "/summon" => {
            if let Some(raw_mob) = parts.get(1) {
                let mx = parts.get(2).and_then(|s| parse_rel_f32(s, game.player.x)).unwrap_or(game.player.x + 2.0);
                let my = parts.get(3).and_then(|s| parse_rel_f32(s, game.player.y)).unwrap_or(game.player.y + 0.5);
                let mz = parts.get(4).and_then(|s| parse_rel_f32(s, game.player.z)).unwrap_or(game.player.z + 2.0);
                let q = normalize_key(raw_mob);
                if q == "tnt" || q == "primed_tnt" {
                    game.tnt_primed.push((mx, my, mz, 3.5));
                    game.plat.play(Sfx::Furnace);
                    game.push_chat(&format!("Summoned Primed TNT at {:.1}, {:.1}, {:.1}", mx, my, mz));
                } else if let Some(kind) = resolve_mob(raw_mob) {
                    game.mobs.push(Mob::spawn(kind, mx, my, mz));
                    game.push_chat(&format!("Summoned new {} at {:.1}, {:.1}, {:.1}", kind.name(), mx, my, mz));
                } else {
                    game.push_chat(&format!("Unknown entity '{}'", raw_mob));
                }
            } else {
                game.push_chat("Usage: /summon <entity> [x] [y] [z]");
            }
            true
        }

        "/kill" => {
            let target = parts.get(1).copied().unwrap_or("@s");
            if target == "@e" {
                let n = game.mobs.len();
                game.mobs.clear();
                game.drops.clear();
                game.damage_player(1000.0, true);
                game.push_chat(&format!("Killed Player and {} entities", n));
            } else if target.starts_with("@e[type=") && target.ends_with(']') {
                let want = &target[8..target.len() - 1];
                let before = game.mobs.len();
                if let Some(k) = resolve_mob(want) {
                    game.mobs.retain(|m| m.kind != k);
                    game.push_chat(&format!("Killed {} {}(s)", before - game.mobs.len(), k.name()));
                } else if normalize_key(want) == "item" {
                    let d = game.drops.len();
                    game.drops.clear();
                    game.push_chat(&format!("Killed {} dropped item(s)", d));
                } else {
                    game.push_chat(&format!("Unknown entity type '{}'", want));
                }
            } else if target == "@e[type=!player]" || target == "mobs" {
                let n = game.mobs.len();
                game.mobs.clear();
                game.push_chat(&format!("Killed {} mob(s)", n));
            } else {
                game.player.hp = 0.0;
                game.damage_player(1000.0, true);
                game.push_chat("Killed Player");
            }
            true
        }

        "/setblock" => {
            if parts.len() >= 5 {
                if let (Some(x), Some(y), Some(z), Some(b)) = (
                    parse_rel_i32(parts[1], game.player.x),
                    parse_rel_i32(parts[2], game.player.y),
                    parse_rel_i32(parts[3], game.player.z),
                    resolve_block(parts[4]),
                ) {
                    if (1..world::WORLD_H as i32).contains(&y) {
                        game.world.set(x, y, z, b);
                        game.redstone_dirty = true;
                        game.notify_fluid_neighbors(x, y, z);
                        game.push_chat(&format!("Set block at {}, {}, {} to {}", x, y, z, world::block_name(b)));
                    } else {
                        game.push_chat("Y must be between 1 and 95");
                    }
                } else {
                    game.push_chat("Usage: /setblock <x> <y> <z> <block>");
                }
            } else {
                game.push_chat("Usage: /setblock <x> <y> <z> <block>");
            }
            true
        }

        "/fill" => {
            if parts.len() >= 8 {
                if let (Some(x1), Some(y1), Some(z1), Some(x2), Some(y2), Some(z2), Some(b)) = (
                    parse_rel_i32(parts[1], game.player.x),
                    parse_rel_i32(parts[2], game.player.y),
                    parse_rel_i32(parts[3], game.player.z),
                    parse_rel_i32(parts[4], game.player.x),
                    parse_rel_i32(parts[5], game.player.y),
                    parse_rel_i32(parts[6], game.player.z),
                    resolve_block(parts[7]),
                ) {
                    let (lx, hx) = (x1.min(x2), x1.max(x2));
                    let (ly, hy) = (y1.min(y2).max(1), y1.max(y2).min(world::WORLD_H as i32 - 1));
                    let (lz, hz) = (z1.min(z2), z1.max(z2));
                    let vol = ((hx - lx + 1) as i64) * ((hy - ly + 1) as i64) * ((hz - lz + 1) as i64);
                    if vol > 32768 {
                        game.push_chat(&format!("Too many blocks ({} > 32768)", vol));
                        return true;
                    }
                    let mode = parts.get(8).map(|s| s.to_ascii_lowercase()).unwrap_or_else(|| String::from("replace"));
                    let mut changed = 0usize;
                    for x in lx..=hx {
                        for y in ly..=hy {
                            for z in lz..=hz {
                                let edge = x == lx || x == hx || y == ly || y == hy || z == lz || z == hz;
                                let cur = game.world.get(x, y, z);
                                let put = match mode.as_str() {
                                    "keep" => cur == world::AIR,
                                    "hollow" => true,
                                    "outline" => edge,
                                    _ => true,
                                };
                                if put {
                                    let val = if mode == "hollow" && !edge { world::AIR } else { b };
                                    if mode == "destroy" && cur != world::AIR {
                                        game.drops.push(Drop::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5, cur));
                                    }
                                    game.world.set(x, y, z, val);
                                    changed += 1;
                                }
                            }
                        }
                    }
                    game.redstone_dirty = true;
                    game.push_chat(&format!("Successfully filled {} block(s) with {}", changed, world::block_name(b)));
                } else {
                    game.push_chat("Usage: /fill <x1> <y1> <z1> <x2> <y2> <z2> <block> [replace|keep|destroy|hollow|outline]");
                }
            } else {
                game.push_chat("Usage: /fill <x1> <y1> <z1> <x2> <y2> <z2> <block>");
            }
            true
        }

        "/clone" => {
            if parts.len() >= 10 {
                if let (Some(x1), Some(y1), Some(z1), Some(x2), Some(y2), Some(z2), Some(dx), Some(dy), Some(dz)) = (
                    parse_rel_i32(parts[1], game.player.x),
                    parse_rel_i32(parts[2], game.player.y),
                    parse_rel_i32(parts[3], game.player.z),
                    parse_rel_i32(parts[4], game.player.x),
                    parse_rel_i32(parts[5], game.player.y),
                    parse_rel_i32(parts[6], game.player.z),
                    parse_rel_i32(parts[7], game.player.x),
                    parse_rel_i32(parts[8], game.player.y),
                    parse_rel_i32(parts[9], game.player.z),
                ) {
                    let (lx, hx) = (x1.min(x2), x1.max(x2));
                    let (ly, hy) = (y1.min(y2).max(1), y1.max(y2).min(world::WORLD_H as i32 - 1));
                    let (lz, hz) = (z1.min(z2), z1.max(z2));
                    let vol = ((hx - lx + 1) as i64) * ((hy - ly + 1) as i64) * ((hz - lz + 1) as i64);
                    if vol > 32768 {
                        game.push_chat(&format!("Too many blocks ({} > 32768)", vol));
                        return true;
                    }
                    let mut buf = Vec::with_capacity(vol as usize);
                    for x in lx..=hx {
                        for y in ly..=hy {
                            for z in lz..=hz {
                                buf.push((x - lx, y - ly, z - lz, game.world.get(x, y, z)));
                            }
                        }
                    }
                    for (ox, oy, oz, b) in &buf {
                        game.world.set(dx + ox, dy + oy, dz + oz, *b);
                    }
                    game.redstone_dirty = true;
                    game.push_chat(&format!("Cloned {} block(s)", buf.len()));
                } else {
                    game.push_chat("Usage: /clone <x1> <y1> <z1> <x2> <y2> <z2> <dx> <dy> <dz>");
                }
            } else {
                game.push_chat("Usage: /clone <x1> <y1> <z1> <x2> <y2> <z2> <dx> <dy> <dz>");
            }
            true
        }

        "/effect" => {
            let sub = parts.get(1).map(|s| s.to_ascii_lowercase());
            if sub.as_deref() == Some("clear") {
                game.potion_speed = 0.0;
                game.potion_fire = 0.0;
                game.potion_jump = 0.0;
                game.potion_strength = 0.0;
                game.potion_regen = 0.0;
                game.push_chat("Removed all effects from Player");
                return true;
            }
            if sub.as_deref() == Some("give") && parts.len() >= 4 {
                let eff = normalize_key(parts[3]);
                let secs = parts.get(4).and_then(|s| s.parse::<f32>().ok()).unwrap_or(30.0).clamp(1.0, 3600.0);
                let amp = parts.get(5).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
                match eff.as_str() {
                    "speed" | "swiftness" => {
                        game.potion_speed = secs;
                        game.push_chat(&format!("Gave Speed {} for {}s", amp + 1, secs as i32));
                    }
                    "fire_resistance" => {
                        game.potion_fire = secs;
                        game.player.burn = 0.0;
                        game.push_chat(&format!("Gave Fire Resistance for {}s", secs as i32));
                    }
                    "jump_boost" | "jump" => {
                        game.potion_jump = secs;
                        game.push_chat(&format!("Gave Jump Boost for {}s", secs as i32));
                    }
                    "strength" => {
                        game.potion_strength = secs;
                        game.push_chat(&format!("Gave Strength for {}s", secs as i32));
                    }
                    "regeneration" => {
                        game.potion_regen = secs;
                        game.push_chat(&format!("Gave Regeneration for {}s", secs as i32));
                    }
                    "instant_health" | "heal" => {
                        game.player.hp = (game.player.hp + 8.0 * (amp as f32 + 1.0)).min(20.0);
                        game.push_chat("Applied Instant Health");
                    }
                    "saturation" => {
                        game.player.hunger = 20.0;
                        game.push_chat("Applied Saturation");
                    }
                    "haste" | "night_vision" => {
                        game.potion_speed = secs;
                        game.push_chat(&format!("Applied {} for {}s", eff, secs as i32));
                    }
                    _ => game.push_chat("Effects: speed, fire_resistance, regeneration, instant_health, saturation, jump_boost, strength"),
                }
            } else {
                game.push_chat("Usage: /effect <give|clear> @s <effect> [seconds] [amplifier]");
            }
            true
        }

        "/enchant" => {
            if parts.len() >= 3 {
                let held = game.player.held();
                if held.is_empty() {
                    game.push_chat("Hold an item in your main hand to enchant it");
                    return true;
                }
                let lvl = parts.get(3).and_then(|s| s.parse::<u8>().ok()).unwrap_or(1).clamp(1, 5);
                let ench = normalize_key(parts[2]);
                if matches!(ench.as_str(), "sharpness" | "efficiency" | "unbreaking" | "fortune" | "protection" | "power" | "looting") {
                    game.player.enchants.insert(held.id, lvl);
                    game.plat.play(Sfx::LevelUp);
                    game.push_chat(&format!("Applied {} {} to {}", ench, lvl, items::name(held.id)));
                } else {
                    game.push_chat("Enchantments: sharpness, efficiency, unbreaking, fortune, protection");
                }
            } else {
                game.push_chat("Usage: /enchant @s <enchantment> [level]");
            }
            true
        }

        "/xp" | "/experience" => {
            let sub = parts.get(1).map(|s| s.to_ascii_lowercase());
            match sub.as_deref() {
                Some("query") => {
                    game.push_chat(&format!("Player has level {} ({} total XP)", game.player.level, game.player.xp));
                }
                Some("add") | Some("set") => {
                    let amt = parts.get(3).and_then(|s| s.parse::<i32>().ok()).unwrap_or(10).max(0);
                    let unit = parts.get(4).map(|s| s.to_ascii_lowercase()).unwrap_or_else(|| String::from("points"));
                    if sub.as_deref() == Some("set") {
                        if unit.starts_with('l') {
                            game.player.level = amt as u32;
                            game.player.xp = amt as u32 * 10;
                        } else {
                            game.player.xp = amt as u32;
                            game.player.level = (amt as u32) / 10;
                        }
                    } else if unit.starts_with('l') {
                        game.player.add_xp(amt as u32 * 10);
                    } else {
                        game.player.add_xp(amt as u32);
                    }
                    game.plat.play(Sfx::LevelUp);
                    game.push_chat(&format!("Player is now level {} ({} XP)", game.player.level, game.player.xp));
                }
                _ => game.push_chat("Usage: /xp <add|set|query> @s <amount> [levels|points]"),
            }
            true
        }

        "/seed" => {
            game.push_chat("Seed: [79712026]");
            true
        }

        "/spawnpoint" | "/setworldspawn" => {
            let (sx, sy, sz) = if parts.len() >= 4 {
                (
                    parse_rel_f32(parts[1], game.player.x).unwrap_or(game.player.x),
                    parse_rel_f32(parts[2], game.player.y).unwrap_or(game.player.y),
                    parse_rel_f32(parts[3], game.player.z).unwrap_or(game.player.z),
                )
            } else {
                (game.player.x, game.player.y, game.player.z)
            };
            game.spawn = (sx, sy, sz);
            game.push_chat(&format!("Set spawn point to {:.1}, {:.1}, {:.1}", sx, sy, sz));
            true
        }

        "/difficulty" => {
            if let Some(arg) = parts.get(1) {
                match arg.to_ascii_lowercase().as_str() {
                    "peaceful" | "p" | "0" => {
                        game.difficulty = 0;
                        game.mobs.retain(|m| !m.kind.hostile());
                        game.push_chat("Difficulty set to Peaceful");
                    }
                    "easy" | "e" | "1" => { game.difficulty = 1; game.push_chat("Difficulty set to Easy"); }
                    "normal" | "n" | "2" => { game.difficulty = 2; game.push_chat("Difficulty set to Normal"); }
                    "hard" | "h" | "3" => { game.difficulty = 3; game.push_chat("Difficulty set to Hard"); }
                    _ => game.push_chat("Usage: /difficulty <peaceful|easy|normal|hard>"),
                }
            } else {
                let dname = ["Peaceful", "Easy", "Normal", "Hard"][game.difficulty.min(3) as usize];
                game.push_chat(&format!("The difficulty is {}", dname));
            }
            true
        }

        "/gamerule" => {
            if let Some(rule) = parts.get(1) {
                let val = parts.get(2).map(|s| s.eq_ignore_ascii_case("true"));
                let rlow = rule.to_ascii_lowercase();
                let target_ref: Option<(&str, &mut bool)> = match rlow.as_str() {
                    "keepinventory" => Some(("keepInventory", &mut game.rule_keep_inventory)),
                    "dodaylightcycle" => Some(("doDaylightCycle", &mut game.rule_daylight_cycle)),
                    "domobspawning" => Some(("doMobSpawning", &mut game.rule_mob_spawning)),
                    "mobgriefing" => Some(("mobGriefing", &mut game.rule_mob_griefing)),
                    "falldamage" => Some(("fallDamage", &mut game.rule_fall_damage)),
                    "drowningdamage" => Some(("drowningDamage", &mut game.rule_drowning_damage)),
                    "doweathercycle" => Some(("doWeatherCycle", &mut game.rule_weather_cycle)),
                    _ => None,
                };
                if let Some((canon, slot)) = target_ref {
                    let cur = if let Some(b) = val {
                        *slot = b;
                        b
                    } else {
                        *slot
                    };
                    if val.is_some() {
                        game.push_chat(&format!("Gamerule {} is now set to: {}", canon, cur));
                    } else {
                        game.push_chat(&format!("Gamerule {} is currently set to: {}", canon, cur));
                    }
                } else {
                    game.push_chat("Rules: keepInventory, doDaylightCycle, doMobSpawning, mobGriefing, fallDamage, drowningDamage, doWeatherCycle");
                }
            } else {
                game.push_chat("Usage: /gamerule <rule> [true|false]");
            }
            true
        }

        "/attribute" => {
            if parts.len() >= 4 {
                let attr = normalize_key(parts[2]);
                let op = parts[3].to_ascii_lowercase();
                if op == "get" {
                    let v = match attr.as_str() {
                        "block_interaction_range" => game.block_reach(),
                        "entity_interaction_range" => game.entity_reach(),
                        "movement_speed" => game.speed_mul,
                        "max_health" => 20.0,
                        _ => { game.push_chat("Unknown attribute"); return true; }
                    };
                    game.push_chat(&format!("Attribute {} is {:.2}", attr, v));
                } else if op == "base" && parts.get(4).map(|s| s.eq_ignore_ascii_case("set")).unwrap_or(false) {
                    if let Some(Ok(v)) = parts.get(5).map(|s| s.parse::<f32>()) {
                        let clamped = v.clamp(0.5, 64.0);
                        match attr.as_str() {
                            "block_interaction_range" => { game.block_reach_base = clamped; game.push_chat(&format!("block_interaction_range base set to {:.2}", clamped)); }
                            "entity_interaction_range" => { game.entity_reach_base = clamped; game.push_chat(&format!("entity_interaction_range base set to {:.2}", clamped)); }
                            "movement_speed" => { game.speed_mul = clamped.clamp(0.2, 5.0); game.push_chat(&format!("movement_speed set to {:.2}", game.speed_mul)); }
                            _ => game.push_chat("Editable: block_interaction_range, entity_interaction_range, movement_speed"),
                        }
                    } else {
                        game.push_chat("Usage: /attribute @s <attr> base set <value>");
                    }
                } else {
                    game.push_chat("Usage: /attribute @s <block_interaction_range|entity_interaction_range> <get|base set <val>>");
                }
            } else {
                game.push_chat("Usage: /attribute @s <block_interaction_range|entity_interaction_range> <get|base set <val>>");
            }
            true
        }

        "/locate" => {
            let what = parts.get(2).or_else(|| parts.get(1)).map(|s| normalize_key(s)).unwrap_or_default();
            let (lx, ly, lz, label) = match what.as_str() {
                "stronghold" => (100.0f32, 19.0f32, -140.0f32, "Stronghold"),
                "village" => (48.0, 34.0, 48.0, "Village"),
                "ruined_portal" => (-48.0, 34.0, 32.0, "Ruined Portal"),
                "fortress" | "nether_fortress" => (32.0, 36.0, 32.0, "Nether Fortress"),
                "end_city" => (0.0, 62.0, -18.0, "End Island"),
                _ => (100.0, 19.0, -140.0, "Stronghold"),
            };
            let dist = ((lx - game.player.x).powi(2) + (lz - game.player.z).powi(2)).sqrt();
            game.push_chat(&format!("The nearest {} is at [{:.0}, {:.0}, {:.0}] ({:.0} blocks away)", label, lx, ly, lz, dist));
            true
        }

        "/execute" => {
            if let Some(run_pos) = parts.iter().position(|p| p.eq_ignore_ascii_case("run")) {
                let mut idx = 1;
                while idx < run_pos {
                    if parts[idx].eq_ignore_ascii_case("in") && idx + 1 < run_pos {
                        let d = normalize_key(parts[idx + 1]);
                        match d.as_str() {
                            "overworld" => game.travel_to(Dimension::Overworld, game.spawn.0, game.spawn.1, game.spawn.2),
                            "the_nether" | "nether" => game.travel_to(Dimension::Nether, game.player.x / 8.0, 36.0, game.player.z / 8.0),
                            "the_end" | "end" => game.travel_to(Dimension::End, 0.5, 62.5, -18.5),
                            _ => {}
                        }
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                if run_pos + 1 < parts.len() {
                    let sub_cmd = format!("/{}", parts[run_pos + 1..].join(" ").trim_start_matches('/'));
                    return execute(game, &sub_cmd);
                }
            }
            game.push_chat("Usage: /execute in <overworld|the_nether|the_end> run <command>");
            true
        }

        "/playsound" => {
            if let Some(s) = parts.get(1) {
                let q = s.to_ascii_lowercase();
                let sfx = if q.contains("explode") || q.contains("tnt") { Sfx::Explode }
                    else if q.contains("level") || q.contains("xp") { Sfx::LevelUp }
                    else if q.contains("pop") || q.contains("pickup") { Sfx::Pickup }
                    else if q.contains("cow") { Sfx::Cow }
                    else if q.contains("sheep") { Sfx::Sheep }
                    else if q.contains("pig") { Sfx::Pig }
                    else if q.contains("chicken") { Sfx::Chicken }
                    else if q.contains("creeper") { Sfx::Creeper }
                    else if q.contains("zombie") { Sfx::Zombie }
                    else if q.contains("skeleton") { Sfx::Skeleton }
                    else if q.contains("enderman") { Sfx::Enderman }
                    else if q.contains("break") { Sfx::Break }
                    else if q.contains("place") { Sfx::Place }
                    else if q.contains("door") { Sfx::Door }
                    else { Sfx::Click };
                game.plat.play(sfx);
                game.push_chat(&format!("Played sound '{}'", s));
            } else {
                game.push_chat("Usage: /playsound <sound>");
            }
            true
        }

        "/stopsound" => {
            game.push_chat("Stopped active sounds");
            true
        }

        "/particle" => {
            let name = parts.get(1).copied().unwrap_or("spark");
            game.push_chat(&format!("Spawned particle {}", name));
            true
        }

        "/title" => {
            if parts.len() >= 3 {
                let sub = parts[2].to_ascii_lowercase();
                let txt = if parts.len() >= 4 { parts[3..].join(" ") } else { String::new() };
                match sub.as_str() {
                    "clear" | "reset" => { game.message.clear(); game.msg_timer = 0.0; }
                    "title" | "subtitle" | "actionbar" => {
                        game.message = txt;
                        game.msg_timer = 4.5;
                    }
                    _ => game.push_chat("Usage: /title @s <title|subtitle|actionbar|clear> <text>"),
                }
            } else {
                game.push_chat("Usage: /title @s <title|actionbar|clear> <text>");
            }
            true
        }

        "/say" => {
            let msg = if parts.len() > 1 { parts[1..].join(" ") } else { String::new() };
            game.push_chat(&format!("[Server] {}", msg));
            true
        }

        "/me" => {
            let msg = if parts.len() > 1 { parts[1..].join(" ") } else { String::new() };
            game.push_chat(&format!("* Player {}", msg));
            true
        }

        "/msg" | "/tell" | "/w" => {
            if parts.len() >= 3 {
                let who = parts[1];
                let msg = parts[2..].join(" ");
                game.push_chat(&format!("You whisper to {}: {}", who, msg));
            } else {
                game.push_chat("Usage: /msg <player> <message>");
            }
            true
        }

        "/list" => {
            let n = 1 + game.remote.len();
            game.push_chat(&format!("There are {} of a max of 20 players online: Player", n));
            true
        }

        "/heal" => {
            game.player.hp = 20.0;
            game.player.burn = 0.0;
            game.push_chat("Healed Player to full health");
            true
        }

        "/feed" => {
            game.player.hunger = 20.0;
            game.push_chat("Fed Player to full hunger");
            true
        }

        "/fly" => {
            game.flying = !game.flying;
            game.player.vy = 0.0;
            game.push_chat(&format!("Flight {}", if game.flying { "enabled" } else { "disabled" }));
            true
        }

        "/speed" => {
            if let Some(Ok(v)) = parts.get(1).map(|s| s.parse::<f32>()) {
                game.speed_mul = v.clamp(0.2, 5.0);
                game.push_chat(&format!("Movement speed set to {:.2}x", game.speed_mul));
            } else {
                game.push_chat("Usage: /speed <0.2..5.0>");
            }
            true
        }

        "/home" | "/spawn" => {
            game.player.x = game.spawn.0;
            game.player.y = game.spawn.1;
            game.player.z = game.spawn.2;
            game.player.vy = 0.0;
            game.fall_y = game.spawn.1;
            game.push_chat(&format!("Teleported to spawn ({:.1}, {:.1}, {:.1})", game.spawn.0, game.spawn.1, game.spawn.2));
            true
        }

        "/shader" => {
            let sh = if let Some(arg) = parts.get(1) {
                if let Ok(idx) = arg.parse::<usize>() {
                    packs::set_shader_preset(idx)
                } else {
                    let al = arg.to_ascii_lowercase();
                    let idx = if al.contains("bsl") { 1 }
                        else if al.contains("comp") { 2 }
                        else if al.contains("sildur") { 3 }
                        else if al.contains("nost") { 4 }
                        else if al.contains("van") || al.contains("off") { 0 }
                        else {
                            let s = packs::cycle_shader_preset();
                            game.push_chat(&format!("Shader: {}", s.name));
                            return true;
                        };
                    packs::set_shader_preset(idx)
                }
            } else {
                packs::cycle_shader_preset()
            };
            game.push_chat(&format!("Shader set to {}", sh.name));
            true
        }

        "/packs" => {
            let loaded = packs::loaded_packs();
            if loaded.is_empty() {
                game.push_chat("No packs loaded. Drop .zip/.mrpack in packs/ or use the web Pack Loader.");
            } else {
                for p in &loaded {
                    game.push_chat(&format!("[{}] {}: {}", p.kind.label(), p.name, p.description));
                }
            }
            true
        }

        "/reloadpacks" => {
            #[cfg(not(target_arch = "wasm32"))]
            game.reload_native_packs();
            #[cfg(target_arch = "wasm32")]
            game.push_chat(&format!("{} pack(s) active in browser", packs::pack_count()));
            true
        }

        _ => {
            game.push_chat(&format!("Unknown command '{}'. Type /help for all commands.", parts[0]));
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_resolution_and_autocomplete_work() {
        assert_eq!(resolve_item_or_block("diamond_sword"), Some(items::DIAMOND_SWORD));
        assert_eq!(resolve_item_or_block("minecraft:cobblestone"), Some(world::COBBLE));
        assert_eq!(resolve_block("oak_planks"), Some(world::PLANKS));
        assert_eq!(resolve_mob("minecraft:creeper"), Some(MobKind::Creeper));
        assert_eq!(autocomplete("/gamem"), Some(String::from("/gamemode ")));
        assert_eq!(autocomplete("/weather r"), Some(String::from("/weather rain ")));
    }
}

