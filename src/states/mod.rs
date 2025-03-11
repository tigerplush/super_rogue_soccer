use bevy::prelude::*;
use bevy_old_tv_shader::OldTvSettings;

use crate::prelude::{apply_image_node_fades, tick_image_node_fades};

mod gameplay;
mod loading;
mod splashscreen;

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSet {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    UpdateMovement,
    UpdateUi,
}

/// Adds all game relevant systems for different game states.
/// Each state is self-contained and should not export anything else.
/// Each state should also only apply systems for itself.
pub fn plugin(app: &mut App) {
    app.init_state::<AppStates>()
        .enable_state_scoped_entities::<AppStates>()
        .configure_sets(
            Update,
            (
                AppSet::TickTimers,
                AppSet::RecordInput,
                AppSet::UpdateMovement,
                AppSet::UpdateUi,
            )
                .chain(),
        )
        .add_plugins((splashscreen::plugin, loading::plugin, gameplay::plugin))
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                tick_image_node_fades.in_set(AppSet::TickTimers),
                apply_image_node_fades.in_set(AppSet::UpdateUi),
            ),
        );
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
    commands.spawn((
        Camera2d,
        Transform::from_xyz(172.0, -55.0, 0.0),
        OldTvSettings {
            screen_shape_factor: 0.1,
            rows: 720.0,
            brightness: 3.0,
            edges_transition_size: 0.025,
            channels_mask_min: 0.1,
        },
    ));
}
