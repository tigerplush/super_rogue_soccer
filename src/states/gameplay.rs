use bevy::{
    color::palettes::css::{DARK_CYAN, GREEN, ORANGE},
    prelude::*,
    ui::widget::NodeImageMode,
};

use crate::{
    AppSet, FontAsset, PanelBorderAsset,
    actors::{self, enemy::enemy_ai},
    map,
    theme::prelude::*,
};

use super::{AppState, GameplayStates, splash::ImageNodeFadeInOut};
use crate::actors::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::Gameplay),
        (
            actors::startup,
            actions::setup_slotmap,
            map::spawn_field,
            startup,
        ),
    )
    .add_systems(
        OnEnter(GameplayStates::PlayerTurn),
        (
            designate_current_player,
            (show_banner, paint_character).after(designate_current_player),
        ),
    )
    .add_systems(
        OnExit(GameplayStates::PlayerTurn),
        (spend_player, paint_character.after(spend_player)),
    )
    .add_systems(
        OnEnter(GameplayStates::EnemyTurn),
        (
            designate_current_player,
            (show_banner, paint_character).after(designate_current_player),
        ),
    )
    .add_systems(Update, enemy_ai.run_if(in_state(GameplayStates::EnemyTurn)))
    .add_systems(
        OnExit(GameplayStates::EnemyTurn),
        (spend_player, paint_character.after(spend_player)),
    )
    .add_systems(Update, (fade, remove_banner).in_set(AppSet::Update));
}

#[derive(Component)]
pub struct InfoContainer;

#[derive(Component)]
pub struct Log;

fn startup(panel_border: Res<PanelBorderAsset>, mut commands: Commands) {
    commands.ui_root().with_children(|root| {
        root.spawn((
            Name::from("Header"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(36.0),
                padding: UiRect::all(Val::Percent(1.5)),
                ..default()
            },
            ImageNode {
                image: panel_border.image.clone_weak(),
                image_mode: NodeImageMode::Sliced(panel_border.slicer.clone()),
                ..default()
            },
        ));
        root.spawn((
            Name::from("Info Container"),
            Node {
                width: Val::Px(344.0),
                height: Val::Percent(100.0),
                align_self: AlignSelf::FlexEnd,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Percent(1.5)),
                ..default()
            },
            ImageNode {
                image: panel_border.image.clone_weak(),
                image_mode: NodeImageMode::Sliced(panel_border.slicer.clone()),
                ..default()
            },
            InfoContainer,
        ));
        root.spawn((
            Name::from("Footer"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(180.0),
                overflow: Overflow::clip_y(),
                overflow_clip_margin: OverflowClipMargin::content_box().with_margin(8.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Percent(1.5)),
                ..default()
            },
            ImageNode {
                image: panel_border.image.clone_weak(),
                image_mode: NodeImageMode::Sliced(panel_border.slicer.clone()),
                ..default()
            },
            Log,
        ));
    });
}

#[derive(Component)]
struct HasActed;

fn designate_current_player(
    current_team: Res<State<GameplayStates>>,
    query: Query<(Entity, &Team, &Stats, Option<&HasActed>)>,
    mut commands: Commands,
) {
    let current = match current_team.get() {
        GameplayStates::PlayerTurn => Team::Player,
        GameplayStates::EnemyTurn => Team::Enemy,
    };

    let mut available_players: Vec<(Entity, &Stats)> = query
        .iter()
        .filter(|(_, team, _, acted_option)| **team == current && acted_option.is_none())
        .map(|(entity, _, stats, _)| (entity, stats))
        .collect();

    if available_players.is_empty() {
        for (entity, _, _, _) in &query {
            commands.entity(entity).remove::<HasActed>();
        }
    }
    available_players = query
        .iter()
        .filter(|(_, team, _, acted_option)| **team == current && acted_option.is_none())
        .map(|(entity, _, stats, _)| (entity, stats))
        .collect();
    if let Some((selected, _)) = available_players
        .iter()
        .max_by_key(|(_, stats)| stats.initiative)
    {
        commands.entity(*selected).insert(CurrentPlayer);
    }
}

fn paint_character(mut query: Query<(&mut Sprite, &Team, Option<&CurrentPlayer>)>) {
    for (mut sprite, team, player_option) in &mut query {
        let color = match player_option {
            Some(_) => GREEN,
            None => match team {
                Team::Player => ORANGE.into(),
                Team::Enemy => DARK_CYAN.into(),
            },
        };
        sprite.color = color.into();
    }
}

fn spend_player(query: Query<Entity, With<CurrentPlayer>>, mut commands: Commands) {
    for entity in &query {
        commands
            .entity(entity)
            .remove::<CurrentPlayer>()
            .insert(HasActed);
    }
}

fn show_banner(
    font_asset: Res<FontAsset>,
    panel_border: Res<PanelBorderAsset>,
    query: Single<(&Name, &Team), With<CurrentPlayer>>,
    mut commands: Commands,
) {
    commands
        .ui_root()
        .insert((ZIndex(1), ImageNodeFadeInOut::default().with_t(0.6)))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(20.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode {
                    image: panel_border.image.clone_weak(),
                    image_mode: NodeImageMode::Sliced(panel_border.slicer.clone()),
                    ..default()
                },
            ))
            .with_children(|banner| {
                let (name, team) = query.into_inner();
                let designation = match team {
                    Team::Player => "PLAYERS",
                    Team::Enemy => "ENEMIES",
                };
                banner.spawn((
                    Text::new(format!("{} TURN", designation)),
                    TextFont {
                        font: font_asset.clone_weak(),
                        font_size: 50.0,
                        ..default()
                    },
                ));
                banner.spawn((
                    Text::new(format!("CURRENT PLAYER: {}", name)),
                    TextFont {
                        font: font_asset.clone_weak(),
                        ..default()
                    },
                ));
            });
        });
}

fn fade(
    fades: Query<(&ImageNodeFadeInOut, &Children)>,
    mut images: Query<(&mut ImageNode, &Children)>,
    mut texts: Query<&mut TextColor>,
) {
    for (fade, children) in &fades {
        for entity in children {
            if let Ok((mut image, node_children)) = images.get_mut(*entity) {
                image.color.set_alpha(fade.alpha());
                for node_child in node_children {
                    if let Ok(mut text_color) = texts.get_mut(*node_child) {
                        text_color.0.set_alpha(fade.alpha());
                    }
                }
            }
        }
    }
}

fn remove_banner(fades: Query<(Entity, &ImageNodeFadeInOut)>, mut commands: Commands) {
    for (entity, fade) in &fades {
        if fade.elapsed() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
