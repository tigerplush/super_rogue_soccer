use std::collections::HashMap;

use bevy::prelude::*;
use leafwing_input_manager::{plugin::InputManagerSystem, prelude::*};

use crate::{
    AppSet, PostUpdateSet,
    entities::{Interactable, Map},
    states::*,
    to_ivec2,
};

use super::{
    CurrentPlayer, PointerObject, is_dirty,
    pathfinding::{self, CalculatedPath},
};

pub fn plugin(app: &mut App) {
    app.register_type::<CurrentActions>()
        .insert_resource(CurrentActions { actions: vec![] })
        .add_plugins(InputManagerPlugin::<Slots>::default())
        .add_plugins(InputManagerPlugin::<PlayerAbilities>::default())
        .add_systems(
            PreUpdate,
            copy_action_state.after(InputManagerSystem::ManualControl),
        )
        .add_systems(Update, report_abilities_used.in_set(AppSet::Update))
        .add_systems(
            PostUpdate,
            calculate_current_actions
                .in_set(PostUpdateSet::Move)
                .run_if(in_state(AppState::Gameplay).and(is_dirty)),
        );
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct CurrentActions {
    pub actions: Vec<(String, String, PlayerAbilities)>,
}

fn calculate_current_actions(
    map: Res<Map>,
    pointer: Single<&Transform, With<PointerObject>>,
    interactables: Query<&Interactable>,
    ability_slot: Single<&mut AbilitySlotMap>,
    mut commands: Commands,
) {
    let transform = pointer.into_inner();
    let position = to_ivec2(transform.translation);
    let mut actions = vec![];
    let mut slot_map = ability_slot.into_inner();
    slot_map.clear();
    match map.get(&position) {
        Some(entities) => {
            for &entity in entities {
                match interactables.get(entity).unwrap() {
                    &Interactable::Ball => {
                        actions.push(("f".to_string(), "walk".to_string(), PlayerAbilities::Walk));
                        slot_map.insert(Slots::Ability1, PlayerAbilities::Walk);
                        actions.push((
                            "g".to_string(),
                            "take control".to_string(),
                            PlayerAbilities::Walk,
                        ));
                        slot_map.insert(Slots::Ability2, PlayerAbilities::TakeControl);
                        actions.push(("h".to_string(), "kick".to_string(), PlayerAbilities::Kick));
                        slot_map.insert(Slots::Ability3, PlayerAbilities::Kick);
                    }
                    &Interactable::Person => {
                        actions.push(("f".to_string(), "walk".to_string(), PlayerAbilities::Walk));
                        slot_map.insert(Slots::Ability1, PlayerAbilities::Walk);
                        actions.push((
                            "g".to_string(),
                            "take control".to_string(),
                            PlayerAbilities::Walk,
                        ));
                        slot_map.insert(Slots::Ability2, PlayerAbilities::TakeControl);
                        actions.push(("h".to_string(), "kick".to_string(), PlayerAbilities::Kick));
                        slot_map.insert(Slots::Ability3, PlayerAbilities::Kick);
                        actions.push(("i".to_string(), "foul".to_string(), PlayerAbilities::Foul));
                        slot_map.insert(Slots::Ability3, PlayerAbilities::Foul);
                    }
                }
            }
        }
        None => {
            actions.push(("f".to_string(), "walk".to_string(), PlayerAbilities::Walk));
            slot_map.insert(Slots::Ability1, PlayerAbilities::Walk);
        }
    }
    commands.insert_resource(CurrentActions { actions });
}

#[derive(Actionlike, Reflect, Clone, Hash, Eq, PartialEq, Debug, Copy)]
pub enum PlayerAbilities {
    Walk,
    TakeControl,
    Kick,
    Foul,
}

#[derive(Actionlike, Reflect, Clone, Hash, Eq, PartialEq, Debug, Copy)]
enum Slots {
    Ability1,
    Ability2,
    Ability3,
    Ability4,
    Ability5,
    Ability6,
}

impl Slots {
    fn variants() -> impl Iterator<Item = Slots> {
        use Slots::*;
        [Ability1, Ability2, Ability3, Ability4, Ability5, Ability6]
            .iter()
            .copied()
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut)]
struct AbilitySlotMap {
    map: HashMap<Slots, PlayerAbilities>,
}

pub fn setup_slotmap(mut commands: Commands) {
    commands.spawn((
        Name::from("Player Controls"),
        InputMap::new([
            (Slots::Ability1, KeyCode::KeyF),
            (Slots::Ability2, KeyCode::KeyG),
            (Slots::Ability3, KeyCode::KeyH),
            (Slots::Ability4, KeyCode::KeyJ),
            (Slots::Ability5, KeyCode::KeyK),
            (Slots::Ability6, KeyCode::KeyL),
        ]),
        ActionState::<Slots>::default(),
        ActionState::<PlayerAbilities>::default(),
        AbilitySlotMap::default(),
    ));
}

fn copy_action_state(
    mut query: Query<(
        &mut ActionState<Slots>,
        &mut ActionState<PlayerAbilities>,
        &AbilitySlotMap,
    )>,
) {
    for (mut slot_state, mut ability_state, ability_slot_map) in &mut query {
        for slot in Slots::variants() {
            if let Some(matching_ability) = ability_slot_map.get(&slot) {
                ability_state.set_button_data(
                    *matching_ability,
                    slot_state.button_data_mut_or_default(&slot).clone(),
                );
            }
        }
    }
}

fn report_abilities_used(
    query: Query<&ActionState<PlayerAbilities>>,
    player: Option<Single<(Entity, &Transform), With<CurrentPlayer>>>,
    target: Option<Single<&Transform, With<PointerObject>>>,
    mut commands: Commands,
) {
    if player.is_none() || target.is_none() {
        return;
    }
    let (player_entity, player_transform) = player.unwrap().into_inner();
    let target_transform = target.unwrap().into_inner();
    for ability_state in &query {
        for ability in ability_state.get_just_pressed() {
            match ability {
                PlayerAbilities::Walk => {
                    // find current team member
                    // calculate path
                    let path_result = pathfinding::calculate_path(
                        player_transform.translation,
                        target_transform.translation,
                    );
                    match path_result {
                        Ok(path) => {
                            commands
                                .entity(player_entity)
                                .insert(CalculatedPath::new(path, 0.5));
                        }
                        Err(_) => {
                            info!("no path available");
                        }
                    }
                    // move until AP are consumed
                }
                PlayerAbilities::Kick => {
                    // find current team member
                    // check if in range
                    // if in range, move there and add velocity to target
                    // velocity is proportional to the moved fields and kick strength
                    // vector and accuracy is proportional to taken path and skill

                    // this could lead to fun experiments where e.g. Dugle McFrouglas takes control over a team member
                    // kicks the team member into an enemy, who will die and the team member, because of hurting the enemy
                    // will be expulsed from the game while Dugle McFrouglas still remains

                    // Also, you could use one player really good at kicking to maneuver team members over the field
                }
                PlayerAbilities::TakeControl => {
                    // find current team member
                    // check if in range
                    // if in range, move there and try to take control
                    // with unclaimed ball will always work
                    // with claimed ball, roll wit against each other
                    // with enemy, roll atk vs defense?
                    // now target is claimed and will move with the entity
                    // enemies get a chance to free themselves every round
                }
                PlayerAbilities::Foul => {
                    // find current team member
                    // check if in range
                    // if in range, move there
                    // roll atk vs defense
                    // if succesful, target is hurt -> may die
                    // entity will receive a caution, when receiving a second caution, entity is eliminated from play
                }
                _ => (),
            }
        }
    }
}
