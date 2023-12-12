use bevy::prelude::*;
use bevy_debug_text_overlay::screen_print;

const HEALTH_BAR_Z: f32 = 100.0;
const HEALTH_BAR_HEIGHT: f32 = 5.0;
const HEALTH_BAR_OFFSET: f32 = 15.0;

#[derive(Component, Debug)]
pub struct Health {
    pub health: f32,
    pub max_health: f32,
}

impl Health {
    pub fn percent(&self) -> f32 {
        self.health / self.max_health * 100.0
    }
}

#[derive(Component)]
pub struct HealthBar;

fn add_healthbar(
    mut commands: Commands,
    q: Query<(Entity, Option<(&Sprite, &Transform)>), Added<Health>>,
) {
    for (entity, sprite_transform) in &q {
        let y = sprite_transform
            .and_then(|(sprite, transform)| {
                sprite.custom_size.map(|size| {
                    size.y * transform.scale.y - HEALTH_BAR_HEIGHT / 2. + HEALTH_BAR_OFFSET
                })
            })
            .unwrap_or(0.0)
            / 2.0;

        commands.entity(entity).with_children(|c| {
            c.spawn((HealthBar, SpatialBundle::HIDDEN_IDENTITY))
                .with_children(|c| {
                    // Foreground
                    c.spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::BLUE.with_a(0.5),
                            custom_size: Some(Vec2::new(100.0, HEALTH_BAR_HEIGHT)),
                            ..default()
                        },
                        transform: Transform::from_xyz(0., y, HEALTH_BAR_Z),
                        ..default()
                    });
                    // Background
                    c.spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::BLACK.with_a(0.5),
                            custom_size: Some(Vec2::new(100.0, HEALTH_BAR_HEIGHT)),
                            ..default()
                        },
                        transform: Transform::from_xyz(0., y, HEALTH_BAR_Z - 1.0),
                        ..default()
                    });
                });
        });
    }
}

fn update_healthbar(
    healths: Query<(&Health, &Children), (Changed<Health>, Without<HealthBar>)>,
    mut health_bars: Query<(&Children, &mut Visibility), (With<HealthBar>, Without<Health>)>,
    mut transforms: Query<&mut Transform>,
) {
    for (health, children) in &healths {
        // screen_print!("Health: {:?} (children = {})", health, children.len());

        for child in children {
            if let Ok((bar_children, mut vis)) = health_bars.get_mut(*child) {
                if health.percent() >= 100.0 {
                    vis.set_if_neq(Visibility::Hidden);
                } else {
                    vis.set_if_neq(Visibility::Inherited);
                }

                // screen_print!("bar_children = {:?}", bar_children);
                let foreground = bar_children[0];
                let mut tf = transforms.get_mut(foreground).unwrap();
                tf.scale.x = health.percent() / 100.0;
                tf.translation.x = -50.0 + health.percent() / 2.0;
            }
        }
    }
}

pub struct HealthBarPlugin;

impl Plugin for HealthBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (add_healthbar, apply_deferred, update_healthbar).chain(),
        );
    }
}
