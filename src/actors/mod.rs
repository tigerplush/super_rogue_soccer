use bevy::{
    color::palettes::css::{GREEN, RED, YELLOW},
    prelude::*,
};
use leafwing_input_manager::prelude::*;
use pathfinding::calculate_path;

use crate::{AppSet, GlyphAsset, entities::Interactable, to_world};

pub mod actions;
mod pathfinding;

pub fn plugin(app: &mut App) {
    app.register_type::<Stats>().insert_resource(PointerIsDirty(true))
        .add_plugins(InputManagerPlugin::<PointerActions>::default())
        .add_plugins((pathfinding::plugin, actions::plugin))
        .add_systems(
            Update,
            (
                tick_pointer.in_set(AppSet::TickTimers),
                (update_pointer, preview_path.after(update_pointer)).in_set(AppSet::Update),
            ),
        )
        .add_systems(Last, remove_dirty.run_if(is_dirty));
}

#[derive(Component)]
pub struct PointerObject {
    timer: Timer,
}

#[derive(Component)]
struct CurrentPlayer;

#[derive(Resource)]
pub struct PointerIsDirty(bool);

fn remove_dirty(mut dirt: ResMut<PointerIsDirty>) {
    dirt.0 = false;
}

pub fn is_dirty(dirt: Res<PointerIsDirty>) -> bool {
    dirt.0
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
        Interactable::Ball,
    ));

    let positions = [
        (Name::from("Center-Forward"), -4.0, 0.0),
        (Name::from("Goalkeeper"), -45.0, 0.0),
        (Name::from("Central-Defender Left"), -30.0, 8.0),
        (Name::from("Central-Defender Right"), -30.0, -8.0),
        (Name::from("Left-Back"), -30.0, 24.0),
        (Name::from("Right-Back"), -30.0, -24.0),
        (Name::from("Center-Right"), -5.0, 16.0),
        (Name::from("Center-Left"), -5.0, -16.0),
        (Name::from("Midfield-Center"), -18.0, 0.0),
        (Name::from("Midfield-Right"), -15.0, 12.0),
        (Name::from("Midfield-Left"), -15.0, -12.0),
    ];

    for (index, position) in positions.iter().enumerate() {
        let mut player = commands.spawn((
            position.0.clone(),
            Sprite {
                image: glyphs.glyph.clone_weak(),
                texture_atlas: Some(TextureAtlas {
                    index: 1,
                    layout: glyphs.atlas.clone_weak(),
                }),
                ..default()
            },
            Transform::from_xyz(position.1 * 8.0, position.2 * 8.0, 1.0),
            Interactable::Person,
            Stats { ap: 8 },
        ));
        if index == 0 {
            player.insert(CurrentPlayer);
        }
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
            PointerObject {
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
    info!("done spawning");
}

#[derive(Actionlike, Reflect, Clone, Hash, Eq, PartialEq, Debug)]
enum PointerActions {
    #[actionlike(DualAxis)]
    Move,
}

fn tick_pointer(time: Res<Time>, mut query: Query<&mut PointerObject>) {
    for mut pointer in &mut query {
        pointer.timer.tick(time.delta());
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct PreviewPath {
    path: Vec<IVec2>,
}

fn update_pointer(
    mut dirt: ResMut<PointerIsDirty>,
    mut query: Query<
        (&ActionState<PointerActions>, &mut Transform, &PointerObject),
        Without<CurrentPlayer>,
    >,
    current_players: Query<&Transform, With<CurrentPlayer>>,
    mut commands: Commands,
) {
    for (action_state, mut transform, pointer) in &mut query {
        if pointer.timer.finished() && action_state.axis_pair(&PointerActions::Move) != Vec2::ZERO {
            let input = action_state.axis_pair(&PointerActions::Move);
            transform.translation += Vec3::new(input.x * 8.0, input.y * 8.0, 0.0);
            if let Ok(start_transform) = current_players.get_single() {
                if let Ok(path) = calculate_path(start_transform.translation, transform.translation)
                {
                    commands.insert_resource(PreviewPath { path });
                }
            }
            dirt.0 = true;
        }
    }
}

fn preview_path(path: Option<Res<PreviewPath>>, current_player: Option<Single<&Stats, With<CurrentPlayer>>>, mut gizmos: Gizmos) {
    if path.is_none() || current_player.is_none(){
        return;
    }
    let path = path.unwrap();
    let stats = current_player.unwrap().into_inner();
    for (index, window) in path.path.windows(2).enumerate() {
        let color = if index < stats.ap { GREEN } else { RED };
        gizmos.arrow_2d(to_world(window[0]), to_world(window[1]), color);
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Stats {
    ap: usize,
}
