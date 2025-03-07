use bevy::prelude::*;

use crate::actors::Team;

pub mod gameplay;
mod loading;
mod splash;

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum AppState {
    #[default]
    Splash,
    Loading,
    Title,
    Credits,
    Gameplay,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, SubStates)]
#[source(AppState = AppState::Gameplay)]
pub enum GameplayStates {
    PlayerTurn,
    EnemyTurn,
    Banner(Team),
}

impl Default for GameplayStates {
    fn default() -> Self {
        GameplayStates::Banner(Team::Player)
    }
}

pub fn plugin(app: &mut App) {
    app.init_state::<AppState>()
        .add_sub_state::<GameplayStates>();

    app.enable_state_scoped_entities::<AppState>();
    app.add_plugins((splash::plugin, loading::plugin, gameplay::plugin));
}
