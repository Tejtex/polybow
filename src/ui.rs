use std::collections::HashMap;

use crate::player::{Crystal, Inventory, PlayerHealth};
use crate::sfx::SFX;
use crate::AppState;
use crate::global::UnwrapOrLogDefault;

use bevy::color::palettes::css::WHITE;
use bevy::window::PrimaryWindow;
use bevy::{color::palettes::css::RED, prelude::*};

use rand::Rng;
use std::error::Error;
use std::fs::File;
use csv::Reader;

fn read_color_names(path: &str) -> Result<Vec<ColorEntry>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = Reader::from_reader(file);
    let mut entries = Vec::new();

    for result in rdr.deserialize() {
        let record: ColorEntry = result?;
        entries.push(record);
    }

    Ok(entries)
}

fn hex_to_rgb(hex: &str) -> Option<[f32; 3]> {
    let hex = hex.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0])
}

fn rgb_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}
fn closest_color<'a>(
    input: &bevy::prelude::Color,
    list: &'a [ColorEntry]
) -> Option<&'a ColorEntry> {
    let color = input.to_srgba();
    let input_rgb = [color.red, color.green, color.blue];

    list.iter()
        .filter_map(|entry| {
            hex_to_rgb(&entry.hex).map(|c| (entry, c))
        })
        .min_by(|(_, c1), (_, c2)| {
            rgb_distance(input_rgb, *c1)
                .partial_cmp(&rgb_distance(input_rgb, *c2))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(entry, _)| entry)
}


const REGENERATE_SPEED: f32 = 1.0;
const REGENERATE_COOLDOWN: f32 = 3.0;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ColorEntry {
    name: String,
    hex: String,
    #[serde(rename = "good name")]
    good_name: Option<String>, // albo String jeśli zawsze coś tam jest
}


#[derive(Component)]
pub struct TooltipNode;

#[derive(Component)]
pub struct TooltipText;


#[derive(Component)]
pub struct HealthBarSegment {
    pub index: usize,
}

#[derive(Component)]
pub struct PlayerHealthBar;

#[derive(Component)]
pub struct XPBar {
    pub level: i32,
    pub(crate) current: f32,
}

#[derive(Resource)]
pub struct LastDamageTime(pub f32);

#[derive(Resource)]
pub struct InventoryVisible(pub bool);

#[derive(Resource)]
pub struct Colors(Vec<ColorEntry>);

#[derive(Component)]
pub struct InventoryNode;

#[derive(Component)]
pub struct CrystalSlot {
    pub index: usize, // indeks w inventory
}

#[derive(Component)]
pub struct CrystalFrame;

#[derive(Resource, Default)]
pub struct SelectedCrystals {
    pub first: Option<usize>,
    pub second: Option<usize>,
}

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_health_bar_ui,
                regenerate_healthbar,
                update_xp_bar,
                handle_keyboard,
                handle_crystal_clicks,
                handle_inventory_shortcuts,
                handle_selected_crystal,
                update_crystal_tooltip
            )
                .run_if(in_state(AppState::InGame)),
        )
            .add_systems(OnEnter(AppState::InGame), load_colors)
        .insert_resource(LastDamageTime(0.0))
        .insert_resource(InventoryVisible(false))
        .insert_resource(SelectedCrystals::default())
            .insert_resource(Colors(Vec::new()));
    }
}

fn load_colors(
    mut colors: ResMut<Colors>,
) {
    colors.0 = read_color_names("colornames.csv").unwrap_or_default_with_log();
}

fn update_health_bar_ui(
    segments_query: Query<(&HealthBarSegment, Entity)>,
    mut nodes_query: Query<(&mut Node, &mut BackgroundColor, Entity), With<HealthBarSegment>>,
    mut player_query: Query<&mut PlayerHealth>,
    mut commands: Commands,
) {
    let segments_map: HashMap<usize, Entity> = segments_query
        .iter()
        .map(|(segment, entity)| (segment.index, entity))
        .collect();

    let mut player = player_query.single_mut().unwrap();

    let current_segment = f32::floor(player.current / player.per_segment as f32) as i32;
    if current_segment + 1 < player.num_segments {
        for i in (current_segment + 1)..player.num_segments {
            let (_, _, ent) = nodes_query
                .get_mut(segments_map.get(&(i as usize)).unwrap().clone())
                .unwrap();

            commands.entity(ent).despawn();
            player.num_segments -= 1;
        }
    }

    for i in 0..(current_segment) {
        let (mut current_node, mut color, _) = nodes_query
            .get_mut(segments_map.get(&(i as usize)).unwrap().clone())
            .unwrap();

        current_node.width = Val::Px(100.0);
        color.0 = Color::Srgba(RED);
    }

    if current_segment == player.num_segments {
        return;
    }

    let (mut last_node, mut last_color, _) = nodes_query
        .get_mut(
            segments_map
                .get(&((current_segment) as usize))
                .unwrap()
                .clone(),
        )
        .unwrap();

    last_node.width = Val::Px(
        100.0
            * ((player.current - current_segment as f32 * player.per_segment as f32)
                / player.per_segment as f32),
    );
    last_color.0 = Color::Srgba(RED);
}

fn regenerate_healthbar(
    last_damage: Res<LastDamageTime>,
    time: Res<Time>,
    mut health_query: Query<&mut PlayerHealth>,
) {
    let mut health = health_query.single_mut().unwrap();
    if last_damage.0 + REGENERATE_COOLDOWN < time.elapsed_secs()
        && health.current + time.delta_secs() * REGENERATE_SPEED
            < (health.num_segments * health.per_segment) as f32
    {
        health.current += time.delta_secs() * REGENERATE_SPEED;
    }
}

fn update_xp_bar(
    mut xp_query: Query<(&mut XPBar, &mut Node, &Children)>,
    mut text_query: Query<&mut Text>,
    sfx: Res<SFX>,
    mut commands: Commands,
) {
    let (mut xp_bar, mut node, children) = xp_query.single_mut().unwrap();
    let per_level = f32::exp2(xp_bar.level as f32) * 50.;
    if xp_bar.current > per_level {
        commands.spawn(AudioPlayer(sfx.levelup.clone()));
        xp_bar.level += 1;
        xp_bar.current = 0.;
    }

    for &child in children {
        if let Ok(mut text) = text_query.get_mut(child) {
            text.0 = xp_bar.level.to_string();
        }
    }

    node.width = Val::Px(600. * (xp_bar.current / per_level));
}

fn handle_keyboard(
    input: Res<ButtonInput<KeyCode>>,
    commands: Commands,
    inventory: Res<Inventory>,
    mut visible: ResMut<InventoryVisible>,
    inventory_query: Query<Entity, With<InventoryNode>>,
    mut time: ResMut<Time<Virtual>>,
    window: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    if input.just_pressed(KeyCode::KeyE) {
        if !visible.0 {
            visible.0 = true;
            spawn_inventory_ui(commands, inventory, window, asset_server);
            time.set_relative_speed(0.2);
        } else {
            visible.0 = false;
            despawn_inventory_ui(commands, inventory, inventory_query);
            time.set_relative_speed(1.);
        }
    }
}

fn despawn_inventory_ui(
    mut commands: Commands,
    inventory: Res<Inventory>,
    inventory_query: Query<Entity, With<InventoryNode>>,
) {
    for node in inventory_query.iter() {
        commands.entity(node).despawn();
    }
}

fn spawn_inventory_ui(
    mut commands: Commands,
    inventory: Res<Inventory>,
    window: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window.single().unwrap();
    let mut crystals = Vec::new();
    for i in 1..=3 {
        let image = asset_server.load(format!("crystal{}.png", i));
        crystals.push(image);
    }

    let frame = asset_server.load("crystalframe.png");

    let mut rng = rand::rng();

    commands
        .spawn((
            Node {
                width: Val::Percent(60.),
                height: Val::Percent(30.),
                margin: UiRect::AUTO,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(WHITE.into()),
            InventoryNode,
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    margin: UiRect::top(Val::Px(50.)),
                    ..default()
                })
                .with_children(|parent2| {
                    let mut i = 0;
                    for crystal in &inventory.crystals {
                        parent2
                            .spawn((
                                Node {
                                    margin: UiRect::all(Val::Px(10.)),
                                    width: Val::Px(100.),
                                    height: Val::Px(100.),
                                    ..default()
                                },
                                ImageNode {
                                    image: frame.clone(),
                                    ..default()
                                },
                                CrystalFrame,
                            ))
                            .with_child((
                                Node {
                                    width: Val::Px(60.),
                                    height: Val::Px(60.),
                                    margin: UiRect::AUTO,
                                    ..default()
                                },
                                ImageNode {
                                    image: crystals[rng.random_range(0..crystals.len())].clone(),
                                    image_mode: NodeImageMode::Stretch,
                                    color: crystal.color.to_bevy(),
                                    ..default()
                                },
                                CrystalSlot { index: i },
                                BackgroundColor(WHITE.into()),
                                Button,
                            ));
                        i += 1;
                    }
                });
        });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.),
                top: Val::Px(10.),
                width: Val::Px(150.),
                height: Val::Auto,
                ..default()
            },
            ZIndex(10),
            BackgroundColor(Color::linear_rgba(0.0, 0.0, 0.0, 0.7).into()),
            TooltipNode,
            Visibility::Hidden,
            InventoryNode
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TooltipText,
            ));
        });
}

fn update_crystal_tooltip(
    mut tooltip_query: Query<(&mut Text, &ChildOf), With<TooltipText>>,
    mut tooltip_node: Query<(&mut Visibility, &mut Node), With<TooltipNode>>,
    interaction_query: Query<(&Interaction, &CrystalSlot), (Changed<Interaction>, With<Button>)>,
    inventory: Res<Inventory>,
    window: Query<&Window, With<PrimaryWindow>>,
    colors: Res<Colors>
) {
    let window = window.single().unwrap();
    for (mut text, parent) in &mut tooltip_query {
        for (interaction, slot) in &interaction_query {
            if *interaction == Interaction::Hovered {
                if let Some(position) = window.cursor_position() {
                    let data = &inventory.crystals[slot.index];
                    let mut visibility = tooltip_node.get_mut(parent.parent()).unwrap().0;
                    *visibility = Visibility::Visible;
                    let mut node = tooltip_node.get_mut(parent.parent()).unwrap().1;
                    node.left = Val::Px(position.x);
                    node.top = Val::Px(position.y);
                    let color_name = closest_color(&data.color.to_bevy(), &colors.0);
                    text.0 = format!(
                        "Level: {}\nColor: {}\nEffect: {:?}",
                        data.effect.level, color_name.unwrap().name, data.effect.effect_type
                    );
                }
            } else {
                let mut visibility = tooltip_node.get_mut(parent.parent()).unwrap().0;
                *visibility = Visibility::Hidden;
            }
        }
    }
}

fn handle_crystal_clicks(
    mut interactions: Query<
        (&Interaction, &CrystalSlot, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut selected: ResMut<SelectedCrystals>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    for (interaction, slot, mut color) in &mut interactions {
        if *interaction == Interaction::Pressed {
            // if buttons.pressed(MouseButton::Left) {
            //     selected.first = Some(slot.index);
            //     *color = BackgroundColor(Color::linear_rgb(0.6, 0.6, 1.0)); // niebieska ramka
            // }
            if buttons.pressed(MouseButton::Right) {
                println!("HI");
                selected.second = Some(slot.index);
                *color = BackgroundColor(Color::linear_rgb(1.0, 0.6, 0.6)); // czerwona ramka
            }
        }
    }
}

fn handle_selected_crystal(
    selected: ResMut<SelectedCrystals>,
    mut crystals: Query<(&CrystalSlot, &mut BackgroundColor)>,
) {
    for (slot, mut color) in &mut crystals {
        if selected.first == Some(slot.index) {
            *color = BackgroundColor(Color::linear_rgb(0.6, 0.6, 1.0));
        } else if selected.second == Some(slot.index) {
            *color = BackgroundColor(Color::linear_rgb(1.0, 0.6, 0.6));
        } else {
            *color = BackgroundColor(WHITE.into());
        }
    }
}

fn handle_inventory_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedCrystals>,
    mut frames: Query<Entity, With<CrystalFrame>>,
    mut crystals: Query<(&CrystalSlot, &ChildOf)>,
    mut inventory: ResMut<Inventory>,
    sfx: Res<SFX>,
    mut commands: Commands,
    bar_query: Query<&mut XPBar>,
) {
    if keys.just_pressed(KeyCode::KeyS) {
        if let Some(index) = selected.first {
            inventory.sell(index, bar_query);
            commands.spawn(AudioPlayer(sfx.sell.clone()));
            for (crystal, parent) in &mut crystals {
                if Some(crystal.index) == selected.first {
                    commands
                        .entity(frames.get(parent.parent()).unwrap_or_default_with_log(""))
                        .despawn();
                }
            }
            selected.first = None;
        }
    }

    if keys.just_pressed(KeyCode::KeyC) {
        if let (Some(a), Some(b)) = (selected.first, selected.second) {
            inventory.combine(a, b);
            commands.spawn(AudioPlayer(sfx.combine.clone()));
            for (crystal, parent) in &mut crystals {
                if Some(crystal.index) == selected.first {
                    commands
                        .entity(frames.get(parent.parent()).unwrap())
                        .despawn();
                }
                if Some(crystal.index) == selected.second {
                    commands
                        .entity(frames.get(parent.parent()).unwrap())
                        .despawn();
                }
            }
            selected.first = None;
            selected.second = None;
        }
    }
}
