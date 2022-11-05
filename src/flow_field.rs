use bevy::prelude::*;

use crate::grid::{self, Coords, Grid};
// use rand::prelude::*;

pub struct FlowFieldPlugin;

impl Plugin for FlowFieldPlugin {
    fn build(&self, _app: &mut App) {
        // app.add_startup_system(test_flow);
    }
}

pub fn generate_flow_field_grid(
    destination: Coords,
    grid: Res<Grid>,
    mut q_tiles: Query<&mut grid::Tile>,
) {
    // reset
    // let mut rng = rand::thread_rng();
    for mut t in q_tiles.iter_mut() {
        // t.cost = if rng.gen_bool(0.5) { 1 } else { 10 };
        t.weight = u32::MAX;
        t.next_pos = None;
    }

    let width = grid::GRID_WIDTH;
    let height = grid::GRID_HEIGHT;

    let destination_index = calculate_index(destination.x, destination.y, height);
    if let Some(&dest_ent) = grid.tiles.get(destination_index) {
        if let Ok(mut destination_node) = q_tiles.get_mut(dest_ent) {
            destination_node.weight = 0;

            let mut open_set = Vec::new();
            let mut closed_set: Vec<usize> = Vec::new();

            open_set.push(destination_index);

            while !open_set.is_empty() {
                // get the lowest weight in the open set
                let mut current_node_index = None;
                let mut current_lowest_weight = u32::MAX;
                for &i in open_set.iter() {
                    if let Some(&entity) = grid.tiles.get(i) {
                        if let Ok(tile) = q_tiles.get(entity) {
                            if tile.weight < current_lowest_weight {
                                current_lowest_weight = tile.weight;
                                current_node_index = Some(i);
                            }
                        }
                    }
                }
                if let Some(current_node_index) = current_node_index {
                    // remove from the open list
                    for (i, val) in open_set.iter().enumerate() {
                        if *val == current_node_index {
                            open_set.remove(i);
                            break;
                        }
                    }

                    closed_set.push(current_node_index);

                    let neighbours =
                        get_neighbour_indicies(current_node_index, width, height, false);
                    let current_node;
                    if let Some(&entity) = grid.tiles.get(current_node_index) {
                        if let Ok(tile) = q_tiles.get(entity).cloned() {
                            current_node = tile;

                            for n_index in neighbours {
                                if closed_set.contains(&n_index) {
                                    continue;
                                }

                                if let Some(&entity) = grid.tiles.get(n_index) {
                                    if let Ok(mut neighbour_node) = q_tiles.get_mut(entity) {
                                        let tentative_weight =
                                            current_node.weight + neighbour_node.cost as u32;

                                        if tentative_weight < neighbour_node.weight {
                                            neighbour_node.weight = tentative_weight;

                                            if !open_set.contains(&n_index) {
                                                open_set.push(n_index);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // calculate direction
            for i in 0..grid.tiles.len() {
                let neighbours = get_neighbour_indicies(i, width, height, true);

                // to fix the diagonals
                // could make it so you can only take a diagonal
                // if both of the adjacent tiles are open.
                // stops you cutting a corner, but can still go diagonal out in the open
                let smallest_index = neighbours
                    .iter()
                    .filter(|x| grid.tiles.get(**x).is_some())
                    .min_by_key(|x| q_tiles.get(*grid.tiles.get(**x).unwrap()).unwrap().weight)
                    .copied()
                    .unwrap();

                if let Some(&entity) = grid.tiles.get(i) {
                    if let Ok(mut tile) = q_tiles.get_mut(entity) {
                        if tile.weight == 0 {
                            // destination
                            tile.next_pos = None;
                        } else {
                            let small_coords = Coords {
                                x: smallest_index / height,
                                y: smallest_index % height,
                            };
                            tile.next_pos = Some(small_coords.get_vec2());
                        }
                    }
                }
            }
        }
    }
    println!("Updated flow field");
}

fn get_neighbour_indicies(
    index: usize,
    width: usize,
    height: usize,
    eight_connected: bool,
) -> Vec<usize> {
    let mut v = Vec::new();

    let x = index / height;
    let y = index % height;

    if y < height - 1 {
        // up
        v.push(calculate_index(x, y + 1, height));
    }
    if eight_connected && y < height - 1 && x < width - 1 {
        // up right
        v.push(calculate_index(x + 1, y + 1, height));
    }
    if x < width - 1 {
        // right
        v.push(calculate_index(x + 1, y, height));
    }
    if eight_connected && y > 0 && x < width - 1 {
        // down right
        v.push(calculate_index(x + 1, y - 1, height));
    }
    if y > 0 {
        // down
        v.push(calculate_index(x, y - 1, height));
    }
    if eight_connected && y > 0 && x > 0 {
        // down left
        v.push(calculate_index(x - 1, y - 1, height));
    }
    if x > 0 {
        // left
        v.push(calculate_index(x - 1, y, height));
    }
    if eight_connected && x > 0 && y < height - 1 {
        // up left
        v.push(calculate_index(x - 1, y + 1, height));
    }

    v
}

fn calculate_index(x: usize, y: usize, height: usize) -> usize {
    y + x * height
}
