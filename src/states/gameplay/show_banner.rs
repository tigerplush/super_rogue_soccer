use bevy::{
    color::palettes::css::{DARK_CYAN, GREEN, ORANGE},
    prelude::*,
    ui::widget::NodeImageMode,
};

use crate::prelude::*;

use super::GameplayStates;

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameplayStates::ShowBanner),
        (
            designate_current_player,
            (show_banner, paint_character).after(designate_current_player),
        ),
    )
    .add_systems(Update, check_node.run_if(in_state(GameplayStates::ShowBanner)))
    .add_systems(OnExit(GameplayStates::ShowBanner), clean_up);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct HasActed;

fn designate_current_player(
    current_team: Res<CurrentTeam>,
    mut query: Query<(Entity, &Teams, &mut Stats, Option<&HasActed>)>,
    mut commands: Commands,
) {
    let mut available_players: Vec<(Entity, &Stats)> = query
        .iter()
        .filter(|(_, team, _, acted_option)| **team == current_team.0 && acted_option.is_none())
        .map(|(entity, _, stats, _)| (entity, stats))
        .collect();
    if available_players.is_empty() {
        for (entity, _, mut stats, _) in &mut query {
            stats.reset_ap();
            commands.entity(entity).remove::<HasActed>();
        }
        available_players = query
            .iter()
            .filter(|(_, team, _, _)| **team == current_team.0)
            .map(|(entity, _, stats, _)| (entity, stats))
            .collect();
    }
    if let Some((selected, _)) = available_players
        .iter()
        .max_by_key(|(_, stats)| stats.initiative)
    {
        commands.entity(*selected).insert(CurrentPlayer);
    }
}

fn paint_character(mut query: Query<(&mut Sprite, &Teams, Option<&CurrentPlayer>)>) {
    for (mut sprite, team, player_option) in &mut query {
        let color = match player_option {
            Some(_) => GREEN,
            None => match team {
                Teams::Player => ORANGE.into(),
                Teams::Enemy => DARK_CYAN.into(),
            },
        };
        sprite.color = color.into();
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Banner;

fn show_banner(
    font_asset: Res<FontAsset>,
    panel_border: Res<PanelBorderAsset>,
    query: Single<(&Name, &Teams), With<CurrentPlayer>>,
    mut commands: Commands,
) {
    commands
        .ui_root()
        .insert((ZIndex(1), Banner))
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
                ImageNodeFadeInOut::default().faded_in()
            ))
            .with_children(|banner| {
                let (name, team) = query.into_inner();
                let designation = match team {
                    Teams::Player => "PLAYERS",
                    Teams::Enemy => "ENEMIES",
                };
                banner.spawn((
                    Text::new(format!("{} TURN", designation)),
                    TextFont {
                        font: font_asset.clone_weak(),
                        font_size: 50.0,
                        ..default()
                    },
                    ImageNodeFadeInOut::default().faded_in()
                ));
                banner.spawn((
                    Text::new(format!("CURRENT PLAYER: {}", name)),
                    TextFont {
                        font: font_asset.clone_weak(),
                        ..default()
                    },
                    ImageNodeFadeInOut::default().faded_in()
                ));
            });
        });
}

fn check_node(current_team: Res<CurrentTeam>, mut next: ResMut<NextState<GameplayStates>>, query: Query<&ImageNodeFadeInOut>) {
    if query.iter().all(|element| element.elapsed()) {
        let next_state = match current_team.0 {
            Teams::Player => GameplayStates::PlayerTurn,
            Teams::Enemy => GameplayStates::EnemyTurn,
        };
        next.set(next_state);
    }
}

fn clean_up(query: Query<Entity, With<Banner>>, mut commands: Commands) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}