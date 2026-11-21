//! 3D voxel box-model definitions and creature behaviour (Minecraft mobs).
use crate::world::{self, World};

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
        }
    }

    pub fn hostile(self) -> bool {
        matches!(
            self,
            MobKind::Zombie | MobKind::Skeleton | MobKind::Creeper | MobKind::Spider | MobKind::Enderman
        )
    }

    pub fn health(self) -> f32 {
        match self {
            MobKind::Spider => 16.0,
            MobKind::Enderman => 40.0,
            MobKind::Chicken => 4.0,
            MobKind::Villager | MobKind::Zombie | MobKind::Skeleton | MobKind::Creeper => 20.0,
            _ => 10.0,
        }
    }

    pub fn drop(self) -> u16 {
        match self {
            MobKind::Sheep => world::WOOL_WHITE,
            MobKind::Cow => 273,      // Raw Beef
            MobKind::Pig => 272,      // Raw Porkchop
            MobKind::Chicken => 275,  // Raw Chicken
            MobKind::Zombie => 288,   // Rotten Flesh
            MobKind::Skeleton => 287, // Bone
            MobKind::Creeper => 306,  // Gunpowder
            MobKind::Spider => 289,   // String
            MobKind::Enderman => 302, // Ender Pearl
            MobKind::Villager => 0,
        }
    }

    pub fn height(self) -> f32 {
        match self {
            MobKind::Chicken => 0.7,
            MobKind::Spider => 0.9,
            MobKind::Enderman => 2.9,
            MobKind::Pig => 0.9,
            MobKind::Sheep | MobKind::Cow => 1.35,
            _ => 1.8,
        }
    }
}

/// Returns the 3D articulated box parts (feet at y = 0, facing +Z in local space).
pub fn steve_parts(walk: f32) -> Vec<Part3D> {
    let s = walk.sin() * 0.55;
    vec![
        // head
        Part3D { cx: 0.0, cy: 1.62, cz: 0.0, hx: 0.24, hy: 0.24, hz: 0.24, pivot_y: 1.38, swing: 0.0, col: [218, 172, 134], col_alt: [85, 55, 32], face: FaceArt::Steve },
        // torso (cyan shirt)
        Part3D { cx: 0.0, cy: 1.05, cz: 0.0, hx: 0.24, hy: 0.34, hz: 0.13, pivot_y: 1.05, swing: 0.0, col: [46, 154, 174], col_alt: [36, 128, 148], face: FaceArt::None },
        // arms
        Part3D { cx: -0.35, cy: 1.05, cz: 0.0, hx: 0.11, hy: 0.34, hz: 0.11, pivot_y: 1.36, swing: -s, col: [215, 168, 130], col_alt: [46, 154, 174], face: FaceArt::None },
        Part3D { cx: 0.35, cy: 1.05, cz: 0.0, hx: 0.11, hy: 0.34, hz: 0.11, pivot_y: 1.36, swing: s, col: [215, 168, 130], col_alt: [46, 154, 174], face: FaceArt::None },
        // legs (blue jeans)
        Part3D { cx: -0.12, cy: 0.36, cz: 0.0, hx: 0.12, hy: 0.36, hz: 0.12, pivot_y: 0.72, swing: s, col: [56, 62, 142], col_alt: [45, 50, 120], face: FaceArt::None },
        Part3D { cx: 0.12, cy: 0.36, cz: 0.0, hx: 0.12, hy: 0.36, hz: 0.12, pivot_y: 0.72, swing: -s, col: [56, 62, 142], col_alt: [45, 50, 120], face: FaceArt::None },
    ]
}

pub fn mob_parts(kind: MobKind, walk: f32) -> Vec<Part3D> {
    let s = walk.sin() * 0.55;
    match kind {
        MobKind::Zombie => vec![
            Part3D { cx: 0.0, cy: 1.62, cz: 0.0, hx: 0.24, hy: 0.24, hz: 0.24, pivot_y: 1.38, swing: 0.0, col: [92, 144, 72], col_alt: [72, 118, 56], face: FaceArt::Zombie },
            Part3D { cx: 0.0, cy: 1.05, cz: 0.0, hx: 0.24, hy: 0.34, hz: 0.13, pivot_y: 1.05, swing: 0.0, col: [48, 136, 150], col_alt: [38, 112, 124], face: FaceArt::None },
            // zombie arms outstretched forward
            Part3D { cx: -0.35, cy: 1.24, cz: 0.26, hx: 0.11, hy: 0.11, hz: 0.32, pivot_y: 1.32, swing: s * 0.2, col: [92, 144, 72], col_alt: [48, 136, 150], face: FaceArt::None },
            Part3D { cx: 0.35, cy: 1.24, cz: 0.26, hx: 0.11, hy: 0.11, hz: 0.32, pivot_y: 1.32, swing: -s * 0.2, col: [92, 144, 72], col_alt: [48, 136, 150], face: FaceArt::None },
            Part3D { cx: -0.12, cy: 0.36, cz: 0.0, hx: 0.12, hy: 0.36, hz: 0.12, pivot_y: 0.72, swing: s, col: [72, 62, 134], col_alt: [55, 48, 110], face: FaceArt::None },
            Part3D { cx: 0.12, cy: 0.36, cz: 0.0, hx: 0.12, hy: 0.36, hz: 0.12, pivot_y: 0.72, swing: -s, col: [72, 62, 134], col_alt: [55, 48, 110], face: FaceArt::None },
        ],
        MobKind::Skeleton => vec![
            Part3D { cx: 0.0, cy: 1.62, cz: 0.0, hx: 0.24, hy: 0.24, hz: 0.24, pivot_y: 1.38, swing: 0.0, col: [222, 222, 212], col_alt: [190, 190, 180], face: FaceArt::Skeleton },
            Part3D { cx: 0.0, cy: 1.05, cz: 0.0, hx: 0.22, hy: 0.33, hz: 0.11, pivot_y: 1.05, swing: 0.0, col: [212, 212, 202], col_alt: [165, 165, 158], face: FaceArt::None },
            Part3D { cx: -0.31, cy: 1.20, cz: 0.22, hx: 0.06, hy: 0.06, hz: 0.28, pivot_y: 1.30, swing: s * 0.2, col: [216, 216, 206], col_alt: [185, 185, 175], face: FaceArt::None },
            Part3D { cx: 0.31, cy: 1.20, cz: 0.22, hx: 0.06, hy: 0.06, hz: 0.28, pivot_y: 1.30, swing: -s * 0.2, col: [216, 216, 206], col_alt: [185, 185, 175], face: FaceArt::None },
            Part3D { cx: -0.12, cy: 0.36, cz: 0.0, hx: 0.06, hy: 0.36, hz: 0.06, pivot_y: 0.72, swing: s, col: [216, 216, 206], col_alt: [185, 185, 175], face: FaceArt::None },
            Part3D { cx: 0.12, cy: 0.36, cz: 0.0, hx: 0.06, hy: 0.36, hz: 0.06, pivot_y: 0.72, swing: -s, col: [216, 216, 206], col_alt: [185, 185, 175], face: FaceArt::None },
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
            Part3D { cx: 0.0, cy: 0.46, cz: -0.22, hx: 0.32, hy: 0.24, hz: 0.36, pivot_y: 0.46, swing: 0.0, col: [40, 36, 40], col_alt: [28, 24, 28], face: FaceArt::None },
            // 4 legs left + right
            Part3D { cx: -0.50, cy: 0.24, cz: 0.24, hx: 0.26, hy: 0.07, hz: 0.07, pivot_y: 0.32, swing: s, col: [54, 46, 50], col_alt: [34, 30, 34], face: FaceArt::None },
            Part3D { cx: 0.50, cy: 0.24, cz: 0.24, hx: 0.26, hy: 0.07, hz: 0.07, pivot_y: 0.32, swing: -s, col: [54, 46, 50], col_alt: [34, 30, 34], face: FaceArt::None },
            Part3D { cx: -0.52, cy: 0.24, cz: 0.0, hx: 0.28, hy: 0.07, hz: 0.07, pivot_y: 0.32, swing: -s, col: [54, 46, 50], col_alt: [34, 30, 34], face: FaceArt::None },
            Part3D { cx: 0.52, cy: 0.24, cz: 0.0, hx: 0.28, hy: 0.07, hz: 0.07, pivot_y: 0.32, swing: s, col: [54, 46, 50], col_alt: [34, 30, 34], face: FaceArt::None },
            Part3D { cx: -0.50, cy: 0.24, cz: -0.24, hx: 0.26, hy: 0.07, hz: 0.07, pivot_y: 0.32, swing: s, col: [54, 46, 50], col_alt: [34, 30, 34], face: FaceArt::None },
            Part3D { cx: 0.50, cy: 0.24, cz: -0.24, hx: 0.26, hy: 0.07, hz: 0.07, pivot_y: 0.32, swing: -s, col: [54, 46, 50], col_alt: [34, 30, 34], face: FaceArt::None },
        ],
        MobKind::Pig => vec![
            Part3D { cx: 0.0, cy: 0.68, cz: 0.42, hx: 0.22, hy: 0.20, hz: 0.20, pivot_y: 0.68, swing: 0.0, col: [242, 166, 174], col_alt: [215, 135, 146], face: FaceArt::Pig },
            Part3D { cx: 0.0, cy: 0.60, cz: 0.64, hx: 0.11, hy: 0.08, hz: 0.06, pivot_y: 0.60, swing: 0.0, col: [246, 142, 156], col_alt: [185, 95, 110], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.58, cz: -0.02, hx: 0.26, hy: 0.22, hz: 0.38, pivot_y: 0.58, swing: 0.0, col: [236, 156, 166], col_alt: [210, 130, 142], face: FaceArt::None },
            Part3D { cx: -0.15, cy: 0.18, cz: 0.22, hx: 0.09, hy: 0.18, hz: 0.09, pivot_y: 0.36, swing: s, col: [228, 148, 158], col_alt: [195, 115, 128], face: FaceArt::None },
            Part3D { cx: 0.15, cy: 0.18, cz: 0.22, hx: 0.09, hy: 0.18, hz: 0.09, pivot_y: 0.36, swing: -s, col: [228, 148, 158], col_alt: [195, 115, 128], face: FaceArt::None },
            Part3D { cx: -0.15, cy: 0.18, cz: -0.24, hx: 0.09, hy: 0.18, hz: 0.09, pivot_y: 0.36, swing: -s, col: [228, 148, 158], col_alt: [195, 115, 128], face: FaceArt::None },
            Part3D { cx: 0.15, cy: 0.18, cz: -0.24, hx: 0.09, hy: 0.18, hz: 0.09, pivot_y: 0.36, swing: s, col: [228, 148, 158], col_alt: [195, 115, 128], face: FaceArt::None },
        ],
        MobKind::Cow => vec![
            Part3D { cx: 0.0, cy: 1.06, cz: 0.48, hx: 0.22, hy: 0.22, hz: 0.18, pivot_y: 1.06, swing: 0.0, col: [82, 58, 42], col_alt: [232, 232, 228], face: FaceArt::Cow },
            Part3D { cx: 0.0, cy: 0.90, cz: -0.04, hx: 0.30, hy: 0.28, hz: 0.44, pivot_y: 0.90, swing: 0.0, col: [76, 54, 38], col_alt: [228, 228, 224], face: FaceArt::None },
            Part3D { cx: -0.18, cy: 0.31, cz: 0.28, hx: 0.10, hy: 0.31, hz: 0.10, pivot_y: 0.62, swing: s, col: [68, 48, 34], col_alt: [215, 215, 210], face: FaceArt::None },
            Part3D { cx: 0.18, cy: 0.31, cz: 0.28, hx: 0.10, hy: 0.31, hz: 0.10, pivot_y: 0.62, swing: -s, col: [68, 48, 34], col_alt: [215, 215, 210], face: FaceArt::None },
            Part3D { cx: -0.18, cy: 0.31, cz: -0.32, hx: 0.10, hy: 0.31, hz: 0.10, pivot_y: 0.62, swing: -s, col: [68, 48, 34], col_alt: [215, 215, 210], face: FaceArt::None },
            Part3D { cx: 0.18, cy: 0.31, cz: -0.32, hx: 0.10, hy: 0.31, hz: 0.10, pivot_y: 0.62, swing: s, col: [68, 48, 34], col_alt: [215, 215, 210], face: FaceArt::None },
        ],
        MobKind::Sheep => vec![
            Part3D { cx: 0.0, cy: 0.98, cz: 0.44, hx: 0.20, hy: 0.19, hz: 0.20, pivot_y: 0.98, swing: 0.0, col: [228, 198, 174], col_alt: [238, 238, 232], face: FaceArt::Sheep },
            Part3D { cx: 0.0, cy: 0.86, cz: -0.02, hx: 0.32, hy: 0.28, hz: 0.42, pivot_y: 0.86, swing: 0.0, col: [240, 240, 234], col_alt: [215, 215, 210], face: FaceArt::None },
            Part3D { cx: -0.16, cy: 0.29, cz: 0.24, hx: 0.10, hy: 0.29, hz: 0.10, pivot_y: 0.58, swing: s, col: [218, 190, 168], col_alt: [190, 164, 145], face: FaceArt::None },
            Part3D { cx: 0.16, cy: 0.29, cz: 0.24, hx: 0.10, hy: 0.29, hz: 0.10, pivot_y: 0.58, swing: -s, col: [218, 190, 168], col_alt: [190, 164, 145], face: FaceArt::None },
            Part3D { cx: -0.16, cy: 0.29, cz: -0.26, hx: 0.10, hy: 0.29, hz: 0.10, pivot_y: 0.58, swing: -s, col: [218, 190, 168], col_alt: [190, 164, 145], face: FaceArt::None },
            Part3D { cx: 0.16, cy: 0.29, cz: -0.26, hx: 0.10, hy: 0.29, hz: 0.10, pivot_y: 0.58, swing: s, col: [218, 190, 168], col_alt: [190, 164, 145], face: FaceArt::None },
        ],
        MobKind::Chicken => vec![
            Part3D { cx: 0.0, cy: 0.58, cz: 0.18, hx: 0.12, hy: 0.14, hz: 0.10, pivot_y: 0.58, swing: 0.0, col: [244, 244, 240], col_alt: [235, 185, 50], face: FaceArt::Chicken },
            Part3D { cx: 0.0, cy: 0.52, cz: 0.30, hx: 0.08, hy: 0.05, hz: 0.06, pivot_y: 0.52, swing: 0.0, col: [235, 185, 50], col_alt: [210, 45, 45], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.40, cz: -0.02, hx: 0.17, hy: 0.16, hz: 0.22, pivot_y: 0.40, swing: 0.0, col: [236, 236, 232], col_alt: [212, 212, 208], face: FaceArt::None },
            Part3D { cx: -0.09, cy: 0.12, cz: 0.0, hx: 0.04, hy: 0.12, hz: 0.04, pivot_y: 0.24, swing: s, col: [235, 185, 50], col_alt: [200, 150, 35], face: FaceArt::None },
            Part3D { cx: 0.09, cy: 0.12, cz: 0.0, hx: 0.04, hy: 0.12, hz: 0.04, pivot_y: 0.24, swing: -s, col: [235, 185, 50], col_alt: [200, 150, 35], face: FaceArt::None },
        ],
        MobKind::Villager => vec![
            Part3D { cx: 0.0, cy: 1.66, cz: 0.0, hx: 0.23, hy: 0.28, hz: 0.23, pivot_y: 1.38, swing: 0.0, col: [210, 162, 126], col_alt: [95, 62, 42], face: FaceArt::Villager },
            Part3D { cx: 0.0, cy: 1.54, cz: 0.25, hx: 0.07, hy: 0.09, hz: 0.06, pivot_y: 1.54, swing: 0.0, col: [195, 148, 114], col_alt: [175, 130, 98], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 0.88, cz: 0.0, hx: 0.26, hy: 0.50, hz: 0.18, pivot_y: 0.88, swing: 0.0, col: [122, 82, 52], col_alt: [96, 64, 40], face: FaceArt::None },
            Part3D { cx: 0.0, cy: 1.04, cz: 0.16, hx: 0.24, hy: 0.12, hz: 0.12, pivot_y: 1.04, swing: 0.0, col: [134, 92, 60], col_alt: [106, 72, 46], face: FaceArt::None },
            Part3D { cx: -0.12, cy: 0.22, cz: 0.0, hx: 0.11, hy: 0.22, hz: 0.11, pivot_y: 0.44, swing: s * 0.6, col: [96, 64, 40], col_alt: [72, 48, 30], face: FaceArt::None },
            Part3D { cx: 0.12, cy: 0.22, cz: 0.0, hx: 0.11, hy: 0.22, hz: 0.11, pivot_y: 0.44, swing: -s * 0.6, col: [96, 64, 40], col_alt: [72, 48, 30], face: FaceArt::None },
        ],
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
    pub yaw: f32,
    pub hp: f32,
    pub timer: f32,
    pub attack_cd: f32,
    pub hurt: f32,
    pub fuse: f32,
    pub walk: f32,
    pub on_ground: bool,
}

impl Mob {
    pub fn spawn(kind: MobKind, x: f32, y: f32, z: f32) -> Self {
        Self {
            kind,
            x,
            y,
            z,
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            yaw: 0.0,
            hp: kind.health(),
            timer: 0.0,
            attack_cd: 0.0,
            hurt: 0.0,
            fuse: 0.0,
            walk: 0.0,
            on_ground: true,
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
    ) {
        self.timer -= dt;
        self.attack_cd = (self.attack_cd - dt).max(0.0);
        self.hurt = (self.hurt - dt).max(0.0);
        let dx = px - self.x;
        let dz = pz - self.z;
        let dist = (dx * dx + dz * dz).sqrt();
        let hostile_active = self.kind.hostile() && grace <= 0.0 && !ignore_player;

        if matches!(self.kind, MobKind::Zombie | MobKind::Skeleton) && day > 0.65 && tick % 25 == 0 {
            let ix = self.x.floor() as i32;
            let iz = self.z.floor() as i32;
            if self.y >= world.highest_ground(ix, iz) as f32 - 0.5 {
                self.hp -= 1.0;
                self.hurt = 0.25;
            }
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
            if hostile_active && dist < 18.0 && dist > 1.0 {
                let sp = if self.kind == MobKind::Spider { 3.1 } else { 2.4 };
                self.vx = dx / dist * sp;
                self.vz = dz / dist * sp;
                self.yaw = dx.atan2(dz);
            } else if self.timer <= 0.0 {
                self.timer = 1.5 + ((tick as f32 * 0.13) % 2.0);
                let ang = ((tick as f32 * 1.618 + self.x * 0.37) % 6.283) - 3.1415;
                if (tick % 3) == 0 {
                    self.vx = 0.0;
                    self.vz = 0.0;
                } else {
                    self.vx = ang.sin() * 1.4;
                    self.vz = ang.cos() * 1.4;
                    self.yaw = ang;
                }
            }
        }

        let speed_xz = (self.vx * self.vx + self.vz * self.vz).sqrt();
        if speed_xz > 0.1 && self.on_ground {
            self.walk += speed_xz * dt * 3.0;
        }

        let nx = self.x + self.vx * dt;
        let nz = self.z + self.vz * dt;
        let blocked_x = world::is_solid(world.get(nx.floor() as i32, (self.y + 0.4).floor() as i32, self.z.floor() as i32));
        let blocked_z = world::is_solid(world.get(self.x.floor() as i32, (self.y + 0.4).floor() as i32, nz.floor() as i32));

        if (blocked_x || blocked_z) && self.on_ground {
            let head_clear = !world::is_solid(world.get(nx.floor() as i32, (self.y + 1.5).floor() as i32, nz.floor() as i32));
            if head_clear {
                self.vy = 6.8;
                self.on_ground = false;
            }
        }
        if !blocked_x { self.x = nx; }
        if !blocked_z { self.z = nz; }

        self.vy -= 22.0 * dt;
        let ny = self.y + self.vy * dt;
        let foot_b = world.get(self.x.floor() as i32, ny.floor() as i32, self.z.floor() as i32);
        let head_b = world.get(self.x.floor() as i32, (ny + self.kind.height() - 0.1).floor() as i32, self.z.floor() as i32);
        if self.vy < 0.0 && world::is_solid(foot_b) {
            self.y = ny.floor() + 1.0;
            self.vy = 0.0;
            self.on_ground = true;
        } else if self.vy > 0.0 && world::is_solid(head_b) {
            self.vy = 0.0;
        } else {
            self.y = ny;
            self.on_ground = false;
        }
        if self.y < 1.0 { self.y = 40.0; self.vy = 0.0; }
    }
}

pub fn random_kind(seed: u32, day: f32) -> MobKind {
    if day < 0.42 && seed % 2 == 0 {
        match (seed >> 3) % 5 {
            0 => MobKind::Zombie,
            1 => MobKind::Skeleton,
            2 => MobKind::Creeper,
            3 => MobKind::Spider,
            _ => MobKind::Enderman,
        }
    } else {
        match (seed >> 3) % 5 {
            0 => MobKind::Sheep,
            1 => MobKind::Cow,
            2 => MobKind::Pig,
            3 => MobKind::Chicken,
            _ => MobKind::Sheep,
        }
    }
}

#[derive(Clone)]
pub struct Drop {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vy: f32,
    pub item: u16,
}

impl Drop {
    pub fn new(x: f32, y: f32, z: f32, item: u16) -> Self {
        Self { x, y, z, vy: 2.0, item }
    }
    pub fn update(&mut self, world: &mut World, dt: f32) {
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
