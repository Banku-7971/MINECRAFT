// Pixel-art sprite definitions and simple creature behaviour.
use crate::world::{self, World};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum MobKind { Sheep, Cow, Pig, Chicken, Zombie, Skeleton, Creeper, Spider, Enderman, Villager }

impl MobKind {
    pub fn name(self) -> &'static str {
        match self {
            MobKind::Sheep => "Sheep", MobKind::Cow => "Cow", MobKind::Pig => "Pig",
            MobKind::Chicken => "Chicken", MobKind::Zombie => "Zombie", MobKind::Skeleton => "Skeleton",
            MobKind::Creeper => "Creeper", MobKind::Spider => "Spider", MobKind::Enderman => "Enderman",
            MobKind::Villager => "Villager",
        }
    }
    pub fn hostile(self) -> bool {
        matches!(self, MobKind::Zombie | MobKind::Skeleton | MobKind::Creeper | MobKind::Spider | MobKind::Enderman)
    }
    pub fn health(self) -> f32 {
        match self { MobKind::Spider => 16.0, MobKind::Enderman => 40.0, MobKind::Chicken => 4.0, MobKind::Villager => 20.0, _ => 10.0 }
    }
    pub fn drop(self) -> u16 {
        match self {
            MobKind::Sheep => 33, MobKind::Cow => 287, MobKind::Pig => 272,
            MobKind::Chicken => 275, MobKind::Zombie => 288, MobKind::Skeleton => 287,
            MobKind::Creeper => 289, MobKind::Spider => 289, MobKind::Enderman => 303,
            MobKind::Villager => 269,
        }
    }
    pub fn height(self) -> f32 {
        match self { MobKind::Chicken => 0.7, MobKind::Spider => 0.9, MobKind::Enderman => 2.9, _ => 1.8 }
    }
}

#[derive(Clone)]
pub struct Mob {
    pub kind: MobKind,
    pub x: f32, pub y: f32, pub z: f32,
    pub vx: f32, pub vy: f32, pub vz: f32,
    pub yaw: f32,
    pub hp: f32,
    pub timer: f32,
    pub attack_cd: f32,
    pub hurt: f32,
    pub on_ground: bool,
}

impl Mob {
    pub fn spawn(kind: MobKind, x: f32, y: f32, z: f32) -> Self {
        Self { kind, x, y, z, vx: 0.0, vy: 0.0, vz: 0.0, yaw: 0.0, hp: kind.health(), timer: 0.0, attack_cd: 0.0, hurt: 0.0, on_ground: false }
    }

    pub fn update(&mut self, world: &mut World, px: f32, py: f32, pz: f32, dt: f32, tick: u32, day: f32) {
        self.timer -= dt;
        self.attack_cd -= dt;
        self.hurt = (self.hurt - dt * 2.0).max(0.0);
        let dx = px - self.x;
        let dz = pz - self.z;
        let dist = (dx * dx + dz * dz).sqrt();
        let speed = match self.kind { MobKind::Spider => 2.8, MobKind::Chicken => 1.0, _ => 1.5 };
        if self.timer <= 0.0 {
            self.timer = 1.5 + (tick.wrapping_mul(2654435761) % 100) as f32 / 100.0;
            if self.kind.hostile() {
                self.yaw = dz.atan2(dx);
            } else if dist < 5.0 {
                self.yaw = (-dz).atan2(-dx);
            } else {
                self.yaw = (tick as f32 * 0.13 + self.x * 0.7 + self.z * 0.3).sin() * 3.14;
            }
        }
        let mut move_speed = 0.0;
        if self.kind.hostile() {
            if dist < 16.0 && (day < 0.7 || self.kind != MobKind::Creeper) {
                move_speed = speed;
            }
        } else if dist > 2.5 || dist < 1.0 {
            move_speed = speed * 0.6;
        }
        self.vx = self.yaw.cos() * move_speed;
        self.vz = self.yaw.sin() * move_speed;
        let nx = self.x + self.vx * dt;
        let nz = self.z + self.vz * dt;
        let r = 0.3;
        if !self.blocked(world, nx, self.y, self.z, r) { self.x = nx; }
        if !self.blocked(world, self.x, self.y, nz, r) { self.z = nz; }
        self.vy -= 22.0 * dt;
        let ny = self.y + self.vy * dt;
        if self.blocked(world, self.x, ny, self.z, r) {
            if self.vy < 0.0 { self.on_ground = true; }
            self.vy = 0.0;
        } else { self.y = ny; self.on_ground = false; }
        if self.y < -5.0 { self.hp = 0.0; }
        if day > 0.75 && (self.kind == MobKind::Zombie || self.kind == MobKind::Skeleton) && tick % 120 == 0 { self.hp -= 1.0; }
    }

    fn blocked(&self, world: &mut World, x: f32, y: f32, z: f32, r: f32) -> bool {
        let h = self.kind.height();
        for dx in [-r, r] {
            for dz in [-r, r] {
                for dy in [0.05, h * 0.5, h - 0.05] {
                    if world::is_solid(world.get((x + dx).floor() as i32, (y + dy).floor() as i32, (z + dz).floor() as i32)) { return true; }
                }
            }
        }
        false
    }
}

#[derive(Clone)]
pub struct Drop { pub item: u16, pub x: f32, pub y: f32, pub z: f32, pub vy: f32, pub age: f32 }

impl Drop {
    pub fn new(x: f32, y: f32, z: f32, item: u16) -> Self {
        Drop { item, x, y, z, vy: 0.0, age: 0.0 }
    }

    pub fn update(&mut self, world: &mut World, dt: f32) {
        self.age += dt;
        self.vy -= 18.0 * dt;
        let ny = self.y + self.vy * dt;
        if world::is_solid(world.get(self.x.floor() as i32, ny.floor() as i32, self.z.floor() as i32)) {
            self.y = ny.floor() + 1.0;
            self.vy = 0.0;
        } else { self.y = ny; }
    }
}

// Procedural pixel sprites. Coordinates are 0..16 for mobs, 0..32 for Steve.
pub fn sprite_pixel(kind: MobKind, x: u32, y: u32, tick: u32, back: bool) -> [u8; 4] {
    let bob = ((tick / 8) % 2) as i32;
    match kind {
        MobKind::Sheep => animal(x, y, 245, 245, 245, 220, 180, 170, 60, 40, 40, bob, false),
        MobKind::Cow => animal(x, y, 245, 240, 230, 90, 60, 45, 60, 35, 35, bob, true),
        MobKind::Pig => animal(x, y, 245, 165, 175, 240, 150, 160, 230, 140, 150, bob, false),
        MobKind::Chicken => {
            if (x >= 5 && x <= 11 && y >= 5 && y <= 11) || (x >= 4 && x <= 12 && y >= 9 && y <= 13) {
                if x == 12 && y >= 7 && y <= 9 { return [230, 160, 40, 255]; }
                if x == 14 && y == 8 { return [220, 60, 40, 255]; }
                return [245, 245, 235, 255];
            }
            [0, 0, 0, 0]
        }
        MobKind::Zombie => humanoid(x, y, bob, 110, 165, 85, 70, 110, 160, 45, 90, 60, 30, 20, 35, back, false),
        MobKind::Skeleton => humanoid(x, y, bob, 225, 225, 215, 225, 225, 215, 225, 225, 215, 20, 20, 20, back, true),
        MobKind::Enderman => humanoid(x, y, bob, 25, 20, 35, 20, 18, 28, 20, 18, 28, 180, 80, 255, back, false),
        MobKind::Villager => humanoid(x, y, bob, 180, 140, 105, 90, 65, 135, 60, 45, 95, 40, 25, 15, back, false),
        MobKind::Creeper => {
            let green = if (x * 7 + y * 11) % 13 < 3 { [70, 135, 60] } else { [92, 165, 78] };
            if y >= 2 && y <= 13 && x >= 4 && x <= 11 {
                let face = y >= 4 && y <= 9;
                if face && ((x == 6 && y == 5) || (x == 9 && y == 5) || (x == 6 && y == 8 && y == 8) || (y == 8 && (x == 5 || x == 10))) {
                    return [20, 20, 20, 255];
                }
                if face && y == 7 && x >= 6 && x <= 9 { return [20, 20, 20, 255]; }
                return [green[0], green[1], green[2], 255];
            }
            [0, 0, 0, 0]
        }
        MobKind::Spider => {
            let body = (x >= 5 && x <= 10 && y >= 5 && y <= 11);
            let head = x >= 2 && x <= 5 && y >= 7 && y <= 10;
            let leg = (y == 4 || y == 12 || (x == 4 && y >= 3) || (x == 11 && y >= 3)) && x >= 1 && x <= 14;
            if body { return [45, 30, 28, 255]; }
            if head {
                if (y == 8 && (x == 3 || x == 4)) { return [220, 40, 30, 255]; }
                return [60, 40, 35, 255];
            }
            if leg { return [35, 22, 20, 255]; }
            [0, 0, 0, 0]
        }
    }
}

fn animal(x: u32, y: u32, b0: u8, b1: u8, b2: u8, h0: u8, h1: u8, h2: u8, l0: u8, l1: u8, l2: u8, bob: i32, spots: bool) -> [u8; 4] {
    let body = x >= 3 && x <= 12 && y >= 5 && y <= 11;
    let head = x >= 11 && x <= 15 && y >= 4 && y <= 9;
    let legs = y >= 11 && y <= 14 && (x == 4 || x == 6 || x == 9 || x == 11);
    if body {
        if spots && (x == 5 || x == 9) && (y == 6 || y == 9) { return [90, 55, 40, 255]; }
        return [b0, b1, b2, 255];
    }
    if head {
        if y == 6 && (x == 12 || x == 14) { return [20, 18, 18, 255]; }
        if y == 8 && x >= 12 && x <= 14 { return [l0, l1, l2, 255]; }
        return [h0, h1, h2, 255];
    }
    if legs { return [l0, l1, l2, 255]; }
    let _ = bob;
    [0, 0, 0, 0]
}

#[allow(clippy::too_many_arguments)]
fn humanoid(x: u32, y: u32, bob: i32, s0: u8, s1: u8, s2: u8, h0: u8, h1: u8, h2: u8, l0: u8, l1: u8, l2: u8, e0: u8, e1: u8, e2: u8, back: bool, bones: bool) -> [u8; 4] {
    let head = x >= 5 && x <= 10 && y >= 1 && y <= 6;
    let body = x >= 5 && x <= 10 && y >= 7 && y <= 11;
    let arm = (x == 3 || x == 4 || x == 11 || x == 12) && y >= 7 && y <= 11;
    let leg = ((x == 5 || x == 6) || (x == 9 || x == 10)) && y >= 12 && y <= 15;
    if head {
        if !back && ((x == 6 && y == 3) || (x == 9 && y == 3)) { return [e0, e1, e2, 255]; }
        if !back && y == 5 && x >= 6 && x <= 9 { return [e0 / 2, e1 / 2, e2 / 2, 255]; }
        return [h0, h1, h2, 255];
    }
    if body { return if bones && (x == 6 || x == 9) { [245, 245, 240, 255] } else { [s0, s1, s2, 255] }; }
    if arm { return [h0, h1, h2, 255]; }
    if leg {
        let sw = if (x < 8) { bob } else { -bob };
        if y == 15 && sw != 0 { return [0, 0, 0, 0]; }
        return [l0, l1, l2, 255];
    }
    [0, 0, 0, 0]
}

pub fn steve_pixel(x: u32, y: u32, tick: u32, back: bool) -> [u8; 4] {
    // 32x32 Steve, Minecraft proportions: head 10 rows, body 11 rows, legs 11 rows.
    let (x, y) = (x as i32, y as i32);
    let swing = ((tick as i32 / 6) % 4) - 1;
    let skin = [224, 178, 138, 255];
    let skin_dark = [198, 152, 114, 255];
    let hair = [66, 44, 30, 255];
    let hair_hi = [94, 66, 46, 255];
    let shirt = [0, 162, 170, 255];
    let shirt_hi = [52, 196, 202, 255];
    let shirt_dark = [0, 128, 136, 255];
    let jeans = [56, 68, 138, 255];
    let jeans_dark = [42, 50, 106, 255];
    let shoe = [72, 68, 64, 255];
    let eye_white = [242, 246, 250, 255];
    let iris = [72, 108, 196, 255];
    let mouth = [146, 96, 74, 255];

    // --- head (rows 0..10, columns 10..22) ---
    if (0..10).contains(&y) && (10..22).contains(&x) {
        let hairline = y < 2 || (y < 6 && (x < 12 || x > 19));
        if hairline { return if (x * 3 + y * 5) % 7 == 0 { hair_hi } else { hair }; }
        if y >= 9 { return skin_dark; }
        if !back {
            if (5..7).contains(&y) {
                if (12..15).contains(&x) || (17..20).contains(&x) {
                    return if x == 13 || x == 18 { iris } else { eye_white };
                }
            }
            if (15..17).contains(&x) && (6..8).contains(&y) { return skin_dark; }
            if (13..19).contains(&x) && y == 8 { return mouth; }
        }
        return if x > 19 { skin_dark } else { skin };
    }

    // --- torso and arms (rows 10..21) ---
    if (10..21).contains(&y) {
        if (12..20).contains(&x) {
            if y == 10 { return if back { shirt_dark } else { shirt_hi }; }
            return if x >= 18 { shirt_dark } else if (x + y) % 9 == 0 { shirt_hi } else { shirt };
        }
        let left = (8..12).contains(&x);
        let right = (20..24).contains(&x);
        if left || right {
            let off = if left { swing } else { -swing };
            let yy = y - off;
            if !(10..=21).contains(&yy) { return [0, 0, 0, 0]; }
            if yy <= 12 { return if left && x == 8 { shirt_dark } else { shirt }; }
            if yy >= 20 { return skin_dark; }
            return if left { skin } else { skin_dark };
        }
        return [0, 0, 0, 0];
    }

    // --- legs (rows 21..32) ---
    if (21..32).contains(&y) {
        let leg = if (12..16).contains(&x) { 0 } else if (16..20).contains(&x) { 1 } else { return [0, 0, 0, 0] };
        let step = if leg == 0 { swing } else { -swing };
        let yy = y + step;
        if yy >= 31 { return if leg == 0 { shoe } else { [58, 54, 50, 255] }; }
        if yy >= 30 { return jeans_dark; }
        return if x == 12 || x == 16 || x == 19 { jeans_dark } else { jeans };
    }
    [0, 0, 0, 0]
}

pub fn random_kind(n: u32, day: f32) -> MobKind {
    let hostile_chance = if day < 0.45 { 45 } else { 0 };
    if n % 100 < hostile_chance {
        match (n / 7) % 5 {
            0 => MobKind::Zombie, 1 => MobKind::Skeleton, 2 => MobKind::Creeper,
            3 => MobKind::Spider, _ => MobKind::Enderman,
        }
    } else {
        match (n / 11) % 6 {
            0 => MobKind::Sheep, 1 => MobKind::Cow, 2 => MobKind::Pig,
            3 => MobKind::Chicken, 4 => MobKind::Villager, _ => MobKind::Sheep,
        }
    }
}
