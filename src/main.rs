use bevy::{
    prelude::*

};
use bevy::core_pipeline::bloom::Bloom;
use bevy_hanabi::prelude::*;


use crate::enemy::EnemyPlugin;
use crate::global::ENEMY_COLOR;
use crate::particles::ParticlePlugin;
use crate::player::spawn_player;
use crate::world::WorldPlugin;
use crate::{global::GlobalPlugin, player::PlayerPlugin};

pub mod player;
pub mod global;
pub mod enemy;
pub mod world;
pub mod particles;

const GLOW_FACTOR: f32 = 10.0;


fn main() {
    App::new()
        .add_plugins((DefaultPlugins, HanabiPlugin))
        .add_systems(Startup, setup)
        .add_plugins((PlayerPlugin,GlobalPlugin, EnemyPlugin, WorldPlugin, ParticlePlugin))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>, 
    assets: Res<AssetServer>
) {
    commands.spawn((
        Name::new("CameraParent"),
        Transform::default(),
        GlobalTransform::default()
    )).with_children(|parent| {
        parent.spawn((
            Camera2d::default(),
            Camera {
                hdr: true,
                ..default()
            },
            Bloom::NATURAL
        ));
    });
    spawn_player(&mut commands, &mut meshes, &mut materials, &assets);
    
}

