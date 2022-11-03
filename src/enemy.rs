use bevy::prelude::*;

use crate::grid::{self, Grid, Tile, GRID_HEIGHT, GRID_WIDTH, TILE_SIZE};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_enemy).add_system(move_enemy);
    }
}

#[derive(Component)]
struct Enemy;

fn spawn_enemy(mut commands: Commands, keyboard: Res<Input<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::E) {
        let offset = Vec3::new(
            -0.5 * ((GRID_WIDTH - 1) as f32) * TILE_SIZE,
            -0.5 * ((GRID_HEIGHT - 1) as f32) * TILE_SIZE,
            0.0,
        );
        for i in 0..grid::GRID_WIDTH {
            for j in 0..grid::GRID_HEIGHT {
                let pos = Vec3::new(i as f32 * 32.0, j as f32 * 32.0, 0.1);
                commands
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: Color::CRIMSON,
                            custom_size: Some(Vec2::splat(25.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(offset + pos),
                        ..default()
                    })
                    .insert(Enemy);
            }
        }
    }
}

fn move_enemy(
    grid: Res<Grid>,
    q_tiles: Query<(&Transform, &Tile), Without<Enemy>>,
    mut q_enemies: Query<&mut Transform, With<Enemy>>,
    time: Res<Time>,
) {
    for mut trans in q_enemies.iter_mut() {
        let mut dir = Vec3::new(-25.0, 0.0, 0.0);

        if let Some(entity) = grid.get_vec2(trans.translation.truncate()) {
            if let Ok((_transform, tile)) = q_tiles.get(entity) {
                if let Some(dir_tile) = tile.dir {
                    // trans.translation += dir.extend(0.0) * time.delta_seconds();
                    dir = dir_tile.extend(0.0) * 25.0;
                }
            }
        }

        trans.translation += dir * time.delta_seconds();
    }
}
