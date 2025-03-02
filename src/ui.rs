use bevy::prelude::*;

use crate::{
    FontAsset, PostUpdateSet,
    actors::{actions::CurrentActions, PointerObject, is_dirty},
    entities::{Interactable, Map},
    states::{AppState, gameplay::InfoContainer},
    to_ivec2,
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        update_ui
            .in_set(PostUpdateSet::Ui)
            .run_if(in_state(AppState::Gameplay).and(is_dirty)),
    );
}

fn update_ui(
    actions: Res<CurrentActions>,
    font_asset: Res<FontAsset>,
    map: Res<Map>,
    ui_elements: Single<Entity, With<InfoContainer>>,
    pointer: Single<&Transform, With<PointerObject>>,
    interactables: Query<(&Name, &Interactable)>,
    mut commands: Commands,
) {
    let entity = ui_elements.into_inner();
    let pointer = pointer.into_inner();
    commands
        .entity(entity)
        .despawn_descendants()
        .with_children(|info| {
            let position = to_ivec2(pointer.translation);
            if let Some(vec) = map.get(&position) {
                for &entity in vec {
                    let (name, _) = interactables.get(entity).unwrap();
                    info.spawn((
                        Text::from(name.as_str()),
                        TextFont {
                            font: font_asset.clone_weak(),
                            ..default()
                        },
                    ));
                }
            };
            for (key, name, _) in &actions.actions {
                info.spawn((
                    Text::from(format!("{} - {}", key, name)),
                    TextFont {
                        font: font_asset.clone_weak(),
                        ..default()
                    },
                ));
            }
        });
}
