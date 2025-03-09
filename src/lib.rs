use bevy::prelude::*;

mod prelude;
mod states;

pub struct SuperRogueSoccerPlugin;

impl Plugin for SuperRogueSoccerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((prelude::plugin, states::plugin));
    }
}
