// https://vanhunteradams.com/Pico/Animal_Movement/Boids-algorithm.html
// #![allow(unused)]

use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};

// Constants
const NUM_BOIDS: u32 = 300;
const BOID_SIZE: f32 = 32.0; // number of pixels
const TURN_FACTOR: f32 = 0.2;
const VISUAL_RANGE: f32 = 16.0; // pixels
const PROJECTED_RANGE: f32 = 9.0; // pixels
const CENTERING_FACTOR: f32 = 0.0005;
const AVOID_FACTOR: f32 = 0.05;
const MATCHING_FACTOR: f32 = 0.05;
const MAX_SPEED: f32 = 6.0; // pixels per second
const MIN_SPEED: f32 = 3.0;
// const MAX_BIAS: f32 = 0.01;
// const BIAS_INCREMENT: f32 = 0.00004;
const BIAS_VAL: f32 = 0.001;
const MARGIN: f32 = 100.0;

pub struct SimulationPlugin;
impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_boids)
            .add_systems(
                Update,
                (
                    separation,
                    alignment,
                    cohesion,
                    wall_collision,
                    bias,
                    limit_speed,
                    // update_position,
                ),
            )
            .add_systems(FixedUpdate, update_position);
    }
}

#[derive(Component, Default)]
struct Boid {
    scout_group: u32,
}

#[derive(Component, Default)]
struct Velocity(Vec2);

#[derive(Bundle, Default)]
struct BoidBundle {
    boid: Boid,
    sprite: SpriteBundle,
    vel: Velocity,
}

fn spawn_boids(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();
    let texture = asset_server.load("sprites/ship_B.png");

    for _ in 0..NUM_BOIDS {
        let x = (BOID_SIZE / 2.0) + rand::random::<f32>() * (window.width() - BOID_SIZE);
        let y = (BOID_SIZE / 2.0) + rand::random::<f32>() * (window.height() - BOID_SIZE);

        // assign this boid a scout group
        let mut scout_group = 0u32;
        if rand::random::<f32>() < 0.12 {
            scout_group = if rand::random::<f32>() < 0.5 { 1 } else { 2 };
        }

        let _entity = commands
            .spawn(BoidBundle {
                sprite: SpriteBundle {
                    transform: Transform::from_xyz(x, y, 0.0)
                        .with_scale(Vec3::new(0.25, 0.25, 1.0)),
                    texture: texture.clone(),
                    ..default()
                },
                vel: Velocity(Vec2::new(
                    // (rand::random::<f32>() - 0.5) * MAX_SPEED,
                    // (rand::random::<f32>() - 0.5) * MAX_SPEED,
                    0.0, 0.0,
                )),
                boid: Boid { scout_group },
                // ..default(),
            })
            .id();
    }
}

fn separation(mut boids: Query<(Entity, &Transform, &mut Velocity), With<Boid>>) {
    // let mut boids_list = boids
    //     .iter_mut()
    //     .collect::<Vec<(Entity, &Transform, Mut<Velocity>)>>();
    // for (e_i, t_i, v_i) in boids_list {
    //     // stuff
    // }
    let mut combinations = boids.iter_combinations_mut();
    while let Some([(_, t_i, mut v_i), (_, t_j, _)]) = combinations.fetch_next() {
        let r_i = Vec2::new(t_i.translation.x, t_i.translation.y);
        let r_j = Vec2::new(t_j.translation.x, t_j.translation.y);

        let delta = r_j.distance(r_i);
        if delta < PROJECTED_RANGE {
            v_i.0 += delta * AVOID_FACTOR;
        }
    }
}

fn alignment(mut boids: Query<(Entity, &Transform, &mut Velocity), With<Boid>>) {
    let mut neighbors_map: HashMap<Entity, Vec2> = HashMap::with_capacity(NUM_BOIDS as usize);

    for (e_i, t_i, _) in &boids {
        let mut neighboring_boids = 0;
        let mut vel_avg = Vec2::ZERO;

        let r_i = Vec2::new(t_i.translation.x, t_i.translation.y);

        for (e_j, t_j, v_j) in &boids {
            if e_i == e_j {
                continue;
            }
            let r_j = Vec2::new(t_j.translation.x, t_j.translation.y);
            let u_j = v_j.0;

            let delta = r_j.distance(r_i);
            if delta < VISUAL_RANGE {
                neighboring_boids += 1;
                vel_avg += u_j;
            }
        }
        if neighboring_boids > 0 {
            vel_avg /= neighboring_boids as f32;
        }
        neighbors_map.entry(e_i).or_insert(vel_avg);
    }
    // update the velocity
    for (&e_i, vel_avg) in neighbors_map.iter() {
        let Ok((_, _, mut v_i)) = boids.get_mut(e_i) else {
            panic!();
        };
        let u_i = v_i.0;
        v_i.0 += (*vel_avg - u_i) * MATCHING_FACTOR;
    }

    // create a neighbors map where the <key> is an entity and the value is a tuple of num_neighbors, velocity sum, velocity average
    // let mut neighbors: HashMap<Entity, (u32, Vec2, Vec2)> =
    //     HashMap::with_capacity(NUM_BOIDS as usize);

    // let mut combinations = boids.iter_combinations_mut();
    // while let Some([(e_i, t_i, _), (_, t_j, v_j)]) = combinations.fetch_next() {
    //     // get the hash map values from this entity, otherwise, if it doesn't exist, create it an initialize it
    //     let (mut num_neighbors, mut vel_sum, mut _vel_avg) =
    //         neighbors
    //             .entry(e_i)
    //             .or_insert((0, Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)));

    //     let u_j = v_j.0;
    //     let r_i = Vec2::new(t_i.translation.x, t_i.translation.y);
    //     let r_j = Vec2::new(t_j.translation.x, t_j.translation.y);
    //     let delta = r_j.distance(r_i);

    //     if delta < VISUAL_RANGE {
    //         num_neighbors += 1;
    //         vel_sum += u_j;
    //         _vel_avg = vel_sum / (num_neighbors as f32);
    //     }
    // }

    // // update the velocity
    // for (&e_i, (_, _, vel_avg)) in neighbors.iter() {
    //     let Ok((_, _, mut v_i)) = boids.get_mut(e_i) else {
    //         panic!();
    //     };
    //     let u_i = v_i.0;
    //     v_i.0 += (*vel_avg - u_i) * MATCHING_FACTOR;
    // }
}

fn cohesion(mut boids: Query<(Entity, &Transform, &mut Velocity), With<Boid>>) {
    let mut neighbors_map: HashMap<Entity, Vec2> = HashMap::with_capacity(NUM_BOIDS as usize);

    for (e_i, t_i, _) in &boids {
        let mut neighboring_boids = 0;
        let mut pos_avg = Vec2::ZERO;

        let r_i = Vec2::new(t_i.translation.x, t_i.translation.y);

        for (e_j, t_j, _) in &boids {
            if e_i == e_j {
                continue;
            }
            let r_j = Vec2::new(t_j.translation.x, t_j.translation.y);

            let delta = r_j.distance(r_i);
            if delta < VISUAL_RANGE {
                neighboring_boids += 1;
                pos_avg += r_j;
            }
        }
        if neighboring_boids > 0 {
            pos_avg /= neighboring_boids as f32;
        }
        neighbors_map.entry(e_i).or_insert(pos_avg);
    }
    // update the velocity
    for (&e_i, pos_avg) in neighbors_map.iter() {
        let Ok((_, t_i, mut v_i)) = boids.get_mut(e_i) else {
            panic!();
        };
        let r_i = Vec2::new(t_i.translation.x, t_i.translation.y);
        v_i.0 += (*pos_avg - r_i) * CENTERING_FACTOR;
    }
}

fn wall_collision(
    mut boids: Query<(Entity, &Transform, &mut Velocity), With<Boid>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();
    // (0, 0) is the bottom left bottom corner of the window, center is (w/2, h/2)

    let left = (BOID_SIZE / 2.0) + 0.0 * (window.width() - BOID_SIZE) + MARGIN;
    let right = (BOID_SIZE / 2.0) + 1.0 * (window.width() - BOID_SIZE) - MARGIN;
    let bottom = (BOID_SIZE / 2.0) + 0.0 * (window.height() - BOID_SIZE) + MARGIN;
    let top = (BOID_SIZE / 2.0) + 1.0 * (window.height() - BOID_SIZE) - MARGIN;

    for (_, t_i, mut v_i) in &mut boids {
        let r_i = Vec2::new(t_i.translation.x, t_i.translation.y);
        let mut u_i = v_i.0;

        if r_i.x < left {
            u_i.x += TURN_FACTOR;
        } else if r_i.x > right {
            u_i.x -= TURN_FACTOR;
        }

        if r_i.y < bottom {
            u_i.y += TURN_FACTOR;
        } else if r_i.y > top {
            u_i.y -= TURN_FACTOR;
        }
        v_i.0 = u_i;
    }
}

fn bias(mut boids: Query<(&mut Velocity, &Boid), With<Boid>>) {
    for (mut v_i, b_i) in &mut boids {
        if b_i.scout_group == 1 {
            v_i.0.x = (1.0 - BIAS_VAL) * v_i.0.x + (BIAS_VAL * 1.0);
        } else if b_i.scout_group == 2 {
            v_i.0.x = (1.0 - BIAS_VAL) * v_i.0.x + (BIAS_VAL * -1.0);
        }
    }
}

fn limit_speed(mut boids: Query<(Entity, &Transform, &mut Velocity), With<Boid>>) {
    for (_, _, mut v_i) in &mut boids {
        let mut u_i = v_i.0;
        let speed = u_i.length();

        if speed > MAX_SPEED {
            u_i = (u_i / speed) * MAX_SPEED;
        } else if speed < MIN_SPEED {
            u_i = (u_i / speed) * MIN_SPEED;
        }
        v_i.0 = u_i;
    }
}

fn update_position(
    mut boids: Query<(Entity, &mut Transform, &Velocity), With<Boid>>,
    // fixed_time: Res<FixedTime>,
) {
    for (_, mut t_i, v_i) in &mut boids {
        let mut r_i = Vec2::new(t_i.translation.x, t_i.translation.y);
        let u_i = v_i.0;

        r_i += u_i;

        t_i.translation.x = r_i.x;
        t_i.translation.y = r_i.y;
    }
}
