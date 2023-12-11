use bevy::prelude::*;
use bevy_debug_text_overlay::screen_print;
use bevy_xpbd_2d::prelude::*;
use rand::Rng;

use crate::{
    enemy::{Enemy, EnemyBullet},
    health::Health,
    item::Item,
    player::{Player, PlayerBullet},
};

fn enemy_damage(
    mut commands: Commands,
    mut enemies: Query<(Entity, &CollidingEntities, &mut Health, &Transform), With<Enemy>>,
    player_bullets: Query<(), With<PlayerBullet>>,
) {
    for (enemy_id, colliding_entities, mut health, &transform) in &mut enemies {
        for &entity in colliding_entities.iter() {
            if player_bullets.contains(entity) && health.health > 0. {
                commands.entity(entity).despawn();
                health.health -= 2.;
                if health.health <= 0. {
                    health.health = 0.;
                    commands.entity(enemy_id).despawn_recursive();
                    screen_print!(push, "Enemy is dead!");

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
    }
}

fn player_damage(
    mut commands: Commands,
    mut player: Query<(&CollidingEntities, &mut Player), With<Player>>,
    enemy_bullets: Query<(), With<EnemyBullet>>,
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
        player.increase(-colliding_bullets as f32 * 20.);
    }
}

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (enemy_damage, player_damage));
    }
}
