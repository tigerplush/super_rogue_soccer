use std::collections::HashMap;

use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::transform;
use leafwing_input_manager::{plugin::InputManagerSystem, prelude::*};
use rand::Rng;

use crate::{
    AppSet, PostUpdateSet,
    actors::Team,
    entities::{Interactable, Map},
    states::*,
    to_ivec2,
    ui::LogEvent,
};

use super::{
    CurrentPlayer, PointerObject, PreviewPath, Sampler, Stats, Velocity,
    pathfinding::{self, CalculatedPath},
};

pub fn plugin(app: &mut App) {
    app.register_type::<CurrentActions>()
        .register_type::<Kicked>()
        .insert_resource(CurrentActions { actions: vec![] })
        .insert_resource(PreviewPath { path: vec![] })
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
            (
                calculate_ui_actions,
                calculate_current_actions.run_if(in_state(GameplayStates::PlayerTurn)),
            )
                .in_set(PostUpdateSet::Move)
                .run_if(in_state(AppState::Gameplay)),
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
    Header(Entity),
    EntityCommands(Vec<(String, String, bool)>),
    Command(String, String, bool),
}

fn calculate_ui_actions(
    map: Res<Map>,
    path: Res<PreviewPath>,
    pointer: Single<&Transform, With<PointerObject>>,
    current_player: Single<(Entity, &Stats, Option<&Claimed>), With<CurrentPlayer>>,
    mut commands: Commands,
) {
    let transform = pointer.into_inner();
    let target_position = to_ivec2(transform.translation);
    let mut actions = vec![];
    let (current_entity, stats, claimed_option) = current_player.into_inner();
    if claimed_option.is_some() {
        actions.push(PossibleAction::Command(
            "j".to_string(),
            "pass".to_string(),
            true,
        ));
    }
    if let Some(entities) = map.get(&target_position) {
        if entities
            .iter()
            .map(|(e, _)| *e)
            .collect::<Vec<Entity>>()
            .contains(&current_entity)
        {
            actions.push(PossibleAction::StatBlock(current_entity));
        }
    }
    let in_range = path.path.len() <= stats.ap + 1;
    actions.push(PossibleAction::Command(
        "f".to_string(),
        "walk".to_string(),
        in_range,
    ));
    if let Some(entities) = map.get(&target_position) {
        let mut sorted_vec = entities.clone();
        sorted_vec.sort_by_key(|(e, _)| if *e == current_entity { 0 } else { 1 });
        for (entity, interactable) in sorted_vec {
            if entity != current_entity {
                let mut entity_actions = vec![];
                if Interactable::Person == interactable {
                    actions.push(PossibleAction::StatBlock(entity));
                } else {
                    actions.push(PossibleAction::Header(entity));
                }
                entity_actions.push((
                    "g".to_string(),
                    "take control".to_string(),
                    in_range && claimed_option.is_none(),
                ));
                entity_actions.push(("h".to_string(), "kick".to_string(), in_range));
                match interactable {
                    Interactable::Person => {
                        entity_actions.push(("i".to_string(), "foul".to_string(), in_range));
                    }
                    _ => (),
                }
                actions.push(PossibleAction::EntityCommands(entity_actions));
            }
        }
    }
    if claimed_option.is_some() {
        actions.push(PossibleAction::Command(
            "j".to_string(),
            "pass".to_string(),
            true,
        ));
    }
    actions.push(PossibleAction::Command(
        "SPACE".to_string(),
        "skip".to_string(),
        true,
    ));
    commands.insert_resource(CurrentActions { actions });
}

fn calculate_current_actions(
    map: Res<Map>,
    path: Res<PreviewPath>,
    pointer: Single<&Transform, With<PointerObject>>,
    ability_slot: Single<&mut AbilitySlotMap>,
    current_player: Single<(Entity, &Stats, Option<&Claimed>), With<CurrentPlayer>>,
) {
    let mut slot_map = ability_slot.into_inner();
    slot_map.clear();

    let transform = pointer.into_inner();
    let target_position = to_ivec2(transform.translation);
    let (current_entity, stats, claimed_option) = current_player.into_inner();
    let in_range = path.path.len() <= stats.ap + 1;

    slot_map.insert(Slots::Ability0, PlayerAbilities::Skip);
    if in_range {
        slot_map.insert(Slots::Ability1, PlayerAbilities::Walk);
    }
    if let Some(entities) = map.get(&target_position) {
        let mut sorted_vec = entities.clone();
        sorted_vec.sort_by_key(|(e, _)| if *e == current_entity { 0 } else { 1 });
        for (entity, interactable) in sorted_vec {
            if entity != current_entity {
                if in_range {
                    slot_map.insert(Slots::Ability2, PlayerAbilities::TakeControl(entity));
                    slot_map.insert(Slots::Ability3, PlayerAbilities::Kick(entity));
                }
                match interactable {
                    Interactable::Person => {
                        slot_map.insert(Slots::Ability4, PlayerAbilities::Foul(entity));
                    }
                    _ => (),
                }
            }
        }
    }
    if claimed_option.is_some() {
        slot_map.insert(
            Slots::Ability4,
            PlayerAbilities::Pass(claimed_option.unwrap().0),
        );
    }
}

#[derive(Actionlike, Reflect, Clone, Hash, Eq, PartialEq, Debug, Copy)]
pub enum PlayerAbilities {
    Walk,
    TakeControl(Entity),
    Kick(Entity),
    Foul(Entity),
    Pass(Entity),
    Skip,
}

#[derive(Actionlike, Reflect, Clone, Hash, Eq, PartialEq, Debug, Copy)]
pub enum Slots {
    Ability0,
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
        [
            Ability0, Ability1, Ability2, Ability3, Ability4, Ability5, Ability6,
        ]
        .iter()
        .copied()
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct AbilitySlotMap {
    map: HashMap<Slots, PlayerAbilities>,
}

pub fn setup_slotmap(mut commands: Commands) {
    commands.spawn((
        Name::from("Player Controls"),
        InputMap::new([
            (Slots::Ability0, KeyCode::Space),
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

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct ActionQueue(pub Vec<Action>);

#[derive(Reflect)]
pub enum Action {
    MoveTo(Vec3),
    Kick(Entity),
    TakeControl(Entity),
    Foul(Entity),
    /// Which entity has to be passed where
    Pass(Entity, Vec3),
    DefendGoal,
    SkipTurn,
    EndTurn(Team),
    Advance,
    PassDown,
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
                PlayerAbilities::Skip => {
                    queue.0.push(Action::EndTurn(Team::Enemy));
                    queue.0.push(Action::SkipTurn);
                }
            }
        }
    }
}

fn process_actions(
    mut sampler: ResMut<Sampler>,
    time: Res<Time<Fixed>>,
    map: Res<Map>,
    mut query: Query<(
        Entity,
        &Name,
        &Transform,
        &mut ActionQueue,
        &Velocity,
        Option<&CalculatedPath>,
        &Team,
    )>,
    interactables: Query<(Entity, &Transform, &Interactable, &Name)>,
    stat_query: Query<&Stats>,
    mut events: EventWriter<LogEvent>,
    mut next: ResMut<NextState<GameplayStates>>,
    mut commands: Commands,
) {
    for (entity, name, transform, mut queue, velocity, path_option, team) in &mut query {
        if path_option.is_some() {
            continue;
        }

        if let Some(action) = queue.0.pop() {
            match action {
                Action::MoveTo(target) => {
                    let Ok(path) = pathfinding::calculate_path(transform.translation, target, &map)
                    else {
                        events.send(LogEvent(format!("{} can't find a way to the target", name)));
                        continue;
                    };
                    commands
                        .entity(entity)
                        .insert((Velocity(Vec2::ZERO), CalculatedPath::new(path, 0.25)));
                }
                Action::Kick(target) => {
                    let (_, _, interactable, target_name) = interactables.get(target).unwrap();
                    let article = match interactable {
                        Interactable::Ball => "the",
                        Interactable::Person => {
                            let stat = stat_query.get(target).unwrap();
                            let random = sampler.0.random_range(0.0..=1.0);
                            if random < stat.defense {
                                events.send(LogEvent(format!(
                                    "{} tried to kicked {}, but {} evaded",
                                    name, target_name, target_name
                                )));
                                continue;
                            }
                            ""
                        }
                        _ => "",
                    };
                    let stats = stat_query.get(entity).unwrap();
                    commands
                        .entity(target)
                        .insert(Kicked(velocity.0 * stats.kick_strength));
                    events.send(LogEvent(format!(
                        "{} kicked {}{}",
                        name, article, target_name
                    )));
                }
                Action::TakeControl(target) => {
                    commands.entity(entity).insert(Claimed(target));
                    commands.entity(target).insert(ClaimedBy(entity));
                    let (_, _, interactable, target_name) = interactables.get(target).unwrap();
                    let article = if &Interactable::Ball == interactable {
                        "the "
                    } else {
                        ""
                    };
                    events.send(LogEvent(format!(
                        "{} takes control of {}{}",
                        name, article, target_name
                    )));
                }
                Action::Foul(target) => {}
                Action::Pass(target, target_position) => {
                    let stats = stat_query.get(entity).unwrap();
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
                    events.send(LogEvent(format!("{} is passing the ball", name)));
                }
                Action::DefendGoal => {
                    let (sum, count) = interactables
                        .iter()
                        .filter(|(_, _, interactable, _)| {
                            *interactable == &Interactable::Goal(*team)
                        })
                        .map(|(_, transform, _, _)| transform.translation)
                        .fold((Vec3::ZERO, 0), |(acc, count), v| (acc + v, count + 1));

                    let midpoint = sum / count as f32;
                    let ball = interactables
                        .iter()
                        .filter(|(_, _, interactable, _)| *interactable == &Interactable::Ball)
                        .map(|(_, transform, _, _)| transform.translation)
                        .next()
                        .unwrap();
                    let direction = (ball - midpoint).normalize_or_zero();
                    // radius of goal is 7 tiles times 8.0 pixel per tiles
                    const RADIUS: f32 = 7.0 * 8.0;

                    queue.0.push(Action::MoveTo(midpoint + direction * RADIUS));
                    events.send(LogEvent(format!("{} moves to defend the goal", name)));
                }
                Action::SkipTurn => {
                    events.send(LogEvent(format!("{} is skipping their turn", name)));
                }
                Action::EndTurn(next_team) => {
                    next.set(GameplayStates::Banner(next_team));
                }
                Action::Advance => {
                    let closest = interactables
                        .iter()
                        .filter(|(_, _, interactable, _)| {
                            *interactable == &Interactable::Goal(Team::Player)
                        })
                        .map(|(_, transform, _, _)| transform)
                        .min_by(|a, b| {
                            let dist_a = (a.translation - transform.translation).length();
                            let dist_b = (b.translation - transform.translation).length();
                            dist_a
                                .partial_cmp(&dist_b)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .unwrap();
                    info!("trying to advance to {}", closest.translation);
                    queue
                        .0
                        .push(Action::MoveTo(closest.translation + Vec3::X * 8.0));
                }
                Action::PassDown => {
                    let closest = interactables
                        .iter()
                        .filter(|(_, _, interactable, _)| {
                            *interactable == &Interactable::Goal(Team::Enemy)
                        })
                        .map(|(_, transform, _, _)| transform)
                        .min_by(|a, b| {
                            let dist_a = (a.translation - transform.translation).length();
                            let dist_b = (b.translation - transform.translation).length();
                            dist_a
                                .partial_cmp(&dist_b)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .unwrap();
                    let target = interactables
                        .iter()
                        .filter(|(_, _, interactable, _)| *interactable == &Interactable::Ball)
                        .map(|(entity, _, _, _)| entity)
                        .next()
                        .unwrap();
                    queue.0.push(Action::Pass(target, closest.translation));
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
    mut sampler: ResMut<Sampler>,
    time: Res<Time>,
    map: Res<Map>,
    current_player: Option<Single<Entity, With<CurrentPlayer>>>,
    mut query: Query<(&Name, &mut Transform, &mut Kicked, Entity)>,
    interactables: Query<(&Name, &Stats), With<Interactable>>,
    mut commands: Commands,
    mut events: EventWriter<LogEvent>,
) {
    const EPSILON: f32 = 0.1;
    let current_entity = if let Some(current) = current_player {
        current.into_inner()
    } else {
        Entity::PLACEHOLDER
    };
    for (name, mut transform, mut kicked, entity) in &mut query {
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
                for (next_entity, interactable) in entities {
                    if *next_entity == current_entity && *next_entity == entity {
                        continue;
                    }
                    match interactable {
                        &Interactable::Wall => {
                            let normal = get_wall_normal(current_position, &map);
                            kicked.0 = reflect_velocity(kicked.0, normal);
                            exit = true;
                        }
                        &Interactable::Person => {
                            let (player, stats) = interactables.get(*next_entity).unwrap();
                            let random = sampler.0.random_range(0.0..=1.0);
                            if random < stats.defense {
                                kicked.0 = Vec2::ZERO;
                                exit = true;
                                events.send(LogEvent(format!(
                                    "{} blocked incoming {}",
                                    player, name
                                )));
                            }
                        }
                        &Interactable::Goal(team) => {
                            let normal = get_wall_normal(current_position, &map);
                            let is_goal = match team {
                                Team::Enemy => normal.x < 0.0,
                                Team::Player => normal.x > 0.0,
                            };
                            kicked.0 = if is_goal {
                                // count goal
                                // reset ball?
                                Vec2::ZERO
                            } else {
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
pub struct ClaimedBy(pub Entity);

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
