use std::collections::HashMap;

use bevy::{color::palettes::css::{DARK_GRAY, RED}, prelude::*, text::*};

use crate::{global::{CircleCollider, Velocity, PLAYER_COLOR}, GLOW_FACTOR};


const PLAYER_SPEED: f32 = 10.0;
const FRICTION: f32 = 0.5;
const BOW_OFFSET: f32 = 55.0;
const ARROW_SPEED: f32 = 10.0;
const ARROW_COOLDOWN: f32 = 0.5;

#[derive(Component)]
#[require(Velocity, Mesh2d, MeshMaterial2d<ColorMaterial>)]
pub struct Player;

#[derive(Component)]
pub struct Bow;

#[derive(Component)]
pub struct Arrow;

#[derive(Component, Default, Copy, Clone)]
pub struct PreviousPosition(pub Vec3);

#[derive(Component)]
pub struct PlayerHealth {
    pub current: i32,
    per_segment: i32,
    num_segments: i32
}

#[derive(Component)]
pub struct HealthBarSegment {
    pub index: usize
}

#[derive(Component)]
pub struct PlayerHealthBar;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (handle_keys, apply_friction, update_bow_position, handle_mouse, smooth_camera_follow, update_health_bar_ui));
    }
}

fn handle_keys(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut Transform), With<Player>>
) {
    let mut player = query.single_mut().unwrap();

    if keyboard_input.pressed(KeyCode::KeyA) {
        player.0.dx -= PLAYER_SPEED * time.delta_secs()
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        player.0.dx += PLAYER_SPEED * time.delta_secs()

    }
    if keyboard_input.pressed(KeyCode::KeyW) {
        player.0.dy += PLAYER_SPEED * time.delta_secs()
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        player.0.dy -= PLAYER_SPEED * time.delta_secs()

    }
}

fn apply_friction(
    mut query: Query<&mut Velocity, With<Player>>,
    time: Res<Time>
) {
    let mut player = query.single_mut().unwrap();
    player.dx *= FRICTION.powf(time.delta_secs());
    player.dy *= FRICTION.powf(time.delta_secs());
}

pub fn spawn_player(
    commands:  &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>, 
    assets: &Res<AssetServer>
) {
    let bow = assets.load("bow.png");

    commands.spawn((
        Player,
        Mesh2d(meshes.add(RegularPolygon::new(30.0,3))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(PLAYER_COLOR))),
        PreviousPosition(Vec3::ZERO),
        PlayerHealth {
            current: 36,
            per_segment: 10,
            num_segments: 4
        },
        CircleCollider(30.0)
    ));

    commands.spawn((
        Bow,
        Sprite {
            image: bow,
            color: Color::linear_rgb(GLOW_FACTOR, GLOW_FACTOR, GLOW_FACTOR),
            ..default()
        },
        Transform::from_scale(Vec3::new(0.5, 0.5, 0.5))
    ));

    commands.spawn((
        Node {
            width: Val::Percent(100.0), 
            height: Val::Px(30.0),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            bottom: Val::Px(15.0),
            left: Val::Px(15.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
        PlayerHealthBar
    ))
    .with_children(|parent| {
        // tworzymy 4 segmenty
        for i in 0..4 {
            parent.spawn((
                Node {
                    width: Val::Px(100.0),
                    height: Val::Px(30.0),
                    margin: UiRect {
                        right: Val::Px(15.0),
                        ..default()
                    },
                    ..default()
                },
                // domyślny kolor (pusty)
                BackgroundColor(Color::Srgba(DARK_GRAY)),
                HealthBarSegment { index: i },
            ));
        }
    });

}

fn update_health_bar_ui(
    segments_query: Query<(&HealthBarSegment, Entity)>,
    mut nodes_query: Query<(&mut Node, &mut BackgroundColor), With<HealthBarSegment>>,
    player_query: Query<&PlayerHealth>
) {
    let segments_map: HashMap<usize, Entity> = segments_query
        .iter()
        .map(|(segment, entity)| (segment.index, entity))
        .collect();

    let player = player_query.single().unwrap();

    let current_segment = f32::floor(player.current as f32 / player.per_segment as f32) as i32;

    for i in 0..player.num_segments {
        let (_, mut color) = nodes_query.get_mut(segments_map.get(&(i as usize)).unwrap().clone()).unwrap();
        color.0 = Color::Srgba(DARK_GRAY);
    }

    for i in 0..(current_segment) {
        let (mut current_node, mut color) = nodes_query.get_mut(segments_map.get(&(i as usize)).unwrap().clone()).unwrap();

        current_node.width = Val::Px(100.0);
        color.0 = Color::Srgba(RED);
    }

    if current_segment == player.num_segments {
        return;
    }

    let (mut last_node, mut last_color) = nodes_query.get_mut(segments_map.get(&((current_segment) as usize)).unwrap().clone()).unwrap();

    last_node.width = Val::Px(100.0 * ((player.current as f32 - current_segment as f32 * player.per_segment as f32) / player.per_segment as f32));
    last_color.0 = Color::Srgba(RED);
}

fn update_bow_position(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut bow_query: Query<&mut Transform, (With<Bow>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<Bow>)>
) {
    let mut bow = bow_query.single_mut().unwrap();
    let player = player_query.single().unwrap();

    let window = windows.single().unwrap();

    if let Some(cursor_pos) = window.cursor_position() {
        let (camera, camera_transform) = camera_q.single().unwrap();

        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            let alpha = f32::atan2(world_pos.y - player.translation.y, world_pos.x - player.translation.x);

            let x = player.translation.x + BOW_OFFSET * f32::cos(alpha);
            let y = player.translation.y + BOW_OFFSET * f32::sin(alpha);

            bow.translation.x = x;
            bow.translation.y = y;

            bow.rotation = Quat::from_rotation_z(alpha);
        }
    }
}

fn handle_mouse(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>, 
    assets: Res<AssetServer>,
    player_query: Query<&Transform, (With<Player>, Without<Bow>)>,
    mut cooldown: Local<f32>,
    time: Res<Time>
) {
    let window = windows.single().unwrap();
    let player = player_query.single().unwrap();
    if mouse.pressed(MouseButton::Left) && *cooldown > ARROW_COOLDOWN {
        if let Some(cursor_pos) = window.cursor_position() {
            let (camera, camera_transform) = camera_q.single().unwrap();
    
            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                let alpha = f32::atan2(world_pos.y - player.translation.y, world_pos.x - player.translation.x);

                let arrow = assets.load("arrow.png");

                commands.spawn((
                    Arrow,
                    Sprite {
                        image: arrow,
                        color: Color::linear_rgb(GLOW_FACTOR, GLOW_FACTOR, GLOW_FACTOR),
                        ..default()
                    },
                    Transform {
                        scale: Vec3::new(0.3, 0.3, 0.3),
                        rotation: Quat::from_rotation_z(alpha),
                        translation: Vec3::new(player.translation.x + f32::cos(alpha) * BOW_OFFSET, player.translation.y + f32::sin(alpha) * BOW_OFFSET, 0.0)
                    },
                    Velocity {
                        dx: ARROW_SPEED * f32::cos(alpha),
                        dy: ARROW_SPEED * f32::sin(alpha)
                    }
                ));

                *cooldown = 0.0;
            }
        }
    }
    *cooldown += time.delta_secs();
}

fn smooth_camera_follow(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    if let Ok(player_transform) = player_query.single() {
        if let Ok(mut camera_transform) = camera_query.single_mut() {
            
            let smoothness = 5.0;

            
            let current = camera_transform.translation;

            
            let target = Vec3::new(
                player_transform.translation.x,
                player_transform.translation.y,
                current.z,
            );

            
            camera_transform.translation = current.lerp(target, time.delta_secs() * smoothness);
        }
    }
}