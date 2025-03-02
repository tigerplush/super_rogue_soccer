use std::collections::HashMap;

use bevy::prelude::*;

use crate::to_ivec2;

pub fn plugin(app: &mut App) {
    app.register_type::<Map>()
        .insert_resource(Map::default())
        .add_systems(PreUpdate, update_map);
}

#[derive(Component)]
pub enum Interactable {
    Ball,
    Person,
}

#[derive(Resource, Default, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub struct Map {
    map: HashMap<IVec2, Vec<Entity>>,
}

fn update_map(mut map: ResMut<Map>, query: Query<(&Transform, Entity), With<Interactable>>) {
    map.clear();
    for (transform, entity) in &query {
        let position = to_ivec2(transform.translation);
        map.entry(position)
            .and_modify(|vec| vec.push(entity))
            .or_insert(vec![entity]);
    }
}
