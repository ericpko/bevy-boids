mod simulation;

use bevy::{prelude::*, window::PrimaryWindow};
use simulation::SimulationPlugin;

const DT: f32 = 1. / 120.;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            brightness: 0.03,
            ..default()
        })
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .insert_resource(FixedTime::new_from_secs(DT))
        .add_systems(Startup, spawn_camera)
        .add_plugins((DefaultPlugins, SimulationPlugin))
        .run();
}

fn spawn_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.get_single().unwrap();
    // window.resolution.set(350.0, 600.0);

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    });
}
