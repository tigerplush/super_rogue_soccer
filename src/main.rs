use bevy::prelude::*;
use super_rogue_soccer::SuperRogueSoccerPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SuperRogueSoccerPlugin)
        .run();
}
