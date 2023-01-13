use bevy::prelude::*;

use crate::{
    grid::{Coords, Grid, Tile, TILE_SIZE},
    resource_container::Resource,
    MouseWorldPos,
};

pub struct SwapPlugin;

impl Plugin for SwapPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SwapEvent>()
            .add_system(start_drag)
            .add_system(drop.after(drag_selection))
            .add_system(drag_selection)
            .add_system(swap_selection)
            .add_system(swap_event_combine_resources)
            .add_system(swap_event_update_drag_home)
            .add_system(swap_event_update_tile_coords)
            .add_system(swap_event_update_grid);
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

struct SwapEvent {
    from: Coords,
    to: Coords,
    from_ent: Entity,
    to_ent: Entity,
}

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
    }
}

fn swap_selection(
    q_dragged: Query<(Entity, &Transform, &Tile), (With<Draggable>, With<Dragged>)>,
    q_draggable: Query<(Entity, &Tile), (With<Draggable>, Without<Dragged>)>,
    grid: Res<Grid>,
    mut ev_swap: EventWriter<SwapEvent>,
) {
    for (drag_ent, drag_trans, drag_tile) in &q_dragged {
        let coords = Coords::from_vec2(drag_trans.translation.truncate());
        if let Some(entity) = grid.get_coords(coords) {
            if let Ok((other_ent, other_tile)) = q_draggable.get(entity) {
                ev_swap.send(SwapEvent {
                    from: drag_tile.coords,
                    to: other_tile.coords,
                    from_ent: drag_ent,
                    to_ent: other_ent,
                });
            }
        }
    }
}

fn swap_event_update_grid(mut grid: ResMut<Grid>, mut ev_swap: EventReader<SwapEvent>) {
    for ev in ev_swap.iter() {
        grid.swap(ev.from, ev.to);
    }
}

fn swap_event_update_tile_coords(
    mut q_tiles: Query<&mut Tile>,
    mut ev_swap: EventReader<SwapEvent>,
) {
    for ev in ev_swap.iter() {
        let from = ev.from_ent;
        let to = ev.to_ent;

        if let Ok(mut v) = q_tiles.get_many_mut([from, to]) {
            let temp = v[0].coords;
            v[0].coords = v[1].coords;
            v[1].coords = temp;
        }
    }
}

fn swap_event_update_drag_home(
    mut q_draggable: Query<(&mut Transform, &mut Draggable)>,
    mut ev_swap: EventReader<SwapEvent>,
) {
    for ev in ev_swap.iter() {
        if let Ok(mut v) = q_draggable.get_many_mut([ev.from_ent, ev.to_ent]) {
            let temp = v[0].1.home;
            v[0].1.home = v[1].1.home;
            v[1].1.home = temp;

            v[1].0.translation = v[1].1.home;
        }
    }
}

fn swap_event_combine_resources(
    mut q_resource: Query<&mut Resource>,
    mut ev_swap: EventReader<SwapEvent>,
) {
    for ev in ev_swap.iter() {
        if let Ok(mut v) = q_resource.get_many_mut([ev.from_ent, ev.to_ent]) {
            match (*v[0], *v[1]) {
                (Resource::Gold(a), Resource::Gold(b)) => {
                    *v[0] = Resource::Gold(a + b);
                    *v[1] = Resource::None;
                }
                _ => {}
            }
        }
    }
}
