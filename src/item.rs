use bevy::{prelude::*, sprite::Mesh2dHandle};
use bevy_xpbd_2d::prelude::*;

use crate::MyLayer;

#[derive(Resource)]
struct ItemResource {
    material: Handle<ColorMaterial>,
}

fn startup(mut commands: Commands, mut color_materials: ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(ItemResource {
        material: color_materials.add(Color::BLUE.into()),
    });
}

#[derive(Component)]
pub struct Item;

fn spawn_item(
    mut commands: Commands,
    q: Query<Entity, Added<Item>>,
    mut meshes: ResMut<Assets<Mesh>>,
    item_res: Res<ItemResource>,
) {
    for id in &q {
        commands
            .entity(id)
            .insert((
                Collider::ball(8.),
                RigidBody::Kinematic,
                CollisionLayers::new([MyLayer::Item], [MyLayer::Player]),
            ))
            .with_children(|c| {
                c.spawn(ColorMesh2dBundle {
                    mesh: Mesh2dHandle(
                        meshes.add(
                            shape::Circle {
                                radius: 8.,
                                vertices: 8,
                            }
                            .into(),
                        ),
                    ),
                    material: item_res.material.clone(),
                    ..default()
                });
            });
    }
}

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup)
            .add_systems(PostUpdate, spawn_item);
    }
}
