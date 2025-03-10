use bevy::{color::palettes::css::{DARK_CYAN, GREEN, ORANGE}, prelude::*};

use crate::prelude::*;

use super::GameplayStates;

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameplayStates::ShowBanner),
        (designate_current_player, paint_character.after(designate_current_player))
    )
    .add_systems(Update, dummy.run_if(in_state(GameplayStates::ShowBanner)))
    .add_systems(OnExit(GameplayStates::ShowBanner), dummy);
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

fn dummy() {}
