use bevy::prelude::*;

mod gameplay;
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

pub fn plugin(app: &mut App) {
    app.init_state::<AppState>();

    app.enable_state_scoped_entities::<AppState>();
    app.add_plugins((splash::plugin, loading::plugin, gameplay::plugin));
}
