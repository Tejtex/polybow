use bevy::{
    prelude::*

};
use bevy::core_pipeline::bloom::Bloom;
use bevy_hanabi::HanabiPlugin;


use crate::enemy::{EnemyPlugin, HealthBar, HealthBarOwner, HP};
use crate::global::{CircleCollider, ENEMY_COLOR};
use crate::player::spawn_player;
use crate::world::WorldPlugin;
use crate::{enemy::Enemy, global::GlobalPlugin, player::PlayerPlugin};

pub mod player;
pub mod global;
pub mod enemy;
pub mod world;

const GLOW_FACTOR: f32 = 10.0;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, HanabiPlugin))
        .add_systems(Startup, (setup,))
        .add_plugins((PlayerPlugin,GlobalPlugin, EnemyPlugin, WorldPlugin))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>, 
    assets: Res<AssetServer>
) {
    commands.spawn((
        Camera2d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Bloom::NATURAL
    ));
    spawn_player(&mut commands, &mut meshes, &mut materials, &assets);
    
}

