//! Renders a 2D scene containing a single, moving sprite.

use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, sprite::Anchor, time::FixedTimestep};
// use rand::Fill;
use getrandom::getrandom;

const TIME_STEP:f32 = 1.0 / 60.0;

const W_WIDTH: f32 = 1080.;
const W_HEIGHT: f32 = 720.;

fn get_randu64() -> u64 {
    let result: u64 = unsafe {
        let mut data = [0u8; 8];
        getrandom(&mut data[..]).unwrap();
        std::mem::transmute(data)
    };
    result
}

fn get_rand01() -> f64 {
    let v1 = loop {
        let v = get_randu64();
        if v != 0 {
            break v;
        }
    };
    let v2 = loop {
        let v = get_randu64();
        if v != 0 {
            break v;
        }
    };

    if v1 > v2 {
        v2 as f64 / v1 as f64
    } else {
        v1 as f64 / v2 as f64
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "PPOid".to_string(),
            width: W_WIDTH,
            height: W_HEIGHT,
            resizable: false,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                // .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(sprite_movement)
                .with_system(move_block.before(sprite_movement))
                .with_system(warp_system.after(sprite_movement))
                .with_system(bullet_hits.after(warp_system))
                .with_system(player_hits.after(bullet_hits))
                .with_system(cleanup.after(player_hits))
        )
        .run();
}

#[derive(Component, Default)]
struct Movment {
    speed: f32,
    heading: f32,
    look_direction: f32,
}

impl Movment {
    fn fill_rand(self: &mut Self) {
        self.speed = get_rand01() as f32 * 300. + 200.;
        self.heading = get_rand01() as f32 * PI * 2.;
        self.look_direction = self.heading;
    }
}
// impl rand::Fill for Movment {
//     fn try_fill<R: rand::Rng + ?Sized>(&mut self, rng: &mut R) -> Result<(), rand::Error> {
//         self.speed = rng.gen::<f32>() * 300. + 200.;
//         let heading = rng.gen::<f32>() * PI * 2.;
//         self.heading = heading;
//         self.look_direction = heading;
//         Ok(())
//     }
// }

#[derive(Component)]
struct Player(Timer);

#[derive(Component)]
struct Nowarp;

#[derive(Component)]
struct Enemy(u32);

#[derive(Component)]
struct Bullet;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("patron.png"),
            sprite: Sprite {
                // color: Color::rgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(42.0, 75.0)),
                anchor: Anchor::TopCenter,
                ..default()
            },
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        })
        .insert(Movment::default())
        .insert(Player(Timer::from_seconds(0.2, false).tick(Duration::from_secs_f32(0.2)).to_owned()));

    for i in 0..3 {
        let mut movement = Movment::default();
        // movement.try_fill(&mut rng).unwrap();
        movement.fill_rand();
        commands
            .spawn_bundle(SpriteBundle {
                // texture: asset_server.load("branding/icon.png"),
                sprite: Sprite {
                    color: Color::GOLD,
                    custom_size: Some(Vec2::new(35.0, 50.0)),
                    ..default()
                },
                transform: Transform::from_xyz(1080. * get_rand01() as f32 - 540., 720. * get_rand01() as f32 - 360., 0.),
                ..default()
            })
            .insert(movement)
            .insert(Enemy(2));
    }
}

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn sprite_movement(time: Res<Time>, mut sprite_position: Query<(&Movment, &mut Transform)>) {
    for (move_params, mut transform) in &mut sprite_position {
        let delta = Transform::from_xyz(0., move_params.speed * time.delta_seconds(), 0.);
        // info!("Delta {delta}, rotation deg: {}", move_params.heading);
        transform.rotate_local_z(-move_params.look_direction + move_params.heading);
        *transform = transform.mul_transform(delta).with_rotation(Quat::from_rotation_z(move_params.heading));
        transform.rotate_local_z(move_params.look_direction - move_params.heading);
    }
}

fn move_block(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut query: Query<(&mut Movment, &Transform, &mut Player), With<Player>>,
) {
    if let Ok((mut block, trans, mut pl)) = query.get_single_mut() {
        pl.0.tick(time.delta());

        if keyboard_input.pressed(KeyCode::W) {     
            let delta_speed = 250. * time.delta_seconds();
            let new_speed = (((delta_speed * block.look_direction.sin()) + (block.speed * block.heading.sin())).powi(2) + 
                ((delta_speed * block.look_direction.cos()) + (block.speed * block.heading.cos())).powi(2)).sqrt();
            let new_heading = ((delta_speed * block.look_direction.sin()) + (block.speed * block.heading.sin()))
                .atan2((delta_speed * block.look_direction.cos()) + (block.speed * block.heading.cos()));

            block.speed = new_speed;
            block.heading = new_heading;

            if block.speed > 500. {
                block.speed = 500.
            }
        }

        if keyboard_input.pressed(KeyCode::A) {
            block.look_direction += (180. * time.delta_seconds()).to_radians();
        }

        if keyboard_input.pressed(KeyCode::D) {
            block.look_direction -= (180. * time.delta_seconds()).to_radians();
        }
        
        if keyboard_input.pressed(KeyCode::Space) {
            if pl.0.finished() {
                commands.spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::WHITE,
                        custom_size: Some(Vec2::new(5.0, 17.0)),
                        ..default()
                    },
                    transform: trans.mul_transform(Transform::from_translation(trans.up())),
                    ..default()
                })
                    .insert(Movment {speed: 700., 
                        heading: block.look_direction, 
                        look_direction: block.look_direction})
                    .insert(Nowarp)
                    .insert(Bullet);
                pl.0.reset();
            }
        }
    }
}

fn warp_system(mut query: Query<&mut Transform, Without<Nowarp>>) {
    for mut b in &mut query {
        if b.translation.x.abs() > W_WIDTH / 2. + 5. {
            b.translation.x = -(b.translation.x - (5. * b.translation.x.signum()))
        }
        if b.translation.y.abs() > W_HEIGHT / 2. + 5. {
            b.translation.y = -(b.translation.y  - (5. * b.translation.y.signum()))
        }
    }
}

fn cleanup(mut commands: Commands,
    query: Query<(Entity, &Transform), With<Nowarp>>) {
        for (e, b) in &query {
            match (b.translation.x.abs(), b.translation.y.abs()) {
                (a, _) if a > W_WIDTH / 2. => {commands.entity(e).despawn();},
                (_, b) if b > W_HEIGHT / 2. => {commands.entity(e).despawn();},
                _ => ()
            }
        }
}

fn check_colision(t1: &Transform, w1: f32, h1: f32, r1: f32, t2: &Transform, w2: f32, h2: f32, r2:f32) -> bool {
    let mut t1c = t1.clone();
    let mut t2c = t2.clone();

    t2c.translation -= t1.translation;

    for (x, y) in [(0.5, 0.5), (-0.5, 0.5), (0.5, -0.5), (-0.5, -0.5)] {
        let p = Vec2::from_angle(r2 - r1).rotate(Vec2{x: w2 * x + t2c.translation.x, y: h2 * y + t2c.translation.y});
        if p.abs().cmple(Vec2{x: w1 / 2., y: h1 / 2.}).all() {
            return true;
        }
    }

    t1c.translation -= t2.translation;

    for (x, y) in [(0.5, 0.5), (-0.5, 0.5), (0.5, -0.5), (-0.5, -0.5)] {
        let p = Vec2::from_angle(r1 - r2).rotate(Vec2{x: w1 * x + t1c.translation.x, y: h1 * y + t1c.translation.y});
        if p.abs().cmple(Vec2{x: w2 / 2., y: h2 / 2.}).all() {
            return true;
        }
    }

    false
}

fn bullet_hits(   
    mut commands: Commands,
    bullets: Query<(Entity, &Transform, &Movment), With<Bullet>>,
    enemies: Query<(Entity, &Transform, &Movment, &Enemy), With<Enemy>>
) {
    bullets.for_each(|(be, bt, bm)| {
        enemies.for_each(|(ee, et, em, e)| {
            if check_colision(bt, 5., 17., bm.look_direction, et, 35., 50., em.look_direction) {
                commands.entity(be).despawn();
                let lifes = e.0;
                commands.entity(ee).despawn();

                if lifes > 0 {
                    for _i in 0..2 {
                        // let mut rng = rand::rngs::OsRng;
                        let mut movement = Movment::default();
                        movement.fill_rand();
                        // movement.try_fill(&mut rng).unwrap();
                        commands
                            .spawn_bundle(SpriteBundle {
                                // texture: asset_server.load("branding/icon.png"),
                                sprite: Sprite {
                                    color: Color::GOLD,
                                    custom_size: Some(Vec2::new(35.0, 50.0)),
                                    ..default()
                                },
                                transform: et.clone(),
                                ..default()
                            })
                            .insert(movement)
                            .insert(Enemy(lifes - 1));
                    }
                }
            }
        })
    })

}

fn player_hits(   
    mut commands: Commands,
    player: Query<(&Transform, &Movment), With<Player>>,
    enemies: Query<(&Transform, &Movment), With<Enemy>>,
    ents: Query<Entity, Without<Camera>>,
    asset_server: Res<AssetServer>
) {
    if let Ok((pt, pm)) = player.get_single() {
        enemies.for_each(|(et, em)| {
            let mut p_centr = pt.clone();
            p_centr = p_centr.mul_transform(Transform::from_xyz(0., -75. / 2., 0.)).with_rotation(Quat::from_rotation_z(pm.look_direction));
            if check_colision(&p_centr, 42., 75., pm.look_direction, et, 35., 50., em.look_direction) {
                for e in &ents {
                    commands.entity(e).despawn_recursive();
                }

                commands
                    .spawn_bundle(SpriteBundle {
                        texture: asset_server.load("patron.png"),
                        sprite: Sprite {
                            // color: Color::rgb(0.25, 0.25, 0.75),
                            custom_size: Some(Vec2::new(42.0, 75.0)),
                            anchor: Anchor::TopCenter,
                            ..default()
                        },
                        transform: Transform::from_xyz(0., 0., 0.),
                        ..default()
                    })
                    .insert(Movment::default())
                    .insert(Player(Timer::from_seconds(0.2, false).tick(Duration::from_secs_f32(0.2)).to_owned()));

                for _i in 0..3 {
                    // let mut rng = rand::rngs::OsRng;
                    let mut movement = Movment::default();
                    movement.fill_rand();
                    // movement.try_fill(&mut rng).unwrap();
                    commands
                        .spawn_bundle(SpriteBundle {
                            // texture: asset_server.load("branding/icon.png"),
                            sprite: Sprite {
                                color: Color::GOLD,
                                custom_size: Some(Vec2::new(35.0, 50.0)),
                                ..default()
                            },
                            transform: Transform::from_xyz(1080. * get_rand01() as f32 - 540., 720. * get_rand01() as f32 - 360., 0.),
                            ..default()
                        })
                        .insert(movement)
                        .insert(Enemy(2));
                }
            }
        })
    }
}