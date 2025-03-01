use asset_tracking::LoadResource;
use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

mod actors;
mod asset_tracking;
mod states;
mod theme;

pub struct SuperRogueSoccerPlugin;

impl Plugin for SuperRogueSoccerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((asset_tracking::plugin, actors::plugin, states::plugin));
        app.load_resource::<GlyphAsset>();
        app.add_systems(Startup, startup);
    }
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(Resource, Asset, TypePath, Clone)]
pub struct GlyphAsset {
    pub glyph: Handle<Image>,
    pub atlas: Handle<TextureAtlasLayout>,
}

impl GlyphAsset {
    const PATH: &'static str = "terminal8x8.png";
}

impl FromWorld for GlyphAsset {
    fn from_world(world: &mut World) -> Self {
        let layout_handle = {
            let layout = TextureAtlasLayout::from_grid(UVec2::splat(8), 16, 16, None, None);
            let mut layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
            layouts.add(layout)
        };
        let assets = world.resource::<AssetServer>();
        GlyphAsset {
            glyph: assets.load_with_settings(
                GlyphAsset::PATH,
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve the pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            atlas: layout_handle,
        }
    }
}
