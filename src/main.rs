mod enemy;
mod player;

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    window::{PrimaryWindow, WindowResolution},
};
use bevy_debug_text_overlay::{screen_print, OverlayPlugin};
use bevy_framepace::FramepacePlugin;
use bevy_xpbd_2d::prelude::*;
use enemy::{EnemyPlugin, ENEMY_WIDTH, spawn_enemy};
use player::{Player, PlayerPlugin};

pub const SCREEN_WIDTH: f32 = 800.0;
pub const SCREEN_HEIGHT: f32 = 600.0;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: WindowResolution::new(SCREEN_WIDTH, SCREEN_HEIGHT),
            resizable: false,
            ..default()
        }),
        ..default()
    }));

    app.add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Update, |diagnostics: Res<DiagnosticsStore>| {
            if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
                screen_print!("FPS: {}", fps.average().unwrap_or_default());
            }
        });

    app.add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin::default())
        .add_plugins(OverlayPlugin::default())
        .add_plugins(FramepacePlugin);

    app.add_plugins(PlayerPlugin)
        .add_plugins(EnemyPlugin)
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, update_mouse_pos)
        .add_systems(
            Update,
            |mouse_pos: Option<Res<MouseWorldPos>>, mut q: Query<&mut Transform, With<Player>>| {
                let Some(mouse_pos) = mouse_pos else { return };
                if !mouse_pos.is_changed() {
                    return;
                }

                for mut transform in q.iter_mut() {
                    transform.translation = transform.translation
                        + (mouse_pos.0 - transform.translation.xy()).extend(0.0) * 0.3;
                }
            },
        )
        .run();
}

fn setup(mut commands: Commands) {
    // Add camera
    commands.spawn(Camera2dBundle::default());

    spawn_enemy(
        &mut commands,
        Vec2::new(-SCREEN_WIDTH / 2.0 - ENEMY_WIDTH, 0.0),
    );

    commands.spawn((Player::default(), Transform::from_translation(Vec3::Z)));
}

#[derive(Resource, Default, PartialEq)]
struct MouseWorldPos(Vec2);

// https://bevy-cheatbook.github.io/cookbook/cursor2world.html
fn update_mouse_pos(
    mut commands: Commands,
    mycoords: Option<ResMut<MouseWorldPos>>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        if let Some(mut mycoords) = mycoords {
            mycoords.set_if_neq(MouseWorldPos(world_position));
        } else {
            commands.insert_resource(MouseWorldPos(world_position));
        }
        screen_print!("World coords: {}/{}", world_position.x, world_position.y);
    }
}