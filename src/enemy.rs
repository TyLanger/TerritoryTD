use bevy::prelude::*;

use crate::grid::{self, Grid, Tile, GRID_HEIGHT, GRID_WIDTH, TILE_SIZE};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_enemy).add_system(move_enemy);
    }
}

#[derive(Component)]
struct Enemy {
    dir: Vec2,
    pos: Option<Vec2>,
    speed: f32,
}

impl Enemy {
    fn new() -> Self {
        Enemy {
            dir: Vec2::new(1.0, 0.0),
            pos: None,
            speed: 25.0,
        }
    }
}

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
                    .insert(Enemy::new());
            }
        }
    }
}

fn move_enemy(
    grid: Res<Grid>,
    q_tiles: Query<(&Transform, &Tile), Without<Enemy>>,
    mut q_enemies: Query<(&mut Transform, &mut Enemy)>,
    time: Res<Time>,
) {
    for (mut trans, mut enemy) in q_enemies.iter_mut() {
        let mut want_pos = false;
        if let Some(pos) = enemy.pos {
            // you have a pos
            if trans.translation.truncate().distance_squared(pos) < 1.0 {
                // close to pos
                // find a new one
                want_pos = true;
            } else {
                // keep going where you're going
            }
        } else {
            // find one if possible
            want_pos = true;
        }

        // let mut dir = Vec3::new(-25.0, 0.0, 0.0);
        if want_pos {
            if let Some(entity) = grid.get_vec2(trans.translation.truncate()) {
                if let Ok((_transform, tile)) = q_tiles.get(entity) {
                    if let Some(dir_tile) = tile.next_pos {
                        // trans.translation += dir.extend(0.0) * time.delta_seconds();
                        // dir = dir_tile.extend(0.0) * 25.0;
                        enemy.pos = Some(dir_tile);
                        enemy.dir = (dir_tile - trans.translation.truncate()).normalize_or_zero();
                    } else {
                        // at destination
                        // stop moving
                        if enemy.dir != Vec2::ZERO {
                            enemy.dir = Vec2::ZERO;
                        }
                    }
                }
            }
        }

        trans.translation += enemy.dir.extend(0.0) * enemy.speed * time.delta_seconds();
    }
}
