use actions::{ActionQueue, Claimed, calculate_kick_velocity};
use bevy::{
    color::palettes::css::{DARK_CYAN, GREEN, ORANGE, RED, WHITE, YELLOW},
    prelude::*,
};
use leafwing_input_manager::prelude::*;
use pathfinding::{CalculatedPath, calculate_path};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{
    AppSet, GlyphAsset,
    entities::{Interactable, Map},
    states::{AppState, GameplayStates},
    to_world,
    ui::LogEvent,
};

pub mod actions;
pub mod enemy;
mod names;
mod pathfinding;

pub fn plugin(app: &mut App) {
    app.register_type::<Stats>()
        .register_type::<Velocity>()
        .register_type::<CharacterClass>()
        .insert_resource(Sampler(ChaCha8Rng::from_os_rng()))
        .insert_resource(PointerIsDirty(true))
        .insert_gizmo_config(
            PassPreviewGizmos {},
            GizmoConfig {
                line_style: GizmoLineStyle::Dotted,
                ..default()
            },
        )
        .add_plugins(InputManagerPlugin::<PointerActions>::default())
        .add_plugins((pathfinding::plugin, actions::plugin))
        .add_systems(
            Update,
            (
                tick_pointer.in_set(AppSet::TickTimers),
                update_pointer
                    .run_if(in_state(AppState::Gameplay))
                    .in_set(AppSet::Update),
                (preview_path.after(update_pointer), preview_pass)
                    .run_if(in_state(GameplayStates::PlayerTurn))
                    .in_set(AppSet::Update),
            ),
        )
        .add_systems(Last, remove_dirty.run_if(is_dirty));
}

#[derive(Resource)]
pub struct Sampler(ChaCha8Rng);

#[derive(Component)]
pub struct PointerObject {
    timer: Timer,
}

#[derive(Component)]
pub struct CurrentPlayer;

#[derive(Resource)]
pub struct PointerIsDirty(bool);

fn remove_dirty(mut dirt: ResMut<PointerIsDirty>) {
    dirt.0 = false;
}

pub fn is_dirty(dirt: Res<PointerIsDirty>) -> bool {
    dirt.0
}

#[derive(Component, Clone, Copy, PartialEq, Reflect, Debug, Eq, Hash)]
pub enum Team {
    Player,
    Enemy,
}

#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub enum CharacterClass {
    Goalkeeper,
    CentralDefender,
    Midfielder,
    Attacker,
}

pub fn startup(mut sampler: ResMut<Sampler>, glyphs: Res<GlyphAsset>, mut commands: Commands) {
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
        (-4.0, 0.0, CharacterClass::Attacker),
        (-45.0, 0.0, CharacterClass::Goalkeeper),
        (-30.0, 8.0, CharacterClass::CentralDefender),
        (-30.0, -8.0, CharacterClass::CentralDefender),
        (-30.0, 24.0, CharacterClass::CentralDefender),
        (-30.0, -24.0, CharacterClass::CentralDefender),
        (-5.0, 16.0, CharacterClass::Attacker),
        (-5.0, -16.0, CharacterClass::Attacker),
        (-18.0, 0.0, CharacterClass::Midfielder),
        (-15.0, 12.0, CharacterClass::Midfielder),
        (-15.0, -12.0, CharacterClass::Midfielder),
    ];

    for (index, position) in positions.iter().enumerate() {
        let mut player = commands.spawn((
            Name::from(random_name(&mut sampler.0)),
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
            Interactable::Person,
            Stats {
                ap: 8,
                kick_strength: 15.0,
                passing_skill: 50.0,
                wit: 1.0,
                defense: 1.0,
                initiative: 10,
            },
            ActionQueue::default(),
            Velocity(Vec2::ZERO),
            Team::Player,
            position.2.clone(),
        ));
        if index == 0 {
            player.insert(CurrentPlayer);
        }
    }

    for (_index, position) in positions.iter().enumerate() {
        commands.spawn((
            Name::from(random_name(&mut sampler.0)),
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
            Interactable::Person,
            Stats {
                ap: 8,
                kick_strength: 15.0,
                passing_skill: 50.0,
                wit: 1.0,
                defense: 1.0,
                initiative: 10,
            },
            ActionQueue::default(),
            Velocity(Vec2::ZERO),
            Team::Enemy,
            position.2.clone(),
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

fn random_name(sampler: &mut ChaCha8Rng) -> String {
    let first_names = if sampler.random_bool(0.5) {
        names::FIRST_NAMES
    } else {
        names::FIRST_NAMES_2
    };

    let first_index = sampler.random_range(0..first_names.len());
    let first_name = first_names[first_index];

    let last_names = if sampler.random_bool(0.5) {
        names::LAST_NAMES
    } else {
        names::LAST_NAMES_2
    };

    let last_index = sampler.random_range(0..last_names.len());
    let last_name = last_names[last_index];
    format!("{} {}", first_name, last_name)
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
    map: Res<Map>,
    mut dirt: ResMut<PointerIsDirty>,
    mut query: Query<
        (&ActionState<PointerActions>, &mut Transform, &PointerObject),
        Without<CurrentPlayer>,
    >,
    current_players: Single<&Transform, With<CurrentPlayer>>,
    mut commands: Commands,
) {
    let start_transform = current_players.into_inner();
    for (action_state, mut transform, pointer) in &mut query {
        if pointer.timer.finished() && action_state.axis_pair(&PointerActions::Move) != Vec2::ZERO {
            let input = action_state.axis_pair(&PointerActions::Move);
            transform.translation += Vec3::new(input.x * 8.0, input.y * 8.0, 0.0);
            if let Ok(path) =
                calculate_path(start_transform.translation, transform.translation, &map)
            {
                commands.insert_resource(PreviewPath { path });
            }
            dirt.0 = true;
        }
    }
}

fn preview_path(
    path_preview: Option<Res<PreviewPath>>,
    current_player: Option<
        Single<(&Stats, &Transform, Option<&CalculatedPath>), With<CurrentPlayer>>,
    >,
    mut gizmos: Gizmos,
) {
    if path_preview.is_none() || current_player.is_none() {
        return;
    }
    let (stats, transform, path_option) = current_player.unwrap().into_inner();

    if let Some(calculated_path) = path_option {
        for window in calculated_path.path.windows(2) {
            gizmos.arrow_2d(to_world(window[0]), to_world(window[1]), GREEN);
        }
    } else {
        let path = path_preview.unwrap();

        for (index, window) in path.path.windows(2).enumerate() {
            let color = if index < stats.ap { GREEN } else { RED };
            gizmos.arrow_2d(to_world(window[0]), to_world(window[1]), color);
        }
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct PassPreviewGizmos {}

fn preview_pass(
    time: Res<Time<Fixed>>,
    current_player_option: Option<
        Single<(&Stats, &Transform, &Velocity), (With<CurrentPlayer>, With<Claimed>)>,
    >,
    query: Option<Single<&Transform, (With<PointerObject>, Without<CurrentPlayer>)>>,
    mut gizmos: Gizmos<PassPreviewGizmos>,
) {
    let Some(current_player) = current_player_option else {
        return;
    };
    let Some(pointer) = query else {
        return;
    };
    let (stats, transform, velocity) = current_player.into_inner();
    let mut kick_vel = calculate_kick_velocity(
        stats.passing_skill,
        transform.translation.truncate(),
        pointer.translation.truncate(),
        time.delta_secs(),
        velocity.0,
    );

    let mut end_point = Vec2::ZERO;
    while kick_vel.length() > 0.1 {
        end_point += kick_vel * time.delta_secs();
        kick_vel *= 0.9;
    }

    gizmos.line_2d(transform.translation.truncate(), end_point, WHITE);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Stats {
    ap: usize,
    kick_strength: f32,
    passing_skill: f32,
    wit: f32,
    defense: f32,
    pub initiative: u8,
}

impl Stats {
    fn from_class() -> Self {
        Stats {
            ap: 10,
            kick_strength: 15.0,
            passing_skill: 50.0,
            wit: 1.0,
            defense: 1.0,
            initiative: 10,
        }
    }

    pub fn reset_ap(&mut self) {
        self.ap = 10;
    }
}

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AP: {} | KICK STRENGTH: {}\nPASSING SKILL: {} | WIT {}\nDEFENSE: {}",
            self.ap, self.kick_strength, self.passing_skill, self.wit, self.defense
        )
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Velocity(Vec2);
