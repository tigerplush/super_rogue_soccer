use std::collections::HashMap;

use bevy::prelude::*;
use leafwing_input_manager::{plugin::InputManagerSystem, prelude::*};

use crate::{
    actors::Team, entities::{Interactable, Map}, states::*, to_ivec2, AppSet, PostUpdateSet
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
            (report_abilities_used, process_actions, process_control).in_set(AppSet::Update),
        )
        .add_systems(FixedUpdate, process_kick)
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
    pub actions: Vec<PossibleAction>,
}

#[derive(Reflect)]
pub enum PossibleAction {
    StatBlock(Entity),
    EntityCommands(Entity, Vec<(String, String, bool)>),
    Command(String, String, bool),
}

fn calculate_current_actions(
    map: Res<Map>,
    path: Res<PreviewPath>,
    pointer: Single<&Transform, With<PointerObject>>,
    ability_slot: Single<&mut AbilitySlotMap>,
    current_player: Single<(Entity, &Stats, Option<&Claimed>), With<CurrentPlayer>>,
    mut commands: Commands,
) {
    let mut slot_map = ability_slot.into_inner();
    slot_map.clear();

    let transform = pointer.into_inner();
    let target_position = to_ivec2(transform.translation);
    let mut actions = vec![];
    let (current_entity, stats, claimed_option) = current_player.into_inner();
    let in_range = path.path.len() <= stats.ap;

    if let Some(entities) = map.get(&target_position) {
        if entities.iter().map(|(e, _)| *e).collect::<Vec<Entity>>().contains(&current_entity) {
            actions.push(PossibleAction::StatBlock(current_entity));
        }
    }
    actions.push(PossibleAction::Command(
        "f".to_string(),
        "walk".to_string(),
        in_range,
    ));
    if in_range {
        slot_map.insert(Slots::Ability1, PlayerAbilities::Walk);
    }
    if let Some(entities) = map.get(&target_position) {
        let mut sorted_vec = entities.clone();
        sorted_vec.sort_by_key(|(e, _)| if *e == current_entity { 0 } else { 1 });
        for (entity, interactable) in sorted_vec {
            if entity != current_entity {
                let mut entity_actions = vec![];
                entity_actions.push((
                    "g".to_string(),
                    "take control".to_string(),
                    in_range && claimed_option.is_none(),
                ));
                entity_actions.push(("h".to_string(), "kick".to_string(), in_range));
                if in_range {
                    slot_map.insert(Slots::Ability2, PlayerAbilities::TakeControl(entity));
                    slot_map.insert(Slots::Ability3, PlayerAbilities::Kick(entity));
                }
                match interactable {
                    Interactable::Person => {
                        entity_actions.push(("i".to_string(), "foul".to_string(), in_range));
                        slot_map.insert(Slots::Ability4, PlayerAbilities::Foul(entity));
                    }
                    _ => (),
                }
                actions.push(PossibleAction::EntityCommands(entity, entity_actions));
            }
        }
    }
    if claimed_option.is_some() {
        actions.push(PossibleAction::Command(
            "j".to_string(),
            "pass".to_string(),
            true,
        ));
        slot_map.insert(
            Slots::Ability4,
            PlayerAbilities::Pass(claimed_option.unwrap().0),
        );
    }
    commands.insert_resource(CurrentActions { actions });
}

#[derive(Actionlike, Reflect, Clone, Hash, Eq, PartialEq, Debug, Copy)]
pub enum PlayerAbilities {
    Walk,
    TakeControl(Entity),
    Kick(Entity),
    Foul(Entity),
    Pass(Entity),
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
    /// Which entity has to be passed where
    Pass(Entity, Vec3),
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
            info!("pressed {:?}", ability);
            match ability {
                PlayerAbilities::Walk => {
                    queue.0.push(Action::MoveTo(target_transform.translation));
                }
                PlayerAbilities::Kick(target) => {
                    info!("trying to kick {}", target);
                    queue.0.push(Action::Kick(target));
                    queue.0.push(Action::MoveTo(target_transform.translation));
                }
                PlayerAbilities::TakeControl(target) => {
                    queue.0.push(Action::TakeControl(target));
                    queue.0.push(Action::MoveTo(target_transform.translation));
                }
                PlayerAbilities::Foul(target) => {
                    queue.0.push(Action::Foul(target));
                    queue.0.push(Action::MoveTo(target_transform.translation));
                }
                PlayerAbilities::Pass(target) => {
                    queue
                        .0
                        .push(Action::Pass(target, target_transform.translation));
                }
            }
        }
    }
}

fn process_actions(
    time: Res<Time<Fixed>>,
    map: Res<Map>,
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
                    let Ok(path) = pathfinding::calculate_path(transform.translation, target, &map)
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
                    info!(
                        "kicking, {} should now have {} velocity",
                        target,
                        velocity.0 * stats.kick_strength
                    );
                }
                Action::TakeControl(target) => {
                    commands.entity(entity).insert(Claimed(target));
                    commands.entity(target).insert(ClaimedBy(entity));
                }
                Action::Foul(target) => {}
                Action::Pass(target, target_position) => {
                    let velocity = calculate_kick_velocity(
                        stats.passing_skill,
                        transform.translation.truncate(),
                        target_position.truncate(),
                        time.delta_secs(),
                        velocity.0,
                    );
                    commands
                        .entity(target)
                        .insert(Kicked(velocity))
                        .remove::<ClaimedBy>();
                    commands.entity(entity).remove::<Claimed>();
                }
            }
        }
    }
}

pub fn calculate_kick_velocity(
    pass_distance: f32,
    ball_position: Vec2,
    target_position: Vec2,
    time: f32,
    velocity: Vec2,
) -> Vec2 {
    let diff = target_position - ball_position;
    let direction = diff.normalize_or_zero();
    let actual_distance = diff.length() / 8.0;

    let speed = (pass_distance / time).min(actual_distance / time);
    let kick_velocity = direction * speed + velocity * 0.5;
    kick_velocity
}

fn process_kick(
    time: Res<Time>,
    map: Res<Map>,
    mut query: Query<(&mut Transform, &mut Kicked, Entity)>,
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
                for (_, interactable) in entities {
                    match interactable {
                        &Interactable::Wall => {
                            let normal = get_wall_normal(current_position, &map);
                            kicked.0 = reflect_velocity(kicked.0, normal);
                            exit = true;
                        }
                        &Interactable::Person => {}
                        &Interactable::Goal(team) => {
                            let normal = get_wall_normal(current_position, &map);
                            let is_goal= match team {
                                Team::Enemy => normal.x < 0.0,
                                Team::Player => normal.x > 0.0,
                            };
                            kicked.0 = if is_goal {
                                // count goal
                                // reset ball?
                                Vec2::ZERO
                            }
                            else {
                                reflect_velocity(kicked.0, normal)
                            };
                            exit = true;
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

#[derive(Component, Reflect)]
#[reflect(Component)]
struct ClaimedBy(Entity);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Claimed(Entity);

fn process_control(
    mut query: Query<(&mut Transform, &ClaimedBy), Without<Claimed>>,
    transforms: Query<&Transform, With<Claimed>>,
) {
    for (mut transform, claim) in &mut query {
        let parent = transforms.get(claim.0).unwrap();
        transform.translation = parent.translation;
    }
}
