use bevy::prelude::*;

mod components;
mod consts;
mod resources;
mod systems;
mod traits;

pub use components::*;
pub use consts::*;
pub use resources::*;
pub use traits::*;

pub fn plugin(app: &mut App) {
    app.register_type::<ImageNodeFadeInOut>()
        .init_resource::<ResourceHandles>()
        .load_resource::<GlyphAsset>()
        .add_systems(PreUpdate, systems::load_resource_assets);
}
