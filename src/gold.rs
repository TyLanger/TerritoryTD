use std::f32::consts;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::{
    grid::{Coords, Grid, Tile, TileType},
    resource_container::Resource,
};

pub struct GoldPlugin;

impl Plugin for GoldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnGoldEvent>()
            .add_system(tick_spawner)
            .add_system(tick_gold)
            .add_system(spawn_gold);
    }
}

#[derive(Component)]
pub struct GoldSpawner {
    timer: Timer,
    range: u32,
}

impl GoldSpawner {
    pub fn new() -> Self {
        GoldSpawner {
            timer: Timer::from_seconds(3.0, true),
            range: 2,
        }
    }
}

#[derive(Component)]
struct Gold {
    lifetime: Timer,
}

impl Gold {
    fn new() -> Self {
        Gold {
            lifetime: Timer::from_seconds(1.1, false),
        }
    }
}

struct SpawnGoldEvent {
    pos: Vec3,
}

fn tick_spawner(
    mut q_tiles: Query<(&Transform, &Tile, &mut Resource)>,
    mut q_gold_spawners: Query<(&Transform, &mut GoldSpawner)>,
    mut ev_spawn: EventWriter<SpawnGoldEvent>,
    time: Res<Time>,
    grid: Res<Grid>,
) {
    // is &mut query a better pattern than
    // query.iter_mut() ?

    // for each spawner, spawn gold in the range around it
    // only spawn gold on friendly territory
    for (trans, mut spawner) in &mut q_gold_spawners {
        if spawner.timer.tick(time.delta()).just_finished() {
            // println!("Spawn a gold");
            let center_coords = Coords::from_vec2(trans.translation.truncate());
            for i in 0..=spawner.range {
                // get coords in range
                let neighbours = grid.get_diamond_ring(center_coords, i as usize);
                // iterate over each coord that exists
                for n in neighbours.iter().flatten() {
                    // does it have a tile?
                    if let Ok((tile_trans, tile, mut res)) = q_tiles.get_mut(*n) {
                        // does tile.tile_type match my allegience?
                        if matches!(tile.tile_type, TileType::Friendly) {
                            // spawn a gold there
                            ev_spawn.send(SpawnGoldEvent {
                                pos: tile_trans.translation,
                            });

                            match *res {
                                Resource::None => {
                                    *res = Resource::Gold(1);
                                }
                                Resource::Gold(mut count) => {
                                    count += 1;
                                    *res = Resource::Gold(count);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn spawn_gold(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut ev_gold: EventReader<SpawnGoldEvent>,
) {
    for ev in ev_gold.iter() {
        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(12.0).into()).into(),
                material: materials.add(ColorMaterial::from(Color::GOLD)),
                transform: Transform::from_translation(ev.pos + Vec3::new(0.0, 0.0, 0.1)),
                ..default()
            })
            .insert(Gold::new());
    }
}

fn tick_gold(
    mut commands: Commands,
    mut q_gold: Query<(Entity, &mut Transform, &mut Gold)>,
    time: Res<Time>,
) {
    for (entity, mut trans, mut gold) in &mut q_gold {
        if gold.lifetime.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn_recursive();
        }
        // move upwards over time.
        trans.translation += Vec3::new(0.0, 70.0 * time.delta_seconds(), 0.0);
        // spin
        let t = gold.lifetime.percent();
        // 1.0 - 0.0 - 1.0
        // peak of sin to trough
        // gives a better curve. Stays at 0 for less time and lingers at 1.0 for longer
        // abs. don't want negative
        // * 2.0 to get 2 spins
        let x = f32::sin(consts::FRAC_PI_2 + t * consts::PI * 2.0).abs();
        trans.scale = Vec3::new(x, 1.0, 1.0);
    }
}
