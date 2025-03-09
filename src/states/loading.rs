use bevy::prelude::*;

use crate::prelude::*;

use super::AppStates;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppStates::Loading), spawn_loading_screen)
        .add_systems(
            Update,
            continue_to_title_screen.run_if(all_assets_loaded.and(in_state(AppStates::Loading))),
        );
}

fn spawn_loading_screen(mut commands: Commands) {
    commands
        .ui_root()
        .insert(StateScoped(AppStates::Loading))
        .with_children(|children| {
            children.label("Loading...").insert(Node {
                justify_content: JustifyContent::Center,
                ..default()
            });
        });
}

fn continue_to_title_screen(mut next_screen: ResMut<NextState<AppStates>>) {
    next_screen.set(AppStates::Gameplay);
}

fn all_assets_loaded(resource_handles: Res<ResourceHandles>) -> bool {
    resource_handles.is_all_done()
}
