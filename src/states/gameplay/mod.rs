use std::collections::HashMap;

use bevy::{
    color::palettes::css::{DARK_GRAY, WHITE, YELLOW},
    prelude::*,
    ui::widget::NodeImageMode,
};
use leafwing_input_manager::prelude::*;

use crate::prelude::*;

use super::*;

mod enemy_turn;
mod player_turn;
mod setup;
mod show_banner;

pub fn plugin(app: &mut App) {
    app.add_sub_state::<GameplayStates>()
        .register_type::<DisplayActions>()
        .register_type::<PointerLocation>()
        .insert_resource(DisplayActions(Vec::new()))
        .insert_resource(PointerLocation::new())
        .insert_resource(CurrentTeam(Teams::Player))
        .add_plugins((
            enemy_turn::plugin,
            player_turn::plugin,
            setup::plugin,
            show_banner::plugin,
        ))
        .add_plugins(InputManagerPlugin::<PointerActions>::default())
        .add_systems(
            OnEnter(AppStates::Gameplay),
            (spawn_pointer_controls, spawn_ui),
        )
        .add_systems(PreUpdate, update_map.run_if(in_state(AppStates::Gameplay)))
        .add_systems(
            Update,
            (
                tick_pointer.in_set(AppSet::TickTimers),
                move_pointer.in_set(AppSet::UpdateMovement),
            )
                .run_if(in_state(AppStates::Gameplay)),
        )
        .add_systems(
            PostUpdate,
            (update_ui.run_if(pointer_is_dirty)).run_if(in_state(AppStates::Gameplay)),
        )
        .add_systems(
            Last,
            (reset_pointer_location).run_if(in_state(AppStates::Gameplay)),
        )
        .add_systems(OnExit(AppStates::Gameplay), clean_up);
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, SubStates)]
#[source(AppStates = AppStates::Gameplay)]
enum GameplayStates {
    #[default]
    Setup,
    ShowBanner,
    PlayerTurn,
    EnemyTurn,
}

#[derive(Debug, Resource, Reflect)]
#[reflect(Resource)]
struct PointerLocation {
    old: IVec2,
    new: IVec2,
}

impl PointerLocation {
    fn new() -> Self {
        PointerLocation {
            old: IVec2::NEG_ONE,
            new: IVec2::ZERO,
        }
    }
}

fn pointer_is_dirty(pointer_location: Res<PointerLocation>) -> bool {
    pointer_location.new != pointer_location.old
}

#[derive(Actionlike, Reflect, Clone, Hash, Eq, PartialEq, Debug)]
enum PointerActions {
    #[actionlike(DualAxis)]
    Move,
}
#[derive(Component)]
pub struct InfoContainer;

#[derive(Component)]
pub struct Log;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct DisplayActions(Vec<DisplayAction>);

type Action = (KeyCode, PlayerCommand, bool);

#[derive(Reflect)]
enum PlayerCommand {
    Walk,
    TakeControl,
    Pass,
    Kick,
    Foul,
    Skip,
}

impl PlayerCommand {
    fn cost(&self) -> usize {
        match self {
            &PlayerCommand::Walk => 0,
            &PlayerCommand::TakeControl => 2,
            &PlayerCommand::Pass => 2,
            &PlayerCommand::Kick => 2,
            &PlayerCommand::Foul => 2,
            &PlayerCommand::Skip => 0,
        }
    }
}

impl ToString for PlayerCommand {
    fn to_string(&self) -> String {
        match self {
            &PlayerCommand::Walk => "walk",
            &PlayerCommand::TakeControl => "take control",
            &PlayerCommand::Pass => "pass",
            &PlayerCommand::Kick => "kick",
            &PlayerCommand::Foul => "foul",
            &PlayerCommand::Skip => "SKIP",
        }
        .to_string()
    }
}

#[derive(Reflect)]
enum DisplayAction {
    SingleAction(Action),
    EntityAction(Entity, Vec<Action>),
    StatBlock(Entity),
}

fn spawn_pointer_controls(glyphs: Res<GlyphAsset>, mut commands: Commands) {
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
            PointerObject(Timer::from_seconds(0.08, TimerMode::Repeating)),
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

fn spawn_ui(panel_border: Res<PanelBorderAsset>, mut commands: Commands) {
    commands.ui_root().with_children(|root| {
        root.spawn((
            Name::from("Header"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(36.0),
                padding: UiRect::all(Val::Percent(1.5)),
                ..default()
            },
            ImageNode {
                image: panel_border.image.clone_weak(),
                image_mode: NodeImageMode::Sliced(panel_border.slicer.clone()),
                ..default()
            },
        ));
        root.spawn((
            Name::from("Info Container"),
            Node {
                width: Val::Px(344.0),
                height: Val::Percent(100.0),
                align_self: AlignSelf::FlexEnd,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Percent(1.5)),
                ..default()
            },
            ImageNode {
                image: panel_border.image.clone_weak(),
                image_mode: NodeImageMode::Sliced(panel_border.slicer.clone()),
                ..default()
            },
            InfoContainer,
        ));
        root.spawn((
            Name::from("Footer"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(180.0),
                overflow: Overflow::clip_y(),
                overflow_clip_margin: OverflowClipMargin::content_box().with_margin(8.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Percent(1.5)),
                ..default()
            },
            ImageNode {
                image: panel_border.image.clone_weak(),
                image_mode: NodeImageMode::Sliced(panel_border.slicer.clone()),
                ..default()
            },
            Log,
        ));
    });
}

fn update_map(
    mut positions: ResMut<EntityPositions>,
    query: Query<(Entity, &Interactables, &Transform)>,
) {
    positions.0.clear();
    for (entity, interactable, transform) in &query {
        let position = transform.as_ivec2();
        positions
            .0
            .entry(position)
            .and_modify(|e| e.push((entity, interactable.clone())))
            .or_insert(vec![(entity, interactable.clone())]);
    }
}

fn tick_pointer(time: Res<Time>, mut query: Query<&mut PointerObject>) {
    for mut pointer in &mut query {
        pointer.0.tick(time.delta());
    }
}

fn move_pointer(
    mut pointer_location: ResMut<PointerLocation>,
    pointer: Single<(&ActionState<PointerActions>, &mut Transform, &PointerObject)>,
) {
    let (action_state, mut transform, pointer) = pointer.into_inner();
    if pointer.0.finished() && action_state.axis_pair(&PointerActions::Move) != Vec2::ZERO {
        let input = action_state.axis_pair(&PointerActions::Move);
        transform.translation += (input * 8.0).extend(0.0);
    }
    pointer_location.new = transform.as_ivec2();
}

fn update_ui(
    possible_action: Res<DisplayActions>,
    container: Single<Entity, With<InfoContainer>>,
    names: Query<&Name>,
    stats: Query<&Stats>,
    mut commands: Commands,
) {
    let key_names: HashMap<KeyCode, &str> = HashMap::from([
        (KeyCode::KeyF, "f"),
        (KeyCode::KeyG, "g"),
        (KeyCode::KeyH, "h"),
        (KeyCode::KeyJ, "j"),
        (KeyCode::KeyK, "k"),
        (KeyCode::KeyL, "l"),
        (KeyCode::KeyV, "v"),
        (KeyCode::KeyB, "b"),
        (KeyCode::Space, "space"),
    ]);
    let entity = container.into_inner();
    commands.entity(entity).despawn_descendants();
    for display_action in &possible_action.0 {
        match display_action {
            DisplayAction::SingleAction((key, action, available)) => {
                let color = if *available { WHITE } else { DARK_GRAY };
                commands.entity(entity).with_child((
                    Text::new(format!(
                        "{} - {}",
                        key_names.get(key).unwrap(),
                        action.to_string()
                    )),
                    TextColor(color.into()),
                ));
            }
            DisplayAction::StatBlock(target) => {
                let name = names.get(*target).unwrap();
                let stat = stats.get(*target).unwrap();
                commands.entity(entity).with_children(|parent| {
                    parent.spawn(Text::new(format!("{}", name)));
                    parent.spawn(Text::new(format!("{}", stat)));
                });
            }
            DisplayAction::EntityAction(target, items) => {
                let name = names.get(*target).unwrap();
                commands.entity(entity).with_children(|parent| {
                    parent.spawn(Text::new(format!("{}", name)));
                    for (key, action, available) in items {
                        let color = if *available { WHITE } else { DARK_GRAY };
                        parent.spawn((
                            Text::new(format!(
                                "{} - {}",
                                key_names.get(key).unwrap(),
                                action.to_string()
                            )),
                            TextColor(color.into()),
                        ));
                    }
                });
            }
        }
    }
}
fn reset_pointer_location(
    mut pointer_location: ResMut<PointerLocation>,
    pointer: Single<&Transform, With<PointerObject>>,
) {
    let transform = pointer.into_inner();
    pointer_location.old = transform.as_ivec2();
}

fn clean_up() {}
