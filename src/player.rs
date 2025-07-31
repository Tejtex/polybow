use crate::arrow::MAX_ARROW_SPEED;
use crate::global::{ScreenShake, regular_polygon_vertices};
use crate::planets::Effect;
use crate::{
    AppState, FirstPass, GLOW_FACTOR, SCALE,
    arrow::{ARROW_SPEED, Arrow},
    global::{CircleCollider, PLAYER_COLOR},
    particles::ParticleHandles,
    ui::{HealthBarSegment, PlayerHealthBar, XPBar},
    xp,
};
use bevy::render::view::RenderLayers;
use bevy::{
    color::palettes::css::{DARK_GRAY, YELLOW},
    prelude::*,
};
use bevy_hanabi::ParticleEffect;
use bevy_rapier2d::prelude::*;
use std::collections::HashSet;
use rand::Rng;

const PLAYER_SPEED: f32 = 200.0;
const KNOCKBACK: f32 = 20.0;
const BOW_OFFSET: f32 = 55.0;
const XP_PER_LEVEL: f32 = 10.;

#[derive(Component)]
#[require(Velocity, Mesh2d, MeshMaterial2d<ColorMaterial>)]
pub struct Player;

#[derive(Component)]
pub struct Bow;

#[derive(Default)]
struct BowState {
    charging: bool,
    charge_time: f32,
    charging_arrow: Option<Entity>,
}

#[derive(Component)]
pub struct ChargingArrow {
    pub charge_time: f32,
}

#[derive(Component, Default, Copy, Clone)]
pub struct PreviousPosition(pub Vec3);

#[derive(Component)]
pub struct PlayerHealth {
    pub current: f32,
    pub per_segment: i32,
    pub num_segments: i32,
}

#[derive(Resource, Default)]
pub struct Inventory {
    pub crystals: Vec<Crystal>,
}

impl Inventory {
    pub fn sell(&mut self, index: usize, mut xp_bar_query: Query<&mut XPBar>) {
        if index >= self.crystals.len() {
            warn!("Index out of bounds: {}", index);
            return;
        }
        let lvl = self.crystals[index].effect.level;
        let mut bar = xp_bar_query.single_mut().unwrap();
        bar.current += lvl as f32 * XP_PER_LEVEL;
        self.crystals.remove(index);
    }

    pub fn combine(&mut self, a: usize, b: usize) {
        if a >= self.crystals.len()  {
            warn!("Index out of bounds: {}", a);
            return;
        }
        if b >= self.crystals.len() {
            warn!("Index out of bounds: {}", b);
            return;
        }

        let mut rng = rand::rng();

        let crystal1 = &self.crystals[b];
        let crystal2 = &self.crystals[a];

        let alignment = ((crystal1.phase - crystal2.phase).abs().powf(3.) + (crystal1.resonance - crystal2.resonance).abs().powf(3.)) / 2.;

        let avrg_level = (crystal1.effect.level + crystal2.effect.level)  as f32 / 2.;

        let level = (avrg_level * (1. - alignment) * 2.) as i32;

        let effect_type = if rng.random::<bool>() { &crystal1.effect.effect_type } else { &crystal2.effect.effect_type };

        let phase = self.bounded_random_around((crystal1.phase - crystal2.phase) / 2., alignment, 0.5, &mut rng);
        let resonance = self.bounded_random_around((crystal1.resonance - crystal2.resonance) / 2., alignment, 0.5, &mut rng);

        let r = ((( crystal1.color.r as i32 + crystal2.color.r as i32) % 255 / 2)) as u8;
        let g = ((( crystal1.color.g as i32 + crystal2.color.g as i32) % 255 / 2)) as u8;
        let blue = ((( crystal1.color.b as i32 + crystal2.color.b as i32) % 255 / 2)) as u8;
        let color = ColorId::new(r, g, blue);

        let new_crystal = Crystal {
            effect: Effect {
                level,
                effect_type: effect_type.clone()
            },
            phase,
            resonance,
            color
        };
        if a > b {
            self.crystals.remove(a);
            self.crystals[b] = new_crystal;
        } else {
            self.crystals.remove(b);
            self.crystals[a] = new_crystal;
        }


    }
    fn bounded_random_around(&self, avg: f32, alignment: f32, range: f32, rng: &mut impl Rng) -> f32 {
        let spread = range * alignment;
        let min = (avg - spread).clamp(0.0, 1.0);
        let max = (avg + spread).clamp(0.0, 1.0);
        rng.random_range(min..=max)
    }

}

#[derive(Default, PartialEq)]
pub struct Crystal {
    pub color: ColorId,
    pub effect: Effect,
    pub phase: f32,
    pub resonance: f32
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct ColorId {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColorId {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        ColorId { r, g, b }
    }

    pub fn to_bevy(&self) -> bevy::prelude::Color {
        Color::srgb_u8(self.r, self.g, self.b)
    }

    pub fn from_bevy(color: &Color, with_glow: bool) -> Option<Self> {
        // tylko jeśli pochodzi z Color::rgb_u8
        if let Color::Srgba(Srgba {
            red,
            blue,
            green,
            alpha: _,
        }) = *color
        {
            Some(Self::new(
                (red * 255.0 / if with_glow { GLOW_FACTOR } else { 1. }) as u8,
                (green * 255.0 / if with_glow { GLOW_FACTOR } else { 1. }) as u8,
                (blue * 255.0 / if with_glow { GLOW_FACTOR } else { 1. }) as u8,
            ))
        } else if let Color::LinearRgba(LinearRgba {
            red,
            green,
            blue,
            alpha: _,
        }) = *color
        {
            Some(Self::new(
                (red * 255.0 / if with_glow { GLOW_FACTOR } else { 1. }) as u8,
                (green * 255.0 / if with_glow { GLOW_FACTOR } else { 1. }) as u8,
                (blue * 255.0 / if with_glow { GLOW_FACTOR } else { 1. }) as u8,
            ))
        } else {
            None // nie da się dokładnie przeliczyć np. z Hsla
        }
    }
}

struct ActiveEffects {
    pub effects: Vec<Effect>,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_keys,
                update_bow_position,
                handle_mouse,
                smooth_camera_follow,
                update_charging_arrow,
            )
                .run_if(in_state(AppState::InGame)),
        )
        .insert_resource(Inventory::default());
    }
}

fn handle_keys(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut Transform), With<Player>>,
) {
    let mut player = query.single_mut().unwrap();

    if keyboard_input.pressed(KeyCode::KeyA) {
        player.0.linvel.x -= PLAYER_SPEED * time.delta_secs()
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        player.0.linvel.x += PLAYER_SPEED * time.delta_secs()
    }
    if keyboard_input.pressed(KeyCode::KeyW) {
        player.0.linvel.y += PLAYER_SPEED * time.delta_secs()
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        player.0.linvel.y -= PLAYER_SPEED * time.delta_secs()
    }
}

pub fn spawn_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    assets: &Res<AssetServer>,
) {
    let bow = assets.load("bow.png");

    commands.spawn((
        Player,
        Mesh2d(meshes.add(RegularPolygon::new(30., 3))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(PLAYER_COLOR))),
        PreviousPosition(Vec3::ZERO),
        PlayerHealth {
            current: 36.0,
            per_segment: 10,
            num_segments: 4,
        },
        CircleCollider(30.0),
        Collider::convex_hull(
            &regular_polygon_vertices(30., 3)
                .iter()
                .map(|e| Vect::from(e.clone()))
                .collect::<Vec<Vect>>(),
        )
        .unwrap(),
        RigidBody::Dynamic,
        // Velocity::angular(2.0 * PI / 3.0),
        Damping {
            linear_damping: 0.5,
            angular_damping: 0.,
        },
        Restitution::coefficient(1.0),
        ExternalImpulse::default(),
        RenderLayers::layer(0),
        FirstPass,
    ));

    commands.spawn((
        Bow,
        Sprite {
            image: bow,
            color: Color::linear_rgb(GLOW_FACTOR, GLOW_FACTOR, GLOW_FACTOR),
            ..default()
        },
        Transform::from_scale(Vec3::new(0.5, 0.5, 0.5)),
        RenderLayers::layer(0),
        FirstPass,
    ));

    commands
        .spawn((
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
            PlayerHealthBar,
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

    commands
        .spawn((
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
                current: 0.0,
            },
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
                },
            ));
        });
}

fn update_bow_position(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<FirstPass>>,
    mut bow_query: Query<&mut Transform, (With<Bow>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<Bow>)>,
) {
    let mut bow = bow_query.single_mut().unwrap();
    let player = player_query.single().unwrap();

    let window = windows.single().unwrap();

    if let Some(cursor_pos) = window.cursor_position() {
        let (camera, camera_transform) = camera_q.single().unwrap();

        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos * SCALE) {
            let alpha = f32::atan2(
                world_pos.y - player.translation.y,
                world_pos.x - player.translation.x,
            );

            let x = player.translation.x + BOW_OFFSET * f32::cos(alpha);
            let y = player.translation.y + BOW_OFFSET * f32::sin(alpha);

            bow.translation.x = x;
            bow.translation.y = y;

            bow.rotation = Quat::from_rotation_z(alpha);
        }
    }
}

fn update_charging_arrow(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<FirstPass>>,
    mut arrow_query: Query<(&mut Transform, &mut ChargingArrow)>,
    player_query: Query<&Transform, (With<Player>, Without<ChargingArrow>)>,
    time: Res<Time>,
    mut camera_shake: ResMut<ScreenShake>,
) {
    if arrow_query.iter().count() != 1 {
        return;
    }
    let mut arrow = arrow_query.single_mut().unwrap();
    if arrow.1.charge_time < 2. {
        arrow.1.charge_time += time.delta_secs();
    }

    let player = player_query.single().unwrap();

    let window = windows.single().unwrap();

    if let Some(cursor_pos) = window.cursor_position() {
        let (camera, camera_transform) = camera_q.single().unwrap();

        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos * SCALE) {
            let alpha = f32::atan2(
                world_pos.y - player.translation.y,
                world_pos.x - player.translation.x,
            );

            let x =
                player.translation.x + (BOW_OFFSET - arrow.1.charge_time * 5.) * f32::cos(alpha);
            let y =
                player.translation.y + (BOW_OFFSET - arrow.1.charge_time * 5.) * f32::sin(alpha);
            camera_shake.trauma = arrow.1.charge_time * 0.2;

            arrow.0.translation.x = x;
            arrow.0.translation.y = y;

            arrow.0.rotation = Quat::from_rotation_z(alpha);
        }
    }
}

fn handle_mouse(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<FirstPass>>,
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    assets: Res<AssetServer>,
    mut player_query: Query<(&Transform, &mut Velocity), (With<Player>, Without<Bow>)>,
    mut bow_state: Local<BowState>,
    time: Res<Time>,
    particle_handles: Res<ParticleHandles>,
) {
    let window = windows.single().unwrap();
    let mut player = player_query.single_mut().unwrap();

    if mouse.just_pressed(MouseButton::Left) {
        bow_state.charging = true;
        bow_state.charge_time = 0.0;
        let mut id = None;
        if let Some(cursor_pos) = window.cursor_position() {
            let (camera, camera_transform) = camera_q.single().unwrap();

            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos * SCALE)
            {
                let alpha = f32::atan2(
                    world_pos.y - player.0.translation.y,
                    world_pos.x - player.0.translation.x,
                );

                let arrow = assets.load("arrow.png");

                id = Some(
                    commands
                        .spawn((
                            ChargingArrow { charge_time: 0. },
                            Sprite {
                                image: arrow,
                                color: Color::linear_rgb(GLOW_FACTOR, GLOW_FACTOR, GLOW_FACTOR),
                                ..default()
                            },
                            Transform {
                                scale: Vec3::new(0.3, 0.3, 0.3),
                                rotation: Quat::from_rotation_z(alpha),
                                translation: Vec3::new(
                                    player.0.translation.x + f32::cos(alpha) * BOW_OFFSET,
                                    player.0.translation.y + f32::sin(alpha) * BOW_OFFSET,
                                    0.0,
                                ),
                            },
                            RenderLayers::layer(0),
                            FirstPass,
                        ))
                        .id(),
                );
            }
        }
        bow_state.charging_arrow = id;
    }

    if bow_state.charging && mouse.pressed(MouseButton::Left) && bow_state.charge_time < 2. {
        bow_state.charge_time += time.delta_secs();
    }

    if bow_state.charging && mouse.just_released(MouseButton::Left) {
        bow_state.charging = false;
        if let Some(cursor_pos) = window.cursor_position() {
            let (camera, camera_transform) = camera_q.single().unwrap();

            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos * SCALE)
            {
                let alpha = f32::atan2(
                    world_pos.y - player.0.translation.y,
                    world_pos.x - player.0.translation.x,
                );

                commands
                    .entity(bow_state.charging_arrow.unwrap())
                    .insert((
                        Arrow {
                            damage: bow_state.charge_time,
                        },
                        ActiveEvents::COLLISION_EVENTS,
                        Velocity {
                            linvel: Vect::new(
                                (ARROW_SPEED * f32::cos(alpha) * bow_state.charge_time)
                                    .clamp(-MAX_ARROW_SPEED, MAX_ARROW_SPEED),
                                (ARROW_SPEED * f32::sin(alpha) * bow_state.charge_time)
                                    .clamp(-MAX_ARROW_SPEED, MAX_ARROW_SPEED),
                            ),
                            ..default()
                        },
                        RigidBody::Dynamic,
                        ParticleEffect::new(particle_handles.arrow_trail.clone()),
                        Collider::cuboid(27.0, 3.15),
                        Restitution::coefficient(1.),
                        RenderLayers::layer(0),
                        FirstPass,
                    ))
                    .remove::<ChargingArrow>();
                player.1.linvel.x -= f32::cos(alpha) * KNOCKBACK * bow_state.charge_time;
                player.1.linvel.y -= f32::sin(alpha) * KNOCKBACK * bow_state.charge_time;
            }
        }

        bow_state.charge_time = 0.0;
    }
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
