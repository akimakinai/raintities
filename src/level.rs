use bevy::prelude::*;

use crate::{
    enemy::{spawn_enemy, EnemyController, ENEMY_SIZE},
    MainCamera, SCREEN_WIDTH,
};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (scroll_system, spawn_enemies).chain());
    }
}

#[derive(Resource)]
struct Level {
    enemies: Vec<(Vec2, EnemyController)>,
}

fn setup(mut commands: Commands) {
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
            Vec2::new(SCREEN_WIDTH / 2. + ENEMY_SIZE / 2., -i as f32 * 100. - 400.),
            Vec2::new(2. * SCREEN_WIDTH / 3. - SCREEN_WIDTH / 2., -i as f32 * 100. - 400.),
            Vec2::new(SCREEN_WIDTH / 3. - SCREEN_WIDTH / 2., -i as f32 * 100. - 400.),
        ];
        enemies.push((
            Vec2::new(SCREEN_WIDTH / 2. + ENEMY_SIZE / 2., -i as f32 * 100. - 400.),
            enemy_positions.into_iter().rev().collect::<Vec<_>>().into(),
        ));
    }

    let level = Level { enemies };
    commands.insert_resource(level);
}

fn scroll_system(mut camera: Query<&mut Transform, With<MainCamera>>, time: Res<Time>) {
    let mut camera = camera.single_mut();
    camera.translation.y -= 15. * time.delta_seconds();
}

fn spawn_enemies(
    mut commands: Commands,
    camera: Query<&Transform, With<MainCamera>>,
    mut level: ResMut<Level>,
) {
    let camera_y = camera.single().translation.y;

    let mut rem_enemies = Vec::new();
    for (pos, enemy) in level.enemies.drain(..) {
        if enemy.attack_pos.last().unwrap().y >= camera_y - SCREEN_WIDTH / 2. - ENEMY_SIZE / 2. {
            debug!("spawning {:?}", enemy);
            spawn_enemy(&mut commands, pos, enemy);
        } else {
            rem_enemies.push((pos, enemy));
        }
    }
    level.enemies = rem_enemies;
}
