use bevy::prelude::*;

mod components;
mod consts;
mod resources;
mod systems;
mod traits;

use bevy_ecs_tilemap::TilemapPlugin;
pub use components::*;
pub use consts::*;
pub use resources::*;
pub use systems::*;
pub use traits::*;

pub fn plugin(app: &mut App) {
    app.register_type::<ImageNodeFadeInOut>()
        .init_resource::<ResourceHandles>()
        .insert_resource(Names::new(None))
        .insert_resource(StatsFactory::new(None))
        .load_resource::<GlyphAsset>()
        .load_resource::<PanelBorderAsset>()
        .add_plugins(TilemapPlugin)
        .add_systems(PreUpdate, systems::load_resource_assets);
}
