use bevy::prelude::*;
#[cfg(feature = "debug")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use super_rogue_soccer::SuperRogueSoccerPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    #[cfg(feature = "debug")]
    app.add_plugins(WorldInspectorPlugin::new());
    app.add_plugins(SuperRogueSoccerPlugin).run();
}
