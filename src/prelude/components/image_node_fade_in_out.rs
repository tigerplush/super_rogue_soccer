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

    pub fn faded_in(mut self) -> Self {
        self.t += self.fade_duration;
        self
    }

    fn alpha(&self) -> f32 {
        // Normalize by duration.
        let t = (self.t / self.total_duration).clamp(0.0, 1.0);
        let fade = self.fade_duration / self.total_duration;

        // Regular trapezoid-shaped graph, flat at the top with alpha = 1.0.
        ((1.0 - (2.0 * t - 1.0).abs()) / fade).min(1.0)
    }

    pub fn elapsed(&self) -> bool {
        self.t > self.total_duration
    }
}

pub const SPLASH_DURATION_SECS: f32 = 1.8;
const SPLASH_FADE_DURATION_SECS: f32 = 0.6;

impl Default for ImageNodeFadeInOut {
    fn default() -> Self {
        ImageNodeFadeInOut::new(SPLASH_DURATION_SECS, SPLASH_FADE_DURATION_SECS)
    }
}

pub fn tick_image_node_fades(time: Res<Time>, mut query: Query<&mut ImageNodeFadeInOut>) {
    for mut image_node in &mut query {
        image_node.t += time.delta_secs();
    }
}

pub fn apply_image_node_fades(mut query: Query<(&ImageNodeFadeInOut, Option<&mut ImageNode>, Option<&mut TextColor>)>) {
    for (image_node_fade, image_node_option, text_color_option) in &mut query {
        if let Some(mut image_node) = image_node_option {
            image_node.color.set_alpha(image_node_fade.alpha());
        }
        if let Some(mut text_color) = text_color_option {
            text_color.0.set_alpha(image_node_fade.alpha());
        }
    }
}
