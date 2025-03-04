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
    CurrentPlayer, PointerObject, PreviewPath, Stats, Velocity, is_dirty,
    pathfinding::{self, CalculatedPath},
};

pub fn plugin(app: &mut App) {
    app.register_type::<CurrentActions>()
        .register_type::<Kicked>()
        .insert_resource(CurrentActions { actions: vec![] })
        .add_plugins(InputManagerPlugin::<Slots>::default())
        .add_plugins(InputManagerPlugin::<PlayerAbilities>::default())
        .add_systems(
            PreUpdate,
            copy_action_state.after(InputManagerSystem::ManualControl),
        )
        .add_systems(
            Update,
            (report_abilities_used, process_actions, process_kick).in_set(AppSet::Update),
        )
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
    pub actions: Vec<(String, String, bool)>,
}

fn calculate_current_actions(
    map: Res<Map>,
    path: Res<PreviewPath>,
    pointer: Single<&Transform, With<PointerObject>>,
    interactables: Query<&Interactable>,
    ability_slot: Single<&mut AbilitySlotMap>,
    stats: Single<&Stats, With<CurrentPlayer>>,
    mut commands: Commands,
) {
    let mut slot_map = ability_slot.into_inner();
    slot_map.clear();

    let transform = pointer.into_inner();
    let target_position = to_ivec2(transform.translation);
    let mut actions = vec![];
    let in_range = path.path.len() <= stats.ap;

    actions.push(("f".to_string(), "walk".to_string(), in_range));
    if in_range {
        slot_map.insert(Slots::Ability1, PlayerAbilities::Walk);
    }

    if let Some(entities) = map.get(&target_position) {
        for &entity in entities {
            actions.push(("g".to_string(), "take control".to_string(), in_range));
            actions.push(("h".to_string(), "kick".to_string(), in_range));
            if in_range {
                slot_map.insert(Slots::Ability2, PlayerAbilities::TakeControl(entity));
                slot_map.insert(Slots::Ability3, PlayerAbilities::Kick(entity));
            }
            match interactables.get(entity).unwrap() {
                &Interactable::Person => {
                    actions.push(("i".to_string(), "foul".to_string(), in_range));
                    slot_map.insert(Slots::Ability3, PlayerAbilities::Foul(entity));
                }
                _ => (),
            }
        }
    }
    commands.insert_resource(CurrentActions { actions });
}

#[derive(Actionlike, Reflect, Clone, Hash, Eq, PartialEq, Debug, Copy)]
pub enum PlayerAbilities {
    Walk,
    TakeControl(Entity),
    Kick(Entity),
    Foul(Entity),
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

#[derive(Component, Default)]
pub struct ActionQueue(Vec<Action>);

enum Action {
    MoveTo(Vec3),
    Kick(Entity),
    TakeControl(Entity),
    Foul(Entity),
}

fn report_abilities_used(
    query: Query<&ActionState<PlayerAbilities>>,
    player: Option<Single<&mut ActionQueue, With<CurrentPlayer>>>,
    target: Option<Single<&Transform, With<PointerObject>>>,
) {
    if player.is_none() || target.is_none() {
        return;
    }
    let mut queue = player.unwrap().into_inner();
    let target_transform = target.unwrap().into_inner();
    for ability_state in &query {
        for ability in ability_state.get_just_pressed() {
            match ability {
                PlayerAbilities::Walk => {
                    queue.0.push(Action::MoveTo(target_transform.translation));
                }
                PlayerAbilities::Kick(target) => {
                    queue.0.push(Action::Kick(target));
                    queue.0.push(Action::MoveTo(target_transform.translation));
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
                PlayerAbilities::TakeControl(target) => {
                    queue.0.push(Action::TakeControl(target));
                    queue.0.push(Action::MoveTo(target_transform.translation));
                    // find current team member
                    // check if in range
                    // if in range, move there and try to take control
                    // with unclaimed ball will always work
                    // with claimed ball, roll wit against each other
                    // with enemy, roll atk vs defense?
                    // now target is claimed and will move with the entity
                    // enemies get a chance to free themselves every round
                }
                PlayerAbilities::Foul(target) => {
                    queue.0.push(Action::Foul(target));
                    queue.0.push(Action::MoveTo(target_transform.translation));
                    // find current team member
                    // check if in range
                    // if in range, move there
                    // roll atk vs defense
                    // if succesful, target is hurt -> may die
                    // entity will receive a caution, when receiving a second caution, entity is eliminated from play
                }
            }
        }
    }
}

fn process_actions(
    mut query: Query<(
        Entity,
        &Transform,
        &mut ActionQueue,
        &Stats,
        &Velocity,
        Option<&CalculatedPath>,
    )>,
    mut commands: Commands,
) {
    for (entity, transform, mut queue, stats, velocity, path_option) in &mut query {
        if path_option.is_some() {
            continue;
        }

        if let Some(action) = queue.0.pop() {
            match action {
                Action::MoveTo(target) => {
                    let Ok(path) = pathfinding::calculate_path(transform.translation, target)
                    else {
                        continue;
                    };
                    commands
                        .entity(entity)
                        .insert((Velocity(Vec2::ZERO), CalculatedPath::new(path, 0.5)));
                }
                Action::Kick(target) => {
                    commands
                        .entity(target)
                        .insert(Kicked(velocity.0 * stats.kick_strength));
                }
                Action::TakeControl(target) => {}
                Action::Foul(target) => {}
            }
        }
    }
}

fn process_kick(
    time: Res<Time>,
    map: Res<Map>,
    mut query: Query<(&mut Transform, &mut Kicked, Entity)>,
    interactables: Query<&Interactable>,
    mut commands: Commands,
) {
    const EPSILON: f32 = 0.1;
    for (mut transform, mut kicked, entity) in &mut query {
        let mut translation = transform.translation;
        let total_movement = kicked.0 * time.delta_secs();
        let steps = total_movement.length().ceil() as i32;
        let step_size = total_movement.extend(0.0) / steps as f32;

        let mut exit = false;
        for _ in 0..steps {
            let next_translation = translation + step_size;
            let current_position = to_ivec2(translation);
            let next_position = to_ivec2(next_translation);
            if let Some(entities) = map.get(&next_position) {
                for entity in entities {
                    match interactables.get(*entity).unwrap() {
                        Interactable::Wall => {
                            let normal = get_wall_normal(current_position, &map);
                            kicked.0 = reflect_velocity(kicked.0, normal);
                            exit = true;
                        }
                        Interactable::Person => {
                            
                        }
                        _ => (),
                    }
                }
            }
            if exit {
                break;
            }
            translation = next_translation;
        }
        transform.translation = translation;
        kicked.0 *= 0.9;
        if kicked.0.length() < EPSILON {
            commands.entity(entity).remove::<Kicked>();
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Kicked(pub Vec2);

fn reflect_velocity(velocity: Vec2, normal: Vec2) -> Vec2 {
    velocity - 2.0 * velocity.dot(normal) * normal
}

fn get_wall_normal(position: IVec2, map: &Map) -> Vec2 {
    let mut normal = Vec2::ZERO;
    if map.contains_key(&(position + IVec2::X)) {
        normal += Vec2::NEG_X;
    }
    if map.contains_key(&(position - IVec2::X)) {
        normal += Vec2::X;
    }
    if map.contains_key(&(position + IVec2::Y)) {
        normal += Vec2::NEG_Y;
    }
    if map.contains_key(&(position - IVec2::Y)) {
        normal += Vec2::Y;
    }

    normal.normalize_or_zero()
}
