use std::{cmp::Reverse, collections::HashMap};

use bevy::prelude::*;
use priority_queue::PriorityQueue;

use crate::{AppSet, to_ivec2, to_world};

use super::PointerIsDirty;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            tick_path.in_set(AppSet::TickTimers),
            follow_path.in_set(AppSet::Update),
        ),
    );
}

#[derive(Component)]
pub struct CalculatedPath {
    path: Vec<IVec2>,
    timer: Timer,
    current: usize,
}

impl CalculatedPath {
    pub fn new(path: Vec<IVec2>, duration: f32) -> Self {
        CalculatedPath {
            path,
            timer: Timer::from_seconds(duration, TimerMode::Repeating),
            current: 0,
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

pub fn calculate_path(start: Vec3, target: Vec3) -> Result<Vec<IVec2>, PathError> {
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
            let new_cost = cost_so_far.get(&current_coordinates).unwrap() + 1;
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
    mut query: Query<(&mut Transform, &mut CalculatedPath, Entity)>,
    mut commands: Commands,
) {
    for (mut transform, mut path, entity) in &mut query {
        if !path.timer.finished() {
            continue;
        }

        if let Some(next) = path.next() {
            transform.translation = (to_world(next), transform.translation.z).into()
        } else {
            commands.entity(entity).remove::<CalculatedPath>();
        }
        dirt.0 = true;
    }
}
