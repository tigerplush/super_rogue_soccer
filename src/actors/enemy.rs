use bevy::prelude::*;

use crate::actors::actions::Action;

use super::{
    CharacterClass, CurrentPlayer, Team,
    actions::{ActionQueue, Claimed},
};

pub fn enemy_ai(
    query: Single<(Option<&Claimed>, &CharacterClass, &mut ActionQueue), With<CurrentPlayer>>,
) {
    let (claim_option, class, mut action_queue) = query.into_inner();
    if claim_option.is_some() {
        match class {
            &CharacterClass::Goalkeeper => {}
            &CharacterClass::CentralDefender => {}
            &CharacterClass::Midfielder => {}
            &CharacterClass::Attacker => {}
        }
    } else {
        match class {
            &CharacterClass::Goalkeeper => {
                action_queue.0.push(Action::EndTurn(Team::Player));
                action_queue.0.push(Action::DefendGoal);
            }
            &CharacterClass::CentralDefender => {}
            &CharacterClass::Midfielder => {}
            &CharacterClass::Attacker => {}
        }
    }
    if action_queue.0.is_empty() {
        action_queue.0.push(Action::EndTurn(Team::Player));
        action_queue.0.push(Action::SkipTurn);
    }
}
