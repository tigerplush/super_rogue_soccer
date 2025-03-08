use std::{cmp::Reverse, collections::HashMap};

use bevy::prelude::*;
use priority_queue::PriorityQueue;

use crate::{
    AppSet,
    entities::{Interactable, Map},
    to_ivec2, to_world,
    ui::LogEvent,
};

use super::{PointerIsDirty, Stats, Velocity};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            tick_path.in_set(AppSet::TickTimers),
            follow_path.in_set(AppSet::Update),
        ),
    )
    // .add_observer(message_move_end)
    ;
}

#[derive(Component)]
pub struct CalculatedPath {
    pub path: Vec<IVec2>,
    timer: Timer,
    current: usize,
}

impl CalculatedPath {
    pub fn new(path: Vec<IVec2>, duration: f32) -> Self {
        CalculatedPath {
            path,
            timer: Timer::from_seconds(duration, TimerMode::Repeating),
            current: 1,
        }
    }

    pub fn next(&mut self) -> Option<IVec2> {
        if self.current < self.path.len() {
            let next = Some(self.path[self.current]);
            self.current += 1;
            return next;
        }
        None
    }
}

pub enum PathError {
    Failed,
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

pub fn calculate_path(start: Vec3, target: Vec3, map: &Map) -> Result<Vec<IVec2>, PathError> {
    let start_position = to_ivec2(start);
    let target_position = to_ivec2(target);

    let mut frontier: PriorityQueue<IVec2, Reverse<usize>> = PriorityQueue::new();
    let mut cost_so_far: HashMap<IVec2, usize> = HashMap::new();
    let mut came_from: HashMap<IVec2, Option<IVec2>> = HashMap::new();

    frontier.push(start_position, Reverse(0));
    cost_so_far.insert(start_position, 0);
    loop {
        let Some((current_coordinates, _current_priority)) = frontier.pop() else {
            return Err(PathError::Failed);
        };

        if current_coordinates == target_position {
            let mut path = vec![];
            let mut next = target_position;
            path.push(target_position);
            while let Some(point_option) = came_from.get(&next) {
                if let Some(point) = point_option {
                    path.push(*point);
                    next = *point;
                } else {
                    break;
                }
            }
            path.reverse();
            return Ok(path);
        }

        for direction in DIRECTIONS {
            let neighbor = current_coordinates + direction;
            let mut cost = 1;
            if let Some(next) = map.get(&neighbor) {
                for (_, interactable) in next {
                    cost += match interactable {
                        &Interactable::Person => 10,
                        _ => 0,
                    };
                }
            }
            let new_cost = cost_so_far.get(&current_coordinates).unwrap() + cost;
            let current_cost = cost_so_far.get(&neighbor);
            if current_cost.is_none() || new_cost < *current_cost.unwrap() {
                cost_so_far.insert(neighbor, new_cost);
                let priority = new_cost + 1 + neighbor.distance_squared(target_position) as usize;
                frontier.push(neighbor, Reverse(priority));
                came_from.insert(neighbor, Some(current_coordinates));
            }
        }
    }
}

fn tick_path(time: Res<Time>, mut query: Query<&mut CalculatedPath>) {
    for mut path in &mut query {
        path.timer.tick(time.delta());
    }
}

fn follow_path(
    mut dirt: ResMut<PointerIsDirty>,
    mut query: Query<(
        &mut Transform,
        &mut CalculatedPath,
        Option<&mut Velocity>,
        Entity,
        &mut Stats,
    )>,
    mut commands: Commands,
) {
    for (mut transform, mut path, velocity_option, entity, mut stats) in &mut query {
        if !path.timer.finished() {
            continue;
        }
        let next_option = path.next();
        if let Some(next) = next_option {
            if stats.ap > 0 {
                let previous = transform.translation;
                transform.translation = to_world(next).extend(transform.translation.z);
                if let Some(mut velocity) = velocity_option {
                    velocity.0 += (transform.translation - previous).truncate();
                }
                stats.ap -= 1;
            }
        }

        if next_option.is_none() || stats.ap == 0 {
            commands.entity(entity).remove::<CalculatedPath>();
        }
        dirt.0 = true;
    }
}

#[allow(dead_code)]
fn message_move_end(
    trigger: Trigger<OnRemove, CalculatedPath>,
    query: Query<&Name>,
    mut events: EventWriter<LogEvent>,
) {
    if let Ok(name) = query.get(trigger.entity()) {
        events.send(LogEvent(format!("{} stopped moving", name)));
    }
}
