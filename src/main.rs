//! Renders a 2D scene containing a single, moving sprite.

use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, sprite::Anchor};
use getrandom::getrandom;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Start,
    InGame,
    GameOver,
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
        .add_state(AppState::Start)
        .add_startup_system(setup)
        .add_system_set(SystemSet::on_enter(AppState::Start).with_system(setup_start))
        .add_system_set(
            SystemSet::on_update(AppState::Start)
                .with_system(update_start)
                .with_system(update_player_name),
        )
        .add_system_set(SystemSet::on_exit(AppState::Start).with_system(teardown_all))
        .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup_game))
        .add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(sprite_movement)
                .with_system(move_block.before(sprite_movement))
                .with_system(enemy_spawner.after(move_block))
                .with_system(warp_system.after(enemy_spawner))
                .with_system(bullet_hits.after(warp_system))
                .with_system(player_hits.after(bullet_hits))
                .with_system(cleanup.after(player_hits)),
        )
        .add_system_set(SystemSet::on_exit(AppState::InGame).with_system(teardown_all))
        .add_system_set(SystemSet::on_enter(AppState::GameOver).with_system(setup_game_over))
        .add_system_set(SystemSet::on_update(AppState::GameOver).with_system(update_start))
        .add_system_set(SystemSet::on_exit(AppState::GameOver).with_system(teardown_all))
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

#[derive(Component)]
struct Player(Timer, Timer);

#[derive(Component)]
struct Nowarp;

#[derive(Component)]
struct Enemy(u32);

impl Enemy {
    fn get_asset_path(health: u32) -> &'static str {
        match health {
            2 => "enemy-big.png",
            1 => "enemy-medium.png",
            0 => "enemy-small.png",
            _ => "",
        }
    }
}

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct Background;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct PlayerName;

#[derive(Hash)]
struct PlayerNameText(String);

struct Score(u64);

impl Score {
    fn reset(&mut self) {
        self.0 = 0;
    }

    fn add(&mut self, points: u64) {
        self.0 += points;
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
    commands.insert_resource(Score(0))
}

fn setup_start(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                },
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|builder| {
            builder
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                        // center button
                        margin: UiRect::all(Val::Auto),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    color: Color::AQUAMARINE.into(),
                    ..default()
                })
                .with_children(|child| {
                    child.spawn_bundle(TextBundle::from_section(
                        "Start!",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::DARK_GRAY.into(),
                            font: asset_server.load("FiraSans-Bold.ttf"),
                        },
                    ));
                });
        })
        .with_children(|builder| {
            builder
                .spawn_bundle(TextBundle::from_sections(vec![
                    TextSection::new(
                        "Enter yout name:\n",
                        TextStyle {
                            font_size: 36.0,
                            color: Color::DARK_GRAY.into(),
                            font: asset_server.load("FiraSans-Bold.ttf"),
                        },
                    ),
                    TextSection::new(
                        "",
                        TextStyle {
                            font_size: 36.0,
                            color: Color::DARK_GRAY.into(),
                            font: asset_server.load("FiraSans-Bold.ttf"),
                        },
                    ),
                ]))
                .insert(PlayerName);
        });
}

fn update_start(
    mut commands: Commands,
    mut state: ResMut<State<AppState>>,
    interaction: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    player_name_queue: Query<&Text, With<PlayerName>>,
) {
    for int in &interaction {
        match *int {
            Interaction::Clicked => {
                if !player_name_queue.is_empty() {
                    commands.insert_resource(PlayerNameText(
                        player_name_queue.single().sections[1].value.clone(),
                    ));
                }
                state.set(AppState::InGame).unwrap();
            }
            _ => (),
        }
    }
}

fn update_player_name(
    mut char_input_events: EventReader<ReceivedCharacter>,
    keyboard_input: Res<Input<KeyCode>>,
    mut player_name_queue: Query<&mut Text, With<PlayerName>>,
) {
    let player_text = &mut player_name_queue.single_mut().sections[1].value;

    if keyboard_input.just_pressed(KeyCode::Back) {
        player_text.pop();
    }

    for ev in char_input_events.iter() {
        if player_text.len() < 16 {
            if ev.char != '\n'
                && ev.char != '\r'
                && !ev.char.is_whitespace()
                && ev.char.is_alphanumeric()
            {
                player_text.push(ev.char);
            }
        }
    }
}

fn setup_game_over(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    score: Res<Score>,
    name: Res<PlayerNameText>,
) {
    // Publish scores if not empty name
    if !name.0.is_empty() {
        post_score(name.0.clone(), score.0);
    }

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                },
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|builder| {
            builder
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(200.0), Val::Px(65.0)),
                        // center button
                        margin: UiRect::all(Val::Auto),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    color: Color::AQUAMARINE.into(),
                    ..default()
                })
                .with_children(|child| {
                    child.spawn_bundle(TextBundle::from_section(
                        "Play again!",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::DARK_GRAY.into(),
                            font: asset_server.load("FiraSans-Bold.ttf"),
                        },
                    ));
                });
        })
        .with_children(|builder| {
            let mut leaderboard = String::default();
            get_leaderboard()
                .into_iter()
                .for_each(|LeaderRow { player, score }| {
                    leaderboard.push_str(format!("\n{}: {}", player.as_str(), score).as_str());
                });
            builder.spawn_bundle(
                // Create a TextBundle that has a Text with a single section.
                TextBundle::from_section(
                    // Accepts a `String` or any type that converts into a `String`, such as `&str`
                    format!("Leaderboard: {}", leaderboard.as_str()),
                    TextStyle {
                        font: asset_server.load("FiraSans-Bold.ttf"),
                        font_size: 28.0,
                        color: Color::GOLD,
                    },
                ) // Set the alignment of the Text
                .with_text_alignment(TextAlignment::TOP_CENTER)
                // Set the style of the TextBundle itself.
                .with_style(Style {
                    justify_content: JustifyContent::Center,
                    ..default()
                }),
            );
        })
        .with_children(|builder| {
            let fscore_text = if name.0.len() == 0 {
                format!("Your final score: {}", score.0)
            } else {
                format!("{}, your final score: {}", name.0, score.0)
            };
            builder.spawn_bundle(
                TextBundle::from_section(
                    // Accepts a `String` or any type that converts into a `String`, such as `&str`
                    fscore_text,
                    TextStyle {
                        font: asset_server.load("FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::WHITE,
                    },
                ) // Set the alignment of the Text
                .with_text_alignment(TextAlignment::TOP_CENTER)
                // Set the style of the TextBundle itself.
                .with_style(Style {
                    position: UiRect {
                        top: Val::Px(5.0),
                        right: Val::Px(15.0),
                        ..default()
                    },
                    ..default()
                }),
            );
        });
}

fn teardown_all(mut commands: Commands, query: Query<Entity, Without<Camera2d>>) {
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
}

fn spawn_new_enemy(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    player_pos: &Transform,
) {
    let mut movement = Movment::default();
    let distance = 200. + (get_rand01() * ((W_HEIGHT as f64).min(W_WIDTH as f64) - 400.));
    let new_displaysment = Transform::from_translation(
        (Vec2::from_angle((get_rand01() * 2. * PI as f64) as f32)
            .rotate(Vec2::new(distance as f32, 0.)))
        .extend(0.),
    );
    let mut new_transform = player_pos.clone().mul_transform(new_displaysment);
    if new_transform.translation.x > W_WIDTH / 2. {
        new_transform.translation.x -= W_WIDTH;
    }
    if new_transform.translation.x < -W_WIDTH / 2. {
        new_transform.translation.x += W_WIDTH;
    }
    if new_transform.translation.y > W_HEIGHT / 2. {
        new_transform.translation.y -= W_HEIGHT;
    }
    if new_transform.translation.y < -W_HEIGHT / 2. {
        new_transform.translation.y += W_HEIGHT;
    }
    movement.fill_rand();
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load(Enemy::get_asset_path(2)),
            sprite: Sprite {
                color: Color::GOLD,
                custom_size: Some(Vec2::new(35.0, 50.0)),
                ..default()
            },
            transform: new_transform,
            ..default()
        })
        .insert(movement)
        .insert(Enemy(2));
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>, mut score: ResMut<Score>) {
    score.reset();
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("bg.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(W_WIDTH, W_HEIGHT)),
                ..default()
            },
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        })
        .insert(Background);
    commands.spawn_bundle(
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            format!("Score: {}", score.0),
            TextStyle {
                font: asset_server.load("FiraSans-Bold.ttf"),
                font_size: 40.0,
                color: Color::WHITE,
            },
        ) // Set the alignment of the Text
        .with_text_alignment(TextAlignment::TOP_CENTER)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                right: Val::Px(15.0),
                ..default()
            },
            ..default()
        }),
    );
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("patron.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(42.0, 75.0)),
                anchor: Anchor::TopCenter,
                ..default()
            },
            transform: Transform::from_xyz(0., 0., 1.),
            ..default()
        })
        .insert(Movment::default())
        .insert(Player(
            Timer::from_seconds(0.2, false)
                .tick(Duration::from_secs_f32(0.2))
                .to_owned(),
            Timer::from_seconds(10., false),
        ));

    for _i in 0..2 {
        spawn_new_enemy(
            &mut commands,
            &asset_server,
            &Transform::from_xyz(0., 0., 1.),
        );
    }
}

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn sprite_movement(time: Res<Time>, mut sprite_position: Query<(&Movment, &mut Transform)>) {
    for (move_params, mut transform) in &mut sprite_position {
        let delta = Transform::from_xyz(0., move_params.speed * time.delta_seconds(), 0.);
        transform.rotate_local_z(-move_params.look_direction + move_params.heading);
        *transform = transform
            .mul_transform(delta)
            .with_rotation(Quat::from_rotation_z(move_params.heading));
        transform.rotate_local_z(move_params.look_direction - move_params.heading);
    }
}

fn move_block(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut query: Query<(&mut Movment, &Transform, &mut Player), With<Player>>,
) {
    if let Ok((mut block, trans, mut pl)) = query.get_single_mut() {
        pl.0.tick(time.delta());

        if keyboard_input.pressed(KeyCode::W) {
            let delta_speed = 250. * time.delta_seconds();
            let new_speed = (((delta_speed * block.look_direction.sin())
                + (block.speed * block.heading.sin()))
            .powi(2)
                + ((delta_speed * block.look_direction.cos())
                    + (block.speed * block.heading.cos()))
                .powi(2))
            .sqrt();
            let new_heading = ((delta_speed * block.look_direction.sin())
                + (block.speed * block.heading.sin()))
            .atan2(
                (delta_speed * block.look_direction.cos()) + (block.speed * block.heading.cos()),
            );

            block.speed = new_speed;
            block.heading = new_heading;

            if block.speed > 500. {
                block.speed = 500.
            }
        } else {
            block.speed -= 125. * time.delta_seconds();
            if block.speed < 0. {
                block.speed = 0.;
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
                commands
                    .spawn_bundle(SpriteBundle {
                        texture: asset_server.load("bullet.png"),
                        sprite: Sprite {
                            color: Color::WHITE,
                            custom_size: Some(Vec2::new(5.0, 17.0)),
                            ..default()
                        },
                        transform: trans.mul_transform(Transform::from_translation(trans.up())),
                        ..default()
                    })
                    .insert(Movment {
                        speed: 700.,
                        heading: block.look_direction,
                        look_direction: block.look_direction,
                    })
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
            b.translation.y = -(b.translation.y - (5. * b.translation.y.signum()))
        }
    }
}

fn cleanup(mut commands: Commands, query: Query<(Entity, &Transform), With<Nowarp>>) {
    for (e, b) in &query {
        match (b.translation.x.abs(), b.translation.y.abs()) {
            (a, _) if a > W_WIDTH / 2. => {
                commands.entity(e).despawn();
            }
            (_, b) if b > W_HEIGHT / 2. => {
                commands.entity(e).despawn();
            }
            _ => (),
        }
    }
}

fn enemy_spawner(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    query: Query<&Transform, With<Player>>,
    mut player: Query<&mut Player, With<Player>>,
) {
    if let Ok(mut pl) = player.get_single_mut() {
        let spawn_timer = &mut pl.1;
        spawn_timer.tick(time.delta());

        if spawn_timer.finished() {
            spawn_new_enemy(&mut commands, &asset_server, query.single());
            spawn_timer.reset();
        }
    }
}

fn check_colision(
    t1: &Transform,
    w1: f32,
    h1: f32,
    r1: f32,
    t2: &Transform,
    w2: f32,
    h2: f32,
    r2: f32,
) -> bool {
    let mut t1c = t1.clone();
    let mut t2c = t2.clone();

    t2c.translation -= t1.translation;

    for (x, y) in [(0.5, 0.5), (-0.5, 0.5), (0.5, -0.5), (-0.5, -0.5)] {
        let p = Vec2::from_angle(r2 - r1).rotate(Vec2 {
            x: w2 * x + t2c.translation.x,
            y: h2 * y + t2c.translation.y,
        });
        if p.abs()
            .cmple(Vec2 {
                x: w1 / 2.,
                y: h1 / 2.,
            })
            .all()
        {
            return true;
        }
    }

    t1c.translation -= t2.translation;

    for (x, y) in [(0.5, 0.5), (-0.5, 0.5), (0.5, -0.5), (-0.5, -0.5)] {
        let p = Vec2::from_angle(r1 - r2).rotate(Vec2 {
            x: w1 * x + t1c.translation.x,
            y: h1 * y + t1c.translation.y,
        });
        if p.abs()
            .cmple(Vec2 {
                x: w2 / 2.,
                y: h2 / 2.,
            })
            .all()
        {
            return true;
        }
    }

    false
}

fn bullet_hits(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    bullets: Query<(Entity, &Transform, &Movment), With<Bullet>>,
    enemies: Query<(Entity, &Transform, &Movment, &Enemy), With<Enemy>>,
    mut text: Query<&mut Text>,
    mut score: ResMut<Score>,
) {
    bullets.for_each(|(be, bt, bm)| {
        enemies.for_each(|(ee, et, em, e)| {
            if et.translation.distance(bt.translation) < (17. + 50.) / 2. {
                if check_colision(
                    bt,
                    5.,
                    17.,
                    bm.look_direction,
                    et,
                    35.,
                    50.,
                    em.look_direction,
                ) {
                    commands.entity(be).despawn();
                    let lifes = e.0;
                    commands.entity(ee).despawn();
                    score.add((4 - lifes) as u64 * 50);
                    text.get_single_mut().unwrap().sections[0].value =
                        format!("Score: {}", score.0);

                    if lifes > 0 {
                        for _i in 0..2 {
                            // let mut rng = rand::rngs::OsRng;
                            let mut movement = Movment::default();
                            movement.fill_rand();
                            // movement.try_fill(&mut rng).unwrap();
                            commands
                                .spawn_bundle(SpriteBundle {
                                    texture: asset_server.load(Enemy::get_asset_path(lifes - 1)),
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
            }
        })
    })
}

fn player_hits(
    player: Query<(&Transform, &Movment), With<Player>>,
    enemies: Query<(&Transform, &Movment), With<Enemy>>,
    mut state: ResMut<State<AppState>>,
) {
    if let Ok((pt, pm)) = player.get_single() {
        for (et, em) in enemies.into_iter() {
            let mut p_centr = pt.clone();
            p_centr = p_centr
                .mul_transform(Transform::from_xyz(0., -75. / 2., 0.))
                .with_rotation(Quat::from_rotation_z(pm.look_direction));
            if et.translation.distance(p_centr.translation) < (75. + 50.) / 2. {
                if check_colision(
                    &p_centr,
                    42.,
                    75.,
                    pm.look_direction,
                    et,
                    35.,
                    50.,
                    em.look_direction,
                ) {
                    if state.set(AppState::GameOver).is_ok() {
                        return;
                    }
                }
            }
        }
    }
}

#[wasm_bindgen(module = "/scores.js")]
extern "C" {
    fn get_scores() -> String;
    fn post_score(name: String, score: u64);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log_str(s: &str);
}

#[derive(Serialize, Deserialize)]
struct LeaderRow {
    player: String,
    score: u64,
}

fn get_leaderboard() -> Vec<LeaderRow> {
    serde_json::from_str(get_scores().as_str()).unwrap()
}
