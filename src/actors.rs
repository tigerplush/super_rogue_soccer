use bevy::prelude::*;

use crate::GlyphAsset;

pub fn plugin(app: &mut App) {}

pub fn startup(glyphs: Res<GlyphAsset>, mut commands: Commands) {
    commands.spawn(Sprite {
        image: glyphs.glyph.clone_weak(),
        texture_atlas: Some(TextureAtlas {
            index: 1,
            layout: glyphs.atlas.clone_weak(),
        }),
        ..default()
    });
}
