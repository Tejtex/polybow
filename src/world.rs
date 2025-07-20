use bevy::prelude::*;
use rand::prelude::*;

use crate::enemy::{Enemy, HealthBar, HealthBarOwner, HP};
use crate::global::CircleCollider;
use crate::player::Player;
use crate::{ENEMY_COLOR, GLOW_FACTOR};

#[derive(Resource)]
pub struct EnemiesCounter(pub i32);

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, spawn_enemies)
            .insert_resource(EnemiesCounter(0));
    }
}

fn spawn_enemies(
    mut commands: Commands,
    mut enemies: ResMut<EnemiesCounter>,
    mut cooldown: Local<f32>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_query: Query<&Transform, With<Player>>
) {
    *cooldown += time.delta_secs();
    if *cooldown > f32::powf(enemies.0 as f32 / 5.0, 2.0) + 0.1 {
        *cooldown = 0.0;
        let player = player_query.single().unwrap();

        let mut rng = rand::rng();

        let angle: f32 = rng.random_range(0.0..355.0);
        let r: f32 = rng.random_range(120.0..500.0);
        let sides = rng.random_range(3..5);

        let id = commands.spawn((
            Enemy {
                sides
            },
            Mesh2d(meshes.add(RegularPolygon::new(30.0,sides as u32))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(ENEMY_COLOR))),
            HP { 
                current: 4,
                max: 4
            },
            CircleCollider(30.0),
            Transform::from_translation(Vec3::new(r * angle.cos() + player.translation.x, r * angle.sin() +player.translation.y, 0.0)),
            GlobalTransform::default(),
        )).id();
    
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(75.0, 15.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::linear_rgb(GLOW_FACTOR, 0.0, 0.0)))),
            Transform {
                translation: Vec3::new(r * angle.cos() + player.translation.x, r * angle.sin() + 40.0 + player.translation.y, 0.1),
                ..default()
            },
            GlobalTransform::default(),
            HealthBar,
            HealthBarOwner(id)
        ));
        enemies.0 += 1;
    }
}