use bevy::{
    app::AppExit, input::ButtonInput, log::info, prelude::*, render::sync_world::SyncToRenderWorld,
};
use bevy_fur::{Fur, FurMode, FurPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Ogre Geometry — Fur Demo".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FurPlugin)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, handle_input)
        .run();
}

// ---- Systems ---------------------------------------------------

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Fur {
            mesh: asset_server.load("models/Fur.glb#Mesh0/Primitive0"),
            mode: FurMode::default(),
        },
        SyncToRenderWorld,
    ));

    commands.spawn((
        Camera3d::default(),
        Projection::from(PerspectiveProjection {
            near: 0.01,
            far: 100.0,
            ..default()
        }),
        Transform::from_xyz(0.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        AmbientLight {
            color: Color::srgb(0.0, 0.1, 0.1),
            brightness: 1.0,
            affects_lightmapped_meshes: true,
        },
    ));

    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_xyz(-100.0, 100.0, -100.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: MessageWriter<AppExit>,
    mut furs: Query<&mut Fur>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }

    let new_mode = if keys.just_pressed(KeyCode::Digit1) {
        info!("Switched to approach 1 — layer shells");
        Some(FurMode::Approach1)
    } else if keys.just_pressed(KeyCode::Digit2) {
        info!("Switched to approach 2 — edge ridges");
        Some(FurMode::Approach2)
    } else if keys.just_pressed(KeyCode::Digit3) {
        info!("Switched to approach 3 — centre cone");
        Some(FurMode::Approach3)
    } else if keys.just_pressed(KeyCode::Digit4) {
        info!("Switched to approach 4 — animated fur");
        Some(FurMode::Approach4)
    } else {
        None
    };

    if let Some(mode) = new_mode {
        for mut fur in &mut furs {
            fur.mode = mode;
        }
    }
}
