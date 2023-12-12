use bevy::{
    audio::{Volume, VolumeLevel},
    prelude::*,
};
use bevy_debug_text_overlay::screen_print;
use bevy_xpbd_2d::prelude::*;
use rand::Rng;

use crate::{
    boss::Boss,
    enemy::{Enemy, EnemyBullet},
    health::Health,
    item::Item,
    player::{Player, PlayerBullet},
};

#[derive(Event)]
pub struct BossDiedEvent;

#[derive(Resource)]
struct DamageRes {
    hit_sound: Handle<AudioSource>,
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(DamageRes {
        hit_sound: asset_server.load("sounds/explosion.ogg"),
    });
}

fn enemy_damage(
    mut commands: Commands,
    mut enemies: Query<
        (
            Entity,
            &CollidingEntities,
            &mut Health,
            &Transform,
            Has<Boss>,
        ),
        Or<(With<Enemy>, With<Boss>)>,
    >,
    player_bullets: Query<(), With<PlayerBullet>>,
    res: Res<DamageRes>,
    mut boss_died_event: EventWriter<BossDiedEvent>,
) {
    let mut hit_any = false;

    for (enemy_id, colliding_entities, mut health, &transform, is_boss) in &mut enemies {
        if is_boss {
            let angle = transform.rotation.to_axis_angle().1 / std::f32::consts::PI;
            if angle < 0.5 || angle > 1.5{
                // screen_print!("boss is not accepting damage: {}", angle);
                continue;
            }
            // screen_print!("boss is accepting damage: {}", angle);
        }
        let mut hit = false;
        for &entity in colliding_entities.iter() {
            if player_bullets.contains(entity) && health.health > 0. {
                hit = true;
                commands.entity(entity).despawn();
                health.health -= 2.;
                if health.health <= 0. {
                    health.health = 0.;
                    commands.entity(enemy_id).despawn_recursive();
                    // screen_print!(push, "Enemy died!");

                    if is_boss {
                        boss_died_event.send(BossDiedEvent);
                    }

                    let mut rng = rand::thread_rng();
                    for _ in 0..32 {
                        let dev = Vec2::new(rng.gen::<f32>() - 0.5, rng.gen::<f32>() - 0.5) * 100.0;

                        commands.spawn((
                            Item,
                            SpatialBundle::from_transform(Transform::from_translation(
                                transform.translation + dev.extend(0.0),
                            )),
                        ));
                    }
                }
            }
        }

        if hit && !hit_any {
            commands.spawn(AudioBundle {
                source: res.hit_sound.clone(),
                settings: PlaybackSettings::DESPAWN
                    .with_volume(Volume::Relative(VolumeLevel::new(0.5))),
            });
        }
        hit_any |= hit;
    }
}

fn player_damage(
    mut commands: Commands,
    mut player: Query<(&CollidingEntities, &mut Player), With<Player>>,
    enemy_bullets: Query<(), With<EnemyBullet>>,
    res: Res<DamageRes>,
) {
    let Ok((colliding_entities, mut player)) = player.get_single_mut() else {
        return;
    };

    let mut colliding_bullets = 0;
    for &entity in colliding_entities.iter() {
        if enemy_bullets.contains(entity) {
            colliding_bullets += 1;
            commands.entity(entity).despawn_recursive();
        }
    }
    if colliding_bullets > 0 {
        player.increase(-colliding_bullets as f32 * 80.);

        commands.spawn(AudioBundle {
            source: res.hit_sound.clone(),
            settings: PlaybackSettings::DESPAWN.with_volume(Volume::Relative(VolumeLevel::new(0.5))),
        });
    }
}

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BossDiedEvent>();
        app.add_systems(Startup, startup);
        app.add_systems(Update, (enemy_damage, player_damage));
    }
}
