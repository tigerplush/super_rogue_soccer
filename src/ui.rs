use bevy::{color::palettes::css::GREY, prelude::*};

use crate::{
    FontAsset, PostUpdateSet,
    actors::{
        Stats,
        actions::{CurrentActions, PossibleAction},
        is_dirty,
    },
    entities::Interactable,
    states::{
        AppState,
        gameplay::{InfoContainer, Log},
    },
};

pub fn plugin(app: &mut App) {
    app.add_event::<LogEvent>()
        .insert_resource(Logs::default())
        .add_systems(
            PostUpdate,
            update_ui
                .in_set(PostUpdateSet::Ui)
                .run_if(in_state(AppState::Gameplay).and(is_dirty)),
        )
        .add_systems(
            PostUpdate,
            update_log
                .in_set(PostUpdateSet::Ui)
                .run_if(in_state(AppState::Gameplay)),
        );
}

fn update_ui(
    actions: Res<CurrentActions>,
    font_asset: Res<FontAsset>,
    ui_elements: Single<Entity, With<InfoContainer>>,
    interactables: Query<&Name, With<Interactable>>,
    stat_blocks: Query<&Stats>,
    mut commands: Commands,
) {
    let entity = ui_elements.into_inner();
    commands
        .entity(entity)
        .despawn_descendants()
        .with_children(|info| {
            for action in &actions.actions {
                match action {
                    PossibleAction::StatBlock(entity) => {
                        if let Ok(stat) = stat_blocks.get(*entity) {
                            let name = interactables.get(*entity).unwrap();
                            info.spawn((
                                Text::from(name.to_string()),
                                TextFont {
                                    font: font_asset.clone_weak(),
                                    ..default()
                                },
                            ));
                            info.spawn((
                                Text::from(stat.to_string()),
                                TextFont {
                                    font: font_asset.clone_weak(),
                                    ..default()
                                },
                            ));
                        }
                    }
                    PossibleAction::Header(target) => {
                        let target = interactables.get(*target).unwrap();
                        info.spawn((
                            Text::from(target.to_string()),
                            TextFont {
                                font: font_asset.clone_weak(),
                                ..default()
                            },
                        ));
                    }
                    PossibleAction::EntityCommands(commands) => {
                        for (key, command, available) in commands {
                            let color = if *available {
                                Color::WHITE
                            } else {
                                GREY.into()
                            };
                            info.spawn((
                                Text::from(format!("{} - {}", key, command)),
                                TextFont {
                                    font: font_asset.clone_weak(),
                                    ..default()
                                },
                                TextColor(color),
                            ));
                        }
                    }
                    PossibleAction::Command(key, command, available) => {
                        let color = if *available {
                            Color::WHITE
                        } else {
                            GREY.into()
                        };
                        info.spawn((
                            Text::from(format!("{} - {}", key, command)),
                            TextFont {
                                font: font_asset.clone_weak(),
                                ..default()
                            },
                            TextColor(color),
                        ));
                    }
                }
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
