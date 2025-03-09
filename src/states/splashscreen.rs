use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

use super::AppStates;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppStates::Splashscreen), startup);
}

fn startup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands
        .ui_root()
        .insert((
            Name::new("Splash Screen"),
            StateScoped(AppStates::Splashscreen),
        ))
        .with_child((
            Name::new("Splash Image"),
            ImageNode {
                image: asset_server.load_with_settings(
                    SPLASH_SCREEN_IMAGE_FILE_PATH,
                    |settings: &mut ImageLoaderSettings| settings.sampler = ImageSampler::nearest(),
                ),
                ..default()
            },
            ImageNodeFadeInOut::default(),
        ));
}
