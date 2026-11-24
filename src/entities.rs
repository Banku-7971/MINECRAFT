//! 3D voxel box-model definitions and creature behaviour (Minecraft mobs across all 3 dimensions).
use crate::world::{self, Biome, Dimension, World};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum MobKind {
    Sheep,
    Cow,
    Pig,
    Chicken,
    Zombie,
    Skeleton,
    Creeper,
    Spider,
    Enderman,
    Villager,
    ZombifiedPiglin,
    Piglin,
    Ghast,
    Blaze,
    WitherSkeleton,
    Slime,
    Wolf,
    PolarBear,
    Parrot,
    EnderDragon,
    IronGolem,
    SnowGolem,
    Wither,
    Squid,
    Fox,
    Bee,
    Phantom,
    Panda,
    Allay,
    WanderingTrader,
    Guardian,
    Warden,
    Mooshroom,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FaceArt {
    None,
    Steve,
    Zombie,
    Skeleton,
    Creeper,
    Spider,
    Enderman,
    Pig,
    Cow,
    Sheep,
    Chicken,
    Villager,
    ZombifiedPiglin,
    Piglin,
    Ghast,
    Blaze,
    WitherSkeleton,
    Slime,
    Wolf,
    PolarBear,
    Parrot,
    EnderDragon,
    IronGolem,
    SnowGolem,
    Wither,
    Squid,
    Fox,
    Bee,
    Phantom,
    Panda,
    Allay,
    WanderingTrader,
    Guardian,
    Warden,
}

#[derive(Copy, Clone)]
pub struct Part3D {
    pub cx: f32,
    pub cy: f32,
    pub cz: f32,
    pub hx: f32,
    pub hy: f32,
    pub hz: f32,
    pub pivot_y: f32,
    pub swing: f32,
    pub col: [u8; 3],
    pub col_alt: [u8; 3],
    pub face: FaceArt,
}

impl MobKind {
    pub fn to_u8(self) -> u8 {
        match self {
            MobKind::Sheep => 0, MobKind::Cow => 1, MobKind::Pig => 2, MobKind::Chicken => 3,
            MobKind::Zombie => 4, MobKind::Skeleton => 5, MobKind::Creeper => 6, MobKind::Spider => 7,
            MobKind::Enderman => 8, MobKind::Villager => 9, MobKind::ZombifiedPiglin => 10,
            MobKind::Piglin => 11, MobKind::Ghast => 12, MobKind::Blaze => 13,
            MobKind::WitherSkeleton => 14, MobKind::Slime => 15, MobKind::Wolf => 16,
            MobKind::PolarBear => 17, MobKind::Parrot => 18, MobKind::EnderDragon => 19,
            MobKind::IronGolem => 20, MobKind::SnowGolem => 21,
            MobKind::Wither => 22, MobKind::Squid => 23, MobKind::Fox => 24,
            MobKind::Bee => 25, MobKind::Phantom => 26,
            MobKind::Panda => 27, MobKind::Allay => 28, MobKind::WanderingTrader => 29,
            MobKind::Guardian => 30, MobKind::Warden => 31, MobKind::Mooshroom => 32,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => MobKind::Sheep, 1 => MobKind::Cow, 2 => MobKind::Pig, 3 => MobKind::Chicken,
            4 => MobKind::Zombie, 5 => MobKind::Skeleton, 6 => MobKind::Creeper, 7 => MobKind::Spider,
            8 => MobKind::Enderman, 9 => MobKind::Villager, 10 => MobKind::ZombifiedPiglin,
            11 => MobKind::Piglin, 12 => MobKind::Ghast, 13 => MobKind::Blaze,
            14 => MobKind::WitherSkeleton, 15 => MobKind::Slime, 16 => MobKind::Wolf,
            17 => MobKind::PolarBear, 18 => MobKind::Parrot, 19 => MobKind::EnderDragon,
            20 => MobKind::IronGolem, 21 => MobKind::SnowGolem,
            22 => MobKind::Wither, 23 => MobKind::Squid, 24 => MobKind::Fox,
            25 => MobKind::Bee, 26 => MobKind::Phantom,
            27 => MobKind::Panda, 28 => MobKind::Allay, 29 => MobKind::WanderingTrader,
            30 => MobKind::Guardian, 31 => MobKind::Warden, 32 => MobKind::Mooshroom,
            _ => MobKind::Sheep,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            MobKind::Sheep => "Sheep",
            MobKind::Cow => "Cow",
            MobKind::Pig => "Pig",
            MobKind::Chicken => "Chicken",
            MobKind::Zombie => "Zombie",
            MobKind::Skeleton => "Skeleton",
            MobKind::Creeper => "Creeper",
            MobKind::Spider => "Spider",
            MobKind::Enderman => "Enderman",
            MobKind::Villager => "Villager",
            MobKind::ZombifiedPiglin => "Zombified Piglin",
            MobKind::Piglin => "Piglin",
            MobKind::Ghast => "Ghast",
            MobKind::Blaze => "Blaze",
            MobKind::WitherSkeleton => "Wither Skeleton",
            MobKind::Slime => "Slime",
            MobKind::Wolf => "Wolf",
            MobKind::PolarBear => "Polar Bear",
            MobKind::Parrot => "Parrot",
            MobKind::EnderDragon => "Ender Dragon",
            MobKind::IronGolem => "Iron Golem",
            MobKind::SnowGolem => "Snow Golem",
            MobKind::Wither => "Wither",
            MobKind::Squid => "Squid",
            MobKind::Fox => "Fox",
            MobKind::Bee => "Bee",
            MobKind::Phantom => "Phantom",
            MobKind::Panda => "Panda",
            MobKind::Allay => "Allay",
            MobKind::WanderingTrader => "Wandering Trader",
            MobKind::Guardian => "Guardian",
            MobKind::Warden => "Warden",
            MobKind::Mooshroom => "Mooshroom",
        }
    }

    pub fn hostile(self) -> bool {
        matches!(
            self,
            MobKind::Zombie | MobKind::Skeleton | MobKind::Creeper | MobKind::Spider
                | MobKind::Enderman | MobKind::Piglin | MobKind::Ghast
                | MobKind::Blaze | MobKind::WitherSkeleton | MobKind::Slime
                | MobKind::EnderDragon | MobKind::Wither | MobKind::Phantom
                | MobKind::Guardian | MobKind::Warden
        )
    }

    pub fn flying(self) -> bool {
        matches!(
            self,
            MobKind::Ghast | MobKind::Blaze | MobKind::Parrot | MobKind::EnderDragon
                | MobKind::Wither | MobKind::Bee | MobKind::Phantom | MobKind::Squid
                | MobKind::Allay | MobKind::Guardian
        )
    }

    pub fn health(self) -> f32 {
        match self {
            MobKind::Spider | MobKind::Piglin | MobKind::Slime => 16.0,
            MobKind::Enderman => 40.0,
            MobKind::PolarBear | MobKind::Guardian => 30.0,
            MobKind::Ghast | MobKind::Squid | MobKind::Bee | MobKind::Fox => 10.0,
            MobKind::EnderDragon => 200.0,
            MobKind::Wither => 300.0,
            MobKind::Warden => 500.0,
            MobKind::IronGolem => 100.0,
            MobKind::Chicken | MobKind::Parrot | MobKind::SnowGolem => 4.0,
            MobKind::Wolf => 8.0,
            MobKind::Villager | MobKind::WanderingTrader | MobKind::Zombie | MobKind::Skeleton | MobKind::Creeper
                | MobKind::ZombifiedPiglin | MobKind::Blaze | MobKind::WitherSkeleton | MobKind::Phantom
                | MobKind::Panda | MobKind::Allay => 20.0,
            _ => 10.0,
        }
    }

    pub fn drop(self) -> u16 {
        match self {
            MobKind::Sheep => world::WOOL_WHITE,
            MobKind::Cow => 273,
            MobKind::Pig => 272,
            MobKind::Chicken => 275,
            MobKind::Zombie => 288,
            MobKind::Skeleton => 287,
            MobKind::Creeper => 306,
            MobKind::Spider => 289,
            MobKind::Enderman => 302,
            MobKind::Villager => 0,
            MobKind::ZombifiedPiglin => 335, // Gold Nugget
            MobKind::Piglin => 267,          // Gold Ingot
            MobKind::Ghast => 306,           // Gunpowder
            MobKind::Blaze => 303,           // Blaze Rod
            MobKind::WitherSkeleton => 265,  // Coal
            MobKind::Slime => 307,           // Slimeball
            MobKind::Wolf => 287,
            MobKind::PolarBear => 278,       // Fish
            MobKind::Parrot => 285,          // Feather
            MobKind::EnderDragon => 304,     // Nether Star / Dragon trophy
            MobKind::IronGolem => 279,       // Iron Ingot
            MobKind::SnowGolem => world::SNOW,
            MobKind::Wither => 304,          // Nether Star
            MobKind::Squid => 278,           // Fish
            MobKind::Fox => 285,             // Feather
            MobKind::Bee => 270,             // Apple / Sweet
            MobKind::Phantom => 286,         // Leather (Membrane)
            MobKind::Panda => 282,           // Bamboo / Stick
            MobKind::Allay => 338,           // Amethyst Shard
            MobKind::WanderingTrader => 269, // Emerald
            MobKind::Guardian => 278,        // Fish / Prismarine
            MobKind::Warden => 268,          // Diamond
            MobKind::Mooshroom => 273,       // Raw Beef
        }
    }

    pub fn height(self) -> f32 {
        match self {
            MobKind::Chicken | MobKind::Parrot | MobKind::Bee | MobKind::Allay => 0.7,
            MobKind::Spider | MobKind::Pig | MobKind::Wolf | MobKind::Fox | MobKind::Phantom => 0.9,
            MobKind::Slime | MobKind::Squid | MobKind::Guardian => 1.0,
            MobKind::Panda => 1.25,
            MobKind::PolarBear => 1.4,
            MobKind::Sheep | MobKind::Cow | MobKind::Mooshroom => 1.35,
            MobKind::SnowGolem => 1.9,
            MobKind::WitherSkeleton => 2.4,
            MobKind::IronGolem => 2.7,
            MobKind::Enderman | MobKind::Warden => 2.9,
            MobKind::Wither => 3.2,
            MobKind::Ghast => 3.4,
            MobKind::EnderDragon => 3.8,
            _ => 1.8,
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            MobKind::Chicken | MobKind::Parrot | MobKind::Allay => 0.22,
            MobKind::Pig | MobKind::Wolf => 0.36,
            MobKind::Cow | MobKind::Mooshroom | MobKind::Sheep | MobKind::Villager | MobKind::WanderingTrader | MobKind::PolarBear | MobKind::Panda => 0.42,
            MobKind::Spider => 0.52,
            MobKind::IronGolem | MobKind::Warden => 0.58,
            MobKind::Slime | MobKind::Guardian => 0.46,
            _ => 0.32,
        }
    }
}

pub fn mob_collides(world: &mut World, x: f32, y: f32, z: f32, r: f32, h: f32) -> bool {
    let x0 = (x - r).floor() as i32;
    let x1 = (x + r).floor() as i32;
    let y0 = (y + 0.002).floor() as i32;
    let y1 = (y + h - 0.05).floor() as i32;
    let z0 = (z - r).floor() as i32;
    let z1 = (z + r).floor() as i32;
    for bx in x0..=x1 {
        for by in y0..=y1 {
            for bz in z0..=z1 {
                if world::is_solid(world.get(bx, by, bz)) {
                    return true;
                }
            }
        }
    }
    false
}

/// Find exact top Y of the solid ground beneath a mob's footprint `(x ± r, z ± r)`.
pub fn ground_top_at(world: &mut World, x: f32, y_probe: f32, z: f32, r: f32) -> Option<f32> {
    let x0 = (x - r).floor() as i32;
    let x1 = (x + r).floor() as i32;
    let z0 = (z - r).floor() as i32;
    let z1 = (z + r).floor() as i32;
    let by = y_probe.floor() as i32;
    let mut best: Option<i32> = None;
    for bx in x0..=x1 {
        for bz in z0..=z1 {
            if world::is_solid(world.get(bx, by, bz)) {
                best = Some(best.map_or(by, |b| b.max(by)));
            }
        }
    }
    best.map(|b| (b + 1) as f32)
}

pub fn steve_parts(walk: f32, airborne: bool, swimming: bool) -> Vec<Part3D> {
    if swimming {
        let kick = (walk * 2.1).sin() * 0.45;
        let stroke = (walk * 1.4).sin() * 0.75;
        return vec![
            Part3D { cx: 0.0, cy: 0.74, cz: 0.56, hx: 0.24, hy: 0.24, hz: 0.24, pivot_y: 0.74, swing: -0.25, col: [218, 172, 134], col_alt: [85, 55, 32], face: FaceArt::Steve },
            Part3D { cx: 0.0, cy: 0.54, cz: 0.08, hx: 0.24, hy: 0.14, hz: 0.34, pivot_y: 0.54, swing: 0.0, col: [46, 154, 174], col_alt: [36, 128, 148], face: FaceArt::None },
            Part3D { cx: -0.35, cy: 0.54, cz: 0.52, hx: 0.11, hy: 0.11, hz: 0.34, pivot_y: 0.54, swing: 1.25 + stroke, col: [215, 168, 130], col_alt: [46, 154, 174], face: FaceArt::None },
            Part3D { cx: 0.35, cy: 0.54, cz: 0.52, hx: 0.11, hy: 0.11, hz: 0.34, pivot_y: 0.54, swing: 1.25 - stroke, col: [215, 168, 130], col_alt: [46, 154, 174], face: FaceArt::None },
            Part3D { cx: -0.12, cy: 0.52, cz: -0.52, hx: 0.12, hy: 0.12, hz: 0.36, pivot_y: 0.52, swing: 1.35 + kick, col: [56, 62, 142], col_alt: [45, 50, 120], face: FaceArt::None },
            Part3D { cx: 0.12, cy: 0.52, cz: -0.52, hx: 0.12, hy: 0.12, hz: 0.36, pivot_y: 0.52, swing: 1.35 - kick, col: [56, 62, 142], col_alt: [45, 50, 120], face: FaceArt::None },
        ];
    }
    let (arm_swing, leg_swing) = if airborne {
        (0.46, 0.38)
    } else {
        let s = walk.sin() * 0.62;
        (s, s)
    };
    vec![
        Part3D { cx: 0.0, cy: 1.62, cz: 0.0, hx: 0.24, hy: 0.24, hz: 0.24, pivot_y: 1.38, swing: 0.0, col: [218, 172, 134], col_alt: [85, 55, 32], face: FaceArt::Steve },
        Part3D { cx: 0.0, cy: 1.05, cz: 0.0, hx: 0.24, hy: 0.34, hz: 0.13, pivot_y: 1.05, swing: 0.0, col: [46, 154, 174], col_alt: [36, 128, 148], face: FaceArt::None },
        Part3D { cx: -0.35, cy: 1.05, cz: 0.0, hx: 0.11, hy: 0.34, hz: 0.11, pivot_y: 1.36, swing: -arm_swing, col: [215, 168, 130], col_alt: [46, 154, 174], face: FaceArt::None },
        Part3D { cx: 0.35, cy: 1.05, cz: 0.0, hx: 0.11, hy: 0.34, hz: 0.11, pivot_y: 1.36, swing: arm_swing, col: [215, 168, 130], col_alt: [46, 154, 174], face: FaceArt::None },
        Part3D { cx: -0.12, cy: 0.36, cz: 0.0, hx: 0.12, hy: 0.36, hz: 0.12, pivot_y: 0.72, swing: leg_swing, col: [56, 62, 142], col_alt: [45, 50, 120], face: FaceArt::None },
        Part3D { cx: 0.12, cy: 0.36, cz: 0.0, hx: 0.12, hy: 0.36, hz: 0.12, pivot_y: 0.72, swing: -leg_swing, col: [56, 62, 142], col_alt: [45, 50, 120], face: FaceArt::None },
    ]
}

pub fn mob_parts(kind: MobKind, walk: f32, airborne: bool) -> Vec<Part3D> {
    let s = walk.sin() * 0.55;
    match kind {
        MobKind::Zombie => vec![
            Part3D { cx: 0.0, cy: 1.62, cz: 0.0, hx: 0.24, hy: 0.24, hz: 0.24, pivot_y: 1.38, swing: 0.0, col: [92, 144, 72], col_alt: [72, 118, 56], face: FaceArt::Zombie },
            Part3D { cx: 0.0, cy: 1.05, cz: 0.0, hx: 0.24, hy: 0.34, hz: 0.13, pivot_y: 1.05, swing: 0.0, col: [48, 136, 150], col_alt: [38, 112, 124], face: FaceArt::None },
            Part3D { cx: -0.35, cy: 1.24, cz: 0.26, hx: 0.11, hy: 0.11, hz: 0.32, pivot_y: 1.32, swing: s * 0.2, col: [92, 144, 72], col_alt: [48, 136, 150], face: FaceArt::None },
            Part3D { cx: 0.35, cy: 1.24, cz: 0.26, hx: 0.11, hy: 0.11, hz: 0.32, pivot_y: 1.32, swing: -s * 0.2, col: [92, 144, 72], col_alt: [48, 136, 150], face: FaceArt::None },
            Part3D { cx: -0.12, cy: 0.36, cz: 0.0, hx: 0.12, hy: 0.36, hz: 0.12, pivot_y: 0.72, swing: s, col: [72, 62, 134], col_alt: [55, 48, 110], face: FaceArt::None },
            Part3D { cx: 0.12, cy: 0.36, cz: 0.0, hx: 0.12, hy: 0.36, hz: 0.12, pivot_y: 0.72, swing: -s, col: [72, 62, 134], col_alt: [55, 48, 110], face: FaceArt::None },
        ],
        MobKind::ZombifiedPiglin => vec![
            Part3D { cx: 0.0, cy: 1.62, cz: 0.0, hx: 0.24, hy: 0.24, hz: 0.24, pivot_y: 1.38, swing: 0.0, col: [216, 142, 148], col_alt: [96, 148, 72], face: FaceArt::ZombifiedPiglin },
            Part3D { cx: 0.0, cy: 1.05, cz: 0.0, hx: 0.24, hy: 0.34, hz: 0.13, pivot_y: 1.05, swing: 0.0, col: [110, 74, 52], col_alt: [96, 148, 72], face: FaceArt::None },
            Part3D { cx: -0.35, cy: 1.24, cz: 0.26, hx: 0.11, hy: 0.11, hz: 0.32, pivot_y: 1.32, swing: s * 0.2, col: [216, 142, 148], col_alt: [96, 148, 72], face: FaceArt::None },
            Part3D { cx: 0.35, cy: 1.24, cz: 0.26, hx: 0.11, hy: 0.11, hz: 0.32, pivot_y: 1.32, swing: -s * 0.2, col: [216, 142, 148], col_alt: [238, 198, 62], face: FaceArt::None },
            Part3D { cx: -0.12, cy: 0.36, cz: 0.0, hx: 0.12, hy: 0.36, hz: 0.12, pivot_y: 0.72, swing: s, col: [84, 56, 38], col_alt: [66, 42, 28], face: FaceArt::None },
            Part3D { cx: 0.12, cy: 0.36, cz: 0.0, hx: 0.12, hy: 0.36, hz: 0.12, pivot_y: 0.72, swing: -s, col: [84, 56, 38], col_alt: [66, 42, 28], face: FaceArt::None },
        ],
        MobKind::Piglin => vec![
            Part3D { cx: 0.0, cy: 1.62, cz: 0.0, hx: 0.24, hy: 0.24, hz: 0.24, pivot_y: 1.38, swing: 0.0, col: [234, 164, 146], col_alt: [212, 134, 122], face: FaceArt::Piglin },
            Part3D { cx: 0.0, cy: 1.05, cz: 0.0, hx: 0.24, hy: 0.34, hz: 0.13, pivot_y: 1.05, swing: 0.0, col: [118, 66, 42], col_alt: [218, 174, 52], face: FaceArt::None },
            Part3D { cx: -0.35, cy: 1.05, cz: 0.0, hx: 0.11, hy: 0.34, hz: 0.11, pivot_y: 1.36, swing: -s, col: [234, 164, 146], col_alt: [118, 66, 42], face: FaceArt::None },
            Part3D { cx: 0.35, cy: 1.05, cz: 0.0, hx: 0.11, hy: 0.34, hz: 0.11, pivot_y: 1.36, swing: s, col: [234, 164, 146], col_alt: [238, 198, 62], face: FaceArt::None },
            Part3D { cx: -0.12, cy: 0.36, cz: 0.0, hx: 0.12, hy: 0.36, hz: 0.12, pivot_y: 0.72, swing: s, col: [82, 48, 34], col_alt: [62, 36, 26], face: FaceArt::None },
            Part3D { cx: 0.12, cy: 0.36, cz: 0.0, hx: 0.12, hy: 0.36, hz: 0.12, pivot_y: 0.72, swing: -s, col: [82, 48, 34], col_alt: [62, 36, 26], face: FaceArt::None },
        ],
        MobKind::Skeleton => vec![
            Part3D { cx: 0.0, cy: 1.62, cz: 0.0, hx: 0.24, hy: 0.24, hz: 0.24, pivot_y: 1.38, swing: 0.0, col: [222, 222, 212], col_alt: [190, 190, 180], face: FaceArt::Skeleton },
            Part3D { cx: 0.0, cy: 1.05, cz: 0.0, hx: 0.22, hy: 0.33, hz: 0.11, pivot_y: 1.05, swing: 0.0, col: [212, 212, 202], col_alt: [165, 165, 158], face: FaceArt::None },
            Part3D { cx: -0.31, cy: 1.20, cz: 0.22, hx: 0.06, hy: 0.06, hz: 0.28, pivot_y: 1.30, swing: s * 0.2, col: [216, 216, 206], col_alt: [185, 185, 175], face: FaceArt::None },
            Part3D { cx: 0.31, cy: 1.20, cz: 0.22, hx: 0.06, hy: 0.06, hz: 0.28, pivot_y: 1.30, swing: -s * 0.2, col: [216, 216, 206], col_alt: [185, 185, 175], face: FaceArt::None },
            Part3D { cx: -0.12, cy: 0.36, cz: 0.0, hx: 0.06, hy: 0.36, hz: 0.06, pivot_y: 0.72, swing: s, col: [216, 216, 206], col_alt: [185, 185, 175], face: FaceArt::None },
            Part3D { cx: 0.12, cy: 0.36, cz: 0.0, hx: 0.06, hy: 0.36, hz: 0.06, pivot_y: 0.72, swing: -s, col: [216, 216, 206], col_alt: [185, 185, 175], face: FaceArt::None },
        ],
        MobKind::WitherSkeleton => vec![
            Part3D { cx: 0.0, cy: 2.12, cz: 0.0, hx: 0.26, hy: 0.26, hz: 0.26, pivot_y: 1.86, swing: 0.0, col: [44, 42, 46], col_alt: [28, 26, 30], face: FaceArt::WitherSkeleton },
            Part3D { cx: 0.0, cy: 1.38, cz: 0.0, hx: 0.24, hy: 0.44, hz: 0.12, pivot_y: 1.38, swing: 0.0, col: [40, 38, 42], col_alt: [26, 24, 28], face: FaceArt::None },
            Part3D { cx: -0.34, cy: 1.54, cz: 0.24, hx: 0.07, hy: 0.07, hz: 0.34, pivot_y: 1.66, swing: s * 0.2, col: [42, 40, 44], col_alt: [26, 24, 28], face: FaceArt::None },
            Part3D { cx: 0.34, cy: 1.54, cz: 0.24, hx: 0.07, hy: 0.07, hz: 0.34, pivot_y: 1.66, swing: -s * 0.2, col: [42, 40, 44], col_alt: [26, 24, 28], face: FaceArt::None },
            Part3D { cx: -0.14, cy: 0.46, cz: 0.0, hx: 0.07, hy: 0.46, hz: 0.07, pivot_y: 0.92, swing: s, col: [40, 38, 42], col_alt: [26, 24, 28], face: FaceArt::None },
            Part3D { cx: 0.14, cy: 0.46, cz: 0.0, hx: 0.07, hy: 0.46, hz: 0.07, pivot_y: 0.92, swing: -s, col: [40, 38, 42], col_alt: [26, 24, 28], face: FaceArt::None },
        ],
        MobKind::Creeper => vec![
            Part3D { cx: 0.0, cy: 1.54, cz: 0.0, hx: 0.24, hy: 0.24, hz: 0.24, pivot_y: 1.30, swing: 0.0, col: [88, 176, 74], col_alt: [62, 134, 52], face: FaceArt::Creeper },
            Part3D { cx: 0.0, cy: 0.90, cz: 0.0, hx: 0.20, hy: 0.40, hz: 0.14, pivot_y: 0.90, swing: 0.0, col: [82, 166, 68], col_alt: [62, 134, 52], face: FaceArt::None },
            Part3D { cx: -0.14, cy: 0.25, cz: 0.18, hx: 0.11, hy: 0.25, hz: 0.11, pivot_y: 0.50, swing: s, col: [76, 152, 62], col_alt: [56, 122, 46], face: FaceArt::None },
            Part3D { cx: 0.14, cy: 0.25, cz: 0.18, hx: 0.11, hy: 0.25, hz: 0.11, pivot_y: 0.50, swing: -s, col: [76, 152, 62], col_alt: [56, 122, 46], face: FaceArt::None },
            Part3D { cx: -0.14, cy: 0.25, cz: -0.18, hx: 0.11, hy: 0.25, hz: 0.11, pivot_y: 0.50, swing: -s, col: [76, 152, 62], col_alt: [56, 122, 46], face: FaceArt::None },
            Part3D { cx: 0.14, cy: 0.25, cz: -0.18, hx: 0.11, hy: 0.25, hz: 0.11, pivot_y: 0.50, swing: s, col: [76, 152, 62], col_alt: [56, 122, 46], face: FaceArt::None },
        ],
        MobKind::Enderman => vec![
            Part3D { cx: 0.0, cy: 2.62, cz: 0.0, hx: 0.24, hy: 0.24, hz: 0.24, pivot_y: 2.38, swing: 0.0, col: [22, 20, 28], col_alt: [14, 12, 18], face: FaceArt::Enderman },
            Part3D { cx: 0.0, cy: 1.94, cz: 0.0, hx: 0.22, hy: 0.42, hz: 0.12, pivot_y: 1.94, swing: 0.0, col: [20, 18, 26], col_alt: [14, 12, 18], face: FaceArt::None },
            Part3D { cx: -0.30, cy: 1.56, cz: 0.0, hx: 0.06, hy: 0.72, hz: 0.06, pivot_y: 2.28, swing: -s * 0.4, col: [18, 16, 24], col_alt: [12, 10, 16], face: FaceArt::None },
            Part3D { cx: 0.30, cy: 1.56, cz: 0.0, hx: 0.06, hy: 0.72, hz: 0.06, pivot_y: 2.28, swing: s * 0.4, col: [18, 16, 24], col_alt: [12, 10, 16], face: FaceArt::None },
            Part3D { cx: -0.12, cy: 0.76, cz: 0.0, hx: 0.06, hy: 0.76, hz: 0.06, pivot_y: 1.52, swing: s * 0.4, col: [18, 16, 24], col_alt: [12, 10, 16], face: FaceArt::None },
            Part3D { cx: 0.12, cy: 0.76, cz: 0.0, hx: 0.06, hy: 0.76, hz: 0.06, pivot_y: 1.52, swing: -s * 0.4, col: [18, 16, 24], col_alt: [12, 10, 16], face: FaceArt::None },
        ],
        MobKind::Spider => vec![
            Part3D { cx: 0.0, cy: 0.44, cz: 0.36, hx: 0.24, hy: 0.20, hz: 0.22, pivot_y: 0.44, swing: 0.0, col: [48, 42, 46], col_alt: [32, 28, 32], face: FaceArt::Spider },
            // Mandibles
            Part3D { cx: -0.09, cy: 0.31, cz: 0.59, hx: 0.04, hy: 0.05, hz: 0.04, pivot_y: 0.31, swing: 0.0, col: [26, 22, 26], col_alt: [85, 22, 22], face: FaceArt::None },
            Part3D { cx: 0.09, cy: 0.31, cz: 0.59, hx: 0.04, hy: 0.05, hz: 0.04, pivot_y: 0.31, swing: 0.0, col: [26, 22, 26], col_alt: [85, 22, 22], face: FaceArt::None },
            // Thorax + Abdomen
            Part3D { cx: 0.0, cy: 0.43, cz: 0.08, hx: 0.18, hy: 0.16, hz: 0.14, pivot_y: 0.43, swing: 0.0, col: [44, 38, 42], col_alt: [30, 26, 30], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.46, cz: -0.26, hx: 0.32, hy: 0.24, hz: 0.36, pivot_y: 0.46, swing: 0.0, col: [38, 34, 38], col_alt: [26, 22, 26], face: FaceArt::None },
            // 8 legs (4 per side)
            Part3D { cx: -0.50, cy: 0.22, cz: 0.26, hx: 0.26, hy: 0.06, hz: 0.06, pivot_y: 0.32, swing: s, col: [54, 46, 50], col_alt: [34, 30, 34], face: FaceArt::None },
            Part3D { cx: 0.50, cy: 0.22, cz: 0.26, hx: 0.26, hy: 0.06, hz: 0.06, pivot_y: 0.32, swing: -s, col: [54, 46, 50], col_alt: [34, 30, 34], face: FaceArt::None },
            Part3D { cx: -0.54, cy: 0.22, cz: 0.08, hx: 0.28, hy: 0.06, hz: 0.06, pivot_y: 0.32, swing: -s, col: [54, 46, 50], col_alt: [34, 30, 34], face: FaceArt::None },
            Part3D { cx: 0.54, cy: 0.22, cz: 0.08, hx: 0.28, hy: 0.06, hz: 0.06, pivot_y: 0.32, swing: s, col: [54, 46, 50], col_alt: [34, 30, 34], face: FaceArt::None },
            Part3D { cx: -0.54, cy: 0.22, cz: -0.10, hx: 0.28, hy: 0.06, hz: 0.06, pivot_y: 0.32, swing: s, col: [54, 46, 50], col_alt: [34, 30, 34], face: FaceArt::None },
            Part3D { cx: 0.54, cy: 0.22, cz: -0.10, hx: 0.28, hy: 0.06, hz: 0.06, pivot_y: 0.32, swing: -s, col: [54, 46, 50], col_alt: [34, 30, 34], face: FaceArt::None },
            Part3D { cx: -0.48, cy: 0.22, cz: -0.28, hx: 0.26, hy: 0.06, hz: 0.06, pivot_y: 0.32, swing: -s, col: [54, 46, 50], col_alt: [34, 30, 34], face: FaceArt::None },
            Part3D { cx: 0.48, cy: 0.22, cz: -0.28, hx: 0.26, hy: 0.06, hz: 0.06, pivot_y: 0.32, swing: s, col: [54, 46, 50], col_alt: [34, 30, 34], face: FaceArt::None },
        ],
        MobKind::Ghast => {
            let mut p = vec![
                Part3D { cx: 0.0, cy: 2.1, cz: 0.0, hx: 1.05, hy: 1.05, hz: 1.05, pivot_y: 2.1, swing: 0.0, col: [242, 242, 246], col_alt: [204, 206, 214], face: FaceArt::Ghast },
            ];
            for &(tx, tz, ph) in &[(-0.7, 0.5, 0.0f32), (0.0, 0.5, 0.4), (0.7, 0.5, -0.3), (-0.5, -0.5, 0.2), (0.5, -0.5, -0.2)] {
                p.push(Part3D { cx: tx, cy: 0.65, cz: tz, hx: 0.14, hy: 0.55, hz: 0.14, pivot_y: 1.15, swing: s * 0.4 + ph * 0.3, col: [228, 228, 234], col_alt: [195, 198, 206], face: FaceArt::None });
            }
            p
        }
        MobKind::Blaze => {
            let a = walk * 0.8;
            let mut p = vec![
                Part3D { cx: 0.0, cy: 1.45, cz: 0.0, hx: 0.26, hy: 0.26, hz: 0.26, pivot_y: 1.45, swing: 0.0, col: [244, 192, 34], col_alt: [216, 128, 18], face: FaceArt::Blaze },
                Part3D { cx: 0.0, cy: 0.90, cz: 0.0, hx: 0.10, hy: 0.36, hz: 0.10, pivot_y: 0.90, swing: 0.0, col: [255, 224, 72], col_alt: [232, 134, 24], face: FaceArt::None },
            ];
            for i in 0..4 {
                let ang = a + (i as f32) * 1.57;
                p.push(Part3D {
                    cx: ang.cos() * 0.44, cy: 1.05 + (ang * 2.0).sin() * 0.15, cz: ang.sin() * 0.44,
                    hx: 0.07, hy: 0.30, hz: 0.07, pivot_y: 1.05, swing: 0.0,
                    col: [248, 204, 42], col_alt: [222, 118, 16], face: FaceArt::None,
                });
            }
            p
        }
        MobKind::Slime => vec![
            Part3D { cx: 0.0, cy: 0.50, cz: 0.0, hx: 0.48, hy: 0.48, hz: 0.48, pivot_y: 0.50, swing: 0.0, col: [112, 208, 92], col_alt: [74, 162, 58], face: FaceArt::Slime },
        ],
        MobKind::Wolf => vec![
            Part3D { cx: 0.0, cy: 0.64, cz: 0.40, hx: 0.18, hy: 0.18, hz: 0.18, pivot_y: 0.64, swing: 0.0, col: [212, 206, 198], col_alt: [164, 158, 150], face: FaceArt::Wolf },
            // Snout + pointed ears + collar + tail
            Part3D { cx: 0.0, cy: 0.58, cz: 0.62, hx: 0.09, hy: 0.07, hz: 0.08, pivot_y: 0.58, swing: 0.0, col: [188, 182, 174], col_alt: [38, 34, 34], face: FaceArt::None },
            Part3D { cx: -0.12, cy: 0.85, cz: 0.36, hx: 0.05, hy: 0.06, hz: 0.04, pivot_y: 0.85, swing: 0.0, col: [184, 178, 170], col_alt: [148, 128, 124], face: FaceArt::None },
            Part3D { cx: 0.12, cy: 0.85, cz: 0.36, hx: 0.05, hy: 0.06, hz: 0.04, pivot_y: 0.85, swing: 0.0, col: [184, 178, 170], col_alt: [148, 128, 124], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.58, cz: 0.18, hx: 0.21, hy: 0.20, hz: 0.12, pivot_y: 0.58, swing: 0.0, col: [222, 216, 208], col_alt: [176, 168, 158], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.54, cz: -0.06, hx: 0.18, hy: 0.18, hz: 0.34, pivot_y: 0.54, swing: 0.0, col: [204, 198, 190], col_alt: [158, 152, 144], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.58, cz: -0.48, hx: 0.06, hy: 0.06, hz: 0.16, pivot_y: 0.60, swing: s * 0.3, col: [194, 188, 180], col_alt: [148, 142, 134], face: FaceArt::None },
            Part3D { cx: -0.11, cy: 0.18, cz: 0.20, hx: 0.07, hy: 0.18, hz: 0.07, pivot_y: 0.36, swing: s, col: [192, 186, 178], col_alt: [148, 142, 134], face: FaceArt::None },
            Part3D { cx: 0.11, cy: 0.18, cz: 0.20, hx: 0.07, hy: 0.18, hz: 0.07, pivot_y: 0.36, swing: -s, col: [192, 186, 178], col_alt: [148, 142, 134], face: FaceArt::None },
            Part3D { cx: -0.11, cy: 0.18, cz: -0.24, hx: 0.07, hy: 0.18, hz: 0.07, pivot_y: 0.36, swing: -s, col: [192, 186, 178], col_alt: [148, 142, 134], face: FaceArt::None },
            Part3D { cx: 0.11, cy: 0.18, cz: -0.24, hx: 0.07, hy: 0.18, hz: 0.07, pivot_y: 0.36, swing: s, col: [192, 186, 178], col_alt: [148, 142, 134], face: FaceArt::None },
        ],
        MobKind::PolarBear => vec![
            Part3D { cx: 0.0, cy: 1.02, cz: 0.58, hx: 0.24, hy: 0.22, hz: 0.22, pivot_y: 1.02, swing: 0.0, col: [242, 244, 248], col_alt: [210, 218, 226], face: FaceArt::PolarBear },
            Part3D { cx: 0.0, cy: 0.94, cz: 0.82, hx: 0.12, hy: 0.09, hz: 0.08, pivot_y: 0.94, swing: 0.0, col: [232, 236, 242], col_alt: [38, 38, 42], face: FaceArt::None },
            Part3D { cx: -0.18, cy: 1.25, cz: 0.54, hx: 0.06, hy: 0.05, hz: 0.04, pivot_y: 1.25, swing: 0.0, col: [232, 236, 242], col_alt: [196, 204, 214], face: FaceArt::None },
            Part3D { cx: 0.18, cy: 1.25, cz: 0.54, hx: 0.06, hy: 0.05, hz: 0.04, pivot_y: 1.25, swing: 0.0, col: [232, 236, 242], col_alt: [196, 204, 214], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.88, cz: -0.02, hx: 0.36, hy: 0.32, hz: 0.52, pivot_y: 0.88, swing: 0.0, col: [238, 240, 244], col_alt: [208, 214, 224], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.96, cz: -0.56, hx: 0.09, hy: 0.09, hz: 0.07, pivot_y: 0.96, swing: 0.0, col: [242, 244, 248], col_alt: [210, 218, 226], face: FaceArt::None },
            Part3D { cx: -0.22, cy: 0.30, cz: 0.34, hx: 0.13, hy: 0.30, hz: 0.13, pivot_y: 0.60, swing: s, col: [230, 234, 240], col_alt: [198, 206, 216], face: FaceArt::None },
            Part3D { cx: 0.22, cy: 0.30, cz: 0.34, hx: 0.13, hy: 0.30, hz: 0.13, pivot_y: 0.60, swing: -s, col: [230, 234, 240], col_alt: [198, 206, 216], face: FaceArt::None },
            Part3D { cx: -0.22, cy: 0.30, cz: -0.36, hx: 0.13, hy: 0.30, hz: 0.13, pivot_y: 0.60, swing: -s, col: [230, 234, 240], col_alt: [198, 206, 216], face: FaceArt::None },
            Part3D { cx: 0.22, cy: 0.30, cz: -0.36, hx: 0.13, hy: 0.30, hz: 0.13, pivot_y: 0.60, swing: s, col: [230, 234, 240], col_alt: [198, 206, 216], face: FaceArt::None },
        ],
        MobKind::Parrot => vec![
            Part3D { cx: 0.0, cy: 0.62, cz: 0.08, hx: 0.11, hy: 0.12, hz: 0.11, pivot_y: 0.62, swing: 0.0, col: [226, 44, 38], col_alt: [32, 134, 218], face: FaceArt::Parrot },
            Part3D { cx: 0.0, cy: 0.40, cz: 0.0, hx: 0.11, hy: 0.16, hz: 0.11, pivot_y: 0.40, swing: 0.0, col: [32, 134, 218], col_alt: [238, 202, 42], face: FaceArt::None },
            Part3D { cx: -0.14, cy: 0.42, cz: 0.0, hx: 0.04, hy: 0.14, hz: 0.09, pivot_y: 0.54, swing: s * 0.7, col: [238, 202, 42], col_alt: [44, 182, 72], face: FaceArt::None },
            Part3D { cx: 0.14, cy: 0.42, cz: 0.0, hx: 0.04, hy: 0.14, hz: 0.09, pivot_y: 0.54, swing: -s * 0.7, col: [238, 202, 42], col_alt: [44, 182, 72], face: FaceArt::None },
        ],
        MobKind::EnderDragon => {
            let flap = (walk * 0.7).sin() * 0.45;
            vec![
                Part3D { cx: 0.0, cy: 2.2, cz: 1.55, hx: 0.46, hy: 0.40, hz: 0.55, pivot_y: 2.2, swing: 0.0, col: [28, 24, 34], col_alt: [194, 82, 238], face: FaceArt::EnderDragon },
                Part3D { cx: 0.0, cy: 2.0, cz: 0.0, hx: 0.65, hy: 0.55, hz: 1.20, pivot_y: 2.0, swing: 0.0, col: [24, 20, 30], col_alt: [62, 58, 72], face: FaceArt::None },
                Part3D { cx: 0.0, cy: 1.9, cz: -1.8, hx: 0.24, hy: 0.24, hz: 0.90, pivot_y: 1.9, swing: 0.0, col: [26, 22, 32], col_alt: [62, 58, 72], face: FaceArt::None },
                Part3D { cx: -1.75, cy: 2.35 + flap * 0.5, cz: 0.1, hx: 1.25, hy: 0.08, hz: 0.75, pivot_y: 2.3, swing: flap, col: [20, 18, 26], col_alt: [48, 44, 58], face: FaceArt::None },
                Part3D { cx: 1.75, cy: 2.35 + flap * 0.5, cz: 0.1, hx: 1.25, hy: 0.08, hz: 0.75, pivot_y: 2.3, swing: -flap, col: [20, 18, 26], col_alt: [48, 44, 58], face: FaceArt::None },
            ]
        }
        MobKind::Pig => vec![
            Part3D { cx: 0.0, cy: 0.68, cz: 0.42, hx: 0.22, hy: 0.20, hz: 0.20, pivot_y: 0.68, swing: 0.0, col: [242, 166, 174], col_alt: [215, 135, 146], face: FaceArt::Pig },
            // Floppy ears + protruding snout + curly tail + hooves
            Part3D { cx: -0.22, cy: 0.77, cz: 0.40, hx: 0.04, hy: 0.07, hz: 0.07, pivot_y: 0.77, swing: 0.0, col: [232, 148, 158], col_alt: [200, 120, 132], face: FaceArt::None },
            Part3D { cx: 0.22, cy: 0.77, cz: 0.40, hx: 0.04, hy: 0.07, hz: 0.07, pivot_y: 0.77, swing: 0.0, col: [232, 148, 158], col_alt: [200, 120, 132], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.60, cz: 0.65, hx: 0.11, hy: 0.08, hz: 0.06, pivot_y: 0.60, swing: 0.0, col: [246, 142, 156], col_alt: [175, 85, 102], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.58, cz: -0.02, hx: 0.26, hy: 0.22, hz: 0.38, pivot_y: 0.58, swing: 0.0, col: [236, 156, 166], col_alt: [210, 130, 142], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.68, cz: -0.43, hx: 0.04, hy: 0.06, hz: 0.05, pivot_y: 0.68, swing: 0.0, col: [242, 148, 162], col_alt: [205, 118, 134], face: FaceArt::None },
            Part3D { cx: -0.15, cy: 0.18, cz: 0.22, hx: 0.09, hy: 0.18, hz: 0.09, pivot_y: 0.36, swing: s, col: [228, 148, 158], col_alt: [195, 115, 128], face: FaceArt::None },
            Part3D { cx: 0.15, cy: 0.18, cz: 0.22, hx: 0.09, hy: 0.18, hz: 0.09, pivot_y: 0.36, swing: -s, col: [228, 148, 158], col_alt: [195, 115, 128], face: FaceArt::None },
            Part3D { cx: -0.15, cy: 0.18, cz: -0.24, hx: 0.09, hy: 0.18, hz: 0.09, pivot_y: 0.36, swing: -s, col: [228, 148, 158], col_alt: [195, 115, 128], face: FaceArt::None },
            Part3D { cx: 0.15, cy: 0.18, cz: -0.24, hx: 0.09, hy: 0.18, hz: 0.09, pivot_y: 0.36, swing: s, col: [228, 148, 158], col_alt: [195, 115, 128], face: FaceArt::None },
        ],
        MobKind::Cow => vec![
            Part3D { cx: 0.0, cy: 1.06, cz: 0.48, hx: 0.22, hy: 0.22, hz: 0.18, pivot_y: 1.06, swing: 0.0, col: [82, 58, 42], col_alt: [232, 232, 228], face: FaceArt::Cow },
            // Horns + ears + snout + udder + tail
            Part3D { cx: -0.22, cy: 1.29, cz: 0.48, hx: 0.04, hy: 0.08, hz: 0.04, pivot_y: 1.29, swing: 0.0, col: [216, 206, 186], col_alt: [176, 166, 148], face: FaceArt::None },
            Part3D { cx: 0.22, cy: 1.29, cz: 0.48, hx: 0.04, hy: 0.08, hz: 0.04, pivot_y: 1.29, swing: 0.0, col: [216, 206, 186], col_alt: [176, 166, 148], face: FaceArt::None },
            Part3D { cx: -0.25, cy: 1.14, cz: 0.44, hx: 0.05, hy: 0.04, hz: 0.06, pivot_y: 1.14, swing: 0.0, col: [72, 50, 36], col_alt: [215, 185, 185], face: FaceArt::None },
            Part3D { cx: 0.25, cy: 1.14, cz: 0.44, hx: 0.05, hy: 0.04, hz: 0.06, pivot_y: 1.14, swing: 0.0, col: [72, 50, 36], col_alt: [215, 185, 185], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.90, cz: -0.04, hx: 0.30, hy: 0.28, hz: 0.44, pivot_y: 0.90, swing: 0.0, col: [76, 54, 38], col_alt: [228, 228, 224], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.58, cz: -0.22, hx: 0.13, hy: 0.06, hz: 0.14, pivot_y: 0.58, swing: 0.0, col: [236, 174, 186], col_alt: [208, 144, 158], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.86, cz: -0.50, hx: 0.04, hy: 0.22, hz: 0.04, pivot_y: 1.06, swing: s * 0.25, col: [64, 44, 32], col_alt: [42, 30, 22], face: FaceArt::None },
            Part3D { cx: -0.18, cy: 0.31, cz: 0.28, hx: 0.10, hy: 0.31, hz: 0.10, pivot_y: 0.62, swing: s, col: [68, 48, 34], col_alt: [215, 215, 210], face: FaceArt::None },
            Part3D { cx: 0.18, cy: 0.31, cz: 0.28, hx: 0.10, hy: 0.31, hz: 0.10, pivot_y: 0.62, swing: -s, col: [68, 48, 34], col_alt: [215, 215, 210], face: FaceArt::None },
            Part3D { cx: -0.18, cy: 0.31, cz: -0.32, hx: 0.10, hy: 0.31, hz: 0.10, pivot_y: 0.62, swing: -s, col: [68, 48, 34], col_alt: [215, 215, 210], face: FaceArt::None },
            Part3D { cx: 0.18, cy: 0.31, cz: -0.32, hx: 0.10, hy: 0.31, hz: 0.10, pivot_y: 0.62, swing: s, col: [68, 48, 34], col_alt: [215, 215, 210], face: FaceArt::None },
        ],
        MobKind::Mooshroom => vec![
            Part3D { cx: 0.0, cy: 1.06, cz: 0.48, hx: 0.22, hy: 0.22, hz: 0.18, pivot_y: 1.06, swing: 0.0, col: [162, 34, 30], col_alt: [236, 232, 224], face: FaceArt::Cow },
            Part3D { cx: -0.22, cy: 1.29, cz: 0.48, hx: 0.04, hy: 0.08, hz: 0.04, pivot_y: 1.29, swing: 0.0, col: [216, 206, 186], col_alt: [176, 166, 148], face: FaceArt::None },
            Part3D { cx: 0.22, cy: 1.29, cz: 0.48, hx: 0.04, hy: 0.08, hz: 0.04, pivot_y: 1.29, swing: 0.0, col: [216, 206, 186], col_alt: [176, 166, 148], face: FaceArt::None },
            // Red mushrooms sprouting from head & back
            Part3D { cx: -0.04, cy: 1.34, cz: 0.48, hx: 0.03, hy: 0.06, hz: 0.03, pivot_y: 1.34, swing: 0.0, col: [216, 208, 192], col_alt: [190, 180, 164], face: FaceArt::None },
            Part3D { cx: -0.04, cy: 1.42, cz: 0.48, hx: 0.09, hy: 0.05, hz: 0.09, pivot_y: 1.42, swing: 0.0, col: [196, 36, 32], col_alt: [238, 232, 224], face: FaceArt::None },
            Part3D { cx: 0.08, cy: 1.24, cz: 0.06, hx: 0.03, hy: 0.06, hz: 0.03, pivot_y: 1.24, swing: 0.0, col: [216, 208, 192], col_alt: [190, 180, 164], face: FaceArt::None },
            Part3D { cx: 0.08, cy: 1.32, cz: 0.06, hx: 0.09, hy: 0.05, hz: 0.09, pivot_y: 1.32, swing: 0.0, col: [196, 36, 32], col_alt: [238, 232, 224], face: FaceArt::None },
            Part3D { cx: -0.10, cy: 1.24, cz: -0.22, hx: 0.03, hy: 0.06, hz: 0.03, pivot_y: 1.24, swing: 0.0, col: [216, 208, 192], col_alt: [190, 180, 164], face: FaceArt::None },
            Part3D { cx: -0.10, cy: 1.32, cz: -0.22, hx: 0.09, hy: 0.05, hz: 0.09, pivot_y: 1.32, swing: 0.0, col: [196, 36, 32], col_alt: [238, 232, 224], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.90, cz: -0.04, hx: 0.30, hy: 0.28, hz: 0.44, pivot_y: 0.90, swing: 0.0, col: [158, 32, 28], col_alt: [234, 230, 222], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.58, cz: -0.22, hx: 0.13, hy: 0.06, hz: 0.14, pivot_y: 0.58, swing: 0.0, col: [236, 174, 186], col_alt: [208, 144, 158], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.86, cz: -0.50, hx: 0.04, hy: 0.22, hz: 0.04, pivot_y: 1.06, swing: s * 0.25, col: [132, 26, 22], col_alt: [90, 18, 16], face: FaceArt::None },
            Part3D { cx: -0.18, cy: 0.31, cz: 0.28, hx: 0.10, hy: 0.31, hz: 0.10, pivot_y: 0.62, swing: s, col: [148, 30, 26], col_alt: [228, 224, 218], face: FaceArt::None },
            Part3D { cx: 0.18, cy: 0.31, cz: 0.28, hx: 0.10, hy: 0.31, hz: 0.10, pivot_y: 0.62, swing: -s, col: [148, 30, 26], col_alt: [228, 224, 218], face: FaceArt::None },
            Part3D { cx: -0.18, cy: 0.31, cz: -0.32, hx: 0.10, hy: 0.31, hz: 0.10, pivot_y: 0.62, swing: -s, col: [148, 30, 26], col_alt: [228, 224, 218], face: FaceArt::None },
            Part3D { cx: 0.18, cy: 0.31, cz: -0.32, hx: 0.10, hy: 0.31, hz: 0.10, pivot_y: 0.62, swing: s, col: [148, 30, 26], col_alt: [228, 224, 218], face: FaceArt::None },
        ],
        MobKind::Sheep => vec![
            Part3D { cx: 0.0, cy: 0.98, cz: 0.44, hx: 0.20, hy: 0.19, hz: 0.20, pivot_y: 0.98, swing: 0.0, col: [228, 198, 174], col_alt: [238, 238, 232], face: FaceArt::Sheep },
            // Fluffy wool hood + ears + thick wool coat + tail
            Part3D { cx: 0.0, cy: 1.15, cz: 0.38, hx: 0.22, hy: 0.06, hz: 0.18, pivot_y: 1.15, swing: 0.0, col: [242, 242, 236], col_alt: [220, 220, 214], face: FaceArt::None },
            Part3D { cx: -0.23, cy: 1.02, cz: 0.42, hx: 0.04, hy: 0.06, hz: 0.06, pivot_y: 1.02, swing: 0.0, col: [218, 188, 164], col_alt: [190, 160, 140], face: FaceArt::None },
            Part3D { cx: 0.23, cy: 1.02, cz: 0.42, hx: 0.04, hy: 0.06, hz: 0.06, pivot_y: 1.02, swing: 0.0, col: [218, 188, 164], col_alt: [190, 160, 140], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.86, cz: -0.02, hx: 0.33, hy: 0.29, hz: 0.43, pivot_y: 0.86, swing: 0.0, col: [242, 242, 236], col_alt: [218, 218, 212], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.92, cz: -0.46, hx: 0.08, hy: 0.08, hz: 0.06, pivot_y: 0.92, swing: 0.0, col: [242, 242, 236], col_alt: [218, 218, 212], face: FaceArt::None },
            Part3D { cx: -0.16, cy: 0.29, cz: 0.24, hx: 0.10, hy: 0.29, hz: 0.10, pivot_y: 0.58, swing: s, col: [218, 190, 168], col_alt: [240, 240, 234], face: FaceArt::None },
            Part3D { cx: 0.16, cy: 0.29, cz: 0.24, hx: 0.10, hy: 0.29, hz: 0.10, pivot_y: 0.58, swing: -s, col: [218, 190, 168], col_alt: [240, 240, 234], face: FaceArt::None },
            Part3D { cx: -0.16, cy: 0.29, cz: -0.26, hx: 0.10, hy: 0.29, hz: 0.10, pivot_y: 0.58, swing: -s, col: [218, 190, 168], col_alt: [240, 240, 234], face: FaceArt::None },
            Part3D { cx: 0.16, cy: 0.29, cz: -0.26, hx: 0.10, hy: 0.29, hz: 0.10, pivot_y: 0.58, swing: s, col: [218, 190, 168], col_alt: [240, 240, 234], face: FaceArt::None },
        ],
        MobKind::Chicken => {
            let flap = if airborne { (walk * 1.4).sin() } else { 0.0 };
            let (wx, wy, whx, why) = if airborne {
                (0.25, 0.50 + flap * 0.09, 0.11, 0.03)
            } else {
                (0.19, 0.42, 0.03, 0.11)
            };
            vec![
                Part3D { cx: 0.0, cy: 0.58, cz: 0.18, hx: 0.12, hy: 0.14, hz: 0.10, pivot_y: 0.58, swing: 0.0, col: [244, 244, 240], col_alt: [235, 185, 50], face: FaceArt::Chicken },
                // Red comb + orange beak + red wattle + wings + tail + feet
                Part3D { cx: 0.0, cy: 0.74, cz: 0.19, hx: 0.03, hy: 0.04, hz: 0.07, pivot_y: 0.74, swing: 0.0, col: [214, 38, 38], col_alt: [174, 28, 28], face: FaceArt::None },
                Part3D { cx: 0.0, cy: 0.54, cz: 0.30, hx: 0.08, hy: 0.04, hz: 0.06, pivot_y: 0.54, swing: 0.0, col: [238, 186, 48], col_alt: [210, 148, 30], face: FaceArt::None },
                Part3D { cx: 0.0, cy: 0.46, cz: 0.28, hx: 0.04, hy: 0.05, hz: 0.03, pivot_y: 0.46, swing: 0.0, col: [212, 42, 42], col_alt: [176, 30, 30], face: FaceArt::None },
                Part3D { cx: 0.0, cy: 0.40, cz: -0.02, hx: 0.17, hy: 0.16, hz: 0.22, pivot_y: 0.40, swing: 0.0, col: [236, 236, 232], col_alt: [212, 212, 208], face: FaceArt::None },
                Part3D { cx: -wx, cy: wy, cz: -0.02, hx: whx, hy: why, hz: 0.15, pivot_y: 0.50, swing: if airborne { flap * 0.45 } else { 0.0 }, col: [246, 246, 242], col_alt: [216, 216, 212], face: FaceArt::None },
                Part3D { cx: wx, cy: wy, cz: -0.02, hx: whx, hy: why, hz: 0.15, pivot_y: 0.50, swing: if airborne { flap * 0.45 } else { 0.0 }, col: [246, 246, 242], col_alt: [216, 216, 212], face: FaceArt::None },
                Part3D { cx: 0.0, cy: 0.50, cz: -0.25, hx: 0.07, hy: 0.09, hz: 0.06, pivot_y: 0.50, swing: 0.0, col: [228, 228, 224], col_alt: [198, 198, 194], face: FaceArt::None },
                Part3D { cx: -0.09, cy: 0.12, cz: 0.0, hx: 0.04, hy: 0.12, hz: 0.04, pivot_y: 0.24, swing: s, col: [235, 185, 50], col_alt: [200, 150, 35], face: FaceArt::None },
                Part3D { cx: 0.09, cy: 0.12, cz: 0.0, hx: 0.04, hy: 0.12, hz: 0.04, pivot_y: 0.24, swing: -s, col: [235, 185, 50], col_alt: [200, 150, 35], face: FaceArt::None },
            ]
        }
        MobKind::Villager => vec![
            Part3D { cx: 0.0, cy: 1.66, cz: 0.0, hx: 0.23, hy: 0.28, hz: 0.23, pivot_y: 1.38, swing: 0.0, col: [210, 162, 126], col_alt: [95, 62, 42], face: FaceArt::Villager },
            Part3D { cx: 0.0, cy: 1.54, cz: 0.25, hx: 0.07, hy: 0.09, hz: 0.06, pivot_y: 1.54, swing: 0.0, col: [195, 148, 114], col_alt: [175, 130, 98], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.88, cz: 0.0, hx: 0.26, hy: 0.50, hz: 0.18, pivot_y: 0.88, swing: 0.0, col: [122, 82, 52], col_alt: [96, 64, 40], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 1.04, cz: 0.16, hx: 0.24, hy: 0.12, hz: 0.12, pivot_y: 1.04, swing: 0.0, col: [134, 92, 60], col_alt: [106, 72, 46], face: FaceArt::None },
            Part3D { cx: -0.12, cy: 0.22, cz: 0.0, hx: 0.11, hy: 0.22, hz: 0.11, pivot_y: 0.44, swing: s * 0.6, col: [96, 64, 40], col_alt: [72, 48, 30], face: FaceArt::None },
            Part3D { cx: 0.12, cy: 0.22, cz: 0.0, hx: 0.11, hy: 0.22, hz: 0.11, pivot_y: 0.44, swing: -s * 0.6, col: [96, 64, 40], col_alt: [72, 48, 30], face: FaceArt::None },
        ],
        MobKind::IronGolem => vec![
            // Head + villager nose
            Part3D { cx: 0.0, cy: 2.38, cz: 0.08, hx: 0.26, hy: 0.30, hz: 0.26, pivot_y: 2.08, swing: 0.0, col: [212, 210, 204], col_alt: [106, 146, 74], face: FaceArt::IronGolem },
            Part3D { cx: 0.0, cy: 2.24, cz: 0.36, hx: 0.07, hy: 0.11, hz: 0.06, pivot_y: 2.24, swing: 0.0, col: [194, 192, 186], col_alt: [168, 166, 160], face: FaceArt::None },
            // Broad upper chest + waist
            Part3D { cx: 0.0, cy: 1.66, cz: 0.0, hx: 0.56, hy: 0.38, hz: 0.32, pivot_y: 1.66, swing: 0.0, col: [222, 220, 214], col_alt: [94, 138, 64], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 1.16, cz: 0.0, hx: 0.28, hy: 0.16, hz: 0.20, pivot_y: 1.16, swing: 0.0, col: [188, 186, 180], col_alt: [158, 156, 150], face: FaceArt::None },
            // Heavy hanging arms
            Part3D { cx: -0.72, cy: 1.24, cz: 0.0, hx: 0.14, hy: 0.72, hz: 0.16, pivot_y: 1.94, swing: -s * 0.85, col: [208, 206, 200], col_alt: [96, 140, 68], face: FaceArt::None },
            Part3D { cx: 0.72, cy: 1.24, cz: 0.0, hx: 0.14, hy: 0.72, hz: 0.16, pivot_y: 1.94, swing: s * 0.85, col: [208, 206, 200], col_alt: [96, 140, 68], face: FaceArt::None },
            // Thick legs
            Part3D { cx: -0.22, cy: 0.50, cz: 0.0, hx: 0.17, hy: 0.50, hz: 0.17, pivot_y: 1.00, swing: s * 0.7, col: [194, 192, 186], col_alt: [92, 134, 64], face: FaceArt::None },
            Part3D { cx: 0.22, cy: 0.50, cz: 0.0, hx: 0.17, hy: 0.50, hz: 0.17, pivot_y: 1.00, swing: -s * 0.7, col: [194, 192, 186], col_alt: [92, 134, 64], face: FaceArt::None },
        ],
        MobKind::SnowGolem => vec![
            Part3D { cx: 0.0, cy: 1.56, cz: 0.0, hx: 0.26, hy: 0.26, hz: 0.26, pivot_y: 1.30, swing: 0.0, col: [222, 122, 28], col_alt: [168, 82, 14], face: FaceArt::SnowGolem },
            Part3D { cx: 0.0, cy: 1.02, cz: 0.0, hx: 0.28, hy: 0.28, hz: 0.28, pivot_y: 1.02, swing: 0.0, col: [242, 248, 252], col_alt: [218, 230, 240], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.40, cz: 0.0, hx: 0.36, hy: 0.40, hz: 0.36, pivot_y: 0.40, swing: 0.0, col: [236, 244, 248], col_alt: [210, 224, 236], face: FaceArt::None },
            Part3D { cx: -0.44, cy: 1.08, cz: 0.08, hx: 0.20, hy: 0.05, hz: 0.05, pivot_y: 1.12, swing: -s * 0.3, col: [112, 76, 44], col_alt: [86, 56, 32], face: FaceArt::None },
            Part3D { cx: 0.44, cy: 1.08, cz: 0.08, hx: 0.20, hy: 0.05, hz: 0.05, pivot_y: 1.12, swing: s * 0.3, col: [112, 76, 44], col_alt: [86, 56, 32], face: FaceArt::None },
        ],
        MobKind::Wither => {
            let hover = (walk * 0.5).sin() * 0.14;
            vec![
                // Center boss head + two flanking heads + bone ribcage + spine tail
                Part3D { cx: 0.0, cy: 2.65 + hover, cz: 0.08, hx: 0.28, hy: 0.28, hz: 0.28, pivot_y: 2.37, swing: 0.0, col: [34, 34, 40], col_alt: [120, 175, 255], face: FaceArt::Wither },
                Part3D { cx: -0.64, cy: 2.45 + hover, cz: 0.02, hx: 0.20, hy: 0.20, hz: 0.20, pivot_y: 2.25, swing: 0.0, col: [32, 32, 38], col_alt: [105, 165, 250], face: FaceArt::Wither },
                Part3D { cx: 0.64, cy: 2.45 + hover, cz: 0.02, hx: 0.20, hy: 0.20, hz: 0.20, pivot_y: 2.25, swing: 0.0, col: [32, 32, 38], col_alt: [105, 165, 250], face: FaceArt::Wither },
                Part3D { cx: 0.0, cy: 2.18 + hover, cz: 0.0, hx: 0.58, hy: 0.10, hz: 0.12, pivot_y: 2.18, swing: 0.0, col: [42, 42, 48], col_alt: [28, 28, 34], face: FaceArt::None },
                Part3D { cx: 0.0, cy: 1.56 + hover, cz: 0.0, hx: 0.16, hy: 0.58, hz: 0.14, pivot_y: 1.56, swing: 0.0, col: [36, 36, 42], col_alt: [24, 24, 30], face: FaceArt::None },
                Part3D { cx: 0.0, cy: 1.72 + hover, cz: 0.0, hx: 0.42, hy: 0.08, hz: 0.18, pivot_y: 1.72, swing: 0.0, col: [48, 48, 56], col_alt: [30, 30, 36], face: FaceArt::None },
                Part3D { cx: 0.0, cy: 1.42 + hover, cz: 0.0, hx: 0.36, hy: 0.08, hz: 0.16, pivot_y: 1.42, swing: 0.0, col: [48, 48, 56], col_alt: [30, 30, 36], face: FaceArt::None },
                Part3D { cx: 0.0, cy: 0.85 + hover, cz: -0.14, hx: 0.12, hy: 0.38, hz: 0.12, pivot_y: 1.15, swing: s * 0.25, col: [32, 32, 38], col_alt: [22, 22, 26], face: FaceArt::None },
            ]
        }
        MobKind::Squid => vec![
            Part3D { cx: 0.0, cy: 0.82, cz: 0.0, hx: 0.30, hy: 0.38, hz: 0.30, pivot_y: 0.82, swing: 0.0, col: [44, 62, 108], col_alt: [32, 46, 86], face: FaceArt::Squid },
            Part3D { cx: -0.16, cy: 0.28, cz: -0.16, hx: 0.05, hy: 0.24, hz: 0.05, pivot_y: 0.50, swing: s * 0.45, col: [36, 52, 94], col_alt: [26, 38, 74], face: FaceArt::None },
            Part3D { cx: 0.16, cy: 0.28, cz: -0.16, hx: 0.05, hy: 0.24, hz: 0.05, pivot_y: 0.50, swing: -s * 0.45, col: [36, 52, 94], col_alt: [26, 38, 74], face: FaceArt::None },
            Part3D { cx: -0.16, cy: 0.28, cz: 0.16, hx: 0.05, hy: 0.24, hz: 0.05, pivot_y: 0.50, swing: -s * 0.45, col: [36, 52, 94], col_alt: [26, 38, 74], face: FaceArt::None },
            Part3D { cx: 0.16, cy: 0.28, cz: 0.16, hx: 0.05, hy: 0.24, hz: 0.05, pivot_y: 0.50, swing: s * 0.45, col: [36, 52, 94], col_alt: [26, 38, 74], face: FaceArt::None },
        ],
        MobKind::Fox => vec![
            Part3D { cx: 0.0, cy: 0.44, cz: 0.0, hx: 0.20, hy: 0.18, hz: 0.38, pivot_y: 0.44, swing: 0.0, col: [216, 106, 36], col_alt: [242, 236, 224], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.52, cz: 0.46, hx: 0.19, hy: 0.16, hz: 0.18, pivot_y: 0.44, swing: 0.0, col: [222, 114, 40], col_alt: [28, 26, 28], face: FaceArt::Fox },
            Part3D { cx: 0.0, cy: 0.46, cz: 0.68, hx: 0.08, hy: 0.06, hz: 0.09, pivot_y: 0.44, swing: 0.0, col: [245, 240, 230], col_alt: [24, 24, 28], face: FaceArt::None },
            Part3D { cx: -0.12, cy: 0.72, cz: 0.44, hx: 0.05, hy: 0.07, hz: 0.04, pivot_y: 0.65, swing: 0.0, col: [44, 34, 30], col_alt: [216, 106, 36], face: FaceArt::None },
            Part3D { cx: 0.12, cy: 0.72, cz: 0.44, hx: 0.05, hy: 0.07, hz: 0.04, pivot_y: 0.65, swing: 0.0, col: [44, 34, 30], col_alt: [216, 106, 36], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.42, cz: -0.58, hx: 0.12, hy: 0.11, hz: 0.26, pivot_y: 0.44, swing: s * 0.25, col: [242, 238, 228], col_alt: [216, 106, 36], face: FaceArt::None },
            Part3D { cx: -0.12, cy: 0.16, cz: 0.24, hx: 0.06, hy: 0.16, hz: 0.06, pivot_y: 0.32, swing: s, col: [44, 34, 30], col_alt: [30, 24, 22], face: FaceArt::None },
            Part3D { cx: 0.12, cy: 0.16, cz: 0.24, hx: 0.06, hy: 0.16, hz: 0.06, pivot_y: 0.32, swing: -s, col: [44, 34, 30], col_alt: [30, 24, 22], face: FaceArt::None },
            Part3D { cx: -0.12, cy: 0.16, cz: -0.24, hx: 0.06, hy: 0.16, hz: 0.06, pivot_y: 0.32, swing: -s, col: [44, 34, 30], col_alt: [30, 24, 22], face: FaceArt::None },
            Part3D { cx: 0.12, cy: 0.16, cz: -0.24, hx: 0.06, hy: 0.16, hz: 0.06, pivot_y: 0.32, swing: s, col: [44, 34, 30], col_alt: [30, 24, 22], face: FaceArt::None },
        ],
        MobKind::Bee => {
            let flap = if airborne { (walk * 3.5).sin() * 0.65 } else { 0.0 };
            vec![
                Part3D { cx: 0.0, cy: 0.48, cz: 0.0, hx: 0.20, hy: 0.20, hz: 0.28, pivot_y: 0.48, swing: 0.0, col: [244, 198, 42], col_alt: [44, 34, 28], face: FaceArt::Bee },
                Part3D { cx: -0.26, cy: 0.68, cz: 0.0, hx: 0.18, hy: 0.03, hz: 0.14, pivot_y: 0.66, swing: flap, col: [224, 242, 250], col_alt: [190, 215, 232], face: FaceArt::None },
                Part3D { cx: 0.26, cy: 0.68, cz: 0.0, hx: 0.18, hy: 0.03, hz: 0.14, pivot_y: 0.66, swing: -flap, col: [224, 242, 250], col_alt: [190, 215, 232], face: FaceArt::None },
            ]
        }
        MobKind::Phantom => {
            let flap = (walk * 1.4).sin() * 0.45;
            vec![
                Part3D { cx: 0.0, cy: 0.56, cz: 0.34, hx: 0.20, hy: 0.12, hz: 0.18, pivot_y: 0.56, swing: 0.0, col: [66, 78, 126], col_alt: [110, 248, 86], face: FaceArt::Phantom },
                Part3D { cx: 0.0, cy: 0.54, cz: -0.04, hx: 0.22, hy: 0.10, hz: 0.32, pivot_y: 0.54, swing: 0.0, col: [72, 86, 138], col_alt: [198, 194, 180], face: FaceArt::None },
                Part3D { cx: -0.58, cy: 0.54, cz: 0.0, hx: 0.38, hy: 0.04, hz: 0.26, pivot_y: 0.54, swing: flap, col: [58, 70, 118], col_alt: [44, 54, 96], face: FaceArt::None },
                Part3D { cx: 0.58, cy: 0.54, cz: 0.0, hx: 0.38, hy: 0.04, hz: 0.26, pivot_y: 0.54, swing: -flap, col: [58, 70, 118], col_alt: [44, 54, 96], face: FaceArt::None },
                Part3D { cx: 0.0, cy: 0.52, cz: -0.54, hx: 0.07, hy: 0.05, hz: 0.26, pivot_y: 0.52, swing: -flap * 0.5, col: [64, 76, 124], col_alt: [46, 56, 98], face: FaceArt::None },
            ]
        }
        MobKind::Panda => vec![
            // White rear torso (-0.44..+0.22)
            Part3D { cx: 0.0, cy: 0.56, cz: -0.11, hx: 0.34, hy: 0.25, hz: 0.33, pivot_y: 0.56, swing: 0.0, col: [238, 238, 236], col_alt: [30, 30, 34], face: FaceArt::None },
            // Black chest/shoulder band cleanly in front of rear torso (0.22..0.42)
            Part3D { cx: 0.0, cy: 0.56, cz: 0.32, hx: 0.34, hy: 0.25, hz: 0.10, pivot_y: 0.56, swing: 0.0, col: [32, 32, 36], col_alt: [24, 24, 28], face: FaceArt::None },
            // White head cleanly forward of chest (0.42..0.82) and higher than shoulders
            Part3D { cx: 0.0, cy: 0.76, cz: 0.62, hx: 0.26, hy: 0.22, hz: 0.20, pivot_y: 0.72, swing: 0.0, col: [242, 242, 240], col_alt: [24, 24, 28], face: FaceArt::Panda },
            // Snout
            Part3D { cx: 0.0, cy: 0.66, cz: 0.85, hx: 0.11, hy: 0.07, hz: 0.03, pivot_y: 0.72, swing: 0.0, col: [228, 228, 224], col_alt: [28, 28, 32], face: FaceArt::None },
            // Round black ears on top corners of head
            Part3D { cx: -0.20, cy: 1.03, cz: 0.60, hx: 0.07, hy: 0.06, hz: 0.04, pivot_y: 0.96, swing: 0.0, col: [28, 28, 32], col_alt: [18, 18, 20], face: FaceArt::None },
            Part3D { cx: 0.20, cy: 1.03, cz: 0.60, hx: 0.07, hy: 0.06, hz: 0.04, pivot_y: 0.96, swing: 0.0, col: [28, 28, 32], col_alt: [18, 18, 20], face: FaceArt::None },
            // Chunky black legs
            Part3D { cx: -0.20, cy: 0.17, cz: 0.26, hx: 0.11, hy: 0.17, hz: 0.11, pivot_y: 0.34, swing: s, col: [30, 30, 34], col_alt: [20, 20, 22], face: FaceArt::None },
            Part3D { cx: 0.20, cy: 0.17, cz: 0.26, hx: 0.11, hy: 0.17, hz: 0.11, pivot_y: 0.34, swing: -s, col: [30, 30, 34], col_alt: [20, 20, 22], face: FaceArt::None },
            Part3D { cx: -0.20, cy: 0.17, cz: -0.26, hx: 0.11, hy: 0.17, hz: 0.11, pivot_y: 0.34, swing: -s, col: [30, 30, 34], col_alt: [20, 20, 22], face: FaceArt::None },
            Part3D { cx: 0.20, cy: 0.17, cz: -0.26, hx: 0.11, hy: 0.17, hz: 0.11, pivot_y: 0.34, swing: s, col: [30, 30, 34], col_alt: [20, 20, 22], face: FaceArt::None },
        ],
        MobKind::Allay => {
            let hover = (walk * 1.4).sin() * 0.08;
            let flap = (walk * 3.8).sin() * 0.65;
            vec![
                Part3D { cx: 0.0, cy: 0.56 + hover, cz: 0.0, hx: 0.16, hy: 0.16, hz: 0.16, pivot_y: 0.56, swing: 0.0, col: [86, 224, 248], col_alt: [232, 250, 255], face: FaceArt::Allay },
                Part3D { cx: 0.0, cy: 0.30 + hover, cz: 0.0, hx: 0.11, hy: 0.14, hz: 0.09, pivot_y: 0.30, swing: 0.0, col: [56, 198, 232], col_alt: [38, 166, 208], face: FaceArt::None },
                Part3D { cx: -0.20, cy: 0.36 + hover, cz: -0.10, hx: 0.18, hy: 0.14, hz: 0.03, pivot_y: 0.36, swing: flap, col: [158, 240, 255], col_alt: [110, 218, 245], face: FaceArt::None },
                Part3D { cx: 0.20, cy: 0.36 + hover, cz: -0.10, hx: 0.18, hy: 0.14, hz: 0.03, pivot_y: 0.36, swing: -flap, col: [158, 240, 255], col_alt: [110, 218, 245], face: FaceArt::None },
            ]
        }
        MobKind::WanderingTrader => vec![
            Part3D { cx: 0.0, cy: 1.66, cz: 0.0, hx: 0.23, hy: 0.27, hz: 0.23, pivot_y: 1.40, swing: 0.0, col: [48, 86, 158], col_alt: [224, 186, 62], face: FaceArt::WanderingTrader },
            Part3D { cx: 0.0, cy: 1.54, cz: 0.25, hx: 0.06, hy: 0.08, hz: 0.06, pivot_y: 1.40, swing: 0.0, col: [188, 134, 102], col_alt: [160, 108, 78], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 1.00, cz: 0.0, hx: 0.25, hy: 0.40, hz: 0.16, pivot_y: 1.00, swing: 0.0, col: [42, 74, 142], col_alt: [218, 178, 56], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 1.12, cz: 0.20, hx: 0.22, hy: 0.14, hz: 0.12, pivot_y: 1.32, swing: 0.0, col: [46, 82, 150], col_alt: [224, 186, 62], face: FaceArt::None },
            Part3D { cx: -0.12, cy: 0.32, cz: 0.0, hx: 0.11, hy: 0.32, hz: 0.11, pivot_y: 0.64, swing: s, col: [64, 52, 42], col_alt: [44, 34, 28], face: FaceArt::None },
            Part3D { cx: 0.12, cy: 0.32, cz: 0.0, hx: 0.11, hy: 0.32, hz: 0.11, pivot_y: 0.64, swing: -s, col: [64, 52, 42], col_alt: [44, 34, 28], face: FaceArt::None },
        ],
        MobKind::Guardian => {
            let bob = (walk * 1.6).sin() * 0.08;
            vec![
                Part3D { cx: 0.0, cy: 0.54 + bob, cz: 0.0, hx: 0.34, hy: 0.34, hz: 0.34, pivot_y: 0.54, swing: 0.0, col: [74, 152, 140], col_alt: [228, 128, 58], face: FaceArt::Guardian },
                Part3D { cx: 0.0, cy: 0.54 + bob, cz: -0.46, hx: 0.10, hy: 0.12, hz: 0.22, pivot_y: 0.54, swing: s * 0.6, col: [64, 134, 124], col_alt: [224, 122, 54], face: FaceArt::None },
                Part3D { cx: -0.44, cy: 0.54 + bob, cz: 0.0, hx: 0.12, hy: 0.06, hz: 0.06, pivot_y: 0.54, swing: 0.0, col: [228, 128, 58], col_alt: [194, 98, 42], face: FaceArt::None },
                Part3D { cx: 0.44, cy: 0.54 + bob, cz: 0.0, hx: 0.12, hy: 0.06, hz: 0.06, pivot_y: 0.54, swing: 0.0, col: [228, 128, 58], col_alt: [194, 98, 42], face: FaceArt::None },
                Part3D { cx: 0.0, cy: 0.96 + bob, cz: 0.0, hx: 0.06, hy: 0.12, hz: 0.06, pivot_y: 0.96, swing: 0.0, col: [228, 128, 58], col_alt: [194, 98, 42], face: FaceArt::None },
            ]
        }
        MobKind::Warden => vec![
            Part3D { cx: 0.0, cy: 2.38, cz: 0.06, hx: 0.36, hy: 0.30, hz: 0.28, pivot_y: 2.08, swing: 0.0, col: [24, 58, 68], col_alt: [56, 224, 212], face: FaceArt::Warden },
            Part3D { cx: -0.48, cy: 2.52, cz: 0.04, hx: 0.14, hy: 0.16, hz: 0.06, pivot_y: 2.38, swing: 0.0, col: [48, 188, 180], col_alt: [22, 48, 56], face: FaceArt::None },
            Part3D { cx: 0.48, cy: 2.52, cz: 0.04, hx: 0.14, hy: 0.16, hz: 0.06, pivot_y: 2.38, swing: 0.0, col: [48, 188, 180], col_alt: [22, 48, 56], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 1.54, cz: 0.0, hx: 0.44, hy: 0.54, hz: 0.26, pivot_y: 1.54, swing: 0.0, col: [20, 44, 52], col_alt: [64, 238, 224], face: FaceArt::None },
            Part3D { cx: -0.60, cy: 1.32, cz: 0.08, hx: 0.16, hy: 0.66, hz: 0.16, pivot_y: 1.96, swing: -s * 0.9, col: [26, 60, 70], col_alt: [56, 216, 204], face: FaceArt::None },
            Part3D { cx: 0.60, cy: 1.32, cz: 0.08, hx: 0.16, hy: 0.66, hz: 0.16, pivot_y: 1.96, swing: s * 0.9, col: [26, 60, 70], col_alt: [56, 216, 204], face: FaceArt::None },
            Part3D { cx: -0.22, cy: 0.50, cz: 0.0, hx: 0.17, hy: 0.50, hz: 0.17, pivot_y: 1.00, swing: s * 0.8, col: [22, 50, 60], col_alt: [16, 36, 44], face: FaceArt::None },
            Part3D { cx: 0.22, cy: 0.50, cz: 0.0, hx: 0.17, hy: 0.50, hz: 0.17, pivot_y: 1.00, swing: -s * 0.8, col: [22, 50, 60], col_alt: [16, 36, 44], face: FaceArt::None },
        ],
    }
}

pub fn vehicle_parts(is_boat: bool) -> Vec<Part3D> {
    if is_boat {
        vec![
            Part3D { cx: 0.0, cy: 0.10, cz: 0.0, hx: 0.45, hy: 0.08, hz: 0.68, pivot_y: 0.1, swing: 0.0, col: [158, 114, 68], col_alt: [122, 84, 48], face: FaceArt::None },
            Part3D { cx: -0.40, cy: 0.28, cz: 0.0, hx: 0.07, hy: 0.14, hz: 0.66, pivot_y: 0.28, swing: 0.0, col: [142, 100, 58], col_alt: [110, 74, 42], face: FaceArt::None },
            Part3D { cx: 0.40, cy: 0.28, cz: 0.0, hx: 0.07, hy: 0.14, hz: 0.66, pivot_y: 0.28, swing: 0.0, col: [142, 100, 58], col_alt: [110, 74, 42], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.28, cz: -0.60, hx: 0.42, hy: 0.14, hz: 0.07, pivot_y: 0.28, swing: 0.0, col: [142, 100, 58], col_alt: [110, 74, 42], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.28, cz: 0.60, hx: 0.42, hy: 0.14, hz: 0.07, pivot_y: 0.28, swing: 0.0, col: [142, 100, 58], col_alt: [110, 74, 42], face: FaceArt::None },
        ]
    } else {
        vec![
            Part3D { cx: 0.0, cy: 0.14, cz: 0.0, hx: 0.42, hy: 0.08, hz: 0.48, pivot_y: 0.14, swing: 0.0, col: [98, 102, 108], col_alt: [68, 72, 78], face: FaceArt::None },
            Part3D { cx: -0.38, cy: 0.34, cz: 0.0, hx: 0.06, hy: 0.18, hz: 0.48, pivot_y: 0.34, swing: 0.0, col: [122, 128, 134], col_alt: [86, 90, 96], face: FaceArt::None },
            Part3D { cx: 0.38, cy: 0.34, cz: 0.0, hx: 0.06, hy: 0.18, hz: 0.48, pivot_y: 0.34, swing: 0.0, col: [122, 128, 134], col_alt: [86, 90, 96], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.34, cz: -0.44, hx: 0.42, hy: 0.18, hz: 0.06, pivot_y: 0.34, swing: 0.0, col: [122, 128, 134], col_alt: [86, 90, 96], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.34, cz: 0.44, hx: 0.42, hy: 0.18, hz: 0.06, pivot_y: 0.34, swing: 0.0, col: [122, 128, 134], col_alt: [86, 90, 96], face: FaceArt::None },
        ]
    }
}

#[derive(Clone)]
pub struct Mob {
    pub kind: MobKind,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub kb_vx: f32,
    pub kb_vz: f32,
    pub yaw: f32,
    pub hp: f32,
    pub timer: f32,
    pub panic: f32,
    pub attack_cd: f32,
    pub ranged_cd: f32,
    pub breed_cd: f32,
    pub hurt: f32,
    pub fuse: f32,
    pub walk: f32,
    pub on_ground: bool,
    pub tamed: bool,
    pub sitting: bool,
}

impl Mob {
    pub fn spawn(kind: MobKind, x: f32, y: f32, z: f32) -> Self {
        Self {
            kind, x, y, z, vx: 0.0, vy: 0.0, vz: 0.0, kb_vx: 0.0, kb_vz: 0.0, yaw: 0.0,
            hp: kind.health(), timer: 0.0, panic: 0.0, attack_cd: 0.0, ranged_cd: 2.0, breed_cd: 0.0,
            hurt: 0.0, fuse: 0.0, walk: 0.0, on_ground: true, tamed: false, sitting: false,
        }
    }

    pub fn is_attracted_to(kind: MobKind, held: u16) -> bool {
        if held == 0 { return false; }
        match kind {
            MobKind::Cow | MobKind::Mooshroom | MobKind::Sheep => held == world::WHEAT || held == world::HAY,
            MobKind::Pig => matches!(held, crate::items::APPLE | crate::items::GOLDEN_APPLE | crate::items::BREAD) || held == crate::items::CARROT || held == crate::items::BEETROOT,
            MobKind::Chicken => held == world::WHEAT || matches!(held, crate::items::BREAD | crate::items::WHEAT_SEEDS),
            MobKind::Wolf => matches!(held, crate::items::BONE | crate::items::RAW_BEEF | crate::items::STEAK | crate::items::RAW_PORK | crate::items::COOKED_PORK),
            MobKind::Bee => matches!(held, world::FLOWER_RED | world::FLOWER_YELLOW),
            MobKind::Panda => held == crate::items::BAMBOO || held == crate::items::SUGAR,
            _ => false,
        }
    }

    pub fn update(
        &mut self,
        world: &mut World,
        px: f32,
        py: f32,
        pz: f32,
        dt: f32,
        tick: u32,
        day: f32,
        grace: f32,
        ignore_player: bool,
        held_item: u16,
    ) {
        self.timer -= dt;
        self.panic = (self.panic - dt).max(0.0);
        self.attack_cd = (self.attack_cd - dt).max(0.0);
        self.ranged_cd = (self.ranged_cd - dt).max(0.0);
        self.breed_cd = (self.breed_cd - dt).max(0.0);
        self.hurt = (self.hurt - dt).max(0.0);
        let kb_drag = if self.on_ground { 0.015f32 } else { 0.08f32 };
        let kb_decay = kb_drag.powf(dt);
        self.kb_vx *= kb_decay;
        self.kb_vz *= kb_decay;
        if self.kb_vx.abs() < 0.04 { self.kb_vx = 0.0; }
        if self.kb_vz.abs() < 0.04 { self.kb_vz = 0.0; }

        let dx = px - self.x;
        let dz = pz - self.z;
        let dist = (dx * dx + dz * dz).sqrt();
        // Spiders are neutral in bright daylight unless provoked (`self.panic > 0.0`)
        let spider_docile = self.kind == MobKind::Spider && day >= 0.55 && self.panic <= 0.0;
        let hostile_active = self.kind.hostile() && !spider_docile && grace <= 0.0 && !ignore_player;

        if self.tamed && self.sitting {
            self.vx = 0.0;
            self.vz = 0.0;
            return;
        }
        if self.tamed && self.kind == MobKind::Wolf && dist > 3.5 && dist < 32.0 {
            self.vx = dx / dist * 3.4;
            self.vz = dz / dist * 3.4;
            self.yaw = dx.atan2(dz);
        }

        if world.dim == Dimension::Overworld && matches!(self.kind, MobKind::Zombie | MobKind::Skeleton) && day > 0.65 && tick % 25 == 0 {
            let ix = self.x.floor() as i32;
            let iz = self.z.floor() as i32;
            if self.y >= world.highest_ground(ix, iz) as f32 - 0.5 {
                self.hp -= 1.0;
                self.hurt = 0.25;
            }
        }

        if self.kind == MobKind::EnderDragon {
            let t = tick as f32 * 0.04;
            let target_x = if hostile_active && dist < 44.0 { px } else { t.cos() * 24.0 };
            let target_z = if hostile_active && dist < 44.0 { pz } else { t.sin() * 24.0 };
            let tdx = target_x - self.x;
            let tdz = target_z - self.z;
            let td = (tdx * tdx + tdz * tdz).sqrt().max(0.1);
            self.vx = tdx / td * 6.5;
            self.vz = tdz / td * 6.5;
            self.yaw = tdx.atan2(tdz);
            let target_y = if hostile_active && dist < 24.0 { py + 2.0 } else { 70.0 + (t * 2.0).sin() * 4.0 };
            self.vy = (target_y - self.y).clamp(-4.0, 4.0);
            self.x += (self.vx + self.kb_vx) * dt;
            self.y += self.vy * dt;
            self.z += (self.vz + self.kb_vz) * dt;
            self.walk += dt * 7.0;
            return;
        }

        if self.kind.flying() {
            if self.panic > 0.0 && !self.kind.hostile() {
                let away = (-dx).atan2(-dz) + ((tick as f32 * 0.7).sin()) * 0.5;
                self.vx = away.sin() * 4.0;
                self.vz = away.cos() * 4.0;
                self.yaw = away;
            } else if hostile_active && dist < 24.0 && dist > 2.5 {
                self.vx = dx / dist * 2.6;
                self.vz = dz / dist * 2.6;
                self.yaw = dx.atan2(dz);
            } else if self.timer <= 0.0 {
                self.timer = 2.0;
                let ang = ((tick as f32 * 1.618 + self.x * 0.37) % 6.283) - 3.1415;
                self.vx = ang.sin() * 1.5;
                self.vz = ang.cos() * 1.5;
                self.yaw = ang;
            }
            let g = world.highest_ground(self.x.floor() as i32, self.z.floor() as i32) as f32;
            let hover = if self.kind == MobKind::Squid || self.kind == MobKind::Guardian {
                (world::SEA as f32 - 1.5).min(g + 0.5)
            } else if self.kind == MobKind::Ghast || self.kind == MobKind::Wither {
                g + 5.5
            } else if self.kind == MobKind::Phantom {
                if hostile_active && dist < 24.0 { py + 2.5 } else { g + 7.5 }
            } else {
                g + 2.2
            };
            self.vy = (hover - self.y).clamp(-2.5, 2.5);
            self.x += (self.vx + self.kb_vx) * dt;
            self.y += self.vy * dt;
            self.z += (self.vz + self.kb_vz) * dt;
            self.walk += dt * 5.0;
            return;
        }

        if self.kind == MobKind::Creeper && hostile_active && dist < 2.6 && (py - self.y).abs() < 2.5 {
            self.fuse += dt;
            self.vx = 0.0;
            self.vz = 0.0;
            self.yaw = dx.atan2(dz);
        } else {
            if self.kind == MobKind::Creeper {
                self.fuse = (self.fuse - dt * 2.0).max(0.0);
            }
            if self.panic > 0.0 && !self.kind.hostile() {
                if self.timer <= 0.0 {
                    self.timer = 1.05;
                    let away = (-dx).atan2(-dz);
                    let jitter = ((tick as f32 * 0.65 + self.x * 0.4) % 0.8) - 0.4;
                    let ang = away + jitter;
                    let sp = match self.kind {
                        MobKind::Wolf => 3.4,
                        MobKind::Cow | MobKind::Mooshroom | MobKind::Pig | MobKind::Sheep => 2.9,
                        MobKind::Chicken => 2.6,
                        _ => 2.8,
                    };
                    self.vx = ang.sin() * sp;
                    self.vz = ang.cos() * sp;
                    self.yaw = ang;
                }
            } else if self.kb_vx.abs() + self.kb_vz.abs() > 0.6 {
                // While experiencing knockback, do not let AI chase override the knockback impulse
                self.vx *= (0.05f32).powf(dt);
                self.vz *= (0.05f32).powf(dt);
            } else if !self.kind.hostile() && !ignore_player && Self::is_attracted_to(self.kind, held_item) && dist < 11.0 {
                // Passive animal follows player holding its corresponding food!
                let target_yaw = dx.atan2(dz);
                self.yaw = target_yaw;
                if dist > 2.2 {
                    self.vx = dx / dist * 1.85;
                    self.vz = dz / dist * 1.85;
                } else {
                    self.vx = 0.0;
                    self.vz = 0.0;
                }
                self.timer = 0.4;
            } else if hostile_active && dist < 22.0 && dist > 1.0 {
                let sp = if self.kind == MobKind::Warden { 3.1 } else if matches!(self.kind, MobKind::Spider | MobKind::WitherSkeleton) { 2.9 } else { 2.2 };
                self.vx = dx / dist * sp;
                self.vz = dz / dist * sp;
                self.yaw = dx.atan2(dz);
            } else if self.timer <= 0.0 {
                self.timer = 2.0 + ((tick as f32 * 0.13) % 2.4);
                let ang = ((tick as f32 * 1.618 + self.x * 0.37) % 6.283) - 3.1415;
                if (tick % 3) == 0 {
                    self.vx = 0.0;
                    self.vz = 0.0;
                } else {
                    self.vx = ang.sin() * 1.15;
                    self.vz = ang.cos() * 1.15;
                    self.yaw = ang;
                }
            } else if !self.kind.hostile() && self.vx.abs() < 0.05 && self.vz.abs() < 0.05 && dist < 6.0 {
                // Idle passive mobs & villagers gently look toward nearby players
                let target_yaw = dx.atan2(dz);
                let mut diff = target_yaw - self.yaw;
                while diff > std::f32::consts::PI { diff -= std::f32::consts::TAU; }
                while diff < -std::f32::consts::PI { diff += std::f32::consts::TAU; }
                self.yaw += diff * (dt * 4.5).min(1.0);
            }
        }

        if self.kind == MobKind::Slime && self.on_ground && (tick % 35) == 0 {
            self.vy = 5.4;
            self.on_ground = false;
        }

        let r = self.kind.radius();
        let h = self.kind.height();
        let move_vx = self.vx + self.kb_vx;
        let move_vz = self.vz + self.kb_vz;

        let speed_xz = (move_vx * move_vx + move_vz * move_vz).sqrt();
        if self.kind == MobKind::Chicken && !self.on_ground && self.vy < -0.35 {
            self.walk += dt * 12.0;
        } else if speed_xz > 0.1 && self.on_ground {
            self.walk += speed_xz * dt * 1.85;
        }

        // If mob somehow ended up inside a solid block, push it up to free space
        if mob_collides(world, self.x, self.y, self.z, r, h) {
            if !mob_collides(world, self.x, self.y + 1.0, self.z, r, h) {
                self.y += 1.0;
                self.vy = 0.0;
            } else {
                let gx = self.x.floor() as i32;
                let gz = self.z.floor() as i32;
                let gy = world.highest_ground(gx, gz) as f32;
                if (gy - self.y).abs() <= 3.0 && !mob_collides(world, self.x, gy, self.z, r, h) {
                    self.y = gy;
                    self.vy = 0.0;
                }
            }
        }

        let nx = self.x + move_vx * dt;
        let nz = self.z + move_vz * dt;
        let blocked_x = mob_collides(world, nx, self.y, self.z, r, h);
        let blocked_z = mob_collides(world, self.x, self.y, nz, r, h);

        if (blocked_x || blocked_z) && self.on_ground {
            let can_step_x = blocked_x && !mob_collides(world, nx, self.y + 1.05, self.z, r, h);
            let can_step_z = blocked_z && !mob_collides(world, self.x, self.y + 1.05, nz, r, h);
            let step_ok = (!blocked_x || can_step_x) && (!blocked_z || can_step_z);
            if step_ok || can_step_x || can_step_z {
                // 7.7 upward impulse under 22.0 gravity reaches ~1.35 blocks so mobs reliably clear 1-block ledges
                self.vy = 7.7;
                self.on_ground = false;
            } else {
                // Wall is 2+ blocks tall: turn away instead of getting stuck
                if blocked_x { self.kb_vx = 0.0; self.vx = -self.vx * 0.75; }
                if blocked_z { self.kb_vz = 0.0; self.vz = -self.vz * 0.75; }
                self.yaw = self.vx.atan2(self.vz);
                if self.panic > 0.0 { self.timer = 0.25; }
            }
        }
        if !blocked_x { self.x = nx; } else { self.kb_vx = 0.0; }
        if !blocked_z { self.z = nz; } else { self.kb_vz = 0.0; }

        self.vy -= 22.0 * dt;
        if self.kind == MobKind::Chicken && !self.on_ground && self.vy < 0.0 {
            // Chickens slowly glide downward while flapping their wings
            self.vy = self.vy.max(-2.2);
        }
        let ny = self.y + self.vy * dt;
        if self.vy <= 0.0 && mob_collides(world, self.x, ny - 0.01, self.z, r, h) {
            let snap_y = ground_top_at(world, self.x, ny + 0.02, self.z, r)
                .unwrap_or_else(|| (ny + 0.02).floor() + 1.0);
            if !mob_collides(world, self.x, snap_y, self.z, r, h) {
                self.y = snap_y;
            } else {
                self.y = ny.ceil();
            }
            self.vy = 0.0;
            self.on_ground = true;
        } else if self.vy > 0.0 && mob_collides(world, self.x, ny, self.z, r, h) {
            self.vy = 0.0;
        } else {
            self.y = ny;
            self.on_ground = false;
        }
        if self.y < 1.0 && world.dim != Dimension::End {
            self.y = 40.0;
            self.vy = 0.0;
        }
    }
}

pub fn biome_mob(biome: Biome, dim: Dimension, seed: u32, day: f32) -> MobKind {
    match dim {
        Dimension::Nether => match biome {
            Biome::CrimsonForest => match (seed >> 3) % 4 {
                0 | 1 => MobKind::Piglin,
                2 => MobKind::ZombifiedPiglin,
                _ => MobKind::Ghast,
            },
            Biome::WarpedForest => MobKind::Enderman,
            Biome::SoulSandValley => match (seed >> 3) % 3 {
                0 => MobKind::Skeleton,
                1 => MobKind::Ghast,
                _ => MobKind::WitherSkeleton,
            },
            Biome::BasaltDeltas => match (seed >> 3) % 3 {
                0 => MobKind::Blaze,
                1 => MobKind::Ghast,
                _ => MobKind::Slime,
            },
            _ => match (seed >> 3) % 5 {
                0 | 1 => MobKind::ZombifiedPiglin,
                2 => MobKind::Ghast,
                3 => MobKind::Blaze,
                _ => MobKind::WitherSkeleton,
            },
        },
        Dimension::End => MobKind::Enderman,
        Dimension::Overworld => {
            if day < 0.42 && seed % 2 == 0 && biome != Biome::MushroomIsland {
                return match (seed >> 3) % 6 {
                    0 => MobKind::Zombie,
                    1 => MobKind::Skeleton,
                    2 => MobKind::Creeper,
                    3 => MobKind::Spider,
                    4 => MobKind::Phantom,
                    _ => if biome == Biome::Swamp { MobKind::Slime } else { MobKind::Enderman },
                };
            }
            match biome {
                Biome::Snowy => if seed % 3 == 0 { MobKind::PolarBear } else { MobKind::Wolf },
                Biome::Taiga => match (seed >> 3) % 4 { 0 => MobKind::Wolf, 1 => MobKind::Fox, 2 => MobKind::Sheep, _ => MobKind::Pig },
                Biome::Jungle => match (seed >> 3) % 16 {
                    0 => MobKind::Panda,
                    1..=4 => MobKind::Parrot,
                    5..=9 => MobKind::Chicken,
                    10..=12 => MobKind::Pig,
                    _ => MobKind::Sheep,
                },
                Biome::Ocean => if (seed >> 3) % 3 == 0 { MobKind::Guardian } else { MobKind::Squid },
                Biome::Swamp => match (seed >> 3) % 4 {
                    0 => if day < 0.42 { MobKind::Slime } else { MobKind::Squid },
                    1 => MobKind::Sheep,
                    2 => MobKind::Pig,
                    _ => MobKind::Chicken,
                },
                Biome::DarkForest => match (seed >> 3) % 5 {
                    0 => MobKind::Allay,
                    1 => MobKind::Wolf,
                    2 => MobKind::Fox,
                    3 => MobKind::Sheep,
                    _ => MobKind::Bee,
                },
                Biome::MushroomIsland => MobKind::Mooshroom,
                Biome::Desert | Biome::Badlands => {
                    if day < 0.42 {
                        if seed % 2 == 0 { MobKind::Skeleton } else { MobKind::Spider }
                    } else {
                        match (seed >> 3) % 3 {
                            0 => MobKind::Sheep,
                            1 => MobKind::Chicken,
                            _ => MobKind::Pig,
                        }
                    }
                }
                _ => match (seed >> 3) % 7 {
                    0 => MobKind::Sheep,
                    1 => MobKind::Cow,
                    2 => MobKind::Pig,
                    3 => MobKind::Chicken,
                    4 => MobKind::Bee,
                    5 => if (seed >> 7) % 10 == 0 { MobKind::WanderingTrader } else { MobKind::Sheep },
                    _ => if biome == Biome::Forest { MobKind::Wolf } else { MobKind::Sheep },
                },
            }
        }
    }
}

#[derive(Clone)]
pub struct Drop {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub age: f32,
    pub item: u16,
}

impl Drop {
    pub fn new(x: f32, y: f32, z: f32, item: u16) -> Self {
        Self { x, y, z, vx: 0.0, vy: 3.0, vz: 0.0, age: 0.0, item }
    }
    pub fn with_scatter(x: f32, y: f32, z: f32, item: u16, vx: f32, vz: f32) -> Self {
        Self { x, y, z, vx, vy: 3.4, vz, age: 0.0, item }
    }
    pub fn update(&mut self, world: &mut World, dt: f32) {
        self.age += dt;
        let drag = (0.04f32).powf(dt);
        self.vx *= drag;
        self.vz *= drag;
        let nx = self.x + self.vx * dt;
        if !world::is_solid(world.get(nx.floor() as i32, self.y.floor() as i32, self.z.floor() as i32)) {
            self.x = nx;
        } else {
            self.vx = 0.0;
        }
        let nz = self.z + self.vz * dt;
        if !world::is_solid(world.get(self.x.floor() as i32, self.y.floor() as i32, nz.floor() as i32)) {
            self.z = nz;
        } else {
            self.vz = 0.0;
        }
        self.vy -= 18.0 * dt;
        let ny = self.y + self.vy * dt;
        if self.vy < 0.0 && world::is_solid(world.get(self.x.floor() as i32, ny.floor() as i32, self.z.floor() as i32)) {
            self.y = ny.floor() + 1.0;
            self.vy = 0.0;
        } else {
            self.y = ny;
        }
    }
}

#[derive(Clone, Copy)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub life: f32,
    pub size: f32,
    pub color: [u8; 3],
    pub gravity: f32,
}

impl Particle {
    pub fn update(&mut self, world: &mut World, dt: f32) -> bool {
        self.life -= dt;
        if self.life <= 0.0 {
            return false;
        }
        self.vy -= self.gravity * dt;
        let nx = self.x + self.vx * dt;
        let ny = self.y + self.vy * dt;
        let nz = self.z + self.vz * dt;
        if self.gravity > 0.0
            && self.vy < 0.0
            && world::is_solid(world.get(nx.floor() as i32, ny.floor() as i32, nz.floor() as i32))
        {
            self.y = ny.floor() + 1.02;
            self.vy = -self.vy * 0.25;
            self.vx *= 0.55;
            self.vz *= 0.55;
        } else {
            self.x = nx;
            self.y = ny;
            self.z = nz;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panicking_cow_is_fast_and_jumps_one_block() {
        let mut world = World::new();
        // Flat stone floor at y = 39 (top at y = 40.0) and a 1-block-tall step at z = 6..10
        for x in -4..=12 {
            for z in -4..=16 {
                for y in 38..=45 {
                    world.set(x, y, z, world::AIR);
                }
                world.set(x, 39, z, world::STONE);
                if z >= 6 {
                    world.set(x, 40, z, world::STONE);
                }
            }
        }
        let mut cow = Mob::spawn(MobKind::Cow, 4.5, 40.0, 4.0);
        cow.panic = 4.0;
        cow.hurt = 0.45;
        cow.timer = 0.0;
        // Player behind cow at z = 1.0 -> cow runs in +z toward the 1-block ledge at z = 6
        let mut max_y = cow.y;
        let mut max_speed = 0.0f32;
        for t in 0..80 {
            cow.update(&mut world, 4.5, 40.0, 1.0, 0.03, t, 1.0, 0.0, false, 0);
            max_y = max_y.max(cow.y);
            let sp = (cow.vx * cow.vx + cow.vz * cow.vz).sqrt();
            max_speed = max_speed.max(sp);
        }
        assert!(max_speed >= 2.75, "expected panic speed >= 2.75 m/s, got {}", max_speed);
        assert!(max_y > 41.0, "expected cow to jump over 1-block ledge (y > 41.0), got {}", max_y);
        assert!(cow.y >= 40.99 && cow.z >= 5.5, "expected cow to climb onto 1-block step, got y={} z={}", cow.y, cow.z);
    }

    #[test]
    fn desert_daytime_never_spawns_hostiles() {
        for seed in 0..200u32 {
            let m = biome_mob(Biome::Desert, Dimension::Overworld, seed * 7919, 0.85);
            assert!(!m.hostile(), "desert daytime spawned hostile {:?} for seed {}", m, seed);
            let b = biome_mob(Biome::Badlands, Dimension::Overworld, seed * 7919, 0.85);
            assert!(!b.hostile(), "badlands daytime spawned hostile {:?} for seed {}", b, seed);
        }
    }

    #[test]
    fn chicken_slow_falls_and_flaps() {
        let mut world = World::new();
        for x in -2..=4 {
            for z in -2..=4 {
                world.set(x, 10, z, world::STONE);
            }
        }
        let mut chick = Mob::spawn(MobKind::Chicken, 1.5, 25.0, 1.5);
        chick.on_ground = false;
        let w0 = chick.walk;
        for t in 0..40 {
            chick.update(&mut world, 10.0, 25.0, 10.0, 0.03, t, 1.0, 0.0, true, 0);
        }
        assert!(chick.vy >= -2.25, "chicken fell too fast: vy={}", chick.vy);
        assert!(chick.walk > w0 + 10.0, "chicken wings did not flap while airborne: walk={}", chick.walk);
        let parts = mob_parts(MobKind::Chicken, chick.walk, !chick.on_ground);
        assert!(parts[5].hx > 0.05, "expected chicken airborne wing spread");
    }

    #[test]
    fn iron_golem_is_tall_and_non_hostile() {
        assert!(!MobKind::IronGolem.hostile());
        assert_eq!(MobKind::IronGolem.health(), 100.0);
        assert!(MobKind::IronGolem.height() > 2.5);
        assert!(mob_parts(MobKind::IronGolem, 0.0, false).len() >= 8);
    }
}

