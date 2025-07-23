use std::collections::HashMap;

use bevy::{color::palettes::css::{DARK_GRAY, RED, YELLOW}, prelude::*};
use bevy_hanabi::ParticleEffect;

use crate::{global::{CircleCollider, Velocity, PLAYER_COLOR}, particles::ParticleHandles, GLOW_FACTOR};


const PLAYER_SPEED: f32 = 10.0;
const FRICTION: f32 = 0.5;
const BOW_OFFSET: f32 = 55.0;
const ARROW_SPEED: f32 = 10.0;
const ARROW_COOLDOWN: f32 = 0.5;
const REGENERATE_SPEED: f32 = 1.0;
const REGENERATE_COOLDOWN: f32 = 3.0;
const G: f32 =50000.0;

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
    pub current: f32,
    per_segment: i32,
    num_segments: i32
}

#[derive(Component)]
pub struct HealthBarSegment {
    pub index: usize
}

#[derive(Component)]
pub struct PlayerHealthBar;

#[derive(Component)]
pub struct XPBar {
    level: i32,
    current: f32
}

#[derive(Component)]
pub struct XPOrb(pub f32);

#[derive(Resource)]
pub struct LastDamageTime(pub f32);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (xp_orb_collision, move_xp_orb,update_xp_bar, handle_keys, apply_friction, update_bow_position, handle_mouse, smooth_camera_follow, update_health_bar_ui, regenerate_healthbar))
            .insert_resource(LastDamageTime(0.0));
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
            current: 36.0,
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

    commands.spawn((
        Node {
            width: Val::Px(600.0),
            height: Val::Px(30.0),
            margin: UiRect {
                top: Val::Px(15.0),
                left: Val::Auto,
                right: Val::Auto,
                ..default()
            },
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::Srgba(YELLOW)),
        XPBar {
            level: 0,
            current: 0.0
        }
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("0"),
            Node {
                top: Val::Px(50.0),
                height: Val::Px(100.0),
                ..default()
            },
            TextFont {
                font: assets.load("Kenneymini.ttf"),
                font_size: 60.0,
                ..default()
            }

        ));
    });

}

fn update_health_bar_ui(
    segments_query: Query<(&HealthBarSegment, Entity)>,
    mut nodes_query: Query<(&mut Node, &mut BackgroundColor, Entity), With<HealthBarSegment>>,
    mut player_query: Query<&mut PlayerHealth>,
    mut commands: Commands
) {
    let segments_map: HashMap<usize, Entity> = segments_query
        .iter()
        .map(|(segment, entity)| (segment.index, entity))
        .collect();

    let mut player = player_query.single_mut().unwrap();

    let current_segment = f32::floor(player.current / player.per_segment as f32) as i32;
    if current_segment + 1 < player.num_segments {

        for i in (current_segment + 1)..player.num_segments {
            let (_, _, ent) = nodes_query.get_mut(segments_map.get(&(i as usize)).unwrap().clone()).unwrap();

            
            commands.entity(ent).despawn();
            player.num_segments -= 1;
        }
    }

    for i in 0..(current_segment) {
        let (mut current_node, mut color, _) = nodes_query.get_mut(segments_map.get(&(i as usize)).unwrap().clone()).unwrap();

        current_node.width = Val::Px(100.0);
        color.0 = Color::Srgba(RED);
    }

    if current_segment == player.num_segments {
        return;
    }

    let (mut last_node, mut last_color, _) = nodes_query.get_mut(segments_map.get(&((current_segment) as usize)).unwrap().clone()).unwrap();

    last_node.width = Val::Px(100.0 * ((player.current - current_segment as f32 * player.per_segment as f32) / player.per_segment as f32));
    last_color.0 = Color::Srgba(RED);
}

fn regenerate_healthbar(
    last_damage: Res<LastDamageTime>,
    time: Res<Time>,
    mut health_query: Query<&mut PlayerHealth>
) {
    let mut health = health_query.single_mut().unwrap();
    if last_damage.0 + REGENERATE_COOLDOWN < time.elapsed_secs() && health.current + time.delta_secs() * REGENERATE_SPEED < (health.num_segments * health.per_segment) as f32 {
        health.current += time.delta_secs() * REGENERATE_SPEED;
    }

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
    time: Res<Time>,
    particle_handles: Res<ParticleHandles>
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
                    },
                    ParticleEffect::new(particle_handles.arrow_trail.clone()),

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
    mut camera_query: Query<&mut Transform, (With<Name>, Without<Player>)>,
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

fn update_xp_bar(
    mut xp_query: Query<(&mut XPBar, &mut Node, &Children)>,
    mut text_query: Query<&mut Text>
) {
    let (mut xp_bar, mut node, children) = xp_query.single_mut().unwrap();
    let per_level = (f32::exp2(xp_bar.level as f32) * 50.);
    if xp_bar.current > per_level {
        xp_bar.level += 1;
        xp_bar.current = 0.;
    }

    for &child in children {
        if let Ok(mut text) = text_query.get_mut(child) {
            text.0 = xp_bar.level.to_string();
        }
    }

    node.width = Val::Px(600. * (xp_bar.current / per_level));


}

fn move_xp_orb(
    xp_orb_query: Query<&mut Transform, (With<XPOrb>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,
    time: Res<Time>
) {
    let player = player_query.single().unwrap();
    for mut orb in xp_orb_query {
        let d = player.translation.truncate().distance(orb.translation.truncate());

        let dir = (player.translation.truncate() - orb.translation.truncate()).normalize();
        let min_force = 50.0;
        let force_magnitude = (G / (d.powf(1.2))).max(min_force);

        let vel = dir * force_magnitude * time.delta_secs();
        orb.translation.x += vel.x;
        orb.translation.y += vel.y;
    }
}

fn xp_orb_collision(
    player_query: Query<(&Transform, &CircleCollider), With<Player>>,
    orb_query: Query<(&Transform, Entity, &XPOrb)>,
    mut xp_bar_query: Query<&mut XPBar>,
    mut commands: Commands
) {
    let (tr, collider) = player_query.single().unwrap();
    let mut xp_bar = xp_bar_query.single_mut().unwrap();

    for (orb_tr, ent, orb) in orb_query {
        if orb_tr.translation.truncate().distance(tr.translation.truncate()) < collider.0 {
            commands.entity(ent).despawn();
            
            xp_bar.current += orb.0;
        }
    }
}