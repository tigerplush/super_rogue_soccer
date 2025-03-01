use bevy::prelude::*;

use crate::{asset_tracking::ResourceHandles, theme::prelude::*};

use super::AppState;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Loading), spawn_loading_screen);
    app.add_systems(
        Update,
        continue_to_title_screen.run_if(in_state(AppState::Loading).and(all_assets_loaded)),
    );
}

fn spawn_loading_screen(mut commands: Commands) {
    commands
        .ui_root()
        .insert(StateScoped(AppState::Loading))
        .with_children(|children| {
            children.label("Loading...").insert(Node {
                justify_content: JustifyContent::Center,
                ..default()
            });
        });
}

fn continue_to_title_screen(mut next_screen: ResMut<NextState<AppState>>) {
    next_screen.set(AppState::Gameplay);
}

fn all_assets_loaded(resource_handles: Res<ResourceHandles>) -> bool {
    resource_handles.is_all_done()
}
