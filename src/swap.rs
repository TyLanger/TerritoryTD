use bevy::prelude::*;

use crate::{
    grid::{Coords, Grid, Tile, TILE_SIZE},
    resource_container::Resource,
    MouseWorldPos,
};

pub struct SwapPlugin;

impl Plugin for SwapPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(start_drag)
            .add_system(drop.after(drag_selection))
            .add_system(drag_selection)
            .add_system(swap_selection);
    }
}

#[derive(Component)]
pub struct Draggable {
    home: Vec3,
}

impl Draggable {
    pub fn new(start_pos: Vec3) -> Self {
        Draggable { home: start_pos }
    }
}

#[derive(Component)]
struct Dragged;

fn start_drag(
    mut commands: Commands,
    mouse: Res<MouseWorldPos>,
    mouse_click: Res<Input<MouseButton>>,
    q_draggable: Query<(Entity, &Transform, &Sprite), With<Draggable>>,
) {
    if mouse_click.just_pressed(MouseButton::Left) {
        for (entity, trans, sprite) in &q_draggable {
            // is the mouse inside this object?

            if let Some(size) = sprite.custom_size {
                let half_width = size.x / 2.0;
                let half_height = size.y / 2.0;
                if trans.translation.x - half_width < mouse.0.x
                    && trans.translation.x + half_width > mouse.0.x
                    && trans.translation.y - half_height < mouse.0.y
                    && trans.translation.y + half_height > mouse.0.y
                {
                    // mouse is inside
                    // println!("Mouse selected something");
                    commands.entity(entity).insert(Dragged);
                }
            }
        }
    }
}

fn drop(
    mut commands: Commands,
    mouse_click: Res<Input<MouseButton>>,
    mut q_dragged: Query<(Entity, &mut Transform, &Draggable), With<Dragged>>,
) {
    if mouse_click.just_released(MouseButton::Left) {
        for (entity, mut trans, drag) in &mut q_dragged {
            // println!("Dropped {:?} at {:?}", entity, trans.translation);
            trans.translation = drag.home;
            commands.entity(entity).remove::<Dragged>();
        }
    }
}

fn drag_selection(
    mouse: Res<MouseWorldPos>,
    mut q_dragged: Query<(&mut Transform, &Draggable), With<Dragged>>,
) {
    for (mut trans, drag) in &mut q_dragged {
        let distance = Vec2::ONE * TILE_SIZE * 0.55;
        let leash_pos = mouse.0.clamp(
            drag.home.truncate() - distance,
            drag.home.truncate() + distance,
        );
        trans.translation = leash_pos.extend(1.0);

        // is there something underneath that you can swap to?
    }
}

fn swap_selection(
    mut q_dragged: Query<(&Transform, &mut Tile, &mut Resource, &mut Draggable), With<Dragged>>,
    mut q_draggable: Query<
        (&mut Transform, &mut Tile, &mut Resource, &mut Draggable),
        Without<Dragged>,
    >,
    mut grid: ResMut<Grid>,
) {
    for (drag_trans, mut drag_tile, mut drag_res, mut drag) in &mut q_dragged {
        let coords = Coords::from_vec2(drag_trans.translation.truncate());
        if let Some(entity) = grid.get_coords(coords) {
            if let Ok((mut other_trans, mut other_tile, mut other_res, mut other_drag)) =
                q_draggable.get_mut(entity)
            {
                grid.swap(Coords::from_vec2(drag.home.truncate()), coords);

                let temp = other_drag.home;

                other_trans.translation = drag.home;
                other_drag.home = drag.home;
                drag.home = temp;

                let temp = other_tile.coords;

                other_tile.coords = drag_tile.coords;
                drag_tile.coords = temp;

                // group resources
                match (*drag_res, *other_res) {
                    (Resource::Gold(a), Resource::Gold(b)) => {
                        *drag_res = Resource::Gold(a + b);
                        *other_res = Resource::None;
                    }
                    _ => {}
                }
            }
        }
    }
}
