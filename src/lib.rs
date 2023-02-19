use animation::{animation_update_internal, animate_sprite};
use asset_loader::{SpriteSheetAnimationLoader, SpriteSheetAnimationSet};
use bevy::prelude::{AddAsset, App, Plugin, IntoSystemDescriptor};

pub mod asset_loader;
pub mod animation;

/// Adds support for spritesheet animation manifest files loading to the app.
pub struct SpriteSheetAnimationLoaderPlugin;

impl Plugin for SpriteSheetAnimationLoaderPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_asset::<SpriteSheetAnimationSet>()
        .init_asset_loader::<SpriteSheetAnimationLoader>()
        .add_system(animation_update_internal)
        .add_system(animate_sprite.after(animation_update_internal));
    }
}
