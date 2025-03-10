use bevy::{
    color::palettes::css::{DARK_CYAN, ORANGE},
    prelude::*,
};

use crate::prelude::*;

pub fn spawn(
    mut names: ResMut<Names>,
    mut stat_factory: ResMut<StatsFactory>,
    glyphs: Res<GlyphAsset>,
    mut commands: Commands,
) {
    commands.spawn((
        Name::from("Ball"),
        Sprite {
            image: glyphs.glyph.clone_weak(),
            texture_atlas: Some(TextureAtlas {
                index: 7,
                layout: glyphs.atlas.clone_weak(),
            }),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 2.0),
        Interactables::Ball,
        Ball,
    ));

    let positions = [
        (-45.0, 0.0, CharacterClasses::Goalkeeper),
        (-30.0, 8.0, CharacterClasses::CentralDefender),
        (-30.0, -8.0, CharacterClasses::CentralDefender),
        (-30.0, 24.0, CharacterClasses::CentralDefender),
        (-30.0, -24.0, CharacterClasses::CentralDefender),
        (-18.0, 0.0, CharacterClasses::Midfielder),
        (-15.0, 12.0, CharacterClasses::Midfielder),
        (-15.0, -12.0, CharacterClasses::Midfielder),
        (-5.0, 16.0, CharacterClasses::Attacker),
        (-5.0, -16.0, CharacterClasses::Attacker),
        (-4.0, 0.0, CharacterClasses::Attacker),
    ];

    for (index, position) in positions.iter().enumerate() {
        commands.spawn((
            Name::from(names.random()),
            Sprite {
                image: glyphs.glyph.clone_weak(),
                texture_atlas: Some(TextureAtlas {
                    index: 1,
                    layout: glyphs.atlas.clone_weak(),
                }),
                color: ORANGE.into(),
                ..default()
            },
            Transform::from_xyz(position.0 * 8.0, position.1 * 8.0, 1.0),
            Interactables::Person,
            stat_factory
                .create_from_class(&position.2)
                .with_initiative(index),
            ActionQueue::default(),
            Velocity(Vec2::ZERO),
            Teams::Player,
            position.2.clone(),
        ));
    }

    for (index, position) in positions.iter().enumerate() {
        commands.spawn((
            Name::from(names.random()),
            Sprite {
                image: glyphs.glyph.clone_weak(),
                texture_atlas: Some(TextureAtlas {
                    index: 1,
                    layout: glyphs.atlas.clone_weak(),
                }),
                color: DARK_CYAN.into(),
                ..default()
            },
            Transform::from_xyz(position.0 * -8.0, position.1 * 8.0, 1.0),
            Interactables::Person,
            stat_factory
                .create_from_class(&position.2)
                .with_initiative(index),
            ActionQueue::default(),
            Velocity(Vec2::ZERO),
            Teams::Enemy,
            position.2.clone(),
        ));
    }
}
