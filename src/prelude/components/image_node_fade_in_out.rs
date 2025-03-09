use bevy::prelude::*;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ImageNodeFadeInOut {
    total_duration: f32,
    fade_duration: f32,
    t: f32,
}

impl ImageNodeFadeInOut {
    const fn new(total_duration: f32, fade_duration: f32) -> Self {
        ImageNodeFadeInOut {
            total_duration,
            fade_duration,
            t: 0.0,
        }
    }
}

pub const SPLASH_DURATION_SECS: f32 = 1.8;
const SPLASH_FADE_DURATION_SECS: f32 = 0.6;

impl Default for ImageNodeFadeInOut {
    fn default() -> Self {
        ImageNodeFadeInOut::new(SPLASH_DURATION_SECS, SPLASH_FADE_DURATION_SECS)
    }
}
