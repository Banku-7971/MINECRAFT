mod world;
mod entities;
mod render;

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::path::Path;

use world::{World, Biome, Hit, WORLD_H};
use entities::{Mob, MobKind, Drop};
use render::{Cam, Framebuffer, Textures, SpriteKind};

const HOT_SIZE: usize = 9;
const STORE_SIZE: usize = 27;
const REACH: f32 = 5.5;

#[derive(Clone)]
struct Player {
    x: f32, y: f32, z: f32, vy: f32,
    yaw: f32, pitch: f32,
    hp: f32, hunger: f32,
    on_ground: bool,
    selected: usize,
    hot: [u16; HOT_SIZE],
    counts: [u32; HOT_SIZE],
    store: [u16; STORE_SIZE],
    store_counts: [u32; STORE_SIZE],
}

impl Player {
    fn new(x: f32, y: f32, z: f32) -> Self {
        let mut start: [u16; 9] = [0; 9];
        let mut counts: [u32; 9] = [0; 9];
        let defaults = [world::STONE, world::DIRT, world::GRASS, world::PLANKS, world::GLASS, world::TORCH, world::COBBLE, world::LOG, world::LEAVES];
        for i in 0..9 { start[i] = defaults[i]; counts[i] = 64; }
        Self {
            x, y, z, vy: 0.0, yaw: 0.0, pitch: -0.08, hp: 20.0, hunger: 20.0, on_ground: false,
            selected: 0, hot: start, counts, store: [0; STORE_SIZE], store_counts: [0; STORE_SIZE],
        }
    }
}

fn field_str(line: &str, key: &str) -> Option<String> {
    let pat = format!("\"{}\"", key);
    let i = line.find(&pat)? + pat.len();
    let rest = &line[i..];
    let c = rest.find(':')?;
    let rest = rest[c + 1..].trim_start();
    if !rest.starts_with('"') { return None; }
    let end = rest[1..].find('"')? + 1;
    Some(rest[1..end].to_string())
}
fn field_num(line: &str, key: &str) -> Option<f32> {
    let pat = format!("\"{}\"", key);
    let i = line.find(&pat)? + pat.len();
    let c = line[i..].find(':')?;
    let rest = line[i + c + 1..].trim_start();
    let mut end = 0;
    for (j, ch) in rest.char_indices() {
        if ch.is_ascii_digit() || ch == '-' || ch == '.' || ch == 'e' { end = j + ch.len_utf8(); } else { break; }
    }
    if end == 0 { None } else { rest[..end].parse().ok() }
}
fn field_bool(line: &str, key: &str) -> bool {
    let pat = format!("\"{}\"", key);
    if let Some(i) = line.find(&pat) {
        if let Some(c) = line[i + pat.len()..].find(':') {
            return line[i + pat.len() + c + 1..].trim_start().starts_with("true");
        }
    }
    false
}

fn add_item(player: &mut Player, item: u16, n: u32) {
    for i in 0..HOT_SIZE {
        if player.hot[i] == item { player.counts[i] += n; return; }
    }
    for i in 0..STORE_SIZE {
        if player.store[i] == item { player.store_counts[i] += n; return; }
    }
    for i in 0..HOT_SIZE {
        if player.hot[i] == 0 { player.hot[i] = item; player.counts[i] = n; return; }
    }
    for i in 0..STORE_SIZE {
        if player.store[i] == 0 { player.store[i] = item; player.store_counts[i] = n; return; }
    }
    player.counts[player.selected] += n;
}

fn collides(world: &mut World, x: f32, y: f32, z: f32) -> bool {
    let r = 0.29;
    for dx in [-r, r] {
        for dz in [-r, r] {
            for dy in [0.1, 0.9, 1.7] {
                let b = world.get((x + dx).floor() as i32, (y + dy).floor() as i32, (z + dz).floor() as i32);
                if world::is_solid(b) { return true; }
            }
        }
    }
    false
}

fn all_items() -> Vec<u16> {
    let mut v = Vec::new();
    for i in 1..=world::MAX_BLOCK { v.push(i); }
    for i in 256..=305 { v.push(i); }
    v
}

fn item_name(id: u16) -> &'static str {
    match id {
        256 => "Wooden Pickaxe", 257 => "Stone Pickaxe", 258 => "Iron Pickaxe", 259 => "Diamond Pickaxe",
        260 => "Netherite Pickaxe", 261 => "Wooden Axe", 262 => "Iron Axe", 263 => "Diamond Axe",
        264 => "Wooden Shovel", 265 => "Iron Shovel", 266 => "Wooden Sword", 267 => "Iron Sword",
        268 => "Diamond Sword", 269 => "Apple", 270 => "Bread", 271 => "Cooked Porkchop",
        272 => "Raw Porkchop", 273 => "Raw Beef", 274 => "Steak", 275 => "Raw Chicken",
        276 => "Cooked Chicken", 277 => "Fish", 278 => "Coal", 279 => "Iron Ingot", 280 => "Gold Ingot",
        281 => "Diamond", 282 => "Emerald", 283 => "Stick", 284 => "Flint", 285 => "Feather",
        286 => "Leather", 287 => "Bone", 288 => "Rotten Flesh", 289 => "String", 290 => "Wool",
        291 => "Egg", 292 => "Bucket", 293 => "Water Bucket", 294 => "Lava Bucket", 295 => "Compass",
        296 => "Map", 297 => "Clock", 298 => "Elytra", 299 => "Firework Rocket", 300 => "Shield",
        301 => "Totem of Undying", 302 => "Ender Pearl", 303 => "Blaze Rod", 304 => "Nether Star",
        305 => "Experience Bottle", _ => world::block_name(id),
    }
}

fn mob_ray(player: &Player, mobs: &[Mob], dx: f32, dy: f32, dz: f32, max: f32) -> Option<(usize, f32)> {
    let mut best = None;
    for (i, m) in mobs.iter().enumerate() {
        let h = m.kind.height();
        let cy = m.y + h * 0.55;
        let ox = m.x - player.x;
        let oy = cy - (player.y + 1.62);
        let oz = m.z - player.z;
        let proj = ox * dx + oy * dy + oz * dz;
        if proj <= 0.0 || proj > max { continue; }
        let cx = ox - dx * proj;
        let cy2 = oy - dy * proj;
        let cz = oz - dz * proj;
        let d2 = cx * cx + cy2 * cy2 + cz * cz;
        let r = if m.kind == MobKind::Spider { 0.55 } else { 0.45 };
        if d2 < r * r && best.map_or(true, |(_, t)| proj < t) { best = Some((i, proj)); }
    }
    best
}

fn main() {
    let runtime = Path::new("runtime");
    let _ = std::fs::create_dir_all(runtime);
    let event_path = runtime.join("events.log");
    let _ = std::fs::write(&event_path, b"");
    let frame_path = runtime.join("frame.png");
    let tmp_path = runtime.join("frame.tmp.png");

    let mut world = World::new();
    let spawn_x = 0.5;
    let spawn_z = -239.5;
    let spawn_y = world.highest(spawn_x as i32, spawn_z as i32) as f32 + 0.2;
    let mut player = Player::new(spawn_x, spawn_y, spawn_z);
    let mut mobs: Vec<Mob> = Vec::new();
    let mut drops: Vec<Drop> = Vec::new();
    // Populate the neighbourhood so first frame is a real world, not an empty plain.
    for i in 0..26 {
        let a = i as f32 * 2.39996;
        let d = 6.0 + (i % 9) as f32 * 2.4;
        let x = spawn_x + a.cos() * d;
        let z = spawn_z + a.sin() * d;
        let k = entities::random_kind(i * 17 + 3, 0.6);
        mobs.push(Mob::spawn(k, x, world.highest(x as i32, z as i32) as f32, z));
    }

    let mut scene = Framebuffer::new(render::RW, render::RH);
    let mut fb = Framebuffer::new(render::W, render::H);
    let tex = Textures::new();
    let items = all_items();
    let keys = std::collections::HashSet::<String>::new();
    let mut keys = keys;
    let mut inv_open = false;
    let mut camera_mode = 0u8;
    let mut inv_page = 0usize;
    let mut tick: u32 = 0;
    let mut time = 0.30f32;
    let mut message = String::from("Welcome! WASD move, mouse look, F5 camera, E inventory");
    let mut msg_timer = 6.0;
    let mut fps = 0u32;
    let mut last_fps = Instant::now();
    let mut frames = 0u32;
    let mut last_frame = Instant::now();
    let mut offset = 0u64;
    let mut last_spawn = Instant::now();
    let mut last_click = 0u32;
    let mut hurt_flash = 0.0f32;
    let mut grace = 18.0f32;
    let mut regen = 0.0f32;
    let mut hurt_cd = 0.0f32;

    loop {
        let now = Instant::now();
        let elapsed = now.duration_since(last_frame).as_secs_f32().min(0.12);
        last_frame = now;
        tick = tick.wrapping_add(1);

        // --- events ---
        if let Ok(data) = std::fs::read(&event_path) {
            if data.len() as u64 > offset {
                let text = String::from_utf8_lossy(&data[offset as usize..]);
                offset = data.len() as u64;
                for line in text.lines() {
                    if line.trim().is_empty() { continue; }
                    let t = field_str(line, "t").unwrap_or_default();
                    match t.as_str() {
                        "key" => {
                            let code = field_str(line, "code").unwrap_or_default();
                            let down = field_bool(line, "down");
                            if down {
                                if code == "F5" {
                                    camera_mode = (camera_mode + 1) % 3;
                                    message = format!("Camera: {}", ["First person", "Third person", "Distant third person"][camera_mode as usize]);
                                    msg_timer = 2.0;
                                } else if code == "KeyE" {
                                    inv_open = !inv_open;
                                } else if code == "Escape" {
                                    inv_open = false;
                                } else if code.starts_with("Digit") {
                                    if let Ok(n) = code[5..].parse::<usize>() { if n >= 1 && n <= 9 { player.selected = n - 1; } }
                                }
                                keys.insert(code);
                            } else { keys.remove(&code); }
                        }
                        "mouse" => {
                            player.yaw += field_num(line, "dx").unwrap_or(0.0) * 0.0026;
                            player.pitch = (player.pitch - field_num(line, "dy").unwrap_or(0.0) * 0.0024).clamp(-1.52, 1.52);
                        }
                        "click" => {
                            let seq = field_num(line, "seq").unwrap_or(0.0) as u32;
                            if seq != last_click {
                                last_click = seq;
                                let button = field_num(line, "button").unwrap_or(0.0) as i32;
                                let x = field_num(line, "x").unwrap_or(0.0) as i32;
                                let y = field_num(line, "y").unwrap_or(0.0) as i32;
                                if inv_open {
                                    handle_inventory_click(&mut player, &items, &mut inv_page, x, y);
                                } else if button == 2 {
                                    player.selected = ((player.selected as i32 + field_num(line, "delta").unwrap_or(1.0) as i32) + 9) as usize % 9;
                                } else {
                                    // Clicks outside the canvas still aim at the crosshair.
                                    let _ = (x, y);
                                    interact(&mut world, &mut player, &mut mobs, &mut drops, button == 0, &mut message, &mut msg_timer);
                                }
                            }
                        }
                        "wheel" => {
                            if inv_open {
                                let d = field_num(line, "delta").unwrap_or(1.0) as i32;
                                inv_page = (inv_page as i32 + d).rem_euclid(((items.len() + 23) / 24).max(1) as i32) as usize;
                            } else {
                                let d = field_num(line, "delta").unwrap_or(1.0) as i32;
                                player.selected = (player.selected as i32 + d).rem_euclid(9) as usize;
                            }
                        }
                        "blur" => keys.clear(),
                        _ => {}
                    }
                }
            }
        }

        let day = (time * std::f32::consts::PI * 2.0).sin() * 0.5 + 0.5;

        // --- physics ---
        if !inv_open {
            let mut mx: f32 = 0.0; let mut mz: f32 = 0.0;
            if keys.contains("KeyW") { mz -= 1.0; }
            if keys.contains("KeyS") { mz += 1.0; }
            if keys.contains("KeyA") { mx -= 1.0; }
            if keys.contains("KeyD") { mx += 1.0; }
            let len = (mx * mx + mz * mz).sqrt().max(1.0);
            mx /= len; mz /= len;
            let sprint = keys.contains("ShiftLeft") || keys.contains("ShiftRight");
            let speed = if sprint { 7.2 } else { 4.5 };
            let sin = player.yaw.sin(); let cos = player.yaw.cos();
            let vx = (mx * cos + mz * sin) * speed;
            let vz = (mx * sin - mz * cos) * speed;
            let dt = elapsed.min(0.05);
            let nx = player.x + vx * dt;
            if !collides(&mut world, nx, player.y, player.z) { player.x = nx; }
            let nz = player.z + vz * dt;
            if !collides(&mut world, player.x, player.y, nz) { player.z = nz; }
            if keys.contains("Space") && player.on_ground {
                player.vy = 8.4;
                player.on_ground = false;
            }
            player.vy -= 22.0 * dt;
            let ny = player.y + player.vy * dt;
            if collides(&mut world, player.x, ny, player.z) {
                if player.vy < 0.0 {
                    // step up one block when walking into it
                    if !collides(&mut world, player.x, player.y + 0.65, player.z) && player.vy < -1.0 {
                        player.y += 0.65;
                    } else { player.on_ground = true; }
                }
                player.vy = 0.0;
            } else { player.y = ny; player.on_ground = false; }
        }

        // --- entities ---
        for m in mobs.iter_mut() { m.update(&mut world, player.x, player.y, player.z, elapsed.min(0.05), tick, day); }
        mobs.retain(|m| m.hp > 0.0 && (m.x - player.x).abs() < 90.0 && (m.z - player.z).abs() < 90.0);
        for m in mobs.iter_mut() {
            if m.kind.hostile() && grace <= 0.0 {
                let d = ((m.x - player.x).powi(2) + (m.z - player.z).powi(2)).sqrt();
                if d < 1.7 && m.attack_cd <= 0.0 && hurt_cd <= 0.0 {
                    player.hp -= 2.0; hurt_flash = 0.4; m.attack_cd = 1.2; hurt_cd = 0.65;
                }
            }
        }
        // slow health regeneration while well fed
        if player.hp < 20.0 && player.hunger > 6.0 {
            regen = regen + elapsed;
            if regen > 3.5 { regen = 0.0; player.hp = (player.hp + 1.0).min(20.0); }
        } else { regen = 0.0; }
        if last_spawn.elapsed() > Duration::from_millis(1600) {
            last_spawn = Instant::now();
            if mobs.len() < 30 {
                let a = (tick as f32 * 0.618).sin() * 6.283;
                let d = 24.0 + (tick % 17) as f32;
                let x = player.x + a.cos() * d;
                let z = player.z + a.sin() * d;
                let k = entities::random_kind(tick.wrapping_mul(7919), day);
                mobs.push(Mob::spawn(k, x, world.highest(x as i32, z as i32) as f32, z));
            }
        }
        for d in drops.iter_mut() { d.update(&mut world, elapsed.min(0.05)); }
        drops.retain(|d| {
            let dist = ((d.x - player.x).powi(2) + (d.y - player.y - 1.0).powi(2) + (d.z - player.z).powi(2)).sqrt();
            if dist < 1.8 { add_item(&mut player, d.item, 1); message = format!("Picked up {}", item_name(d.item)); msg_timer = 1.5; false } else { dist < 50.0 }
        });
        if hurt_flash > 0.0 { hurt_flash -= elapsed; }
        grace = (grace - elapsed).max(0.0);
        hurt_cd = (hurt_cd - elapsed).max(0.0);
        time = (time + elapsed * 0.0018) % 1.0;
        msg_timer = (msg_timer - elapsed).max(0.0);
        player.hunger = (player.hunger - elapsed * 0.00012).max(0.0);
        if player.hp <= 0.0 {
            player.x = spawn_x; player.z = spawn_z; player.y = spawn_y;
            player.hp = 20.0; player.hunger = 20.0;
            message = "You died! Respawned at spawn.".to_string(); msg_timer = 4.0;
        }
        world.prune(player.x.floor() as i32 / 16, player.z.floor() as i32 / 16);

        // --- camera ---
        let cp = player.pitch.cos();
        let fx = player.yaw.sin() * cp;
        let fy = player.pitch.sin();
        let fz = -player.yaw.cos() * cp;
        let eye_y = player.y + 1.62;
        let (cam_x, cam_y, cam_z) = if camera_mode == 0 {
            (player.x, eye_y, player.z)
        } else {
            let back = if camera_mode == 1 { 3.6 } else { 6.5 };
            let tx = player.x - fx * back;
            let ty = eye_y - fy * back - 0.35;
            let tz = player.z - fz * back;
            if let Some(h) = world::raycast(&mut world, player.x, eye_y, player.z, -fx, -fy, -fz, back + 0.5, false) {
                if h.dist > 1.0 {
                    let min_back = if camera_mode == 1 { 2.4 } else { 4.6 };
                    let f = ((h.dist - 0.35).max(min_back) / back).min(1.0);
                    (player.x - fx * back * f, eye_y - fy * back * f - 0.35 * f, player.z - fz * back * f)
                } else { (tx, ty, tz) }
            } else { (tx, ty, tz) }
        };
        let cam = Cam::from(player.yaw, player.pitch, cam_x, cam_y, cam_z);

        // --- draw ---
        let render_start = Instant::now();
        render::render_world(&mut scene, &tex, &mut world, &cam, time, tick);
        if camera_mode != 0 {
            render::draw_sprite(&mut scene, &cam, player.x, player.y, player.z, 1.8, tick, day, SpriteKind::Steve, true);
        }
        for m in mobs.iter() {
            render::draw_sprite(&mut scene, &cam, m.x, m.y, m.z, m.kind.height(), tick, day, SpriteKind::Mob(m.kind), false);
        }
        for d in drops.iter() {
            render::draw_sprite(&mut scene, &cam, d.x, d.y, d.z, 0.35, tick, day, SpriteKind::Drop(d.item), false);
        }
        if hurt_flash > 0.0 {
            let a = (hurt_flash * 110.0) as u8;
            scene.rect(0, 0, scene.w as i32, scene.h as i32, [180, 0, 0, a]);
        }
        render::upscale(&scene, &mut fb);
        if camera_mode == 0 { render::draw_held_item(&mut fb, player.hot[player.selected], tick); }
        let biome = world::biome_at(player.x.floor() as i32, player.z.floor() as i32);
        render::draw_hud(&mut fb, &player.hot, &player.counts, player.selected, player.hp, player.hunger, tick, &message, msg_timer, biome, (player.x, player.y, player.z), fps, camera_mode, time);
        if inv_open {
            render::draw_inventory(&mut fb, &player.store, &player.store_counts, &player.hot, &player.counts, player.selected, &items, inv_page, tick);
        }
        if let Err(e) = render::encode_png(&tmp_path, &fb.rgba, fb.w, fb.h) { eprintln!("png error: {}", e); }
        let _ = std::fs::rename(&tmp_path, &frame_path);

        frames += 1;
        if last_fps.elapsed() >= Duration::from_secs(1) {
            fps = frames; frames = 0; last_fps = Instant::now();
        }
        let target = Duration::from_millis(40);
        let used = now.elapsed();
        if used < target { std::thread::sleep(target - used); }
        let _ = WORLD_H;
        let mut _unused: HashMap<u8, u8> = HashMap::new();
        let _ = &mut _unused;
        let _hit: Option<Hit> = None;
    }
}

fn handle_inventory_click(player: &mut Player, items: &[u16], page: &mut usize, x: i32, y: i32) {
    let inside = |r: (i32, i32, i32, i32)| x >= r.0 && x <= r.0 + r.2 && y >= r.1 && y <= r.1 + r.3;
    for i in 0..HOT_SIZE { if inside(render::inv_hotbar_rect(i)) { player.selected = i; return; } }
    for i in 0..STORE_SIZE { if inside(render::inv_storage_rect(i)) { player.selected = i % 9; return; } }
    let p = render::inv_prev_rect();
    if inside(p) { *page = (*page + items.len() / 24 + 1) % ((items.len() + 23) / 24); return; }
    let n = render::inv_next_rect();
    if inside(n) { *page = (*page + 1) % ((items.len() + 23) / 24); return; }
    for i in 0..24 {
        let idx = *page * 24 + i;
        if idx < items.len() && inside(render::inv_list_rect(i)) {
            let id = items[idx];
            player.hot[player.selected] = id;
            player.counts[player.selected] = if id >= 256 { 1 } else { 64 };
            return;
        }
    }
}

fn interact(world: &mut World, player: &mut Player, mobs: &mut Vec<Mob>, drops: &mut Vec<Drop>, left: bool, message: &mut String, msg_timer: &mut f32) {
    let cp = player.pitch.cos();
    let dx = player.yaw.sin() * cp;
    let dy = player.pitch.sin();
    let dz = -player.yaw.cos() * cp;
    let eye = player.y + 1.62;
    let block = world::raycast(world, player.x, eye, player.z, dx, dy, dz, REACH, false);
    let mob = mob_ray(player, mobs, dx, dy, dz, REACH);
    if left {
        if let Some((idx, t)) = mob {
            if block.map_or(true, |b| t < b.dist) {
                let m = &mut mobs[idx];
                m.hp -= if player.hot[player.selected] >= 266 && player.hot[player.selected] <= 268 { 7.0 } else { 4.0 };
                m.hurt = 0.5;
                m.vx += dx * 5.0; m.vz += dz * 5.0;
                *message = format!("Hit {}", m.kind.name());
                *msg_timer = 1.0;
                if m.hp <= 0.0 {
                    let dead = mobs.remove(idx);
                    drops.push(Drop { item: dead.kind.drop(), x: dead.x, y: dead.y + 0.6, z: dead.z, vy: 2.0, age: 0.0 });
                    *message = format!("{} dropped {}", dead.kind.name(), item_name(dead.kind.drop()));
                    *msg_timer = 2.0;
                }
                return;
            }
        }
        if let Some(h) = block {
            if h.block == world::BEDROCK { return; }
            world.set(h.x, h.y, h.z, world::AIR);
            add_item(player, h.block, 1);
            *message = format!("Broke {}", world::block_name(h.block));
            *msg_timer = 1.2;
        }
    } else if let Some(h) = block {
        let item = player.hot[player.selected];
        if item == 0 || item >= 256 { return; }
        let x = h.x + h.nx; let y = h.y + h.ny; let z = h.z + h.nz;
        if y < 0 || y >= WORLD_H as i32 { return; }
        if world::is_solid(world.get(x, y, z)) { return; }
        if collides(world, player.x, player.y, player.z) { return; }
        world.set(x, y, z, item);
        player.counts[player.selected] = player.counts[player.selected].saturating_sub(1);
        if player.counts[player.selected] == 0 { player.hot[player.selected] = 0; }
        *message = format!("Placed {}", world::block_name(item));
        *msg_timer = 1.0;
    }
}
