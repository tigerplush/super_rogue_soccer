use bevy::{color::palettes::css::GREY, prelude::*};

use crate::{
    FontAsset, PostUpdateSet,
    actors::{PointerObject, actions::CurrentActions, is_dirty},
    entities::{Interactable, Map},
    states::{
        AppState,
        gameplay::{InfoContainer, Log},
    },
    to_ivec2,
};

pub fn plugin(app: &mut App) {
    app.add_event::<LogEvent>().insert_resource(Logs::default()).add_systems(
        PostUpdate,
        update_ui
            .in_set(PostUpdateSet::Ui)
            .run_if(in_state(AppState::Gameplay).and(is_dirty)),
    ).add_systems(PostUpdate, update_log.in_set(PostUpdateSet::Ui).run_if(in_state(AppState::Gameplay)));
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
            for (key, name, available) in &actions.actions {
                let color = if *available {
                    Color::WHITE
                } else {
                    GREY.into()
                };
                info.spawn((
                    Text::from(format!("{} - {}", key, name)),
                    TextFont {
                        font: font_asset.clone_weak(),
                        ..default()
                    },
                    TextColor(color),
                ));
            }
        });
}

#[derive(Event)]
pub struct LogEvent(pub String);

#[derive(Resource, Default)]
struct Logs(Vec<String>);

fn update_log(
    mut events: EventReader<LogEvent>,
    mut logs: ResMut<Logs>,
    font_asset: Res<FontAsset>,
    log_ui: Query<Entity, With<Log>>,
    mut commands: Commands,
) {
    for event in events.read() {
        logs.0.push(event.0.clone());
    }
    for ui in &log_ui {
        commands
            .entity(ui)
            .despawn_descendants()
            .with_children(|log| {
                for entry in logs.0.iter().rev().take(10) {
                    log.spawn((
                        Text::new(entry),
                        TextFont {
                            font: font_asset.font.clone_weak(),
                            ..default()
                        },
                    ));
                }
            });
    }
}
