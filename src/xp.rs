use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_hanabi::ParticleEffect;
use rand::prelude::*;

use crate::{global::CircleCollider, particles::ParticleHandles, player::Player, ui::XPBar, AppState, GLOW_FACTOR};
use crate::sfx::SFX;

const G: f32 =50000.0;

#[derive(Component)]
pub struct XPOrb(pub f32);

pub struct XPPlugin;

impl Plugin for XPPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (move_xp_orb, xp_orb_collision).run_if(in_state(AppState::InGame)));
    }
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
    mut commands: Commands,
    sfx: Res<SFX>,
) {
    let (tr, collider) = player_query.single().unwrap();
    let mut xp_bar = xp_bar_query.single_mut().unwrap();

    for (orb_tr, ent, orb) in orb_query {
        if orb_tr.translation.truncate().distance(tr.translation.truncate()) < collider.0 {
            commands.entity(ent).despawn();
            
            xp_bar.current += orb.0;
            commands.spawn(AudioPlayer(sfx.xp.clone()));
        }
    }
}

pub fn spawn_orbs(
    commands: &mut Commands,
    sum: f64,
    translation: Vec3,
    particle_handles: &Res<ParticleHandles>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>
) {
    let mut rng = rand::rng();
    for size in losowe_sumujace_sie_do_x(4, sum) {
        commands.spawn((
            XPOrb(size as f32),
            Transform::from_translation(translation + Vec3::new(rng.random_range(-30.0..30.0), rng.random_range(-30.0..30.0), 0.)),
            Mesh2d(meshes.add(Circle::new(rng.random_range(0.5..2.3)))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::linear_rgb(GLOW_FACTOR, GLOW_FACTOR, 0.)))),
            ParticleEffect::new(particle_handles.xp_trail.clone()),
            RenderLayers::layer(0)
        ));
    }
    
}

fn losowe_sumujace_sie_do_x(n: usize, x: f64) -> Vec<f64> {
    let mut rng = rand::rng();

    // Wygeneruj n losowych liczb
    let mut liczby: Vec<f64> = (0..n).map(|_| rng.random::<f64>()).collect();

    // Policz sumę
    let suma: f64 = liczby.iter().sum();

    // Przeskaluj, żeby suma wynosiła dokładnie x
    for i in 0..n {
        liczby[i] = liczby[i] / suma * x;
    }

    liczby
}