use bevy::prelude::*;
use bevy_hanabi::ParticleEffect;
use bevy_rapier2d::prelude::*;


use crate::{arrow::Arrow, global::{CircleCollider, ScreenShake}, particles::ParticleHandles, player::{Player, PlayerHealth}, ui::LastDamageTime, world::EnemiesCounter, xp::spawn_orbs, AppState};
use crate::sfx::SFX;

const ENEMY_SPEED: f32 = 50.0;
const ENEMY_DAMAGE: i32 = 1;

#[derive(Component)]
#[require(Velocity, Mesh2d, MeshMaterial2d<ColorMaterial>, HP, CircleCollider)]
pub struct Enemy {
    pub sides: i32,
}


#[derive(Component, Default)]
pub struct HP {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Default)]
pub struct HealthBar;

#[derive(Component)]
pub struct HealthBarOwner(pub Entity);

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (update_health_bars, update_health_bar_position, handle_ai, handle_collision, collision_events_system).run_if(in_state(AppState::InGame)));
    }
}

fn update_health_bars(
    health_query: Query<(&HP, Entity)>,
    mut bar_query: Query<(&mut Transform, &HealthBarOwner), With<HealthBar>>,
) {
    for (mut bar_transform, owner) in &mut bar_query {
        if let Ok((hp, _)) = health_query.get(owner.0) {
            let health_percent: f32 = hp.current.max(0.) as f32 / hp.max.max(1.) as f32;
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

fn collision_events_system(
    mut collision_events: EventReader<CollisionEvent>,
    mut enemy_query: Query<(Entity, &mut HP, &mut Velocity, &Transform), With<Enemy>>,
    mut arrow_query: Query<(Entity, &Velocity, &Arrow, &Transform), Without<Enemy>>,
    health_bar_query: Query<(Entity, &HealthBarOwner), With<HealthBar>>,
    mut commands: Commands,
    mut shake: ResMut<ScreenShake>,
    particle_handles: Res<ParticleHandles>,
    mut enemies: ResMut<EnemiesCounter>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    sfx: Res<SFX>,
) {
    for collision in collision_events.read() {
        match collision {
            CollisionEvent::Started(e1, e2, _) => {
                if let Ok(mut enemy) = enemy_query.get_mut(e1.entity()) {
                    if let Ok(mut arrow) = arrow_query.get_mut(e2.entity()) {
                        handle_enemy_damage(
                            &mut commands,
                            &mut enemy,
                            &mut arrow,
                            health_bar_query,
                            &mut enemies,
                            &mut shake,
                            &particle_handles,
                            &mut meshes,
                            &mut materials,
                            &sfx,
                        );
                    }
                }
                if let Ok(mut enemy) = enemy_query.get_mut(e2.entity()) {
                    if let Ok(mut arrow) = arrow_query.get_mut(e1.entity()) {
                        handle_enemy_damage(
                            &mut commands,
                            &mut enemy,
                            &mut arrow,
                            health_bar_query,
                            &mut enemies,
                            &mut shake,
                            &particle_handles,
                            &mut meshes,
                            &mut materials,
                            &sfx,
                        );
                    }
                }
            }
            _ => ()
        }
    }
}

fn handle_enemy_damage(
    mut commands: &mut Commands,
    enemy: &mut (Entity, Mut<HP>, Mut<Velocity>, &Transform),
    arrow: &(Entity, &Velocity, &Arrow, &Transform),
    health_bar_query: Query<(Entity, &HealthBarOwner), With<HealthBar>>,
    enemies: &mut ResMut<EnemiesCounter>,
    shake: &mut ResMut<ScreenShake>,
    particle_handles: &Res<ParticleHandles>,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    sfx: &Res<SFX>,
) {
    enemy.1.current -= arrow.2.damage;
    commands.entity(arrow.0).despawn();
    shake.trauma = 1.0;
    commands.spawn((
        ParticleEffect::new(particle_handles.enemy_damage.clone()),
        Transform::from_translation(arrow.3.translation)
    ));
    commands.spawn(AudioPlayer(sfx.hurt.clone()));
    enemy.2.linvel.x += arrow.1.linvel.x;
    enemy.2.linvel.y += arrow.1.linvel.y;

    if enemy.1.current <= 0. {
        for (bar_ent, owner) in &health_bar_query {
            if owner.0 == enemy.0 {
                commands.entity(bar_ent).despawn();
            }
        }

        commands.entity(enemy.0).despawn();
        shake.trauma = 4.0;
        enemies.0 -= 1;

        commands.spawn((
            ParticleEffect::new(particle_handles.enemy_death.clone()),
            Transform::from_translation(enemy.3.translation),
        ));

        spawn_orbs(
            &mut commands,
            5.,
            enemy.3.translation,
            &particle_handles,
            &mut meshes,
            &mut materials,
        );
    }
}




fn handle_ai(
    enemy_query: Query<(&Transform, &mut Velocity, &Enemy), Without<Player>>,
    player_query: Query<(&Transform, &Velocity), (With<Player>, Without<Enemy>)>,
) {
    let player = player_query.single().unwrap();
    for (enemy_tr, mut vel, enemy) in enemy_query {
        match enemy.sides {
            3 => {
                let player_tr = player.0;

                let d = (player_tr.translation - enemy_tr.translation).truncate().normalize();
                vel.linvel = vel.linvel.lerp(d * ENEMY_SPEED, 0.1);
            }
            4 => {
                let player_tr = player.0;
                if let Some(d) = calculate_intercept_direction(enemy_tr.translation.truncate(), player_tr.translation.truncate(), Vec2::new(player.1.linvel.x, player.1.linvel.y), ENEMY_SPEED) {
                    vel.linvel = vel.linvel.lerp(d * ENEMY_SPEED, 0.1);
                } else {
                    vel.linvel = vel.linvel.lerp((-enemy_tr.translation + player_tr.translation).truncate().normalize() * ENEMY_SPEED, 0.1);
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
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 { return None; }
    let sqrt = disc.sqrt();
    let t1 = (-b - sqrt) / (2.0 * a);
    let t2 = (-b + sqrt) / (2.0 * a);
    let t = if t1 > 0.0 && t2 > 0.0 {
        t1.min(t2)
    } else if t1 > 0.0 {
        t1
    } else if t2 > 0.0 {
        t2
    } else {
        return None;
    };

    let aim = target_pos + target_vel * t;
    Some((aim - shooter_pos).normalize())
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
    particle_handles: Res<ParticleHandles>,
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
