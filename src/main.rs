mod background;
mod boss;
mod damage;
mod enemy;
mod health;
mod item;
mod level;
mod player;
mod title;

use background::{Background2dBundle, BackgroundMaterial, BackgroundPlugin};
use bevy::{
    asset::AssetMetaCheck,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    sprite::Mesh2dHandle,
    window::{PrimaryWindow, WindowResolution},
};
use bevy_debug_text_overlay::{screen_print, OverlayPlugin};
use bevy_framepace::FramepacePlugin;
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_tweening::Animator;
use bevy_xpbd_2d::prelude::*;
use boss::{Boss, BossPlugin};
use damage::{BossDiedEvent, DamagePlugin};
use enemy::{Enemy, EnemyBullet, EnemyPlugin, ENEMY_SIZE};
use health::HealthBarPlugin;
use item::{Item, ItemPlugin};
use level::{Level, LevelPlugin};
use player::{Player, PlayerBullet, PlayerDiedEvent, PlayerPlugin};
use title::{TitlePlugin, TitleRoot};

pub const SCREEN_WIDTH: f32 = 800.0;
pub const SCREEN_HEIGHT: f32 = 600.0;

#[derive(PhysicsLayer)]
pub enum MyLayer {
    Player,
    PlayerBullet,
    Enemy,
    EnemyBullet,
    Item,
}

#[derive(Debug, States, Default, Hash, PartialEq, Eq, Clone, Copy)]
enum GameState {
    #[default]
    Title,
    Main,
    GameOver,
}

fn main() {
    let mut app = App::new();

    app.insert_resource(AssetMetaCheck::Never).add_plugins(
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(SCREEN_WIDTH, SCREEN_HEIGHT),
                resizable: false,
                ..default()
            }),
            ..default()
        }), // .set(AssetPlugin {
            //     watch_for_changes_override: Some(true),
            //     ..Default::default()
            // }),
    );
    // app.insert_resource(ClearColor(Color::rgb(
    //     150. / 255. / 2.,
    //     180. / 255. / 2.,
    //     218. / 255. / 2.,
    // )));

    app.add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Update, |diagnostics: Res<DiagnosticsStore>| {
            if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
                // screen_print!("FPS: {}", fps.average().unwrap_or_default());
            }
        });

    app.add_plugins(PhysicsPlugins::default())
        // .add_plugins(PhysicsDebugPlugin::default())
        // .add_plugins(OverlayPlugin::default())
        // .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(FramepacePlugin);

    app.add_state::<GameState>();

    app.add_plugins(PlayerPlugin)
        .add_plugins(EnemyPlugin)
        .add_plugins(HealthBarPlugin)
        .add_plugins(DamagePlugin)
        .add_plugins(ItemPlugin)
        .add_plugins(LevelPlugin)
        .add_plugins(BossPlugin)
        .add_plugins(TitlePlugin)
        .add_plugins(BackgroundPlugin)
        .insert_resource(ScrollSpeed(1.))
        .add_systems(Startup, setup)
        .add_systems(PostUpdate, scroll_background)
        .add_systems(PreUpdate, update_mouse_pos)
        .add_systems(
            Update,
            (|mouse_pos: Option<Res<MouseWorldPos>>,
              mut q: Query<&mut Transform, With<Player>>| {
                let Some(mouse_pos) = mouse_pos else { return };
                if !mouse_pos.is_changed() {
                    return;
                }

                for mut transform in q.iter_mut() {
                    // transform.translation = transform.translation
                    //     + (mouse_pos.0 - transform.translation.xy()).extend(0.0) * 0.3;
                    transform.translation = mouse_pos.0.extend(2.0);
                }
            })
            .run_if(in_state(GameState::Main)),
        )
        .add_systems(
            Update,
            (|mut q: Query<&mut Transform, Or<(With<MainCamera>, With<TitleRoot>)>>,
              time: Res<Time>| {
                for mut tf in &mut q {
                    tf.translation.y -= 10. * time.delta_seconds();
                }
            })
            .run_if(in_state(GameState::Title)),
        )
        .add_systems(OnEnter(GameState::Main), |mut commands: Commands| {
            // screen_print!("OnEnter(GameState::Main)");
            commands.insert_resource(ScrollSpeed(1.));
            commands.spawn((Player::default(), Transform::from_translation(2. * Vec3::Z)));
        })
        .add_systems(OnEnter(GameState::Main), setup_level)
        .add_systems(OnEnter(GameState::Title), |mut commands: Commands| {
            // screen_print!("OnEnter(GameState::Title)");
            commands.remove_resource::<Level>();
        })
        .add_systems(
            OnEnter(GameState::GameOver),
            |mut commands: Commands, animators: Query<Entity, With<Animator<Transform>>>| {
                for animator in animators.iter() {
                    commands.entity(animator).remove::<Animator<Transform>>();
                }
            },
        )
        .add_systems(
            PostUpdate,
            (|mut commands: Commands, mut state: ResMut<NextState<GameState>>| {
                // screen_print!("Game over");
                commands.insert_resource(GameOverTimer(Timer::from_seconds(1.5, TimerMode::Once)));
                commands.remove_resource::<PlayerDisposition>();
                state.set(GameState::GameOver);
            })
            .run_if(on_event::<PlayerDiedEvent>()),
        )
        .add_systems(
            Update,
            (|mut timer: ResMut<GameOverTimer>,
              time: Res<Time>,
              mut state: ResMut<NextState<GameState>>| {
                if timer.0.tick(time.delta()).just_finished() {
                    // screen_print!("Go back to title");
                    state.set(GameState::Title);
                }
            })
            .run_if(in_state(GameState::GameOver)),
        )
        .add_systems(
            OnExit(GameState::GameOver),
            |mut commands: Commands,
             mut camera: Query<&mut Transform, With<MainCamera>>,
             main_entities: Query<
                Entity,
                Or<(
                    With<Enemy>,
                    With<Boss>,
                    With<PlayerBullet>,
                    With<EnemyBullet>,
                    With<Item>,
                )>,
            >| {
                commands.remove_resource::<GameOverTimer>();
                commands.remove_resource::<Level>();

                camera.single_mut().translation.y = 0.;

                for e in &main_entities {
                    commands.entity(e).despawn_recursive();
                }
            },
        )
        .add_systems(
            Update,
            (|mut commands: Commands,
              mut state: ResMut<NextState<GameState>>,
              bullets: Query<Entity, With<EnemyBullet>>| {
                for bullet in &bullets {
                    commands.entity(bullet).despawn_recursive();
                }

                commands.insert_resource(GameOverTimer(Timer::from_seconds(4., TimerMode::Once)));
                state.set(GameState::GameOver);
                commands.insert_resource(ScrollSpeed(3.));
                commands.insert_resource(PlayerDisposition(0.));
            })
            .run_if(on_event::<BossDiedEvent>()),
        )
        .add_systems(
            Update,
            (|mut player: Query<&mut Transform, (With<Player>, Without<MainCamera>)>,
              camera: Query<&Transform, (With<MainCamera>, Without<Player>)>,
              res: Option<ResMut<PlayerDisposition>>,
              time: Res<Time>| {
                let Ok(mut player) = player.get_single_mut() else {
                    return;
                };
                let camera = camera.single();
                let disp;
                if let Some(mut res) = res {
                    disp = res.0;
                    res.0 += 20.0 * time.delta_seconds();
                } else {
                    disp = 0.;
                }
                player.translation = camera.translation - Vec3::Y * disp.powi(2);
            })
            .run_if(in_state(GameState::GameOver)),
        )
        // .add_systems(
        //     PostUpdate,
        //     |parent: Query<&Parent>, vis: Query<(Entity, Option<&Name>), With<Visibility>>| {
        //         for (id, name) in vis.iter().collect::<Vec<_>>() {
        //             if let Ok(parent) = parent.get(id) {
        //                 if !vis.contains(parent.get()) {
        //                     error!(
        //                         "Entity {:?} ({:?}) has parent without Visibility",
        //                         id,
        //                         name
        //                     );
        //                 }
        //             }
        //         }
        //     },
        // )
        .run();
}

#[derive(Resource)]
struct ScrollSpeed(f32);

#[derive(Resource)]
struct PlayerDisposition(f32);

fn setup_level(mut commands: Commands) {
    let mut enemies = Vec::new();
    for i in 0..3 {
        let enemy_positions = [
            Vec2::new(SCREEN_WIDTH / 3. - SCREEN_WIDTH / 2., -i as f32 * 100.),
            Vec2::new(
                2. * SCREEN_WIDTH / 3. - SCREEN_WIDTH / 2.,
                -(i + 1) as f32 * 100.,
            ),
            Vec2::new(SCREEN_WIDTH / 2. + ENEMY_SIZE / 2., -i as f32 * 100.),
        ];
        enemies.push((
            Vec2::new(-SCREEN_WIDTH / 2. - ENEMY_SIZE / 2., -i as f32 * 100.),
            enemy_positions.into_iter().rev().collect::<Vec<_>>().into(),
        ));
    }

    for i in 0..3 {
        let enemy_positions = [
            Vec2::new(
                SCREEN_WIDTH / 3. + SCREEN_WIDTH / 2.,
                -i as f32 * 100. - 400.,
            ),
            Vec2::new(
                -SCREEN_WIDTH / 3. + SCREEN_WIDTH / 2.,
                -i as f32 * 100. - 400.,
            ),
            Vec2::new(
                -SCREEN_WIDTH / 2. - ENEMY_SIZE / 2.,
                -i as f32 * 100. - 400.,
            ),
        ];
        enemies.push((
            Vec2::new(SCREEN_WIDTH / 2. + ENEMY_SIZE / 2., -i as f32 * 100. - 400.),
            enemy_positions.into_iter().rev().collect::<Vec<_>>().into(),
        ));
    }

    enemies.push((
        Vec2::new(0., -900.),
        vec![
            Vec2::new(0., -800.),
            Vec2::new(0., -750.),
            Vec2::new(SCREEN_WIDTH / 3., -800.),
            Vec2::new(SCREEN_WIDTH / 2. + ENEMY_SIZE / 2., -800.),
        ]
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .into(),
    ));

    let level = Level {
        enemies,
        boss_pos: Vec2::new(0., -1000.).into(),
    };
    commands.insert_resource(level);
}

#[derive(Component)]
struct MainCamera;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut bg_materials: ResMut<Assets<BackgroundMaterial>>,
) {
    // Add camera
    commands
        .spawn((Camera2dBundle::default(), MainCamera))
        .with_children(|c| {
            c.spawn(Background2dBundle {
                mesh: Mesh2dHandle(
                    meshes.add(shape::Quad::new(Vec2::new(SCREEN_WIDTH, SCREEN_HEIGHT)).into()),
                ),
                material: bg_materials.add(BackgroundMaterial {
                    scroll: 0.,
                    alpha: 1.,
                }),
                transform: Transform::from_translation(-Vec3::Z),
                ..default()
            });
        })
        .insert(Name::new("Background"));
}

fn scroll_background(
    camera: Query<&Transform, With<MainCamera>>,
    background: Query<&Handle<BackgroundMaterial>>,
    mut bg_materials: ResMut<Assets<BackgroundMaterial>>,
) {
    let camera_y = camera.single().translation.y;

    for handle in &background {
        let material = bg_materials.get_mut(handle).unwrap();
        material.scroll = camera_y;
    }
}

#[derive(Resource)]
struct GameOverTimer(Timer);

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
        // screen_print!("World coords: {}/{}", world_position.x, world_position.y);
    }
}
