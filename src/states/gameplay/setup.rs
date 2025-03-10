use super::GameplayStates;
use crate::prelude::*;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameplayStates::Setup), (map::spawn, actors::spawn))
        .add_systems(Update, advance_state.run_if(in_state(GameplayStates::Setup)))
        .add_systems(OnExit(GameplayStates::Setup), clean_up);
}

fn advance_state(mut next: ResMut<NextState<GameplayStates>>) {
    next.set(GameplayStates::PlayerTurn);
}

fn clean_up() {}
