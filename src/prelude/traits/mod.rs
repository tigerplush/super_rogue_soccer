use bevy::prelude::*;

mod containers;
mod load_resource;
mod spawn;
mod widgets;

use bevy::math::IVec2;
pub use containers::Containers;
pub use load_resource::LoadResource;
pub use widgets::*;

pub trait ToIvec2 {
    fn as_ivec2(&self) -> IVec2;
}

impl ToIvec2 for Transform {
    fn as_ivec2(&self) -> IVec2 {
        (self.translation.truncate() / 8.0).round().as_ivec2()
    }
}
