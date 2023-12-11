mod damage_effect;

use bevy::{prelude::*, sprite::Mesh2dHandle};
use bevy_debug_text_overlay::screen_print;
use bevy_xpbd_2d::prelude::*;
use leafwing_input_manager::prelude::*;
use rand::Rng;

use crate::{item::Item, MainCamera, MyLayer};

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

impl Player {
    /// Changes the player's radius by the given amount in terms of area.
    pub fn increase(&mut self, by: f32) {
        self.radius = (self.radius.powi(2) + by).clamp(0., 1600.).sqrt();
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
                            vertices: 32,
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
                CollisionLayers::new(
                    [MyLayer::Player],
                    [MyLayer::Enemy, MyLayer::EnemyBullet, MyLayer::Item],
                ),
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
                        vertices: 32,
                    }
                    .into(),
                ),
            ))
            .insert(Collider::ball(player.radius * 0.8));
    }
}

#[derive(Component)]
pub struct PlayerBullet;

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
                },
            });

            let bullet_bundle = ColorMesh2dBundle {
                mesh: Mesh2dHandle(res.bullet_mesh.clone()),
                material: res.bullet_material.clone(),
                ..default()
            };

            let num = (player.radius / 50. * 32.) as usize;

            player.increase(-100.);
            screen_print!("Player radius: {}", player.radius);

            let mut rng = rand::thread_rng();
            for _ in 0..num {
                let r = rng.gen::<f32>() * 50.0;
                let theta = rng.gen::<f32>() * std::f32::consts::PI * 2.0;
                let pos = Vec2::new(theta.cos(), theta.sin()) * r;

                commands
                    .spawn(ColorMesh2dBundle {
                        transform: Transform::from_translation(
                            transform.translation + pos.extend(0.0),
                        ),
                        ..bullet_bundle.clone()
                    })
                    .insert((
                        PlayerBullet,
                        RigidBody::Dynamic,
                        Collider::ball(8.0),
                        CollisionLayers::new([MyLayer::PlayerBullet], [MyLayer::Enemy]),
                        LinearVelocity(pos * 2.0),
                    ));
            }
        }
    }
}

fn remove_bullets(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform), With<PlayerBullet>>,
    camera: Query<&Transform, With<MainCamera>>,
) {
    let camera = camera.single();
    for (entity, transform) in &bullets {
        if (transform.translation - camera.translation).length() > 1000.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// system to handle player getting an item
fn player_item_system(
    mut commands: Commands,
    mut player: Query<(&mut Player, &CollidingEntities)>,
    items: Query<&Item>,
) {
    let Ok((mut player, collisions)) = player.get_single_mut() else {
        return;
    };
    for &collision in collisions.iter() {
        if items.contains(collision) {
            player.increase(10.);
            commands.entity(collision).despawn_recursive();
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(damage_effect::DamageEffectPlugin);

        app.add_plugins(InputManagerPlugin::<Action>::default());
        app.add_systems(Startup, startup);
        app.add_systems(PostUpdate, (player_spawn, update_player_radius));
        app.add_systems(Update, (attack_system, remove_bullets));
        app.add_systems(Update, player_item_system);
        app.insert_resource(Gravity(Vec2::NEG_Y * 300.0));
    }
}
