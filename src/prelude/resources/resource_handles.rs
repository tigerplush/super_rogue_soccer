use std::collections::VecDeque;

use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct ResourceHandles {
    // Use a queue for waiting assets so they can be cycled through and moved to
    // `finished` one at a time.
    pub waiting: VecDeque<(UntypedHandle, fn(&mut World, &UntypedHandle))>,
    pub(crate) finished: Vec<UntypedHandle>,
}

impl ResourceHandles {
    /// Returns true if all requested [`Asset`]s have finished loading and are available as [`Resource`]s.
    pub fn is_all_done(&self) -> bool {
        self.waiting.is_empty()
    }
}
