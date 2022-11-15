use std::{f32::consts::FRAC_PI_2, time::Duration};

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
        .add_event::<ChangeAllegianceEvent>()
        .add_startup_system(setup_grid)
        .add_system(clear_interaction.before(check_interaction))
        .add_system(check_interaction.before(tile_interaction))
        .add_system(tile_interaction.before(clear_selection))
        .add_system(clear_selection)
        .add_system(gen_flow_field)
        // .add_system(change_alegience.after(tile_interaction))
        .add_system(change_allegiance.after(tile_interaction))
        // .add_system(change_colour_animation)
        .add_system(territory_flip_animation)
        .add_system(grab_territory);
    }
}

pub const GRID_WIDTH: usize = 20;
pub const GRID_HEIGHT: usize = 20;
pub const TILE_SIZE: f32 = 32.0;

// events
pub struct ClearSelectionsEvent;

#[derive(Copy, Clone, Debug)]
pub struct Coords {
    pub x: usize,
    pub y: usize,
}

impl Coords {
    pub fn from_vec2(pos: Vec2) -> Self {
        let x = (((GRID_WIDTH - 1) as f32 * 0.5 * TILE_SIZE) + TILE_SIZE * 0.5 + pos.x) / TILE_SIZE;
        let y =
            (((GRID_HEIGHT - 1) as f32 * 0.5 * TILE_SIZE) + TILE_SIZE * 0.5 + pos.y) / TILE_SIZE;

        // if x < 0.0 || y < 0.0 {
        //     println!("x or y < 0 x,y:{:?}, {:?}", x, y);
        //     println!("converted to usize: {:?}, {:?}", x as usize, y as usize);
        // }
        // should I clamp?
        // lower bounds are auto clamped by converting to usize

        // negative values get clamped to 0
        Coords {
            x: x as usize,
            y: y as usize,
        }
    }

    pub fn get_vec2(&self) -> Vec2 {
        let offset = Vec2::new(
            -0.5 * ((GRID_WIDTH - 1) as f32) * TILE_SIZE,
            -0.5 * ((GRID_HEIGHT - 1) as f32) * TILE_SIZE,
        );
        offset + Vec2::new(self.x as f32 * TILE_SIZE, self.y as f32 * TILE_SIZE)
    }
}

#[derive(Component, Clone, Copy)]
pub struct Tile {
    pub coords: Coords,
    pub cost: u8,
    pub weight: u32,
    pub next_pos: Option<Vec2>,
    pub tile_type: TileType,
}

impl Tile {
    fn new(x: usize, y: usize) -> Self {
        Tile {
            coords: Coords { x, y },
            cost: 1,
            weight: u32::MAX,
            next_pos: None,
            tile_type: TileType::Neutral,
        }
    }

    pub fn is_even(&self) -> bool {
        (self.coords.x + self.coords.y) % 2 == 0
    }

    /// Updates the tile's colour to what it should be based on its [`TileType`] and `self.is_even()`.
    ///  
    /// `sprite` is the [`Sprite`] to update.
    /// `tile_colours` is the Res that holds the possible colours.
    fn update_colour(&self, sprite: &mut Sprite, tile_colours: &Res<TileColours>) {
        match self.tile_type {
            TileType::Neutral => {
                if self.is_even() {
                    if sprite.color != tile_colours.even_grass {
                        sprite.color = tile_colours.even_grass;
                    }
                } else {
                    if sprite.color != tile_colours.odd_grass {
                        sprite.color = tile_colours.odd_grass;
                    }
                }
            }
            TileType::Friendly => {
                if self.is_even() {
                    if sprite.color != tile_colours.even_friend {
                        sprite.color = tile_colours.even_friend;
                    }
                } else {
                    if sprite.color != tile_colours.odd_friend {
                        sprite.color = tile_colours.odd_friend;
                    }
                }
            }
            TileType::Hostile => {
                if self.is_even() {
                    if sprite.color != tile_colours.even_hostile {
                        sprite.color = tile_colours.even_hostile;
                    }
                } else {
                    if sprite.color != tile_colours.odd_hostile {
                        sprite.color = tile_colours.odd_hostile;
                    }
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum TileType {
    Neutral,
    Friendly,
    Hostile,
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

    #[allow(dead_code)]
    fn get_coords(&self, coords: Coords) -> Option<Entity> {
        self.get_xy(coords.x, coords.y)
    }

    #[allow(dead_code)]
    fn get_neighbours(&self, coords: Coords, eight_connected: bool) -> Vec<Option<Entity>> {
        let mut v = Vec::new();

        let x = coords.x;
        let y = coords.y;

        // top
        v.push(self.get_xy(x, y + 1));
        if eight_connected {
            // top right
            v.push(self.get_xy(x + 1, y + 1));
        }
        // right
        v.push(self.get_xy(x + 1, y));
        if y > 0 {
            if eight_connected {
                // bottom right
                v.push(self.get_xy(x + 1, y - 1));
            }
            // bottom
            v.push(self.get_xy(x, y - 1));
            if x > 0 && eight_connected {
                // bottom left
                v.push(self.get_xy(x - 1, y - 1));
            }
        }
        if x > 0 {
            // left
            v.push(self.get_xy(x - 1, y));
            if eight_connected {
                // up left
                v.push(self.get_xy(x - 1, y + 1));
            }
        }

        v
    }

    /// Gets a ring of entities around a point `center`. Ring is hollow.
    /// If you want it solid, run it for each distance.
    /// This will get a diamond shape `distance` tiles away from `center`.</br>
    /// `distance`: 1 is the 4 cardinal neighbours around the tile.</br>
    /// `distance`: 2 is the 8 tiles in a diamond adjacent to those 4.
    #[allow(unused_comparisons)] // reason = "Compare >= 0 before converting to usize")
    fn get_diamond_ring(&self, center: Coords, distance: usize) -> Vec<Option<Entity>> {
        let mut v = Vec::new();

        let x = center.x;
        let y = center.y;

        // dist 0 is self
        if distance == 0 {
            v.push(self.get_coords(center));
            return v;
        }

        // need to keep negative offsets
        // for larger rings, maybe the edge is off the grid, but not all values are
        // so as it iterates, some might come back to non-negative

        // up
        let mut up_x = x as i32;
        let mut up_y = (y + distance) as i32;
        for _ in 0..distance {
            if x >= 0 && y >= 0 {
                v.push(self.get_xy(up_x as usize, up_y as usize));
            }
            // move down right
            up_x += 1;
            up_y -= 1;
        }
        // right
        let mut right_x = (x + distance) as i32;
        let mut right_y = y as i32;
        for _ in 0..distance {
            if x >= 0 && y >= 0 {
                v.push(self.get_xy(right_x as usize, right_y as usize));
            }
            // move down left
            right_x -= 1;
            right_y -= 1;
        }
        // down
        let mut down_x = x as i32;
        let mut down_y = y as i32 - distance as i32;
        for _ in 0..distance {
            if x >= 0 && y >= 0 {
                v.push(self.get_xy(down_x as usize, down_y as usize));
            }
            // move up left
            down_x -= 1;
            down_y += 1;
        }
        // left
        let mut left_x = x as i32 - distance as i32;
        let mut left_y = y as i32;
        for _ in 0..distance {
            if x >= 0 && y >= 0 {
                v.push(self.get_xy(left_x as usize, left_y as usize));
            }
            // move up right
            left_x += 1;
            left_y += 1;
        }

        v
    }
}

struct TileColours {
    even_grass: Color,
    odd_grass: Color,
    even_friend: Color,
    odd_friend: Color,
    even_hostile: Color,
    odd_hostile: Color,
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
            // #124e89 #0099db
            odd_friend: Color::rgb_u8(0x12, 0x4e, 0x89),
            even_friend: Color::rgb_u8(0x00, 0x99, 0xdb),
            // #3e2731 #a22633
            odd_hostile: Color::rgb_u8(0x3e, 0x27, 0x31),
            even_hostile: Color::rgb_u8(0xa2, 0x26, 0x33),
            hover_color: Color::ALICE_BLUE,
            select_color: Color::MIDNIGHT_BLUE,
        }
    }
}

#[derive(Component)]
pub struct Selection;

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

fn clear_interaction(mut q_tiles: Query<&mut Interaction, With<Tile>>) {
    for mut interaction in q_tiles.iter_mut() {
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

/// Handles the interaction
///
/// `Clicked` adds [`Selected`]<br />
/// `Hovered` changes the colour to a highlight<br />
/// `None` changes the colour back to its base   
fn tile_interaction(
    mut commands: Commands,
    mut q_interaction: Query<
        (Entity, &Interaction, &mut Sprite, &Tile),
        (Without<TerritoryFlipper>, Without<Selection>),
    >,
    tile_colours: Res<TileColours>,
) {
    for (entity, interaction, mut sprite, tile) in q_interaction.iter_mut() {
        // Without<ColourChanger> to skip tiles that are currently animating
        match *interaction {
            Interaction::Clicked => {
                sprite.color = tile_colours.select_color;
                commands.entity(entity).insert(Selection);
            }
            Interaction::Hovered => {
                sprite.color = tile_colours.hover_color;
            }
            Interaction::None => {
                tile.update_colour(&mut sprite, &tile_colours);
            }
        }
    }
}

pub fn clear_selection(
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
    q_tiles: Query<&mut Tile>,
) {
    if keyboard.just_pressed(KeyCode::F) {
        let mut dest = Coords::from_vec2(mouse.0);

        // clamp in range
        dest.x = dest.x.clamp(0, GRID_WIDTH - 1);
        dest.y = dest.y.clamp(0, GRID_HEIGHT - 1);
        // dest.x = dest.x.min(GRID_WIDTH-1);
        // dest.y = dest.y.min(GRID_HEIGHT-1);

        generate_flow_field_grid(dest, grid, q_tiles);
    }
}

pub struct ChangeAllegianceEvent {
    pub center_coords: Coords,
    pub range: u32,
}

fn change_allegiance(
    mut commands: Commands,
    mut ev_change: EventReader<ChangeAllegianceEvent>,
    mut q_tiles: Query<&mut Tile>,
    grid: Res<Grid>,
) {
    for ev in ev_change.iter() {
        for i in 0..ev.range {
            let neighbours = grid.get_diamond_ring(ev.center_coords, i as usize);
            for n in neighbours.iter().flatten() {
                if let Ok(mut tile) = q_tiles.get_mut(*n) {
                    match tile.tile_type {
                        TileType::Neutral => {
                            tile.tile_type = TileType::Friendly;
                        }
                        TileType::Friendly => {
                            tile.tile_type = TileType::Hostile;
                        }
                        TileType::Hostile => {
                            tile.tile_type = TileType::Neutral;
                        }
                    }
                    commands.entity(*n).insert(TerritoryFlipper::new(10 * i));
                }
            }
        }
    }
}

#[derive(Component)]
pub struct TerritoryGrabber {
    range: u32,
    timer: Timer,
    // my allegiance to refresh instead of cycling.
    // todo
}

impl TerritoryGrabber {
    pub fn new(range: u32) -> Self {
        let mut timer = Timer::from_seconds(10.0, true);
        // ticks immediately the first time
        // if it's exact, I need grab_territory to run after whatever system
        // that inserts the component
        timer.tick(Duration::from_secs_f32(9.9));
        TerritoryGrabber { range, timer }
    }
}

fn grab_territory(
    mut q_grabber: Query<(&Tile, &mut TerritoryGrabber)>,
    mut ev_allegiance: EventWriter<ChangeAllegianceEvent>,
    time: Res<Time>,
) {
    for (tile, mut grabber) in &mut q_grabber {
        if grabber.timer.tick(time.delta()).just_finished() {
            // send
            ev_allegiance.send(ChangeAllegianceEvent {
                center_coords: tile.coords,
                range: grabber.range,
            });
        }
    }
}

// place a tower
// then add the colour changer to each thing in range
// give each ring a different start time
// then have a system that iterates over each colourchanger
// changing scale and colour at 50%
// who sets TileType and when?
// set at beginning by tower
// don't update interactions while component attached
// when finished, remove component
// what about an enemy being there when it flips?
// enemy code ignores tiles with the colour changer too

#[derive(Component)]
struct TerritoryFlipper {
    animation_timer: Timer,
    start_frame: u32,
    current_frame: u32,
    colour_changed_already: bool,
}

impl TerritoryFlipper {
    fn new(start_frame: u32) -> Self {
        TerritoryFlipper {
            animation_timer: Timer::from_seconds(0.5, false),
            start_frame,
            current_frame: 0,
            colour_changed_already: false,
        }
    }
}

fn territory_flip_animation(
    mut commands: Commands,
    mut q_territories: Query<(
        Entity,
        &mut Transform,
        &Tile,
        &mut Sprite,
        &mut TerritoryFlipper,
    )>,
    tile_colours: Res<TileColours>,
    time: Res<Time>,
) {
    for (entity, mut trans, tile, mut sprite, mut flipper) in q_territories.iter_mut() {
        if flipper.current_frame == flipper.start_frame {
            let percent = flipper.animation_timer.tick(time.delta()).percent();

            if percent < 0.5 {
                // shouldn't be linear
                // spinning tile in 2D should be
                // sin()
                // from PI/2 to PI
                // that's 1.0 to 0.0 in the shape I want I think
                // start at 1.0, then slowly go to 0.0
                // so the lerp is backwards
                let p = FRAC_PI_2 + FRAC_PI_2 * percent * 2.0;
                trans.scale = Vec3::lerp(Vec3::new(1.0, 0.0, 1.0), Vec3::ONE, f32::sin(p));
            } else {
                // only run once
                if !flipper.colour_changed_already {
                    flipper.colour_changed_already = true;
                    tile.update_colour(&mut sprite, &tile_colours);
                }

                // want from 0 to pi/2
                // have 0.5-1.0
                // convert 0.5-1.0 to 0.0-1.0
                // (percent - 0.5) * 2.0
                let p = (percent - 0.5) * 2.0 * FRAC_PI_2;
                trans.scale = Vec3::lerp(Vec3::new(1.0, 0.0, 1.0), Vec3::ONE, f32::sin(p));
            }

            if flipper.animation_timer.just_finished() {
                commands.entity(entity).remove::<TerritoryFlipper>();
            }
        } else {
            flipper.current_frame += 1;
        }
    }
}
