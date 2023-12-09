use bevy::{prelude::*, sprite::Mesh2dHandle};
use bevy_xpbd_2d::prelude::*;
use leafwing_input_manager::prelude::*;
use rand::Rng;

#[derive(PhysicsLayer)]
enum Layer {
    Player,
    Bullet,
    Enemy,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<Action>::default());
        app.add_systems(Startup, startup);
        app.add_systems(PostUpdate, (player_spawn, update_player_radius));
        app.add_systems(Update, (attack_system, remove_bullets));
        app.insert_resource(Gravity(Vec2::NEG_Y * 300.0));
    }
}

#[derive(Resource)]
struct PlayerResource {
    bullet_mesh: Handle<Mesh>,
    bullet_material: Handle<ColorMaterial>,
    attack_sound: Handle<AudioSource>,
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(PlayerResource {
        bullet_mesh: meshes.add(
            shape::Circle {
                radius: 8.0,
                vertices: 6,
            }
            .into(),
        ),
        bullet_material: color_materials.add(ColorMaterial::from(Color::CYAN)),
        attack_sound: asset_server.load("sounds/splash_03.ogg"),
    });
}

#[derive(Component)]
pub struct Player {
    pub radius: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self { radius: 50.0 }
    }
}

#[derive(Actionlike, Reflect, Clone, Hash, PartialEq, Eq, Debug)]
enum Action {
    Attack,
    Dodge,
}

fn player_spawn(
    mut commands: Commands,
    q: Query<(Entity, &Player, Option<&Transform>), Added<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, player, transform) in q.iter() {
        commands
            .entity(entity)
            .insert(ColorMesh2dBundle {
                mesh: Mesh2dHandle(
                    meshes.add(
                        shape::Circle {
                            radius: player.radius,
                            vertices: 8,
                        }
                        .into(),
                    ),
                ),
                material: color_materials.add(ColorMaterial::from(Color::CYAN)),
                transform: transform.copied().unwrap_or_default(),
                ..default()
            })
            .insert(InputManagerBundle::<Action> {
                input_map: InputMap::new([
                    (MouseButton::Left, Action::Attack),
                    (MouseButton::Right, Action::Dodge),
                ]),
                ..default()
            })
            .insert((
                RigidBody::Kinematic,
                CollisionLayers::new([Layer::Player], [Layer::Enemy]),
                // Collider is added in update_player_radius
            ));
    }
}

fn update_player_radius(
    mut commands: Commands,
    q: Query<(Entity, &Player), Changed<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, player) in &q {
        commands
            .entity(entity)
            .insert(Mesh2dHandle(
                meshes.add(
                    shape::Circle {
                        radius: player.radius,
                        vertices: 8,
                    }
                    .into(),
                ),
            ))
            .insert(Collider::ball(player.radius * 0.8));
    }
}

#[derive(Component)]
struct Bullet;

fn attack_system(
    mut commands: Commands,
    mut q: Query<
        (Entity, &ActionState<Action>, &Transform, &mut Player),
        (With<Player>, Changed<ActionState<Action>>),
    >,
    res: Res<PlayerResource>,
) {
    for (id, state, transform, mut player) in &mut q {
        if state.just_pressed(Action::Attack) {
            commands.entity(id).insert(AudioBundle {
                source: res.attack_sound.clone(),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Remove,
                    ..default()
                }
            });

            let bullet_bundle = ColorMesh2dBundle {
                mesh: Mesh2dHandle(res.bullet_mesh.clone()),
                material: res.bullet_material.clone(),
                ..default()
            };

            player.radius *= 0.9;

            let mut rng = rand::thread_rng();
            for _ in 0..32 {
                let dev = Vec2::new(rng.gen::<f32>() - 0.5, rng.gen::<f32>() - 0.5) * 100.0;

                commands
                    .spawn(ColorMesh2dBundle {
                        transform: Transform::from_translation(
                            transform.translation + dev.extend(0.0),
                        ),
                        ..bullet_bundle.clone()
                    })
                    .insert((
                        Bullet,
                        RigidBody::Dynamic,
                        Collider::ball(8.0),
                        CollisionLayers::new([Layer::Bullet], [Layer::Enemy]),
                        LinearVelocity(dev * 2.0),
                    ));
            }
        }
    }
}

fn remove_bullets(mut commands: Commands, bullets: Query<(Entity, &Transform), With<Bullet>>) {
    for (entity, transform) in &bullets {
        if transform.translation.length() > 1000.0 {
            commands.entity(entity).despawn();
        }
    }
}
