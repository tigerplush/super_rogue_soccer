use bevy::prelude::*;

#[derive(Resource, Asset, TypePath, Clone, Deref, DerefMut)]
pub struct FontAsset {
    pub font: Handle<Font>,
}

impl FontAsset {
    const PATH: &'static str = "FreePixel.ttf";
    // const PATH: &'static str = "16bfZX.ttf";
    // const PATH: &'static str = "PixelifySans-VariableFont_wght.ttf";
}

impl FromWorld for FontAsset {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        FontAsset {
            font: assets.load(FontAsset::PATH),
        }
    }
}
