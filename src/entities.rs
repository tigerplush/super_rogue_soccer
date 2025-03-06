use std::collections::HashMap;

use bevy::prelude::*;

use crate::{PostUpdateSet, to_ivec2};

pub fn plugin(app: &mut App) {
    app.register_type::<Map>()
        .insert_resource(Map::default())
        .add_systems(PreUpdate, update_map)
        .add_systems(PostUpdate, update_map.in_set(PostUpdateSet::Calculate));
}

#[derive(Component, PartialEq, Reflect, Clone)]
pub enum Interactable {
    Ball,
    Person,
    Wall,
    Goal,
}

#[derive(Resource, Default, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub struct Map {
    map: HashMap<IVec2, Vec<(Entity, Interactable)>>,
}

fn update_map(mut map: ResMut<Map>, query: Query<(&Transform, Entity, &Interactable)>) {
    map.clear();
    for (transform, entity, interactable) in &query {
        let position = to_ivec2(transform.translation);
        map.entry(position)
            .and_modify(|vec| vec.push((entity, interactable.clone())))
            .or_insert(vec![(entity, interactable.clone())]);
    }
}
