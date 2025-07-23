use bevy::prelude::*;
use bevy_hanabi::ParticleEffect;
use rand::Rng;


use crate::{global::{CircleCollider, ScreenShake, Velocity}, particles::ParticleHandles, player::{Arrow, LastDamageTime, Player, PlayerHealth, XPOrb}, world::EnemiesCounter, GLOW_FACTOR};


const ENEMY_SPEED: f32 = 1.0;
const ENEMY_DAMAGE: i32 = 1;

#[derive(Component)]
#[require(Velocity, Mesh2d, MeshMaterial2d<ColorMaterial>, HP, CircleCollider)]
pub struct Enemy {
    pub sides: i32
}

#[derive(Component, Default)]
pub struct HP {
    pub current: i32,
    pub max: i32
}

#[derive(Component, Default)]
pub struct HealthBar;

#[derive(Component)]
pub struct HealthBarOwner(pub Entity);

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (update_health_bars, update_health_bar_position, handle_enemy_damage, handle_ai, handle_collision));
    }
}

fn update_health_bars(
    health_query: Query<(&HP, Entity)>,
    mut bar_query: Query<(&mut Transform, &HealthBarOwner), With<HealthBar>>,
) {
    for (mut bar_transform, owner) in &mut bar_query {
        if let Ok((hp, _)) = health_query.get(owner.0) {
            let health_percent: f32 = hp.current.max(0) as f32 / hp.max.max(1) as f32;
            bar_transform.scale.x = health_percent.clamp(0.0, 1.0);
        }
    }
}

fn update_health_bar_position(
    enemy_query: Query<&GlobalTransform, With<Enemy>>,
    mut bar_query: Query<(&mut Transform, &HealthBarOwner), With<HealthBar>>,
) {
    for (mut bar_transform, owner) in &mut bar_query {
        if let Ok(enemy_transform) = enemy_query.get(owner.0) {
            let pos = enemy_transform.translation();
            bar_transform.translation = Vec3::new(pos.x, pos.y + 40.0, pos.z + 0.1);
            bar_transform.rotation = Quat::IDENTITY;
        }
    }
}

fn handle_enemy_damage(
    mut commands: Commands,
    mut enemy_query: Query<(&Transform, &CircleCollider, Entity, &mut HP), With<Enemy>>,
    arrow_query: Query<(&Transform, Entity), With<Arrow>>,
    health_bar_query: Query<(Entity, &HealthBarOwner), With<HealthBar>>,
    mut enemies: ResMut<EnemiesCounter>,
    mut shake: ResMut<ScreenShake>,
    particle_handles: Res<ParticleHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>

) {
    for (enemy_tf, collider, enemy_ent, mut hp) in &mut enemy_query {
        let enemy_pos = enemy_tf.translation.truncate();
        let radius = collider.0 * enemy_tf.scale.x.max(enemy_tf.scale.y);

        for (arrow_tf, arrow_ent) in &arrow_query {
            let arrow_pos = arrow_tf.translation.truncate();

            let dist = enemy_pos.distance(arrow_pos);
            if dist <= radius {

                hp.current -= 1;
                commands.entity(arrow_ent).despawn();
                shake.trauma = 1.0;
                commands.spawn((
                    ParticleEffect::new(particle_handles.enemy_damage.clone()),
                    Transform::from_translation(arrow_pos.extend(0.0))
                ));

                if hp.current <= 0 {
                    for (bar_ent, owner) in &health_bar_query {
                        if owner.0 == enemy_ent {
                            commands.entity(bar_ent).despawn();
                        }
                    }

                    commands.entity(enemy_ent).despawn();
                    shake.trauma = 4.0;
                    enemies.0 -= 1;

                    commands.spawn((
                        ParticleEffect::new(particle_handles.enemy_death.clone()),
                        Transform::from_translation(enemy_tf.translation),
                    ));

                    spawn_orbs(
                        &mut commands,
                        5.,
                        enemy_tf.translation,
                        &particle_handles,
                        &mut meshes,
                        &mut materials
                    );

                    
                }
            }
        }
    }
}

fn handle_ai(
    enemy_query: Query<(& Transform, &mut Velocity, &Enemy), Without<Player>>,
    player_query: Query<(&Transform, &Velocity), (With<Player>, Without<Enemy>)>
) {
    let player  = player_query.single().unwrap();
    for (enemy_tr, mut vel, enemy) in enemy_query {
        match enemy.sides {
            3 => {
                let player_tr = player.0;

                let angle = (player_tr.translation.y - enemy_tr.translation.y).atan2(player_tr.translation.x - enemy_tr.translation.x);
                vel.dx = angle.cos() * ENEMY_SPEED;
                vel.dy = angle.sin() * ENEMY_SPEED;
            }
            4 => {
                let player_tr = player.0;
                if let Some(d) = calculate_intercept_direction(enemy_tr.translation.truncate(), player_tr.translation.truncate(), Vec2::new(player.1.dx, player.1.dy), ENEMY_SPEED) {
                    vel.dx = d.x * ENEMY_SPEED;
                    vel.dy = d.y * ENEMY_SPEED;
                }
                
            }
            _ => ()
        }
    }
}

fn calculate_intercept_direction(
    shooter_pos: Vec2,
    target_pos: Vec2,
    target_vel: Vec2,
    projectile_speed: f32,
) -> Option<Vec2> {
    let to_target = target_pos - shooter_pos;

    let a = target_vel.length_squared() - projectile_speed * projectile_speed;
    let b = 2.0 * to_target.dot(target_vel);
    let c = to_target.length_squared();

    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        
        return None;
    }

    let sqrt_disc = discriminant.sqrt();
    let t1 = (-b - sqrt_disc) / (2.0 * a);
    let t2 = (-b + sqrt_disc) / (2.0 * a);

    let t = if t1 > 0.0 {
        t1
    } else if t2 > 0.0 {
        t2
    } else {
        return None; 
    };

    let aim_point = target_pos + target_vel * t;
    let direction = (aim_point - shooter_pos).normalize();
    Some(direction)
}

fn handle_collision(
    mut player_query: Query<(&mut PlayerHealth, &CircleCollider, &Transform)>,
    enemy_query: Query<(&CircleCollider, &Transform, &Enemy, Entity)>,
    health_bar_query: Query<(Entity, &HealthBarOwner), With<HealthBar>>,
    mut commands: Commands,
    mut counter: ResMut<EnemiesCounter>,
    mut last_damage: ResMut<LastDamageTime>,
    time: Res<Time>,
    mut shake: ResMut<ScreenShake>,
    particle_handles: Res<ParticleHandles>
) {
    let (mut health, player_collider, player_tr) = player_query.single_mut().unwrap();
    for (enemy_collider, enemy_tr, enemy, entity) in enemy_query {
        if enemy_tr.translation.truncate().distance(player_tr.translation.truncate()) < enemy_collider.0 + player_collider.0 {
            match enemy.sides {
                3..=4 => {
                    health.current -= ENEMY_DAMAGE as f32;
                    commands.entity(entity).despawn();
                    for (health_bar_ent, owner) in health_bar_query {
                        if owner.0 == entity {
                            commands.entity(health_bar_ent).despawn();
                        }
                    }
                    counter.0 -= 1;
                    last_damage.0 = time.elapsed_secs();
                    shake.trauma = 2.0;
                    commands.spawn((
                        ParticleEffect::new(particle_handles.enemy_damage.clone()),
                        Transform::from_translation(enemy_tr.translation),
                    ));
                }
                _ => ()
            }
        }
    }
}

fn spawn_orbs(
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
            ParticleEffect::new(particle_handles.xp_trail.clone())
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