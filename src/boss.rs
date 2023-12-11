use std::time::Duration;

use bevy::{prelude::*, sprite::Mesh2dHandle};
use bevy_debug_text_overlay::screen_print;
use bevy_tweening::{
    lens::{TransformPositionLens, TransformRotationLens},
    Animator, BoxedTweenable, Delay, EaseFunction, Tracks, Tween, TweenCompleted, TweeningPlugin,
};
use bevy_xpbd_2d::components::CollisionLayers;
use bevy_xpbd_2d::prelude::*;
use rand::Rng;
use seldom_state::{
    prelude::StateMachine,
    trigger::{Done, DoneTrigger},
    StateMachinePlugin,
};

use crate::{
    enemy::{EnemyBullet, StraightBullet},
    health::Health,
    item::Item,
    level::ScrollDoneEvent,
    MyLayer, SCREEN_HEIGHT, SCREEN_WIDTH,
};

pub const BOSS_SIZE: f32 = 100.0;

pub const BOSS_PADDING: f32 = 25.0;

pub const BOSS_BULLET_SIZE: f32 = 6.;

pub struct BossPlugin;

impl Plugin for BossPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ScrollDoneEvent>();
        app.add_plugins(StateMachinePlugin);
        app.add_plugins(TweeningPlugin);

        app.add_systems(Startup, startup);
        app.add_systems(PostUpdate, spawn_boss);
        app.add_systems(Update, idle_system);
        app.add_systems(Update, attack_bottom_start);
        app.add_systems(Update, attack_top_start);
        app.add_systems(Update, attack_system);
        app.add_systems(Update, attack_end);
        app.add_systems(
            Update,
            (
                moving_to_top_start,
                moving_to_bottom_start,
                rotating_start,
                tween_end,
            ),
        );
        // app.add_systems(
        //     Update,
        //     |boss_bullets: Query<(), With<BossBullet>>, mut prev_cnt: Local<Option<usize>>| {
        //         let cnt = boss_bullets.iter().count();
        //         if Some(cnt) != *prev_cnt {
        //             debug!("Boss bullets: {}", cnt);
        //             *prev_cnt = Some(cnt);
        //         }
        //     },
        // );
    }
}

#[derive(Component)]
pub struct Boss;

fn spawn_boss(
    mut commands: Commands,
    query: Query<Entity, Added<Boss>>,
    asset_server: Res<AssetServer>,
) {
    for id in &query {
        screen_print!("Boss Spawned as {:?}", id);

        commands
            .entity(id)
            .insert((
                Name::new("Boss"),
                Health {
                    health: 200.,
                    max_health: 200.,
                },
            ))
            .insert((
                Idle,
                StateMachine::default()
                    .trans::<Idle>(DoneTrigger::Success, AttackBottom)
                    .trans::<AttackBottom>(DoneTrigger::Success, MovingToTop)
                    .trans::<MovingToTop>(DoneTrigger::Success, AttackTop)
                    .trans::<AttackTop>(DoneTrigger::Success, MovingToBottom)
                    .trans::<MovingToBottom>(DoneTrigger::Success, Rotating)
                    .trans::<Rotating>(DoneTrigger::Success, AttackBottom),
            ))
            .insert((
                Collider::cuboid(BOSS_SIZE, BOSS_SIZE),
                CollisionLayers::new([MyLayer::Enemy], [MyLayer::PlayerBullet]),
                RigidBody::Kinematic,
            ))
            .with_children(|parent| {
                parent.spawn(SpriteBundle {
                    sprite: Sprite {
                        // color: Color::GREEN,
                        custom_size: Vec2::splat(BOSS_SIZE).into(),
                        ..default()
                    },
                    texture: asset_server.load("sprites/umbrella.png"),
                    ..Default::default()
                });
            });
    }
}

#[derive(Resource)]
struct BossResource {
    bullet_mesh: Handle<Mesh>,
    bullet_material: Handle<ColorMaterial>,
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(BossResource {
        bullet_mesh: meshes.add(
            shape::Circle {
                radius: BOSS_BULLET_SIZE,
                vertices: 6,
            }
            .into(),
        ),
        bullet_material: color_materials.add(ColorMaterial::from(Color::CYAN)),
    })
}

// Boss states
#[derive(Clone, Component, Reflect)]
struct Idle;

#[derive(Clone, Component, Reflect)]
struct AttackBottom;

#[derive(Clone, Component, Reflect)]
struct MovingToTop;

#[derive(Clone, Component, Reflect)]
struct AttackTop;

#[derive(Clone, Component, Reflect)]
struct MovingToBottom;

#[derive(Clone, Component, Reflect)]
struct Rotating;

fn idle_system(
    mut commands: Commands,
    boss: Query<Entity, With<Boss>>,
    mut scroll_done_event: EventReader<ScrollDoneEvent>,
) {
    let Ok(id) = boss.get_single() else {
        return;
    };

    if scroll_done_event.read().count() > 0 {
        debug!("Done Idle");
        commands.entity(id).insert(Done::Success);
    }
}

#[derive(Component)]
struct BossBullet;

const ATTACK_NUM: u32 = 64;

#[derive(Component)]
struct AttackState {
    direction: Vec2,
    timer: Timer,
    num: u32,
}

fn attack_bottom_start(mut commands: Commands, boss: Query<Entity, Added<AttackBottom>>) {
    let Ok(id) = boss.get_single() else {
        return;
    };

    debug!("Start AttackBottom");

    commands.entity(id).insert(AttackState {
        timer: Timer::new(Duration::from_millis(100), TimerMode::Repeating),
        num: 0,
        direction: Vec2::Y,
    });
}

fn attack_top_start(mut commands: Commands, boss: Query<Entity, Added<AttackTop>>) {
    let Ok(id) = boss.get_single() else {
        return;
    };

    debug!("Start AttackBottom");

    commands.entity(id).insert(AttackState {
        timer: Timer::new(Duration::from_millis(100), TimerMode::Repeating),
        num: 0,
        direction: -Vec2::Y,
    });
}

fn attack_system(
    mut commands: Commands,
    mut boss: Query<(&Transform, &mut AttackState)>,
    time: Res<Time<Virtual>>,
    res: Res<BossResource>,
) {
    for (transform, mut state) in &mut boss {
        if state.num < ATTACK_NUM && state.timer.tick(time.elapsed()).just_finished() {
            let num = state.num;

            let center_angle = -state.direction.angle_between(Vec2::X);
            let start_angle = center_angle + std::f32::consts::PI / 2.;

            let angle = start_angle - std::f32::consts::PI / ATTACK_NUM as f32 * num as f32;

            let direction = Vec2::new(angle.cos(), angle.sin());

            let bullet_pos = transform.translation + direction.extend(0.) * BOSS_SIZE / 2. * 0.8;

            let id = commands
                .spawn((
                    Name::new("BossBullet"),
                    BossBullet,
                    EnemyBullet,
                    StraightBullet(direction.extend(0.) * 50.),
                    ColorMesh2dBundle {
                        transform: Transform::from_translation(bullet_pos.xy().extend(5.)),
                        mesh: Mesh2dHandle(res.bullet_mesh.clone()),
                        material: res.bullet_material.clone(),
                        ..default()
                    },
                    Collider::ball(BOSS_BULLET_SIZE / 2. * 0.6),
                    CollisionLayers::new([MyLayer::EnemyBullet], [MyLayer::Player]),
                    RigidBody::Kinematic,
                ))
                .id();
            debug!("boss position = {transform:?}");
            debug!("Spawned BossBullet({id:?}) at {bullet_pos:?}, direction = {direction:?}");

            state.num += 1;
        }
    }
}

fn attack_end(mut commands: Commands, boss: Query<(Entity, &GlobalTransform, &AttackState)>) {
    let Ok((id, transform, state)) = boss.get_single() else {
        return;
    };

    if state.num >= ATTACK_NUM {
        debug!("Done Attack");

        // Spawn items
        let mut rng = rand::thread_rng();
        for _ in 0..16 {
            let r = rng.gen::<f32>() * 50.0;
            let theta = rng.gen::<f32>() * std::f32::consts::PI * 2.0;
            let pos = Vec2::new(theta.cos(), theta.sin()) * r;
            commands.spawn((
                Item,
                SpatialBundle::from_transform(Transform::from_translation(
                    transform.translation() + pos.extend(0.0),
                )),
            ));
        }

        commands
            .entity(id)
            .remove::<AttackState>()
            .insert(Done::Success);
    }
}

fn moving_to_top_start(
    mut commands: Commands,
    boss: Query<(Entity, &Transform), Added<MovingToTop>>,
) {
    let Ok((id, &transform)) = boss.get_single() else {
        return;
    };

    let tween_transform = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_secs(2),
        TransformPositionLens {
            start: transform.translation,
            end: transform.translation
                + Vec3::new(-SCREEN_WIDTH / 2. + BOSS_SIZE / 2. + BOSS_PADDING, 0., 0.),
        },
    )
    .then(Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_secs(2),
        TransformPositionLens {
            start: transform.translation
                + Vec3::new(-SCREEN_WIDTH / 2. + BOSS_SIZE / 2. + BOSS_PADDING, 0., 0.),
            end: transform.translation
                + Vec3::new(
                    -SCREEN_WIDTH / 2. + BOSS_SIZE / 2. + BOSS_PADDING,
                    SCREEN_HEIGHT - BOSS_SIZE - BOSS_PADDING * 2.,
                    0.,
                ),
        },
    ))
    .then(
        Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(2),
            TransformPositionLens {
                start: transform.translation
                    + Vec3::new(
                        -SCREEN_WIDTH / 2. + BOSS_SIZE / 2. + BOSS_PADDING,
                        SCREEN_HEIGHT - BOSS_SIZE - BOSS_PADDING * 2.,
                        0.,
                    ),
                end: transform.translation
                    + Vec3::new(0., SCREEN_HEIGHT - BOSS_SIZE - BOSS_PADDING * 2., 0.),
            },
        )
        .with_completed_event(0),
    );

    let tween_rotation = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_secs(6),
        TransformRotationLens {
            start: transform.rotation,
            end: transform.rotation * Quat::from_rotation_z(std::f32::consts::PI),
        },
    );

    commands.entity(id).insert(Animator::new(Tracks::new([
        Box::new(tween_transform) as BoxedTweenable<_>,
        Box::new(tween_rotation),
    ])));
}

fn moving_to_bottom_start(
    mut commands: Commands,
    boss: Query<(Entity, &Transform), Added<MovingToBottom>>,
) {
    let Ok((id, &transform)) = boss.get_single() else {
        return;
    };

    let tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_secs(2),
        TransformPositionLens {
            start: transform.translation,
            end: transform.translation
                + Vec3::new(SCREEN_WIDTH / 2. - BOSS_SIZE / 2. - BOSS_PADDING, 0., 0.),
        },
    )
    .then(Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_secs(2),
        TransformPositionLens {
            start: transform.translation
                + Vec3::new(SCREEN_WIDTH / 2. - BOSS_SIZE / 2. - BOSS_PADDING, 0., 0.),
            end: transform.translation
                + Vec3::new(
                    SCREEN_WIDTH / 2. - BOSS_SIZE / 2. - BOSS_PADDING,
                    -SCREEN_HEIGHT + BOSS_SIZE + BOSS_PADDING * 2.,
                    0.,
                ),
        },
    ))
    .then(
        Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(2),
            TransformPositionLens {
                start: transform.translation
                    + Vec3::new(
                        SCREEN_WIDTH / 2. - BOSS_SIZE / 2. - BOSS_PADDING,
                        -SCREEN_HEIGHT + BOSS_SIZE + BOSS_PADDING * 2.,
                        0.,
                    ),
                end: transform.translation
                    + Vec3::new(0., -SCREEN_HEIGHT + BOSS_SIZE + BOSS_PADDING * 2., 0.),
            },
        )
        .with_completed_event(0),
    );

    commands.entity(id).insert(Animator::new(tween));
}

fn tween_end(
    mut commands: Commands,
    boss: Query<Entity, Or<(With<MovingToTop>, With<MovingToBottom>, With<Rotating>)>>,
    mut completed: EventReader<TweenCompleted>,
) {
    let Ok(boss) = boss.get_single() else {
        return;
    };

    for ev in completed.read() {
        if ev.entity == boss {
            debug!("Done tweening");
            commands.entity(boss).insert(Done::Success);
        }
    }
}

fn rotating_start(mut commands: Commands, boss: Query<Entity, Added<Rotating>>) {
    let Ok(id) = boss.get_single() else {
        return;
    };

    let tween_rotation = Delay::new(Duration::from_secs(2)).then(
        Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(3),
            TransformRotationLens {
                start: Quat::from_rotation_z(std::f32::consts::PI),
                end: Quat::IDENTITY,
            },
        )
        .with_completed_event(0),
    );

    commands.entity(id).insert(Animator::new(tween_rotation));
}
