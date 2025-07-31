use std::f32::consts::PI;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_rapier2d::prelude::*;
use rand::prelude::*;

use crate::enemy::{Enemy, HealthBar, HealthBarOwner, HP};
use crate::global::{adjusted_glow, regular_polygon_vertices, CircleCollider};
use crate::player::Player;
use crate::{AppState, FirstPass, ENEMY_COLOR, GLOW_FACTOR};
use crate::planets::{Effect, EffectType, Planet};

#[derive(Resource, Default)]
pub struct PlanetData(pub Vec<(Vec2, f32)>);

#[derive(Resource)]
pub struct EnemiesCounter(pub i32);

const  NUM_COLORS: i32 = 4;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, spawn_enemies.run_if(in_state(AppState::InGame)))
            .add_systems(OnEnter(AppState::InGame), spawn_planets)
            .insert_resource(EnemiesCounter(0))
            .insert_resource(PlanetData::default());
    }
}

fn spawn_enemies(
    mut commands: Commands,
    mut enemies: ResMut<EnemiesCounter>,
    mut cooldown: Local<f32>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_query: Query<&Transform, With<Player>>,
    planet_data: Res<PlanetData>
) {
    *cooldown += time.delta_secs();
    if *cooldown > f32::powf(enemies.0 as f32 / 5.0, 2.0) + 0.1 {
        *cooldown = 0.0;
        let player = player_query.single().unwrap();

        let mut rng = rand::rng();
        let sides = rng.random_range(3..5);
        let mut tries = 0;
        let spawn_pos = loop {
            tries += 1;
            if tries > 100 { return; }

            let angle = rng.random_range(0.0..2.0 * PI);
            let r = rng.random_range(120.0..500.0);
            let pos = Vec2::new(r * angle.cos(), r * angle.sin()) + player.translation.truncate();

            if is_position_safe(pos, &planet_data) {
                break pos;
            }
        };


        let id = commands.spawn((
            Enemy {
                sides
            },
            Mesh2d(meshes.add(RegularPolygon::new(30.0,sides as u32))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(ENEMY_COLOR))),
            HP {
                current: 4.,
                max: 4.
            },
            CircleCollider(30.0),
            Transform::from_translation(spawn_pos.extend(0.)),
            GlobalTransform::default(),
            Collider::convex_hull(&regular_polygon_vertices(30., sides as usize).iter().map(|e| {Vect::from(e.clone())}).collect::<Vec<Vect>>()).unwrap(),
            RigidBody::Dynamic,
            Velocity::angular(2.0 * PI / 3.0),
            Restitution::coefficient(1.0),
            RenderLayers::layer(0),

            FirstPass,
        )).id();

        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(75.0, 15.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::linear_rgb(GLOW_FACTOR, 0.0, 0.0)))),
            Transform {
                translation: (spawn_pos + Vec2::new(0., 40.)).extend(0.1),
                ..default()
            },
            GlobalTransform::default(),
            HealthBar,
            HealthBarOwner(id),
            RenderLayers::layer(0),
            FirstPass,
        ));
        enemies.0 += 1;
    }
}

fn spawn_planets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut planet_data: ResMut<PlanetData>,
) {
    info!("Spawning planets.");
    let mut rng = rand::rng();
    let mut z = 0;
    let mut colours: std::vec::Vec<Color> = std::vec::Vec::new();
    for _i in 0..NUM_COLORS {
        colours.push(Color::srgb(rng.random::<f32>() , rng.random::<f32>() , rng.random::<f32>()));
    }
    'outer: for _ in 0..30 {

        let pos_x =  loop { let x = rng.random_range(-1500.0..1500.0); if (x as f32).abs() > 200. {break x} };
        let pos_y =  loop { let y = rng.random_range(-1500.0..1500.0); if (y as f32).abs() > 200. {break y} };
        let sides = rng.random_range(5..=10);
        let mut points = Vec::new();

        let angle_offset = rng.random_range(0.0..std::f32::consts::TAU);
        let mut max_radius = 0.;

        for i in 0..sides {
            let angle = angle_offset + i as f32 / sides as f32 * std::f32::consts::TAU;
            let radius = rng.random_range(50.0..100.0);
            if radius > max_radius {
                max_radius = radius;
            }
            let mut x = radius * angle.cos();
            let mut y = radius * angle.sin();
            let mut j = 0;
            while planet_data.0.iter().any(|e| {
                Vec2::new(x + pos_x, y + pos_y).distance(e.0) < e.1 + max_radius
            }) {
                let angle = angle_offset + i as f32 / sides as f32 * std::f32::consts::TAU;
                let radius = rng.random_range(50.0..100.0);
                if radius > max_radius {
                    max_radius = radius;
                }
                x = radius * angle.cos();
                y = radius * angle.sin();
                j += 1;
                if j > 100 {
                    continue 'outer;
                }
            }
            points.push([x, y, 0.0]);
        }

        let mut indices: Vec<u32> = Vec::new();
        for i in 1..(sides - 1) {
            indices.push(0);
            indices.push(i);
            indices.push(i + 1);
        }

        let positions = points.clone();
        let uvs = vec![[0.5, 0.5]; sides as usize];
        let normals = vec![[0.0, 0.0, 1.0]; sides as usize];

        let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

        planet_data.0.push((Vec2::new(pos_x, pos_y), max_radius));

        let collider_vertices: Vec<Vec2> = points.iter().map(|f| Vec2::new(f[0], f[1])).collect();
        let collider_indices: Vec<[u32; 2]> = (0..sides as u32)
            .map(|i| [i, (i + 1) % sides as u32])
            .collect();
        let color = adjusted_glow(colours[rng.random_range(0..NUM_COLORS as usize)], GLOW_FACTOR);

        commands.spawn( (
            Planet {
                color,
                effect: Effect { effect_type: EffectType::Poison, level: 2 },
                hp: rng.random_range(1.0..12.0),
            },
            Mesh2d(meshes.add(mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_translation(Vec3::new(pos_x, pos_y, 0.)).with_scale(Vec3::splat(2.)),
            RigidBody::Dynamic,
            Collider::polyline(collider_vertices, Some(collider_indices)),
            Restitution::coefficient(1.0),
            ActiveEvents::COLLISION_EVENTS,

            RenderLayers::layer(0),
            FirstPass,


        ));
        z += 1;

    }
    info!("Spawned {} planets.", z);
}

fn is_position_safe(pos: Vec2, planets: &PlanetData) -> bool {
    for (planet_pos, planet_radius) in &planets.0 {
        if pos.distance(*planet_pos) < *planet_radius + 30.0 {
            return false;
        }
    }
    true
}
