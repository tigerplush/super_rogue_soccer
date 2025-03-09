use bevy::prelude::*;

mod loading;
mod splashscreen;

/// Adds all game relevant systems for different game states.
/// Each state is self-contained and should not export anything else.
/// Each state should also only apply systems for itself.
pub fn plugin(app: &mut App) {
    app.init_state::<AppStates>()
        .enable_state_scoped_entities::<AppStates>()
        .add_plugins((splashscreen::plugin, loading::plugin))
        .add_systems(Startup, startup);
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
enum AppStates {
    #[default]
    Splashscreen,
    Loading,
    Title,
    Credits,
    Gameplay,
}

fn startup(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(172.0, -55.0, 0.0)));
}
