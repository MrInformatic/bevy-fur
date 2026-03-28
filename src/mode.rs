use bevy::prelude::*;

// ---- FurMode ---------------------------------------------------

/// Which fur geometry approach is active (switchable via keys 1–4).
#[derive(Component, Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum FurMode {
    Approach1,
    Approach2,
    Approach3,
    #[default]
    Approach4,
}

impl FurMode {
    pub fn verts_per_tri(&self) -> u32 {
        match self {
            FurMode::Approach1 => 75,
            FurMode::Approach2 => 21,
            FurMode::Approach3 => 9,
            FurMode::Approach4 => 279,
        }
    }
}
