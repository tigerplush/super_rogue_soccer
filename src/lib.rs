use asset_tracking::LoadResource;
use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

mod actors;
mod asset_tracking;
mod entities;
mod map;
mod states;
mod theme;
mod ui;

#[cfg(feature = "debug")]
mod debug;

pub struct SuperRogueSoccerPlugin;

impl Plugin for SuperRogueSoccerPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (AppSet::TickTimers, AppSet::RecordInput, AppSet::Update).chain(),
        )
        .configure_sets(
            PostUpdate,
            (
                PostUpdateSet::Calculate,
                PostUpdateSet::Move,
                PostUpdateSet::Ui,
            )
                .chain(),
        )
        .add_plugins((
            asset_tracking::plugin,
            actors::plugin,
            states::plugin,
            map::plugin,
            entities::plugin,
            ui::plugin,
            #[cfg(feature = "debug")]
            debug::plugin,
        ));
        app.load_resource::<GlyphAsset>();
        app.load_resource::<PanelBorderAsset>();
        app.load_resource::<FontAsset>();
        app.add_systems(Startup, startup);
    }
}

fn startup(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(172.0, -55.0, 0.0)));
}

#[derive(Resource, Asset, TypePath, Clone)]
pub struct GlyphAsset {
    pub glyph: Handle<Image>,
    pub atlas: Handle<TextureAtlasLayout>,
}

impl GlyphAsset {
    const PATH: &'static str = "terminal8x8.png";
}

impl FromWorld for GlyphAsset {
    fn from_world(world: &mut World) -> Self {
        let layout_handle = {
            let layout = TextureAtlasLayout::from_grid(UVec2::splat(8), 16, 16, None, None);
            let mut layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
            layouts.add(layout)
        };
        let assets = world.resource::<AssetServer>();
        GlyphAsset {
            glyph: assets.load_with_settings(
                GlyphAsset::PATH,
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve the pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            atlas: layout_handle,
        }
    }
}

#[derive(Resource, Asset, TypePath, Clone)]
pub struct PanelBorderAsset {
    pub image: Handle<Image>,
    pub slicer: TextureSlicer,
}

impl PanelBorderAsset {
    const PATH: &'static str = "panel_border.png";
}

impl FromWorld for PanelBorderAsset {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        PanelBorderAsset {
            image: assets.load_with_settings(
                PanelBorderAsset::PATH,
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve the pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            slicer: TextureSlicer {
                border: BorderRect::square(8.0),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 1.0,
            },
        }
    }
}

#[derive(Resource, Asset, TypePath, Clone, Deref, DerefMut)]
pub struct FontAsset {
    pub font: Handle<Font>,
}

impl FontAsset {
    const PATH: &'static str = "PixelifySans-VariableFont_wght.ttf";
}

impl FromWorld for FontAsset {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        FontAsset {
            font: assets.load(FontAsset::PATH),
        }
    }
}

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSet {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum PostUpdateSet {
    Calculate,
    Move,
    Ui,
}

fn to_ivec2(from: Vec3) -> IVec2 {
    IVec2 {
        x: (from.x / 8.0).floor() as i32,
        y: (from.y / 8.0).floor() as i32,
    }
}

fn to_world(from: IVec2) -> Vec2 {
    Vec2 {
        x: from.x as f32 * 8.0,
        y: from.y as f32 * 8.0,
    }
}
