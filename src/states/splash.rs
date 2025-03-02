use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    input::common_conditions::input_just_pressed,
    prelude::*,
};

use super::AppState;

use crate::{AppSet, theme::prelude::*};

pub fn plugin(app: &mut App) {
    app.register_type::<ImageNodeFadeInOut>()
        .register_type::<SplashTimer>()
        .add_systems(OnEnter(AppState::Splash), (startup, insert_splash_timer))
        .add_systems(
            Update,
            (
                tick_fade_in_out.in_set(AppSet::TickTimers),
                apply_fade_in_out.in_set(AppSet::Update),
            )
                .run_if(in_state(AppState::Splash)),
        )
        .add_systems(
            Update,
            (
                tick_splash_timer.in_set(AppSet::TickTimers),
                check_splash_timer.in_set(AppSet::Update),
            )
                .run_if(in_state(AppState::Splash)),
        )
        .add_systems(
            Update,
            continue_to_loading_screen
                .run_if(input_just_pressed(KeyCode::Escape).and(in_state(AppState::Splash))),
        )
        .add_systems(OnExit(AppState::Splash), remove_splash_timer);
}

const SPLASH_DURATION_SECS: f32 = 1.8;
const SPLASH_FADE_DURATION_SECS: f32 = 0.6;

fn startup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands
        .ui_root()
        .insert((Name::new("Splash Screen"), StateScoped(AppState::Splash)))
        .with_children(|root| {
            root.spawn((
                Name::new("Splash Image"),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ImageNode {
                    image: asset_server.load_with_settings(
                        "super_rogue_soccer.png",
                        |settings: &mut ImageLoaderSettings| {
                            settings.sampler = ImageSampler::nearest()
                        },
                    ),
                    ..default()
                },
                ImageNodeFadeInOut {
                    total_duration: SPLASH_DURATION_SECS,
                    fade_duration: SPLASH_FADE_DURATION_SECS,
                    t: 0.0,
                },
            ));
        });
}

fn tick_fade_in_out(time: Res<Time>, mut animation_query: Query<&mut ImageNodeFadeInOut>) {
    for mut anim in &mut animation_query {
        anim.t += time.delta_secs();
    }
}

fn apply_fade_in_out(mut animation_query: Query<(&ImageNodeFadeInOut, &mut ImageNode)>) {
    for (anim, mut image) in &mut animation_query {
        image.color.set_alpha(anim.alpha())
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct ImageNodeFadeInOut {
    /// Total duration in seconds.
    total_duration: f32,
    /// Fade duration in seconds.
    fade_duration: f32,
    /// Current progress in seconds, between 0 and [`Self::total_duration`].
    t: f32,
}

impl ImageNodeFadeInOut {
    fn alpha(&self) -> f32 {
        // Normalize by duration.
        let t = (self.t / self.total_duration).clamp(0.0, 1.0);
        let fade = self.fade_duration / self.total_duration;

        // Regular trapezoid-shaped graph, flat at the top with alpha = 1.0.
        ((1.0 - (2.0 * t - 1.0).abs()) / fade).min(1.0)
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
struct SplashTimer(Timer);

impl Default for SplashTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(SPLASH_DURATION_SECS, TimerMode::Once))
    }
}

fn check_splash_timer(timer: ResMut<SplashTimer>, mut next_screen: ResMut<NextState<AppState>>) {
    if timer.0.just_finished() {
        next_screen.set(AppState::Loading);
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

fn continue_to_loading_screen(mut next_screen: ResMut<NextState<AppState>>) {
    next_screen.set(AppState::Loading);
}