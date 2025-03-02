use std::collections::HashMap;

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.register_type::<Map>()
        .insert_resource(Map::default())
        .add_systems(PreUpdate, update_map);
}

#[derive(Component)]
pub struct Interactable;

#[derive(Resource, Default, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub struct Map {
    map: HashMap<IVec2, Vec<Entity>>,
}

fn update_map(mut map: ResMut<Map>, query: Query<(&Transform, Entity), With<Interactable>>) {
    map.clear();
    for (transform, entity) in &query {
        let x = (transform.translation.x / 8.0).floor() as i32;
        let y = (transform.translation.y / 8.0).floor() as i32;
        let position = IVec2::new(x, y);
        map.entry(position)
            .and_modify(|vec| vec.push(entity))
            .or_insert(vec![entity]);
    }
}
