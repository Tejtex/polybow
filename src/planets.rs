use std::ops::DerefMut;
use bevy::prelude::*;
use bevy_hanabi::ParticleEffect;
use bevy_rapier2d::prelude::*;
use crate::AppState;
use crate::arrow::Arrow;
use crate::global::ScreenShake;
use crate::particles::ParticleHandles;
use crate::player::{ColorId, Crystal, Inventory};
use crate::sfx::SFX;

#[derive(Component, Clone)]
pub struct Planet {
    pub color: Color,
    pub effect: Effect,
    pub hp: f32,
    pub max_hp: f32
}

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct Effect {
    pub effect_type: EffectType,
    pub level: i32
}

#[derive(Default, Clone, PartialEq, Eq, Hash, Debug)]
pub enum EffectType {
    #[default]
    Poison,
    Speed,
    Fire
}

pub struct PlanetPlugin;

impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (collision_events_system).run_if(in_state(AppState::InGame)));
    }
}

fn collision_events_system(
    mut collision_events: EventReader<CollisionEvent>,
    mut planet_query: Query<(&mut Planet, Entity)>,
    mut arrow_query: Query<(&mut Arrow, &Transform, Entity)>,
    mut commands: Commands,
    mut shake: ResMut<ScreenShake>,
    mut inventory: ResMut<Inventory>,
    particle_handles: Res<ParticleHandles>,
    sfx: Res<SFX>,
) {
    for event in collision_events.read() {
        match event {
            CollisionEvent::Started(e1, e2, _flags) => {

                if let Ok(mut planet) = planet_query.get_mut(*e1) {
                    if let Ok(mut arrow) = arrow_query.get_mut(*e2) {
                        collision(&sfx, planet.0.deref_mut(), arrow.0.deref_mut(),  &mut arrow.2.clone(),&arrow.1.clone().translation.truncate(), &mut planet.1.clone(), &mut commands, &mut shake, &particle_handles, &mut inventory);
                    }
                } else if let Ok(mut planet) = planet_query.get_mut(*e2) {
                    if let Ok(mut arrow) = arrow_query.get_mut(*e1) {
                        collision(&sfx, planet.0.deref_mut(), arrow.0.deref_mut(),  &mut arrow.2.clone(),&arrow.1.clone().translation.truncate(), &mut planet.1.clone(), &mut commands, &mut shake, &particle_handles, &mut inventory);
                    }
                }

            },
            _ => ()
        }
    }
}

fn collision(
    sfx: &Res<SFX>,
    planet: &mut Planet,
    arrow: &mut Arrow,
    arrow_ent: &mut Entity,
    arrow_pos: &Vec2,
    planet_ent: &mut Entity,
    commands: &mut Commands,
    shake: &mut ResMut<ScreenShake>,
    particle_handles: &Res<ParticleHandles>,
    inventory: &mut ResMut<Inventory>,
) {
    planet.hp -= arrow.damage;
    commands.entity(*arrow_ent).despawn();
    shake.trauma = 1.0;
    commands.spawn((
        ParticleEffect::new(particle_handles.enemy_damage.clone()),
        Transform::from_translation(arrow_pos.extend(0.0))
    ));
    commands.spawn(AudioPlayer(sfx.hurt.clone()));

    if planet.hp <= 0.0 {
        commands.entity(*planet_ent).despawn();
        inventory.crystals.push(Crystal { color: ColorId::from_bevy(&planet.color.clone(), true).unwrap(), effect: planet.effect.clone()});
    }
}