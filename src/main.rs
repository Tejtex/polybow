use bevy::asset::RenderAssetUsages;
use bevy::color::palettes::css::{DARK_GRAY, GRAY};
use bevy::core_pipeline::bloom::Bloom;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use bevy::{audio::Volume, prelude::*};
use bevy_hanabi::prelude::*;

use crate::AppState::{InGame, MainMenu};
use crate::enemy::EnemyPlugin;
use crate::global::ENEMY_COLOR;
use crate::particles::ParticlePlugin;
use crate::planets::PlanetPlugin;
use crate::player::spawn_player;
use crate::sfx::SFXPlugin;
use crate::ui::UIPlugin;
use crate::world::WorldPlugin;
use crate::xp::XPPlugin;
use crate::{global::GlobalPlugin, player::PlayerPlugin};
use bevy_rapier2d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier2d::prelude::RapierConfiguration;

pub mod arrow;
pub mod enemy;
pub mod global;
pub mod particles;
pub mod planets;
pub mod player;
pub mod sfx;
pub mod ui;
pub mod world;
pub mod xp;

const GLOW_FACTOR: f32 = 10.0;
const SCALE: f32 = 0.3;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
}

#[derive(Component)]
struct MenuButtonAction(String);

#[derive(Component)]
pub struct FirstPass;

#[derive(Component)]
pub struct CursorCamera;


fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            HanabiPlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.),
        ))
        .insert_state(AppState::MainMenu)
        .add_systems(OnEnter(MainMenu), setup_main_menu)
        .add_systems(OnExit(MainMenu), despawn_menu)
        .add_systems(Update, handle_menu_buttons.run_if(in_state(MainMenu)))
        .add_systems(OnEnter(InGame), (setup, set_gravity))
        .add_plugins((
            PlayerPlugin,
            GlobalPlugin,
            EnemyPlugin,
            WorldPlugin,
            ParticlePlugin,
            XPPlugin,
            UIPlugin,
            PlanetPlugin,
            SFXPlugin,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    info!("Starting game loop.");
    let window = window.single().unwrap();
    let mut image = Image::new_fill(
        Extent3d {
            width: (window.width() * SCALE) as u32,
            height: (window.height() * SCALE) as u32,
            ..default()
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    // You need to set these texture usage flags in order to use the image as a render target
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);

    commands
        .spawn((
            Name::new("CameraParent"),
            Transform::default(),
            GlobalTransform::default(),
            FirstPass,
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera2d::default(),
                Camera {
                    hdr: true,
                    target: image_handle.clone().into(),

                    ..default()
                },
                Bloom::NATURAL,
                Projection::Orthographic(OrthographicProjection {
                    scale: 1. / SCALE,
                    scaling_mode: bevy::render::camera::ScalingMode::WindowSize,
                    ..OrthographicProjection::default_2d()
                }),
                RenderLayers::layer(0),
                FirstPass,
            ));
        });

    commands.spawn((
        Sprite {
            image: image_handle.clone(),
            ..default()
        },
        Transform::from_scale(Vec3::splat(1. / SCALE)),
        RenderLayers::layer(1),

    ));

    commands.spawn((Camera2d::default(), RenderLayers::layer(1), CursorCamera));

    commands.spawn((
        AudioPlayer::<AudioSource>(assets.load("music1.wav")),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: Volume::Linear(0.1),
            ..default()
        },
    ));
    spawn_player(&mut commands, &mut meshes, &mut materials, &assets);
}
fn setup_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Spawning main menu");
    commands.spawn(Camera2d::default());
    commands
        .spawn(Node {
            width: Val::Percent(100.),
            margin: UiRect::AUTO,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|parent| {
            // Tytuł
            parent.spawn((
                Text::new("POLYBOW"),
                TextFont {
                    font: asset_server.load("Kenneymini.ttf"),
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Przycisk "Start"
            spawn_button(parent, &asset_server, "Play");
            spawn_button(parent, &asset_server, "Credits");
            spawn_button(parent, &asset_server, "Options");
            spawn_button(parent, &asset_server, "Exit");
        });
}
fn handle_menu_buttons(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &MenuButtonAction),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: EventWriter<bevy::app::AppExit>,
) {
    for (interaction, mut color, action) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                match action.0.as_str() {
                    "Play" => next_state.set(AppState::InGame),
                    "Exit" => {
                        exit.write(bevy::app::AppExit::Success);
                    }
                    _ => {} // inne przyciski, np. encyklopedia
                }
            }
            Interaction::Hovered => *color = GRAY.into(),
            Interaction::None => *color = DARK_GRAY.into(),
        }
    }
}

fn spawn_button(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    asset_server: &Res<AssetServer>,
    text: &str,
) {
    parent
        .spawn((
            Node {
                width: Val::Px(200.),
                height: Val::Px(50.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(10.)),
                ..default()
            },
            BackgroundColor(DARK_GRAY.into()),
            Button::default(),
            MenuButtonAction(text.to_string()),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(text),
                TextFont {
                    font: asset_server.load("Kenneymini.ttf"),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn despawn_menu(
    mut commands: Commands,
    query: Query<Entity, With<Node>>,
    camera: Query<(Entity, &Camera2d)>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    for cam in camera {
        commands.entity(cam.0).despawn_recursive();
    }
}

fn set_gravity(mut config: Query<&mut RapierConfiguration>) {
    if let Ok(mut cfg) = config.single_mut() {
        cfg.gravity = Vec2::ZERO; // lub Vec3::ZERO w 3D
    }
}
