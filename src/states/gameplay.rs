use bevy::prelude::*;

use crate::{actors, map};

use super::AppState;

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::Gameplay),
        (actors::startup, map::spawn_field),
    );
}
