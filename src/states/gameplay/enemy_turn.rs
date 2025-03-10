use bevy::prelude::*;

use super::GameplayStates;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameplayStates::EnemyTurn), dummy)
        .add_systems(Update, dummy.run_if(in_state(GameplayStates::EnemyTurn)))
        .add_systems(OnExit(GameplayStates::EnemyTurn), dummy);
}

fn dummy() {}
