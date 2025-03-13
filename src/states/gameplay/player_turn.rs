use std::cmp::Reverse;

use bevy::{color::palettes::css::GREEN, prelude::*};
use priority_queue::PriorityQueue;

use crate::states::AppSet;

use super::*;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameplayStates::PlayerTurn), set_pointer_dirty)
        .add_systems(
            Update,
            (
                (check_commands, calculate_current_path)
                    .run_if(pointer_is_dirty)
                    .in_set(AppSet::UpdateUi),
                preview_path,
            )
                .run_if(in_state(GameplayStates::PlayerTurn)),
        )
        .add_systems(OnExit(GameplayStates::PlayerTurn), clean_up);
}

fn set_pointer_dirty(mut pointer_location: ResMut<PointerLocation>) {
    pointer_location.old += IVec2::ONE;
}

const POSSIBLE_KEYS: [KeyCode; 8] = [
    KeyCode::KeyF,
    KeyCode::KeyG,
    KeyCode::KeyH,
    KeyCode::KeyJ,
    KeyCode::KeyK,
    KeyCode::KeyL,
    KeyCode::KeyV,
    KeyCode::KeyB,
];

fn check_commands(
    entity_positions: Res<EntityPositions>,
    pointer: Single<&Transform, With<PointerObject>>,
    current_player: Single<Entity, With<CurrentPlayer>>,
    mut possible_action: ResMut<DisplayActions>,
) {
    possible_action.0.clear();

    let pointer_position = pointer.as_ivec2();
    let current_entity = current_player.into_inner();
    let mut keys = POSSIBLE_KEYS.iter();
    if let Some(entities) = entity_positions.0.get(&pointer_position) {
        let mut sorted_vec = entities.clone();
        sorted_vec.sort_by_key(|(e, _)| if *e == current_entity { 0 } else { 1 });
        for (entity, interactable) in sorted_vec {
            if entity == current_entity {
                possible_action
                    .0
                    .push(DisplayAction::StatBlock(current_entity));
            } else {
                match interactable {
                    Interactables::Ball => {
                        let actions = vec![
                            (*keys.next().unwrap(), PlayerCommand::Walk, true),
                            (*keys.next().unwrap(), PlayerCommand::TakeControl, true),
                            (*keys.next().unwrap(), PlayerCommand::Kick, true),
                        ];
                        possible_action
                            .0
                            .push(DisplayAction::EntityAction(entity, actions));
                    }
                    Interactables::Person => {
                        let actions = vec![
                            (*keys.next().unwrap(), PlayerCommand::Walk, true),
                            (*keys.next().unwrap(), PlayerCommand::TakeControl, true),
                            (*keys.next().unwrap(), PlayerCommand::Kick, true),
                        ];
                        possible_action
                            .0
                            .push(DisplayAction::EntityAction(entity, actions));
                    }
                    _ => (),
                }
            }
        }
    }

    possible_action.0.push(DisplayAction::SingleAction((
        KeyCode::Space,
        PlayerCommand::Skip,
        true,
    )));
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct PathPreview(Vec<IVec2>);

fn calculate_current_path(
    entity_positions: Res<EntityPositions>,
    pointer: Single<&Transform, With<PointerObject>>,
    current_player: Single<&Transform, With<CurrentPlayer>>,
    mut commands: Commands,
) {
    let path = calculate_path(
        current_player.as_ivec2(),
        pointer.as_ivec2(),
        &entity_positions,
    );
    commands.insert_resource(PathPreview(path));
}

fn preview_path(path_preview: Option<Res<PathPreview>>, mut gizmos: Gizmos) {
    let Some(path) = path_preview else {
        return;
    };

    for (index, window) in path.0.windows(2).enumerate() {
        gizmos.arrow_2d(window[0].as_vec2() * TILE_SIZE, window[1].as_vec2() * TILE_SIZE, GREEN);
    }
}

fn clean_up(mut possible_action: ResMut<DisplayActions>, mut commands: Commands) {
    possible_action.0.clear();

    commands.remove_resource::<PathPreview>();
}

const DIRECTIONS: [IVec2; 8] = [
    IVec2::X,
    IVec2::ONE,
    IVec2::Y,
    IVec2::new(-1, 1),
    IVec2::NEG_X,
    IVec2::NEG_ONE,
    IVec2::NEG_Y,
    IVec2::new(1, -1),
];

fn calculate_path(start: IVec2, target: IVec2, map: &EntityPositions) -> Vec<IVec2> {
    let mut frontier: PriorityQueue<IVec2, Reverse<usize>> = PriorityQueue::new();
    let mut cost_so_far: HashMap<IVec2, usize> = HashMap::new();
    let mut came_from: HashMap<IVec2, Option<IVec2>> = HashMap::new();

    frontier.push(start, Reverse(0));
    cost_so_far.insert(start, 0);
    loop {
        let Some((current_coordinates, _current_priority)) = frontier.pop() else {
            return vec![];
        };

        if current_coordinates == target {
            let mut path = vec![];
            let mut next = target;
            path.push(target);
            while let Some(point_option) = came_from.get(&next) {
                if let Some(point) = point_option {
                    path.push(*point);
                    next = *point;
                } else {
                    break;
                }
            }
            path.reverse();
            return path;
        }

        for direction in DIRECTIONS {
            let neighbor = current_coordinates + direction;
            let mut cost = 1;
            if let Some(next) = map.0.get(&neighbor) {
                for (_, interactable) in next {
                    cost += match interactable {
                        &Interactables::Person => 10,
                        &Interactables::Goal(_) => 100,
                        &Interactables::Wall => 100,
                        _ => 0,
                    };
                }
            }
            let new_cost = cost_so_far.get(&current_coordinates).unwrap() + cost;
            let current_cost = cost_so_far.get(&neighbor);
            if current_cost.is_none() || new_cost < *current_cost.unwrap() {
                cost_so_far.insert(neighbor, new_cost);
                let priority = new_cost + 1 + neighbor.distance_squared(target) as usize;
                frontier.push(neighbor, Reverse(priority));
                came_from.insert(neighbor, Some(current_coordinates));
            }
        }
    }
}
