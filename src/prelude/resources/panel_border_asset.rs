use bevy::{image::{ImageLoaderSettings, ImageSampler}, prelude::*};

#[derive(Resource, Asset, TypePath, Clone)]
pub struct PanelBorderAsset {
    pub image: Handle<Image>,
    pub slicer: TextureSlicer,
}

impl PanelBorderAsset {
    const PATH: &'static str = "panel_border.png";
}

impl FromWorld for PanelBorderAsset {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        PanelBorderAsset {
            image: assets.load_with_settings(
                PanelBorderAsset::PATH,
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve the pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            slicer: TextureSlicer {
                border: BorderRect::square(8.0),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 1.0,
            },
        }
    }
}