use bevy::prelude::*;
use bevy_hanabi::prelude::*;

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ParticleHandles::default())
            .add_systems(Startup, (setup_xp_trail_particles,setup_enemy_death_particles, setup_enemy_damage_particles, setup_arrow_trail_particles));
    }
}


#[derive(Resource, Default)]
pub struct ParticleHandles {
    pub enemy_death: Handle<EffectAsset>,
    pub enemy_damage: Handle<EffectAsset>,
    pub arrow_trail: Handle<EffectAsset>,
    pub xp_trail: Handle<EffectAsset>
}

fn setup_enemy_death_particles(
    mut particle_handles: ResMut<ParticleHandles>,
    mut effects: ResMut<Assets<EffectAsset>>
) {
    let mut gradient = Gradient::new();
    gradient.add_key(0., Vec4::new(1., 0., 0., 1.));
    gradient.add_key(1., Vec4::splat(0.));

    let mut module = Module::default();

    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(10.),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocitySphereModifier {
        speed: module.lit(60.),
        center: module.lit(Vec3::ZERO),
    };

    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(0.8));
    let init_size = SetAttributeModifier::new(Attribute::SIZE, module.lit(5.0));

    let effect = EffectAsset::new(
        3000,
        SpawnerSettings::once(40.0.into()),
        module
    )
    .init(init_pos)
    .init(init_vel)
    .init(init_lifetime)
    .init(init_size)
    .render(ColorOverLifetimeModifier {gradient, ..default()});

    particle_handles.enemy_death = effects.add(effect);
}

fn setup_enemy_damage_particles(
    mut particle_handles: ResMut<ParticleHandles>,
    mut effects: ResMut<Assets<EffectAsset>>
) {
    let mut gradient = Gradient::new();
    gradient.add_key(0., Vec4::new(1., 0., 0., 1.));
    gradient.add_key(1., Vec4::splat(0.));

    let mut module = Module::default();

    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(3.),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocitySphereModifier {
        speed: module.lit(60.),
        center: module.lit(Vec3::ZERO),
    };

    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(0.5));
    let init_size = SetAttributeModifier::new(Attribute::SIZE, module.lit(3.0));

    let effect = EffectAsset::new(
        3000,
        SpawnerSettings::once(10.0.into()),
        module
    )
    .init(init_pos)
    .init(init_vel)
    .init(init_lifetime)
    .init(init_size)
    .render(ColorOverLifetimeModifier {gradient, ..default()});

    particle_handles.enemy_damage = effects.add(effect);
}

fn setup_arrow_trail_particles(
    mut particle_handles: ResMut<ParticleHandles>,
    mut effects: ResMut<Assets<EffectAsset>>
    
) {
    let mut gradient = Gradient::new();
    gradient.add_key(0., Vec4::new(1., 1., 1., 1.));
    gradient.add_key(1., Vec4::splat(0.));

    let mut module = Module::default();

    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(1.),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocitySphereModifier {
        speed: module.lit(60.),
        center: module.lit(Vec3::ZERO),
    };

    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(0.5));
    let init_size = SetAttributeModifier::new(Attribute::SIZE, module.lit(3.0));

    let effect = EffectAsset::new(
        3000,
        SpawnerSettings::rate(10.0.into()),
        module
    )
    .init(init_pos)
    .init(init_vel)
    .init(init_lifetime)
    .init(init_size)
    .render(ColorOverLifetimeModifier {gradient, ..default()});

    particle_handles.arrow_trail = effects.add(effect);
}

fn setup_xp_trail_particles(
    mut particle_handles: ResMut<ParticleHandles>,
    mut effects: ResMut<Assets<EffectAsset>>
    
) {
    let mut gradient = Gradient::new();
    gradient.add_key(0., Vec4::new(1., 1., 0., 1.));
    gradient.add_key(1., Vec4::splat(0.));

    let mut module = Module::default();

    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(1.),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocitySphereModifier {
        speed: module.lit(60.),
        center: module.lit(Vec3::ZERO),
    };

    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(0.5));
    let init_size = SetAttributeModifier::new(Attribute::SIZE, module.lit(3.0));

    let effect = EffectAsset::new(
        3000,
        SpawnerSettings::rate(3.0.into()),
        module
    )
    .init(init_pos)
    .init(init_vel)
    .init(init_lifetime)
    .init(init_size)
    .render(ColorOverLifetimeModifier {gradient, ..default()});

    particle_handles.xp_trail = effects.add(effect);
}