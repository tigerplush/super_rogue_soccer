use bevy::prelude::*;

use crate::{actors::actions::Kicked, entities::Interactable};

pub fn plugin(app: &mut App) {
    app.register_type::<KickVelocity>()
        .insert_resource(KickVelocity(Vec2::new(500.0, 500.0)))
        .add_systems(Update, add_kick);
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct KickVelocity(Vec2);

fn add_kick(
    vel: Res<KickVelocity>,
    input: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &Interactable)>,
    mut commands: Commands,
) {
    if input.just_pressed(KeyCode::F1) {
        for (entity, interactable) in &query {
            if &Interactable::Ball == interactable {
                commands.entity(entity).insert(Kicked(vel.0));
            }
        }
    }
}
