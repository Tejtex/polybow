use bevy::color::Color::Srgba;
use bevy::prelude::*;
use bevy::prelude::Color::LinearRgba;
use rand::Rng;


use crate::{player::{Player, PreviousPosition}, AppState, GLOW_FACTOR};

const TRAUMA_FALLOFF_SPEED: f32 = 6.0;

pub(crate) const ENEMY_COLOR: Color = Color::linear_rgb(245.0 / 255.0 * GLOW_FACTOR, 59.0 / 255.0 * GLOW_FACTOR, 93.0 / 255.0 * GLOW_FACTOR);
pub(crate) const PLAYER_COLOR: Color = Color::linear_rgb(5.0 / 255.0 * GLOW_FACTOR, 157.0 / 255.0 * GLOW_FACTOR, 240.0 / 255.0 * GLOW_FACTOR);


#[derive(Component, Default)]
pub struct CircleCollider(pub f32);

pub struct GlobalPlugin;

impl Plugin for GlobalPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (fade_trail.after(spawn_trail),spawn_trail, apply_screen_shake).run_if(in_state(AppState::InGame)))
            .insert_resource(ScreenShake::default());
    }
}

#[derive(Resource)]
pub struct ScreenShake {
    pub trauma: f32,
}

impl Default for ScreenShake {
    fn default() -> Self {
        ScreenShake { trauma: 0.0 }
    }
}

#[derive(Component)]
struct Trail {
    lifetime: f32,
}

fn spawn_trail(
    time: Res<Time>,
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut PreviousPosition), With<Player>>,
    mut timer: Local<f32>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    const TRAIL_STEP: f32 = 0.005;
    *timer += time.delta_secs();

    if let Ok((transform, mut prev_pos)) = player_query.single_mut() {
        let start = prev_pos.0;
        let end = transform.translation;

        let distance = end.distance(start);
        let steps = (distance / (TRAIL_STEP * 1000.0)).ceil() as usize;

        for i in 0..steps {
            let t = i as f32 / steps as f32;
            let pos = start.lerp(end, t);

            commands.spawn((
                Trail { lifetime: (0.2 -(steps - i) as f32 * TRAIL_STEP).max(TRAIL_STEP) },
                Mesh2d(meshes.add(Circle::new(20.0))),
                MeshMaterial2d(materials.add(ColorMaterial {
                    color: Color::LinearRgba(PLAYER_COLOR.to_linear().with_alpha(0.2).with_luminance(20.0)),
                    ..default()
                })),
                Transform {
                    translation: pos.with_z(-1.0),
                    ..default()
                },
            ));
        }

        *timer = 0.0;
        prev_pos.0 = end;
    }
}

fn fade_trail(
    time: Res<Time>,
    mut commands: Commands,
    mut trail_query: Query<(Entity, &mut Trail, &mut Transform, &mut MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut trail, mut transform, material_handle) in &mut trail_query {
        trail.lifetime -= time.delta_secs();

        let lifetime_ratio = trail.lifetime / 0.2; 

        
        let scale_factor = (lifetime_ratio.ln_1p()).max(0.01); 

        transform.scale = Vec3::splat(scale_factor);

        
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.color.set_alpha(scale_factor * 0.5); 
        }

        if trail.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}


fn apply_screen_shake(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
    mut shake: ResMut<ScreenShake>,
) {
    shake.trauma = (shake.trauma - time.delta_secs() * TRAUMA_FALLOFF_SPEED).clamp(0.0, 1.0);
    if shake.trauma > 0.0 {
        let mut rng = rand::rng();
        let intensity = f32::powf(shake.trauma, 2.5);

        let offset_x = rng.random_range(-1.0..1.0) * 10.0 * intensity;
        let offset_y = rng.random_range(-1.0..1.0) * 10.0 * intensity;

        for mut transform in camera_query.iter_mut() {
            transform.translation.x = offset_x;
            transform.translation.y = offset_y;
        }
    } else {
        // resetuj kamerę
        for mut transform in camera_query.iter_mut() {
            transform.translation.x = 0.0;
            transform.translation.y = 0.0;
        }
    }
}

pub fn regular_polygon_vertices(radius: f32, sides: usize) -> Vec<Vec2> {
    (0..sides)
        .map(|i| {
            let angle = i as f32 / sides as f32 * std::f32::consts::TAU;
            Vec2::new(-radius * angle.sin(),radius * angle.cos())
        })
        .collect()
}

pub fn adjusted_glow(color: Color, base_glow: f32) -> Color {
    let srgb = color.to_srgba();
    let r = srgb.red;
    let g = srgb.green;
    let b = srgb.blue;
    let luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let correction = (1.0 / luminance.clamp(0.2, 1.0));
    let glow_factor = base_glow * correction;
    Color::linear_rgba(r * glow_factor, g * glow_factor, b * glow_factor, srgb.alpha)
}
pub trait UnwrapOrLogDefault<T> {
    fn unwrap_or_default_with_log(self, msg: &str) -> T;
}

impl<T, E: std::fmt::Debug> UnwrapOrLogDefault<T> for Result<T, E>
where
    T: Default,
{
    fn unwrap_or_default_with_log(self, msg: &str) -> T {
        match self {
            Ok(val) => val,
            Err(e) => {
                error!("{}: {:?}", msg, e);
                T::default()
            }
        }
    }
}


