use bevy::prelude::*;

use super::GameplayStates;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameplayStates::PlayerTurn), dummy)
        .add_systems(Update, dummy.run_if(in_state(GameplayStates::PlayerTurn)))
        .add_systems(OnExit(GameplayStates::PlayerTurn), dummy);
}

fn dummy() {}
