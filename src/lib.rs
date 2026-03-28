pub mod mode;
mod render;
use bevy::asset::embedded_asset;
use bevy::prelude::*;
pub use mode::FurMode;

// ---- Fur component ---------------------------------------------

/// Attach to any entity to enable fur rendering on its mesh.
#[derive(Component, Clone)]
pub struct Fur {
    pub mesh: Handle<Mesh>,
    pub mode: FurMode,
}

// ---- Plugin ----------------------------------------------------

/// Adds the fur rendering infrastructure. Spawn entities with [`Fur`] to display fur.
pub struct FurPlugin;

impl Plugin for FurPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/fur_draw.wgsl");
        embedded_asset!(app, "shaders/fur_compute_1.wgsl");
        embedded_asset!(app, "shaders/fur_compute_2.wgsl");
        embedded_asset!(app, "shaders/fur_compute_3.wgsl");
        embedded_asset!(app, "shaders/fur_compute_4.wgsl");
        app.add_plugins(crate::render::FurRenderPlugin);
    }
}
