use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::Mesh2dHandle,
};

use crate::{MainCamera, SCREEN_HEIGHT, SCREEN_WIDTH};

use super::Player;

const INNER_SIZE: f32 = 100.0;

#[derive(Component)]
struct DamageEffect;

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    let frame_mesh = meshes.add(
        Mesh::new(PrimitiveTopology::TriangleList)
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_POSITION,
                vec![
                    [-SCREEN_WIDTH / 2., SCREEN_HEIGHT / 2., 0.],
                    [SCREEN_WIDTH / 2., SCREEN_HEIGHT / 2., 0.],
                    [SCREEN_WIDTH / 2., -SCREEN_HEIGHT / 2., 0.],
                    [-SCREEN_WIDTH / 2., -SCREEN_HEIGHT / 2., 0.],
                    [
                        -SCREEN_WIDTH / 2. + INNER_SIZE,
                        SCREEN_HEIGHT / 2. - INNER_SIZE,
                        0.,
                    ],
                    [
                        SCREEN_WIDTH / 2. - INNER_SIZE,
                        SCREEN_HEIGHT / 2. - INNER_SIZE,
                        0.,
                    ],
                    [
                        SCREEN_WIDTH / 2. - INNER_SIZE,
                        -SCREEN_HEIGHT / 2. + INNER_SIZE,
                        0.,
                    ],
                    [
                        -SCREEN_WIDTH / 2. + INNER_SIZE,
                        -SCREEN_HEIGHT / 2. + INNER_SIZE,
                        0.,
                    ],
                ],
            )
            .with_indices(Some(Indices::U32(vec![
                0, 1, 4, 1, 5, 4, 1, 2, 5, 2, 6, 5, 2, 3, 6, 3, 7, 6, 3, 0, 7, 0, 4, 7,
            ]))),
    );

    commands
        .spawn(ColorMesh2dBundle {
            mesh: Mesh2dHandle(frame_mesh),
            material: color_materials.add(Color::RED.with_a(0.2).into()),
            ..default()
        })
        .insert(DamageEffect);
}

fn update_effect(
    player: Query<&Player, Changed<Player>>,
    mut damage_effect: Query<&mut Handle<ColorMaterial>, With<DamageEffect>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    let Ok(player) = player.get_single() else {
        return;
    };

    let damage_effect = damage_effect.single_mut();

    color_materials.get_mut(damage_effect.id()).unwrap().color =
        Color::RED.with_a((0.8 - player.radius / 50.).clamp(0., 1.));
}

fn scroll_effect(
    mut transform: Query<&mut Transform, With<DamageEffect>>,
    camera: Query<&Transform, (With<MainCamera>, Without<DamageEffect>)>,
) {
    let mut transform = transform.single_mut();
    let camera_transform = camera.single();

    transform.translation = camera_transform.translation;
}

pub struct DamageEffectPlugin;

impl Plugin for DamageEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup)
            .add_systems(PostUpdate, (update_effect, scroll_effect));
    }
}
