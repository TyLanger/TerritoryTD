use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

pub struct ResourcePlugin;

impl Plugin for ResourcePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(check_max);
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub enum Resource {
    None,
    Gold(u32),
}

#[derive(Component)]
struct ResourceVisual;

fn check_max(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    q_resources: Query<(Entity, Option<&Children>, &Resource), Changed<Resource>>,
    q_visuals: Query<Entity, With<ResourceVisual>>,
) {
    for (parent, children, res) in &q_resources {
        // destroy all children with ResourceVisual component
        // I don't know how many resources got added
        // so just delete and rebuild every time it changes
        if let Some(children) = children {
            for &child in children {
                if q_visuals.contains(child) {
                    commands.entity(parent).remove_children(&[child]);
                    commands.entity(child).despawn();
                }
            }
        }

        match res {
            Resource::None => {}
            Resource::Gold(count) => {
                // build a tower of gold
                for i in 0..*count {
                    let child = commands
                        .spawn_bundle(MaterialMesh2dBundle {
                            mesh: meshes.add(shape::Circle::new(6.0).into()).into(),
                            material: materials.add(ColorMaterial::from(Color::GOLD)),
                            transform: Transform::from_translation(Vec3::new(
                                0.0,
                                1.0 * (1 + i) as f32,
                                0.1 * (1 + i) as f32,
                            )),
                            ..default()
                        })
                        .insert(ResourceVisual)
                        .id();

                    commands.entity(parent).add_child(child);
                }
            }
        }
    }
}
