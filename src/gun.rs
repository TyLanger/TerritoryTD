use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::enemy::Enemy;

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

pub enum GunType {
    Pistol,
    Shotgun,
    Burst(BurstInfo),
}

// data the gun stores for how it operates
#[derive(Copy, Clone, PartialEq)]
pub struct BurstInfo {
    pub base_timer: f32,
    pub max_shots: u32,
}

impl BurstInfo {
    pub fn from(time_between_shots: f32, shots_in_burst: u32) -> Self {
        assert!(shots_in_burst > 0, "shots_in_burst can't be 0");
        BurstInfo {
            base_timer: time_between_shots,
            max_shots: shots_in_burst,
        }
    }
}

// once you shoot, the data about the extra bullets in the burst is stored here
#[derive(Copy, Clone, PartialEq)]
struct BurstInfoStorage {
    timer: f32,
    base_timer: f32,
    shots_left: u32,
    pos: Vec3,
    dir: Vec2,
}

impl BurstInfoStorage {
    fn new(info: BurstInfo, pos: Vec3, dir: Vec2) -> Self {
        BurstInfoStorage {
            timer: info.base_timer,
            base_timer: info.base_timer,
            shots_left: info.max_shots - 1,
            pos,
            dir,
        }
    }
}

#[derive(Component)]
pub struct Gun {
    bullet: Bullet,
    timer_between_shots: Timer,
    current_shots: u32,
    clip_size: u32,
    reload_timer: Timer,
    num_bullets_shot: u32,
    state: GunState,
    gun_type: GunType,
}

impl Gun {
    pub fn new(gun_type: GunType) -> Self {
        Gun {
            bullet: Bullet::new(Vec2::ZERO),
            timer_between_shots: Timer::from_seconds(0.3, true),
            current_shots: 6,
            clip_size: 6,
            reload_timer: Timer::from_seconds(1.0, true),
            num_bullets_shot: 1,
            state: GunState::Ready,
            gun_type,
        }
    }

    fn tick(&mut self, delta: Duration, commands: &mut Commands) {
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
            GunState::Firing(mut b) => {
                b.timer -= delta.as_secs_f32();
                if b.timer < 0.0 {
                    self.bullet.spawn(commands, b.pos, b.dir);
                    // b.shots is number in the burst
                    // self.current_shots is the number in your clip
                    b.shots_left -= 1;
                    self.current_shots -= 1;
                    // even if you have shots left in your burst,
                    // still need to reload if you run out of ammo
                    if self.current_shots == 0 {
                        self.state = GunState::Reloading;
                        return;
                    }
                    if b.shots_left > 0 {
                        b.timer = b.base_timer;
                    } else {
                        // once you've shot all 3 in the burst,
                        // do the normal gun ShotCooldown
                        self.state = GunState::ShotCooldown;
                        return;
                    }
                }
                self.state = GunState::Firing(b);
            }
            _ => {}
        }
    }

    pub fn shoot(&mut self, commands: &mut Commands, pos: Vec3, dir: Vec2) {
        if self.state == GunState::Ready {
            // spawn a bullet somehow
            // what do you need to spawn a bullet?
            // mut commands: Commands,
            // position, direction, etc.
            match self.gun_type {
                GunType::Pistol => {
                    self.bullet.spawn(commands, pos, dir);
                }
                GunType::Shotgun => {
                    self.bullet
                        .spawn(commands, pos + Vec3::new(0.0, 10.0, 0.0), dir);
                    self.bullet
                        .spawn(commands, pos + Vec3::new(0.0, -10.0, 0.0), dir);
                }
                GunType::Burst(b) => {
                    self.bullet.spawn(commands, pos, dir);
                    let storage = BurstInfoStorage::new(b, pos, dir);
                    self.state = GunState::Firing(storage);
                    // println!("Start burst");
                    // return to not go into shotCooldown
                    // burst handles its own shotcooldown in tick()
                    self.current_shots -= 1;
                    return;
                }
            }

            self.current_shots -= 1;
            if self.current_shots == 0 {
                self.state = GunState::Reloading;
            } else {
                self.state = GunState::ShotCooldown;
            }
        }
    }

    fn reload(&mut self) {
        self.current_shots = self.clip_size;
        self.state = GunState::Ready;
    }
}

#[derive(Copy, Clone, PartialEq)]
enum GunState {
    Ready,
    Firing(BurstInfoStorage),
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

    fn spawn(&self, commands: &mut Commands, pos: Vec3, dir: Vec2) {
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
            .insert(Bullet::new(dir).update_entity(ent))
            .insert(RigidBody::Dynamic)
            .insert(Collider::ball(8.0))
            .insert(Sensor);
    }

    fn despawn(&self, commands: &mut Commands) {
        commands.entity(self.entity.unwrap()).despawn_recursive();
    }
}

fn tick_bullets(
    mut commands: Commands,
    mut q_bullets: Query<(Entity, &mut Transform, &mut Bullet)>,
    rapier_context: Res<RapierContext>,
    mut q_enemies: Query<(Entity, &mut Enemy)>,
    time: Res<Time>,
) {
    for (bullet_ent, mut trans, mut bullet) in q_bullets.iter_mut() {
        trans.translation += bullet.dir.extend(0.0) * bullet.speed * time.delta_seconds();

        // does it hit anything?
        let collisions = rapier_context.intersections_with(bullet_ent);
        let mut hit_something = false;
        for (a, b, hit) in collisions {
            if hit {
                let enemy_ent = if a == bullet_ent { b } else { a };

                if let Ok((e_ent, mut enemy)) = q_enemies.get_mut(enemy_ent) {
                    // enemy.take_damage();
                    // println!("Killed something");
                    commands.entity(e_ent).despawn_recursive();
                    hit_something = true;
                }
            }
        }

        // might need to split these up later
        if hit_something || bullet.lifetime.tick(time.delta()).just_finished() {
            //die
            bullet.despawn(&mut commands);
        }
    }
}

fn tick_guns(mut commands: Commands, mut q_guns: Query<&mut Gun>, time: Res<Time>) {
    for mut gun in q_guns.iter_mut() {
        gun.tick(time.delta(), &mut commands);
    }
}

pub struct ShootEvent {
    pos: Vec3,
    dir: Vec2,
}

fn shoot_system(mut ev_shoot: EventReader<ShootEvent>) {}

// send a shoot event
// it will set the parameters on the gun
// then when the gun ticks, it will consume them
// if burst mode, it will shoot 3 shots back to back
// with the last given directions

// or burst works like
// shoot
// self.bullet.spawn(wait = 0)
// self.bullet.spawn(wait = 1)
// self.bullet.spawn(wait - 2)
// create all the bullets. take some time before being active
// I don't like the consequences of this. they will be visible just sitting there
