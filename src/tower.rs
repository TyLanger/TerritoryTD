use bevy::prelude::*;

use crate::{
    gold::GoldSpawner,
    grid::{ChangeAllegianceEvent, ClearSelectionsEvent, Selection, TerritoryGrabber, Tile},
    gun::{BurstInfo, EndBehaviour, ExplosionInfo, Gun, GunType},
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
            .add_system(tower_build_buttons_interactions.before(build_tower_system))
            .add_system(build_tower_system.before(crate::grid::clear_selection))
            .add_system(tower_shoot);
    }
}

fn build_tower_system(
    mut commands: Commands,
    mut q_selection: Query<(Entity, &mut Tile), With<Selection>>,
    mut ev_clear: EventWriter<ClearSelectionsEvent>,
    mut ev_build: EventReader<BuildButtonEvent>,
) {
    for ev in ev_build.iter() {
        ev_clear.send(ClearSelectionsEvent);
        for (tile_ent, mut tile) in q_selection.iter_mut() {
            tile.cost = 200; // don't walk over towers

            match ev.tower_type {
                TowerType::Pistol => {
                    commands
                        .entity(tile_ent)
                        .insert(TowerComponent {})
                        .insert(Gun::new(GunType::Pistol, EndBehaviour::None))
                        .insert(TerritoryGrabber::new(4));
                }
                TowerType::Shotgun => {
                    commands
                        .entity(tile_ent)
                        .insert(TowerComponent {})
                        .insert(Gun::new(
                            GunType::Shotgun,
                            EndBehaviour::Explode(ExplosionInfo::new(30.0, 5)),
                        ))
                        .insert(TerritoryGrabber::new(2));
                }
                TowerType::Burst => {
                    commands
                        .entity(tile_ent)
                        .insert(TowerComponent {})
                        .insert(Gun::new(
                            GunType::Burst(BurstInfo::from(0.1, 3)),
                            EndBehaviour::Split(2),
                        ))
                        .insert(TerritoryGrabber::new(3));
                }
                TowerType::Bomb => {
                    commands
                        .entity(tile_ent)
                        .insert(TowerComponent {})
                        .insert(Gun::new(
                            GunType::Bomb,
                            EndBehaviour::Explode(ExplosionInfo::new(30.0, 5)),
                        ))
                        .insert(TerritoryGrabber::new(3));
                }
                TowerType::NoGun => {
                    commands
                        .entity(tile_ent)
                        .insert(TowerComponent {})
                        .insert(GoldSpawner::new());
                }
            }
        }
    }
}

fn tower_shoot(
    mut commands: Commands,
    mut q_towers: Query<(Entity, &Transform, &TowerComponent, &mut Gun)>,
    mouse: Res<MouseWorldPos>,
) {
    for (entity, trans, _tower, mut gun) in q_towers.iter_mut() {
        let dir = mouse.0 - trans.translation.truncate();
        gun.shoot(
            &mut commands,
            trans.translation,
            dir.normalize_or_zero(),
            entity,
        );
    }
}

#[derive(Component)]
struct TowerComponent {
    // gun: Gun,
    // brain: Box<dyn TowerBrain>,
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
    Bomb,
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
                TowerType::Bomb,
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

fn tower_build_buttons_interactions(
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
