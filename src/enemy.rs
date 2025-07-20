use bevy::prelude::*;


use crate::{global::{CircleCollider, Velocity}, player::{Arrow, Player}, world::EnemiesCounter};


const ENEMY_SPEED: f32 = 1.0;

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
            .add_systems(Update, (update_health_bars, update_health_bar_position, handle_enemy_damage, handle_ai));
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
    mut enemies: ResMut<EnemiesCounter>
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

                if hp.current <= 0 {
                    for (bar_ent, owner) in &health_bar_query {
                        if owner.0 == enemy_ent {
                            commands.entity(bar_ent).despawn();
                        }
                    }

                    commands.entity(enemy_ent).despawn();
                    enemies.0 -= 1;
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
