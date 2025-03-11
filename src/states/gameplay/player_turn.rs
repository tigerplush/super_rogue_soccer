use bevy::prelude::*;

use crate::states::AppSet;

use super::*;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameplayStates::PlayerTurn), set_pointer_dirty)
        .add_systems(
            Update,
            (check_commands
                .run_if(pointer_is_dirty)
                .in_set(AppSet::UpdateUi),)
                .run_if(in_state(GameplayStates::PlayerTurn)),
        )
        .add_systems(OnExit(GameplayStates::PlayerTurn), clean_up);
}

fn set_pointer_dirty(mut pointer_location: ResMut<PointerLocation>) {
    pointer_location.old += IVec2::ONE;
    info!("pointer is now {:?}", pointer_location);
}

fn check_commands(mut possible_action: ResMut<DisplayActions>) {
    possible_action.0.clear();
    possible_action.0.push(DisplayAction::SingleAction((
        KeyCode::Space,
        PlayerCommand::Skip,
        true,
    )));
}

fn clean_up(mut possible_action: ResMut<DisplayActions>) {
    possible_action.0.clear();
}
