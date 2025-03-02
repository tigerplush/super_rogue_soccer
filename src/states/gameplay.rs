use bevy::{prelude::*, ui::widget::NodeImageMode};

use crate::{PanelBorderAsset, actors, map, theme::prelude::*};

use super::AppState;
use crate::actors::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::Gameplay),
        (
            actors::startup,
            actions::setup_slotmap,
            map::spawn_field,
            startup,
        ),
    );
}

#[derive(Component)]
pub struct InfoContainer;

fn startup(panel_border: Res<PanelBorderAsset>, mut commands: Commands) {
    commands.ui_root().with_children(|root| {
        root.spawn((
            Name::from("Info Container"),
            Node {
                width: Val::Percent(25.0),
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
    });
}
