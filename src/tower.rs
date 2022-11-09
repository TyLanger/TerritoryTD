use bevy::prelude::*;

use crate::{
    grid::{ChangeAllegianceEvent, ClearSelectionsEvent, Selection, Tile},
    gun::{Gun, GunPlugin, GunType, BurstInfo},
    MouseWorldPos,
};

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(build_tower_system).add_system(tower_shoot);
    }
}

trait Tower: Component + Copy {
    fn build(&self, commands: &mut Commands, tile_ent: Entity)
    where
        Self: Sized,
    {
        commands.entity(tile_ent).with_children(|commands| {
            commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: self.get_color(),
                        custom_size: Some(Vec2::splat(20.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
                    ..default()
                })
                .insert(*self);
        });
    }

    fn get_color(&self) -> Color;

    fn shoot(&self) {
        println!("Shoot!");
    }
}

#[derive(Component, Copy, Clone)]
struct ArrowTower {
    color: Color,
}

impl ArrowTower {
    fn new() -> Self {
        ArrowTower {
            color: Color::GREEN,
        }
    }
}

impl Tower for ArrowTower {
    fn get_color(&self) -> Color {
        self.color
    }
}

#[derive(Component, Copy, Clone)]
struct BombTower {
    color: Color,
}

impl Tower for BombTower {
    fn get_color(&self) -> Color {
        self.color
    }
}

fn build_tower_system(
    mut commands: Commands,
    mut q_selection: Query<(Entity, &mut Tile), With<Selection>>,
    keyboard: Res<Input<KeyCode>>,
    mut ev_clear: EventWriter<ClearSelectionsEvent>,
    mut ev_allegiance: EventWriter<ChangeAllegianceEvent>,
) {
    if keyboard.just_pressed(KeyCode::T) {
        ev_clear.send(ClearSelectionsEvent);
        for (tile_ent, mut tile) in q_selection.iter_mut() {
            tile.cost = 200; // don't walk over towers
            ev_allegiance.send(ChangeAllegianceEvent {
                center_coords: tile.coords,
                range: 4,
            });
            // if tile.is_even() {
            //     ArrowTower::new().build(&mut commands, tile_ent);
            // } else {
            //     BombTower {
            //         color: Color::BLACK,
            //     }
            //     .build(&mut commands, tile_ent);
            // }
            if tile.is_even() {
                commands
                .entity(tile_ent)
                .insert(TowerComponent {})
                .insert(Gun::new(GunType::Pistol));
            } else {
                commands
                .entity(tile_ent)
                .insert(TowerComponent {})
                .insert(Gun::new(GunType::Burst(BurstInfo::from(0.1, 3))));
            }
            
        }
    }
}

// maybe Tower Trait
// shoot
// TowerComponent {
// tower: Tower
// }
// Query<TowerComponent>
// towercomponent.tower.shoot(command, etc)

// Tower Brain
// chooses when to shoot
// FastBrain
// shoots every 0.5s
// SlowBrain
// shoots every 2s

fn tower_shoot(
    mut commands: Commands,
    mut q_towers: Query<(&Transform, &TowerComponent, &mut Gun)>,
    mouse: Res<MouseWorldPos>,
    keyboard: Res<Input<KeyCode>>,
) {
    // if keyboard.just_pressed(KeyCode::S) {

    
    for (trans, tower, mut gun) in q_towers.iter_mut() {
        let dir = mouse.0 - trans.translation.truncate();
        gun.shoot(&mut commands, trans.translation, dir.normalize_or_zero());
    }

// }
}

#[derive(Component)]
struct TowerComponent {
    // gun: Gun,
    // brain: Box<dyn TowerBrain>,
}

trait TowerBrain: Sync + Send {
    fn think(&self);
}

// Maybe bundles is the way to go?
// what do I even want?
// is it bad to have arrow_tower_shoot, bullet_tower_shoot, bomb_tower_shoot?
