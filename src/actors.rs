use bevy::{color::palettes::css::YELLOW, prelude::*};
use leafwing_input_manager::prelude::*;

use crate::{GlyphAsset, entities::Interactable};

pub fn plugin(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<PointerActions>::default());
    app.add_systems(Update, update_pointer);
}

#[derive(Component)]
struct Pointer {
    timer: Timer,
}

pub fn startup(glyphs: Res<GlyphAsset>, mut commands: Commands) {
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
        Interactable,
    ));

    let positions = [
        (Name::from("Goalkeeper"), -45.0, 0.0),
        (Name::from("Central-Defender Left"), -30.0, 8.0),
        (Name::from("Central-Defender Right"), -30.0, -8.0),
        (Name::from("Left-Back"), -30.0, 24.0),
        (Name::from("Right-Back"), -30.0, -24.0),
        (Name::from("Center-Forward"), -4.0, 0.0),
        (Name::from("Center-Right"), -5.0, 16.0),
        (Name::from("Center-Left"), -5.0, -16.0),
        (Name::from("Midfield-Center"), -18.0, 0.0),
        (Name::from("Midfield-Right"), -15.0, 12.0),
        (Name::from("Midfield-Left"), -15.0, -12.0),
    ];

    for position in positions {
        commands.spawn((
            position.0,
            Sprite {
                image: glyphs.glyph.clone_weak(),
                texture_atlas: Some(TextureAtlas {
                    index: 1,
                    layout: glyphs.atlas.clone_weak(),
                }),
                ..default()
            },
            Transform::from_xyz(position.1 * 8.0, position.2 * 8.0, 1.0),
            Interactable,
        ));
    }

    let input_map = InputMap::default()
        .with_dual_axis(PointerActions::Move, VirtualDPad::numpad())
        .with_dual_axis(PointerActions::Move, VirtualDPad::wasd())
        .with_dual_axis(PointerActions::Move, VirtualDPad::arrow_keys());
    commands
        .spawn((
            Name::from("Pointer"),
            Visibility::default(),
            Transform::from_xyz(0.0, 0.0, 3.0),
            InputManagerBundle::with_map(input_map),
            Pointer {
                timer: Timer::from_seconds(0.08, TimerMode::Repeating),
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Sprite {
                    image: glyphs.glyph.clone_weak(),
                    texture_atlas: Some(TextureAtlas {
                        index: 16 + 15,
                        layout: glyphs.atlas.clone_weak(),
                    }),
                    color: YELLOW.into(),
                    ..default()
                },
                Transform::from_xyz(0.0, 8.0, 0.0),
            ));
        });
}

#[derive(Actionlike, Reflect, Clone, Hash, Eq, PartialEq, Debug)]
enum PointerActions {
    #[actionlike(DualAxis)]
    Move,
}

fn update_pointer(
    time: Res<Time>,
    mut query: Query<(&ActionState<PointerActions>, &mut Transform, &mut Pointer)>,
) {
    for (action_state, mut transform, mut pointer) in &mut query {
        pointer.timer.tick(time.delta());
        if pointer.timer.finished() && action_state.axis_pair(&PointerActions::Move) != Vec2::ZERO {
            let input = action_state.axis_pair(&PointerActions::Move);
            transform.translation += Vec3::new(input.x * 8.0, input.y * 8.0, 0.0);
        }
    }
}
