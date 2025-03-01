use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::GlyphAsset;

mod field;

const EMPTY: u32 = 0;
const MARKINGS: u32 = 11 * 16;
const FULL: u32 = 13 * 16 + 11;

pub fn plugin(app: &mut App) {
    app.add_plugins(TilemapPlugin);
}

pub fn spawn_field(glyph: Res<GlyphAsset>, mut commands: Commands) {
    let height = field::FIELD.lines().count();
    let width = field::FIELD.lines().nth(0).unwrap().chars().count();
    let map_size = TilemapSize {
        x: width as u32,
        y: height as u32,
    };

    let tilemap_entity = commands.spawn(Name::from("Tilemap")).id();
    let mut tile_storage = TileStorage::empty(map_size);

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let line = field::FIELD.lines().nth(y as usize).unwrap();
            let char = line.chars().nth(x as usize).unwrap();
            let index = match char {
                '#' => TileTextureIndex(FULL),
                '.' => TileTextureIndex(MARKINGS),
                _ => TileTextureIndex(EMPTY),
            };
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: index,
                    ..default()
                })
                .set_parent(tilemap_entity)
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 8.0, y: 8.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(glyph.glyph.clone_weak()),
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..default()
    });
}
