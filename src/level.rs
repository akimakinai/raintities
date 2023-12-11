use bevy::prelude::*;

use crate::{
    boss::{Boss, BOSS_PADDING, BOSS_SIZE},
    enemy::{spawn_enemy, EnemyController, ENEMY_SIZE},
    MainCamera, SCREEN_HEIGHT,
};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ScrollDoneEvent>()
            // .add_systems(Startup, setup)
            .add_systems(Update, (scroll_system, spawn_enemies).chain().run_if(resource_exists::<Level>()));
    }
}

#[derive(Resource)]
pub struct Level {
    pub enemies: Vec<(Vec2, EnemyController)>,
    pub boss_pos: Option<Vec2>,
}

#[derive(Event)]
pub struct ScrollDoneEvent;

fn scroll_system(
    mut camera: Query<&mut Transform, With<MainCamera>>,
    boss: Query<&Transform, (With<Boss>, Without<MainCamera>)>,
    time: Res<Time<Virtual>>,
    mut scroll_done_event: EventWriter<ScrollDoneEvent>,
    mut event_sent: Local<bool>,
) {
    if *event_sent { return; }

    let mut camera = camera.single_mut();

    if let Ok(boss_transform) = boss.get_single() {
        if boss_transform.translation.y
            >= camera.translation.y - SCREEN_HEIGHT / 2. + BOSS_SIZE / 2. + BOSS_PADDING
        {
            if !*event_sent {
                scroll_done_event.send(ScrollDoneEvent);
            }
            *event_sent = true;
            return;
        }
    }

    camera.translation.y -= 60. * time.delta_seconds();
}

fn spawn_enemies(
    mut commands: Commands,
    camera: Query<&Transform, With<MainCamera>>,
    mut level: ResMut<Level>,
) {
    let camera_y = camera.single().translation.y;

    let mut rem_enemies = Vec::new();
    for (pos, enemy) in level.enemies.drain(..) {
        if enemy.attack_pos.last().unwrap().y >= camera_y - SCREEN_HEIGHT / 2. - ENEMY_SIZE / 2. {
            debug!("spawning {:?}", enemy);
            spawn_enemy(&mut commands, pos, enemy);
        } else {
            rem_enemies.push((pos, enemy));
        }
    }
    level.enemies = rem_enemies;

    if let Some(boss_pos) = level.boss_pos {
        if camera_y - SCREEN_HEIGHT / 2. - BOSS_SIZE / 2. <= boss_pos.y {
            level.boss_pos = None;
            commands.spawn((
                Name::new("Boss"),
                Boss,
                SpatialBundle::from_transform(Transform::from_translation(boss_pos.extend(0.))),
            ));
        }
    }
}
