use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

use super::AppStates;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppStates::Splashscreen),
        (startup, insert_splash_timer),
    )
    .add_systems(
        Update,
        (tick_splash_timer, check_splash_timer).run_if(in_state(AppStates::Splashscreen)),
    )
    .add_systems(OnExit(AppStates::Splashscreen), remove_splash_timer);
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
            Node {
                width: Val::Percent(100.0),
                ..default()
            },
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

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
struct SplashTimer(Timer);

impl Default for SplashTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(SPLASH_DURATION_SECS, TimerMode::Once))
    }
}

fn check_splash_timer(timer: ResMut<SplashTimer>, mut next_screen: ResMut<NextState<AppStates>>) {
    if timer.0.just_finished() {
        next_screen.set(AppStates::Loading);
    }
}

fn insert_splash_timer(mut commands: Commands) {
    commands.init_resource::<SplashTimer>();
}

fn remove_splash_timer(mut commands: Commands) {
    commands.remove_resource::<SplashTimer>();
}

fn tick_splash_timer(time: Res<Time>, mut timer: ResMut<SplashTimer>) {
    timer.0.tick(time.delta());
}
