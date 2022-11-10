use bevy::prelude::*;

use crate::{
    grid::{ChangeAllegianceEvent, ClearSelectionsEvent, Selection, Tile},
    gun::{BurstInfo, Gun, GunType},
    loading::FontAssets,
    GameState, MouseWorldPos,
};

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_exit(GameState::Loading).with_system(create_tower_store_ui),
        );
        app.add_event::<BuildButtonEvent>()
            .add_system(build_tower_system.before(crate::grid::clear_selection))
            .add_system(tower_shoot)
            .add_system(update_button_colours);
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
    mut ev_clear: EventWriter<ClearSelectionsEvent>,
    mut ev_allegiance: EventWriter<ChangeAllegianceEvent>,
    mut ev_build: EventReader<BuildButtonEvent>,
) {
    for ev in ev_build.iter() {
        ev_clear.send(ClearSelectionsEvent);
        for (tile_ent, mut tile) in q_selection.iter_mut() {
            tile.cost = 200; // don't walk over towers
            ev_allegiance.send(ChangeAllegianceEvent {
                center_coords: tile.coords,
                range: 4,
            });

            match ev.tower_type {
                TowerType::Pistol => {
                    commands
                        .entity(tile_ent)
                        .insert(TowerComponent {})
                        .insert(Gun::new(GunType::Pistol));
                }
                TowerType::Shotgun => {
                    commands
                        .entity(tile_ent)
                        .insert(TowerComponent {})
                        .insert(Gun::new(GunType::Shotgun));
                }
                TowerType::Burst => {
                    commands
                        .entity(tile_ent)
                        .insert(TowerComponent {})
                        .insert(Gun::new(GunType::Burst(BurstInfo::from(0.1, 3))));
                }
                TowerType::NoGun => {
                    commands.entity(tile_ent).insert(TowerComponent {});
                }
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
) {
    for (trans, _tower, mut gun) in q_towers.iter_mut() {
        let dir = mouse.0 - trans.translation.truncate();
        gun.shoot(&mut commands, trans.translation, dir.normalize_or_zero());
    }
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

#[derive(Component)]
struct TowerStoreUI;

#[derive(Component, Debug)]
struct TowerBuildButton {
    tower_type: TowerType,
}

#[derive(Debug, Copy, Clone)]
enum TowerType {
    Pistol,
    Shotgun,
    Burst,
    NoGun,
}
struct BuildButtonEvent {
    tower_type: TowerType,
}

fn create_tower_store_ui(mut commands: Commands, fonts: Res<FontAssets>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                // right side of the screen
                position_type: PositionType::Absolute,
                position: UiRect {
                    right: Val::Px(0.0),
                    ..default()
                },
                size: Size::new(Val::Percent(15.0), Val::Percent(100.0)),
                // left-right
                justify_content: JustifyContent::SpaceEvenly,
                // up-down
                align_content: AlignContent::Center,
                //align_items: AlignItems::FlexEnd,
                flex_wrap: FlexWrap::Wrap,

                ..default()
            },
            // #262b44
            color: Color::rgb_u8(0x26, 0x2b, 0x44).into(),
            ..default()
        })
        .insert(TowerStoreUI)
        .with_children(|root| {
            let arr = [
                TowerType::Pistol,
                TowerType::Shotgun,
                TowerType::Burst,
                TowerType::NoGun,
            ];
            for t in arr.iter() {
                // let tower_type = TowerType::Pistol;
                root.spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Percent(40.0), Val::Auto),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        // coloured box around text
                        padding: UiRect {
                            left: Val::Px(2.0),
                            right: Val::Px(2.0),
                            top: Val::Px(8.0),
                            bottom: Val::Px(8.0),
                        },
                        // whitespace around the button
                        margin: UiRect {
                            left: Val::Px(2.0),
                            right: Val::Px(2.0),
                            top: Val::Px(2.0),
                            bottom: Val::Px(2.0),
                            //..default()
                        },
                        ..default()
                    },
                    color: Color::MIDNIGHT_BLUE.into(),
                    ..default()
                })
                .insert(TowerBuildButton { tower_type: *t })
                .with_children(|button_base| {
                    button_base.spawn_bundle(TextBundle::from_section(
                        format!("{:?}", t),
                        TextStyle {
                            font: fonts.fira_sans.clone(),
                            font_size: 20.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                });
            }
        });
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn update_button_colours(
    mut q_interaction: Query<
        (&Interaction, &mut UiColor, &TowerBuildButton),
        (Changed<Interaction>, With<Button>, With<TowerBuildButton>),
    >,
    mut ev_click: EventWriter<BuildButtonEvent>,
) {
    for (interaction, mut color, tower) in q_interaction.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                // println!("Clicked {:?}", tower);
                ev_click.send(BuildButtonEvent {
                    tower_type: tower.tower_type,
                });
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}
