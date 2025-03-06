use bevy::{prelude::*, ui::widget::NodeImageMode};

use crate::{PanelBorderAsset, actors, map, theme::prelude::*};

use super::{AppState, GameplayStates};
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
        designate_current_player,
    )
    .add_systems(OnExit(GameplayStates::PlayerTurn), spend_player)
    .add_systems(OnEnter(GameplayStates::EnemyTurn), designate_current_player)
    .add_systems(OnExit(GameplayStates::EnemyTurn), spend_player);
}

#[derive(Component)]
pub struct InfoContainer;

fn startup(panel_border: Res<PanelBorderAsset>, mut commands: Commands) {
    commands.ui_root().with_children(|root| {
        root.spawn((
            Name::from("Info Container"),
            Node {
                width: Val::Percent(25.0),
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

fn spend_player(query: Query<Entity, With<CurrentPlayer>>, mut commands: Commands) {
    for entity in &query {
        commands
            .entity(entity)
            .remove::<CurrentPlayer>()
            .insert(HasActed);
    }
}
