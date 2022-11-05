use std::time::Duration;

use bevy::prelude::*;

pub struct GunPlugin;

impl Plugin for GunPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(tick_bullets).add_system(tick_guns);
    }
}

// What is a gun?
// has a bullet
// has time between bullets
// clip size
// reload time
// number of bullet shot

#[derive(Component)]
struct Gun {
    bullet: Bullet,
    timer_between_shots: Timer,
    current_shots: u32,
    clip_size: u32,
    reload_timer: Timer,
    num_bullets_shot: u32,
    state: GunState,
}

impl Gun {
    fn new() -> Self {
        Gun {
            bullet: Bullet::new(Vec2::ZERO),
            timer_between_shots: Timer::from_seconds(0.1, true),
            current_shots: 6,
            clip_size: 6,
            reload_timer: Timer::from_seconds(1.0, true),
            num_bullets_shot: 1,
            state: GunState::Ready,
        }
    }

    fn tick(&mut self, delta: Duration) {
        match self.state {
            GunState::ShotCooldown => {
                if self.timer_between_shots.tick(delta).just_finished() {
                    self.state = GunState::Ready;
                }
            }
            GunState::Reloading => {
                if self.reload_timer.tick(delta).just_finished() {
                    self.reload();
                }
            }
            _ => {}
        }
    }

    fn shoot(&mut self) {
        if self.state == GunState::Ready {
            self.current_shots -= 1;
            // spawn a bullet somehow
            // what do you need to spawn a bullet?
            // mut commands: Commands,
            // position, direction, etc.
            if self.current_shots == 0 {
                self.state = GunState::Reloading;
            } else {
                self.state = GunState::ShotCooldown;
            }
        }
    }

    fn reload(&mut self) {
        self.state = GunState::Ready;
        self.current_shots = self.clip_size;
    }
}

#[derive(Copy, Clone, PartialEq)]
enum GunState {
    Ready,
    ShotCooldown,
    Reloading,
}

#[derive(Component)]
struct Bullet {
    dir: Vec2,
    speed: f32,
    lifetime: Timer,
    entity: Option<Entity>,
}

impl Bullet {
    fn new(dir: Vec2) -> Self {
        Bullet {
            dir,
            speed: 100.0,
            lifetime: Timer::from_seconds(2.0, false),
            entity: None,
        }
    }

    fn update_entity(mut self, entity: Entity) -> Self {
        self.entity = Some(entity);
        self
    }

    fn spawn(&self, mut commands: Commands, pos: Vec3, dir: Vec2) {
        let ent = commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::BLUE,
                    custom_size: Some(Vec2::splat(16.0)),
                    ..default()
                },
                transform: Transform {
                    translation: pos,
                    ..default()
                },
                ..default()
            })
            .id();

        commands
            .entity(ent)
            .insert(Bullet::new(dir).update_entity(ent));
    }

    fn despawn(&self, commands: &mut Commands) {
        commands.entity(self.entity.unwrap()).despawn_recursive();
    }
}

fn tick_bullets(
    mut commands: Commands,
    mut q_bullets: Query<(&mut Transform, &mut Bullet)>,
    time: Res<Time>,
) {
    for (mut trans, mut bullet) in q_bullets.iter_mut() {
        trans.translation += bullet.dir.extend(0.0) * bullet.speed * time.delta_seconds();
        if bullet.lifetime.tick(time.delta()).just_finished() {
            //die
            bullet.despawn(&mut commands);
        }
    }
}

fn tick_guns(mut q_guns: Query<&mut Gun>, time: Res<Time>) {
    for mut gun in q_guns.iter_mut() {
        gun.tick(time.delta());
    }
}
