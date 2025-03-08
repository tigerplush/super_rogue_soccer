use bevy::prelude::*;

use crate::actors::actions::Action;

use super::{
    Ball, CharacterClass, CurrentPlayer, Team,
    actions::{ActionQueue, Claimed, ClaimedBy},
};

pub fn enemy_ai(
    query: Single<(Option<&Claimed>, &CharacterClass, &mut ActionQueue), With<CurrentPlayer>>,
    ball_query: Single<(Entity, &Transform, Option<&ClaimedBy>), With<Ball>>,
    team_members: Query<(&Transform, &Team), Without<Ball>>,
) {
    let (claim_option, class, mut action_queue) = query.into_inner();
    if claim_option.is_some() {
        match class {
            &CharacterClass::Goalkeeper => {
                action_queue.0.push(Action::PassDown);
            }
            &CharacterClass::CentralDefender => {
                action_queue.0.push(Action::PassDown);
            }
            &CharacterClass::Midfielder => {
                action_queue.0.push(Action::PassDown);
            }
            &CharacterClass::Attacker => {
                // when ball is claimed, advance
                // when goal in range, take a shot
            }
        }
    } else {
        match class {
            &CharacterClass::Goalkeeper => {
                action_queue.0.push(Action::EndTurn(Team::Player));
                action_queue.0.push(Action::DefendGoal);
            }
            &CharacterClass::CentralDefender => {
                // mark someone that comes into the defense field
                // try to stay between that player and the ball in range
            }
            &CharacterClass::Midfielder => {
                // mark a player
                // try to stay between that player and the ball in range
            }
            &CharacterClass::Attacker => {
                let (entity, transform, claimed_by_option) = ball_query.into_inner();
                if let Some(claimed_by) = claimed_by_option {
                    let (enemy, team) = team_members.get(claimed_by.0).unwrap();
                    match team {
                        // when ball is claimed by a team member, advance
                        Team::Enemy => {
                            action_queue.0.push(Action::Advance);
                        }
                        // when ball is claimed by an enemy, try to take it away
                        Team::Player => {
                            action_queue.0.push(Action::EndTurn(Team::Player));
                            action_queue.0.push(Action::TakeControl(entity));
                            action_queue.0.push(Action::MoveTo(enemy.translation));
                        }
                    }
                } else {
                    // when ball is unclaimed, try to claim it
                    action_queue.0.push(Action::EndTurn(Team::Player));
                    action_queue.0.push(Action::TakeControl(entity));
                    action_queue.0.push(Action::MoveTo(transform.translation));
                }
            }
        }
    }
    if action_queue.0.is_empty() {
        action_queue.0.push(Action::EndTurn(Team::Player));
        action_queue.0.push(Action::SkipTurn);
    }
}
