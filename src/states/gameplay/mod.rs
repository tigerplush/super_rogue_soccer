use bevy::{color::palettes::css::YELLOW, prelude::*, ui::widget::NodeImageMode};
use leafwing_input_manager::prelude::*;

use crate::prelude::*;

use super::AppStates;

mod enemy_turn;
mod player_turn;
mod setup;
mod show_banner;

pub fn plugin(app: &mut App) {
    app.add_sub_state::<GameplayStates>()
        .add_plugins((
            enemy_turn::plugin,
            player_turn::plugin,
            setup::plugin,
            show_banner::plugin,
        ))
        .add_plugins(InputManagerPlugin::<PointerActions>::default())
        .add_systems(OnEnter(AppStates::Gameplay), (spawn_pointer_controls, spawn_ui))
        .add_systems(
            Update,
            (tick_pointer, move_pointer).run_if(in_state(AppStates::Gameplay)),
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

#[derive(Actionlike, Reflect, Clone, Hash, Eq, PartialEq, Debug)]
enum PointerActions {
    #[actionlike(DualAxis)]
    Move,
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


#[derive(Component)]
pub struct InfoContainer;

#[derive(Component)]
pub struct Log;

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

fn tick_pointer(time: Res<Time>, mut query: Query<&mut PointerObject>) {
    for mut pointer in &mut query {
        pointer.0.tick(time.delta());
    }
}

fn move_pointer(pointer: Single<(&ActionState<PointerActions>, &mut Transform, &PointerObject)>) {
    let (action_state, mut transform, pointer) = pointer.into_inner();
    if pointer.0.finished() && action_state.axis_pair(&PointerActions::Move) != Vec2::ZERO {
        let input = action_state.axis_pair(&PointerActions::Move);
        transform.translation += (input * 8.0).extend(0.0);
    }
}

fn clean_up() {}
