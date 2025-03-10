use bevy::prelude::*;

mod glyph_asset;
mod panel_border_asset;
mod resource_handles;

pub use glyph_asset::GlyphAsset;
pub use panel_border_asset::PanelBorderAsset;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
pub use resource_handles::ResourceHandles;

use super::{CharacterClasses, FIRST_NAMES, LAST_NAMES, Stats, Teams};

#[derive(Resource)]
pub struct Names {
    sampler: ChaCha8Rng,
}

impl Names {
    pub fn new(seed: Option<u64>) -> Self {
        let sampler = match seed {
            Some(number) => ChaCha8Rng::seed_from_u64(number),
            None => ChaCha8Rng::from_os_rng(),
        };
        Names { sampler }
    }

    pub fn random(&mut self) -> String {
        let first_index = self.sampler.random_range(0..FIRST_NAMES.len());
        let last_index = self.sampler.random_range(0..LAST_NAMES.len());
        format!("{} {}", FIRST_NAMES[first_index], LAST_NAMES[last_index])
    }
}

#[derive(Resource)]
pub struct StatsFactory {
    sampler: ChaCha8Rng,
}

impl StatsFactory {
    pub fn new(seed: Option<u64>) -> Self {
        let sampler = match seed {
            Some(number) => ChaCha8Rng::seed_from_u64(number),
            None => ChaCha8Rng::from_os_rng(),
        };
        StatsFactory { sampler }
    }

    pub fn create_from_class(&mut self, class: &CharacterClasses) -> Stats {
        Stats {
            ap: 10,
            intial_ap: 10,
            kick_strength: 15.0,
            passing_skill: 50.0,
            wit: 0.5,
            defense: 0.5,
            initiative: 0,
        }
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct CurrentTeam(pub Teams);

impl CurrentTeam {
    pub fn next(&self) -> Teams {
        match self.0 {
            Teams::Player => Teams::Enemy,
            Teams::Enemy => Teams::Player,
        }
    }
}
