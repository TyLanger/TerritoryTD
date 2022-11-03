use bevy::prelude::*;

use crate::{flow_field::generate_flow_field_grid, MouseWorldPos};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Grid {
            tiles: Vec::with_capacity(GRID_WIDTH * GRID_HEIGHT),
        })
        .insert_resource(TileColours::new())
        .add_event::<ClearSelectionsEvent>()
        .add_startup_system(setup_grid)
        .add_system(clear_interaction.before(check_interaction))
        .add_system(check_interaction.before(tile_interaction))
        .add_system(tile_interaction.before(clear_selection))
        .add_system(clear_selection)
        .add_system(gen_flow_field);
    }
}

pub const GRID_WIDTH: usize = 20;
pub const GRID_HEIGHT: usize = 20;
pub const TILE_SIZE: f32 = 32.0;

// events
struct ClearSelectionsEvent;

#[derive(Copy, Clone)]
pub struct Coords {
    pub x: usize,
    pub y: usize,
}

impl Coords {
    pub fn from_vec2(pos: Vec2) -> Self {
        let x = (((GRID_WIDTH - 1) as f32 * 0.5 * TILE_SIZE) + TILE_SIZE * 0.5 + pos.x) / TILE_SIZE;
        let y =
            (((GRID_HEIGHT - 1) as f32 * 0.5 * TILE_SIZE) + TILE_SIZE * 0.5 + pos.y) / TILE_SIZE;

        Coords {
            x: x as usize,
            y: y as usize,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct Tile {
    pub coords: Coords,
    pub cost: u8,
    pub weight: u32,
    pub dir: Option<Vec2>,
}

impl Tile {
    fn new(x: usize, y: usize) -> Self {
        Tile {
            coords: Coords { x, y },
            cost: 1,
            weight: u32::MAX,
            dir: None,
        }
    }

    fn is_even(&self) -> bool {
        (self.coords.x + self.coords.y) % 2 == 0
    }
}

pub struct Grid {
    pub tiles: Vec<Entity>,
}

impl Grid {
    pub fn get_vec2(&self, pos: Vec2) -> Option<Entity> {
        let x = (((GRID_WIDTH - 1) as f32 * 0.5 * TILE_SIZE) + TILE_SIZE * 0.5 + pos.x) / TILE_SIZE;
        let y =
            (((GRID_HEIGHT - 1) as f32 * 0.5 * TILE_SIZE) + TILE_SIZE * 0.5 + pos.y) / TILE_SIZE;

        if x < 0.0 || y < 0.0 {
            return None;
        }

        self.get_xy(x as usize, y as usize)
    }

    pub fn get_xy(&self, x: usize, y: usize) -> Option<Entity> {
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            return None;
        }

        let index = x * GRID_HEIGHT + y;
        let ent = self.tiles.get(index);
        ent.copied()
    }

    fn get_coords(&self, coords: Coords) -> Option<Entity> {
        self.get_xy(coords.x, coords.y)
    }
}

struct TileColours {
    even_grass: Color,
    odd_grass: Color,
    hover_color: Color,
    select_color: Color,
}

impl TileColours {
    fn new() -> Self {
        TileColours {
            // #3e8948
            even_grass: Color::rgb_u8(0x3e, 0x89, 0x48),
            // #265c42
            odd_grass: Color::rgb_u8(0x26, 0x5c, 0x42),
            hover_color: Color::ALICE_BLUE,
            select_color: Color::MIDNIGHT_BLUE,
        }
    }
}

#[derive(Component)]
struct Selection;

fn setup_grid(mut commands: Commands, mut grid: ResMut<Grid>, tile_colours: Res<TileColours>) {
    let offset = Vec3::new(
        -0.5 * ((GRID_WIDTH - 1) as f32) * TILE_SIZE,
        -0.5 * ((GRID_HEIGHT - 1) as f32) * TILE_SIZE,
        0.0,
    );

    for i in 0..GRID_WIDTH {
        for j in 0..GRID_HEIGHT {
            let pos = offset + Vec3::new(i as f32 * TILE_SIZE, j as f32 * TILE_SIZE, 0.0);

            let tile = Tile::new(i, j);

            let color = if tile.is_even() {
                tile_colours.even_grass
            } else {
                tile_colours.odd_grass
            };

            let tile_ent = commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::splat(TILE_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_translation(pos),
                    ..default()
                })
                .insert(tile)
                .insert(Interaction::None)
                .id();

            grid.tiles.push(tile_ent);
        }
    }
}

fn clear_interaction(mut q_tiles: Query<(&mut Interaction, Option<&Selection>), With<Tile>>) {
    for (mut interaction, o_selection) in q_tiles.iter_mut() {
        // clear all interactions
        *interaction = Interaction::None;
    }
}

fn check_interaction(
    mut q_tiles: Query<&mut Interaction, With<Tile>>,
    grid: Res<Grid>,
    mouse: Res<MouseWorldPos>,
    mouse_click: Res<Input<MouseButton>>,
    mut ev_clear: EventWriter<ClearSelectionsEvent>,
) {
    let left_click = mouse_click.just_pressed(MouseButton::Left);
    let right_click = mouse_click.just_pressed(MouseButton::Right);
    let hovered = grid.get_vec2(mouse.0);
    if let Some(ent) = hovered {
        if let Ok(mut interaction) = q_tiles.get_mut(ent) {
            if *interaction == Interaction::None {
                if left_click {
                    *interaction = Interaction::Clicked;
                } else {
                    *interaction = Interaction::Hovered;
                }
            }
        }
    } else if left_click {
        ev_clear.send(ClearSelectionsEvent);
    }

    if right_click {
        ev_clear.send(ClearSelectionsEvent);
    }
}

fn tile_interaction(
    mut commands: Commands,
    mut q_interaction: Query<(Entity, &Interaction, &mut Sprite, &Tile), Without<Selection>>,
    tile_colours: Res<TileColours>,
) {
    for (entity, interaction, mut sprite, tile) in q_interaction.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                sprite.color = tile_colours.select_color;
                commands.entity(entity).insert(Selection);
            }
            Interaction::Hovered => {
                sprite.color = tile_colours.hover_color;
            }
            Interaction::None => {
                if tile.cost == 10 {
                    sprite.color = Color::DARK_GRAY;
                } else {
                    sprite.color = if tile.is_even() {
                        tile_colours.even_grass
                    } else {
                        tile_colours.odd_grass
                    };
                }
            }
        }
    }
}

fn clear_selection(
    mut commands: Commands,
    q_selection: Query<Entity, With<Selection>>,
    ev_clear: EventReader<ClearSelectionsEvent>,
) {
    if !ev_clear.is_empty() {
        ev_clear.clear();
        for entity in q_selection.iter() {
            commands.entity(entity).remove::<Selection>();
        }
    }
}

fn gen_flow_field(
    keyboard: Res<Input<KeyCode>>,
    mouse: Res<MouseWorldPos>,
    grid: Res<Grid>,
    mut q_tiles: Query<&mut Tile>,
) {
    if keyboard.just_pressed(KeyCode::F) {
        let dest = Coords::from_vec2(mouse.0);
        generate_flow_field_grid(dest, grid, q_tiles);
    }
}

// fn check_mouse(
//     mut q_tiles: Query<&mut Sprite, With<Tile>>,
//     grid: Res<Grid>,
//     tile_colours: Res<TileColours>,
//     mouse: Res<MouseWorldPos>,
// ) {
//     let get_vec2 = grid.get_vec2(mouse.0);
//     let hovered = get_vec2;
//     if let Some(ent) = hovered {
//         if let Ok(mut sprite) = q_tiles.get_mut(ent) {
//             sprite.color = tile_colours.hover_color;
//         }
//     }
// }
