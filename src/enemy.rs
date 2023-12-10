use bevy::{
    audio::{Volume, VolumeLevel},
    prelude::*,
};
use bevy_debug_text_overlay::screen_print;
use bevy_xpbd_2d::prelude::*;

use crate::SCREEN_WIDTH;

fn startup(mut commands: Commands) {
    commands.init_resource::<EnemyResource>();
}

#[derive(Component, Default)]
pub struct Enemy;

pub const ENEMY_WIDTH: f32 = 80.0;

#[derive(Component, Debug)]
pub struct EnemyController {
    state: EnemyState,
    attack_pos: Vec<Vec2>,
}

#[derive(PartialEq, Debug, Default)]
pub enum EnemyState {
    Attacking,
    #[default]
    Moving,
}

#[derive(Bundle)]
pub struct EnemyBundle {
    pub enemy: Enemy,
    pub controller: EnemyController,
    pub sprite: SpriteBundle,
    // pub line_up_bullets: LineUpBullets,
}

pub fn spawn_enemy(commands: &mut Commands, pos: Vec2) {
    const ATTACK_NUM: u32 = 3;

    let mut attack_pos = vec![
        Vec2::new(SCREEN_WIDTH / ATTACK_NUM as f32 - SCREEN_WIDTH / 2., pos.y),
        Vec2::new(
            2. * SCREEN_WIDTH / ATTACK_NUM as f32 - SCREEN_WIDTH / 2.,
            pos.y,
        ),
        Vec2::new(SCREEN_WIDTH / 2. + ENEMY_WIDTH / 2., pos.y),
    ];
    attack_pos.reverse();

    commands
        .spawn(EnemyBundle {
            sprite: SpriteBundle {
                transform: Transform::from_translation(pos.extend(0.0)),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(ENEMY_WIDTH, ENEMY_WIDTH)),
                    ..default()
                },
                ..default()
            },
            enemy: Enemy,
            controller: EnemyController {
                state: EnemyState::Moving,
                attack_pos,
            },
        })
        .insert((Collider::ball(ENEMY_WIDTH / 2.0), RigidBody::Kinematic));
}

#[derive(Resource)]
struct EnemyResource {
    image: Handle<Image>,
    bullet_sound: Handle<AudioSource>,
}

impl FromWorld for EnemyResource {
    fn from_world(world: &mut World) -> Self {
        let image = world
            .get_resource::<AssetServer>()
            .unwrap()
            .load::<Image>("sprites/snow.png");

        let bullet_sound = world
            .get_resource::<AssetServer>()
            .unwrap()
            .load("sounds/ice.ogg");

        Self {
            image,
            bullet_sound,
        }
    }
}

impl EnemyBundle {
    fn set_handles(
        mut new_enemies: Query<&mut Handle<Image>, Added<Enemy>>,
        res: Res<EnemyResource>,
    ) {
        for mut image in new_enemies.iter_mut() {
            *image = res.image.clone();
        }
    }
}

fn enemy_state_behavior(
    mut commands: Commands,
    mut enemies: Query<(Entity, &EnemyController), Changed<EnemyController>>,
) {
    for (entity, ctrl) in &mut enemies {
        screen_print!("Enemy({:?}) ctrl: {:?}", entity, ctrl);

        match ctrl.state {
            EnemyState::Attacking => {
                commands.entity(entity).insert(LineUpBullets::default());
            }
            EnemyState::Moving => {
                commands.entity(entity).remove::<LineUpBullets>();
            }
        }
    }
}

fn enemy_movement(
    mut commands: Commands,
    mut enemies: Query<(Entity, &mut Transform, &mut EnemyController)>,
    time: Res<Time>,
) {
    for (id, mut transform, mut ctrl) in &mut enemies {
        if ctrl.state != EnemyState::Moving {
            continue;
        }

        let Some(move_target) = ctrl.attack_pos.last() else {
            commands.entity(id).despawn();
            continue;
        };

        screen_print!("movement_target: {}", move_target);

        let diff = *move_target - transform.translation.xy();
        let movement = diff.normalize() * 100.0 * time.delta_seconds();
        screen_print!("diff = {}, movement = {}", diff, movement);

        // If the enemy is close to the target, just move it to the target.
        if diff.length() < 0.1 || (*move_target - transform.translation.xy()).length() <= movement.length() {
            transform.translation = move_target.extend(transform.translation.z);

            ctrl.attack_pos.pop();
            if ctrl.attack_pos.len() == 0 {
                commands.entity(id).despawn();
            } else {
                ctrl.state = EnemyState::Attacking;
            }
        } else {
            // Otherwise, move it to the direction of the target.
            transform.translation += movement.extend(0.0);
        }
    }
}

fn enemy_attack_done(
    mut enemies: Query<(&mut EnemyController, &LineUpBullets), Changed<LineUpBullets>>,
) {
    for (mut ctrl, bullets) in &mut enemies {
        if ctrl.state == EnemyState::Attacking && bullets.done {
            ctrl.state = EnemyState::Moving;
        }
    }
}

#[derive(Component)]
struct Bullet;

// fn spawn_enemy_bullet(
//     mut commands: Commands,
//     enemies: Query<&Transform, Added<Enemy>>,
//     assets: Res<AssetServer>,
// ) {
//     for transform in &enemies {
//         commands
//             .spawn(SpriteBundle {
//                 transform: transform.with_scale(Vec3::splat(0.1)),
//                 texture: assets.load("sprites/typhoon_white.png"),
//                 ..default()
//             })
//             .insert(Bullet);
//     }
// }

fn rotate_bullets(time: Res<Time>, mut bullets: Query<&mut Transform, With<Bullet>>) {
    for mut transform in &mut bullets {
        transform.rotate(Quat::from_rotation_z(time.delta_seconds() * 2.0));
    }
}

#[derive(Component)]
pub struct LineUpBullets {
    num: u32,
    bullets: Vec<(Entity, Vec3)>,
    next_timer: Timer,
    angle: f32,
    done: bool,
}

impl Default for LineUpBullets {
    fn default() -> Self {
        Self {
            num: 16,
            bullets: Vec::new(),
            next_timer: Timer::from_seconds(0.05, TimerMode::Repeating),
            angle: 0.0,
            done: false,
        }
    }
}

#[derive(Component)]
struct StillBullet;

fn spawn_still_bullet(
    mut commands: Commands,
    enemies: Query<(Entity, &Transform), Added<StillBullet>>,
    assets: Res<AssetServer>,
) {
    for (entity, transform) in &enemies {
        commands
            .entity(entity)
            .insert(SpriteBundle {
                texture: assets.load("sprites/typhoon_white.png"),
                transform: transform.with_scale(Vec3::splat(0.05)),
                ..default()
            })
            .insert(Bullet);
    }
}

#[derive(Component)]
struct StraightBullet(Vec3);

fn line_up_bullets_system(
    mut commands: Commands,
    mut q: Query<(&mut LineUpBullets, &Transform)>,
    time: Res<Time>,
    enemy_res: Res<EnemyResource>,
) {
    for (mut line_up_bullets, transform) in &mut q {
        if line_up_bullets.next_timer.tick(time.delta()).finished() {
            if line_up_bullets.bullets.len() >= line_up_bullets.num as usize {
                for (entity, delta) in &line_up_bullets.bullets {
                    commands
                        .entity(*entity)
                        .remove::<StillBullet>()
                        .insert(StraightBullet(delta.clone()));
                }
                line_up_bullets.done = true;
                continue;
            }

            let angle = line_up_bullets.angle;
            let delta = Vec3::new(-angle.sin(), angle.cos(), 0.0) * 100.0;
            let id = commands
                .spawn(StillBullet)
                .insert(Transform::from_translation(transform.translation + delta))
                .insert(AudioBundle {
                    source: enemy_res.bullet_sound.clone(),
                    settings: PlaybackSettings {
                        mode: bevy::audio::PlaybackMode::Once,
                        volume: Volume::Relative(VolumeLevel::new(0.2)),
                        ..default()
                    },
                })
                .id();
            line_up_bullets.bullets.push((id, delta));
            line_up_bullets.angle += 2.0 * std::f32::consts::PI / line_up_bullets.num as f32;
        }
    }
}

fn move_straight_bullet(time: Res<Time>, mut bullets: Query<(&mut Transform, &StraightBullet)>) {
    for (mut transform, StraightBullet(delta)) in &mut bullets {
        transform.translation += *delta * time.delta_seconds();
    }
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_systems(PostUpdate, EnemyBundle::set_handles);
        // .add_systems(Update, spawn_enemy_bullet)
        app.add_systems(Update, rotate_bullets);

        app.add_systems(Update, line_up_bullets_system)
            .add_systems(Update, spawn_still_bullet)
            .add_systems(Update, move_straight_bullet);

        app.add_systems(
            Update,
            (enemy_state_behavior, enemy_movement, enemy_attack_done).after(line_up_bullets_system),
        );
    }
}
