use bevy::{ecs::system::RunSystemOnce, prelude::*};

use super::AppState;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, advance_state.run_if(in_state(AppState::Splash)));
}

fn advance_state(world: &mut World) {
    world.run_system_once(set_state).unwrap();
}

fn set_state(mut next: ResMut<NextState<AppState>>) {
    next.set(AppState::Loading);
}
