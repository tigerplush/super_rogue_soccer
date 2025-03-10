use bevy::prelude::*;
use bevy_old_tv_shader::OldTvPlugin;

mod prelude;
mod states;

pub struct SuperRogueSoccerPlugin;

impl Plugin for SuperRogueSoccerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OldTvPlugin)
            .add_plugins((prelude::plugin, states::plugin));
    }
}
