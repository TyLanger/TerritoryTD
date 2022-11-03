use bevy::{math::vec2, prelude::*};
// use rand::prelude::*;

pub struct FlowFieldPlugin;

impl Plugin for FlowFieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(test_flow);
    }
}

#[derive(Copy, Clone, Debug)]
struct Tile {
    cost: u8,
    weight: u32,
    dir: Option<Vec2>,
}

fn test_flow() {
    generate_flow_field(2, 2);
}

fn generate_flow_field(destination_x: usize, destination_y: usize) {
    let width = 10;
    let height = 8;

    let mut nodes: Vec<Tile> = Vec::with_capacity(width * height);
    // create the tiles
    for _i in 0..width {
        for _j in 0..height {
            // let mut rng = rand::thread_rng();

            nodes.push(Tile {
                cost: 1, //if rng.gen_bool(0.5) { 1 } else { 10 },
                weight: u32::MAX,
                dir: None,
            });

            // let index = calculate_index(i, j, height);
            // let x = index / height;
            // let y = index % height;
            // println!("i,j: ({}, {}) index: {} back: ({}, {})", i, j, index, x, y);
        }
    }

    let destination_index = calculate_index(destination_x, destination_y, height);
    let mut dest_node = nodes.get_mut(destination_index).unwrap();
    dest_node.weight = 0;

    let mut open_set = Vec::new();
    let mut closed_set = Vec::new();

    open_set.push(destination_index);

    while open_set.len() > 0 {
        // get the lowest weight in the open_set
        if let Some(current_node_index) = get_lowest_weight_index(&open_set, &nodes) {
            // remove from the open list
            for (i, val) in open_set.iter().enumerate() {
                if *val == current_node_index {
                    open_set.remove(i);
                    break;
                }
            }

            // add self to closed set so it isn't checked again
            closed_set.push(current_node_index);

            // get neighbours (4-connected)
            let neighbours = get_neighbour_indicies(current_node_index, width, height, false);
            let current_node = nodes.get(current_node_index).unwrap().clone();

            for n_index in neighbours {
                // skip neighbours that have already been checked
                if closed_set.contains(&n_index) {
                    continue;
                }

                if let Some(neighbour_node) = nodes.get_mut(n_index) {
                    // update the new weight
                    // unless it's already lower bc it was evaluated by another neighbour
                    let tentative_weight = current_node.weight + neighbour_node.cost as u32;

                    if tentative_weight < neighbour_node.weight {
                        neighbour_node.weight = tentative_weight;

                        // add the neighbour to the open set to be evaluated later
                        if !open_set.contains(&n_index) {
                            open_set.push(n_index);
                        }
                    }
                }
            }
        }
    }

    // calculate dir
    // get 8 neighbours
    // get the lowest weight among them
    // that is the neighbour to move towards
    // your dir is them - you
    for i in 0..nodes.len() {
        let neighbours = get_neighbour_indicies(i, width, height, true);

        let smallest_index = neighbours
            .iter()
            .min_by_key(|x| nodes.get(**x).unwrap().weight)
            .copied()
            .unwrap();

        let mut val = nodes.get_mut(i).unwrap();
        if val.weight == 0 {
            val.dir = Some(Vec2::ZERO);
        } else {
            let small_x = smallest_index / height;
            let small_y = smallest_index % height;

            let my_x = i / height;
            let my_y = i % height;

            let dir = vec2(small_x as f32 - my_x as f32, small_y as f32 - my_y as f32);
            val.dir = Some(dir);
        }
    }

    // for (i, val) in nodes.iter().enumerate() {
    //     // ><^v /\
    //     println!("{}: {:?}", i, val);
    // }
    for j in 0..height {
        for i in 0..width {
            let mut c = "   ";
            match nodes
                .get(calculate_index(i, height - (j + 1), height))
                .unwrap()
                .dir
            {
                Some(v) => {
                    let x = v.x.floor();
                    let y = v.y.floor();
                    if x == 0.0 && y == 1.0 {
                        c = " ^ ";
                    } else if x == 1.0 && y == 1.0 {
                        c = " / ";
                    } else if x == 1.0 && y == 0.0 {
                        c = " > ";
                    } else if x == 1.0 && y == -1.0 {
                        c = " \\ ";
                    } else if x == 0.0 && y == -1.0 {
                        c = " v ";
                    } else if x == -1.0 && y == -1.0 {
                        c = " / ";
                    } else if x == -1.0 && y == 0.0 {
                        c = " < ";
                    } else if x == -1.0 && y == 1.0 {
                        c = " \\ ";
                    }
                }

                None => {
                    c = " ";
                }
            };
            print!("{}", c);
        }
        println!("");
    }
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

fn get_lowest_weight_index(open_set: &[usize], nodes: &[Tile]) -> Option<usize> {
    open_set
        .iter()
        .min_by_key(|x| nodes.get(**x).unwrap().weight)
        .copied()
}

fn calculate_index(x: usize, y: usize, height: usize) -> usize {
    y + x * height
}
