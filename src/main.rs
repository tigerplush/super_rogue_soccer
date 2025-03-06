use bevy::{
    asset::AssetMetaCheck,
    audio::{AudioPlugin, Volume},
    prelude::*,
};
#[cfg(feature = "debug")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use super_rogue_soccer::SuperRogueSoccerPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Window {
                    title: "Super Rogue Soccer".to_string(),
                    canvas: Some("#bevy".to_string()),
                    resize_constraints: WindowResizeConstraints {
                        max_height: 837.0,
                        max_width: 1280.0,
                        min_height: 837.0,
                        min_width: 1280.0,
                    },
                    ..default()
                }
                .into(),
                ..default()
            })
            .set(AssetPlugin {
                // Wasm builds will check for meta files (that don't exist) if this isn't set.
                // This causes errors and even panics on web build on itch.
                // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(AudioPlugin {
                global_volume: GlobalVolume {
                    volume: Volume::new(0.3),
                },
                ..default()
            }),
    );
    #[cfg(feature = "debug")]
    app.add_plugins(WorldInspectorPlugin::new());
    app.add_plugins(SuperRogueSoccerPlugin).run();
}
