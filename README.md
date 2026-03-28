# Bevy Fur

A real-time fur rendering library for **Bevy**, demonstrating four geometry-expansion techniques. Each approach is implemented as a **WGSL compute shader** that expands the base mesh on the GPU every frame.

## Getting Started

Add `bevy_fur` to your `Cargo.toml`:

```toml
[dependencies]
bevy_fur = { path = "..." }
```

Register `FurPlugin`, then spawn any entity with a `Fur` component and `SyncToRenderWorld`:

```rust
use bevy::prelude::*;
use bevy_fur::{Fur, FurMode, FurPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FurPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Fur {
            mesh: asset_server.load("models/MyMesh.glb#Mesh0/Primitive0"),
            mode: FurMode::default(), // Approach4 — animated fur
        },
        SyncToRenderWorld,
    ));
}
```

## Demo

### Dependencies

* Rust toolchain (https://rustup.rs/)
* Vulkan/Metal/DX12-capable GPU

### Running the Demo

```bash
cargo run --example demo
```

### Controls of the Demo

| Key | Action |
|-----|--------|
| `1` | Approach 1 - layer shells |
| `2` | Approach 2 - edge ridges |
| `3` | Approach 3 - centre cone |
| `4` | Approach 4 - animated fur (default) |
| `Esc` | Quit |

## Authors

- **Philipp Haustein** - MrInformatic

## License

This project is licensed under the MIT License - see the LICENSE file for details

## Acknowledgments

Inspiration, code snippets, etc.