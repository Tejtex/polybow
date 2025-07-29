use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct SFX {
    pub hurt: Handle<AudioSource>,
    pub xp: Handle<AudioSource>,
    pub levelup: Handle<AudioSource>,
    pub sell: Handle<AudioSource>,
    pub combine: Handle<AudioSource>,
}

pub struct SFXPlugin;

impl Plugin for SFXPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(SFX::default())
            .add_systems(Startup, setup_sfx);
    }
}

fn setup_sfx(
    asset_server: Res<AssetServer>,
    mut sfx: ResMut<SFX>,
) {
    info!("Loading SFX.");
    sfx.hurt = asset_server.load("hurt.wav");
    sfx.xp = asset_server.load("xp.wav");
    sfx.levelup = asset_server.load("levelup.wav");
    sfx.sell = asset_server.load("sell.wav");
    sfx.combine = asset_server.load("combine.wav");
    info!("SFX loaded.");
}